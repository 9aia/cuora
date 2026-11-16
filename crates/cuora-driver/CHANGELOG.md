# Changelog

## [1.0.3] - 2026-11-16

### Fixed

- **sealed:** confine payload reads to the payload dir; tighten secret-file mode (`99c6e43`)

## [1.1.0] - 2026-11-16

### Added

- canonicalize the age identity under the tardigrada namespace (`a39c1cd`)

## [1.0.1] - 2026-11-16

### Changed

- retire cuora identity verbs — superseded by tardigrada (`3224398`)

## [0.3.0] - 2026-11-16

### Added

- **cuora-driver:** engine ops surface — ops manifest, run passthrough, cli/mcp features (`e6be371`)
- **driver:** tamias-driver-cuora — pavo view driver; retire cuora-tui (`d24c5fa`)
- **config:** [aliases] ref-path remap — retired names resolve before vault/pack lookup (T-052) (`b856232`)
- **driver:** sealed packs — seal/open/materialize/recipients over tamias ADR-0004 ciphertext (PL-017 T3) (`b37ec20`)
- add cuora identity subcommands alongside init (`ab68e68`)
- **ops:** resolve cuora://<pack>/<secret> refs against installed sealed packs (T-051) (`9fd14a0`)
- **cuora-driver:** cli verb is an engine passthrough — the tamias bins[] shim contract (`98c5456`)
- **sealed:** pack@version ref pins + seal --manifest merge (P-027 follow-ups) (`7edd027`)
- **ops:** sealed packs answer when the vault can't open (T-053) (`3486d41`)
- libraries/cuora tamias pack — the cuora command as a bins[] shim (`0e55fd5`)

### Changed

- **cuora-driver:** adopt pavo-driver-kit (`fa11ddd`)

### Fixed

- **driver:** cuora-driver — renderers field + Render arm for pavo-protocol v1 renderer plugins (`ad4a743`)
