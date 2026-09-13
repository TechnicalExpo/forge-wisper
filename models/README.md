# 🧠 Local Models Directory

This directory stores offline model files for Forge Wisper's local speech recognition engines.

## Supported Families

- **Whisper** uses GGML `.bin` files through the existing CPU/Vulkan runtime.
- **Parakeet V3 Int8** uses the ONNX directory `parakeet-tdt-0.6b-v3-int8` through the CPU-only `transcribe-rs` runtime. It performs automatic language detection; the manual recognition-language setting does not apply.

Parakeet V3 is downloaded from Handy's public mirror:

```text
https://blob.handy.computer/parakeet-v3-int8.tar.gz
SHA-256: 43d37191602727524a7d8c6da0eef11c4ba24320f5b4730f1a2497befc2efa77
```

---

## 📦 Supported Model Weights

| Model Name | File Name | Disk Size | Required RAM | Speed / Accuracy Profile |
| :--- | :--- | :--- | :--- | :--- |
| **Tiny** | `ggml-tiny.bin` | ~75 MB | ~390 MB | ⚡ Ultra-fast / Basic English |
| **Base** | `ggml-base.bin` | ~142 MB | ~500 MB | 🚀 Fast / Good everyday accuracy |
| **Small** | `ggml-small.bin` | ~466 MB | ~1.0 GB | ⚖️ Balanced / High accuracy |
| **Medium** | `ggml-medium.bin` | ~1.5 GB | ~2.6 GB | 🎯 High accuracy / Moderate speed |
| **Large-v3-Turbo** | `ggml-large-v3-turbo.bin` | ~1.6 GB | ~2.8 GB | ⚡ Peak accuracy & optimized speed |
| **Large-v3** | `ggml-large-v3.bin` | ~3.1 GB | ~4.7 GB | 🏆 Maximum accuracy for heavy accents |

---

## 📥 How to Download Models

### 1. In-App Model Manager (Recommended)
You can download, activate, and delete models directly from the Forge Wisper desktop UI:
- Open **Forge Wisper** $\to$ Navigate to the **Models** tab.
- Click **Download Model** next to your preferred model.
- The app automatically downloads, verifies the file integrity, and activates it.

### 2. Manual Download (Offline Airgapped Environments)
If you are deploying Forge Wisper in an offline or airgapped environment, you can download model files manually from HuggingFace:
- Source: [ggerganov/whisper.cpp on HuggingFace](https://huggingface.co/ggerganov/whisper.cpp/tree/main)
- Place the downloaded `ggml-*.bin` file directly inside this directory or in your OS app data directory (`%APPDATA%\forge\ForgeWisper\data\models` on Windows).
- In-app downloads are pinned to the Hugging Face model revision and verified against the catalog SHA-256 before activation. Manually installed files are discovered by filename but are not treated as catalog-verified downloads.

For Parakeet V3, extract the archive so the required files are inside
`parakeet-tdt-0.6b-v3-int8/`: `encoder-model.int8.onnx`,
`decoder_joint-model.int8.onnx`, `nemo128.onnx`, and `vocab.txt`.

---

## 🔒 Git Policy

All binary weight files (`*.bin`, `*.gguf`, `*.pt`, `*.onnx`) are **gitignored** to keep the repository lightweight. Only this `README.md` is committed to version control.
