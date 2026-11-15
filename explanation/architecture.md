---
url: /cuora/explanation/architecture.md
---
# Architecture

Ports-and-adapters over a flat `crates/` workspace. `cuora-core`
is pure domain — `cuora://` refs, the redacting `Secret` newtype,
the env axis, actor attribution — with no I/O. Ports are traits
(`cuora-config`, `cuora-vault`, `cuora-deploy`); impls live in
separate crates; `crates/cuora-cli` (package `cuora`) is the
composition root that wires impls to verbs.

Design rules:

* **No `unsafe`, no async** — the binary stays dependency-light
  and synchronous; vault impls that speak HTTP use a blocking
  client (`ureq`), so no runtime embeds anywhere.
* **`Secret` never exposes plaintext** — no `Display`/`Debug` of
  the value; `.expose()` is the only way out, called at edges
  only (child env injection, `cuora env` exports, `get`).
* **Stores hold ciphertext only** — every vault impl encrypts
  client-side with age (ADR-0001); a remote vault is ciphertext
  over the wire *and* at rest, never a plaintext service.
* **Ports are conformance-tested** — `cuora-testkit` runs the same
  contract suite against every vault impl (`cuora-vault-age`,
  `cuora-vault-cf`): upsert/rotate/list/env-isolation semantics
  can't drift per backend.
* **One ops shell** — `cuora-ops` owns config load, env→vault
  routing, the env-pin rule, and audit emission; the CLI and
  `cuora serve` are both thin front-ends over it and cannot
  diverge.

| crate | layer | purpose |
|---|---|---|
| `cuora-core` | domain | cuora domain core: cuora:// refs, redacting Secret, env axis, actor, config model — pure, no I/O |
| `cuora-config` | port | Config port: where cuora.toml-shaped config comes from |
| `cuora-deploy` | port | Deploy port: write-only push targets (sinks) for secret values — never a source |
| `cuora-vault` | port | Vault port: readable secret stores — get/put/rm/list/rotate, per-env; every impl stores opaque ciphertext |
| `cuora-ops` | ops | cuora ops I/O: config load, env/ref resolution, vault open, verb plumbing, audit emission |
| `cuora` | engine | cuora — secrets broker for agents: reference URIs, redaction, env-injection CLI |
| `cuora-config-toml` | adapter | Config impl: cuora.toml file (explicit path, env, ./, ~/.config) |
| `cuora-deploy-cf` | adapter | Deploy impl: push secret values to a Cloudflare account Secrets Store (write-only sink) via the CF API v4 |
| `cuora-driver` | adapter | tamias-driver-cuora — pavo view driver over cuora vault metadata (read-only; never renders secret values), engine ops dispatching to the cuora lib, and the sealed-pack surface (seal/open/materialize/recipients over tamias ADR-0004 ciphertext-only packs) |
| `cuora-identity` | adapter | Identity port: where the age identity (master key) comes from — file, keyring, later sealed variants |
| `cuora-identity-file` | adapter | Identity impl: the ~/.config/cuora/identity.txt age keyfile (0600) — the explicit file backend (headless/CI opt-out) |
| `cuora-identity-keyring` | adapter | Identity impl: the platform session secret store (secret-service / Keychain / Credential Manager) via keyring-core |
| `cuora-notice` | adapter | Process-wide UX notices — cuora:-prefixed stderr notes, mutable for terminal-owning surfaces |
| `cuora-sealed` | adapter | cuora sealed packs: tamias store resolution, age identity, encrypt/decrypt, and the cuora://\<pack>/\<secret> engine read path (tamias ADR-0004) |
| `cuora-serve` | adapter | cuora serve — the ndjson request/response IPC surface orchestrators embed, with the external policy-decision consult (P-005) |
| `cuora-shell` | adapter | Run a command in a working directory via bash -c — the generic shell-out helper |
| `cuora-testkit` | adapter | Vault conformance suite — every cuora-vault impl passes the same contract tests (fake values only) |
| `cuora-vault-age` | adapter | Local vault impl: one age-x25519-encrypted JSON doc — the v1 default backend |
| `cuora-vault-cached` | adapter | Composite vault impl: read-through age-encrypted local cache in front of any backend (P-007) |
| `cuora-vault-cf` | adapter | Remote vault impl: self-hosted Cloudflare Worker + D1 holding per-secret age ciphertext — the Worker never sees plaintext |
| `cuora-xtask` | adapter | Repo chores: check / test / coverage / install / site |

## Front-ends

* **`cuora` (CLI)** — clap verbs + `--json`/`manifest`/`schema` for
  machines; the agent interface. There is no standalone binary:
  the command ships as the `libraries/cuora` tamias pack — a
  generated `bins[]` shim → `tamias-driver-cuora`'s `cli`
  passthrough → `cuora::run_from`.
* **`tamias-driver-cuora`** — the pavo view driver: read-only
  vault *metadata* (never renders values), plus the engine ops
  surface (`ops`/`run`/`cli.*` wire queries) that backs the shim.
* **`cuora serve`** — ndjson IPC over the Vault port with an
  exec-driven `[serve.pdp]` policy-decision hook (ADR-0003);
  peregrin's embedding point.

## Where it sits

Cuora is an **engine** in the 9aia stack — it owns one domain
(secret materialization) and is consumed by everything that runs
code:

```
tamias (distribution)  →  engines  →  orchestration  →  surfaces
                          cuora ◄───     peregrin           pavo,
                          myco,         campaigns,         cuora run,
                          kuda,          bupha CI          agents
                          mimus…
```

Peregrin campaigns consume secrets via `cuora run`; mimus resolves
through it via its `mimus-cuora` adapter; agents get the shimmed
`cuora` command via the `libraries/cuora` tamias pack. The
`cuora serve` ndjson wire is peregrin's embedding point — cuora
consults, peregrin decides.

## Boundaries — secret materialization only

cuora owns **secret materialization** — refs, vaults, env
injection, redaction, audit — and nothing else. If a feature needs
another kind of memory or authority, it belongs to a neighbor:

| neighbor | owns | how it meets cuora |
| --- | --- | --- |
| **myco** | work memory — tasks, issues, proposals, plans, ADRs, handoffs | this repo's `project/` ledger is myco's; cuora has no PM surface |
| **sturn** | ephemeral coordination — live claims/ops/topics | not secrets: sturn state is transient, cuora state is sealed |
| **kuda** | code memory — symbol/index knowledge | cuora refs name *secrets*, not code entities |
| **peregrin** | orchestration — campaigns, multi-actor arbitration, policy | campaigns consume secrets via `cuora run`; audit aggregation + real policy are peregrin's (`CUORA_ACTOR` is self-claimed inside cuora); `cuora serve` — the ndjson IPC wire + `[serve.pdp]` exec consult (ADR-0003) — is its embedding point |
| **bupha** | CI gate | the org gate is the peregrin `ci-gate` campaign, mid-migration back to bupha (`type: bupha` bundles); it asserts no plaintext secrets land in pipelines, `cuora check` in pipeline steps |
| **mimus** | critic | secrets resolve env → cuora → legacy (keyring, `~/.config/mimus/env`) via `mimus-cuora` shelling `cuora`; vaults are canonical, writes land in them |
| **sepia** | session lifecycle | session wrappers can capture cuora's stderr audit stream |
| **tamias** | distribution | sealed packs (`type: secrets`) carry age ciphertext cuora opens consumer-side |

What that means in practice:

* **A hosted vault service is in scope** — `cuora-vault-cf` is a
  store impl behind the same port, ciphertext-only, self-hosted.
* **A daemon is out of scope** — persistent coordination,
  multi-machine views, arbitration between actors → peregrin.
  cuora stays a sync, embeddable binary: env vars are the
  interface, no SDK.
* **Policy is out of scope** — `[guardrails]` are advisory
  accident prevention. Enforcement, identity, and unforgeable
  attribution are orchestration concerns.
* **The audit wire stops at stderr** — emitting ndjson is cuora's;
  collecting, storing, and alerting on it is peregrin's
  (ADR-0001 deliberately removed the local `audit.jsonl`).
