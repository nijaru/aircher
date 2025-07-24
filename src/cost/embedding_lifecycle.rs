use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tokio::fs as async_fs;
use tracing::{debug, info};
use chrono::{DateTime, Utc};

use super::EmbeddingModel;
use crate::utils::xdg_dirs::{XdgDirs, AircherFileType};

/// Manages the lifecycle of embedding models including versions, updates, and cleanup
pub struct EmbeddingLifecycleManager {
    storage_dir: PathBuf,
    registry: ModelRegistry,
}

/// Registry tracking all installed models and their metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistry {
    pub models: HashMap<String, ModelEntry>,
    pub current_default: String,
    pub auto_update: bool,
    pub last_update_check: DateTime<Utc>,
}

/// Individual model entry with version and usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub model: EmbeddingModel,
    pub version: String,
    pub installed_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub checksum: String,
    pub compatible_indices: Vec<String>, // Projects using this model version
}

/// Storage information for cleanup operations
#[derive(Debug)]
pub struct StorageInfo {
    pub total_size: u64,
    pub model_count: usize,
    pub index_count: usize,
    pub unused_models: Vec<String>,
    pub stale_indices: Vec<String>,
}

/// Update information for available model versions
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub model_name: String,
    pub current_version: String,
    pub latest_version: String,
    pub improvement_description: String,
    pub download_size: u64,
    pub breaking_change: bool,
}

impl EmbeddingLifecycleManager {
    pub fn new() -> Result<Self> {
        let storage_dir = Self::get_storage_dir()?;
        fs::create_dir_all(&storage_dir)
            .with_context(|| format!("Failed to create storage directory: {:?}", storage_dir))?;
        
        let registry = Self::load_or_create_registry(&storage_dir)?;
        
        Ok(Self {
            storage_dir,
            registry,
        })
    }
    
    fn get_storage_dir() -> Result<PathBuf> {
        XdgDirs::aircher_data_dir()
    }
    
    fn load_or_create_registry(storage_dir: &PathBuf) -> Result<ModelRegistry> {
        let registry_path = storage_dir.join("model_registry.json");
        
        if registry_path.exists() {
            let content = fs::read_to_string(&registry_path)
                .with_context(|| "Failed to read model registry")?;
            let registry: ModelRegistry = serde_json::from_str(&content)
                .with_context(|| "Failed to parse model registry")?;
            debug!("Loaded model registry with {} models", registry.models.len());
            Ok(registry)
        } else {
            info!("Creating new model registry");
            let registry = ModelRegistry {
                models: HashMap::new(),
                current_default: "swerank-embed-small".to_string(),
                auto_update: true,
                last_update_check: Utc::now(),
            };
            Ok(registry)
        }
    }
    
    fn save_registry(&self) -> Result<()> {
        let registry_path = self.storage_dir.join("model_registry.json");
        let content = serde_json::to_string_pretty(&self.registry)
            .with_context(|| "Failed to serialize model registry")?;
        fs::write(&registry_path, content)
            .with_context(|| "Failed to write model registry")?;
        debug!("Saved model registry");
        Ok(())
    }
    
    /// Check for available updates to installed models
    pub async fn check_for_updates(&mut self) -> Result<Vec<UpdateInfo>> {
        info!("Checking for model updates...");
        
        let mut updates = Vec::new();
        
        // Check SweRankEmbed-Small updates
        if let Some(entry) = self.registry.models.get("swerank-embed-small") {
            if let Some(update) = self.check_swerank_update(&entry).await? {
                updates.push(update);
            }
        }
        
        // Update last check time
        self.registry.last_update_check = Utc::now();
        self.save_registry()?;
        
        info!("Found {} available updates", updates.len());
        Ok(updates)
    }
    
    async fn check_swerank_update(&self, current: &ModelEntry) -> Result<Option<UpdateInfo>> {
        // In a real implementation, this would check a remote registry
        // For now, simulate version checking
        let current_version = &current.version;
        let latest_version = "1.1.0"; // Simulated latest version
        
        if current_version != latest_version {
            Ok(Some(UpdateInfo {
                model_name: "swerank-embed-small".to_string(),
                current_version: current_version.clone(),
                latest_version: latest_version.to_string(),
                improvement_description: "15% better code search accuracy, reduced model size".to_string(),
                download_size: 120_000_000, // 120MB
                breaking_change: false,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Install or update a model to a specific version
    pub async fn install_model(&mut self, model: &EmbeddingModel, version: &str) -> Result<()> {
        info!("Installing {} v{}", model.name, version);
        
        let model_dir = self.storage_dir.join("models");
        fs::create_dir_all(&model_dir)?;
        
        let filename = format!("{}-v{}.onnx", model.name, version);
        let file_path = model_dir.join(&filename);
        
        // Simulate download (in real implementation, download from URL)
        self.simulate_download(&file_path, model.size_mb as u64 * 1_000_000).await?;
        
        // Calculate checksum
        let checksum = self.calculate_checksum(&file_path)?;
        
        // Create model entry
        let entry = ModelEntry {
            model: model.clone(),
            version: version.to_string(),
            installed_at: Utc::now(),
            last_used: Utc::now(),
            file_path: file_path.clone(),
            file_size: model.size_mb as u64 * 1_000_000,
            checksum,
            compatible_indices: Vec::new(),
        };
        
        // Update registry
        let model_key = format!("{}-v{}", model.name, version);
        self.registry.models.insert(model_key, entry);
        
        // Update default if this is the preferred model
        if model.name == "swerank-embed-small" {
            self.registry.current_default = format!("{}-v{}", model.name, version);
        }
        
        self.save_registry()?;
        info!("Successfully installed {} v{}", model.name, version);
        Ok(())
    }
    
    async fn simulate_download(&self, file_path: &PathBuf, size: u64) -> Result<()> {
        // Simulate file creation with progress
        debug!("Simulating download to {:?} ({} bytes)", file_path, size);
        
        // Create empty file of specified size
        let content = vec![0u8; size as usize];
        async_fs::write(file_path, content).await
            .with_context(|| "Failed to write model file")?;
        
        Ok(())
    }
    
    fn calculate_checksum(&self, file_path: &PathBuf) -> Result<String> {
        use sha2::{Sha256, Digest};
        
        let content = fs::read(file_path)
            .with_context(|| "Failed to read file for checksum")?;
        
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let result = hasher.finalize();
        
        Ok(format!("{:x}", result))
    }
    
    /// Get storage usage information
    pub fn get_storage_info(&self) -> Result<StorageInfo> {
        let mut total_size = 0;
        let mut unused_models = Vec::new();
        let mut stale_indices = Vec::new();
        
        // Calculate model storage
        for (key, entry) in &self.registry.models {
            total_size += entry.file_size;
            
            // Mark as unused if not used in 30 days
            let thirty_days_ago = Utc::now() - chrono::Duration::days(30);
            if entry.last_used < thirty_days_ago && key != &self.registry.current_default {
                unused_models.push(key.clone());
            }
        }
        
        // Check for stale indices
        let indices_dir = self.storage_dir.join("indices");
        if indices_dir.exists() {
            for entry in fs::read_dir(&indices_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    let modified = entry.metadata()?.modified()?;
                    let modified_time = DateTime::<Utc>::from(modified);
                    let thirty_days_ago = Utc::now() - chrono::Duration::days(30);
                    
                    if modified_time < thirty_days_ago {
                        stale_indices.push(entry.file_name().to_string_lossy().to_string());
                    }
                }
            }
        }
        
        Ok(StorageInfo {
            total_size,
            model_count: self.registry.models.len(),
            index_count: stale_indices.len() + self.count_active_indices()?,
            unused_models,
            stale_indices,
        })
    }
    
    fn count_active_indices(&self) -> Result<usize> {
        let indices_dir = self.storage_dir.join("indices");
        if !indices_dir.exists() {
            return Ok(0);
        }
        
        let count = fs::read_dir(&indices_dir)?.count();
        Ok(count)
    }
    
    /// Clean up unused models and stale indices
    pub async fn cleanup(&mut self, remove_unused: bool, remove_stale: bool) -> Result<u64> {
        let mut cleaned_size = 0;
        
        if remove_unused {
            cleaned_size += self.cleanup_unused_models().await?;
        }
        
        if remove_stale {
            cleaned_size += self.cleanup_stale_indices().await?;
        }
        
        self.save_registry()?;
        info!("Cleanup completed, freed {} bytes", cleaned_size);
        Ok(cleaned_size)
    }
    
    async fn cleanup_unused_models(&mut self) -> Result<u64> {
        let storage_info = self.get_storage_info()?;
        let mut freed_size = 0;
        
        for unused_key in &storage_info.unused_models {
            if let Some(entry) = self.registry.models.remove(unused_key) {
                if entry.file_path.exists() {
                    freed_size += entry.file_size;
                    async_fs::remove_file(&entry.file_path).await
                        .with_context(|| format!("Failed to remove model file: {:?}", entry.file_path))?;
                    info!("Removed unused model: {} ({}MB)", unused_key, entry.file_size / 1_000_000);
                }
            }
        }
        
        Ok(freed_size)
    }
    
    async fn cleanup_stale_indices(&self) -> Result<u64> {
        let storage_info = self.get_storage_info()?;
        let mut freed_size = 0;
        
        let indices_dir = self.storage_dir.join("indices");
        for stale_index in &storage_info.stale_indices {
            let index_path = indices_dir.join(stale_index);
            if index_path.exists() {
                let size = self.calculate_dir_size(&index_path)?;
                freed_size += size;
                
                async_fs::remove_dir_all(&index_path).await
                    .with_context(|| format!("Failed to remove stale index: {:?}", index_path))?;
                info!("Removed stale index: {} ({}MB)", stale_index, size / 1_000_000);
            }
        }
        
        Ok(freed_size)
    }
    
    fn calculate_dir_size(&self, path: &PathBuf) -> Result<u64> {
        let mut size = 0;
        
        fn visit_dir(dir: &PathBuf, size: &mut u64) -> Result<()> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    visit_dir(&path, size)?;
                } else {
                    *size += entry.metadata()?.len();
                }
            }
            Ok(())
        }
        
        visit_dir(path, &mut size)?;
        Ok(size)
    }
    
    /// Mark a model as used (updates last_used timestamp)
    pub fn mark_model_used(&mut self, model_key: &str) -> Result<()> {
        if let Some(entry) = self.registry.models.get_mut(model_key) {
            entry.last_used = Utc::now();
            self.save_registry()?;
        }
        Ok(())
    }
    
    /// Get the current default model entry
    pub fn get_current_model(&self) -> Option<&ModelEntry> {
        self.registry.models.get(&self.registry.current_default)
    }
    
    /// List all installed models with status
    pub fn list_models(&self) -> Vec<(&String, &ModelEntry, bool)> {
        self.registry.models.iter()
            .map(|(key, entry)| (key, entry, key == &self.registry.current_default))
            .collect()
    }
    
    /// Set the default model
    pub fn set_default_model(&mut self, model_key: &str) -> Result<()> {
        if self.registry.models.contains_key(model_key) {
            self.registry.current_default = model_key.to_string();
            self.save_registry()?;
            info!("Set default model to: {}", model_key);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Model not found: {}", model_key))
        }
    }
}

impl Default for EmbeddingLifecycleManager {
    fn default() -> Self {
        Self::new().expect("Failed to create EmbeddingLifecycleManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_lifecycle_manager_creation() {
        let manager = EmbeddingLifecycleManager::new();
        assert!(manager.is_ok());
    }
    
    #[tokio::test]
    async fn test_storage_info() {
        let manager = EmbeddingLifecycleManager::new().unwrap();
        let info = manager.get_storage_info();
        assert!(info.is_ok());
    }
    
    #[test]
    fn test_checksum_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let storage_dir = temp_dir.path().to_path_buf();
        let registry = ModelRegistry {
            models: HashMap::new(),
            current_default: "test".to_string(),
            auto_update: true,
            last_update_check: Utc::now(),
        };
        
        let manager = EmbeddingLifecycleManager {
            storage_dir,
            registry,
        };
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, b"test content").unwrap();
        
        let checksum = manager.calculate_checksum(&test_file);
        assert!(checksum.is_ok());
        assert!(!checksum.unwrap().is_empty());
    }
}