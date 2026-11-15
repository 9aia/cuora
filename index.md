---
layout: home

hero:
  name: cuora
  text: secrets broker for agents — sealed, not spilled
  tagline: cuora:// references in code, env vars at runtime, an audit trail for every materialization
  actions:
    - theme: brand
      text: Getting Started
      link: /tutorials/getting-started
    - theme: alt
      text: Tutorial
      link: /tutorials/first-secret

features:
  - icon: 🦪
    title: References, not values
    details: cuora://name URIs in code and config — nothing sensitive lives in the repo at all
    link: /explanation/architecture
    linkText: architecture
  - icon: 🤖
    title: Agent-first
    details: cuora run injects secrets as env vars — agents run commands without ever seeing keys; --json + manifest/schema everywhere
    link: /reference/cli
    linkText: CLI reference
  - icon: 🗄️
    title: Pluggable vaults
    details: the cuora-vault port — local age-encrypted file first, a self-hosted Cloudflare Worker + D1 backend too; ciphertext-only everywhere
    link: /how-to/route-envs-to-vaults
    linkText: env routing
  - icon: 🔏
    title: Audited materialization
    details: every resolve, denial, and failure records actor, ref, env, timestamp — an ndjson wire on stderr; Secret redacts by construction
    link: /reference/audit-events
    linkText: audit contract
---

## Where to go next

| You want to… | Go to |
| --- | --- |
| Install cuora and seal your first secret | [Tutorial](/tutorials/getting-started) |
| Do a task (rotate, route envs, deploy a sink) | [How-to](/how-to/add-a-secret) |
| Understand the design (threat model, crypto model) | [Explanation](/explanation/threat-model) |
| Look up a verb, audit event, or env var | [Reference](/reference/cli) |
