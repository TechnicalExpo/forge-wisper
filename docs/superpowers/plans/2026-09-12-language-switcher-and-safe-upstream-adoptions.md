# Language Switcher And Safe Upstream Adoptions

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
