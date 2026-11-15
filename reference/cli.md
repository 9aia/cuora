# Command-Line Help for `cuora`

This document contains the help content for the `cuora` command-line program.

cuora — secrets broker for agents: `cuora://` refs, redaction, env-injection instead of plaintext. Refs name logical secrets (`cuora://billing/stripe`); `--env` picks the resolution context. `[secrets]` in cuora.toml bounds what `run`/`env` may inject. `manifest`/`schema` describe the surface for machines; every verb honors `--json`.

**Command Overview:**

* [`cuora`↴](#cuora)
* [`cuora init`↴](#cuora-init)
* [`cuora put`↴](#cuora-put)
* [`cuora get`↴](#cuora-get)
* [`cuora list`↴](#cuora-list)
* [`cuora rm`↴](#cuora-rm)
* [`cuora rotate`↴](#cuora-rotate)
* [`cuora run`↴](#cuora-run)
* [`cuora env`↴](#cuora-env)
* [`cuora check`↴](#cuora-check)
* [`cuora migrate`↴](#cuora-migrate)
* [`cuora deploy`↴](#cuora-deploy)
* [`cuora serve`↴](#cuora-serve)
* [`cuora manifest`↴](#cuora-manifest)
* [`cuora schema`↴](#cuora-schema)

## `cuora`

cuora — secrets broker for agents: `cuora://` refs, redaction, env-injection instead of plaintext. Refs name logical secrets (`cuora://billing/stripe`); `--env` picks the resolution context. `[secrets]` in cuora.toml bounds what `run`/`env` may inject. `manifest`/`schema` describe the surface for machines; every verb honors `--json`.

**Usage:** `cuora [OPTIONS] <COMMAND>`

###### **Commands:**

* `init` — Bootstrap: store the age identity per `[identity] backend` / `CUORA_IDENTITY_BACKEND` — `keyring` (default) fails closed when no session store is reachable, `auto` falls back to the keyfile with a note, `file` never probes. Imports an existing identity.txt rather than orphaning it. Verifies the vault opens
* `put` — Store a secret. The value comes from stdin (`printf … | cuora put cuora://ns/name`) or a hidden terminal prompt — never args, which land in shell history and `/proc`
* `get` — Print a secret's value to stdout — the escape hatch. Prefer `cuora run` so values never enter agent context; guardrails can deny this op
* `list` — List the refs stored in the selected env — names only, never values
* `rm` — Remove a secret from the selected env
* `rotate` — Rotate a secret to a new value (stdin or hidden prompt, like `put`); bumps version + rotated_at
* `run` — Run a command with declared secrets injected as env vars
* `env` — Print `export VAR='…'` lines for `eval "$(cuora env)"` — the human path (`run` is the agent path). Values print to stdout: `--json` emits a `VAR → value` map
* `check` — Validate config + the declared surface: `[envs]` routes point at declared vaults, vaults decrypt, every `[secrets]` ref resolves in its routed env. Exit 1 on findings
* `migrate` — Rewrite stored records for a wire/config upgrade. Currently one transform: `--private-refs` re-keys every plaintext-named row in each env routed to a cf vault with `[vaults.*] private_refs = true` — ciphertext moves verbatim, old keys are deleted after the new rows land
* `deploy` — Push a stored secret to a write-only deploy target (`[deploy.NAME]` in cuora.toml, e.g. a CF Secrets Store). The vault is the source of truth; the sink never gives the value back. With one configured target, `--target` may be omitted
* `serve` — Serve the Vault port over ndjson — one JSON request per stdin line, one response per stdout line, audit events on stderr unchanged. For orchestrators embedding cuora (peregrin); with `[serve.pdp]` configured, a policy decision point is consulted before every vault-touching op. `{"op":"manifest"}` on the wire describes the protocol
* `manifest` — Capabilities, JSON shapes, env vars, audit contract — agent self-discovery (always JSON)
* `schema` — JSON Schema for `cuora.toml`

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--env <ENV>` — Env context — `dev`/`staging`/`prod`/custom. Overrides `CUORA_ENV`; default `dev`
* `--config <PATH>` — Config file, overriding the search ($CUORA_CONFIG → ./cuora.toml → ~/.config/cuora/cuora.toml)
* `--actor <ACTOR>` — Actor attribution for audit events (default $CUORA_ACTOR, else $USER)
* `-h, --help` — Print help (see a summary with '-h')
* `-V, --version` — Print version

## `cuora init`

Bootstrap: store the age identity per `[identity] backend` / `CUORA_IDENTITY_BACKEND` — `keyring` (default) fails closed when no session store is reachable, `auto` falls back to the keyfile with a note, `file` never probes. Imports an existing identity.txt rather than orphaning it. Verifies the vault opens

**Usage:** `cuora init [OPTIONS]`

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--with-file` — Force the identity keyfile even when a keyring is available — the headless/CI mode
* `--env <ENV>` — Env context — `dev`/`staging`/`prod`/custom. Overrides `CUORA_ENV`; default `dev`
* `--config <PATH>` — Config file, overriding the search ($CUORA_CONFIG → ./cuora.toml → ~/.config/cuora/cuora.toml)
* `--actor <ACTOR>` — Actor attribution for audit events (default $CUORA_ACTOR, else $USER)
* `-h, --help` — Print help

## `cuora put`

Store a secret. The value comes from stdin (`printf … | cuora put cuora://ns/name`) or a hidden terminal prompt — never args, which land in shell history and `/proc`

**Usage:** `cuora put [OPTIONS] <RREF>`

###### **Arguments:**

* `<RREF>` — `cuora://` ref, e.g. cuora://billing/stripe

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--env <ENV>` — Env context — `dev`/`staging`/`prod`/custom. Overrides `CUORA_ENV`; default `dev`
* `--config <PATH>` — Config file, overriding the search ($CUORA_CONFIG → ./cuora.toml → ~/.config/cuora/cuora.toml)
* `--actor <ACTOR>` — Actor attribution for audit events (default $CUORA_ACTOR, else $USER)
* `-h, --help` — Print help

## `cuora get`

Print a secret's value to stdout — the escape hatch. Prefer `cuora run` so values never enter agent context; guardrails can deny this op

**Usage:** `cuora get [OPTIONS] <RREF>`

###### **Arguments:**

* `<RREF>` — `cuora://` ref

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--env <ENV>` — Env context — `dev`/`staging`/`prod`/custom. Overrides `CUORA_ENV`; default `dev`
* `--config <PATH>` — Config file, overriding the search ($CUORA_CONFIG → ./cuora.toml → ~/.config/cuora/cuora.toml)
* `--actor <ACTOR>` — Actor attribution for audit events (default $CUORA_ACTOR, else $USER)
* `-h, --help` — Print help

## `cuora list`

List the refs stored in the selected env — names only, never values

**Usage:** `cuora list [OPTIONS]`

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--env <ENV>` — Env context — `dev`/`staging`/`prod`/custom. Overrides `CUORA_ENV`; default `dev`
* `--config <PATH>` — Config file, overriding the search ($CUORA_CONFIG → ./cuora.toml → ~/.config/cuora/cuora.toml)
* `--actor <ACTOR>` — Actor attribution for audit events (default $CUORA_ACTOR, else $USER)
* `-h, --help` — Print help

## `cuora rm`

Remove a secret from the selected env

**Usage:** `cuora rm [OPTIONS] <RREF>`

###### **Arguments:**

* `<RREF>` — `cuora://` ref

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--env <ENV>` — Env context — `dev`/`staging`/`prod`/custom. Overrides `CUORA_ENV`; default `dev`
* `--config <PATH>` — Config file, overriding the search ($CUORA_CONFIG → ./cuora.toml → ~/.config/cuora/cuora.toml)
* `--actor <ACTOR>` — Actor attribution for audit events (default $CUORA_ACTOR, else $USER)
* `-h, --help` — Print help

## `cuora rotate`

Rotate a secret to a new value (stdin or hidden prompt, like `put`); bumps version + rotated_at

**Usage:** `cuora rotate [OPTIONS] <RREF>`

###### **Arguments:**

* `<RREF>` — `cuora://` ref

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--env <ENV>` — Env context — `dev`/`staging`/`prod`/custom. Overrides `CUORA_ENV`; default `dev`
* `--config <PATH>` — Config file, overriding the search ($CUORA_CONFIG → ./cuora.toml → ~/.config/cuora/cuora.toml)
* `--actor <ACTOR>` — Actor attribution for audit events (default $CUORA_ACTOR, else $USER)
* `-h, --help` — Print help

## `cuora run`

Run a command with declared secrets injected as env vars. Injection surface = `[secrets]` in cuora.toml plus `-s VAR=cuora://ref` ad-hoc pairs; nothing else is injectable. Two exec modes after `--`: `cuora run -- 'pnpm test | tee log'` — one string → `bash -c` `cuora run -- pnpm test --filter x` — argv → direct exec argv mode needs no shell quoting — prefer it when the command has arguments. Exit code is the child's.

**Usage:** `cuora run [OPTIONS] -- <CMD>...`

###### **Arguments:**

* `<CMD>...` — Command to run (after `--`). One arg → `bash -c`; several → `argv[0]` exec'd directly with the rest

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `-s, --secret <VAR=CUORA_REF>` — Extra `VAR=cuora://ref` pairs to inject (refused when `[guardrails] deny_adhoc` is set)
* `--env <ENV>` — Env context — `dev`/`staging`/`prod`/custom. Overrides `CUORA_ENV`; default `dev`
* `--config <PATH>` — Config file, overriding the search ($CUORA_CONFIG → ./cuora.toml → ~/.config/cuora/cuora.toml)
* `--actor <ACTOR>` — Actor attribution for audit events (default $CUORA_ACTOR, else $USER)
* `-h, --help` — Print help (see a summary with '-h')

## `cuora env`

Print `export VAR='…'` lines for `eval "$(cuora env)"` — the human path (`run` is the agent path). Values print to stdout: `--json` emits a `VAR → value` map

**Usage:** `cuora env [OPTIONS]`

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `-s, --secret <VAR=CUORA_REF>` — Extra `VAR=cuora://ref` pairs to resolve
* `--env <ENV>` — Env context — `dev`/`staging`/`prod`/custom. Overrides `CUORA_ENV`; default `dev`
* `--config <PATH>` — Config file, overriding the search ($CUORA_CONFIG → ./cuora.toml → ~/.config/cuora/cuora.toml)
* `--actor <ACTOR>` — Actor attribution for audit events (default $CUORA_ACTOR, else $USER)
* `-h, --help` — Print help

## `cuora check`

Validate config + the declared surface: `[envs]` routes point at declared vaults, vaults decrypt, every `[secrets]` ref resolves in its routed env. Exit 1 on findings

**Usage:** `cuora check [OPTIONS]`

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--env <ENV>` — Env context — `dev`/`staging`/`prod`/custom. Overrides `CUORA_ENV`; default `dev`
* `--config <PATH>` — Config file, overriding the search ($CUORA_CONFIG → ./cuora.toml → ~/.config/cuora/cuora.toml)
* `--actor <ACTOR>` — Actor attribution for audit events (default $CUORA_ACTOR, else $USER)
* `-h, --help` — Print help

## `cuora migrate`

Rewrite stored records for a wire/config upgrade. Currently one transform: `--private-refs` re-keys every plaintext-named row in each env routed to a cf vault with `[vaults.*] private_refs = true` — ciphertext moves verbatim, old keys are deleted after the new rows land

**Usage:** `cuora migrate [OPTIONS]`

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--private-refs` — Re-key cf-vault rows under HMAC-derived storage keys, attaching the encrypted `name` blob (P-004)
* `--env <ENV>` — Env context — `dev`/`staging`/`prod`/custom. Overrides `CUORA_ENV`; default `dev`
* `--config <PATH>` — Config file, overriding the search ($CUORA_CONFIG → ./cuora.toml → ~/.config/cuora/cuora.toml)
* `--actor <ACTOR>` — Actor attribution for audit events (default $CUORA_ACTOR, else $USER)
* `-h, --help` — Print help

## `cuora deploy`

Push a stored secret to a write-only deploy target (`[deploy.NAME]` in cuora.toml, e.g. a CF Secrets Store). The vault is the source of truth; the sink never gives the value back. With one configured target, `--target` may be omitted

**Usage:** `cuora deploy [OPTIONS] <RREF>`

###### **Arguments:**

* `<RREF>` — `cuora://` ref to push

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--target <NAME>` — Deploy target name (`[deploy.NAME]`); required when several are configured
* `--env <ENV>` — Env context — `dev`/`staging`/`prod`/custom. Overrides `CUORA_ENV`; default `dev`
* `--config <PATH>` — Config file, overriding the search ($CUORA_CONFIG → ./cuora.toml → ~/.config/cuora/cuora.toml)
* `--actor <ACTOR>` — Actor attribution for audit events (default $CUORA_ACTOR, else $USER)
* `-h, --help` — Print help

## `cuora serve`

Serve the Vault port over ndjson — one JSON request per stdin line, one response per stdout line, audit events on stderr unchanged. For orchestrators embedding cuora (peregrin); with `[serve.pdp]` configured, a policy decision point is consulted before every vault-touching op. `{"op":"manifest"}` on the wire describes the protocol

**Usage:** `cuora serve [OPTIONS]`

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--env <ENV>` — Env context — `dev`/`staging`/`prod`/custom. Overrides `CUORA_ENV`; default `dev`
* `--config <PATH>` — Config file, overriding the search ($CUORA_CONFIG → ./cuora.toml → ~/.config/cuora/cuora.toml)
* `--actor <ACTOR>` — Actor attribution for audit events (default $CUORA_ACTOR, else $USER)
* `-h, --help` — Print help

## `cuora manifest`

Capabilities, JSON shapes, env vars, audit contract — agent self-discovery (always JSON)

**Usage:** `cuora manifest [OPTIONS]`

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--env <ENV>` — Env context — `dev`/`staging`/`prod`/custom. Overrides `CUORA_ENV`; default `dev`
* `--config <PATH>` — Config file, overriding the search ($CUORA_CONFIG → ./cuora.toml → ~/.config/cuora/cuora.toml)
* `--actor <ACTOR>` — Actor attribution for audit events (default $CUORA_ACTOR, else $USER)
* `-h, --help` — Print help

## `cuora schema`

JSON Schema for `cuora.toml`

**Usage:** `cuora schema [OPTIONS]`

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--env <ENV>` — Env context — `dev`/`staging`/`prod`/custom. Overrides `CUORA_ENV`; default `dev`
* `--config <PATH>` — Config file, overriding the search ($CUORA_CONFIG → ./cuora.toml → ~/.config/cuora/cuora.toml)
* `--actor <ACTOR>` — Actor attribution for audit events (default $CUORA_ACTOR, else $USER)
* `-h, --help` — Print help
