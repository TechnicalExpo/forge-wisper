# Parakeet V3 Model Family Design

**Status:** Approved direction, pending written-spec review

## Goal

Add Parakeet V3 as the first non-Whisper local transcription family while preserving the existing Whisper CPU/Vulkan implementation, model catalog behavior, safe output pipeline, and privacy guarantees.

## Scope

This slice supports one Parakeet family/model variant first: Parakeet V3 Int8. Users can discover, download, validate, select, and use it for local transcription. Existing Whisper models remain available and unchanged.

The implementation does not add Moonshine, Canary, SenseVoice, GigaAM, Qwen ASR, live partial transcription, translation, or GPU acceleration for Parakeet.

## Architecture

Whisper and Parakeet use separate runtime boundaries:

```text
TranscriptionProvider
├── LocalWhisperProvider
│   └── whisper-rs / GGML .bin / CPU or Vulkan
└── LocalParakeetProvider
    └── transcribe-rs 0.2.9 / ONNX / CPU
```

`LocalWhisperProvider` remains responsible only for Whisper-family inference and its existing context cache. `LocalParakeetProvider` owns Parakeet model loading, inference, and a separate loaded-engine cache. The shared pipeline continues to own recording, cleanup, verification, history, and output injection.

## Runtime Dependency

Use `transcribe-rs = "0.2.9"` with the `parakeet` feature. This release exposes `ParakeetEngine`, `ParakeetModelParams`, and `TranscriptionEngine`, and uses ONNX Runtime CPU execution for Parakeet.

Parakeet model directories must contain the runtime-required files:

```text
parakeet-tdt-0.6b-v3-int8/
├── encoder-model.int8.onnx
├── decoder_joint-model.int8.onnx
├── nemo128.onnx
└── vocab.txt
```

The runtime may also read a `config.json` if supplied by the upstream model package, but the provider must validate the files it directly requires before marking the model installed.

## Model Metadata

Extend local model metadata without weakening existing Whisper fields:

- `family`: stable serialized value such as `whisper` or `parakeet`.
- `format`: `ggml-bin` for Whisper or `onnx-directory` for Parakeet.
- `language_mode`: `manual` or `auto-detect`.
- `license`: model license identifier and source attribution.
- `required_files`: relative files needed for installation validation.
- Existing checksum/revision information for every downloadable artifact.

Parakeet is represented as a directory model with a manifest of files and checksums. Downloads must use a staged archive, validate archive contents, validate extracted file sizes and SHA-256 values, then atomically promote the directory. No partially extracted directory may be selectable.

## Settings And Selection

Keep the existing local provider setting and add an explicit selected family/model identity rather than inferring family from a filename. Existing settings without a family continue to resolve to Whisper for backward compatibility.

The model UI groups entries by family:

- Whisper Models
- Parakeet Models

Selecting Parakeet requires a validated installed Parakeet model. If it is not installed, the UI exposes download/install state and does not switch the active provider until validation succeeds.

Parakeet uses automatic language detection. The existing language setting is passed as `auto`/ignored for Parakeet, and the UI explains that manual recognition language selection applies to Whisper/Groq but not to this Parakeet model.

## Provider Contract

`LocalParakeetProvider` implements the existing `TranscriptionProvider` trait:

- Accept the same `AudioData` input as Whisper.
- Validate or convert the pipeline’s 16-bit mono 16 kHz WAV into normalized `Vec<f32>` samples.
- Load the selected model directory with `ParakeetModelParams::int8()`.
- Run inference in `spawn_blocking` so ONNX execution does not block async Tauri work.
- Return `Transcript` with provider `local-parakeet`, model ID, detected language metadata where available, duration, and no fabricated confidence value.
- Return typed `ProviderError::ModelError` or `InternalError` for missing model files, ONNX load failures, and inference failures.

The provider must not log raw audio, transcript text, API keys, or model secrets. Operational logs may include model ID, family, backend, elapsed time, and audio duration.

## Error Handling And Lifecycle

- Missing or incomplete Parakeet files are unavailable, not silently repaired during transcription.
- A failed download removes staged archive/extraction data.
- A failed model load does not replace the active working Whisper model.
- Changing family or model invalidates only the relevant local provider cache.
- No mutex is held across inference or an await point.
- The existing safe paste/output path remains unchanged.

## Licensing And LPGS

Before release enablement:

- Record `transcribe-rs` MIT licensing and dependency notices.
- Record Parakeet model source, license, attribution, and redistribution terms.
- Confirm ONNX Runtime licensing and Windows redistribution requirements.
- Document that Parakeet is downloaded on demand and is not bundled in the installer.
- Update privacy documentation to state that Parakeet inference remains local and no audio is sent to cloud providers.

## Testing

Add deterministic tests for:

- Family/format metadata serialization.
- Required-file validation for complete, missing, and extra/partial model directories.
- Manifest checksum and archive extraction failure cleanup.
- Provider selection preserving Whisper behavior.
- Parakeet refusing to run without a validated model.
- Parakeet language behavior remaining automatic.
- Provider errors not including raw audio or transcript content.

Runtime tests must accept an externally supplied model path such as `FORGE_PARAKEET_MODEL_PATH`; model weights and raw audio must not be committed. When the fixture is unavailable, tests must verify setup errors rather than fabricate transcription output.

The full verification gate remains required:

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
pnpm --filter @forge-wisper/desktop build
cargo fmt --all -- --check
```

## Deferred Work

- Additional model families.
- Parakeet GPU/DirectML/Vulkan execution.
- Live streaming or partial transcript updates.
- Translation into a target language.
- Automatic import of arbitrary Parakeet model directories without manifest metadata.
