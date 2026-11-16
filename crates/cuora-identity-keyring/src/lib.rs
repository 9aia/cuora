//! cuora-identity-keyring — the shared age identity in the
//! platform's session secret store.
//!
//! The org has ONE age identity per human/machine, owned by the
//! `tardigrada` tool; cuora resolves that same identity. The
//! `AGE-SECRET-KEY-…` string lives as a keyed secret in the
//! user-session credential store — it unlocks with the login
//! session, so reads need no prompt:
//!
//! - Linux/*nix — org.freedesktop.secrets (secret-service: GNOME
//!   Keyring, KeePassXC, KDE Wallet's wrapper) via the synchronous
//!   libdbus client — no async runtime.
//! - macOS — the login keychain.
//! - Windows — Credential Manager (DPAPI).
//!
//! [`KeyringIdentity::platform`] is the availability probe: it
//! fails with [`IdentityError::Unavailable`] on a headless box
//! (no session bus, no daemon) and callers fall back to
//! `cuora-identity-file` — never a crash. Tests drive any
//! `keyring-core` store through [`KeyringIdentity::with_store`]
//! (the mock store needs no daemon).
//!
//! Store calls run under a bounded wait (`bounded`): a locked
//! collection's unlock prompt has no client-side deadline on the
//! dbus store, so an unanswered prompt would hang the caller
//! forever. `CUORA_KEYRING_TIMEOUT_MS` tunes the bound (`0`
//! waits forever).

use std::str::FromStr;
use std::sync::Arc;

use age::secrecy::ExposeSecret;
use age::x25519;
use cuora_identity::{IdentityError, IdentitySource};
use keyring_core::{CredentialStore, Entry};

/// The canonical secret-store service name — the shared
/// `tardigrada` identity entry cuora reads and writes.
pub const CANONICAL_SERVICE: &str = "tardigrada";
/// The entry/user name the age identity is stored under.
pub const USER: &str = "age-identity";
/// Legacy service names `load` falls back to, in order — identities
/// stored by `cuora`/`arca` keep resolving; `store` always writes
/// [`CANONICAL_SERVICE`] only.
pub const LEGACY_SERVICES: &[&str] = &["cuora", "arca"];

/// Upper bound on one blocking credential-store call. The dbus
/// secret-service store connects via `SecretService::connect`,
/// which waits *indefinitely* for a user to answer an unlock
/// prompt — after a keyring-daemon restart left the login
/// collection locked, every `cuora` verb sat on the prompt and
/// looked hung while long-lived `cuora serve` legs (identity
/// already in memory) kept working. On timeout the op fails with
/// a typed error naming the escape hatches; the still-blocked
/// worker thread is left to die with the process (CLI verbs are
/// one-shot; `serve` just sees an op error per request).
const OP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

/// The effective op bound — `CUORA_KEYRING_TIMEOUT_MS` overrides
/// [`OP_TIMEOUT`]; `0` disables the bound entirely (the old
/// block-forever behavior, for a human willing to wait out a
/// prompt). An unparseable value falls back to the default.
fn op_timeout() -> Option<std::time::Duration> {
    match std::env::var("CUORA_KEYRING_TIMEOUT_MS") {
        Ok(v) => match v.parse::<u64>() {
            Ok(0) => None,
            Ok(ms) => Some(std::time::Duration::from_millis(ms)),
            Err(_) => Some(OP_TIMEOUT),
        },
        Err(_) => Some(OP_TIMEOUT),
    }
}

/// Run `f` on a worker thread and bound the wait — credential
/// stores are synchronous and a locked collection's unlock prompt
/// has no client-side deadline, so the bound lives here, above
/// every backend (dbus prompt, Keychain dialog, Credential
/// Manager). `what` names the op in the timeout error.
fn bounded<T, F>(what: &str, f: F) -> Result<T, IdentityError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, IdentityError> + Send + 'static,
{
    let Some(timeout) = op_timeout() else {
        return f();
    };
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name(format!("cuora-keyring-{what}"))
        .spawn(move || {
            // A timed-out caller is gone; a late result has
            // nowhere to go — dropping the send is fine.
            let _ = tx.send(f());
        })
        .map_err(|e| IdentityError::Store(format!("keyring {what} thread: {e}")))?;
    match rx.recv_timeout(timeout) {
        Ok(res) => res,
        Err(_) => Err(IdentityError::Store(format!(
            "keyring {what} timed out after {}s — the session store is \
             waiting on an unlock prompt; unlock it, or configure your \
             tool to use its identity-file backend instead \
             (CUORA_KEYRING_TIMEOUT_MS adjusts the bound, 0 waits \
             forever)",
            timeout.as_secs()
        ))),
    }
}

/// The `<service>`/`age-identity` entry in `store` — the
/// store-level twin of [`KeyringIdentity::entry_as`], for calls
/// that run on the bounded worker thread.
fn entry_in(store: &CredentialStore, service: &str) -> Result<Entry, IdentityError> {
    store
        .build(service, USER, None)
        .map_err(|e| IdentityError::Store(format!("keyring entry: {e}")))
}

/// The `load` body, run inside [`bounded`] — `store` is the
/// `Arc`'d credential store moved onto the worker thread. Reads
/// probe [`CANONICAL_SERVICE`] first, then each of
/// [`LEGACY_SERVICES`] in order — the first entry that exists
/// wins, so a tardigrada-canonical identity shadows stale
/// cuora/arca copies.
fn load_from(store: &CredentialStore) -> Result<Option<x25519::Identity>, IdentityError> {
    for service in std::iter::once(CANONICAL_SERVICE).chain(LEGACY_SERVICES.iter().copied()) {
        let entry = entry_in(store, service)?;
        let text = match entry.get_password() {
            Ok(t) => t,
            Err(keyring_core::Error::NoEntry) => continue,
            Err(e) => return Err(IdentityError::Store(format!("keyring get: {e}"))),
        };
        // get_password returns a plain String holding raw key
        // material — zeroize on drop (same hygiene as the
        // keyfile body in cuora-identity-file).
        let text = zeroize::Zeroizing::new(text);
        let line = text.trim();
        // Deliberate: a present-but-corrupt entry is a hard error,
        // never a skip to the next service — silently falling back
        // to an older identity could decrypt with the WRONG key and
        // look like data loss. Surface the corruption.
        return x25519::Identity::from_str(line)
            .map(Some)
            .map_err(|e| IdentityError::Parse(format!("keyring identity {service}/{USER}: {e}")));
    }
    Ok(None)
}

/// An age identity held in a `keyring-core` credential store.
pub struct KeyringIdentity {
    store: Arc<CredentialStore>,
    /// Whether `store()` must claim the `default` collection alias
    /// first — true only for the platform secret-service store
    /// (the I-006 fixup). `with_store` callers own their store's
    /// semantics, so a test/embedded store never triggers a real
    /// dbus mutation.
    ensure_collection: bool,
}

/// Does the platform store need the `default`-alias ensure before
/// writes? Only the unix secret-service backend — Keychain and
/// Credential Manager have no collection-alias concept.
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android")),
    not(coverage)
))]
const PLATFORM_NEEDS_DEFAULT_COLLECTION: bool = true;

#[cfg(not(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android")),
    not(coverage)
)))]
const PLATFORM_NEEDS_DEFAULT_COLLECTION: bool = false;

impl KeyringIdentity {
    /// The platform session store — the availability probe.
    ///
    /// Read-only: probing (and every `load`) never creates
    /// collections or claims aliases — `tardigrada show`-style
    /// consumers that only *load* the identity must not mutate the
    /// keyring. The default-collection fixup runs lazily inside
    /// `store`, the only mutating op.
    ///
    /// `Err(Unavailable)` means no session secret store is
    /// reachable right now (headless, no dbus session, no
    /// daemon); `Ok` means ops are possible but can still fail
    /// per-call (daemon dies, collection locked).
    pub fn platform() -> Result<Self, IdentityError> {
        Ok(Self {
            store: platform_store()?,
            ensure_collection: PLATFORM_NEEDS_DEFAULT_COLLECTION,
        })
    }

    /// Any `keyring-core` credential store — tests (the mock),
    /// alternative backends, embeddings.
    pub fn with_store(store: Arc<CredentialStore>) -> Self {
        KeyringIdentity {
            store,
            ensure_collection: false,
        }
    }

    /// The `tardigrada`/`age-identity` entry in this store — test
    /// surface only; ops go through [`entry_in`] on the bounded
    /// worker thread.
    #[cfg(test)]
    fn entry(&self) -> Result<Entry, IdentityError> {
        self.store
            .build(CANONICAL_SERVICE, USER, None)
            .map_err(|e| IdentityError::Store(format!("keyring entry: {e}")))
    }
}

impl IdentitySource for KeyringIdentity {
    /// `Ok(None)` when no identity entry exists in the store;
    /// store failures and unparseable material are `Err`.
    fn load(&self) -> Result<Option<x25519::Identity>, IdentityError> {
        let store = Arc::clone(&self.store);
        bounded("load", move || load_from(&*store))
    }

    /// Write the identity into the store (upsert — the platform
    /// store owns unlock/prompt policy, bounded by `bounded`).
    fn store(&self, identity: &x25519::Identity) -> Result<(), IdentityError> {
        let store = Arc::clone(&self.store);
        let ensure = self.ensure_collection;
        let key = identity.to_string();
        bounded("store", move || {
            // The only mutation point — claim the `default`
            // collection alias here, never on the load/probe
            // path (a read-only consumer must not create a
            // `tardigrada` collection).
            if ensure {
                ensure_default_collection();
            }
            entry_in(&*store, CANONICAL_SERVICE)?
                .set_password(key.expose_secret())
                .map_err(|e| IdentityError::Store(format!("keyring set: {e}")))
        })
    }

    fn kind(&self) -> &'static str {
        "keyring"
    }

    fn describe(&self) -> String {
        format!("system keyring (entry {CANONICAL_SERVICE}/{USER})")
    }
}

/// Build this platform's native credential store. Store
/// construction *is* the reachability probe — the dbus store
/// connects to the session bus here.
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android")),
    not(coverage)
))]
fn platform_store() -> Result<Arc<CredentialStore>, IdentityError> {
    let store = dbus_secret_service_keyring_store::Store::new()
        .map_err(|e| IdentityError::Unavailable(format!("secret-service: {e}")))?;
    // Probe stays read-only — the default-collection fixup runs
    // inside `store` (see KeyringIdentity::ensure_collection).
    Ok(store as Arc<CredentialStore>)
}

/// Claim the `default` collection alias when the service has none.
///
/// Called only from the `store` path on platform-built identities —
/// writes are the sole mutation this crate performs on the session
/// store; `load`/probe stay read-only.
///
/// `dbus-secret-service-keyring-store`'s create-item path resolves
/// the literal `default` alias; on a fresh Secret Service (daemon
/// just spawned, no keyring unlocked yet) `ReadAlias` returns `/`
/// → `NoResult`, surfaced as `NoStorageAccess` — the first
/// `set_password` fails even though the service is reachable.
/// gnome-keyring only honors the `default` alias (others are
/// `NotSupported`), so the fix is always claiming *that* alias:
/// re-alias an existing collection (the login keyring keeps the
/// secret inside the user's real keyring) or create one when the
/// service is empty. Best-effort: any failure here surfaces later
/// as the store's own error — this never makes things worse.
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android")),
    not(coverage)
))]
fn ensure_default_collection() {
    use dbus_secret_service::{EncryptionType, SecretService};
    // Prompt timeout 0 — a create that would block on a GUI
    // prompter cancels instead; ops stay non-interactive.
    let Ok(ss) = SecretService::connect_with_max_prompt_timeout(EncryptionType::Plain, 0) else {
        return;
    };
    if ss.get_default_collection().is_ok() {
        return;
    }
    // Collections exist but none owns `default` — claim the alias
    // for the login keyring (else the first) via SetAlias, which
    // dbus-secret-service doesn't expose publicly.
    if let Ok(cols) = ss.get_all_collections() {
        let pick = cols
            .iter()
            .find(|c| c.get_label().ok().as_deref() == Some("login"))
            .or_else(|| cols.first());
        if let Some(col) = pick {
            if set_default_alias(&col.path) && ss.get_default_collection().is_ok() {
                return;
            }
        }
    }
    // No usable collection — create one, then claim `default`
    // explicitly: CreateCollection's alias arg is only a hint.
    if ss.create_collection("tardigrada", "default").is_ok() {
        if let Ok(cols) = ss.get_all_collections() {
            if let Some(col) = cols
                .iter()
                .find(|c| c.get_label().ok().as_deref() == Some("tardigrada"))
            {
                set_default_alias(&col.path);
            }
        }
    }
}

/// `org.freedesktop.Secret.Service.SetAlias` — gnome-keyring
/// implements it but dbus-secret-service keeps the proxy private,
/// so this is one raw call on a short timeout. `true` = the method
/// returned without a dbus error.
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android")),
    not(coverage)
))]
fn set_default_alias(collection: &dbus_secret_service::Path) -> bool {
    let Ok(conn) = dbus::blocking::Connection::new_session() else {
        return false;
    };
    let proxy = conn.with_proxy(
        "org.freedesktop.secrets",
        "/org/freedesktop/secrets",
        std::time::Duration::from_secs(5),
    );
    proxy
        .method_call::<(), _, _, _>(
            "org.freedesktop.Secret.Service",
            "SetAlias",
            ("default", collection.clone()),
        )
        .is_ok()
}

/// No-op twin of [`ensure_default_collection`] for platforms
/// without the secret-service alias problem (Keychain, Credential
/// Manager, coverage/headless) — `store` calls it unconditionally
/// when the platform flag is set, and the flag is only set where
/// the real fixup exists.
#[cfg(not(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android")),
    not(coverage)
)))]
fn ensure_default_collection() {}

/// The coverage build's probe — coverage/CI envs are headless, so
/// the dbus store can never connect; the Ok-arm expression would
/// be a phantom miss. Same typed outcome, deterministic.
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android")),
    coverage
))]
fn platform_store() -> Result<Arc<CredentialStore>, IdentityError> {
    Err(IdentityError::Unavailable(
        "secret-service: coverage env".into(),
    ))
}

#[cfg(target_os = "macos")]
fn platform_store() -> Result<Arc<CredentialStore>, IdentityError> {
    apple_native_keyring_store::keychain::Store::new()
        .map(|s| s as Arc<CredentialStore>)
        .map_err(|e| IdentityError::Unavailable(format!("macos keychain: {e}")))
}

#[cfg(target_os = "windows")]
fn platform_store() -> Result<Arc<CredentialStore>, IdentityError> {
    windows_native_keyring_store::Store::new()
        .map(|s| s as Arc<CredentialStore>)
        .map_err(|e| IdentityError::Unavailable(format!("credential manager: {e}")))
}

#[cfg(not(any(
    all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ),
    target_os = "macos",
    target_os = "windows"
)))]
fn platform_store() -> Result<Arc<CredentialStore>, IdentityError> {
    Err(IdentityError::Unavailable(
        "no keyring backend on this platform".into(),
    ))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// A `KeyringIdentity` over keyring-core's in-memory mock
    /// store — no daemon needed, real `CredentialStoreApi` path.
    fn mock() -> (KeyringIdentity, Arc<keyring_core::mock::Store>) {
        let store = keyring_core::mock::Store::new().unwrap();
        (
            KeyringIdentity::with_store(store.clone() as Arc<CredentialStore>),
            store,
        )
    }

    #[test]
    fn store_then_load_roundtrips() {
        let (k, _) = mock();
        assert!(k.load().unwrap().is_none());
        let (id, created) = cuora_identity::ensure(&k).unwrap();
        assert!(created);
        let (again, created) = cuora_identity::ensure(&k).unwrap();
        assert!(!created);
        assert_eq!(id.to_public(), again.to_public());
        assert_eq!(k.kind(), "keyring");
        assert!(k.describe().contains("tardigrada"));
    }

    /// Write `key` into the mock store under `service`/`USER` —
    /// seeds legacy entries the canonical `store` never writes.
    fn seed(store: &Arc<keyring_core::mock::Store>, service: &str, key: &str) {
        use keyring_core::api::CredentialStoreApi;
        store
            .build(service, USER, None)
            .unwrap()
            .set_password(key)
            .unwrap();
    }

    /// Read the mock store's `service`/`USER` entry — `None` when
    /// absent.
    fn stored(store: &Arc<keyring_core::mock::Store>, service: &str) -> Option<String> {
        use keyring_core::api::CredentialStoreApi;
        store
            .build(service, USER, None)
            .unwrap()
            .get_password()
            .ok()
    }

    #[test]
    fn store_writes_only_the_canonical_service() {
        let (k, store) = mock();
        let id = x25519::Identity::generate();
        k.store(&id).unwrap();
        use age::secrecy::ExposeSecret;
        let want = id.to_string().expose_secret().to_string();
        assert_eq!(
            stored(&store, CANONICAL_SERVICE).as_deref(),
            Some(want.as_str())
        );
        for legacy in LEGACY_SERVICES {
            assert_eq!(stored(&store, legacy), None, "{legacy} must stay empty");
        }
    }

    #[test]
    fn load_falls_back_canonical_then_cuora_then_arca() {
        use age::secrecy::ExposeSecret;
        let (k, store) = mock();
        let arca_id = x25519::Identity::generate();
        let cuora_id = x25519::Identity::generate();
        let tardigrada_id = x25519::Identity::generate();
        // arca alone still resolves — the oldest legacy entry.
        seed(&store, "arca", arca_id.to_string().expose_secret());
        assert_eq!(k.load().unwrap().unwrap().to_public(), arca_id.to_public());
        // cuora shadows arca — legacy order is preserved.
        seed(&store, "cuora", cuora_id.to_string().expose_secret());
        assert_eq!(k.load().unwrap().unwrap().to_public(), cuora_id.to_public());
        // the canonical tardigrada entry wins outright.
        seed(
            &store,
            CANONICAL_SERVICE,
            tardigrada_id.to_string().expose_secret(),
        );
        assert_eq!(
            k.load().unwrap().unwrap().to_public(),
            tardigrada_id.to_public()
        );
    }

    #[test]
    fn ensure_migrates_a_legacy_entry_into_tardigrada() {
        // A cuora-only identity is found by `load`, so `ensure`
        // reports it as existing — never clobbered, never
        // duplicated under the canonical name until `store` runs.
        use age::secrecy::ExposeSecret;
        let (k, store) = mock();
        let id = x25519::Identity::generate();
        seed(&store, "cuora", id.to_string().expose_secret());
        let (loaded, created) = cuora_identity::ensure(&k).unwrap();
        assert!(!created);
        assert_eq!(loaded.to_public(), id.to_public());
        // an explicit re-store lands under tardigrada only
        k.store(&loaded).unwrap();
        assert!(stored(&store, CANONICAL_SERVICE).is_some());
        assert!(stored(&store, "cuora").is_some());
        assert_eq!(stored(&store, "arca"), None);
    }

    #[test]
    fn load_rejects_garbage_material() {
        let (k, _) = mock();
        let entry = k.entry().unwrap();
        entry.set_password("not-an-age-key").unwrap();
        assert!(matches!(k.load(), Err(IdentityError::Parse(_))));
    }

    #[test]
    fn corrupt_canonical_entry_never_falls_back_to_legacy() {
        // Deliberate (see load_from): a corrupt tardigrada entry is
        // a hard Parse error even when a valid legacy `cuora`
        // identity sits behind it — falling back could decrypt
        // with the wrong key and look like data loss.
        use age::secrecy::ExposeSecret;
        let (k, store) = mock();
        let legacy_id = x25519::Identity::generate();
        seed(&store, "cuora", legacy_id.to_string().expose_secret());
        let entry = k.entry().unwrap();
        entry.set_password("not-an-age-key").unwrap();
        assert!(matches!(k.load(), Err(IdentityError::Parse(_))));
    }

    #[test]
    fn probe_and_load_never_claim_the_default_collection() {
        // The I-006 default-alias fixup is write-path only: a
        // with_store identity (tests, embeddings, tardigrada-style
        // read-only consumers) must carry ensure_collection=false
        // so load/store never run the dbus alias claim — only
        // platform()-built identities on the secret-service
        // backend set it.
        let (k, _) = mock();
        assert!(!k.ensure_collection);
        k.load().unwrap();
        let id = x25519::Identity::generate();
        k.store(&id).unwrap();
        // A real platform probe sets the flag exactly on the
        // secret-service backend (headless CI → Err, either way
        // the contract holds: Ok ⇒ flag matches the cfg).
        if let Ok(k) = KeyringIdentity::platform() {
            assert_eq!(k.ensure_collection, PLATFORM_NEEDS_DEFAULT_COLLECTION);
        }
    }

    #[test]
    fn store_failures_are_store_errors_not_none() {
        let (k, _) = mock();
        // arm the mock's next call to fail — get must surface the
        // error, never masquerade as "no entry"
        let entry = k.entry().unwrap();
        let cred: &keyring_core::mock::Cred = entry.as_any().downcast_ref().unwrap();
        cred.set_error(keyring_core::Error::PlatformFailure("daemon dead".into()));
        assert!(matches!(k.load(), Err(IdentityError::Store(_))));
        // and `set` failing maps the same way — the `?` inside
        // store surfaces a typed error, not a crash
        cred.set_error(keyring_core::Error::PlatformFailure("daemon dead".into()));
        let id = x25519::Identity::generate();
        assert!(matches!(k.store(&id), Err(IdentityError::Store(_))));
    }

    #[test]
    fn platform_probe_never_panics() {
        // On CI (no session bus) this is Unavailable; on a desktop
        // it may succeed. The contract is "a typed answer, not a
        // crash" — just probing exercises it unconditionally (the
        // Unavailable-vs-Ok branch is environment-dependent).
        let _ = KeyringIdentity::platform();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod store_error_tests {
    use super::*;
    use keyring_core::{Entry, Error};
    use std::any::Any;
    use std::collections::HashMap;

    /// A store whose `build` always fails — no daemon, no entry.
    struct DeadStore;

    impl keyring_core::api::CredentialStoreApi for DeadStore {
        fn vendor(&self) -> String {
            "dead".into()
        }
        fn id(&self) -> String {
            "dead".into()
        }
        fn build(&self, _: &str, _: &str, _: Option<&HashMap<&str, &str>>) -> Result<Entry, Error> {
            Err(Error::PlatformFailure("dead bus".into()))
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn entry_build_failures_surface_as_store_errors() {
        // entry's map_err arm — a typed error, never a panic
        let store = Arc::new(DeadStore);
        let k = KeyringIdentity::with_store(store.clone() as Arc<CredentialStore>);
        assert!(matches!(k.load(), Err(IdentityError::Store(_))));
        // the trait surface DeadStore implements, exercised
        assert_eq!(
            keyring_core::api::CredentialStoreApi::vendor(store.as_ref()),
            "dead"
        );
        assert_eq!(
            keyring_core::api::CredentialStoreApi::id(store.as_ref()),
            "dead"
        );
        let _ = keyring_core::api::CredentialStoreApi::as_any(store.as_ref());
    }

    /// A store whose `build` never answers — the shape of a
    /// secret-service unlock prompt: reachable bus, no deadline.
    struct StuckStore;

    impl keyring_core::api::CredentialStoreApi for StuckStore {
        fn vendor(&self) -> String {
            "stuck".into()
        }
        fn id(&self) -> String {
            "stuck".into()
        }
        fn build(&self, _: &str, _: &str, _: Option<&HashMap<&str, &str>>) -> Result<Entry, Error> {
            std::thread::sleep(std::time::Duration::from_secs(30));
            Err(Error::PlatformFailure("unreachable".into()))
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn a_blocked_store_call_times_out_instead_of_hanging() {
        // CUORA_KEYRING_TIMEOUT_MS shrinks the bound for the test;
        // sibling tests in this binary only drive the in-memory
        // mock (microseconds), so the tiny window can't flake them.
        std::env::set_var("CUORA_KEYRING_TIMEOUT_MS", "200");
        let k = KeyringIdentity::with_store(Arc::new(StuckStore) as Arc<CredentialStore>);
        let e = k.load().err().unwrap().to_string();
        std::env::remove_var("CUORA_KEYRING_TIMEOUT_MS");
        assert!(e.contains("timed out"), "{e}");
        // The timeout hint must stay backend-agnostic — this crate
        // is shared (tardigrada consumes it too), so it names only
        // the shared CUORA_KEYRING_TIMEOUT_MS knob, never cuora's
        // own CUORA_IDENTITY_BACKEND.
        assert!(e.contains("CUORA_KEYRING_TIMEOUT_MS"), "{e}");
        assert!(!e.contains("CUORA_IDENTITY_BACKEND"), "{e}");
    }
}
