use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use super::{ConfigManager, GlobalConfig, ProviderConfig, ModelConfig, HostConfig, UiConfig, DatabaseConfig, IntelligenceConfig};
use crate::cost::CostConfig;

/// Configuration hierarchy manager that implements the layered config approach:
/// Hardcoded defaults -> Global config -> Local config -> Environment variables
pub struct ConfigHierarchy {
    hardcoded_defaults: ConfigManager,
    global_config_path: PathBuf,
    local_config_path: Option<PathBuf>,
}

impl ConfigHierarchy {
    pub fn new() -> Result<Self> {
        let global_config_path = Self::get_global_config_path()?;
        let local_config_path = Self::find_local_config_path();
        
        Ok(Self {
            hardcoded_defaults: Self::create_hardcoded_defaults(),
            global_config_path,
            local_config_path,
        })
    }

    /// Create immutable hardcoded defaults
    fn create_hardcoded_defaults() -> ConfigManager {
        let mut providers = HashMap::new();
        let mut hosts = HashMap::new();

        // Claude provider configuration
        providers.insert(
            "claude".to_string(),
            ProviderConfig {
                name: "Claude".to_string(),
                api_key_env: "ANTHROPIC_API_KEY".to_string(),
                base_url: "https://api.anthropic.com/v1".to_string(),
                fallback_urls: vec![],
                models: vec![
                    ModelConfig {
                        name: "claude-3-5-sonnet-20241022".to_string(),
                        context_window: 200_000,
                        input_cost_per_1m: 3.0,
                        output_cost_per_1m: 15.0,
                        supports_streaming: true,
                        supports_tools: true,
                    },
                    ModelConfig {
                        name: "claude-3-5-haiku-20241022".to_string(),
                        context_window: 200_000,
                        input_cost_per_1m: 0.25,
                        output_cost_per_1m: 1.25,
                        supports_streaming: true,
                        supports_tools: true,
                    },
                ],
                timeout_seconds: 60,
                max_retries: 3,
            },
        );

        // Gemini provider configuration
        providers.insert(
            "gemini".to_string(),
            ProviderConfig {
                name: "Gemini".to_string(),
                api_key_env: "GOOGLE_API_KEY".to_string(),
                base_url: "https://generativelanguage.googleapis.com/v1beta/models/{model}".to_string(),
                fallback_urls: vec![],
                models: vec![
                    ModelConfig {
                        name: "gemini-2.0-flash-exp".to_string(),
                        context_window: 1_000_000,
                        input_cost_per_1m: 0.075,
                        output_cost_per_1m: 0.30,
                        supports_streaming: true,
                        supports_tools: true,
                    },
                ],
                timeout_seconds: 60,
                max_retries: 3,
            },
        );

        // OpenAI provider configuration
        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                name: "OpenAI".to_string(),
                api_key_env: "OPENAI_API_KEY".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
                fallback_urls: vec![],
                models: vec![
                    ModelConfig {
                        name: "gpt-4o".to_string(),
                        context_window: 128_000,
                        input_cost_per_1m: 5.0,
                        output_cost_per_1m: 15.0,
                        supports_streaming: true,
                        supports_tools: true,
                    },
                    ModelConfig {
                        name: "gpt-4o-mini".to_string(),
                        context_window: 128_000,
                        input_cost_per_1m: 0.15,
                        output_cost_per_1m: 0.6,
                        supports_streaming: true,
                        supports_tools: true,
                    },
                ],
                timeout_seconds: 120,
                max_retries: 3,
            },
        );

        // Ollama provider configuration
        providers.insert(
            "ollama".to_string(),
            ProviderConfig {
                name: "Ollama".to_string(),
                api_key_env: "".to_string(),
                base_url: "".to_string(),
                fallback_urls: vec![
                    "http://localhost:11434".to_string(),
                    "http://100.64.0.1:11434".to_string(),
                ],
                models: vec![
                    ModelConfig {
                        name: "llama3.3".to_string(),
                        context_window: 128_000,
                        input_cost_per_1m: 0.0,
                        output_cost_per_1m: 0.0,
                        supports_streaming: true,
                        supports_tools: false,
                    },
                ],
                timeout_seconds: 120,
                max_retries: 3,
            },
        );

        // OpenRouter provider configuration
        providers.insert(
            "openrouter".to_string(),
            ProviderConfig {
                name: "OpenRouter".to_string(),
                api_key_env: "OPENROUTER_API_KEY".to_string(),
                base_url: "https://openrouter.ai/api/v1".to_string(),
                fallback_urls: vec![],
                models: vec![
                    ModelConfig {
                        name: "anthropic/claude-3.5-sonnet".to_string(),
                        context_window: 200_000,
                        input_cost_per_1m: 3.0,
                        output_cost_per_1m: 15.0,
                        supports_streaming: true,
                        supports_tools: true,
                    },
                ],
                timeout_seconds: 60,
                max_retries: 3,
            },
        );

        // Host configurations
        hosts.insert(
            "anthropic".to_string(),
            HostConfig {
                name: "Anthropic API".to_string(),
                description: "Direct Anthropic API access".to_string(),
                base_url: "https://api.anthropic.com/v1".to_string(),
                api_key_env: "ANTHROPIC_API_KEY".to_string(),
                pricing_multiplier: 1.0,
                features: vec!["official".to_string(), "reliable".to_string()],
            },
        );

        hosts.insert(
            "openrouter".to_string(),
            HostConfig {
                name: "OpenRouter".to_string(),
                description: "Universal model access with cost optimization".to_string(),
                base_url: "https://openrouter.ai/api/v1".to_string(),
                api_key_env: "OPENROUTER_API_KEY".to_string(),
                pricing_multiplier: 1.0,
                features: vec!["universal".to_string(), "cost_optimization".to_string()],
            },
        );

        hosts.insert(
            "ollama".to_string(),
            HostConfig {
                name: "Ollama".to_string(),
                description: "Local model access with zero costs".to_string(),
                base_url: "http://localhost:11434".to_string(),
                api_key_env: "".to_string(),
                pricing_multiplier: 0.0,
                features: vec!["local".to_string(), "free".to_string(), "privacy".to_string()],
            },
        );

        let data_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".aircher");

        ConfigManager {
            global: GlobalConfig {
                default_provider: "claude".to_string(),
                default_model: "claude-3-5-sonnet-20241022".to_string(),
                default_host: "anthropic".to_string(),
                max_context_tokens: 100_000,
                budget_limit: None,
                data_directory: data_dir.clone(),
            },
            providers,
            hosts,
            ui: UiConfig {
                theme: "default".to_string(),
                enable_mouse: true,
                refresh_rate_ms: 100,
                show_token_count: true,
                show_cost_estimate: true,
            },
            database: DatabaseConfig {
                conversations_db: data_dir.join("conversations.db"),
                knowledge_db: data_dir.join("knowledge.db"),
                file_index_db: data_dir.join("file_index.db"),
                sessions_db: data_dir.join("sessions.db"),
            },
            intelligence: IntelligenceConfig {
                enable_project_analysis: true,
                enable_file_scoring: true,
                enable_context_optimization: true,
                file_scan_depth: 10,
                relevance_threshold: 0.3,
            },
            cost: CostConfig::default(),
        }
    }

    /// Get the global configuration file path
    fn get_global_config_path() -> Result<PathBuf> {
        Ok(dirs::config_dir()
            .context("Could not determine config directory")?
            .join("aircher")
            .join("config.toml"))
    }

    /// Find local configuration file by walking up the directory tree
    fn find_local_config_path() -> Option<PathBuf> {
        let mut current_dir = env::current_dir().ok()?;
        
        loop {
            let config_file = current_dir.join(".aircher").join("config.toml");
            if config_file.exists() {
                debug!("Found local config at: {:?}", config_file);
                return Some(config_file);
            }
            
            if !current_dir.pop() {
                break;
            }
        }
        
        None
    }

    /// Load configuration using the hierarchy
    pub async fn load_config(&self) -> Result<ConfigManager> {
        info!("Loading configuration with hierarchy");
        
        // Start with hardcoded defaults
        let mut config = self.hardcoded_defaults.clone();
        debug!("Base configuration loaded from hardcoded defaults");

        // Layer 1: Global configuration
        if self.global_config_path.exists() {
            debug!("Loading global config from: {:?}", self.global_config_path);
            match self.load_and_merge_config_file(&self.global_config_path, &mut config).await {
                Ok(()) => info!("Global configuration loaded successfully"),
                Err(e) => warn!("Failed to load global config: {}", e),
            }
        } else {
            debug!("No global config file found at: {:?}", self.global_config_path);
        }

        // Layer 2: Local configuration
        if let Some(local_path) = &self.local_config_path {
            debug!("Loading local config from: {:?}", local_path);
            match self.load_and_merge_config_file(local_path, &mut config).await {
                Ok(()) => info!("Local configuration loaded successfully"),
                Err(e) => warn!("Failed to load local config: {}", e),
            }
        } else {
            debug!("No local config file found");
        }

        // Layer 3: Environment variables
        self.apply_environment_overrides(&mut config)?;

        // Ensure data directory exists
        fs::create_dir_all(&config.global.data_directory)
            .with_context(|| "Failed to create data directory")?;

        // Resolve API keys from environment
        config.resolve_env_vars()?;

        Ok(config)
    }

    /// Load a config file and merge it with the existing configuration
    async fn load_and_merge_config_file(&self, path: &PathBuf, config: &mut ConfigManager) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let file_config: ConfigManager = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        // Merge the loaded config into the existing config
        self.merge_configs(config, &file_config);

        Ok(())
    }

    /// Merge two configurations, with the source taking precedence
    fn merge_configs(&self, target: &mut ConfigManager, source: &ConfigManager) {
        // Merge global config (only non-default values)
        if source.global.default_provider != self.hardcoded_defaults.global.default_provider {
            target.global.default_provider = source.global.default_provider.clone();
        }
        if source.global.default_model != self.hardcoded_defaults.global.default_model {
            target.global.default_model = source.global.default_model.clone();
        }
        if source.global.default_host != self.hardcoded_defaults.global.default_host {
            target.global.default_host = source.global.default_host.clone();
        }
        if source.global.max_context_tokens != self.hardcoded_defaults.global.max_context_tokens {
            target.global.max_context_tokens = source.global.max_context_tokens;
        }
        if source.global.budget_limit != self.hardcoded_defaults.global.budget_limit {
            target.global.budget_limit = source.global.budget_limit;
        }
        if source.global.data_directory != self.hardcoded_defaults.global.data_directory {
            target.global.data_directory = source.global.data_directory.clone();
        }

        // Merge providers (source providers override target providers)
        for (name, provider) in &source.providers {
            target.providers.insert(name.clone(), provider.clone());
        }

        // Merge hosts (source hosts override target hosts)
        for (name, host) in &source.hosts {
            target.hosts.insert(name.clone(), host.clone());
        }

        // Merge UI config (only non-default values)
        if source.ui.theme != self.hardcoded_defaults.ui.theme {
            target.ui.theme = source.ui.theme.clone();
        }
        if source.ui.enable_mouse != self.hardcoded_defaults.ui.enable_mouse {
            target.ui.enable_mouse = source.ui.enable_mouse;
        }
        if source.ui.refresh_rate_ms != self.hardcoded_defaults.ui.refresh_rate_ms {
            target.ui.refresh_rate_ms = source.ui.refresh_rate_ms;
        }
        if source.ui.show_token_count != self.hardcoded_defaults.ui.show_token_count {
            target.ui.show_token_count = source.ui.show_token_count;
        }
        if source.ui.show_cost_estimate != self.hardcoded_defaults.ui.show_cost_estimate {
            target.ui.show_cost_estimate = source.ui.show_cost_estimate;
        }

        // Merge database config
        if source.database.conversations_db != self.hardcoded_defaults.database.conversations_db {
            target.database.conversations_db = source.database.conversations_db.clone();
        }
        if source.database.knowledge_db != self.hardcoded_defaults.database.knowledge_db {
            target.database.knowledge_db = source.database.knowledge_db.clone();
        }
        if source.database.file_index_db != self.hardcoded_defaults.database.file_index_db {
            target.database.file_index_db = source.database.file_index_db.clone();
        }
        if source.database.sessions_db != self.hardcoded_defaults.database.sessions_db {
            target.database.sessions_db = source.database.sessions_db.clone();
        }

        // Merge intelligence config
        if source.intelligence.enable_project_analysis != self.hardcoded_defaults.intelligence.enable_project_analysis {
            target.intelligence.enable_project_analysis = source.intelligence.enable_project_analysis;
        }
        if source.intelligence.enable_file_scoring != self.hardcoded_defaults.intelligence.enable_file_scoring {
            target.intelligence.enable_file_scoring = source.intelligence.enable_file_scoring;
        }
        if source.intelligence.enable_context_optimization != self.hardcoded_defaults.intelligence.enable_context_optimization {
            target.intelligence.enable_context_optimization = source.intelligence.enable_context_optimization;
        }
        if source.intelligence.file_scan_depth != self.hardcoded_defaults.intelligence.file_scan_depth {
            target.intelligence.file_scan_depth = source.intelligence.file_scan_depth;
        }
        if source.intelligence.relevance_threshold != self.hardcoded_defaults.intelligence.relevance_threshold {
            target.intelligence.relevance_threshold = source.intelligence.relevance_threshold;
        }

        // Merge cost config (using the new cost config entirely if present)
        // TODO: Implement proper comparison for CostConfig
        target.cost = source.cost.clone();
    }

    /// Apply environment variable overrides
    fn apply_environment_overrides(&self, config: &mut ConfigManager) -> Result<()> {
        debug!("Applying environment variable overrides");

        // Global overrides
        if let Ok(provider) = env::var("AIRCHER_DEFAULT_PROVIDER") {
            config.global.default_provider = provider;
            debug!("Override default provider from env");
        }
        if let Ok(model) = env::var("AIRCHER_DEFAULT_MODEL") {
            config.global.default_model = model;
            debug!("Override default model from env");
        }
        if let Ok(host) = env::var("AIRCHER_DEFAULT_HOST") {
            config.global.default_host = host;
            debug!("Override default host from env");
        }
        if let Ok(tokens) = env::var("AIRCHER_MAX_CONTEXT_TOKENS") {
            if let Ok(tokens) = tokens.parse::<u32>() {
                config.global.max_context_tokens = tokens;
                debug!("Override max context tokens from env");
            }
        }
        if let Ok(budget) = env::var("AIRCHER_BUDGET_LIMIT") {
            if let Ok(budget) = budget.parse::<f64>() {
                config.global.budget_limit = Some(budget);
                debug!("Override budget limit from env");
            }
        }
        if let Ok(data_dir) = env::var("AIRCHER_DATA_DIR") {
            config.global.data_directory = PathBuf::from(data_dir);
            debug!("Override data directory from env");
        }

        // UI overrides
        if let Ok(theme) = env::var("AIRCHER_UI_THEME") {
            config.ui.theme = theme;
            debug!("Override UI theme from env");
        }
        if let Ok(mouse) = env::var("AIRCHER_UI_ENABLE_MOUSE") {
            config.ui.enable_mouse = mouse.parse().unwrap_or(config.ui.enable_mouse);
            debug!("Override UI mouse from env");
        }
        if let Ok(refresh) = env::var("AIRCHER_UI_REFRESH_RATE_MS") {
            if let Ok(refresh) = refresh.parse::<u64>() {
                config.ui.refresh_rate_ms = refresh;
                debug!("Override UI refresh rate from env");
            }
        }

        Ok(())
    }

    /// Save configuration to appropriate file (global or local)
    pub async fn save_config(&self, config: &ConfigManager, scope: ConfigScope) -> Result<()> {
        let path = match scope {
            ConfigScope::Global => &self.global_config_path,
            ConfigScope::Local => {
                if let Some(local_path) = &self.local_config_path {
                    local_path
                } else {
                    // Create local config in current directory
                    let local_path = env::current_dir()?
                        .join(".aircher")
                        .join("config.toml");
                    
                    if let Some(parent) = local_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    return self.save_config_to_file(config, &local_path).await;
                }
            }
        };

        self.save_config_to_file(config, path).await
    }

    async fn save_config_to_file(&self, config: &ConfigManager, path: &PathBuf) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content = toml::to_string_pretty(config)
            .with_context(|| "Failed to serialize config")?;

        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        debug!("Configuration saved to: {:?}", path);
        Ok(())
    }

    /// Get the current configuration paths for debugging
    pub fn get_config_paths(&self) -> ConfigPaths {
        ConfigPaths {
            global: self.global_config_path.clone(),
            local: self.local_config_path.clone(),
        }
    }
}

/// Configuration scope for saving
#[derive(Debug, Clone)]
pub enum ConfigScope {
    Global,
    Local,
}

/// Configuration paths information
#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub global: PathBuf,
    pub local: Option<PathBuf>,
}

