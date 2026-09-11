# Local Whisper GPU Acceleration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an explicit CPU/GPU compute-device choice for Local Whisper, with CPU as the default and a real Windows Vulkan backend for GPU mode.

**Architecture:** Persist `compute_device` as `cpu` or `gpu` in application settings. The Local Whisper provider resolves the requested device through a backend capability layer, constructs Whisper contexts with GPU enabled only when the Vulkan-enabled native build and runtime device are available, and reports the actual backend. Explicit GPU mode fails clearly when unavailable; CPU mode never attempts GPU.

**Tech Stack:** Tauri 2, Rust 2021, React, TypeScript, `whisper-rs`/whisper.cpp or a compatible Vulkan-capable binding, CMake, Vulkan SDK, tracing, Vite.

## Global Constraints

- CPU remains the default and must continue to work without a GPU or Vulkan SDK at runtime.
- GPU mode must not silently fall back to CPU.
- Do not expose CUDA/Vulkan/DirectML as normal user settings; expose only CPU and GPU.
- Windows x64 is the first GPU target; Intel/AMD/NVIDIA GPU support is through Vulkan where drivers support it.
- Do not commit model weights or generated build output.
- Preserve the existing macOS build scripts and CPU-only behavior unless a platform-specific backend is explicitly added later.
- Every user-visible update bumps all synchronized application versions and updates `CHANGELOG.md`.

---

### Task 1: Define and persist compute-device settings

**Files:**
- Modify: `apps/desktop/src/types/index.ts`
- Modify: `apps/desktop/src-tauri/src/state.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src/views/SettingsView.tsx`
- Test: `apps/desktop/src-tauri/src/state.rs` existing settings tests

**Interfaces:**
- Produces `AppSettings.compute_device: "cpu" | "gpu"` in TypeScript and Rust.
- Produces a persisted default of `cpu` for new and legacy settings.

- [ ] Add the typed `ComputeDevice` union and `compute_device` field in the frontend settings type.
- [ ] Add the matching Rust enum/string field with serde default logic so missing legacy values deserialize as `cpu`.
- [ ] Add the setting to the settings UI with labels `CPU` and `GPU`, explanatory copy, and an unavailable/unsupported state supplied by the backend later.
- [ ] Add a Rust regression test proving legacy settings without `compute_device` resolve to `cpu`.
- [ ] Run `cargo test -p forge-desktop-app` and the frontend build; commit as `feat: add local whisper compute device setting`.

### Task 2: Add a backend capability contract

**Files:**
- Modify: `providers/local-whisper/Cargo.toml`
- Modify: `providers/local-whisper/src/lib.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src/lib/tauri.ts`
- Modify: `apps/desktop/src/types/index.ts`

**Interfaces:**
- Produces `LocalComputeDeviceInfo { requested_device, active_backend, gpu_available, gpu_name, reason }`.
- Produces Tauri command `get_local_compute_device_info`.

- [ ] Write tests for CPU selection and explicit GPU-unavailable error using the backend capability contract.
- [ ] Run the provider tests and verify the new tests fail before implementation.
- [ ] Implement a capability struct that reports CPU unconditionally and GPU capability from the compiled native backend/runtime.
- [ ] Implement the Tauri command and typed frontend wrapper.
- [ ] Log the selected device and actual backend without logging sensitive data.
- [ ] Run provider tests, workspace tests, Clippy, and frontend build; commit as `feat: expose local compute capabilities`.

### Task 3: Enable the Windows Vulkan native backend

**Files:**
- Modify: `providers/local-whisper/Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `scripts/dev-windows.ps1`
- Modify: `docs/local-whisper-validation.md`
- Modify: `README.md` if Windows developer prerequisites change

**Interfaces:**
- Produces a Windows x64 build that links a Vulkan-capable Whisper backend.
- Produces explicit build diagnostics when `VULKAN_SDK`, `glslc`, or Vulkan headers are unavailable.

- [ ] Confirm the exact `whisper-rs`/whisper.cpp feature supported by the locked dependency version; do not guess feature names.
- [ ] Add the smallest Windows-targeted dependency feature configuration that enables Vulkan while preserving CPU builds on other targets.
- [ ] Extend the Windows launcher to validate `VULKAN_SDK` and the shader compiler before invoking Cargo.
- [ ] Run a clean Windows development compile and capture the native backend initialization output.
- [ ] Document the Vulkan SDK requirement and the expected native build/runtime messages.
- [ ] Commit as `build: enable windows whisper vulkan backend` only after the backend compiles.

### Task 4: Implement CPU/GPU runtime selection

**Files:**
- Modify: `providers/local-whisper/src/lib.rs`
- Modify: `apps/desktop/src-tauri/src/state.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src/views/SettingsView.tsx`

**Interfaces:**
- `compute_device = "cpu"` constructs CPU Whisper contexts.
- `compute_device = "gpu"` constructs GPU-enabled contexts or returns a typed actionable error.

- [ ] Write a failing provider test that explicit GPU mode does not silently return a CPU backend.
- [ ] Implement context-parameter selection for CPU and Vulkan GPU modes using the verified binding API.
- [ ] Keep GPU initialization outside the audio callback and inside the existing blocking inference boundary.
- [ ] Add logs for `requested_device`, `active_backend`, `gpu_name`, and `gpu_acceleration`.
- [ ] Update the settings UI to show actual active backend and actionable GPU errors.
- [ ] Run focused provider tests and verify CPU mode remains unchanged.
- [ ] Commit as `feat: select local whisper compute backend`.

### Task 5: Validate on NVIDIA, Intel, and CPU fallback paths

**Files:**
- Modify: `docs/local-whisper-validation.md`
- Modify: `docs/superpowers/plans/2026-09-11-local-whisper-gpu-acceleration.md`

- [ ] Run CPU mode with the Tiny or Base model and record transcript, elapsed time, CPU usage, and logs.
- [ ] Run GPU mode with the same model and identical speech fixture; confirm logs report Vulkan and GPU acceleration.
- [ ] Confirm Windows Task Manager shows GPU compute/memory activity during GPU inference.
- [ ] Confirm explicit GPU mode fails clearly when Vulkan is disabled or unavailable.
- [ ] Confirm CPU mode still succeeds after GPU failure.
- [ ] Compare transcript correctness and timing; treat GPU as a performance/backend change, not an accuracy change.
- [ ] Run `cargo fmt --all -- --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `pnpm --filter @forge-wisper/desktop build`.
- [ ] Bump synchronized versions and changelog for the completed user-visible feature; commit as `test: validate local whisper gpu backends`.
