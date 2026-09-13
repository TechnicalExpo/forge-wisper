# Third-Party Notices

## Parakeet V3 Runtime

Forge Wisper uses `transcribe-rs` version `0.2.9` with its `parakeet` feature
for optional local Parakeet inference. The crate is distributed under the MIT
License. Its Parakeet engine uses ONNX Runtime CPU execution.

## Parakeet V3 Model

The optional Parakeet V3 Int8 model is downloaded from the Handy project mirror:

```text
https://blob.handy.computer/parakeet-v3-int8.tar.gz
```

The archive is verified by SHA-256 before extraction and is not bundled with
Forge Wisper. Model licensing, source attribution, and redistribution terms are
those published by the model distributor and must be reviewed before any
installer bundling or redistribution is enabled.

## Existing Models

Whisper model files remain subject to their original upstream licenses. Forge
Wisper does not bundle model weights in source control or release installers.
