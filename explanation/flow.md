---
url: /cuora/explanation/flow.md
---
# Flow: a secret's life — seal, declare, inject, audit

One end-to-end narrative of cuora's flagship operation: getting a
value from a human's hands into a running process without it ever
appearing in a repo, a prompt, an argument list, or an agent's
context.

## 1. Bootstrap

```sh
cuora init
```

Generates (or imports) the shared tardigrada age identity — into
the session keyring
by default (canonical `tardigrada/age-identity` entry), the `0600`
keyfile (`<config>/tardigrada/identity.txt`) on headless hosts —
and verifies the
vault opens. One audit event lands on stderr: `{"op":"init",…}`.

## 2. Seal

```sh
printf '%s' "$STRIPE_KEY" | cuora put cuora://billing/stripe
# stored cuora://billing/stripe (v1)
```

The value crosses stdin → age encryption → `vault.age` ciphertext.
It is never an argument (no shell history, no `/proc`), never a
file, never printed. `list` shows the ref and metadata — not the
value.

## 3. Declare

```toml
# cuora.toml
[secrets]
STRIPE_KEY = "cuora://billing/stripe"
```

The `[secrets]` table is the *injectable surface* — least
privilege. `cuora check` verifies every declared ref resolves
before anything runs.

## 4. Inject

```sh
cuora run -- 'curl -H "Authorization: Bearer $STRIPE_KEY" \
    https://api.stripe.com/v1/charges'
```

`run` resolves the declared refs, spawns the child with the values
as env vars, and exits with the child's code. The agent that typed
the command sees the child's *output* — the variable itself only
exists inside the child. Missing refs fail closed before spawn;
partial injection is worse than none.

## 5. Audit

Every hop emitted ndjson on stderr:

```json
{"op":"init","outcome":"ok"}
{"op":"put","rref":"billing/stripe","env":"dev","outcome":"ok"}
{"op":"run","rref":"billing/stripe","env":"dev","outcome":"ok"}
```

Capture is `2> audit.jsonl` or a session wrapper; OTLP spans flow
when `OTEL_EXPORTER_OTLP_ENDPOINT` is set. Rotation is `cuora
rotate` (version bump + `rotated_at`); re-deploy to sinks is
`cuora deploy` — write-only, one-way.

## The shape underneath

```
plaintext ──stdin──► age encrypt ──► ciphertext at rest (vault/pack)
                                              │
config [secrets] ──refs──► resolve ──► child env ──► process
                          (never the spawner's context)
        every op ──► ndjson audit event on stderr
```

See [architecture](architecture.md) for the ports,
[the threat model](threat-model.md) for what this does and doesn't
buy, and [concepts](concepts.md) for the vocabulary.
