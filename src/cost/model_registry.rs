use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{warn, info};

/// External model registry loaded from TOML file
/// This allows updating model recommendations without code changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistry {
    pub meta: RegistryMeta,
    pub quality_tiers: HashMap<String, QualityTierInfo>,
    pub providers: HashMap<String, ProviderInfo>,
    pub task_recommendations: HashMap<String, TaskRecommendation>,
    pub pricing_hints: Option<HashMap<String, HashMap<String, PricingHint>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMeta {
    pub version: String,
    pub last_updated: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTierInfo {
    pub description: String,
    pub models: Vec<String>,
    pub use_for: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub flagship_models: Vec<String>,
    pub coding_specialist: String,
    pub reasoning_specialist: String,
    pub cost_effective: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecommendation {
    pub description: String,
    pub minimum_tier: String,
    pub preferred_models: Vec<String>,
    pub fallback_free: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingHint {
    pub input_per_1m: f64,
    pub output_per_1m: f64,
}

impl ModelRegistry {
    /// Load model registry from TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read model registry: {:?}", path.as_ref()))?;
        
        let registry: ModelRegistry = toml::from_str(&content)
            .context("Failed to parse model registry TOML")?;
        
        info!(
            "Loaded model registry v{} (updated: {})", 
            registry.meta.version, 
            registry.meta.last_updated
        );
        
        Ok(registry)
    }

    /// Load with fallback to embedded defaults if file doesn't exist
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(&path) {
            Ok(registry) => registry,
            Err(e) => {
                warn!("Failed to load model registry from {:?}: {}", path.as_ref(), e);
                warn!("Using embedded default registry (may be outdated)");
                Self::embedded_default()
            }
        }
    }

    /// Get the best model for a specific task
    pub fn get_best_model_for_task(&self, task: &str, allow_paid: bool) -> Option<String> {
        let task_rec = self.task_recommendations.get(task)?;
        
        // For critical tasks, prefer flagship models even if expensive
        if self.is_critical_task(task) && allow_paid {
            // Try preferred models first
            for model in &task_rec.preferred_models {
                if self.is_model_available(model) {
                    return Some(model.clone());
                }
            }
        }
        
        // If not allowing paid models or no preferred models available, use free fallbacks
        if !allow_paid {
            for model in &task_rec.fallback_free {
                if self.is_model_available(model) {
                    return Some(model.clone());
                }
            }
        }
        
        // Final fallback - first preferred model even if not available
        task_rec.preferred_models.first().cloned()
    }

    /// Get the best model for a provider
    pub fn get_best_model_for_provider(&self, provider: &str, task_type: Option<&str>) -> Option<String> {
        let provider_info = self.providers.get(provider)?;
        
        // If we know the task type, try to get a specialist model
        if let Some(task) = task_type {
            let specialist = match task {
                "code_review" | "code_generation" | "debugging" => &provider_info.coding_specialist,
                "architecture_review" | "refactoring" => &provider_info.reasoning_specialist,
                _ => &provider_info.cost_effective,
            };
            
            if self.is_model_available(specialist) {
                return Some(specialist.clone());
            }
        }
        
        // Fallback to flagship model
        for model in &provider_info.flagship_models {
            if self.is_model_available(model) {
                return Some(model.clone());
            }
        }
        
        None
    }

    /// Check if a task is considered critical (requires flagship models)
    pub fn is_critical_task(&self, task: &str) -> bool {
        if let Some(task_rec) = self.task_recommendations.get(task) {
            task_rec.minimum_tier == "flagship"
        } else {
            false
        }
    }

    /// Get pricing hint for a model
    pub fn get_pricing_hint(&self, provider: &str, model: &str) -> Option<&PricingHint> {
        self.pricing_hints
            .as_ref()?
            .get(provider)?
            .get(model)
            .or_else(|| {
                // Try wildcard match for free models (e.g., ollama.*)
                self.pricing_hints
                    .as_ref()?
                    .get(provider)?
                    .get("*")
            })
    }

    /// Get all models in a quality tier
    pub fn get_models_in_tier(&self, tier: &str) -> Vec<String> {
        self.quality_tiers
            .get(tier)
            .map(|tier_info| tier_info.models.clone())
            .unwrap_or_default()
    }

    /// Generate a summary of current model recommendations
    pub fn generate_summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str(&format!("🤖 Model Registry v{}\n", self.meta.version));
        summary.push_str(&format!("📅 Updated: {}\n", self.meta.last_updated));
        summary.push_str(&format!("📝 {}\n\n", self.meta.source));
        
        summary.push_str("🏆 Current SOTA Models:\n");
        if let Some(flagship) = self.quality_tiers.get("flagship") {
            for model in &flagship.models {
                summary.push_str(&format!("  • {}\n", model));
            }
        }
        
        summary.push_str("\n💰 Most Cost-Effective:\n");
        for (provider, info) in &self.providers {
            summary.push_str(&format!("  • {}: {}\n", provider, info.cost_effective));
        }
        
        summary.push_str("\n🔥 Coding Specialists:\n");
        for (provider, info) in &self.providers {
            summary.push_str(&format!("  • {}: {}\n", provider, info.coding_specialist));
        }
        
        summary
    }

    /// Check if a model is available (this is a placeholder - in real implementation
    /// this would check against actual provider APIs)
    fn is_model_available(&self, model: &str) -> bool {
        // For now, assume all models in registry are potentially available
        // In real implementation, this would check provider APIs
        !model.contains("sonnet-4") && !model.contains("opus-4") && 
        !model.contains("grok-4") && !model.contains("kimi-k2") &&
        !model.contains("gemini-2.5-pro") // Models that may not be released yet
    }

    /// Embedded default registry for when external file is not available
    fn embedded_default() -> Self {
        // Minimal embedded registry - prefer loading from external file
        Self {
            meta: RegistryMeta {
                version: "embedded-fallback".to_string(),
                last_updated: "unknown".to_string(),
                source: "Embedded fallback - update models.toml for latest".to_string(),
            },
            quality_tiers: {
                let mut tiers = HashMap::new();
                tiers.insert("flagship".to_string(), QualityTierInfo {
                    description: "SOTA models".to_string(),
                    models: vec![
                        "claude-3-5-sonnet-20241022".to_string(),
                        "gpt-4o".to_string(),
                    ],
                    use_for: vec!["code_review".to_string(), "debugging".to_string()],
                });
                tiers.insert("standard".to_string(), QualityTierInfo {
                    description: "Cost-effective models".to_string(),
                    models: vec![
                        "gpt-4o-mini".to_string(),
                        "claude-3-5-haiku-20241022".to_string(),
                    ],
                    use_for: vec!["general".to_string()],
                });
                tiers
            },
            providers: {
                let mut providers = HashMap::new();
                providers.insert("openai".to_string(), ProviderInfo {
                    flagship_models: vec!["gpt-4o".to_string()],
                    coding_specialist: "gpt-4o".to_string(),
                    reasoning_specialist: "gpt-4o".to_string(),
                    cost_effective: "gpt-4o-mini".to_string(),
                    notes: None,
                });
                providers.insert("claude".to_string(), ProviderInfo {
                    flagship_models: vec!["claude-3-5-sonnet-20241022".to_string()],
                    coding_specialist: "claude-3-5-sonnet-20241022".to_string(),
                    reasoning_specialist: "claude-3-5-sonnet-20241022".to_string(),
                    cost_effective: "claude-3-5-haiku-20241022".to_string(),
                    notes: None,
                });
                providers
            },
            task_recommendations: HashMap::new(),
            pricing_hints: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_registry() {
        let toml_content = r#"
[meta]
version = "test"
last_updated = "2024-01-01"
source = "test"

[quality_tiers.flagship]
description = "SOTA"
models = ["test-model"]
use_for = ["test"]

[providers.test]
flagship_models = ["test-model"]
coding_specialist = "test-model"
reasoning_specialist = "test-model"
cost_effective = "test-model"

[task_recommendations.test]
description = "test task"
minimum_tier = "flagship"
preferred_models = ["test-model"]
fallback_free = ["free-model"]
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        
        let registry = ModelRegistry::load_from_file(temp_file.path()).unwrap();
        assert_eq!(registry.meta.version, "test");
        assert!(registry.quality_tiers.contains_key("flagship"));
        assert!(registry.providers.contains_key("test"));
    }

    #[test]
    fn test_fallback_to_default() {
        let registry = ModelRegistry::load_or_default("/nonexistent/path");
        assert_eq!(registry.meta.version, "embedded-fallback");
        assert!(!registry.providers.is_empty());
    }

    #[test]
    fn test_critical_task_detection() {
        let mut registry = ModelRegistry::embedded_default();
        registry.task_recommendations.insert("code_review".to_string(), TaskRecommendation {
            description: "test".to_string(),
            minimum_tier: "flagship".to_string(),
            preferred_models: vec![],
            fallback_free: vec![],
        });
        
        assert!(registry.is_critical_task("code_review"));
        assert!(!registry.is_critical_task("unknown_task"));
    }
}