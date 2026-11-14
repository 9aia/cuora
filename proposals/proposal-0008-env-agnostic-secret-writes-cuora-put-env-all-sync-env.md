+++
id = "P-008"
title = "env-agnostic secret writes: cuora put --env all / sync-env"
status = "open"
created = "2026-11-14"
priority = "low"
+++

## Goal

Today every vault verb is single-env: storing an env-agnostic secret (like `tamias/crates-token`, `tamias/registry-token` was) requires `cuora put --env <e>` once per env, or a `get | put` pipe loop. Error-prone and awkward — the T-037 migration seeded 313 refs × 4 envs by hand-rolled loops.

## Proposal

Add a native cross-env write path. Options (pick in review):

1. `cuora put --env all cuora://scope/name` — prompt once, fan out to every env the ref's vault routing covers.
2. `cuora sync cuora://scope/name --from <env> --to <env|all>` — copy an existing record across envs (useful for rotate-then-replicate too).
3. Config flag: mark a ref env-agnostic so `put`/`rotate` writes all routed envs implicitly.

Constraints: values still stdin/prompt only; audit events per env write; `--json` shapes stable; envs that route to different vaults need explicit handling (probably refuse or require per-vault invocation).

## Context

Surfaced while storing the new crates.io token — the pipe-loop workaround works but shouldn't be the documented path.
