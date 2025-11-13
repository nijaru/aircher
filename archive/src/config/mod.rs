use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use tracing::{debug, info};

use crate::context::{CompactionConfig as ContextCompactionConfig, SummaryDepth};
use crate::cost::CostConfig;
use crate::utils::aircher_dirs::AircherDirs;

pub mod toml_config;
pub use toml_config::ArcherConfig;

pub mod hierarchy;
pub use hierarchy::{ConfigHierarchy, ConfigScope, ConfigPaths};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigManager {
    #[serde(default)]
    pub global: GlobalConfig,
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(default)]
    pub hosts: HashMap<String, HostConfig>,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub intelligence: IntelligenceConfig,
    #[serde(default)]
    pub cost: CostConfig,
    #[serde(default)]
    pub multi_provider: MultiProviderConfig,
    #[serde(default)]
    pub compaction: CompactionConfig,
    #[serde(default)]
    pub model_routing: ModelRoutingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default = "default_provider")]
    pub default_provider: String,
    #[serde(default = "default_model")]
    pub default_model: String,
    #[serde(default = "default_host")]
    pub default_host: String,
    #[serde(default = "default_max_context_tokens")]
    pub max_context_tokens: u32,
    #[serde(default)]
    pub budget_limit: Option<f64>,
    #[serde(default = "default_data_directory")]
    pub data_directory: PathBuf,
}

// Default functions for serde
fn default_provider() -> String { "ollama".to_string() }
fn default_model() -> String { "gpt-oss".to_string() }
fn default_host() -> String { "ollama".to_string() }
fn default_max_context_tokens() -> u32 { 100_000 }
fn default_data_directory() -> PathBuf {
    AircherDirs::data_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub api_key_env: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub fallback_urls: Vec<String>,
    #[serde(default)]
    pub models: Vec<ModelConfig>,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_timeout_seconds() -> u64 { 60 }
fn default_max_retries() -> u32 { 3 }

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
    #[serde(default = "default_submit_on_enter")]
    pub submit_on_enter: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub conversations_db: PathBuf,
    pub knowledge_db: PathBuf,
    pub file_index_db: PathBuf,
    pub sessions_db: PathBuf,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let data_dir = AircherDirs::data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        Self {
            conversations_db: data_dir.join("conversations.db"),
            knowledge_db: data_dir.join("knowledge.db"),
            file_index_db: data_dir.join("file_index.db"),
            sessions_db: data_dir.join("sessions.db"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    pub enable_project_analysis: bool,
    pub enable_file_scoring: bool,
    pub enable_context_optimization: bool,
    pub file_scan_depth: u32,
    pub relevance_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    /// Enable automatic compaction
    pub auto_enabled: bool,
    /// Warning threshold (0.0 - 1.0)
    pub warning_threshold: f32,
    /// Critical threshold (0.0 - 1.0)
    pub critical_threshold: f32,
    /// Minimum messages before allowing compaction
    pub min_messages: u32,
    /// Number of recent messages to keep
    pub keep_recent_messages: usize,
    /// Keep system messages
    pub keep_system_messages: bool,
    /// Keep tool result messages
    pub keep_tool_results: bool,
    /// Summarization depth
    pub summary_depth: SummaryDepth,
    /// Preserve code blocks in summaries
    pub preserve_code_blocks: bool,
    /// Preserve file paths in summaries
    pub preserve_file_paths: bool,
    /// Show warnings to user
    pub show_warnings: bool,
    /// Require user confirmation for non-critical compactions
    pub require_confirmation: bool,
}

impl CompactionConfig {
    /// Convert to the context module's CompactionConfig
    pub fn to_context_config(&self) -> ContextCompactionConfig {
        ContextCompactionConfig {
            auto_enabled: self.auto_enabled,
            warning_threshold: self.warning_threshold,
            critical_threshold: self.critical_threshold,
            min_messages: self.min_messages,
            keep_recent_messages: self.keep_recent_messages,
            keep_system_messages: self.keep_system_messages,
            keep_tool_results: self.keep_tool_results,
            summary_depth: self.summary_depth,
            preserve_code_blocks: self.preserve_code_blocks,
            preserve_file_paths: self.preserve_file_paths,
            show_warnings: self.show_warnings,
            require_confirmation: self.require_confirmation,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiProviderConfig {
    /// Map of model names to provider preferences (with priority order)
    #[serde(default)]
    pub model_preferences: HashMap<String, Vec<ModelPreference>>,
    /// Map of task types to preferred model configurations
    #[serde(default)]
    pub task_preferences: HashMap<String, TaskPreference>,
    /// Global fallback strategy configuration
    #[serde(default)]
    pub fallback_strategy: FallbackStrategy,
    /// Provider-specific fallback rules
    #[serde(default)]
    pub provider_fallbacks: HashMap<String, ProviderFallback>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreference {
    /// Provider name (must match provider config key)
    pub provider: String,
    /// Priority (1 = highest priority)
    pub priority: u8,
    /// Cost multiplier for this provider (1.0 = baseline)
    pub cost_multiplier: f64,
    /// Optional conditions for using this provider
    #[serde(default)]
    pub conditions: Option<ProviderConditions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConditions {
    /// Maximum usage percentage before fallback (for subscription tiers)
    pub max_usage_percent: Option<f64>,
    /// Required features (e.g., ["streaming", "tools"])
    #[serde(default)]
    pub required_features: Vec<String>,
    /// Time-based restrictions (e.g., business hours only)
    pub time_restrictions: Option<TimeRestrictions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestrictions {
    /// Start hour (0-23)
    pub start_hour: u8,
    /// End hour (0-23)
    pub end_hour: u8,
    /// Days of week (0 = Sunday, 6 = Saturday)
    pub days: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPreference {
    /// Preferred model for this task type
    pub model: String,
    /// Override temperature for this task
    pub temperature: Option<f64>,
    /// Override max tokens for this task
    pub max_tokens: Option<u32>,
    /// Custom system prompt for this task
    pub system_prompt_override: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FallbackStrategy {
    /// Strategy type: "fastest", "cheapest", "cost_aware", "quality"
    #[serde(default = "default_fallback_strategy")]
    pub strategy: String,
    /// Maximum cost increase multiplier for fallbacks (e.g., 2.0 = max 2x cost)
    #[serde(default = "default_max_cost_increase")]
    pub max_cost_increase: f64,
    /// Allow fallback to free tiers when available
    #[serde(default = "default_enable_free_fallback")]
    pub enable_free_fallback: bool,
    /// Maximum number of fallback attempts
    #[serde(default = "default_max_fallback_attempts")]
    pub max_fallback_attempts: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderFallback {
    /// List of providers to try in order
    pub fallback_to: Vec<String>,
    /// Enable fallback when usage limits reached
    #[serde(default)]
    pub max_usage_fallback: bool,
    /// Enable fallback on rate limits
    #[serde(default)]
    pub rate_limit_fallback: bool,
    /// Enable fallback on connection failures
    #[serde(default)]
    pub connection_fallback: bool,
}

/// Model routing configuration for task-based smart routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRoutingConfig {
    /// Provider to use: "anthropic" | "openai" | "google" | "ollama" | "openrouter"
    /// If None, defaults to "anthropic"
    pub provider: Option<String>,

    /// Optional: Single model to use for ALL tasks (bypasses routing table)
    /// Example: "claude-sonnet-4-5"
    pub single_model: Option<String>,

    /// Optional: Use OpenRouter exacto premium endpoints
    pub use_exacto: Option<bool>,
}

impl Default for ModelRoutingConfig {
    fn default() -> Self {
        Self {
            provider: None,  // Will default to "anthropic" in Agent initialization
            single_model: None,  // Use smart routing by default
            use_exacto: None,
        }
    }
}

// Default functions for serde
fn default_fallback_strategy() -> String {
    "cost_aware".to_string()
}

fn default_max_cost_increase() -> f64 {
    2.0
}

fn default_enable_free_fallback() -> bool {
    true
}

fn default_max_fallback_attempts() -> u8 {
    3
}

impl Default for MultiProviderConfig {
    fn default() -> Self {
        let mut model_preferences = HashMap::new();
        let mut task_preferences = HashMap::new();
        let mut provider_fallbacks = HashMap::new();

        // Example model preferences - Claude Sonnet available from multiple providers
        model_preferences.insert(
            "claude-3-5-sonnet-20241022".to_string(),
            vec![
                ModelPreference {
                    provider: "anthropic".to_string(),
                    priority: 1,
                    cost_multiplier: 1.0,
                    conditions: None,
                },
                ModelPreference {
                    provider: "openrouter".to_string(),
                    priority: 2,
                    cost_multiplier: 1.2,
                    conditions: None,
                },
            ],
        );

        // Example task preferences
        task_preferences.insert(
            "agent_coding".to_string(),
            TaskPreference {
                model: "claude-3-5-sonnet-20241022".to_string(),
                temperature: Some(0.1),
                max_tokens: Some(4000),
                system_prompt_override: None,
            },
        );
        task_preferences.insert(
            "general_chat".to_string(),
            TaskPreference {
                model: "claude-3-5-sonnet-20241022".to_string(),
                temperature: Some(0.7),
                max_tokens: Some(2000),
                system_prompt_override: None,
            },
        );

        // Example provider fallbacks
        provider_fallbacks.insert(
            "anthropic".to_string(),
            ProviderFallback {
                fallback_to: vec!["openrouter".to_string()],
                max_usage_fallback: false,
                rate_limit_fallback: true,
                connection_fallback: true,
            },
        );
        provider_fallbacks.insert(
            "ollama".to_string(),
            ProviderFallback {
                fallback_to: vec!["openrouter".to_string(), "anthropic".to_string()],
                max_usage_fallback: false,
                rate_limit_fallback: false,
                connection_fallback: true,
            },
        );

        Self {
            model_preferences,
            task_preferences,
            fallback_strategy: FallbackStrategy {
                strategy: default_fallback_strategy(),
                max_cost_increase: default_max_cost_increase(),
                enable_free_fallback: default_enable_free_fallback(),
                max_fallback_attempts: default_max_fallback_attempts(),
            },
            provider_fallbacks,
        }
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            default_provider: "claude".to_string(),
            default_model: "claude-3-5-sonnet-20241022".to_string(),
            default_host: "anthropic".to_string(),
            max_context_tokens: 100_000,
            budget_limit: None,
            data_directory: AircherDirs::data_dir()
                .unwrap_or_else(|_| PathBuf::from(".")),
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
            submit_on_enter: true,
        }
    }
}

fn default_submit_on_enter() -> bool { true }

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

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            auto_enabled: true,
            warning_threshold: 0.75,
            critical_threshold: 0.90,
            min_messages: 10,
            keep_recent_messages: 5,
            keep_system_messages: true,
            keep_tool_results: true,
            summary_depth: SummaryDepth::Standard,
            preserve_code_blocks: true,
            preserve_file_paths: true,
            show_warnings: true,
            require_confirmation: true,
        }
    }
}

impl ConfigManager {
    pub async fn load() -> Result<Self> {
        info!("Loading configuration with hierarchy");

        // Use the new hierarchical configuration system
        let hierarchy = ConfigHierarchy::new()?;
        let config = hierarchy.load_config().await?;

        info!("Configuration loaded successfully");
        Ok(config)
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
        // Default to global scope for backward compatibility
        self.save_with_scope(ConfigScope::Global).await
    }

    pub async fn save_with_scope(&self, scope: ConfigScope) -> Result<()> {
        let hierarchy = ConfigHierarchy::new()?;
        hierarchy.save_config(self, scope).await
    }

    pub fn get_config_paths() -> Result<ConfigPaths> {
        let hierarchy = ConfigHierarchy::new()?;
        Ok(hierarchy.get_config_paths())
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

        let data_dir = AircherDirs::data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));

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
            multi_provider: MultiProviderConfig::default(),
            compaction: CompactionConfig::default(),
            model_routing: ModelRoutingConfig::default(),
        }
    }
}
