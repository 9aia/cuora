# Decisions

ADR-lite: the calls that shaped cuora, the alternatives rejected, and
the consequences we live with. The repo's own `adrs/` hold the full
versions — this page is the map.

## age everywhere; ciphertext-only stores

**Decision:** one crypto primitive — age (X25519) — encrypts every
vault impl client-side; remote stores hold opaque ciphertext and have
no key material.

**Alternatives:** a managed KMS as the crypto boundary; libsodium;
SSH keys; a plaintext-capable remote service.

**Consequence:** a compromised worker or stolen `vault.age` yields
ciphertext, not values — and the same identity unlocks local and
remote vaults. The cost: key management is ours (identity file /
keyring), and there's no passphrase-wrapped identity yet.

## Env vars are the interface; no SDK

**Decision:** secrets materialize as process env vars via `cuora
run`/`cuora env`; there is deliberately no `cuora-sdk`.

**Alternatives:** a Rust-native embed crate; an HTTP secrets API as
the primary surface.

**Consequence:** every consumer — agents, scripts, CI, other org
tools — uses the same mechanism with zero coupling, and the child's
env is the whole contract. `cuora serve` exists for orchestrators
that need a wire, not a library.

## `Secret` redacts by construction

**Decision:** resolved values live in a `Zeroizing` newtype with no
`Display`/`Debug`; `.expose()` is the only escape, called at edges.

**Alternatives:** plain `String` with discipline; redact-at-print
logging filters.

**Consequence:** accidental printing is a compile-time impossibility
in cuora's own code — `list`, the pavo driver, and audit events are
metadata-only by construction. The cost: every legit value out
(exposure to child env, `get`, `env`) must name `.expose()`.

## Values enter via stdin, never argv

**Decision:** `put`/`rotate` read stdin or a hidden `/dev/tty`
prompt; CLI args are refused for values.

**Alternatives:** `--value` flags; env-var input.

**Consequence:** secrets can't leak into shell history or
`/proc/*/cmdline` — the two most common accidental-exposure
channels. Slightly more friction for one-off use; the pipe path is
the documented norm.

## Audit on stderr, aggregation elsewhere

**Decision:** every materialization emits one ndjson event on
stderr (+ OTLP when configured); there is no local audit store —
ADR-0001 removed `audit.jsonl`.

**Alternatives:** a local append-only audit file; direct OTLP only.

**Consequence:** capture is a wrapper away and nothing sensitive is
persisted by cuora — but *collecting* the stream is the consumer's
job (sepia captures it per session; peregrin would aggregate). The
wire stops at stderr by design.

## Guardrails are advisory

**Decision:** `[guardrails]` allow/deny runs before ops but is
documented as accident prevention — `CUORA_ACTOR` is self-claimed
and config is editable by the same actor.

**Alternatives:** cryptographic actor identity; enforcement inside
cuora.

**Consequence:** the honest boundary — cuora *honors* declared rules
and *records* refusals; real policy is peregrin's
(`[serve.pdp]` consult, ADR-0003). Claiming enforcement would be a
lie the threat model refuses to tell.

## The command is a tamias shim

**Decision:** `cuora-cli` is lib-only; the `cuora` command ships as
the `libraries/cuora` tamias pack (`bins[]` shim →
`tamias-driver-cuora cli` → `cuora::run_from`).

**Alternatives:** a `[[bin]]` in `cuora-cli`; crates.io releases.

**Consequence:** install/update is tamias's problem and the engine
is drivable in-process by the driver — but there is no
`cargo install cuora`, and org policy keeps crates off crates.io.
