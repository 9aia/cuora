# Concepts

The vocabulary of the cuora secrets broker — each term defined once.

## Refs and values

- **`cuora://` ref** — the only way a secret is named in code and
  config: `cuora://<ns>/<name>`, optionally env-pinned
  (`cuora://prod/ns/name`). Never a value.
- **`Secret`** — the redacting newtype every resolved value passes
  through: no `Display`/`Debug` of plaintext, `.expose()` is the only
  way out, memory is `Zeroizing`-scrubbed on drop.
- **Env** — the routing axis (`dev`/`staging`/`prod` + customs). The
  same ref holds different values per env; `cuora://prod/…` pins one.
- **Actor** — who performed an op (`CUORA_ACTOR`, `--actor`).
  Self-claimed attribution, not identity — enforcement is
  peregrin's.

## Vaults and identity

- **Vault** — a readable secret store behind the `cuora-vault` port:
  `get`/`put`/`list`/`rotate`/`rm`, per-env variants, ciphertext-only
  at rest. Impls: `age` (local file), `cf` (self-hosted Worker + D1),
  `cached` (read-through local mirror).
- **Identity** — the one X25519 age keypair that unlocks every vault.
  Held by an `cuora-identity` impl: session keyring (default, fails
  closed) or `identity.txt` keyfile (headless/CI opt-in).
- **Recipient** — a public `age1…` key a vault encrypts to;
  `[vaults.*].recipients` adds team keys without sharing the private
  one.
- **`token_env`** — config names the *env var* carrying a credential;
  the value only ever lives in process env.

## Surfaces and packs

- **`cuora run`** — the agent path: resolves `[secrets]` (+ `-s`
  pairs), injects values as env vars into the child only, exits with
  the child's code. Values never enter the spawner's context.
- **`cuora env`** — the human path: `export VAR='…'` lines for
  `eval`. Prints plaintext — that's why agents use `run`.
- **`[secrets]`** — the injectable surface: the `VAR = "cuora://…"`
  table bounding what `run`/`env` may materialize. Least privilege.
- **`[guardrails]`** — advisory allow/deny op table honored before any
  vault touch. Accident prevention, not enforcement.
- **Audit event** — one ndjson line on stderr per op (actor, op, ref,
  env, outcome, timestamp); OTLP span when an endpoint is configured.
- **Sealed pack** — a tamias `type: secrets` package whose payload is
  age ciphertext only; tamias carries it opaquely, cuora opens it
  consumer-side via `cuora://<pack>/<secret>` refs.
- **Deploy sink** — a write-only push target (`[deploy.*]`, e.g. CF
  Secrets Store). Plaintext crosses once, at push; never a source.
- **`cuora serve`** — the ndjson JSON-RPC surface orchestrators embed;
  `[serve.pdp]` is the exec-driven policy-decision consult.
