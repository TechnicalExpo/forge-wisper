# Changelog

All notable Forge Wisper changes are documented here.

The project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

Add user-visible changes here while work is in progress. Move entries into a
versioned section before merging a release/update branch.

## [0.1.9] - 2026-09-11

### Fixed

- Kept `Control+Super` and `Super+Space` as supported two-key Windows shortcuts by routing Super combinations through the native Windows listener.

## [0.1.8] - 2026-09-11

### Fixed

- Restored Windows-key hotkey support through the native Windows listener while keeping Tauri as the canonical path for other shortcuts.
- Preserved two-key shortcuts `Control+Super` and `Super+Space` without requiring an extra primary key.

## [0.1.7] - 2026-09-11

### Fixed

- Restored two-key Windows-key shortcuts such as `Control+Super` and `Super+Space` through the native Windows listener.

## [0.1.6] - 2026-09-11

### Fixed

- Removed the fake Local Whisper success transcript.
- Blocked Local Whisper selection until a real offline runtime is bundled.
- Migrated persisted Local Whisper settings back to Groq Cloud instead of silently using a non-functional provider.

## [0.1.5] - 2026-09-11

### Performance

- Avoided repeated floating recorder monitor queries and repositioning during a single processing session.
- Replaced render-time recorder animation clock reads with CSS animation.

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
