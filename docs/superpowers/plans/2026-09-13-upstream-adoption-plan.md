# Upstream Adoption Plan

**Source:** `zazanali/forge-wisper:main` comparison against the TechnicalExpo fork

**Goal:** Adopt only upstream improvements that remain compatible with Forge Wisper's current Local Whisper CPU/Vulkan runtime, settings patches, dashboard state store, safe paste path, and performance remediation.

## Completed

### Validated Language Support

- Persisted `language` with `auto` default.
- Added validated language codes and UI selection.
- Propagated language through Local Whisper and Groq.
- Preserved CPU/GPU backend selection and model-context cache.
- Added normalization tests.

### Dynamic Snippet Editing

- Added edit action for saved snippets.
- Supports trigger rename and expanded-text updates.
- Supports cancel without persistence.
- Preserved targeted settings patches and Rust cleanup preview.

### Safe Groq Request Behavior

- Does not send `auto` as an explicit language.
- Does not send empty prompts.
- Preserves the current response format and provider behavior.

## Deferred By Design

### Live Transcript Display

Future option: show partial transcript in a non-intrusive pill/overlay only.
It must not write into the active input while the user may be changing windows or
choosing a destination field. This should be implemented before cursor streaming
and should use sequence IDs, partial/final states, cancellation, and the existing
CPU/Vulkan context cache.

### Progressive Cursor Typing

Future opt-in setting only. The default remains safe instant paste after the user
finishes dictation. Do not adopt upstream per-character Enigo typing or blind
backspace correction without focus ownership, an output ledger, Unicode-safe
replacement, and measured performance.

### Incremental Audio Snapshots

Future live-inference work must use bounded windows, ring-buffer/incremental
ownership, a background inference task, and cached model contexts. Do not clone
and re-encode the complete recording for every partial update.

### Verification Threshold Changes

Rejected. Existing verification thresholds protect numbers, URLs, negations,
hallucination detection, and safe paste. Macro expansion must not weaken ordinary
transcript verification globally.

## Remaining Focused Tasks

### Task 1: Startup And Recorder Window Polish

- Compare upstream autostart/minimized/silent handling against current startup.
- Preserve settings-patch/autostart rollback behavior.
- Preserve the working Windows hotkey and Vulkan launcher paths.
- Adopt only isolated improvements that prevent startup/window flashes or avoid
  synchronous startup work.
- Verify Windows normal launch, autostart launch, minimized launch, and recorder
  window visibility manually.

### Task 2: Groq Request Review

- Add focused provider tests for omitted `auto` language and empty prompt.
- Review response format compatibility before changing `verbose_json`.
- Never log API keys, raw audio, or transcript content.
- Keep cloud data-flow documentation current.

### Task 3: Release Documentation Cleanup

- Review upstream README badge/download presentation.
- Keep dynamic release asset names; do not copy hardcoded `0.1.2` names.
- Preserve current version/changelog policy and Local Whisper privacy documentation.
- Add only documentation changes that describe behavior actually shipped.

## Compatibility Rules

- Never cherry-pick upstream commits wholesale; the upstream branch is 58 commits behind the current architecture.
- Do not overwrite current Local Whisper CPU/Vulkan/GPU code.
- Do not replace targeted settings patches with full settings writes.
- Do not reintroduce duplicate hotkey listeners or overlapping microphone polling.
- Do not replace database-backed dashboard/history APIs with renderer-side full-history loading.
- Do not commit model weights, raw audio, API keys, or benchmark transcript fixtures.

## Current Branch State

- Current merged release: `0.3.10`.
- Language switcher: merged.
- Dynamic snippet editing: merged.
- Normal `pnpm tauri:dev` now routes through the Windows launcher on Windows.
- Current next task: Startup and recorder-window polish review.
