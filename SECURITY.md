# Security Policy

Report vulnerabilities privately by emailing
luisfuturist+9aia@gmail.com with "SECURITY" in the subject — do
not open public issues, do not use GitHub's advisory or
private-reporting features, and do not disclose details publicly;
contact us first and we will coordinate disclosure privately.

## Scope

- cuora materializes secrets — `cuora://` refs in code/config,
  age-encrypted vaults, env-var injection, an ndjson audit wire on
  stderr. See `docs/explanation/threat-model.md` (docs branch) for
  what it does and does not defend against.
- `cuora.toml` holds names, paths, and routing — never values;
  credentials reach vaults via env vars (`token_env` names, not
  inline).
- `identity.txt` (0600) is the master key for every vault impl —
  treat it like a private SSH key.
- Refs and env/actor names are validated before they touch store
  keys — strict segment rules, bounded length, no traversal.

## Notes

We respond to reports as quickly as we can; expect acknowledgement
within a few days.
