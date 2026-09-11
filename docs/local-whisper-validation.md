# Local Whisper Validation

This validation uses an externally downloaded model. Model weights must not be committed to the repository.

## Prerequisites

- Windows MSVC Build Tools
- LLVM with `libclang.dll`
- CMake
- A compatible `ggml-*.bin` Whisper model
- A 16-bit mono 16 kHz WAV fixture containing speech

Set the native build environment in PowerShell:

```powershell
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
$env:Path = "C:\Program Files\CMake\bin;C:\Program Files\LLVM\bin;$env:USERPROFILE\.cargo\bin;$env:Path"
```

## Model Path Override

The provider accepts an external model path for validation without changing the persisted model catalog:

```powershell
$env:FORGE_WHISPER_MODEL_PATH = "C:\path\to\ggml-base.bin"
```

The file must be compatible with the bundled `whisper-rs 0.16.0` runtime.

## Manual Validation

1. Start the app with `pnpm tauri:dev` from this shell.
2. Select Local Whisper in Settings.
3. Set/download a model that matches the provider catalog, or use `FORGE_WHISPER_MODEL_PATH`.
4. Record a sentence with known words, for example: `The quick brown fox jumps over the lazy dog.`
5. Confirm the output contains the spoken words and is pasted into the focused application.
6. Repeat once with the same model and record the timing logs:

```powershell
$env:RUST_LOG = "forge.performance=info"
pnpm tauri:dev
```

Record:

- First model-load time
- Second transcription time with the cached context
- Model id/path
- Audio duration
- Final transcript correctness
- RAM impact if available

## Automated Fixture Work

The repository intentionally does not include model weights or raw speech. Add a deterministic generated WAV fixture and an opt-in test harness only when the environment provides `FORGE_WHISPER_MODEL_PATH`; otherwise the test must skip with a clear message rather than fail CI.
