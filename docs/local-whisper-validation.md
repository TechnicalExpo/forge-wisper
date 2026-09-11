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

For development, the repository provides a repeatable launcher that performs this setup and loads the MSVC environment automatically:

```powershell
pnpm tauri:dev:windows
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

For model download and Local Whisper backend logs, use:

```powershell
$env:RUST_LOG = "info,forge.model_download=debug,forge.local_whisper=info,forge.hardware=info"
pnpm tauri:dev:windows
```

Expected download lifecycle entries are:

```text
forge.model_download phase=command_started
forge.model_download phase=started
forge.model_download phase=connected
forge.model_download phase=progress percentage=25/50/75/100
forge.model_download phase=verifying
forge.model_download phase=installed
forge.model_download phase=command_completed success=true
```

The model is ready only after `phase=installed` and
`phase=command_completed success=true`. A verification failure must show
`phase=verification_failed`, and no final model binary should be activated.

## GPU Status

The current Local Whisper runtime is intentionally CPU-only. The hardware card
reports logical CPU cores and system RAM; it does not currently detect or use a
GPU. The following log entries make that explicit:

```text
forge.hardware gpu_acceleration=false backend=cpu
forge.local_whisper phase=transcription_started gpu_acceleration=false backend=cpu
forge.local_whisper phase=model_loaded gpu_acceleration=false backend=cpu
```

Therefore Windows Task Manager should show CPU activity during local inference,
not CUDA/DirectML GPU compute activity. GPU acceleration requires a separate
whisper.cpp build/backend decision and is not enabled by this release.

Record:

- First model-load time
- Second transcription time with the cached context
- Model id/path
- Audio duration
- Final transcript correctness
- RAM impact if available
- Download lifecycle phases and whether verification completed successfully
- Backend log values for `backend` and `gpu_acceleration`

## Automated Fixture Work

The repository intentionally does not include model weights or raw speech. Add a deterministic generated WAV fixture and an opt-in test harness only when the environment provides `FORGE_WHISPER_MODEL_PATH`; otherwise the test must skip with a clear message rather than fail CI.
