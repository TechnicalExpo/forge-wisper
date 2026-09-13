# Language Switcher And Safe Upstream Adoptions

**Source:** `zazanali/forge-wisper:main` comparison against the TechnicalExpo fork.

**Goal:** Add validated language selection to Forge Wisper without breaking the existing CPU/GPU Local Whisper runtime, settings patch flow, or Groq provider.

## Scope

### In scope

- Persist a validated `language` setting with `auto` as the default.
- Pass the selected language through the shared transcription pipeline.
- Use the selected language for Local Whisper and Groq when supported.
- Add a centralized language registry for the UI and Rust validation.
- Keep CPU/GPU/Vulkan context identity correct when language changes.
- Document live transcript/cursor streaming as future work only.

### Deferred

- Real-time partial transcript display.
- Progressive text injection into the active cursor.
- Backspace-based partial transcript correction.
- Full upstream live-audio snapshot architecture.
- Dynamic snippet editing as a separate focused task.
- Groq request cleanup as a separate focused task.
- Startup/window polish as a separate focused task.

## Upstream Review Status

### Completed

- Validated language support is implemented and merged.
- Dynamic snippet editing is implemented and merged.
- Groq safely omits `auto` language and empty prompts while preserving the
  current response format.
- Normal Windows `pnpm tauri:dev` routing now uses the existing Windows launcher.

### Rejected

- Global verification-threshold weakening is rejected because it would weaken
  number, URL, negation, hallucination, and safe-paste protections.
- Upstream per-character cursor injection and blind backspace correction are not
  adopted.
- Upstream whole-buffer audio snapshots are not adopted because they conflict
  with the audio ownership and Local Whisper cache optimizations.

## Remaining Tasks

### Startup And Recorder Window Polish

- [x] Compare upstream autostart/minimized/silent handling against current startup.
- [x] Preserve settings-patch/autostart rollback behavior, Windows hotkeys, and
  the Vulkan launcher.
- [x] Adopt the isolated fix that applies the persisted autostart preference
  instead of forcing startup registration on every launch.
- [ ] Verify Windows autostart launch manually.
- [x] Verify silent/background normal launch, minimized/close-to-tray behavior,
  and recorder window pill visibility manually.

### Groq Request Review

- Add focused provider tests for omitted `auto` language and empty prompts.
- Review response-format compatibility before changing `verbose_json`.
- Keep API keys, raw audio, and transcript content out of logs.

### Release Documentation Cleanup

- Review upstream README/download presentation.
- Keep release asset names dynamic; do not copy hardcoded `0.1.2` names.
- Preserve current version/changelog policy and Local Whisper privacy docs.

## Compatibility Rules

- Never cherry-pick upstream commits wholesale; the upstream branch is based on
  an older architecture and is behind the current implementation.
- Do not overwrite Local Whisper CPU/Vulkan/GPU code or model-context caching.
- Do not replace targeted settings patches with full settings writes.
- Do not reintroduce duplicate hotkey listeners or overlapping mic polling.
- Do not replace database-backed dashboard/history APIs with renderer-side loads.
- Do not commit model weights, raw audio, API keys, or benchmark transcript data.

## Design

- `language` is persisted in `AppSettings` and defaults to `auto` for legacy settings.
- The UI uses a centralized language list with stable language codes and labels.
- Rust validates the language code before pipeline execution.
- Local Whisper passes non-`auto` codes to `FullParams::set_language`.
- Groq sends the language field only when it is non-empty and not `auto`.
- Language is an inference parameter, not a model-context identity parameter, so changing language reuses the same loaded model context while creating a fresh inference state.
- Unsupported/invalid persisted values normalize to `auto`.
- No raw audio, transcript content, or API keys are logged by the language feature.

## Verification

- Legacy settings without `language` deserialize as `auto`.
- Invalid language values normalize to `auto`.
- Local provider receives the selected language without changing CPU/GPU backend selection.
- Groq request behavior omits `auto` and sends explicit language codes.
- Existing full workspace tests, Clippy, and frontend build pass.
- Manual validation covers English, Urdu, and Auto where provider/model support is available.

## Future Work

- Live partial transcript display may be added as an opt-in pill/overlay first.
- Progressive cursor injection must remain separate from the default paste path
  and requires bounded audio windows, output revision tracking, and focus-safe
  cancellation before implementation.
- Dynamic snippet editing, Groq request cleanup, and startup/window polish are
  separate focused tasks.

## Current Status

- Merged release: `0.3.10`.
- Language switcher: complete.
- Language switcher behavior confirmed: the selected language is a speech
  recognition hint, not a translation target; `auto` delegates detection to the
  provider.
- Dynamic snippet editing: complete.
- Normal Windows `pnpm tauri:dev`: routed through the existing Windows launcher.
- Startup and recorder-window review: complete except for manual autostart
  validation.
- Next recommended task: Groq request review tests.
