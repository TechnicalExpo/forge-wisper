use async_trait::async_trait;
use forge_transcription::{AudioData, ProviderCapabilities, ProviderError, Transcript, TranscriptionOptions, TranscriptionProvider};
use hound::WavReader;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use transcribe_rs::engines::parakeet::{ParakeetEngine, ParakeetModelParams};
use transcribe_rs::TranscriptionEngine;

use crate::ModelManager;

const PARAKEET_MODEL_ID: &str = "parakeet-v3-int8";
const REQUIRED_FILES: &[&str] = &[
    "encoder-model.int8.onnx",
    "decoder_joint-model.int8.onnx",
    "nemo128.onnx",
    "vocab.txt",
];

pub fn required_model_files() -> &'static [&'static str] {
    REQUIRED_FILES
}

pub fn validate_model_directory(path: &Path) -> Result<(), ProviderError> {
    if !path.is_dir() {
        return Err(ProviderError::ModelError(
            "Parakeet model directory is missing".to_string(),
        ));
    }

    for file in REQUIRED_FILES {
        let candidate = path.join(file);
        let metadata = std::fs::metadata(&candidate).map_err(|_| {
            ProviderError::ModelError(format!("Parakeet model file is missing: {file}"))
        })?;
        if !metadata.is_file() || metadata.len() == 0 {
            return Err(ProviderError::ModelError(format!(
                "Parakeet model file is empty: {file}"
            )));
        }
    }

    Ok(())
}

type ParakeetEngineCache = Arc<Mutex<Option<(PathBuf, Arc<Mutex<ParakeetEngine>>)>>>;

pub struct LocalParakeetProvider {
    model_manager: Arc<ModelManager>,
    engine_cache: ParakeetEngineCache,
}

impl LocalParakeetProvider {
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        Self {
            model_manager,
            engine_cache: Arc::new(Mutex::new(None)),
        }
    }

    pub fn model_id() -> &'static str {
        PARAKEET_MODEL_ID
    }

    pub fn set_active_model_path(&self, path: Option<PathBuf>) {
        let mut cache = self.engine_cache.lock().unwrap();
        if cache.as_ref().map(|(cached, _)| cached) != path.as_ref() {
            *cache = None;
        }
    }

    fn resolve_model_path(&self) -> Result<PathBuf, ProviderError> {
        if let Some(path) = std::env::var_os("FORGE_PARAKEET_MODEL_PATH").map(PathBuf::from) {
            validate_model_directory(&path)?;
            return Ok(path);
        }

        let path = self.model_manager.find_model_directory("parakeet-tdt-0.6b-v3-int8")?;
        validate_model_directory(&path)?;
        Ok(path)
    }
}

impl Default for LocalParakeetProvider {
    fn default() -> Self {
        Self::new(Arc::new(ModelManager::new()))
    }
}

#[async_trait]
impl TranscriptionProvider for LocalParakeetProvider {
    fn id(&self) -> &str {
        "local-parakeet"
    }

    fn name(&self) -> &str {
        "Local Parakeet V3 (Offline ONNX runtime)"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_local: true,
            supports_cloud: false,
            supported_languages: vec!["auto".to_string()],
            available_models: vec![PARAKEET_MODEL_ID.to_string()],
            requires_api_key: false,
        }
    }

    async fn transcribe(
        &self,
        audio: AudioData,
        _options: TranscriptionOptions,
    ) -> Result<Transcript, ProviderError> {
        if audio.wav_bytes.is_empty() {
            return Err(ProviderError::InvalidAudio("Audio data is empty".to_string()));
        }

        let model_path = self.resolve_model_path()?;
        let engine_cache = Arc::clone(&self.engine_cache);
        let duration_ms = audio.duration_ms;
        let started = Instant::now();

        let result = tokio::task::spawn_blocking(move || -> Result<String, ProviderError> {
            let mut reader = WavReader::new(Cursor::new(audio.wav_bytes))
                .map_err(|error| ProviderError::InvalidAudio(error.to_string()))?;
            let spec = reader.spec();
            if spec.sample_rate != 16_000 || spec.channels != 1 || spec.sample_format != hound::SampleFormat::Int || spec.bits_per_sample != 16 {
                return Err(ProviderError::InvalidAudio(
                    "Local Parakeet requires 16-bit mono 16 kHz WAV audio".to_string(),
                ));
            }
            let samples = reader
                .samples::<i16>()
                .map(|sample| sample.map(|value| value as f32 / i16::MAX as f32))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| ProviderError::InvalidAudio(error.to_string()))?;

            let mut cache = engine_cache
                .lock()
                .map_err(|_| ProviderError::InternalError("Parakeet engine cache was poisoned".to_string()))?;
            if cache.as_ref().map(|(cached, _)| cached) != Some(&model_path) {
                let mut engine = ParakeetEngine::new();
                engine
                    .load_model_with_params(&model_path, ParakeetModelParams::int8())
                    .map_err(|error| ProviderError::ModelError(error.to_string()))?;
                *cache = Some((model_path.clone(), Arc::new(Mutex::new(engine))));
            }
            let engine = Arc::clone(
                &cache
                .as_mut()
                .ok_or_else(|| ProviderError::ModelError("Parakeet model was not loaded".to_string()))?
                .1,
            );
            drop(cache);
            let mut engine = engine
                .lock()
                .map_err(|_| ProviderError::InternalError("Parakeet engine was poisoned".to_string()))?;
            let result = engine
                .transcribe_samples(samples, None)
                .map_err(|error| ProviderError::InternalError(error.to_string()))?;
            let text = result.text.trim().to_string();
            if text.is_empty() {
                return Err(ProviderError::InvalidAudio("No speech detected".to_string()));
            }
            Ok(text)
        })
        .await
        .map_err(|error| ProviderError::InternalError(error.to_string()))??;

        tracing::info!(
            target: "forge.local_parakeet",
            model_id = PARAKEET_MODEL_ID,
            audio_duration_ms = duration_ms,
            elapsed_ms = started.elapsed().as_millis() as u64,
            "Local Parakeet transcription completed"
        );

        Ok(Transcript {
            text: result,
            language: "auto".to_string(),
            provider: self.id().to_string(),
            model: PARAKEET_MODEL_ID.to_string(),
            duration_ms,
            confidence: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{required_model_files, validate_model_directory};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn validation_rejects_missing_model_directory() {
        let directory = tempdir().unwrap();
        let error = validate_model_directory(directory.path().join("missing").as_path()).unwrap_err();
        assert!(error.to_string().contains("directory is missing"));
    }

    #[test]
    fn validation_requires_non_empty_runtime_files() {
        let directory = tempdir().unwrap();
        for file in required_model_files() {
            fs::write(directory.path().join(file), b"model").unwrap();
        }
        fs::write(directory.path().join("vocab.txt"), []).unwrap();

        let error = validate_model_directory(directory.path()).unwrap_err();
        assert!(error.to_string().contains("vocab.txt"));
    }
}
