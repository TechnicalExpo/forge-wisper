# Parakeet V3 Model Family Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Parakeet V3 Int8 as the first non-Whisper local model family without changing existing Whisper CPU/Vulkan behavior or the safe transcription pipeline.

**Architecture:** Keep `LocalWhisperProvider` and its GGML/Vulkan cache unchanged. Add a separate Parakeet provider using `transcribe-rs 0.2.9` and ONNX CPU inference, with explicit family metadata and a directory-model manifest owned by the model manager. Route the selected local family through the existing provider abstraction and preserve cleanup, verification, history, and output injection.

**Tech Stack:** Rust 2021, Tauri 2, `transcribe-rs 0.2.9`, ONNX Runtime CPU, existing `TranscriptionProvider`, React, TypeScript, Vite, SHA-256 model validation.

## Global Constraints

- Do not modify Whisper inference, CPU/Vulkan backend selection, or Whisper context-cache behavior except where a shared type must be extended.
- Do not commit model weights, raw audio, API keys, or benchmark transcript data.
- Parakeet V3 Int8 is downloaded on demand and is not bundled in the installer.
- Parakeet language behavior is automatic detection; manual language selection remains for Whisper/Groq.
- Model archives are staged, validated, and atomically promoted; partial installations are never selectable.
- Do not log raw audio, transcript text, API keys, or model secrets.
- Do not hold settings/storage mutexes across inference or `.await` points.
- Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `pnpm --filter @forge-wisper/desktop build`, and `cargo fmt --all -- --check` before merge.
- Preserve the existing release version until implementation is complete; this feature uses synchronized version `0.4.0` across all manifests and changelog.

---

### Task 1: Add Explicit Model-Family Types

**Files:**
- Modify: `crates/transcription/src/lib.rs`
- Modify: `providers/local-whisper/src/lib.rs`
- Test: `crates/transcription/src/lib.rs`

**Interfaces:**
- Produce serializable `ModelFamily` and `ModelFormat` values for `whisper` and `parakeet`.
- Extend shared model capability metadata without changing existing Whisper model IDs or provider IDs.

- [x] **Step 1: Write failing serialization and defaulting tests**

Add tests asserting that `ModelFamily::Whisper` serializes as `"whisper"`, `ModelFamily::Parakeet` serializes as `"parakeet"`, and legacy metadata defaults to Whisper when the family field is absent.

- [x] **Step 2: Run the focused tests and verify failure**

Run:

```powershell
cargo test -p forge-transcription model_family
```

Expected: compile/test failure because the new family types are not defined.

- [x] **Step 3: Add the shared family and format types**

Implement serde-compatible enums with explicit lowercase names and `Default` for `ModelFamily::Whisper`. Add family/format fields to model metadata types using `#[serde(default)]` where persisted legacy data requires compatibility.

- [x] **Step 4: Run the focused tests and workspace formatting**

Run:

```powershell
cargo test -p forge-transcription model_family
cargo fmt --all -- --check
```

Expected: tests pass and formatting is clean.

- [x] **Step 5: Commit the shared contract**

```powershell
git add crates/transcription/src/lib.rs providers/local-whisper/src/lib.rs
git commit -m "feat: add local model family metadata"
```

---

### Task 2: Add Parakeet Provider Runtime

**Files:**
- Modify: `providers/local-whisper/Cargo.toml`
- Create: `providers/local-whisper/src/parakeet.rs`
- Modify: `providers/local-whisper/src/lib.rs`
- Test: `providers/local-whisper/src/parakeet.rs`

**Interfaces:**
- Produce `LocalParakeetProvider` implementing `TranscriptionProvider`.
- Consume `AudioData` and a validated model directory path.
- Return `Transcript { provider: "local-parakeet", model: "parakeet-v3-int8", confidence: None }`.

- [ ] **Step 1: Add the pinned runtime dependency**

Add `transcribe-rs = { version = "0.2.9", default-features = false, features = ["parakeet"] }` to the local provider crate. Do not enable unrelated model-family features.

- [ ] **Step 2: Write provider tests for missing/incomplete model setup**

Add tests that construct a provider with a temporary model directory and assert that missing required files return `ProviderError::ModelError`, while empty audio returns `ProviderError::InvalidAudio`.

- [ ] **Step 3: Implement WAV decoding and Parakeet inference**

Decode the existing 16-bit mono 16 kHz WAV contract, normalize samples to `f32`, load `ParakeetEngine` with `ParakeetModelParams::int8()`, and execute loading/inference inside `spawn_blocking`. Convert runtime errors into typed provider errors without including audio or transcript content in error text.

- [ ] **Step 4: Add model-load synchronization and cache ownership**

Use a provider-owned mutex/cache that stores the loaded Parakeet engine for one model path at a time. Create a fresh inference operation per request and invalidate the cache when the model path changes. Never hold the cache lock across async waits.

- [ ] **Step 5: Run provider tests and compile checks**

Run:

```powershell
cargo test -p forge-provider-local-whisper parakeet
cargo check -p forge-provider-local-whisper
```

Expected: focused tests pass and the provider compiles with `transcribe-rs 0.2.9`.

- [ ] **Step 6: Commit the provider runtime**

```powershell
git add providers/local-whisper/Cargo.toml providers/local-whisper/src/lib.rs providers/local-whisper/src/parakeet.rs Cargo.lock
git commit -m "feat: add parakeet local provider"
```

---

### Task 3: Add Parakeet Directory Catalog And Validation

**Files:**
- Modify: `providers/local-whisper/src/lib.rs`
- Create: `providers/local-whisper/src/model_catalog.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src-tauri/src/state.rs`
- Test: `providers/local-whisper/src/model_catalog.rs`

**Interfaces:**
- Produce `LocalModelInfo` entries for Whisper files and Parakeet directory models.
- Produce `download_model`, `delete_model`, and `list_available_models` behavior compatible with existing Tauri commands.
- Validate Parakeet files: `encoder-model.int8.onnx`, `decoder_joint-model.int8.onnx`, `nemo128.onnx`, and `vocab.txt`.

- [ ] **Step 1: Write validation tests**

Create temporary directories for complete, missing-file, empty-file, and staged-partial installations. Assert only the complete directory is considered installed and invalid directories are rejected before provider selection.

- [ ] **Step 2: Implement catalog metadata**

Move catalog construction into the focused catalog module where practical. Preserve every current Whisper entry exactly and add one Parakeet V3 Int8 entry with family, directory format, source URL, pinned revision, required files, expected archive checksum, and model license/source metadata.

- [ ] **Step 3: Implement staged archive extraction and validation**

Download the Parakeet archive to a `.part` file, validate its byte count and SHA-256, extract into a temporary sibling directory, validate required relative paths and non-zero file sizes, then rename atomically into the models directory. Remove all temporary files/directories on failure.

- [ ] **Step 4: Wire model commands and active-model resolution**

Ensure existing Whisper download/delete behavior remains unchanged. Add family-aware resolution so a selected Parakeet directory maps to `LocalParakeetProvider`, while legacy settings resolve to Whisper.

- [ ] **Step 5: Run catalog and command tests**

Run:

```powershell
cargo test -p forge-provider-local-whisper model_catalog
cargo test -p forge-desktop-app model
```

Expected: validation, cleanup, and legacy Whisper catalog tests pass.

- [ ] **Step 6: Commit catalog and lifecycle support**

```powershell
git add providers/local-whisper/src apps/desktop/src-tauri/src/commands.rs apps/desktop/src-tauri/src/state.rs Cargo.lock
git commit -m "feat: add validated parakeet model catalog"
```

---

### Task 4: Wire Settings And Provider Selection

**Files:**
- Modify: `apps/desktop/src-tauri/src/state.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src/lib/tauri.ts`
- Modify: `apps/desktop/src/views/SettingsView.tsx`
- Modify: `apps/desktop/src/views/ModelManagerView.tsx`
- Test: `apps/desktop/src-tauri/src/state.rs`

**Interfaces:**
- Persist `local_model_family` with Whisper as the backward-compatible default.
- Keep existing model IDs and settings patch behavior intact.
- Expose family-aware model list/download/select commands to TypeScript.

- [ ] **Step 1: Add settings migration tests**

Assert that settings without `local_model_family` deserialize as Whisper, Parakeet selection is accepted only with the Parakeet family, and switching family invalidates only the relevant local runtime selection.

- [ ] **Step 2: Implement persisted family selection**

Add the minimal settings field and targeted patch handling. Do not replace partial settings updates with full settings writes. Keep language selection unchanged and document that Parakeet uses automatic detection.

- [ ] **Step 3: Route the pipeline to the selected local provider**

Instantiate and register `LocalParakeetProvider` alongside the existing providers. Route local transcription by explicit family, return a user-visible typed error if the selected model is unavailable, and preserve cleanup/verification/output behavior.

- [ ] **Step 4: Add typed frontend IPC wrappers**

Extend `src/lib/tauri.ts` with typed family/model fields and commands. Keep renderer business logic out of presentational components.

- [ ] **Step 5: Update model/settings UI**

Group models into Whisper and Parakeet sections. Show Parakeet’s Int8/CPU/automatic-language behavior, installation state, progress, and selection state. Disable selection until validation succeeds. Preserve keyboard accessibility and responsive layout.

- [ ] **Step 6: Run frontend and Rust checks**

Run:

```powershell
cargo test -p forge-desktop-app settings
pnpm --filter @forge-wisper/desktop build
```

- [ ] **Step 7: Commit selection and UI wiring**

```powershell
git add apps/desktop/src-tauri/src apps/desktop/src/lib/tauri.ts apps/desktop/src/views/SettingsView.tsx apps/desktop/src/views/ModelManagerView.tsx
git commit -m "feat: expose parakeet model selection"
```

---

### Task 5: Licensing, Privacy, Documentation, And Release Metadata

**Files:**
- Modify: `README.md`
- Modify: `SECURITY.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/local-whisper-validation.md`
- Modify: `models/README.md`
- Create or modify: `THIRD_PARTY_NOTICES.md`
- Modify: `package.json`, `Cargo.toml`, `apps/desktop/package.json`, `apps/desktop/src-tauri/tauri.conf.json`

**Interfaces:**
- Document Parakeet’s local-only data flow, model download behavior, supported platform/backend, source, and license.
- Keep version values synchronized and use the next patch/minor version required by the user-visible feature.

- [ ] **Step 1: Add model and privacy documentation**

Document the Parakeet model directory requirements, automatic language detection, CPU-only ONNX execution, download-on-demand behavior, and external fixture variable `FORGE_PARAKEET_MODEL_PATH`.

- [ ] **Step 2: Add third-party and model notices**

Record `transcribe-rs` MIT, ONNX Runtime licensing/redistribution requirements, and the Parakeet model’s exact source/license/attribution after verifying the upstream artifact metadata. Do not claim redistribution rights without source evidence.

- [ ] **Step 3: Update changelog and synchronized versions**

Add the Parakeet feature under the next release section and update all four application manifests consistently. Do not change dependency versions solely for the app version bump.

- [ ] **Step 4: Run documentation consistency checks**

Run:

```powershell
git diff --check
git grep -n "0.3.20" -- package.json Cargo.toml apps/desktop/package.json apps/desktop/src-tauri/tauri.conf.json
```

Expected: no stale application version remains in the four synchronized manifests.

- [ ] **Step 5: Commit release documentation**

```powershell
git add README.md SECURITY.md CHANGELOG.md docs/local-whisper-validation.md models/README.md THIRD_PARTY_NOTICES.md package.json Cargo.toml apps/desktop/package.json apps/desktop/src-tauri/tauri.conf.json
git commit -m "docs: document parakeet local model support"
```

---

### Task 6: Full Verification And Branch Review

**Files:**
- Modify: any implementation files required by verification failures only.

- [ ] **Step 1: Run Rust formatting**

```powershell
cargo fmt --all -- --check
```

- [ ] **Step 2: Run workspace tests**

```powershell
cargo test --workspace
```

- [ ] **Step 3: Run Clippy with warnings denied**

```powershell
cargo clippy --workspace --all-targets -- -D warnings
```

- [ ] **Step 4: Build the desktop frontend**

```powershell
pnpm --filter @forge-wisper/desktop build
```

- [ ] **Step 5: Review repository diff and status**

```powershell
git diff --check
```

Confirm no model weights, raw audio, credentials, generated build output, or unrelated changes are included.

- [ ] **Step 6: Commit any required verification-only fix and report readiness**

Use a focused Conventional Commit only if verification requires a source correction. Otherwise leave the branch ready for review with all checks recorded.
