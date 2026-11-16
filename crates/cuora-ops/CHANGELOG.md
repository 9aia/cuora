# Changelog

## [1.0.0] - 2026-11-16

### Added

- **driver:** tamias-driver-cuora — pavo view driver; retire cuora-tui (`d24c5fa`)
- **config:** [aliases] ref-path remap — retired names resolve before vault/pack lookup (T-052) (`b856232`)
- add cuora identity subcommands alongside init (`ab68e68`)
- **ops:** resolve cuora://<pack>/<secret> refs against installed sealed packs (T-051) (`9fd14a0`)
- **sealed:** pack@version ref pins + seal --manifest merge (P-027 follow-ups) (`7edd027`)
- **ops:** sealed packs answer when the vault can't open (T-053) (`3486d41`)

### Changed

- **BREAKING** rename arca → cuora (crate, binary, env vars, refs, config) (`e66f8a6`)

### Fixed

- **audit:** empty CUORA_* no longer shadows ARCA_* fallback; manifest documents legacy aliases (`1f8a3c1`)
