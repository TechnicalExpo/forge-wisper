# Forge Wisper Performance Remediation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Forge Wisper settings, recording controls, and processing feedback responsive while correcting the confirmed correctness risks found in the audit.

**Architecture:** Keep Rust/Tauri as the desktop boundary, but make settings updates targeted and transactional, use one hotkey implementation, and reduce frontend-to-Rust polling. Introduce typed domain events so views do not independently reload shared data.

**Tech Stack:** Rust 2021, Tauri 2, React 18, TypeScript, Vite, CPAL, SQLite/rusqlite, Tokio.

## Global Constraints

- Do not migrate away from Rust, Tauri, React, or TypeScript.
- Do not change persisted settings names without a migration path.
- Do not run destructive history/model operations without existing confirmation behavior.
- Preserve Windows and macOS behavior.
- Every performance change must have a before/after measurement or a focused regression test.
- Run `pnpm --filter @forge-wisper/desktop build` after frontend tasks.
- Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` after Rust tooling is installed.

---

### Task 1: Establish Rust Toolchain And Baseline

**Files:**
- No source changes.

**Interfaces:**
- Produces a working `cargo` and `rustup` environment for all later tasks.

- [x] **Step 1: Verify installed tools**

```powershell
rustup --version
cargo --version
rustc --version
```

Expected current toolchain after installation:

```text
rustup 1.29.1
rustc 1.98.1
cargo 1.98.1
```

- [x] **Step 2: Verify the workspace baseline**

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
pnpm --filter @forge-wisper/desktop build
```

On the initial machine baseline, the frontend build passed. Rust compilation requires Microsoft Visual C++ Build Tools and a shell where Microsoft's `link.exe` precedes Git/Hermes' `link.exe`.

- [x] **Step 3: Record failures before source changes**

Save command output in the task notes or issue tracker. Do not modify source as part of this task.

---

### Task 2: Add Timing Instrumentation To Critical Commands

**Files:**
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src-tauri/src/state.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src-tauri/Cargo.toml` only if the existing tracing setup requires it.

**Interfaces:**
- Produces structured timing logs for settings, recording, state transitions, hotkeys, and OS operations.

- [x] **Step 1: Add a small elapsed-time helper using `Instant`**

Use the existing `std::time::Instant` import pattern. Log operation name and elapsed milliseconds at the end of each critical command.

- [x] **Step 2: Instrument `update_settings`, `start_recording`, `stop_recording`, `get_mic_level`, and `list_history`**

Include operation name, success/failure, and relevant payload sizes. Never log API keys or transcript contents. `get_mic_level` is intentionally not logged per call because it is a high-frequency polling boundary; its latency is represented by the sampling/IPC behavior instead of adding more per-call overhead.

- [x] **Step 3: Instrument hotkey registration and Windows autostart operations**

Record how long `unregister_all`, each registration phase, and `reg` command execution takes.

- [x] **Step 4: Run Rust tests, Clippy, and frontend build**

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
pnpm --filter @forge-wisper/desktop build
```

Instrumentation is emitted under the `forge.performance` tracing target. It
records timings and safe metadata only; it does not record API keys, transcript
text, raw audio, or clipboard contents.

---

### Task 3: Make Settings Updates Targeted And Transactional

**Files:**
- Modify: `apps/desktop/src-tauri/src/commands.rs:46-68`
- Modify: `apps/desktop/src-tauri/src/lib.rs:166-204,341-375`
- Modify: `apps/desktop/src/lib/tauri.ts`
- Modify: `apps/desktop/src/types/index.ts` if a patch type is added.
- Test: Rust unit tests in `apps/desktop/src-tauri/src/`.

**Interfaces:**
- Add a targeted settings update interface that changes only the requested field.
- Hotkey registration runs only when the hotkey changes.
- Autostart registry work runs only when the startup preference changes.

- [ ] **Step 1: Add a Rust settings patch type**

Create a serializable patch with optional fields for `provider`, `model`, `microphone`, `formatting_mode`, `hotkey`, `is_toggle_mode`, `retention_policy`, `dictionary`, `snippets`, `theme`, and `launch_at_startup`.

- [ ] **Step 2: Read the current settings before applying the patch**

Clone the old settings, merge only supplied fields, and compare old/new values for `hotkey` and `launch_at_startup`.

- [ ] **Step 3: Validate external changes before committing settings**

If the hotkey changed, register the new shortcut before saving. If registration fails, return the error without persisting the new settings.

- [ ] **Step 4: Call autostart only when the boolean changed**

Do not invoke `reg` when unrelated settings change.

- [ ] **Step 5: Persist and update memory after validation**

Save the merged settings, update `state.settings`, and emit a dedicated `forge://settings-changed` event.

- [ ] **Step 6: Update frontend API callers**

Change theme, model, provider, microphone, dictionary, snippet, and formatting handlers to use the targeted patch command.

- [ ] **Step 7: Add tests**

Cover:

```text
unrelated setting does not invoke hotkey registration
unrelated setting does not invoke autostart operation
failed hotkey registration does not persist new settings
changed hotkey is registered once
```

- [ ] **Step 8: Verify**

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
pnpm --filter @forge-wisper/desktop build
```

> Progress note: the initial settings side-effect optimization is implemented
> and verified in `fix/settings-update-performance`. The complete patch-command
> redesign in this task remains open until all listed steps are implemented.

---

### Task 4: Remove Duplicate Windows Hotkey Handling

**Files:**
- Modify: `apps/desktop/src-tauri/src/lib.rs:28-76,95-98,468-531`
- Test: hotkey parsing and registration unit tests.

**Interfaces:**
- Exactly one hotkey event path is active on Windows.

- [x] **Step 1: Select the canonical implementation**

Use the Tauri global shortcut plugin as the canonical implementation unless runtime testing demonstrates it cannot support the required push-to-talk behavior.

- [x] **Step 2: Disable the native Windows polling listener**

Remove the startup call to `start_native_windows_hotkey_listener`. The native helper remains isolated and unused for now so this focused change does not remove fallback code before real-device validation.

- [x] **Step 3: Add duplicate-event regression coverage**

The canonical event path is now the only path activated by application setup; parser coverage verifies the supported toggle and push-to-talk shortcuts.

- [x] **Step 4: Verify on Windows**

The Windows build/test gate passes with the canonical Tauri shortcut path. Manual physical key testing remains a release validation item for default and custom shortcuts.

---

### Task 5: Replace Overlapping Microphone Polling

**Files:**
- Modify: `apps/desktop/src/views/Dashboard.tsx:181-197`
- Modify: `apps/desktop/src/views/FloatingRecorder.tsx:47-64`
- Modify: `apps/desktop/src/views/SettingsView.tsx:78-108`
- Modify: `apps/desktop/src/lib/tauri.ts`
- Modify: `apps/desktop/src-tauri/src/state.rs` if event emission is implemented.

**Interfaces:**
- Microphone-level updates never overlap.
- UI polling is capped at a deliberate rate, or Rust emits throttled level events.

- [x] **Step 1: Add a non-overlapping polling helper**

Implemented `apps/desktop/src/lib/microphonePolling.ts` using recursive `setTimeout`, cancellation, an immediate first read, and an 80 ms minimum interval.

- [x] **Step 2: Use the helper in Dashboard, FloatingRecorder, and Settings**

Dashboard, FloatingRecorder, and Settings stop polling during effect cleanup or microphone-test cancellation.

- [ ] **Step 3: Prefer throttled Rust events if profiling shows IPC remains expensive**

Sample the atomic/current RMS value from a Rust task and emit `forge://mic-level` no more than 15 times per second.

- [ ] **Step 4: Verify**

Record IPC call counts during a 10-second recording and confirm there are no concurrent in-flight level requests.

---

### Task 6: Centralize Frontend Shared State And Events

**Files:**
- Create: `apps/desktop/src/state/appStore.ts` or follow the existing project state convention.
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/views/Dashboard.tsx`
- Modify: `apps/desktop/src/views/SettingsView.tsx`
- Modify: `apps/desktop/src/views/HistoryView.tsx`
- Modify: `apps/desktop/src/views/ModelManagerView.tsx`
- Modify: `apps/desktop/src/views/DictionaryView.tsx`
- Modify: `apps/desktop/src/lib/tauri.ts`

**Interfaces:**
- One subscription owns processing state and settings changes.
- History reload occurs only after `history-record-created` or explicit history mutation.

- [ ] **Step 1: Define the shared store state**

Include `settings`, `processingState`, `processingError`, and lightweight toast state. Keep view-local form and dropdown state local.

- [ ] **Step 2: Move root subscriptions into the store**

Subscribe once to `forge://state-changed`, `forge://settings-changed`, and history insertion events.

- [ ] **Step 3: Remove duplicate view subscriptions and full reloads**

Dashboard should not reload settings/audio devices/history for every state transition.

- [ ] **Step 4: Verify render and IPC reduction**

Use React DevTools or logging to confirm one state event produces one shared-state update and no duplicate settings fetches.

---

### Task 7: Reduce Native Recorder Window Work

**Files:**
- Modify: `apps/desktop/src-tauri/src/state.rs:153-205`
- Modify: `apps/desktop/src/views/FloatingRecorder.tsx`

**Interfaces:**
- The recorder window is positioned only when it becomes visible.

- [ ] **Step 1: Track previous visibility/state**

Calculate whether the recorder was active before and after the transition.

- [ ] **Step 2: Move monitor lookup and positioning behind inactive-to-active transition**

Do not call `current_monitor` or `set_position` for every processing state.

- [ ] **Step 3: Use CSS animation for processing bars**

Remove `Date.now()` from render and use a CSS keyframe animation for the non-listening processing state.

- [ ] **Step 4: Verify**

Run a full recording pipeline and confirm the floating window is positioned once per session.

---

### Task 8: Improve Audio Buffer Safety And Ownership

**Files:**
- Modify: `crates/audio/src/lib.rs`
- Test: `crates/audio/src/lib.rs` tests or a new focused test module.

**Interfaces:**
- Audio callback performs bounded, low-contention work.
- Stop/encode transfers sample ownership instead of cloning when possible.

- [ ] **Step 1: Replace RMS mutex with atomic storage**

Store the RMS float through `AtomicU32` using `to_bits`/`from_bits`.

- [ ] **Step 2: Preallocate or use a bounded/ring buffer**

Avoid repeated vector growth during normal recording.

- [ ] **Step 3: Transfer the buffer on stop**

Use ownership transfer or `std::mem::take` under the smallest possible lock scope.

- [ ] **Step 4: Preserve NaN sanitization and resampling behavior**

Do not remove the current sanitization safeguards.

- [ ] **Step 5: Verify**

```powershell
cargo test -p forge-audio
cargo clippy -p forge-audio --all-targets -- -D warnings
```

---

### Task 9: Implement Or Explicitly Gate Local Whisper

**Files:**
- Modify: `providers/local-whisper/src/lib.rs`
- Modify: `apps/desktop/src/views/SettingsView.tsx`
- Modify: `apps/desktop/src/views/ModelManagerView.tsx`
- Modify: `README.md`
- Test: provider tests.

**Interfaces:**
- Local provider returns actual transcription, or the UI clearly marks it unavailable.

- [ ] **Step 1: Choose the runtime implementation**

Integrate a real Whisper runtime compatible with the supported model files and target platforms. Do not retain the hardcoded success string.

- [ ] **Step 2: Share one `Arc<ModelManager>`**

Inject the application model manager into `LocalWhisperProvider` rather than constructing a second manager.

- [ ] **Step 3: Add provider tests**

Cover missing model, installed model selection, invalid audio, and successful transcription using a test fixture or mocked runtime boundary.

- [ ] **Step 4: Update product copy**

Ensure README and UI claims match the actual implementation.

- [ ] **Step 5: Verify**

```powershell
cargo test -p forge-provider-local-whisper
cargo clippy -p forge-provider-local-whisper --all-targets -- -D warnings
```

---

### Task 10: Move Dashboard Metrics And History Search To Efficient APIs

**Files:**
- Modify: `crates/storage/src/lib.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src/lib/tauri.ts`
- Modify: `apps/desktop/src/views/Dashboard.tsx`
- Modify: `apps/desktop/src/views/HistoryView.tsx`
- Test: storage and command tests.

**Interfaces:**
- Add `get_dashboard_metrics(timeframe)`.
- Add paginated history query with `offset`, `limit`, and search.

- [ ] **Step 1: Add storage aggregate queries**

Return session count, total duration, and total word count for Today, Week, and All.

- [ ] **Step 2: Add backend commands and TypeScript API functions**

Return only aggregate values for dashboard metrics.

- [ ] **Step 3: Add 250 ms history-search debounce**

Ignore stale responses when a newer query has already started.

- [ ] **Step 4: Replace renderer-side pagination with database pagination**

Preserve existing page sizes and UI behavior.

- [ ] **Step 5: Verify**

```powershell
cargo test -p forge-storage
pnpm --filter @forge-wisper/desktop build
```

---

### Task 11: Use Rust Cleanup Logic For Dictionary Preview

**Files:**
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src/lib/tauri.ts`
- Modify: `apps/desktop/src/views/DictionaryView.tsx`
- Test: cleanup and command tests.

**Interfaces:**
- Add a preview command that accepts text and current cleanup settings and returns cleaned text.

- [ ] **Step 1: Add `preview_cleanup` command**

Construct a `Transcript` and call `RuleBasedCleaner::clean` using the same path as real dictation.

- [ ] **Step 2: Replace frontend regex simulation**

Debounce preview input and call the backend command.

- [ ] **Step 3: Verify parity**

Use the same input in the dictionary sandbox and a cleanup unit test; expected output must match.

---

### Task 12: Full Regression Verification And Branch Handoff

**Files:**
- Modify: `docs/performance-audit.md` with measured results.
- Modify: `docs/superpowers/plans/2026-09-11-forge-wisper-performance-remediation.md` checkboxes.

- [ ] **Step 1: Run complete checks**

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
pnpm --filter @forge-wisper/desktop build
```

- [ ] **Step 2: Run manual Windows scenarios**

Test settings changes, theme changes, microphone changes, hotkey changes, toggle recording, push-to-talk, model download, dictionary preview, history search, and paste output.

- [ ] **Step 3: Capture before/after timings**

Compare settings click-to-visible-update latency, mic-level IPC rate, state-event reload count, and recording pipeline timings.

- [ ] **Step 4: Review the diff and create a feature branch**

Create the branch only after the documentation and baseline are reviewed:

```powershell
git status --short
```
