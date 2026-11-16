# Changelog

## [1.0.1] - 2026-11-16

### Changed

- retire cuora identity verbs — superseded by tardigrada (`3224398`)

## [1.0.0] - 2026-11-16

### Added

- **driver:** tamias-driver-cuora — pavo view driver; retire cuora-tui (`d24c5fa`)
- **config:** [aliases] ref-path remap — retired names resolve before vault/pack lookup (T-052) (`b856232`)
- add cuora identity subcommands alongside init (`ab68e68`)
- **cuora-driver:** cli verb is an engine passthrough — the tamias bins[] shim contract (`98c5456`)
- libraries/cuora tamias pack — the cuora command as a bins[] shim (`0e55fd5`)

### Changed

- **BREAKING** rename arca → cuora (crate, binary, env vars, refs, config) (`e66f8a6`)
- **cuora-cli:** lib-ify the engine — Cli/Cmd, parse/parse_from/command/dispatch/run_from (`88b2b46`)

### Fixed

- **audit:** empty CUORA_* no longer shadows ARCA_* fallback; manifest documents legacy aliases (`1f8a3c1`)
