# Changelog

All notable Forge Wisper changes are documented here.

The project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

Add user-visible changes here while work is in progress. Move entries into a
versioned section before merging a release/update branch.

## [0.1.4] - 2026-09-11

### Fixed

- Made the Tauri JSON schema resolve from the installed local CLI package instead of an untrusted remote URL in VS Code.
- Added task completion tracking requirements to repository engineering rules and the performance plan.

## [0.1.3] - 2026-09-11

### Fixed

- Avoided rerunning Windows autostart and global hotkey integration work when unrelated settings change.
- Validated changed OS integrations before persisting settings.

### Tests

- Added regression coverage for targeted settings side effects.

## [0.1.2] - 2026-09-11

### Added

- Added repository engineering rules in `AGENTS.md`.
- Added performance audit and implementation plan under `docs/`.

### Fixed

- Replaced manual `Default` implementations with idiomatic derives where applicable.
- Added `Default` implementations required by Clippy for `ModelManager` and `PipelineState`.

### Verification

- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `pnpm --filter @forge-wisper/desktop build`

## [0.1.1]

- Existing baseline release. See repository history for prior changes.
