use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// TOML-based configuration for Aircher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcherConfig {
    pub providers: HashMap<String, ProviderConfig>,
    pub embedding: EmbeddingConfig,
    pub search: SearchConfig,
    pub ui: UiConfig,
    pub session: SessionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub default_model: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub provider: String,          // "ollama", "embedded", or "api"
    pub model: String,
    pub auto_setup: bool,
    pub similarity_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub max_results: usize,
    pub auto_index: bool,
    pub index_on_startup: bool,
    pub cache_embeddings: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub default_interface: String,  // "tui", "cli", or "auto"
    pub theme: String,
    pub auto_save_sessions: bool,
    pub startup_message: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub max_history_size: usize,
    pub auto_export: bool,
    pub export_format: String,      // "json", "markdown", "csv"
    pub cleanup_old_sessions: bool,
    pub max_session_age_days: u32,
}

impl Default for ArcherConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();
        
        // Claude (default)
        providers.insert("claude".to_string(), ProviderConfig {
            api_key: None,
            base_url: None,
            default_model: "claude-3-5-sonnet-20241022".to_string(),
            enabled: true,
        });
        
        // OpenAI
        providers.insert("openai".to_string(), ProviderConfig {
            api_key: None,
            base_url: None,
            default_model: "gpt-4o".to_string(),
            enabled: true,
        });
        
        // Gemini
        providers.insert("gemini".to_string(), ProviderConfig {
            api_key: None,
            base_url: None,
            default_model: "gemini-2.0-flash-exp".to_string(),
            enabled: true,
        });
        
        // Ollama (local)
        providers.insert("ollama".to_string(), ProviderConfig {
            api_key: None,
            base_url: Some("http://localhost:11434".to_string()),
            default_model: "llama3.3".to_string(),
            enabled: true,
        });
        
        // OpenRouter
        providers.insert("openrouter".to_string(), ProviderConfig {
            api_key: None,
            base_url: Some("https://openrouter.ai/api/v1".to_string()),
            default_model: "anthropic/claude-3.5-sonnet".to_string(),
            enabled: false,
        });

        Self {
            providers,
            embedding: EmbeddingConfig {
                provider: "ollama".to_string(),
                model: "nomic-embed-text".to_string(),
                auto_setup: true,
                similarity_threshold: 0.3,
            },
            search: SearchConfig {
                max_results: 10,
                auto_index: true,
                index_on_startup: false,
                cache_embeddings: true,
            },
            ui: UiConfig {
                default_interface: "tui".to_string(),
                theme: "dark".to_string(),
                auto_save_sessions: true,
                startup_message: true,
            },
            session: SessionConfig {
                max_history_size: 1000,
                auto_export: false,
                export_format: "json".to_string(),
                cleanup_old_sessions: true,
                max_session_age_days: 30,
            },
        }
    }
}

impl ArcherConfig {
    /// Get the configuration directory path
    pub fn config_dir() -> Result<PathBuf> {
        ProjectDirs::from("", "", "aircher")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .context("Failed to determine config directory")
    }
    
    /// Get the cache directory path
    pub fn cache_dir() -> Result<PathBuf> {
        ProjectDirs::from("", "", "aircher")
            .map(|dirs| dirs.cache_dir().to_path_buf())
            .context("Failed to determine cache directory")
    }
    
    /// Get the data directory path
    pub fn data_dir() -> Result<PathBuf> {
        ProjectDirs::from("", "", "aircher")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .context("Failed to determine data directory")
    }

    /// Get the main config file path
    pub fn config_file_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Load configuration from file, creating default if it doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        
        if config_path.exists() {
            debug!("Loading config from: {:?}", config_path);
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {:?}", config_path))?;
            
            let config: ArcherConfig = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;
            
            info!("Configuration loaded successfully");
            Ok(config)
        } else {
            info!("Config file not found, creating default: {:?}", config_path);
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        
        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config to TOML")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        debug!("Configuration saved to: {:?}", config_path);
        Ok(())
    }

    /// Get a provider configuration
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.get(name)
    }

    /// Update a provider configuration
    pub fn set_provider(&mut self, name: String, config: ProviderConfig) {
        self.providers.insert(name, config);
    }

    /// Get the default/current provider
    pub fn default_provider(&self) -> &str {
        // Return first enabled provider, preferring claude
        if let Some(claude) = self.providers.get("claude") {
            if claude.enabled && claude.api_key.is_some() {
                return "claude";
            }
        }
        
        for (name, config) in &self.providers {
            if config.enabled && (config.api_key.is_some() || name == "ollama") {
                return name;
            }
        }
        
        "claude" // Fallback
    }

    /// Update a single configuration value
    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        let parts: Vec<&str> = key.split('.').collect();
        
        match parts.as_slice() {
            ["ui", "theme"] => self.ui.theme = value.to_string(),
            ["ui", "default_interface"] => self.ui.default_interface = value.to_string(),
            ["embedding", "provider"] => self.embedding.provider = value.to_string(),
            ["embedding", "model"] => self.embedding.model = value.to_string(),
            ["search", "max_results"] => {
                self.search.max_results = value.parse()
                    .context("Invalid number for search.max_results")?;
            }
            ["providers", provider, "default_model"] => {
                let provider_key = provider.to_string();
                if let Some(config) = self.providers.get_mut(&provider_key) {
                    config.default_model = value.to_string();
                } else {
                    anyhow::bail!("Unknown provider: {}", provider);
                }
            }
            ["providers", provider, "api_key"] => {
                let provider_key = provider.to_string();
                if let Some(config) = self.providers.get_mut(&provider_key) {
                    config.api_key = if value.is_empty() { None } else { Some(value.to_string()) };
                } else {
                    anyhow::bail!("Unknown provider: {}", provider);
                }
            }
            _ => anyhow::bail!("Unknown config key: {}", key),
        }
        
        Ok(())
    }

    /// Get a configuration value as string
    pub fn get_value(&self, key: &str) -> Result<Option<String>> {
        let parts: Vec<&str> = key.split('.').collect();
        
        match parts.as_slice() {
            ["ui", "theme"] => Ok(Some(self.ui.theme.clone())),
            ["ui", "default_interface"] => Ok(Some(self.ui.default_interface.clone())),
            ["embedding", "provider"] => Ok(Some(self.embedding.provider.clone())),
            ["embedding", "model"] => Ok(Some(self.embedding.model.clone())),
            ["search", "max_results"] => Ok(Some(self.search.max_results.to_string())),
            ["providers", provider, "default_model"] => {
                Ok(self.providers.get(*provider)
                    .map(|c| c.default_model.clone()))
            }
            ["providers", provider, "api_key"] => {
                Ok(self.providers.get(*provider)
                    .and_then(|c| c.api_key.clone()))
            }
            _ => anyhow::bail!("Unknown config key: {}", key),
        }
    }
    
    /// Remove configuration value by dot-notation key
    pub fn unset_value(&mut self, key: &str) -> Result<()> {
        let parts: Vec<&str> = key.split('.').collect();
        
        match parts.as_slice() {
            ["ui", "theme"] => self.ui.theme = "dark".to_string(),
            ["ui", "default_interface"] => self.ui.default_interface = "tui".to_string(),
            ["embedding", "provider"] => self.embedding.provider = "ollama".to_string(),
            ["embedding", "model"] => self.embedding.model = "nomic-embed-text".to_string(),
            ["providers", provider, "api_key"] => {
                let provider_key = provider.to_string();
                if let Some(config) = self.providers.get_mut(&provider_key) {
                    config.api_key = None;
                }
            }
            ["providers", provider, "default_model"] => {
                let provider_key = provider.to_string();
                if let Some(config) = self.providers.get_mut(&provider_key) {
                    config.default_model = match *provider {
                        "claude" => "claude-3-5-sonnet-20241022".to_string(),
                        "openai" => "gpt-4".to_string(),
                        "gemini" => "gemini-pro".to_string(),
                        "ollama" => "llama3.3".to_string(),
                        _ => "default".to_string(),
                    };
                }
            }
            _ => anyhow::bail!("Cannot unset unknown config key: {}", key),
        }
        
        Ok(())
    }

    /// Create default model config file if it doesn't exist
    pub fn ensure_model_config() -> Result<PathBuf> {
        let config_dir = Self::config_dir()?;
        let models_path = config_dir.join("models.toml");
        
        if !models_path.exists() {
            fs::create_dir_all(&config_dir)?;
            
            let default_models = r#"# Aircher Model Configurations
# This file contains model-specific settings and capabilities

[embedding_models]
"nomic-embed-text" = { provider = "ollama", size_mb = 274, performance = "good", description = "Best balance for code search" }
"mxbai-embed-large" = { provider = "ollama", size_mb = 669, performance = "excellent", description = "Highest quality for complex analysis" }
"all-MiniLM-L6-v2" = { provider = "embedded", size_mb = 90, performance = "fair", description = "Lightweight fallback" }

[chat_models]
"claude-3-5-sonnet-20241022" = { provider = "claude", context_window = 200000, cost_per_mtok_input = 3.0, cost_per_mtok_output = 15.0 }
"gpt-4o" = { provider = "openai", context_window = 128000, cost_per_mtok_input = 2.5, cost_per_mtok_output = 10.0 }
"gemini-2.0-flash-exp" = { provider = "gemini", context_window = 1000000, cost_per_mtok_input = 0.075, cost_per_mtok_output = 0.3 }
"llama3.3" = { provider = "ollama", context_window = 128000, cost_per_mtok_input = 0.0, cost_per_mtok_output = 0.0 }
"#;
            
            fs::write(&models_path, default_models)?;
            info!("Created default models config: {:?}", models_path);
        }
        
        Ok(models_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ArcherConfig::default();
        assert!(config.providers.contains_key("claude"));
        assert!(config.providers.contains_key("ollama"));
        assert_eq!(config.ui.default_interface, "tui");
        assert_eq!(config.embedding.provider, "ollama");
    }

    #[test]
    fn test_config_serialization() {
        let config = ArcherConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: ArcherConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.ui.theme, parsed.ui.theme);
    }

    #[test]
    fn test_set_get_value() {
        let mut config = ArcherConfig::default();
        
        config.set_value("ui.theme", "light").unwrap();
        assert_eq!(config.get_value("ui.theme").unwrap(), "light");
        
        config.set_value("providers.claude.default_model", "claude-3-opus").unwrap();
        assert_eq!(config.get_value("providers.claude.default_model").unwrap(), "claude-3-opus");
    }
}