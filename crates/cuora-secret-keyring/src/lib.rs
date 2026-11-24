//! cuora-secret-keyring — OS keyring impl of [`cuora_secret::Secrets`]
//! via `secret-tool` (libsecret). Headless boxes without dbus simply
//! miss every lookup and the caller falls through to the next store.
//!
//! Absorbed the forked `*-keyring` crates (hypha/sturn/kuda/mimus) —
//! the `service` attribute is now explicit per consumer instead of
//! each fork's hardcoded default.

use cuora_secret::Secrets;
use std::io::Write;
use std::process::{Command, Stdio};

/// `secret-tool` (libsecret) backend — headless boxes without dbus
/// simply miss every lookup and the caller falls through to the file
/// store.
pub struct KeyringSecrets {
    /// secret-tool `service` attribute — groups one product's
    /// entries (e.g. `"mimus"`, `"bupha"`). There is deliberately no
    /// `Default`: the service names the on-disk namespace, so each
    /// consumer passes its own.
    pub service: String,
}

impl KeyringSecrets {
    /// A store under `service`.
    #[must_use]
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }
}

/// A usable secret name — non-empty and not `-`-leading (a leading
/// dash would parse as a `secret-tool` flag, not an attribute value).
fn valid_name(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('-')
}

impl Secrets for KeyringSecrets {
    fn name(&self) -> &str {
        "keyring"
    }

    fn available(&self) -> bool {
        Command::new("secret-tool")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
    }

    fn get(&self, name: &str) -> Option<String> {
        if !valid_name(name) {
            return None;
        }
        let o = Command::new("secret-tool")
            .args(["lookup", "service", &self.service, "name", name])
            .output()
            .ok()?;
        o.status
            .success()
            .then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|v| !v.is_empty())
    }

    fn set(&self, name: &str, value: &str) -> std::io::Result<()> {
        if !valid_name(name) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "secret name must be non-empty and not start with '-'",
            ));
        }
        let mut c = Command::new("secret-tool")
            .args([
                "store",
                "--label",
                &format!("{}: {name}", self.service),
                "service",
                &self.service,
                "name",
                name,
            ])
            .stdin(Stdio::piped())
            .spawn()?;
        c.stdin
            .as_mut()
            .ok_or_else(|| std::io::Error::other("secret-tool stdin"))?
            .write_all(value.as_bytes())?;
        // a failed wait() is a failed store — folding it in keeps a
        // `?` arm nothing can ever reach off the error path
        if c.wait().map(|s| s.success()).unwrap_or(false) {
            Ok(())
        } else {
            Err(std::io::Error::other("secret-tool store failed"))
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn invalid_names_never_reach_secret_tool() {
        let s = KeyringSecrets::new("cuora-test");
        // Empty / dash-leading names are rejected before spawn —
        // `get` misses, `set` errors, on any host.
        assert_eq!(s.get(""), None);
        assert_eq!(s.get("-evil"), None);
        assert!(s.set("", "v").is_err());
        assert!(s.set("-evil", "v").is_err());
        assert_eq!(s.name(), "keyring");
    }

    /// Full conformance needs a live Secret Service — skipped
    /// silently on headless boxes. `available()` only proves the
    /// binary is on PATH; a box can have `secret-tool` with no
    /// reachable service (no session bus), where every write fails.
    /// Probe with a real store+clear before running the suite.
    #[test]
    fn conformance_when_keyring_available() {
        let s = KeyringSecrets::new("cuora-secret-keyring-test");
        let probe = "cuora-secret-keyring-probe";
        if s.set(probe, "probe").is_err() {
            return;
        }
        let _ = Command::new("secret-tool")
            .args(["clear", "service", &s.service, "name", probe])
            .status();
        cuora_testkit::secrets::conformance(&s);
    }
}
