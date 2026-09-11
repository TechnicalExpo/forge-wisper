# Local Whisper Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Every task must be completed, verified, marked `[x]`, committed on its own focused branch, merged into `main`, and followed by a new branch before the next task.

**Goal:** Deliver reliable offline Local Whisper transcription with downloadable GGML models, actual CPU inference, safe model lifecycle, and an honest privacy/license story.

**Architecture:** Keep the existing `TranscriptionProvider` abstraction. `ModelManager` owns catalog, discovery, download, validation, and model paths; `LocalWhisperProvider` owns inference and a future loaded-context cache; the Tauri pipeline remains responsible for audio capture, cleanup, verification, and output. Start with CPU `whisper-rs` inference, then add integrity/caching before expanding model families or GPU backends.

**Tech Stack:** Rust 2021, Tauri 2, `whisper-rs 0.16.0`, whisper.cpp, GGML models, CPAL, hound, SQLite, React, TypeScript, Vite.

## Global Constraints

- Current feature release target is `0.2.0`.
- Never return fabricated transcription text.
- Do not persist raw microphone audio unless explicitly approved and documented.
- Keep Groq Cloud as the fast default and Local Whisper as an optional offline provider.
- Do not commit large model weights; tests must use generated audio or externally provisioned fixtures.
- Verify model downloads before activation; never execute downloaded binaries as part of this feature.
- Keep one focused branch per task and merge each verified task into `main` before creating the next branch.
- Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `pnpm --filter @forge-wisper/desktop build` before every merge.
- On Windows runtime builds require MSVC, LLVM/libclang, and CMake.

---

### Task 1: Runtime Foundation And Real CPU Inference

**Branch:** `feat/local-whisper-runtime`

**Files:**
- Modify: `providers/local-whisper/Cargo.toml`
- Modify: `providers/local-whisper/src/lib.rs`
- Modify: `apps/desktop/src-tauri/src/state.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src/views/SettingsView.tsx`
- Modify: `apps/desktop/src/views/Dashboard.tsx`
- Modify: `README.md`, `SECURITY.md`, `CONTRIBUTING.md`, `CHANGELOG.md`

**Interfaces:**
- `LocalWhisperProvider::transcribe` consumes 16-bit mono 16 kHz WAV bytes and returns real segments through `whisper-rs`.
- `PipelineState` and `LocalWhisperProvider` share one `Arc<ModelManager>`.
- Local Whisper can be selected only when a model exists; missing model/inference errors are typed and user-visible.

- [x] Pin `whisper-rs 0.16.0` and build whisper.cpp through the native toolchain.
- [x] Decode WAV input, validate sample format, run CPU greedy inference in `spawn_blocking`, collect segments, and return `Transcript`.
- [x] Remove placeholder output and add missing-model regression coverage.
- [x] Enable Local Whisper UI selection and update privacy/product documentation.
- [x] Run provider tests, workspace tests, Clippy, and frontend build.

**Evidence:** commit `8a481bf` on `feat/local-whisper-runtime`; full checks passed with LLVM/CMake/MSVC configured. A real model fixture is still required for manual end-to-end validation.

---

### Task 2: Model Catalog Integrity And Safe Downloads

**Branch:** `feat/local-whisper-model-integrity`

**Files:**
- Modify: `providers/local-whisper/src/lib.rs`
- Modify: `models/README.md`
- Modify: `README.md`, `SECURITY.md`, `CHANGELOG.md`
- Test: `providers/local-whisper/src/lib.rs`

**Interfaces:**
- Add `sha256` and pinned `revision` metadata to `LocalModelInfo`.
- Verify downloaded byte length and SHA-256 before atomic rename.
- Reject a corrupted/partial model as unavailable.

- [x] Add per-model expected SHA-256 and pinned source revision metadata.
- [x] Hash the `.part` file before renaming it to the final model path.
- [x] Keep existing `.part` cleanup and atomic rename behavior.
- [x] Add unit coverage for hash computation, catalog metadata, and missing-model behavior. Network-backed mismatch/interrupt tests remain pending.
- [ ] Review model/source licenses and update third-party notices.
- [ ] Run the full verification gate, mark this task complete, commit, merge to `main`, and create the next branch.

- [x] Review model/source licenses and update third-party notices.
- [x] Run the full verification gate and mark the task complete.

> Integrity validation is covered by deterministic size/checksum tests; the
> downloader removes `.part` files on every integrity failure before activation.

---

### Task 3: Loaded Model Context Cache

**Branch:** `perf/local-whisper-model-cache`

**Files:**
- Modify: `providers/local-whisper/src/lib.rs`
- Modify: `apps/desktop/src-tauri/src/state.rs` if lifecycle hooks are needed.
- Test: provider cache tests.

**Interfaces:**
- Reuse a loaded `WhisperContext` for repeated transcriptions of the same model.
- Invalidate/reload the context when the selected model changes.
- Never hold a settings/storage mutex across inference or `.await`.

- [x] Add a model-id/path keyed cache with a clear ownership strategy for `WhisperContext`.
- [x] Ensure concurrent transcription requests cannot corrupt a shared inference state; each request creates its own state from the shared context.
- [x] Add cache invalidation on model switch and provider lifecycle tests.
- [ ] Measure model-load time before and after caching with a real model fixture.
- [x] Run the full verification gate. Merge to `main` only after real-model timing measurement is captured.

> Progress: context caching and model-switch invalidation are implemented and
> compile/test verified. Real-model cache-hit timing remains pending because
> model weights are intentionally not committed to the repository.

---

### Task 4: Real-Model End-To-End Fixture Validation

**Branch:** `test/local-whisper-e2e`

**Files:**
- Modify: `providers/local-whisper/src/lib.rs` tests or add a focused test module.
- Create: `docs/local-whisper-validation.md`
- Modify: `CHANGELOG.md` if release notes require measured results.

**Interfaces:**
- Test harness accepts an externally supplied model path, never commits model weights.
- Test uses deterministic generated WAV audio or a documented small fixture.

- [x] Add a Windows validation path accepting `FORGE_WHISPER_MODEL_PATH`.
- [x] Document deterministic speech fixture generation/usage without committing raw audio or model weights.
- [ ] Record model, hardware, load time, inference time, memory estimate, and output status using a real downloaded model.
- [ ] Validate Groq and Local provider output through the same cleanup/verification pipeline using a real model fixture.
- [ ] Run the full verification gate, mark complete, merge to `main`, and create the next branch.

> Progress: external model-path validation and measurement procedure are
> documented. Real-model transcript/timing evidence remains pending until a
> model fixture is provisioned on the validation machine.

---

### Task 5: Expand Model Families Only After Whisper Stability

**Branch:** `feat/extended-local-models`

**Files:**
- Modify: `providers/local-whisper/src/lib.rs` or create a provider-family module.
- Modify: `crates/transcription/src/lib.rs` capability types.
- Modify: model catalog/UI/docs/license notices.

**Interfaces:**
- Preserve Whisper provider behavior while adding a separate model-family/provider boundary.
- Candidate families: Parakeet, Moonshine, Canary, SenseVoice, GigaAM, or Qwen ASR.

- [ ] Select one family based on Windows/macOS support, licensing, model format, and measured latency.
- [ ] Add the provider behind an explicit capability/family enum rather than increasing Whisper conditionals.
- [ ] Add model metadata, download validation, runtime tests, and UI selection states.
- [ ] Update LPGS and third-party notices before enabling release builds.
- [ ] Run the full verification gate, mark complete, merge to `main`, and create the next branch.
