# Changelog

All notable Forge Wisper changes are documented here.

The project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

Add user-visible changes here while work is in progress. Move entries into a
versioned section before merging a release/update branch.

## [0.3.11] - 2026-09-13

### Fixed

- Startup now applies the persisted autostart preference instead of forcibly
  enabling Windows startup on every launch.

## [0.3.10] - 2026-09-13

### Fixed

- Routed the normal `pnpm tauri:dev` command through the platform-specific
  development environment so Windows loads the existing Vulkan/MSVC launcher.
- Preserved the explicit `pnpm tauri:dev:windows` command and macOS/Linux direct
  Tauri development flow.

## [0.3.9] - 2026-09-12

### Added

- Added dynamic voice-snippet editing with trigger rename, value updates, and
  cancel-safe editing.

## [0.3.8] - 2026-09-12

### Added

- Added validated recognition-language selection with Auto Detect as the default.
- Propagated explicit language selection through Local Whisper and Groq while
  preserving CPU/GPU backend selection and model context caching.

### Deferred

- Real-time partial transcript cursor injection remains intentionally deferred;
  future streaming must be opt-in and preserve the current safe paste path.

## [0.3.7] - 2026-09-12

### Tests

- Closed Local Whisper runtime and GPU validation with recorded same-audio CPU,
  Vulkan GPU, cache warm-run, and Groq benchmark evidence.
- Closed the performance-remediation verification checklist and documented the
  remaining optional/manual release-validation boundaries.

## [0.3.6] - 2026-09-12

### Tests

- Recorded a same-audio Local Whisper CPU/GPU and Groq benchmark using the
  externally supplied WAV fixture.
- Documented cold and warm model timings, Vulkan backend evidence, and cloud
  comparison/privacy impact without committing audio or transcript fixtures.

## [0.3.5] - 2026-09-12

### Documentation

- Updated the performance audit with the completed settings, state, audio,
  storage, dictionary, and Local Whisper remediation status.
- Recorded the remaining physical Windows scenario and before/after profiling
  work as release-validation items instead of claiming unmeasured results.

## [0.3.4] - 2026-09-12

### Fixed

- Dictionary preview now uses the same Rust cleanup pipeline as real dictation.
- Added debounced preview requests and stale-response protection.

## [0.3.3] - 2026-09-12

### Fixed

- Enabled Local Whisper selection directly from the Dashboard engine selector.
- Kept the Dashboard dictation button synchronized with hotkey-triggered
  recording and processing state changes without requiring navigation.
- Refreshed the latest dictation and dashboard metrics after terminal pipeline
  state transitions.

## [0.3.2] - 2026-09-12

### Performance

- Moved dashboard word, duration, and session metrics into SQLite aggregate
  queries instead of loading and reducing the history list in the renderer.
- Added database-backed history pagination with offset, limit, and total count.
- Added 250 ms debounced history search with stale-request protection.

## [0.3.1] - 2026-09-12

### Performance

- Added Local Whisper context caching for repeated transcriptions.
- Kept CPU and Vulkan contexts isolated by model path, backend, and GPU device.
- Reduced settings IPC payloads by sending targeted settings patches instead of
  the complete settings object for every change.
- Avoided unrelated hotkey registration and Windows autostart work when other
  settings change.

### Reliability

- Added cache invalidation when the selected Local Whisper model changes.
- Added rollback behavior when hotkey or autostart validation fails before
  settings are persisted.
- Added a `forge://settings-changed` event for synchronized settings consumers.

### Documentation

- Added repeatable real-model Local Whisper E2E evidence collection for CPU,
  Vulkan GPU, cache warm-up, pipeline verification, and optional Groq comparison.

## [0.3.0] - 2026-09-11

### Added

- Added Local Whisper CPU/GPU compute-device selection with CPU as the default.
- Added Windows Vulkan backend support with NVIDIA, AMD, and Intel device discovery.
- Added explicit GPU-mode errors instead of silently falling back to CPU.

## [0.2.3] - 2026-09-11

### Added

- Added Local Whisper download, verification, installation, transcription, and
  backend-selection logs for manual E2E diagnostics.
- Documented that the current Local Whisper runtime uses CPU inference only.

## [0.2.2] - 2026-09-11

### Fixed

- Clarified Local Whisper model download states by showing startup, download,
  verification, and installed-ready phases separately.

## [0.2.1] - 2026-09-11

### Security

- Added pinned Whisper model revisions, expected byte sizes, and SHA-256 catalog metadata for model downloads.
- Model files are verified before atomic activation; checksum/size mismatches remove the partial file.

## [0.2.0] - 2026-09-11

### Added

- Added real CPU Local Whisper inference through `whisper-rs`/whisper.cpp using downloaded GGML models.
- Added offline local transcription support for installed models while preserving Groq Cloud as the fast default.

## [0.1.10] - 2026-09-11

### Fixed

- Added a full formatting matrix covering Raw, Clean, Structured, and Smart output with emails, URLs, numbers, paragraphs, bullets, numbering, fillers, and corrections.
- Fixed spoken weekday corrections without punctuation and preserved numeric comma formatting such as `15,500`.

## [0.1.10] - 2026-09-11

### Fixed

- Made formatting modes behaviorally distinct: Clean no longer applies self-corrections or structure detection, Structured adds outline structure, and Smart applies contextual corrections plus structure.

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
