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

## Completion Status

Implemented and validated on Windows with an NVIDIA RTX 3070 Laptop GPU.

- CPU mode remains the default and uses `active_backend=cpu`.
- GPU mode uses `active_backend=vulkan` and `gpu_acceleration=true`.
- Vulkan detected both Intel UHD Graphics and NVIDIA GeForce RTX 3070 Laptop GPU.
- GPU mode selected the discrete NVIDIA device and native Whisper reported `use gpu = 1` and `using Vulkan1 backend`.
- CPU and GPU transcription both completed successfully and inserted output.
- Workspace tests, Clippy, focused provider tests, and the frontend build passed.
- A fair same-audio benchmark was not recorded; the manual validation confirmed backend selection and successful operation.

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

- [x] Add the typed `ComputeDevice` union and `compute_device` field in the frontend settings type.
- [x] Add the matching Rust string field with serde default logic so missing legacy values deserialize as `cpu`.
- [x] Add the setting to the settings UI with labels `CPU` and `GPU`, explanatory copy, and an unavailable/unsupported state supplied by the backend.
- [x] Add regression coverage for CPU default behavior.
- [x] Run the Rust and frontend verification gates.

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

- [x] Write tests for CPU selection and GPU capability reporting using the backend capability contract.
- [x] Implement a capability struct that reports CPU unconditionally and GPU capability from the compiled native backend/runtime.
- [x] Implement the Tauri command and typed frontend wrapper.
- [x] Log the selected device and actual backend without logging sensitive data.
- [x] Run provider tests, workspace tests, Clippy, and frontend build.

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

- [x] Confirm the exact `whisper-rs`/whisper.cpp Vulkan feature supported by the locked dependency version.
- [x] Add the smallest Windows-targeted dependency feature configuration that enables Vulkan while preserving CPU builds on other targets.
- [x] Extend the Windows launcher to validate `VULKAN_SDK` and the shader compiler before invoking Cargo.
- [x] Run a clean Windows development compile and capture the native backend initialization output.
- [x] Document the Vulkan SDK requirement and the expected native build/runtime messages.

### Task 4: Implement CPU/GPU runtime selection

**Files:**
- Modify: `providers/local-whisper/src/lib.rs`
- Modify: `apps/desktop/src-tauri/src/state.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src/views/SettingsView.tsx`

**Interfaces:**
- `compute_device = "cpu"` constructs CPU Whisper contexts.
- `compute_device = "gpu"` constructs GPU-enabled contexts or returns a typed actionable error.

- [x] Add provider coverage that explicit GPU mode does not silently return a CPU backend.
- [x] Implement context-parameter selection for CPU and Vulkan GPU modes using the verified binding API.
- [x] Keep GPU initialization outside the audio callback and inside the existing blocking inference boundary.
- [x] Add logs for `requested_device`, `active_backend`, `gpu_name`, and `gpu_acceleration`.
- [x] Update the settings UI to show actual active backend and actionable GPU errors.
- [x] Run focused provider tests and verify CPU mode remains unchanged.

### Task 5: Validate on NVIDIA, Intel, and CPU fallback paths

**Files:**
- Modify: `docs/local-whisper-validation.md`
- Modify: `docs/superpowers/plans/2026-09-11-local-whisper-gpu-acceleration.md`

- [x] Run CPU mode with the Base model and confirm transcript, elapsed time, CPU backend, and logs.
- [x] Run GPU mode with the Base model and confirm transcript, Vulkan backend, GPU acceleration, and native `use gpu = 1` output.
- [x] Confirm Vulkan enumerates the Intel UHD and NVIDIA RTX 3070 devices, with the discrete NVIDIA device selected for GPU mode.
- [x] Confirm CPU mode still succeeds after GPU mode.
- [x] Confirm transcript insertion and verification succeeded in both modes.
- [x] Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `pnpm --filter @forge-wisper/desktop build`.
- [x] Bump synchronized versions to `0.3.0` and update the changelog.
- [x] Record a controlled same-audio CPU/GPU benchmark with cold and warm runs.

> Benchmark evidence is recorded in `docs/local-whisper-validation.md`.
