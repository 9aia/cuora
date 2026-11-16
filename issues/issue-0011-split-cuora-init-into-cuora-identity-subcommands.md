+++
id = "I-011"
title = "add `cuora identity` subcommands alongside `init`"
status = "done"
created = "2026-11-16"
priority = "med"
+++

# add `cuora identity` subcommands alongside `init`

Status: **done** · Created: 2026-11-16

## Description

`cuora init` reads like repo setup but bootstraps the **per-machine,
per-human age identity**. Three axes are conflated: identity
(per-machine), vaults (per-repo config), tokens (per-env secrets).
Teammates onboarding to an existing repo+vault config (ktbhealth)
tripped on this.

## Proposal

Keep `cuora init` — it IS a typical init command (first-time identity
bootstrap, like `git init`). Clarify its help text: "initialize the
local age identity (per-machine, per-human) — not repo setup; run
`cuora check` to verify vault access".

Add an `identity` group for the operations init doesn't cover:

- `cuora identity show` — print the `age1…` recipient pubkey (the
  value shared for `[vaults.*].recipients`)
- `cuora identity import <file>` — adopt an existing `identity.txt`
- `cuora identity backend` — inspect keyring|file|auto storage
  (maybe `identity backend <name>` to switch)

Drop "verify the vault opens" from `init`'s remit — `cuora check`
already does that; init shouldn't double as a health check (it can
keep doing it harmlessly, but help shouldn't advertise it).

## Checklist

- [ ] `Cmd::Identity { sub }` clap surface in cuora-cli
- [ ] ops layer: show/import/backend ops (init stays as-is)
- [ ] `init` help text clarified (identity ≠ repo setup; → `cuora check`)
- [ ] manifest/help/docs updated (`verbs`, `--help`, getting-started)
- [ ] tests for new verbs
- [ ] gen-cli-docs regenerated if applicable

## Comments

- 2026-11-16 DONE — `cuora identity show|import|backend` shipped in cuora-v1.0.0 (merge 50e187eb, pack 9aia:cuora@1.0.0). `init` kept + help clarified. Follow-up: standalone `tardigrade` tool will own generic identity lifecycle; cuora identity verbs get removed once it lands.

- 2026-11-16 UPDATE — the `cuora identity` verbs were retired on main in favor of the standalone `tardigrada` tool (github.com/9aia/tardigrada, published as a tamias pack): identity lifecycle now lives there. `cuora init` stays as cuora's own bootstrap; only the `identity` group was removed.
