# Forge Wisper Performance Audit

**Audit date:** 2026-09-12\
**Scope:** React/Vite frontend, Tauri commands, Rust pipeline, audio capture, storage, providers, and output injection.

## Executive Summary

Forge Wisper uses an appropriate technology stack for a cross-platform desktop dictation application. The lag is not evidence that Rust, React, TypeScript, or Tauri are inherently unsuitable. The strongest causes are application-level:

The original performance risks have been remediated in focused tasks. Remaining
work is limited to optional profiling and manual release validation on physical
Windows input/output devices.

Local Whisper now runs real whisper.cpp inference, supports CPU and Vulkan GPU
backends, detects the NVIDIA RTX 3070 on the validation machine, and caches
model contexts by model path/backend/device.

## Evidence And Verification

### Completed checks

The following frontend checks passed:

```text
pnpm --filter @forge-wisper/desktop build
TypeScript compilation: passed
Vite production build: passed
1593 modules transformed
JavaScript: 279.45 kB
CSS: 33.81 kB
Build time: 4.62s
```

The architecture and bundle analyzer scripts completed without findings, but their output is not sufficient to replace runtime profiling.

### Current verification

Rustup, MSVC, LLVM/libclang, CMake, and the Vulkan SDK are configured through
the Windows launcher. The launcher places Microsoft's linker first and uses a
short Cargo target directory to avoid the nested Vulkan shader path limit.

```text
cargo test --workspace                         -> passed
cargo clippy --workspace --all-targets -D warnings -> passed
pnpm --filter @forge-wisper/desktop build      -> passed
```

The linker itself was verified as:

```text
C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\link.exe
```

Once the policy allows generated Rust build scripts to execute, run:

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Findings

### Resolved: Settings updates perform unrelated synchronous work

**Files:**

- `apps/desktop/src-tauri/src/commands.rs:46-68`
- `apps/desktop/src-tauri/src/lib.rs:166-204`
- `apps/desktop/src-tauri/src/lib.rs:341-375`

`update_settings` always writes the complete JSON settings file, calls Windows `reg`, unregisters all global shortcuts, and registers several shortcut variants. This happens even when the user only changes theme, formatting mode, microphone, dictionary, or snippets.

**Resolution:** `SettingsPatch` updates only supplied fields, and OS integration
work runs only when the corresponding setting changes. External validation runs
before settings are persisted, with rollback on failure.

**Recommendation:** implement field-specific updates or a settings patch command. Only modify autostart when `launch_at_startup` changes. Only re-register hotkeys when `hotkey` changes. Persist after validation succeeds.

### Resolved: Duplicate Windows hotkey systems

**File:** `apps/desktop/src-tauri/src/lib.rs`

The Tauri global shortcut plugin is registered and `start_native_windows_hotkey_listener` also starts a 15 ms Windows polling loop.

**Resolution:** the native Windows listener is no longer started alongside the
canonical Tauri path for supported shortcuts.

**Recommendation:** keep one implementation per platform. Do not register the same shortcut through both systems.

### Resolved: Local Whisper placeholder output

**File:** `providers/local-whisper/src/lib.rs:502-539`

The provider checks model availability but returns:

```rust
The provider now loads a compatible GGML model and runs CPU inference through `whisper-rs`.
```

**Resolution:** Placeholder output was removed; missing models and inference failures return typed provider errors.

**Recommendation:** integrate an actual Whisper runtime or mark the feature unavailable until implemented.

### Resolved: Overlapping microphone IPC polling

**Files:**

- `apps/desktop/src/views/Dashboard.tsx:181-197`
- `apps/desktop/src/views/FloatingRecorder.tsx:47-64`
- `apps/desktop/src/views/SettingsView.tsx:78-108`

`setInterval(async () => ...)` starts a new invocation without waiting for the previous `get_mic_level` call. Dashboard and recorder polling can run at the same time.

**Resolution:** views use a cancellable recursive timeout helper with an 80 ms
minimum interval and no overlapping requests.

**Recommendation:** use a non-overlapping recursive timeout or emit throttled audio-level events from Rust.

### Resolved: Processing events trigger redundant reloads

**Files:**

- `apps/desktop/src/App.tsx:60-75`
- `apps/desktop/src/views/Dashboard.tsx:62-94`

The root app reloads settings on every state event. Dashboard also reloads settings, history, processing state, and audio devices on terminal states.

**Resolution:** the shared application store owns processing state and settings;
Dashboard refreshes terminal data through targeted APIs only.

**Recommendation:** centralize frontend state and emit separate events for processing state, inserted history, and settings changes.

### Resolved: Recorder window is repositioned for every state

**File:** `apps/desktop/src-tauri/src/state.rs:153-205`

`set_state` queries the monitor, calculates position, moves the window, and shows it for all active pipeline states.

**Resolution:** monitor lookup and positioning occur on inactive-to-active
transitions rather than every processing state.

**Recommendation:** position/show only on inactive-to-active transition; update content/state without repeated repositioning.

### Resolved: Audio callback uses mutex-protected dynamic buffer

**File:** `crates/audio/src/lib.rs:58-64, 120-173`

The real-time callback locks a `Mutex<Vec<f32>>`, appends samples, and updates RMS under another mutex.

**Resolution:** RMS uses atomic storage, the sample buffer is preallocated, and
stop transfers ownership instead of cloning the complete recording.

**Recommendation:** use a bounded/ring buffer, preallocate where possible, and store RMS in an atomic value.

### P1: Audio data is copied repeatedly on stop

**File:** `crates/audio/src/lib.rs:201-260`

Stopping clones the full buffer and then creates additional mono, normalized, resampled, and WAV representations.

**Impact:** extra allocations and latency for long recordings.

**Recommendation:** transfer ownership of the sample buffer and reduce intermediate allocations.

### Resolved: Settings can be persisted even when hotkey registration fails

**File:** `apps/desktop/src-tauri/src/commands.rs:51-67`

Settings are written and installed in memory before hotkey registration is attempted.

**Resolution:** hotkey/autostart validation completes before persistence and the
previous hotkey is restored if a later external operation fails.

**Recommendation:** validate and register external resources first, then commit settings; define rollback behavior.

### P1: Autostart is forcibly enabled during startup

**File:** `apps/desktop/src-tauri/src/lib.rs:78-89`

Startup calls `set_autostart(true)` regardless of persisted user preference.

**Impact:** the UI setting does not represent actual behavior.

**Recommendation:** apply the persisted `launch_at_startup` value.

### Resolved: Dashboard loads and computes metrics in the renderer

**File:** `apps/desktop/src/views/Dashboard.tsx:142-159`

The dashboard loads up to 100 full history records and calculates word counts and metrics in React.

**Resolution:** SQLite returns aggregate dashboard metrics directly.

**Recommendation:** add a backend aggregate command and database-level pagination.

### Resolved: History search runs on every keystroke

**File:** `apps/desktop/src/views/HistoryView.tsx:24-39`

Every character invokes SQLite search.

**Resolution:** HistoryView debounces search by 250 ms, uses database pagination,
and ignores stale requests.

### Resolved: Dictionary sandbox duplicates Rust cleanup logic

**File:** `apps/desktop/src/views/DictionaryView.tsx:162-189`

The frontend recompiles regexes for every dictionary/snippet item and has behavior separate from `RuleBasedCleaner`.

**Resolution:** DictionaryView calls `preview_cleanup`, which uses the same Rust
cleanup engine as real transcription.

### Resolved: Duplicate model manager instances

**Files:**

- `apps/desktop/src-tauri/src/state.rs:129-148`
- `providers/local-whisper/src/lib.rs:441-450`

`PipelineState` owns one `ModelManager`, while `LocalWhisperProvider` constructs another.

**Resolution:** PipelineState and LocalWhisperProvider share one `Arc<ModelManager>`.

## Strengths

- Clear Rust crate boundaries.
- Clean transcription provider abstraction.
- OS keyring use for Groq credentials.
- Clipboard fallback when simulated paste fails.
- SQLite WAL mode and busy timeout configured.
- Static cleanup regexes are reused.
- Strict TypeScript configuration.
- Frontend production build is currently healthy.

## Target Architecture

```text
Rust domain state and services
        |
        | typed Tauri commands/events
        v
Single frontend application store
        |
        +-- Dashboard
        +-- Settings
        +-- History
        +-- Model Manager
        +-- Dictionary
```

The frontend should consume shared settings and processing state instead of having each view independently fetch and refresh the same information.

## Instrumentation Required Before Optimization

Add timing logs around:

- settings update
- autostart registry operation
- hotkey registration
- audio start
- audio encoding
- transcription
- cleanup
- verification
- history insert
- paste
- history list
- audio-device enumeration
- microphone-level reads

Each record should include operation, duration, provider, model, recording duration, payload size, result, and error.

## Recommended Priority

1. Fix settings update side effects and transactional behavior.
2. Remove duplicate Windows hotkey handling.
3. Replace overlapping microphone polling.
4. Centralize processing/settings events.
5. Avoid repeated recorder-window repositioning.
6. Improve audio buffer ownership and real-time callback safety.
7. Implement real Local Whisper transcription.
8. Move metrics/search/pagination work into efficient backend APIs.
