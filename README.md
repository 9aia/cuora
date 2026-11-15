# cuora

*Secrets broker for agents — sealed, not spilled.*

Named for the box turtle *Cuora* — it seals itself with a hinged
shell, and so does this. Agents need API keys, but every plaintext
handoff is a leak surface: prompts, tool output, logs, `.env` pasted
into configs. cuora removes the handoff — code carries `cuora://`
references, never values; the CLI materializes secrets as env vars
at run time; every materialization emits an audit event — ndjson on
stderr, plus OTLP spans when `OTEL_EXPORTER_OTLP_ENDPOINT` is set.

```
agent ──► cuora run -- cmd ──► resolve refs ──► inject env ──► child process
                │                                  (agent never sees the key)
                └──► --json everywhere + manifest/schema self-discovery
```

## Quickstart

```sh
# the `cuora` command is a tamias pack (libraries/cuora) — a
# generated bins[] shim → tamias-driver-cuora → the engine lib.
cargo xtask pack                              # stage the driver payload
tamias install --allow-drivers ./libraries/cuora   # deploys `cuora`

cuora init                                   # age identity, once
printf '%s' "$STRIPE_KEY" | cuora put cuora://billing/stripe

# the agent path — the value exists only inside the child
cuora run -- 'curl -H "Authorization: Bearer $STRIPE_KEY" \
    https://api.stripe.com/v1/charges'

# the human path
eval "$(cuora env -s STRIPE_KEY=cuora://billing/stripe)"
```

Values never enter via CLI args — `put`/`rotate` read stdin or a
hidden prompt. `cuora get` is the escape hatch; prefer `run` so
values never enter agent context. `cuora check` validates config,
vault readability, and declared-surface resolution.

## Why

Every 9aia product needs secrets at runtime, and every plaintext
handoff (prompt, `.env`, shell arg) is a leak. cuora is the shared
broker: `cuora://` refs in code, age-encrypted vaults (local keyfile
or self-hosted Cloudflare Worker + D1), env injection at run time,
and an audit wire on every materialization. It also owns **sealed
packs** — `type: secrets` tamias content for env-agnostic secrets.

## Position in the stack

cuora = **secret materialization**. Neighbors own theirs: **myco**
(work memory), **sturn** (ephemeral coordination), **kuda** (code
memory), **peregrin** (orchestration — campaigns consume via
`cuora run`; embedding + the policy-decision wire is `cuora serve`),
**bupha**/**ci-gate** (CI gate), **mimus** (critic — resolves keys
through cuora), **sepia** (session lifecycle).

## Docs

- [Docs site](https://9aia.com/cuora) — tutorials, how-tos, reference,
  threat model (VitePress + Diátaxis, `docs/` worktree)
- [AGENTS.md](AGENTS.md) — crate map, commands, rules, boundary
- [CONTRIBUTING.md](CONTRIBUTING.md) · [SECURITY.md](SECURITY.md)
- [LICENSE](LICENSE) (MIT) · [project ledger](project/index.md)
