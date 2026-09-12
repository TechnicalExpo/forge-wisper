# Local Whisper Validation

This validation uses an externally downloaded model. Model weights must not be committed to the repository.

The application does not persist raw microphone audio. Audio is encoded in
memory, passed through transcription/cleanup/verification/output, and then
discarded. History stores transcript text and metadata only: provider, model,
duration, timestamp, and verification status. This means a historical
transcription cannot be reused as an audio benchmark fixture.

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

The Windows launcher automatically discovers the newest Vulkan SDK under
`C:\VulkanSDK`, validates `glslc.exe`, and uses `C:\t` as a
short Cargo target directory to avoid MSBuild's nested Vulkan shader path limit.

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

Windows x64 builds include the Vulkan Whisper backend. The hardware card reports
logical CPU cores and system RAM, while the Local Whisper compute status reports
Vulkan GPU devices separately.

```text
forge.local_whisper requested_device=cpu active_backend=cpu gpu_available=true
forge.local_whisper requested_device=gpu active_backend=vulkan gpu_available=true
forge.local_whisper phase=transcription_started gpu_acceleration=true active_backend=vulkan
forge.local_whisper phase=model_loaded gpu_acceleration=true active_backend=vulkan
```

In CPU mode, the app detects the GPU but does not use it. In GPU mode, Windows
Task Manager should show activity and dedicated memory usage on the selected
Vulkan device. The discrete NVIDIA device is preferred over integrated Intel
when both are available.

Record:

- First model-load time
- Second transcription time with the cached context
- Model id/path
- Audio duration
- Final transcript correctness
- RAM impact if available
- Download lifecycle phases and whether verification completed successfully
- Backend log values for `backend` and `gpu_acceleration`

The cache lifecycle is visible in the Local Whisper logs:

```text
forge.local_whisper phase=model_loaded elapsed_ms=<first-load-time>
forge.local_whisper phase=model_cache_hit
```

For a controlled cache measurement, use the same model, compute device, and
10-30 second WAV input for three runs. Record the first run as cold-start time,
then compare the second and third warm runs. CPU and Vulkan contexts are cached
separately, and changing the selected model invalidates the cached context.

## Recorded Benchmark

On the validation machine, `record.wav` was normalized in memory from stereo
48 kHz PCM16 to the required mono 16 kHz PCM16 format. The source file remained
local and ignored by Git. The same normalized audio was sent to Local Whisper
CPU, Local Whisper Vulkan GPU, and Groq.

```text
Source duration: 125.013 seconds
Model: ggml-base.bin
Local CPU cold run: 28,115 ms
Local CPU warm run: 22,298 ms
Local GPU cold run: 38,260 ms
Local GPU warm run: 4,114 ms
Groq run: 2,422 ms
```

Backend evidence:

```text
CPU: whisper_init_with_params_no_state: use gpu = 0
GPU: whisper_init_with_params_no_state: use gpu = 1
GPU: whisper_backend_init_gpu: using Vulkan1 backend
```

The GPU cold run includes Vulkan/model initialization. The warm GPU run reused
the loaded context and was substantially faster than the warm CPU run. Groq was
fastest for this run but sent the audio to the configured cloud provider. The
Local CPU/GPU and Groq transcripts all completed successfully through the
benchmark cleanup path; transcription wording varied between providers, so
timing and backend selection should not be interpreted as an accuracy ranking.

## Automated Fixture Work

The repository intentionally does not include model weights or raw speech. Add a deterministic generated WAV fixture and an opt-in test harness only when the environment provides `FORGE_WHISPER_MODEL_PATH`; otherwise the test must skip with a clear message rather than fail CI.
