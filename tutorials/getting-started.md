# Getting started

Install the `cuora` CLI, seal your first secret in the local
age-encrypted vault, and run a command that uses it — without the
value ever touching your terminal, a file, or an agent's context.

## Install

The `cuora` command ships as a tamias pack (`libraries/cuora`) — a
generated `bins[]` shim over `tamias-driver-cuora`, not a
cargo-installed binary. With `tamias` installed:

```sh
tamias install 9aia:cuora@<version>    # from the org registry
                                     # (private registry: set TAMIAS_REGISTRY_TOKEN)
```

From a checkout of this repo:

```sh
cargo xtask pack                            # stage the driver payload
tamias install --allow-drivers ./libraries/cuora   # deploys `cuora`
```

`cargo xtask install` additionally puts `tamias-driver-cuora`
itself on `PATH` (`~/.cargo/bin`).

## Bootstrap

```sh
cuora init
```

`init` initializes your **local age identity** — the per-machine,
per-human key every vault decrypts with. It is *not* repo setup:
it stores the identity where it's safest available (the session
keyring on a desktop, the `0600` keyfile on headless boxes) and is
idempotent — run it any time. To verify vault access and the
declared surface, that's `cuora check`'s job (below).

On a desktop session (GNOME Keyring, KeePassXC, KDE Wallet, macOS
Keychain, Windows Credential Manager) the identity lands in the
session keyring — it unlocks with login, no prompt. The default
backend is `keyring`: **fail closed** — if no session store is
reachable, `init` (and every other verb) errors rather than
silently downgrading to a file. Headless boxes (servers, CI,
containers, plain SSH) have no session bus, so there you opt out
explicitly — `[identity] backend = "auto"` in `cuora.toml` (probe
the keyring, fall back to the keyfile with a note), `"file"`
(never probe), or the `CUORA_IDENTITY_BACKEND` env var. The
fallback writes `~/.config/cuora/identity.txt` (mode `0600`) —
raw age key material, protect it like a private SSH key.
`cuora init --with-file` forces the keyfile; pinning
`CUORA_IDENTITY_FILE` or `[vaults.*].identity` in config does the
same — that's the deliberate headless/CI path.

Already have an `identity.txt`? `cuora init` imports it into the
keyring rather than orphaning it, and leaves the file as a backup
you can delete when ready. (On a brand-new keyring with no
collections yet, cuora claims the `default` collection alias
automatically — no manual step.)

The `cuora identity` group manages that identity after bootstrap:

```sh
cuora identity show      # the age1… recipient — the value teammates
                         # paste into [vaults.*].recipients
cuora identity import ~/old/identity.txt   # adopt an existing keyfile
cuora identity backend   # inspect keyring|auto|file (config read-only)
```

## Seal a secret

Values never enter via CLI arguments — they would land in shell
history and `/proc`. Pipe them instead:

```sh
printf '%s' "$STRIPE_KEY" | cuora put cuora://billing/stripe
```

Interactive humans get a hidden `/dev/tty` prompt instead of
stdin. The value is encrypted to your identity and stored under
the `dev` env (the default context).

## Declare the surface

`[secrets]` in `cuora.toml` bounds what `run`/`env` may inject —
least privilege, no ambient vault dump:

```toml
[secrets]
STRIPE_KEY = "cuora://billing/stripe"
```

Config search: `./cuora.toml` → `~/.config/cuora/cuora.toml`
(`--config`/`$CUORA_CONFIG` override). No file at all is fine —
defaults are the local vault and the `dev` env.

## Run with secrets

```sh
cuora run -- 'curl -H "Authorization: Bearer $STRIPE_KEY" \
    https://api.stripe.com/v1/charges'
```

`run` resolves the declared refs, injects them as env vars into
the child only, and exits with the child's code. One arg after
`--` runs via `bash -c` (shell syntax, as above); several args
exec `argv[0]` directly with no shell. The value lives only in
that process — agents never see it.

## List, rotate, audit

`list` shows names plus metadata — never values:

```sh
cuora list
cuora list --json
```

The only surfaces that print a value are `get` (the escape hatch)
and `env` (exports for `eval`). Rotate bumps the version and
stamps `rotated_at` — it requires the ref to exist, so it's
deliberate rotation, not upsert:

```sh
printf '%s' "$NEW_VALUE" | cuora rotate cuora://billing/stripe
```

Every cuora op emits one ndjson audit line on stderr:

```json
{"ts":"2026-01-01T00:00:00Z","actor":"you","op":"run","rref":"billing/stripe","env":"dev","outcome":"ok"}
```

See [the audit wire](/reference/audit-events) for the full contract.

To try all of this without touching your real config, run the whole
tutorial in a throwaway `HOME` (`export HOME="$(mktemp -d)"` plus
`CUORA_IDENTITY_BACKEND=file` — the keyring is per-session, not
per-HOME) and `rm -rf "$HOME"` when done.

## Check the setup

```sh
cuora check
```

Validates the config, that the routed vaults decrypt, and that
every `[secrets]` ref resolves — exit `1` on findings.

## Agent self-discovery

```sh
cuora manifest          # verbs, JSON shapes, env vars, audit contract (JSON)
cuora schema            # JSON Schema for cuora.toml
cuora <verb> --help     # every verb, self-documenting
```

`--json` is honored on every verb; errors are `{"error": "…"}`
on stderr with a non-zero exit. The
[cuora skill](https://github.com/9aia/cuora/tree/main/.agents/skills/cuora)
teaches agents the verbs.

## Where next

- [Add a secret](/how-to/add-a-secret) — multi-env and pinning.
- [Inject into a process](/how-to/inject-into-a-process) — run
  modes, ad-hoc `-s` pairs, the `eval "$(cuora env)"` human path.
- [The threat model](/explanation/threat-model.md) — what cuora
  does and does not protect against.
- [CLI reference](/reference/cli.md) — the full verb surface.
