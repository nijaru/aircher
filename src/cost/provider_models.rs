use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use super::pricing_engine::QualityTier;
use super::pricing_api::PricingAPI;

/// Model capabilities for different features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub supports_tools: bool,
    pub supports_streaming: bool,
    pub max_context: u32,
    pub quality_tier: QualityTier,
    pub cost_tier: CostTier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CostTier {
    Free,      // Local or free models
    Low,       // < $1 per 1M tokens
    Medium,    // $1-10 per 1M tokens  
    High,      // $10+ per 1M tokens
}

/// Simple 3-tier model configuration per provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModels {
    /// Planning model for complex reasoning - highest quality (defaults to main)
    pub planning: String,
    /// Main model for general use - good balance of quality and cost
    pub main: String,
    /// Light model for simple tasks - prioritizes speed and low cost
    pub light: String,
    /// Embedding model for semantic search (optional)
    pub embedding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfiguration {
    /// Model configuration for each provider
    pub providers: HashMap<String, ProviderModels>,
    /// Task routing - which tier to use for which task types
    pub task_routing: TaskRouting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRouting {
    /// Tasks that use the light model (fast/cheap)
    pub light_tasks: Vec<String>,
    /// Tasks that use the planning model (complex reasoning)
    pub planning_tasks: Vec<String>,
    // Everything else uses main model
}

impl Default for ProviderModels {
    fn default() -> Self {
        Self {
            planning: "auto".to_string(),
            main: "auto".to_string(),
            light: "auto".to_string(),
            embedding: None,
        }
    }
}

impl Default for TaskRouting {
    fn default() -> Self {
        Self {
            light_tasks: vec![
                "commit_messages".to_string(),
                "summaries".to_string(),
                "quick_questions".to_string(),
                "simple_edits".to_string(),
            ],
            planning_tasks: vec![
                "planning".to_string(),
                "architecture".to_string(),
                "design_review".to_string(),
                "strategy".to_string(),
            ],
        }
    }
}

impl Default for ModelConfiguration {
    fn default() -> Self {
        let mut providers = HashMap::new();
        
        // Claude - excellent for reasoning and code
        providers.insert("claude".to_string(), ProviderModels {
            planning: "claude-3-5-sonnet-20241022".to_string(), // Could be opus for power users
            main: "claude-3-5-sonnet-20241022".to_string(),
            light: "claude-3-5-haiku-20241022".to_string(),
            embedding: None, // Claude doesn't provide embeddings
        });
        
        // OpenAI - solid general purpose
        providers.insert("openai".to_string(), ProviderModels {
            planning: "gpt-4o".to_string(), // Could be o1-preview for power users
            main: "gpt-4o".to_string(),
            light: "gpt-4o-mini".to_string(),
            embedding: Some("text-embedding-3-small".to_string()),
        });
        
        // Gemini - cost-effective with good capabilities
        providers.insert("gemini".to_string(), ProviderModels {
            planning: "gemini-2.0-flash-exp".to_string(), // Could be 2.5-pro when available
            main: "gemini-2.0-flash-exp".to_string(),
            light: "gemini-1.5-flash".to_string(),
            embedding: Some("text-embedding-004".to_string()),
        });
        
        // Ollama - free local models
        providers.insert("ollama".to_string(), ProviderModels {
            planning: "llama3.3".to_string(), // Same as main for free models
            main: "llama3.3".to_string(),
            light: "llama3.1".to_string(),
            embedding: Some("nomic-embed-text".to_string()), // Code-optimized embeddings
        });
        
        // OpenRouter - flexible routing
        providers.insert("openrouter".to_string(), ProviderModels {
            planning: "claude-3-5-sonnet".to_string(), // Could route to opus
            main: "claude-3-5-sonnet".to_string(),
            light: "gpt-4o-mini".to_string(),
            embedding: None, // Depends on routed provider
        });

        Self {
            providers,
            task_routing: TaskRouting::default(),
        }
    }
}

impl ModelConfiguration {
    /// Get capabilities for a specific model
    pub fn get_model_capabilities(&self, provider: &str, model: &str) -> ModelCapabilities {
        detect_model_capabilities(provider, model)
    }

    /// Select the appropriate model for a provider and task
    pub fn select_model(&self, provider: &str, task_hint: Option<&str>) -> (String, ModelTier, String) {
        let default_models = ProviderModels::default();
        let provider_models = self.providers.get(provider)
            .unwrap_or(&default_models);

        let (model, tier, reason) = if let Some(task) = task_hint {
            if self.task_routing.planning_tasks.contains(&task.to_string()) {
                (provider_models.planning.clone(), ModelTier::Planning, format!("Planning task: {}", task))
            } else if self.task_routing.light_tasks.contains(&task.to_string()) {
                (provider_models.light.clone(), ModelTier::Light, format!("Light task: {}", task))
            } else {
                (provider_models.main.clone(), ModelTier::Main, format!("General task: {}", task))
            }
        } else {
            (provider_models.main.clone(), ModelTier::Main, "No task specified".to_string())
        };

        (model, tier, reason)
    }

    /// Set a model for a specific provider and tier
    pub fn set_model(&mut self, provider: &str, tier: ModelTier, model: &str) {
        let provider_models = self.providers.entry(provider.to_string())
            .or_insert_with(ProviderModels::default);

        match tier {
            ModelTier::Planning => provider_models.planning = model.to_string(),
            ModelTier::Main => provider_models.main = model.to_string(),
            ModelTier::Light => provider_models.light = model.to_string(),
        }

        info!("Set {} {} model to {}", provider, tier.as_str(), model);
    }

    /// Get all models for a provider
    pub fn get_provider_models(&self, provider: &str) -> Option<&ProviderModels> {
        self.providers.get(provider)
    }

    /// Add a task to a specific tier
    pub fn set_task_tier(&mut self, task: &str, tier: ModelTier) {
        // Remove from other tiers first
        self.task_routing.light_tasks.retain(|t| t != task);
        self.task_routing.planning_tasks.retain(|t| t != task);

        // Add to the specified tier
        match tier {
            ModelTier::Light => self.task_routing.light_tasks.push(task.to_string()),
            ModelTier::Planning => self.task_routing.planning_tasks.push(task.to_string()),
            ModelTier::Main => {}, // Main is the default, no list needed
        }
    }

    /// Generate user-friendly configuration summary
    pub fn get_summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str("🤖 Model Configuration:\n\n");
        
        for (provider, models) in &self.providers {
            summary.push_str(&format!("📍 {}:\n", provider));
            let planning_caps = self.get_model_capabilities(provider, &models.planning);
            let main_caps = self.get_model_capabilities(provider, &models.main);
            let light_caps = self.get_model_capabilities(provider, &models.light);
            
            summary.push_str(&format!("  Planning (complex): {}\n", 
                format_model_with_pricing(provider, &models.planning, &planning_caps)));
            summary.push_str(&format!("  Main (general):     {}\n", 
                format_model_with_pricing(provider, &models.main, &main_caps)));
            summary.push_str(&format!("  Light (fast):       {}\n", 
                format_model_with_pricing(provider, &models.light, &light_caps)));
            
            if let Some(ref embedding) = models.embedding {
                summary.push_str(&format!("  Embedding:          {}\n", embedding));
            }
            summary.push('\n');
        }

        summary.push_str("🏷️  Task Categories:\n");
        summary.push_str(&format!("  Light tasks:    {}\n", self.task_routing.light_tasks.join(", ")));
        summary.push_str(&format!("  Planning tasks: {}\n", self.task_routing.planning_tasks.join(", ")));
        summary.push_str("  Everything else uses main model\n\n");

        summary.push_str("💡 To configure:\n");
        summary.push_str("  aircher config model <provider> <model>        # sets main\n");
        summary.push_str("  aircher config model <provider> <model> --light # sets light\n");

        summary
    }

    /// Get optimization suggestions
    pub fn get_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Check for cost optimization opportunities
        for (provider, models) in &self.providers {
            if provider == "ollama" {
                continue; // Already free
            }

            if models.light == models.main {
                suggestions.push(format!(
                    "Consider setting a cheaper 'light' model for {} (currently same as main)",
                    provider
                ));
            }

            if models.planning == models.main && (provider == "claude" || provider == "openai") {
                let suggestion = match provider.as_str() {
                    "claude" => "Consider 'claude-3-opus' for planning tasks (ultimate reasoning)",
                    "openai" => "Consider 'o1-preview' for planning tasks (advanced reasoning)",
                    _ => "",
                };
                if !suggestion.is_empty() {
                    suggestions.push(suggestion.to_string());
                }
            }
        }

        // Suggest trying Ollama
        if !self.providers.contains_key("ollama") {
            suggestions.push("Try 'ollama' provider for free local models".to_string());
        }

        // Suggest specific optimizations
        if let Some(openai) = self.providers.get("openai") {
            if openai.light != "gpt-4o-mini" {
                suggestions.push("Consider 'gpt-4o-mini' for OpenAI light tasks (often cheaper than gpt-3.5-turbo!)".to_string());
            }
        }

        if let Some(claude) = self.providers.get("claude") {
            if claude.light != "claude-3-5-haiku-20241022" {
                suggestions.push("Consider 'claude-3-5-haiku-20241022' for light tasks (excellent for summaries)".to_string());
            }
        }

        suggestions
    }
}

/// Detect capabilities for a specific model
pub fn detect_model_capabilities(provider: &str, model: &str) -> ModelCapabilities {
    match (provider, model) {
        // Claude models - excellent tool support
        ("claude", m) if m.contains("claude-3") => ModelCapabilities {
            supports_tools: true,
            supports_streaming: true,
            max_context: 200_000,
            quality_tier: if m.contains("opus") { QualityTier::Flagship }
                         else if m.contains("sonnet") { QualityTier::Premium }
                         else { QualityTier::Standard },
            cost_tier: if m.contains("opus") { CostTier::High }
                      else if m.contains("sonnet") { CostTier::Medium }
                      else { CostTier::Low },
        },
        
        // OpenAI models
        ("openai", "gpt-4o") => ModelCapabilities {
            supports_tools: true,
            supports_streaming: true,
            max_context: 128_000,
            quality_tier: QualityTier::Premium,
            cost_tier: CostTier::Medium,
        },
        ("openai", "gpt-4o-mini") => ModelCapabilities {
            supports_tools: true,
            supports_streaming: true,
            max_context: 128_000,
            quality_tier: QualityTier::Standard,
            cost_tier: CostTier::Low,
        },
        ("openai", m) if m.contains("o1") => ModelCapabilities {
            supports_tools: false, // o1 models don't support tools yet
            supports_streaming: false,
            max_context: 128_000,
            quality_tier: QualityTier::Flagship,
            cost_tier: CostTier::High,
        },
        ("openai", m) if m.contains("gpt-4") => ModelCapabilities {
            supports_tools: true,
            supports_streaming: true,
            max_context: 128_000,
            quality_tier: QualityTier::Premium,
            cost_tier: CostTier::Medium,
        },
        ("openai", m) if m.contains("gpt-3.5") => ModelCapabilities {
            supports_tools: true,
            supports_streaming: true,
            max_context: 16_385,
            quality_tier: QualityTier::Standard,
            cost_tier: CostTier::Low,
        },
        
        // Gemini models
        ("gemini", m) if m.contains("2.0") || m.contains("1.5") => ModelCapabilities {
            supports_tools: true,
            supports_streaming: true,
            max_context: 1_000_000,
            quality_tier: if m.contains("pro") { QualityTier::Premium } else { QualityTier::Standard },
            cost_tier: CostTier::Low,
        },
        
        // Ollama models - mostly read-only
        ("ollama", m) if m.contains("llama") => ModelCapabilities {
            supports_tools: false, // Most Ollama models don't have reliable tool support
            supports_streaming: true,
            max_context: 128_000,
            quality_tier: if m.contains("3.3") { QualityTier::Standard } else { QualityTier::Basic },
            cost_tier: CostTier::Free,
        },
        ("ollama", _) => ModelCapabilities {
            supports_tools: false,
            supports_streaming: true,
            max_context: 32_000,
            quality_tier: QualityTier::Basic,
            cost_tier: CostTier::Free,
        },
        
        // OpenRouter - depends on the routed model
        ("openrouter", m) if m.contains("claude") => ModelCapabilities {
            supports_tools: true,
            supports_streaming: true,
            max_context: 200_000,
            quality_tier: QualityTier::Premium,
            cost_tier: CostTier::Medium,
        },
        ("openrouter", m) if m.contains("gpt-4") => ModelCapabilities {
            supports_tools: true,
            supports_streaming: true,
            max_context: 128_000,
            quality_tier: QualityTier::Premium,
            cost_tier: CostTier::Medium,
        },
        
        // Default fallback
        _ => ModelCapabilities {
            supports_tools: false,
            supports_streaming: true,
            max_context: 8_000,
            quality_tier: QualityTier::Basic,
            cost_tier: CostTier::Medium,
        },
    }
}

/// Get estimated pricing for a model with live API fallback
pub async fn get_estimated_pricing_live(provider: &str, model: &str, pricing_api: &mut PricingAPI) -> Option<String> {
    // Try live API first
    if let Some(live_pricing) = pricing_api.get_model_pricing(provider, model).await {
        return Some(live_pricing);
    }
    
    // Fallback to hardcoded prices
    get_estimated_pricing_fallback(provider, model)
}

/// Get estimated pricing for a model (fallback/offline)
pub fn get_estimated_pricing_fallback(provider: &str, model: &str) -> Option<String> {
    match (provider, model) {
        // OpenAI pricing ($/1M tokens, input/output)
        ("openai", "gpt-4o") => Some("$5/$15".to_string()),
        ("openai", "gpt-4o-mini") => Some("$0.15/$0.60".to_string()),
        ("openai", m) if m.contains("o1-preview") => Some("$15/$60".to_string()),
        ("openai", m) if m.contains("gpt-4") => Some("$10/$30".to_string()),
        ("openai", m) if m.contains("gpt-3.5") => Some("$0.50/$1.50".to_string()),
        
        // Claude pricing
        ("claude", m) if m.contains("opus") => Some("$15/$75".to_string()),
        ("claude", m) if m.contains("sonnet") => Some("$3/$15".to_string()),
        ("claude", m) if m.contains("haiku") => Some("$0.25/$1.25".to_string()),
        
        // Gemini pricing (very competitive)
        ("gemini", m) if m.contains("2.0") || m.contains("1.5-pro") => Some("$1.25/$5.00".to_string()),
        ("gemini", m) if m.contains("flash") => Some("$0.075/$0.30".to_string()),
        
        // Free models
        ("ollama", _) => Some("Free".to_string()),
        
        // OpenRouter (varies, show typical)
        ("openrouter", _) => Some("Varies".to_string()),
        
        _ => None,
    }
}

/// Sync version for backwards compatibility
pub fn get_estimated_pricing(provider: &str, model: &str) -> Option<String> {
    get_estimated_pricing_fallback(provider, model)
}

/// Format capability indicators for display
pub fn format_capability_indicators(caps: &ModelCapabilities) -> String {
    let mut parts = Vec::new();
    
    // Tool support indicator  
    let tool_indicator = if caps.supports_tools { "🛠️" } else { "📖" };
    parts.push(tool_indicator.to_string());
    
    // Context window
    let context_display = if caps.max_context >= 1_000_000 {
        format!("{}M", caps.max_context / 1_000_000)
    } else if caps.max_context >= 1_000 {
        format!("{}k", caps.max_context / 1_000)
    } else {
        caps.max_context.to_string()
    };
    parts.push(context_display);
    
    format!("({})", parts.join(" "))
}

/// Format model display with pricing (async version with live API)
pub async fn format_model_with_pricing_live(provider: &str, model: &str, caps: &ModelCapabilities, pricing_api: &mut PricingAPI) -> String {
    let pricing = get_estimated_pricing_live(provider, model, pricing_api).await
        .unwrap_or_else(|| "?".to_string());
    
    let indicators = format_capability_indicators(caps);
    
    if pricing == "Free" {
        format!("{} {} 🆓", model, indicators)
    } else {
        format!("{} {} {}", model, indicators, pricing)
    }
}

/// Format model display with pricing (sync version, fallback only)
pub fn format_model_with_pricing(provider: &str, model: &str, caps: &ModelCapabilities) -> String {
    let pricing = get_estimated_pricing(provider, model)
        .unwrap_or_else(|| "?".to_string());
    
    let indicators = format_capability_indicators(caps);
    
    if pricing == "Free" {
        format!("{} {} 🆓", model, indicators)
    } else {
        format!("{} {} {}", model, indicators, pricing)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTier {
    Planning, // Complex reasoning, highest quality  
    Main,     // General purpose, balanced
    Light,    // Fast and cheap
}

impl ModelTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelTier::Planning => "planning",
            ModelTier::Main => "main",
            ModelTier::Light => "light", 
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "planning" | "plan" | "complex" | "reasoning" => Some(ModelTier::Planning),
            "main" | "general" | "default" => Some(ModelTier::Main),
            "light" | "fast" | "cheap" | "economy" => Some(ModelTier::Light),
            _ => None,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ModelTier::Planning => "Complex reasoning model - highest quality for architecture and strategy",
            ModelTier::Main => "General purpose model - good balance of quality and cost",
            ModelTier::Light => "Fast model for simple tasks - prioritizes speed and low cost", 
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_selection() {
        let config = ModelConfiguration::default();
        
        // Test planning task
        let (model, tier, reason) = config.select_model("claude", Some("architecture"));
        assert_eq!(tier, ModelTier::Planning);
        assert!(reason.contains("Planning task"));
        
        // Test light task
        let (model, tier, reason) = config.select_model("openai", Some("commit_messages"));
        assert_eq!(tier, ModelTier::Light);
        assert!(reason.contains("Light task"));
        
        // Test general task
        let (model, tier, reason) = config.select_model("gemini", Some("documentation"));
        assert_eq!(tier, ModelTier::Main);
        assert!(reason.contains("General task"));
    }

    #[test]
    fn test_model_tier_parsing() {
        assert_eq!(ModelTier::from_str("main"), Some(ModelTier::Main));
        assert_eq!(ModelTier::from_str("light"), Some(ModelTier::Light));
        assert_eq!(ModelTier::from_str("planning"), Some(ModelTier::Planning));
        assert_eq!(ModelTier::from_str("invalid"), None);
    }

    #[test]
    fn test_model_configuration() {
        let mut config = ModelConfiguration::default();
        
        config.set_model("openai", ModelTier::Light, "gpt-4o-mini");
        let models = config.get_provider_models("openai").unwrap();
        assert_eq!(models.light, "gpt-3.5-turbo");
    }
}