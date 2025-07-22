use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{info, warn};

/// A saved search preset with filters and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPreset {
    pub name: String,
    pub description: String,
    pub filters: SearchFilters,
    pub created_at: String,
    pub usage_count: u32,
}

/// Search filter configuration for presets
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    pub file_types: Option<Vec<String>>,
    pub languages: Option<Vec<String>>,
    pub scope: Option<Vec<String>>,
    pub chunk_types: Option<Vec<String>>,
    pub min_similarity: Option<f32>,
    pub max_similarity: Option<f32>,
    pub exclude: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
    pub limit: Option<usize>,
}

/// Manages search presets with global and project-local storage
pub struct PresetManager {
    project_presets_path: PathBuf,
    global_presets_path: PathBuf,
    cache: Option<HashMap<String, SearchPreset>>,
}

impl PresetManager {
    /// Create new preset manager
    pub fn new() -> Result<Self> {
        let project_presets_path = PathBuf::from(".aircher/presets");
        
        let global_presets_path = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("aircher")
            .join("presets");

        Ok(Self {
            project_presets_path,
            global_presets_path,
            cache: None,
        })
    }

    /// Load all presets (global + project-local)
    pub async fn load_presets(&mut self) -> Result<HashMap<String, SearchPreset>> {
        let mut presets = HashMap::new();

        // Load global presets first
        if let Ok(global_presets) = self.load_presets_from_dir(&self.global_presets_path).await {
            presets.extend(global_presets);
        }

        // Load project presets (override global ones with same name)
        if let Ok(project_presets) = self.load_presets_from_dir(&self.project_presets_path).await {
            presets.extend(project_presets);
        }

        self.cache = Some(presets.clone());
        Ok(presets)
    }

    /// Load presets from a specific directory
    async fn load_presets_from_dir(&self, dir: &Path) -> Result<HashMap<String, SearchPreset>> {
        let mut presets = HashMap::new();

        if !dir.exists() {
            return Ok(presets);
        }

        let mut entries = fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_preset_file(&path).await {
                    Ok(preset) => {
                        presets.insert(preset.name.clone(), preset);
                    }
                    Err(e) => {
                        warn!("Failed to load preset from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(presets)
    }

    /// Load a single preset file
    async fn load_preset_file(&self, path: &Path) -> Result<SearchPreset> {
        let content = fs::read_to_string(path).await?;
        let preset: SearchPreset = serde_json::from_str(&content)?;
        Ok(preset)
    }

    /// Save a preset (project-local by default, global if specified)
    pub async fn save_preset(&self, preset: &SearchPreset, global: bool) -> Result<()> {
        let dir = if global {
            &self.global_presets_path
        } else {
            &self.project_presets_path
        };

        // Create directory if it doesn't exist
        fs::create_dir_all(dir).await?;

        let filename = format!("{}.json", preset.name.replace(' ', "_").to_lowercase());
        let file_path = dir.join(filename);

        let content = serde_json::to_string_pretty(preset)?;
        fs::write(&file_path, content).await?;

        info!("Saved preset '{}' to {:?}", preset.name, file_path);
        Ok(())
    }

    /// Get a preset by name
    pub async fn get_preset(&mut self, name: &str) -> Result<Option<SearchPreset>> {
        if self.cache.is_none() {
            self.load_presets().await?;
        }

        if let Some(ref cache) = self.cache {
            Ok(cache.get(name).cloned())
        } else {
            Ok(None)
        }
    }

    /// List all available presets
    pub async fn list_presets(&mut self) -> Result<Vec<SearchPreset>> {
        if self.cache.is_none() {
            self.load_presets().await?;
        }

        if let Some(ref cache) = self.cache {
            let mut presets: Vec<SearchPreset> = cache.values().cloned().collect();
            presets.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(presets)
        } else {
            Ok(vec![])
        }
    }

    /// Delete a preset
    pub async fn delete_preset(&mut self, name: &str, global: bool) -> Result<bool> {
        let dir = if global {
            &self.global_presets_path
        } else {
            &self.project_presets_path
        };

        let filename = format!("{}.json", name.replace(' ', "_").to_lowercase());
        let file_path = dir.join(filename);

        if file_path.exists() {
            fs::remove_file(&file_path).await?;
            info!("Deleted preset '{}' from {:?}", name, file_path);
            
            // Invalidate cache
            self.cache = None;
            
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Increment usage count for a preset
    pub async fn increment_usage(&mut self, name: &str) -> Result<()> {
        if let Some(mut preset) = self.get_preset(name).await? {
            preset.usage_count += 1;
            
            // Save to project-local by default
            self.save_preset(&preset, false).await?;
            
            // Invalidate cache
            self.cache = None;
        }
        
        Ok(())
    }

    /// Create built-in presets for common use cases
    pub async fn create_builtin_presets(&mut self) -> Result<()> {
        let builtins = vec![
            SearchPreset {
                name: "rust-functions".to_string(),
                description: "Rust functions and methods".to_string(),
                filters: SearchFilters {
                    file_types: Some(vec!["rust".to_string()]),
                    scope: Some(vec!["functions".to_string()]),
                    chunk_types: Some(vec!["function".to_string()]),
                    ..Default::default()
                },
                created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                usage_count: 0,
            },
            SearchPreset {
                name: "auth-security".to_string(),
                description: "Authentication and security patterns".to_string(),
                filters: SearchFilters {
                    scope: Some(vec!["functions".to_string(), "classes".to_string()]),
                    include: Some(vec!["auth".to_string(), "security".to_string(), "login".to_string()]),
                    exclude: Some(vec!["test".to_string(), "mock".to_string()]),
                    min_similarity: Some(0.7),
                    ..Default::default()
                },
                created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                usage_count: 0,
            },
            SearchPreset {
                name: "error-handling".to_string(),
                description: "Error handling and exception patterns".to_string(),
                filters: SearchFilters {
                    scope: Some(vec!["functions".to_string()]),
                    include: Some(vec!["error".to_string(), "exception".to_string(), "result".to_string()]),
                    exclude: Some(vec!["test".to_string()]),
                    min_similarity: Some(0.6),
                    ..Default::default()
                },
                created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                usage_count: 0,
            },
            SearchPreset {
                name: "config-patterns".to_string(),
                description: "Configuration and settings code".to_string(),
                filters: SearchFilters {
                    include: Some(vec!["config".to_string(), "settings".to_string(), "env".to_string()]),
                    exclude: Some(vec!["test".to_string(), "example".to_string()]),
                    ..Default::default()
                },
                created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                usage_count: 0,
            },
        ];

        for preset in builtins {
            // Only save if it doesn't already exist
            if self.get_preset(&preset.name).await?.is_none() {
                self.save_preset(&preset, true).await?;
            }
        }

        Ok(())
    }
}

impl SearchFilters {
    /// Convert to CLI arguments format for easy integration
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(ref file_types) = self.file_types {
            args.push("--file-types".to_string());
            args.push(file_types.join(","));
        }

        if let Some(ref languages) = self.languages {
            args.push("--languages".to_string());
            args.push(languages.join(","));
        }

        if let Some(ref scope) = self.scope {
            args.push("--scope".to_string());
            args.push(scope.join(","));
        }

        if let Some(ref chunk_types) = self.chunk_types {
            args.push("--chunk-types".to_string());
            args.push(chunk_types.join(","));
        }

        if let Some(min_sim) = self.min_similarity {
            args.push("--min-similarity".to_string());
            args.push(min_sim.to_string());
        }

        if let Some(max_sim) = self.max_similarity {
            args.push("--max-similarity".to_string());
            args.push(max_sim.to_string());
        }

        if let Some(ref exclude) = self.exclude {
            args.push("--exclude".to_string());
            args.push(exclude.join(","));
        }

        if let Some(ref include) = self.include {
            args.push("--include".to_string());
            args.push(include.join(","));
        }

        if let Some(limit) = self.limit {
            args.push("--limit".to_string());
            args.push(limit.to_string());
        }

        args
    }

    /// Create from CLI search arguments
    pub fn from_cli_args(
        file_types: &Option<Vec<String>>,
        languages: &Option<Vec<String>>,
        scope: &Option<Vec<String>>,
        chunk_types: &Option<Vec<String>>,
        min_similarity: Option<f32>,
        max_similarity: Option<f32>,
        exclude: &Option<Vec<String>>,
        include: &Option<Vec<String>>,
        limit: Option<usize>,
    ) -> Self {
        Self {
            file_types: file_types.clone(),
            languages: languages.clone(),
            scope: scope.clone(),
            chunk_types: chunk_types.clone(),
            min_similarity,
            max_similarity,
            exclude: exclude.clone(),
            include: include.clone(),
            limit,
        }
    }

    /// Format for display
    pub fn format_summary(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref file_types) = self.file_types {
            parts.push(format!("files: {}", file_types.join(",")));
        }

        if let Some(ref scope) = self.scope {
            parts.push(format!("scope: {}", scope.join(",")));
        }

        if let Some(min_sim) = self.min_similarity {
            parts.push(format!("min-sim: {:.2}", min_sim));
        }

        if let Some(ref include) = self.include {
            parts.push(format!("include: {}", include.join(",")));
        }

        if let Some(ref exclude) = self.exclude {
            parts.push(format!("exclude: {}", exclude.join(",")));
        }

        if parts.is_empty() {
            "no filters".to_string()
        } else {
            parts.join(", ")
        }
    }
}

impl Default for PresetManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            project_presets_path: PathBuf::from(".aircher/presets"),
            global_presets_path: PathBuf::from("./presets"),
            cache: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_preset_creation() {
        let preset = SearchPreset {
            name: "test-preset".to_string(),
            description: "Test preset".to_string(),
            filters: SearchFilters {
                file_types: Some(vec!["rust".to_string()]),
                scope: Some(vec!["functions".to_string()]),
                ..Default::default()
            },
            created_at: "2025-01-01 00:00:00 UTC".to_string(),
            usage_count: 0,
        };

        assert_eq!(preset.name, "test-preset");
        assert!(preset.filters.file_types.is_some());
    }

    #[tokio::test]
    async fn test_cli_args_conversion() {
        let filters = SearchFilters {
            file_types: Some(vec!["rust".to_string(), "python".to_string()]),
            scope: Some(vec!["functions".to_string()]),
            min_similarity: Some(0.8),
            ..Default::default()
        };

        let args = filters.to_cli_args();
        assert!(args.contains(&"--file-types".to_string()));
        assert!(args.contains(&"rust,python".to_string()));
        assert!(args.contains(&"--min-similarity".to_string()));
        assert!(args.contains(&"0.8".to_string()));
    }
}