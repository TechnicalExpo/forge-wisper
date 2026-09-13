use async_trait::async_trait;
use directories::ProjectDirs;
use forge_transcription::{
    normalize_language, AudioData, ModelFamily, ModelFormat, ProviderCapabilities, ProviderError,
    Transcript, TranscriptionOptions, TranscriptionProvider,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{create_dir_all, remove_file, File};
use std::io::Write;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{debug, info, warn};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use sha2::{Digest, Sha256};

pub mod parakeet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDownloadProgress {
    pub model_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelInfo {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub family: ModelFamily,
    #[serde(default)]
    pub format: ModelFormat,
    pub filename: String,
    pub size_mb: u64,
    pub ram_estimate_mb: u64,
    pub download_url: String,
    pub revision: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub is_installed: bool,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareRecommendation {
    pub logical_cores: usize,
    pub estimated_ram_gb: u32,
    pub recommended_model_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalComputeDeviceInfo {
    pub requested_device: String,
    pub active_backend: String,
    pub gpu_available: bool,
    pub gpu_name: Option<String>,
    pub reason: String,
}

fn gpu_device_info() -> Option<(i32, String)> {
    #[cfg(windows)]
    {
        let devices = whisper_rs::vulkan::list_devices();
        devices
            .iter()
            .find(|device| {
                let name = device.name.to_ascii_lowercase();
                name.contains("nvidia") || name.contains("amd") || name.contains("radeon")
            })
            .or_else(|| devices.first())
            .map(|device| (device.id, device.name.clone()))
    }

    #[cfg(not(windows))]
    {
        None
    }
}

pub fn local_compute_device_info(requested_device: &str) -> LocalComputeDeviceInfo {
    let requested_device = if requested_device == "gpu" { "gpu" } else { "cpu" };
    let gpu = gpu_device_info();
    let gpu_available = gpu.is_some();
    let (active_backend, reason) = match (requested_device, gpu_available) {
        ("gpu", true) => ("vulkan", "Vulkan GPU backend is available"),
        ("gpu", false) => ("unavailable", "Vulkan GPU backend is not available"),
        ("cpu", _) => ("cpu", "CPU mode was selected"),
        _ => ("cpu", "CPU mode was selected"),
    };

    LocalComputeDeviceInfo {
        requested_device: requested_device.to_string(),
        active_backend: active_backend.to_string(),
        gpu_available,
        gpu_name: gpu.map(|(_, name)| name),
        reason: reason.to_string(),
    }
}

#[derive(Clone)]
pub struct ModelManager {
    models_dir: PathBuf,
    active_downloads: Arc<Mutex<HashMap<String, ModelDownloadProgress>>>,
    search_fallback_paths: bool,
}

impl ModelManager {
    pub fn new() -> Self {
        let models_dir = Self::resolve_models_dir();
        if !models_dir.exists() {
            let _ = create_dir_all(&models_dir);
        }
        Self {
            models_dir,
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
            search_fallback_paths: true,
        }
    }

    pub fn get_active_downloads(&self) -> HashMap<String, ModelDownloadProgress> {
        self.active_downloads.lock().unwrap().clone()
    }

    fn resolve_models_dir() -> PathBuf {
        // 1. Check workspace/local models directory
        let local_path = Path::new("models");
        if local_path.exists() {
            return local_path.to_path_buf();
        }

        // 2. Check AppData models directory
        if let Some(proj) = ProjectDirs::from("com", "forge", "ForgeWisper") {
            let dir = proj.data_dir().join("models");
            let _ = create_dir_all(&dir);
            dir
        } else {
            PathBuf::from("models")
        }
    }

    /// Searches across all candidate model directories to find an existing model binary
    pub fn find_model_file(&self, filename: &str) -> Option<PathBuf> {
        let mut candidates = Vec::new();

        // Check primary models_dir
        candidates.push(self.models_dir.join(filename));

        if !self.search_fallback_paths {
            return candidates.into_iter().find(|path| {
                path.exists()
                    && std::fs::metadata(path)
                        .map(|metadata| metadata.len() > 1024 * 1024)
                        .unwrap_or(false)
            });
        }

        // Check workspace "models/"
        candidates.push(Path::new("models").join(filename));

        // Check app data directory
        if let Some(proj) = ProjectDirs::from("com", "forge", "ForgeWisper") {
            candidates.push(proj.data_dir().join("models").join(filename));
            candidates.push(proj.cache_dir().join("whisper").join(filename));
        }

        // Check user home .cache/whisper
        if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            candidates.push(PathBuf::from(home).join(".cache").join("whisper").join(filename));
        }

        // Check current executable directory
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                candidates.push(parent.join("models").join(filename));
                candidates.push(parent.join("..").join("models").join(filename));
            }
        }

        for path in candidates {
            if path.exists() {
                // Ensure the file is not a partial 0-byte download and has valid binary payload (>1MB)
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if metadata.len() > 1024 * 1024 {
                        return Some(path);
                    }
                }
            }
        }

        None
    }

    pub fn find_model_directory(&self, directory_name: &str) -> Result<PathBuf, ProviderError> {
        let candidates = [
            self.models_dir.join(directory_name),
            Path::new("models").join(directory_name),
        ];
        candidates
            .into_iter()
            .find(|path| path.is_dir())
            .ok_or_else(|| ProviderError::ModelError(format!("Model directory '{}' not found", directory_name)))
    }

    pub fn get_models_dir(&self) -> &Path {
        &self.models_dir
    }

    pub fn list_available_models(&self) -> Vec<LocalModelInfo> {
        const REVISION: &str = "5359861c739e955e79d9a303bcbc70fb988958b1";
        let catalog = vec![
            (
                "tiny",
                "Whisper Tiny",
                "ggml-tiny.bin",
                75,
                390,
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin",
                "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
                false,
            ),
            (
                "base",
                "Whisper Base",
                "ggml-base.bin",
                142,
                500,
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin",
                "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
                true,
            ),
            (
                "small",
                "Whisper Small",
                "ggml-small.bin",
                466,
                1024,
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
                "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
                false,
            ),
            (
                "medium",
                "Whisper Medium",
                "ggml-medium.bin",
                1536,
                2600,
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin",
                "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
                false,
            ),
            (
                "large-v3-turbo",
                "Whisper Large v3 Turbo",
                "ggml-large-v3-turbo.bin",
                1638,
                2800,
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin",
                "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69",
                false,
            ),
            (
                "large-v3",
                "Whisper Large v3",
                "ggml-large-v3.bin",
                3100,
                4700,
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin",
                "64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2",
                false,
            ),
        ];

        let mut models: Vec<LocalModelInfo> = catalog
            .into_iter()
            .map(|(id, name, filename, size_mb, ram_mb, _url, sha256, is_def)| {
                let is_installed = self.find_model_file(filename).is_some();
                LocalModelInfo {
                    id: id.to_string(),
                    name: name.to_string(),
                    family: ModelFamily::Whisper,
                    format: ModelFormat::GgmlBin,
                    filename: filename.to_string(),
                    size_mb,
                    ram_estimate_mb: ram_mb,
                    download_url: format!(
                        "https://huggingface.co/ggerganov/whisper.cpp/resolve/{}/{}",
                        REVISION, filename
                    ),
                    revision: REVISION.to_string(),
                    sha256: sha256.to_string(),
                    size_bytes: match id {
                        "tiny" => 77_691_713,
                        "base" => 147_951_465,
                        "small" => 487_601_967,
                        "medium" => 1_533_763_059,
                        "large-v3-turbo" => 1_624_555_275,
                        "large-v3" => 3_095_033_483,
                        _ => 0,
                    },
                    is_installed,
                    is_default: is_def,
                }
            })
            .collect();

        let parakeet_directory = "parakeet-tdt-0.6b-v3-int8";
        models.push(LocalModelInfo {
            id: "parakeet-v3-int8".to_string(),
            name: "Parakeet V3 Int8".to_string(),
            family: ModelFamily::Parakeet,
            format: ModelFormat::OnnxDirectory,
            filename: parakeet_directory.to_string(),
            size_mb: 478,
            ram_estimate_mb: 1200,
            download_url: "https://blob.handy.computer/parakeet-v3-int8.tar.gz".to_string(),
            revision: "handy-parakeet-v3-int8".to_string(),
            sha256: "43d37191602727524a7d8c6da0eef11c4ba24320f5b4730f1a2497befc2efa77".to_string(),
            size_bytes: 0,
            is_installed: self.find_model_directory(parakeet_directory).is_ok(),
            is_default: false,
        });
        models
    }

    /// Auto-picks the best available local model that is already downloaded
    pub fn auto_pick_installed_model(&self) -> Option<LocalModelInfo> {
        let models = self.list_available_models();
        let installed: Vec<LocalModelInfo> = models
            .into_iter()
            .filter(|m| m.family == ModelFamily::Whisper && m.is_installed)
            .collect();

        if installed.is_empty() {
            return None;
        }

        // Priority order: large-v3-turbo > large-v3 > medium > small > base > tiny
        let priority = ["large-v3-turbo", "large-v3", "medium", "small", "base", "tiny"];
        for pref in priority {
            if let Some(found) = installed.iter().find(|m| m.id == pref) {
                return Some(found.clone());
            }
        }

        installed.into_iter().next()
    }

    pub async fn download_model(&self, model_id: &str) -> Result<PathBuf, ProviderError> {
        self.download_model_with_progress(model_id, |_, _| {}).await
    }

    pub async fn download_model_with_progress<F>(
        &self,
        model_id: &str,
        mut progress: F,
    ) -> Result<PathBuf, ProviderError>
    where
        F: FnMut(u64, u64) + Send + 'static,
    {
        let models = self.list_available_models();
        let target = models
            .into_iter()
            .find(|m| m.id == model_id)
            .ok_or_else(|| ProviderError::ModelError(format!("Model ID '{}' not recognized", model_id)))?;

        if target.format == ModelFormat::OnnxDirectory {
            return self.download_directory_model(&target, progress).await;
        }

        let dest_path = self.models_dir.join(&target.filename);
        let part_path = self.models_dir.join(format!("{}.part", target.filename));
        let started = Instant::now();
        info!(
            target: "forge.model_download",
            phase = "started",
            model_id,
            filename = %target.filename,
            destination = %dest_path.display(),
            "Starting Local Whisper model download"
        );
        let client = Client::new();
        let mut response = client
            .get(&target.download_url)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderError::NetworkError(format!(
                "Failed to download model (HTTP {})",
                response.status()
            )));
        }

        let total_size = response
            .content_length()
            .unwrap_or(target.size_mb * 1024 * 1024);

        info!(
            target: "forge.model_download",
            phase = "connected",
            model_id,
            total_bytes = total_size,
            "Connected to model download source"
        );

        let mut downloaded: u64 = 0;
        let mut next_logged_percentage = 25;
        let mut file = File::create(&part_path)
            .map_err(|e| ProviderError::ModelError(e.to_string()))?;

        // Initialize active download status
        {
            let mut guard = self.active_downloads.lock().unwrap();
            guard.insert(
                model_id.to_string(),
                ModelDownloadProgress {
                    model_id: model_id.to_string(),
                    downloaded_bytes: 0,
                    total_bytes: total_size,
                    percentage: 0,
                },
            );
        }

        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| {
                let _ = self.active_downloads.lock().unwrap().remove(model_id);
                let _ = remove_file(&part_path);
                ProviderError::NetworkError(e.to_string())
            })?
        {
            if let Err(e) = file.write_all(&chunk) {
                let _ = self.active_downloads.lock().unwrap().remove(model_id);
                let _ = remove_file(&part_path);
                return Err(ProviderError::ModelError(e.to_string()));
            }
            downloaded += chunk.len() as u64;
            let percentage = if total_size > 0 {
                ((downloaded as f64 / total_size as f64) * 100.0).round() as u32
            } else {
                0
            };

            // Update in-memory active download state
            {
                let mut guard = self.active_downloads.lock().unwrap();
                if let Some(item) = guard.get_mut(model_id) {
                    item.downloaded_bytes = downloaded;
                    item.percentage = percentage;
                }
            }

            progress(downloaded, total_size);

            if percentage >= next_logged_percentage || percentage >= 100 {
                debug!(
                    target: "forge.model_download",
                    phase = "progress",
                    model_id,
                    downloaded_bytes = downloaded,
                    total_bytes = total_size,
                    percentage,
                    "Local Whisper model download progress"
                );
                next_logged_percentage = ((percentage / 25) + 1) * 25;
            }
        }

        if let Err(e) = file.flush() {
            let _ = self.active_downloads.lock().unwrap().remove(model_id);
            let _ = remove_file(&part_path);
            return Err(ProviderError::ModelError(e.to_string()));
        }

        info!(
            target: "forge.model_download",
            phase = "verifying",
            model_id,
            elapsed_ms = started.elapsed().as_millis() as u64,
            "Download complete; verifying model size and SHA-256"
        );

        if let Err(error) = verify_model_file(&part_path, target.size_bytes, &target.sha256) {
            warn!(
                target: "forge.model_download",
                phase = "verification_failed",
                model_id,
                error = %error,
                "Local Whisper model verification failed"
            );
            let _ = self.active_downloads.lock().unwrap().remove(model_id);
            let _ = remove_file(&part_path);
            return Err(ProviderError::ModelError(format!(
                "Model integrity verification failed for '{}': {}",
                target.id, error
            )));
        }

        // Atomically rename .part -> final .bin
        if let Err(e) = std::fs::rename(&part_path, &dest_path) {
            let _ = self.active_downloads.lock().unwrap().remove(model_id);
            let _ = remove_file(&part_path);
            return Err(ProviderError::ModelError(e.to_string()));
        }

        info!(
            target: "forge.model_download",
            phase = "installed",
            model_id,
            path = %dest_path.display(),
            elapsed_ms = started.elapsed().as_millis() as u64,
            "Local Whisper model verified and installed"
        );

        // Cleanup active download
        {
            let mut guard = self.active_downloads.lock().unwrap();
            guard.remove(model_id);
        }

        Ok(dest_path)
    }

    async fn download_directory_model<F>(
        &self,
        target: &LocalModelInfo,
        mut progress: F,
    ) -> Result<PathBuf, ProviderError>
    where
        F: FnMut(u64, u64) + Send + 'static,
    {
        let archive_path = self.models_dir.join(format!("{}.part.tar.gz", target.id));
        let extract_path = self.models_dir.join(format!("{}.part", target.filename));
        let destination = self.models_dir.join(&target.filename);
        let response = Client::new()
            .get(&target.download_url)
            .send()
            .await
            .map_err(|error| ProviderError::NetworkError(error.to_string()))?;
        if !response.status().is_success() {
            return Err(ProviderError::NetworkError(format!(
                "Failed to download model (HTTP {})",
                response.status()
            )));
        }
        let total = response.content_length().unwrap_or(target.size_mb * 1024 * 1024);
        let bytes = response
            .bytes()
            .await
            .map_err(|error| ProviderError::NetworkError(error.to_string()))?;
        progress(bytes.len() as u64, total);
        std::fs::write(&archive_path, &bytes)
            .map_err(|error| ProviderError::ModelError(error.to_string()))?;
        verify_model_hash(&archive_path, &target.sha256)?;

        let _ = std::fs::remove_dir_all(&extract_path);
        std::fs::create_dir_all(&extract_path)
            .map_err(|error| ProviderError::ModelError(error.to_string()))?;
        let archive = File::open(&archive_path)
            .map_err(|error| ProviderError::ModelError(error.to_string()))?;
        let decoder = flate2::read::GzDecoder::new(archive);
        let mut tar = tar::Archive::new(decoder);
        for entry in tar
            .entries()
            .map_err(|error| ProviderError::ModelError(error.to_string()))?
        {
            let mut entry = entry.map_err(|error| ProviderError::ModelError(error.to_string()))?;
            let path = entry
                .path()
                .map_err(|error| ProviderError::ModelError(error.to_string()))?
                .into_owned();
            if path.is_absolute() || path.components().any(|component| matches!(component, std::path::Component::ParentDir)) {
                return Err(ProviderError::ModelError("Model archive contains an unsafe path".to_string()));
            }
            entry
                .unpack(&extract_path)
                .map_err(|error| ProviderError::ModelError(error.to_string()))?;
        }

        let candidate = if extract_path.join(&target.filename).is_dir() {
            extract_path.join(&target.filename)
        } else {
            extract_path.clone()
        };
        parakeet::validate_model_directory(&candidate)?;
        let _ = std::fs::remove_dir_all(&destination);
        std::fs::rename(&candidate, &destination)
            .map_err(|error| ProviderError::ModelError(error.to_string()))?;
        let _ = std::fs::remove_dir_all(&extract_path);
        let _ = std::fs::remove_file(&archive_path);
        Ok(destination)
    }

    pub fn delete_model(&self, model_id: &str) -> Result<bool, ProviderError> {
        let models = self.list_available_models();
        let target = models
            .into_iter()
            .find(|m| m.id == model_id)
            .ok_or_else(|| ProviderError::ModelError(format!("Model ID '{}' not found", model_id)))?;

        if target.format == ModelFormat::OnnxDirectory {
            let path = self.find_model_directory(&target.filename)?;
            std::fs::remove_dir_all(path).map_err(|e| ProviderError::ModelError(e.to_string()))?;
            Ok(true)
        } else if let Some(existing_path) = self.find_model_file(&target.filename) {
            remove_file(existing_path).map_err(|e| ProviderError::ModelError(e.to_string()))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| e.to_string())?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn verify_model_file(path: &Path, expected_size: u64, expected_sha256: &str) -> Result<(), String> {
    let actual_size = std::fs::metadata(path)
        .map_err(|e| e.to_string())?
        .len();
    if actual_size != expected_size {
        return Err(format!(
            "size mismatch: expected {} bytes, got {}",
            expected_size, actual_size
        ));
    }

    let actual_sha256 = sha256_file(path)?;
    if actual_sha256 != expected_sha256 {
        return Err(format!(
            "checksum mismatch: expected {}, got {}",
            expected_sha256, actual_sha256
        ));
    }

    Ok(())
}

fn verify_model_hash(path: &Path, expected_sha256: &str) -> Result<(), ProviderError> {
    let mut file = File::open(path).map_err(|error| ProviderError::ModelError(error.to_string()))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|error| ProviderError::ModelError(error.to_string()))?;
    let actual = format!("{:x}", hasher.finalize());
    if actual != expected_sha256 {
        return Err(ProviderError::ModelError("Model archive integrity verification failed".to_string()));
    }
    Ok(())
}

impl Default for ModelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(windows)]
fn get_system_ram_gb() -> u32 {
    #[repr(C)]
    struct MemoryStatusEx {
        dw_length: u32,
        dw_memory_load: u32,
        ull_total_phys: u64,
        ull_avail_phys: u64,
        ull_total_page_file: u64,
        ull_avail_page_file: u64,
        ull_total_virtual: u64,
        ull_avail_virtual: u64,
        ull_avail_extended_virtual: u64,
    }

    extern "system" {
        fn GlobalMemoryStatusEx(lp_buffer: *mut MemoryStatusEx) -> i32;
    }

    let mut mem = MemoryStatusEx {
        dw_length: std::mem::size_of::<MemoryStatusEx>() as u32,
        dw_memory_load: 0,
        ull_total_phys: 0,
        ull_avail_phys: 0,
        ull_total_page_file: 0,
        ull_avail_page_file: 0,
        ull_total_virtual: 0,
        ull_avail_virtual: 0,
        ull_avail_extended_virtual: 0,
    };

    let success = unsafe { GlobalMemoryStatusEx(&mut mem) };
    if success != 0 {
        ((mem.ull_total_phys as f64) / (1024.0 * 1024.0 * 1024.0)).round() as u32
    } else {
        16
    }
}

#[cfg(not(windows))]
fn get_system_ram_gb() -> u32 {
    16
}

pub struct HardwareDetector;

impl HardwareDetector {
    pub fn detect_and_recommend() -> HardwareRecommendation {
        let logical_cores = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);

        let ram_gb = get_system_ram_gb();

        let (rec_model, reason) = if ram_gb >= 16 && logical_cores >= 8 {
            (
                "large-v3-turbo",
                format!(
                    "High-performance hardware detected ({} cores, {} GB RAM); Whisper Large v3 Turbo recommended.",
                    logical_cores, ram_gb
                ),
            )
        } else if ram_gb >= 8 && logical_cores >= 4 {
            (
                "small",
                format!(
                    "Balanced hardware detected ({} cores, {} GB RAM); Whisper Small recommended for balanced latency and high accuracy.",
                    logical_cores, ram_gb
                ),
            )
        } else {
            (
                "base",
                format!(
                    "Standard hardware detected ({} cores, {} GB RAM); Whisper Base recommended for smooth latency.",
                    logical_cores, ram_gb
                ),
            )
        };

        info!(
            target: "forge.hardware",
            logical_cores,
            estimated_ram_gb = ram_gb,
            gpu_acceleration = false,
            backend = "cpu",
            "Local Whisper hardware recommendation detected"
        );

        HardwareRecommendation {
            logical_cores,
            estimated_ram_gb: ram_gb,
            recommended_model_id: rec_model.to_string(),
            reason,
        }
    }
}

pub struct LocalWhisperProvider {
    model_manager: Arc<ModelManager>,
    active_model_id: Arc<Mutex<String>>,
    context_cache: WhisperContextCache,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WhisperContextKey {
    model_path: PathBuf,
    backend: String,
    gpu_device_id: Option<i32>,
}

type WhisperContextCache = Arc<Mutex<Option<(WhisperContextKey, Arc<WhisperContext>)>>>;

impl LocalWhisperProvider {
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        Self {
            model_manager,
            active_model_id: Arc::new(Mutex::new("base".to_string())),
            context_cache: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_active_model(&self, model_id: &str) {
        let mut active = self.active_model_id.lock().unwrap();
        *active = model_id.to_string();
        self.context_cache.lock().unwrap().take();
    }
}

impl Default for LocalWhisperProvider {
    fn default() -> Self {
        Self::new(Arc::new(ModelManager::new()))
    }
}

#[async_trait]
impl TranscriptionProvider for LocalWhisperProvider {
    fn id(&self) -> &str {
        "local-whisper"
    }

    fn name(&self) -> &str {
        "Local Whisper (Offline cpp runtime)"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_local: true,
            supports_cloud: false,
            supported_languages: vec![
                "auto".to_string(),
                "en".to_string(),
                "es".to_string(),
                "fr".to_string(),
                "de".to_string(),
                "it".to_string(),
                "zh".to_string(),
                "ja".to_string(),
            ],
            available_models: vec![
                "tiny".to_string(),
                "base".to_string(),
                "small".to_string(),
                "medium".to_string(),
                "large-v3-turbo".to_string(),
                "large-v3".to_string(),
            ],
            requires_api_key: false,
        }
    }

    async fn transcribe(
        &self,
        audio: AudioData,
        options: TranscriptionOptions,
    ) -> Result<Transcript, ProviderError> {
        if audio.wav_bytes.is_empty() {
            return Err(ProviderError::InvalidAudio("Audio data is empty".to_string()));
        }

        let requested_id = options.model.clone().unwrap_or_else(|| "base".to_string());
        let requested_device = options
            .compute_device
            .as_deref()
            .unwrap_or("cpu");
        let device_info = local_compute_device_info(requested_device);
        if requested_device == "gpu" && !device_info.gpu_available {
            return Err(ProviderError::ModelError(
                "GPU mode was selected, but the Vulkan Whisper backend is unavailable. Install a Vulkan-capable build and GPU driver, or select CPU mode.".to_string(),
            ));
        }
        let external_model_path = std::env::var_os("FORGE_WHISPER_MODEL_PATH")
            .map(PathBuf::from)
            .filter(|path| path.is_file());
        let models = self.model_manager.list_available_models();
        let target_model = models.iter().find(|m| m.id == requested_id);
        let effective_model_id = if target_model.map(|m| m.is_installed).unwrap_or(false) {
            requested_id
        } else if let Some(auto_picked) = self.model_manager.auto_pick_installed_model() {
            auto_picked.id
        } else {
            return Err(ProviderError::ModelError(
                "No offline Whisper model found. Download a model first.".to_string(),
            ));
        };
        let model_path = if let Some(path) = external_model_path {
            path
        } else {
            let model_info = models
                .iter()
                .find(|model| model.id == effective_model_id)
                .ok_or_else(|| ProviderError::ModelError("Selected model is unavailable".to_string()))?;
            self.model_manager
                .find_model_file(&model_info.filename)
                .ok_or_else(|| ProviderError::ModelError("Selected model file is missing".to_string()))?
        };
        let language = normalize_language(options.language.as_deref());
        let inference_language = language.clone();
        let duration_ms = audio.duration_ms;
        let transcription_started = Instant::now();
        let gpu_device_id = if device_info.active_backend == "vulkan" {
            gpu_device_info().map(|(device_id, _)| device_id)
        } else {
            None
        };
        let context_key = WhisperContextKey {
            model_path: model_path.clone(),
            backend: device_info.active_backend.clone(),
            gpu_device_id,
        };

        info!(
            target: "forge.local_whisper",
            phase = "transcription_started",
            model_id = %effective_model_id,
            model_path = %model_path.display(),
            audio_duration_ms = duration_ms,
            active_backend = %device_info.active_backend,
            gpu_acceleration = device_info.active_backend == "vulkan",
            "Starting Local Whisper transcription"
        );

        let inference_device_info = device_info.clone();
        let context_cache = Arc::clone(&self.context_cache);
        let text = tokio::task::spawn_blocking(move || {
            let mut reader = hound::WavReader::new(Cursor::new(audio.wav_bytes))
                .map_err(|e| ProviderError::InvalidAudio(e.to_string()))?;
            let spec = reader.spec();
            if spec.sample_rate != 16_000 || spec.channels != 1 || spec.bits_per_sample != 16 {
                return Err(ProviderError::InvalidAudio(
                    "Local Whisper requires 16-bit mono 16 kHz WAV audio".to_string(),
                ));
            }
            let samples = reader
                .samples::<i16>()
                .map(|sample| sample.map(|value| value as f32 / i16::MAX as f32))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| ProviderError::InvalidAudio(e.to_string()))?;
            let context = cached_context(
                &context_cache,
                context_key,
                &model_path,
                &inference_device_info,
            )?;
            let mut state = context
                .create_state()
                .map_err(|e| ProviderError::ModelError(e.to_string()))?;
            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });
            params.set_print_special(false);
            params.set_print_progress(false);
            params.set_print_realtime(false);
            params.set_print_timestamps(false);
            if inference_language != "auto" {
                params.set_language(Some(&inference_language));
            }
            state
                .full(params, &samples)
                .map_err(|e| ProviderError::InternalError(e.to_string()))?;
            let transcript = state
                .as_iter()
                .map(|segment| segment.to_string())
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string();
            if transcript.is_empty() {
                return Err(ProviderError::InvalidAudio("No speech detected".to_string()));
            }
            Ok::<_, ProviderError>(transcript)
        })
        .await
        .map_err(|e| ProviderError::InternalError(e.to_string()))??;

        info!(
            target: "forge.local_whisper",
            phase = "transcription_completed",
            model_id = %effective_model_id,
            audio_duration_ms = duration_ms,
            elapsed_ms = transcription_started.elapsed().as_millis() as u64,
            active_backend = %device_info.active_backend,
            gpu_acceleration = device_info.active_backend == "vulkan",
            "Local Whisper transcription completed"
        );

        Ok(Transcript {
            text,
            language,
            provider: "local-whisper".to_string(),
            model: effective_model_id,
            duration_ms,
            confidence: None,
        })
    }
}

fn cached_context(
    cache: &WhisperContextCache,
    key: WhisperContextKey,
    model_path: &Path,
    device_info: &LocalComputeDeviceInfo,
) -> Result<Arc<WhisperContext>, ProviderError> {
    let mut cached = cache.lock().unwrap();
    if let Some((cached_key, context)) = cached.as_ref() {
        if cached_key == &key {
            info!(
                target: "forge.local_whisper",
                phase = "model_cache_hit",
                active_backend = %device_info.active_backend,
                gpu_acceleration = device_info.active_backend == "vulkan",
                "Reusing cached Local Whisper model context"
            );
            return Ok(Arc::clone(context));
        }
    }

    let load_started = Instant::now();
    let mut context_params = WhisperContextParameters::default();
    context_params.use_gpu(device_info.active_backend == "vulkan");
    if let Some(device_id) = key.gpu_device_id {
        context_params.gpu_device(device_id);
    }
    let context = Arc::new(
        WhisperContext::new_with_params(model_path.to_string_lossy().as_ref(), context_params)
            .map_err(|e| ProviderError::ModelError(e.to_string()))?,
    );
    *cached = Some((key, Arc::clone(&context)));
    info!(
        target: "forge.local_whisper",
        phase = "model_loaded",
        active_backend = %device_info.active_backend,
        gpu_acceleration = device_info.active_backend == "vulkan",
        elapsed_ms = load_started.elapsed().as_millis() as u64,
        "Loaded Local Whisper model context"
    );
    Ok(context)
}

#[cfg(test)]
mod tests {
    use super::{
        local_compute_device_info, sha256_file, verify_model_file, LocalWhisperProvider,
        ModelManager, WhisperContextKey,
    };
    use forge_transcription::{AudioData, ModelFamily, ModelFormat, TranscriptionOptions, TranscriptionProvider};
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    #[tokio::test]
    async fn missing_model_returns_a_clear_error() {
        let dir = tempdir().unwrap();
        let provider = LocalWhisperProvider::new(Arc::new(ModelManager {
            models_dir: dir.path().to_path_buf(),
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
            search_fallback_paths: false,
        }));
        let result = provider
            .transcribe(
                AudioData::new(vec![1, 2, 3], 16_000, 1, 10),
                TranscriptionOptions::default(),
            )
            .await;

        assert!(matches!(result, Err(forge_transcription::ProviderError::ModelError(_))));
    }

    #[test]
    fn sha256_file_returns_known_digest() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("model.bin");
        fs::write(&path, b"forge-whisper-model").unwrap();

        assert_eq!(
            sha256_file(&path).unwrap(),
            "6dd540f2a57fb5991926c339ace7368d33cc400cf9090775e93defa243b5c373"
        );
    }

    #[test]
    fn catalog_metadata_is_pinned_and_hashed() {
        for model in LocalWhisperProvider::default().model_manager.list_available_models() {
            assert_eq!(model.revision.len(), 40);
            assert_eq!(model.sha256.len(), 64);
            assert!(model.download_url.contains(&model.revision));
            assert!(model.size_bytes > 0);
        }
    }

    #[test]
    fn parakeet_catalog_entry_uses_verified_directory_metadata() {
        let models = LocalWhisperProvider::default().model_manager.list_available_models();
        let parakeet = models.iter().find(|model| model.id == "parakeet-v3-int8").unwrap();
        assert_eq!(parakeet.family, ModelFamily::Parakeet);
        assert_eq!(parakeet.format, ModelFormat::OnnxDirectory);
        assert_eq!(parakeet.filename, "parakeet-tdt-0.6b-v3-int8");
        assert_eq!(parakeet.sha256.len(), 64);
    }

    #[test]
    fn cpu_compute_device_is_always_available() {
        let info = local_compute_device_info("cpu");

        assert_eq!(info.requested_device, "cpu");
        assert_eq!(info.active_backend, "cpu");
        assert!(info.reason.contains("CPU mode"));
    }

    #[test]
    fn unknown_compute_device_defaults_to_cpu() {
        let info = local_compute_device_info("unknown");

        assert_eq!(info.requested_device, "cpu");
        assert_eq!(info.active_backend, "cpu");
    }

    #[test]
    fn context_keys_separate_cpu_and_vulkan_backends() {
        let path = PathBuf::from("model.bin");
        let cpu_key = WhisperContextKey {
            model_path: path.clone(),
            backend: "cpu".to_string(),
            gpu_device_id: None,
        };
        let gpu_key = WhisperContextKey {
            model_path: path,
            backend: "vulkan".to_string(),
            gpu_device_id: Some(1),
        };

        assert_ne!(cpu_key, gpu_key);
    }

    #[test]
    fn context_keys_separate_vulkan_devices() {
        let path = PathBuf::from("model.bin");
        let first_gpu_key = WhisperContextKey {
            model_path: path.clone(),
            backend: "vulkan".to_string(),
            gpu_device_id: Some(0),
        };
        let second_gpu_key = WhisperContextKey {
            model_path: path,
            backend: "vulkan".to_string(),
            gpu_device_id: Some(1),
        };

        assert_ne!(first_gpu_key, second_gpu_key);
    }

    #[test]
    fn changing_active_model_clears_context_cache() {
        let provider = LocalWhisperProvider::default();
        *provider.context_cache.lock().unwrap() = None;

        provider.set_active_model("small");

        assert!(provider.context_cache.lock().unwrap().is_none());
    }

    #[test]
    fn model_integrity_accepts_matching_size_and_hash() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("model.bin");
        fs::write(&path, b"forge-whisper-model").unwrap();

        verify_model_file(
            &path,
            19,
            "6dd540f2a57fb5991926c339ace7368d33cc400cf9090775e93defa243b5c373",
        )
        .unwrap();
    }

    #[test]
    fn model_integrity_rejects_mismatch() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("model.bin");
        fs::write(&path, b"corrupt-model").unwrap();

        let error = verify_model_file(
            &path,
            19,
            "6dd540f2a57fb5991926c339ace7368d33cc400cf9090775e93defa243b5c373",
        )
        .unwrap_err();
        assert!(error.contains("size mismatch") || error.contains("checksum mismatch"));
    }
}
