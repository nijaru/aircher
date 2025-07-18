use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

use crate::cost::CostConfig;

pub mod toml_config;
pub use toml_config::ArcherConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigManager {
    pub global: GlobalConfig,
    pub providers: HashMap<String, ProviderConfig>,
    pub hosts: HashMap<String, HostConfig>,
    pub ui: UiConfig,
    pub database: DatabaseConfig,
    pub intelligence: IntelligenceConfig,
    pub cost: CostConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub default_provider: String,
    pub default_model: String,
    pub default_host: String,
    pub max_context_tokens: u32,
    pub budget_limit: Option<f64>,
    pub data_directory: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key_env: String,
    pub base_url: String,
    #[serde(default)]
    pub fallback_urls: Vec<String>,
    pub models: Vec<ModelConfig>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub context_window: u32,
    pub input_cost_per_1m: f64,
    pub output_cost_per_1m: f64,
    pub supports_streaming: bool,
    pub supports_tools: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    pub name: String,
    pub description: String,
    pub base_url: String,
    pub api_key_env: String,
    pub pricing_multiplier: f64,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub enable_mouse: bool,
    pub refresh_rate_ms: u64,
    pub show_token_count: bool,
    pub show_cost_estimate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub conversations_db: PathBuf,
    pub knowledge_db: PathBuf,
    pub file_index_db: PathBuf,
    pub sessions_db: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    pub enable_project_analysis: bool,
    pub enable_file_scoring: bool,
    pub enable_context_optimization: bool,
    pub file_scan_depth: u32,
    pub relevance_threshold: f64,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            default_provider: "claude".to_string(),
            default_model: "claude-3-5-sonnet-20241022".to_string(),
            default_host: "anthropic".to_string(),
            max_context_tokens: 100_000,
            budget_limit: None,
            data_directory: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".aircher"),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            enable_mouse: true,
            refresh_rate_ms: 100,
            show_token_count: true,
            show_cost_estimate: true,
        }
    }
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            enable_project_analysis: true,
            enable_file_scoring: true,
            enable_context_optimization: true,
            file_scan_depth: 10,
            relevance_threshold: 0.3,
        }
    }
}

impl ConfigManager {
    pub async fn load() -> Result<Self> {
        info!("Loading configuration");

        // Load from TOML file if it exists
        let config_path = Self::get_config_path();

        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

            let mut config: ConfigManager =
                toml::from_str(&content).with_context(|| "Failed to parse config file")?;

            // Resolve environment variables
            config.resolve_env_vars()?;

            debug!("Configuration loaded from file: {:?}", config_path);
            config
        } else {
            info!("No config file found, using defaults");
            Self::default()
        };

        // Ensure data directory exists
        fs::create_dir_all(&config.global.data_directory)
            .with_context(|| "Failed to create data directory")?;

        Ok(config)
    }

    fn get_config_path() -> PathBuf {
        env::var("AIRCHER_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::config_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("aircher")
                    .join("config.toml")
            })
    }

    fn resolve_env_vars(&mut self) -> Result<()> {
        // Resolve environment variables in provider configs
        for provider in self.providers.values_mut() {
            if let Ok(_api_key) = env::var(&provider.api_key_env) {
                debug!("Resolved API key for provider: {}", provider.name);
            }
        }

        // Resolve environment variables in host configs
        for host in self.hosts.values_mut() {
            if let Ok(_api_key) = env::var(&host.api_key_env) {
                debug!("Resolved API key for host: {}", host.name);
            }
        }

        Ok(())
    }

    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.get(name)
    }

    pub fn get_host(&self, name: &str) -> Option<&HostConfig> {
        self.hosts.get(name)
    }

    pub fn get_model(&self, provider: &str, model: &str) -> Option<&ModelConfig> {
        self.providers
            .get(provider)?
            .models
            .iter()
            .find(|m| m.name == model)
    }

    pub fn get_models_for_provider(&self, provider: &str) -> Option<&Vec<ModelConfig>> {
        self.providers.get(provider).map(|p| &p.models)
    }

    pub async fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path();
        
        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| "Failed to create config directory")?;
        }
        
        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
        
        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;
        
        debug!("Configuration saved to: {:?}", config_path);
        Ok(())
    }

    pub fn has_api_key(&self, provider: &str) -> bool {
        if let Some(provider_config) = self.providers.get(provider) {
            if provider_config.api_key_env.is_empty() {
                return true; // No API key needed (e.g., Ollama)
            }
            env::var(&provider_config.api_key_env).is_ok()
        } else {
            false
        }
    }

    pub fn is_provider_enabled(&self, provider: &str) -> bool {
        // For now, consider a provider enabled if it exists in config
        // Could add an explicit enabled field later if needed
        self.providers.contains_key(provider)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
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
                    ModelConfig {
                        name: "claude-3-opus-20240229".to_string(),
                        context_window: 200_000,
                        input_cost_per_1m: 15.0,
                        output_cost_per_1m: 75.0,
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
                base_url: "https://generativelanguage.googleapis.com/v1beta/models/{model}"
                    .to_string(),
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
                    ModelConfig {
                        name: "gemini-1.5-pro".to_string(),
                        context_window: 2_000_000,
                        input_cost_per_1m: 1.25,
                        output_cost_per_1m: 5.0,
                        supports_streaming: true,
                        supports_tools: true,
                    },
                    ModelConfig {
                        name: "gemini-1.5-flash".to_string(),
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
                    ModelConfig {
                        name: "gpt-4-turbo".to_string(),
                        context_window: 128_000,
                        input_cost_per_1m: 10.0,
                        output_cost_per_1m: 30.0,
                        supports_streaming: true,
                        supports_tools: true,
                    },
                    ModelConfig {
                        name: "gpt-4".to_string(),
                        context_window: 8_192,
                        input_cost_per_1m: 30.0,
                        output_cost_per_1m: 60.0,
                        supports_streaming: true,
                        supports_tools: true,
                    },
                    ModelConfig {
                        name: "gpt-3.5-turbo".to_string(),
                        context_window: 16_385,
                        input_cost_per_1m: 0.5,
                        output_cost_per_1m: 1.5,
                        supports_streaming: true,
                        supports_tools: true,
                    },
                    ModelConfig {
                        name: "o1-preview".to_string(),
                        context_window: 128_000,
                        input_cost_per_1m: 15.0,
                        output_cost_per_1m: 60.0,
                        supports_streaming: false,
                        supports_tools: false,
                    },
                    ModelConfig {
                        name: "o1-mini".to_string(),
                        context_window: 128_000,
                        input_cost_per_1m: 3.0,
                        output_cost_per_1m: 12.0,
                        supports_streaming: false,
                        supports_tools: false,
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
                api_key_env: "".to_string(), // No API key needed for local
                base_url: "".to_string(), // Empty to enable auto-discovery
                fallback_urls: vec![
                    "http://localhost:11434".to_string(),
                    "http://100.64.0.1:11434".to_string(), // Common Tailscale IP
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
                    ModelConfig {
                        name: "llama3.1".to_string(),
                        context_window: 128_000,
                        input_cost_per_1m: 0.0,
                        output_cost_per_1m: 0.0,
                        supports_streaming: true,
                        supports_tools: false,
                    },
                    ModelConfig {
                        name: "mistral".to_string(),
                        context_window: 32_000,
                        input_cost_per_1m: 0.0,
                        output_cost_per_1m: 0.0,
                        supports_streaming: true,
                        supports_tools: false,
                    },
                    ModelConfig {
                        name: "codellama".to_string(),
                        context_window: 16_000,
                        input_cost_per_1m: 0.0,
                        output_cost_per_1m: 0.0,
                        supports_streaming: true,
                        supports_tools: false,
                    },
                    ModelConfig {
                        name: "phi3".to_string(),
                        context_window: 128_000,
                        input_cost_per_1m: 0.0,
                        output_cost_per_1m: 0.0,
                        supports_streaming: true,
                        supports_tools: false,
                    },
                    ModelConfig {
                        name: "qwen2.5".to_string(),
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

        // Direct Anthropic host
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

        // Direct Google AI host
        hosts.insert(
            "google".to_string(),
            HostConfig {
                name: "Google AI".to_string(),
                description: "Direct Google AI API access".to_string(),
                base_url: "https://generativelanguage.googleapis.com/v1beta/models/{model}"
                    .to_string(),
                api_key_env: "GOOGLE_API_KEY".to_string(),
                pricing_multiplier: 1.0,
                features: vec!["official".to_string(), "reliable".to_string()],
            },
        );

        // OpenRouter host - universal model access with cost optimization
        hosts.insert(
            "openrouter".to_string(),
            HostConfig {
                name: "OpenRouter".to_string(),
                description: "Universal model access with cost optimization and fallbacks"
                    .to_string(),
                base_url: "https://openrouter.ai/api/v1".to_string(),
                api_key_env: "OPENROUTER_API_KEY".to_string(),
                pricing_multiplier: 1.0, // OpenRouter uses same pricing as providers
                features: vec![
                    "universal".to_string(),
                    "cost_optimization".to_string(),
                    "fallbacks".to_string(),
                ],
            },
        );

        // Ollama host - local model access
        hosts.insert(
            "ollama".to_string(),
            HostConfig {
                name: "Ollama".to_string(),
                description: "Local model access with zero costs and privacy".to_string(),
                base_url: "http://localhost:11434".to_string(),
                api_key_env: "".to_string(), // No API key needed
                pricing_multiplier: 0.0, // Free local models
                features: vec![
                    "local".to_string(),
                    "free".to_string(),
                    "privacy".to_string(),
                    "offline".to_string(),
                ],
            },
        );

        let data_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".aircher");

        Self {
            global: GlobalConfig::default(),
            providers,
            hosts,
            ui: UiConfig::default(),
            database: DatabaseConfig {
                conversations_db: data_dir.join("conversations.db"),
                knowledge_db: data_dir.join("knowledge.db"),
                file_index_db: data_dir.join("file_index.db"),
                sessions_db: data_dir.join("sessions.db"),
            },
            intelligence: IntelligenceConfig::default(),
            cost: CostConfig::default(),
        }
    }
}
