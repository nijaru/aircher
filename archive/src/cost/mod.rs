use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

pub mod model_selection;
pub mod pricing_engine;
pub mod model_registry;
pub mod simple_selection;
pub mod provider_models;
pub mod pricing_api;
pub mod embedding_manager;
pub mod smart_embedding_setup;
pub mod transparent_embedding_system;
pub mod best_embedding_strategy_2025;
pub mod practical_embedding_reality;
pub mod swerank_integration;
pub mod auto_selection;
pub mod embedding_lifecycle;
pub use model_selection::{IntelligentModelSelector, ModelRecommendation, TaskType};
pub use pricing_engine::{PricingEngine, LivePricingData, QualityTier, ModelComparison};
pub use model_registry::{ModelRegistry, TaskRecommendation};
pub use simple_selection::{SimpleModelSelector, CostAwareSelector};
pub use provider_models::{ModelConfiguration, ProviderModels, ModelTier, ModelCapabilities, CostTier, detect_model_capabilities, format_capability_indicators, get_estimated_pricing, format_model_with_pricing, get_estimated_pricing_live, format_model_with_pricing_live};
pub use pricing_api::{PricingAPI, ModelsDevResponse, ModelInfo, ModelPricing};
pub use embedding_manager::{EmbeddingManager, EmbeddingModel, EmbeddingConfig};
pub use smart_embedding_setup::{SmartEmbeddingSetup, SmartSetupEngine, SetupStrategy, SetupRecommendation, SystemInfo};
pub use transparent_embedding_system::{TransparentEmbeddingSystem, SystemState, SystemStatus, PerformanceMetrics, TransparencyConfig, ReliabilityConfig};
pub use best_embedding_strategy_2025::{BestEmbeddingStrategy2025, PerformanceTier, DeploymentMethod};
pub use practical_embedding_reality::{PracticalEmbeddingReality, IntegrationEffort, BinaryEmbeddingAnalysis, PracticalRecommendation};
pub use swerank_integration::{SweRankEmbedModel, ModelInfo as SweRankModelInfo, cosine_similarity};
pub use auto_selection::{AutoSelectionEngine, SelectionCriteria, TaskType as AutoTaskType, ModelPerformance};
pub use embedding_lifecycle::{EmbeddingLifecycleManager, ModelRegistry as EmbeddingModelRegistry, ModelEntry, StorageInfo, UpdateInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTracker {
    pub daily_usage: HashMap<String, DailyUsage>, // provider -> usage
    pub monthly_total: f64,
    pub daily_total: f64,
    pub config: CostConfig,
    pub last_reset: u64, // timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    pub date: String, // YYYY-MM-DD format
    pub total_cost: f64,
    pub requests: u32,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub models_used: HashMap<String, ModelUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub requests: u32,
    pub cost: f64,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostConfig {
    pub monthly_budget: f64,
    pub daily_limit: f64,
    pub alert_threshold: f64, // 0.0-1.0 percentage
    pub auto_downgrade_on_limit: bool,
    pub provider_limits: HashMap<String, f64>, // provider -> daily limit
    pub task_budgets: HashMap<String, f64>,    // task -> monthly budget
}

#[derive(Debug, Clone)]
pub struct CostEstimate {
    pub estimated_cost: f64,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub provider: String,
    pub model: String,
    pub will_exceed_daily: bool,
    pub will_exceed_monthly: bool,
}

#[derive(Debug, Clone)]
pub enum CostDecision {
    Allow,
    RequireConfirmation(String), // reason
    Deny(String),                // reason
    SuggestAlternative(String),  // alternative model/provider
}

impl Default for CostConfig {
    fn default() -> Self {
        let mut provider_limits = HashMap::new();
        provider_limits.insert("openai".to_string(), 5.0);
        provider_limits.insert("claude".to_string(), 3.0);
        provider_limits.insert("gemini".to_string(), 2.0);
        provider_limits.insert("ollama".to_string(), 0.0);

        let mut task_budgets = HashMap::new();
        task_budgets.insert("commit_messages".to_string(), 1.0);
        task_budgets.insert("summaries".to_string(), 2.0);
        task_budgets.insert("quick_questions".to_string(), 8.0);
        task_budgets.insert("code_review".to_string(), 12.0);
        task_budgets.insert("documentation".to_string(), 3.0);
        task_budgets.insert("debugging".to_string(), 10.0);

        Self {
            monthly_budget: 30.0,
            daily_limit: 2.0,
            alert_threshold: 0.75,
            auto_downgrade_on_limit: true,
            provider_limits,
            task_budgets,
        }
    }
}

impl CostTracker {
    pub fn new(config: CostConfig) -> Self {
        Self {
            daily_usage: HashMap::new(),
            monthly_total: 0.0,
            daily_total: 0.0,
            config,
            last_reset: Self::current_timestamp(),
        }
    }

    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn current_date() -> String {
        use chrono::{DateTime, Utc};
        let dt: DateTime<Utc> = DateTime::from(SystemTime::now());
        dt.format("%Y-%m-%d").to_string()
    }

    pub fn reset_if_needed(&mut self) {
        let now = Self::current_timestamp();
        let current_date = Self::current_date();

        // Reset daily totals if it's a new day
        if let Some(last_usage) = self.daily_usage.values().next() {
            if last_usage.date != current_date {
                self.daily_total = 0.0;
                info!("Reset daily usage totals for new day: {}", current_date);
            }
        }

        // Reset monthly totals if it's a new month (simple 30-day window)
        if now - self.last_reset > 30 * 24 * 60 * 60 {
            self.monthly_total = 0.0;
            self.last_reset = now;
            info!("Reset monthly usage totals after 30 days");
        }
    }

    pub fn estimate_cost(
        &self,
        provider: &str,
        model: &str,
        estimated_input_tokens: u32,
        estimated_output_tokens: u32,
        pricing: Option<(f64, f64)>, // (input_cost_per_1m, output_cost_per_1m)
    ) -> CostEstimate {
        let estimated_cost = if let Some((input_rate, output_rate)) = pricing {
            (estimated_input_tokens as f64 / 1_000_000.0) * input_rate
                + (estimated_output_tokens as f64 / 1_000_000.0) * output_rate
        } else {
            0.0
        };

        let will_exceed_daily = self.daily_total + estimated_cost > self.config.daily_limit;
        let will_exceed_monthly = self.monthly_total + estimated_cost > self.config.monthly_budget;

        CostEstimate {
            estimated_cost,
            input_tokens: estimated_input_tokens,
            output_tokens: estimated_output_tokens,
            provider: provider.to_string(),
            model: model.to_string(),
            will_exceed_daily,
            will_exceed_monthly,
        }
    }

    pub fn check_request_allowed(&self, estimate: &CostEstimate) -> CostDecision {
        // Free providers always allowed
        if estimate.estimated_cost == 0.0 {
            return CostDecision::Allow;
        }

        // Check hard limits
        if self.monthly_total + estimate.estimated_cost > self.config.monthly_budget {
            if self.config.auto_downgrade_on_limit {
                return CostDecision::SuggestAlternative(
                    "Monthly budget exceeded. Try ollama for free local models?".to_string(),
                );
            } else {
                return CostDecision::Deny(format!(
                    "Would exceed monthly budget of ${:.2}",
                    self.config.monthly_budget
                ));
            }
        }

        if self.daily_total + estimate.estimated_cost > self.config.daily_limit {
            if self.config.auto_downgrade_on_limit {
                return CostDecision::SuggestAlternative(
                    "Daily limit reached. Try a cheaper model or ollama?".to_string(),
                );
            } else {
                return CostDecision::Deny(format!(
                    "Would exceed daily limit of ${:.2}",
                    self.config.daily_limit
                ));
            }
        }

        // Check provider-specific limits
        if let Some(&provider_limit) = self.config.provider_limits.get(&estimate.provider) {
            let provider_daily = self
                .daily_usage
                .get(&estimate.provider)
                .map(|u| u.total_cost)
                .unwrap_or(0.0);

            if provider_daily + estimate.estimated_cost > provider_limit {
                return CostDecision::SuggestAlternative(format!(
                    "Provider daily limit reached (${:.2}). Try a different provider?",
                    provider_limit
                ));
            }
        }

        // Check alert thresholds
        let monthly_percentage = (self.monthly_total + estimate.estimated_cost) / self.config.monthly_budget;
        let daily_percentage = (self.daily_total + estimate.estimated_cost) / self.config.daily_limit;

        if monthly_percentage > self.config.alert_threshold {
            return CostDecision::RequireConfirmation(format!(
                "This request will use {:.1}% of your monthly budget (${:.3}). Continue?",
                monthly_percentage * 100.0,
                estimate.estimated_cost
            ));
        }

        if daily_percentage > self.config.alert_threshold {
            return CostDecision::RequireConfirmation(format!(
                "This request will use {:.1}% of your daily limit (${:.3}). Continue?",
                daily_percentage * 100.0,
                estimate.estimated_cost
            ));
        }

        // High-cost requests require confirmation
        if estimate.estimated_cost > 0.10 {
            return CostDecision::RequireConfirmation(format!(
                "High cost request: ${:.3}. Continue?",
                estimate.estimated_cost
            ));
        }

        CostDecision::Allow
    }

    pub fn record_usage(
        &mut self,
        provider: &str,
        model: &str,
        actual_cost: f64,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<()> {
        self.reset_if_needed();

        let current_date = Self::current_date();

        // Update daily usage for provider
        let usage = self
            .daily_usage
            .entry(provider.to_string())
            .or_insert_with(|| DailyUsage {
                date: current_date.clone(),
                total_cost: 0.0,
                requests: 0,
                input_tokens: 0,
                output_tokens: 0,
                models_used: HashMap::new(),
            });

        usage.total_cost += actual_cost;
        usage.requests += 1;
        usage.input_tokens += input_tokens;
        usage.output_tokens += output_tokens;

        // Update model-specific usage
        let model_usage = usage
            .models_used
            .entry(model.to_string())
            .or_insert_with(|| ModelUsage {
                requests: 0,
                cost: 0.0,
                input_tokens: 0,
                output_tokens: 0,
            });

        model_usage.requests += 1;
        model_usage.cost += actual_cost;
        model_usage.input_tokens += input_tokens;
        model_usage.output_tokens += output_tokens;

        // Update totals
        self.daily_total += actual_cost;
        self.monthly_total += actual_cost;

        debug!(
            "Recorded usage: {} ${:.4} ({} in, {} out) - Daily: ${:.2}, Monthly: ${:.2}",
            model, actual_cost, input_tokens, output_tokens, self.daily_total, self.monthly_total
        );

        // Check for alerts
        if self.daily_total / self.config.daily_limit > self.config.alert_threshold {
            warn!(
                "Daily usage alert: ${:.2} / ${:.2} ({:.1}%)",
                self.daily_total,
                self.config.daily_limit,
                (self.daily_total / self.config.daily_limit) * 100.0
            );
        }

        if self.monthly_total / self.config.monthly_budget > self.config.alert_threshold {
            warn!(
                "Monthly usage alert: ${:.2} / ${:.2} ({:.1}%)",
                self.monthly_total,
                self.config.monthly_budget,
                (self.monthly_total / self.config.monthly_budget) * 100.0
            );
        }

        Ok(())
    }

    pub fn get_daily_summary(&self, provider: Option<&str>) -> String {
        let current_date = Self::current_date();

        if let Some(provider) = provider {
            if let Some(usage) = self.daily_usage.get(provider) {
                format!(
                    "{}: ${:.4} ({} requests, {} tokens)",
                    provider,
                    usage.total_cost,
                    usage.requests,
                    usage.input_tokens + usage.output_tokens
                )
            } else {
                format!("{}: $0.00 (0 requests)", provider)
            }
        } else {
            format!(
                "Today ({}): ${:.4} / ${:.2} ({:.1}%)",
                current_date,
                self.daily_total,
                self.config.daily_limit,
                if self.config.daily_limit > 0.0 {
                    (self.daily_total / self.config.daily_limit) * 100.0
                } else {
                    0.0
                }
            )
        }
    }

    pub fn get_monthly_summary(&self) -> String {
        format!(
            "Monthly: ${:.2} / ${:.2} ({:.1}%)",
            self.monthly_total,
            self.config.monthly_budget,
            if self.config.monthly_budget > 0.0 {
                (self.monthly_total / self.config.monthly_budget) * 100.0
            } else {
                0.0
            }
        )
    }

    pub fn get_provider_rankings(&self) -> Vec<(String, f64, u32)> {
        let mut rankings: Vec<_> = self
            .daily_usage
            .iter()
            .map(|(provider, usage)| (provider.clone(), usage.total_cost, usage.requests))
            .collect();

        rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        rankings
    }

    pub fn suggest_cheaper_model(&self, current_provider: &str, task_type: Option<&str>) -> Option<String> {
        // Task-specific suggestions
        if let Some(task) = task_type {
            return match task {
                "commit_messages" | "quick_questions" => {
                    Some("gpt-3.5-turbo (90% cheaper) or ollama (free)".to_string())
                }
                "summaries" | "documentation" => {
                    Some("claude-3-haiku (95% cheaper) or ollama (free)".to_string())
                }
                "code_review" | "debugging" => {
                    Some("claude-3-5-sonnet (50% cheaper) or ollama (free)".to_string())
                }
                _ => Some("ollama for free local models".to_string()),
            };
        }

        // General suggestions based on current provider
        match current_provider {
            "openai" => Some("Try gemini-flash (95% cheaper) or ollama (free)".to_string()),
            "claude" => Some("Try claude-3-haiku (90% cheaper) or ollama (free)".to_string()),
            "gemini" => Some("Try ollama for free local models".to_string()),
            _ => Some("Try ollama for zero-cost local models".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_estimation() {
        let config = CostConfig::default();
        let tracker = CostTracker::new(config);

        let estimate = tracker.estimate_cost(
            "openai",
            "gpt-4",
            1000,    // input tokens
            500,     // output tokens
            Some((30.0, 60.0)), // pricing
        );

        assert_eq!(estimate.estimated_cost, 0.06); // (1000/1M * 30) + (500/1M * 60)
        assert!(!estimate.will_exceed_daily);
        assert!(!estimate.will_exceed_monthly);
    }

    #[test]
    fn test_budget_limits() {
        let mut config = CostConfig::default();
        config.daily_limit = 0.05; // Very low limit for testing

        let tracker = CostTracker::new(config);

        let estimate = tracker.estimate_cost(
            "openai",
            "gpt-4",
            2000,    // input tokens
            1000,    // output tokens
            Some((30.0, 60.0)), // pricing
        );

        assert!(estimate.will_exceed_daily);

        match tracker.check_request_allowed(&estimate) {
            CostDecision::Deny(_) | CostDecision::SuggestAlternative(_) => (),
            _ => panic!("Should deny or suggest alternative for over-budget request"),
        }
    }

    #[test]
    fn test_free_provider_always_allowed() {
        let config = CostConfig::default();
        let tracker = CostTracker::new(config);

        let estimate = tracker.estimate_cost(
            "ollama",
            "llama3.3",
            10000,   // Large number of tokens
            5000,
            Some((0.0, 0.0)), // Free pricing
        );

        assert_eq!(estimate.estimated_cost, 0.0);

        match tracker.check_request_allowed(&estimate) {
            CostDecision::Allow => (),
            _ => panic!("Free provider should always be allowed"),
        }
    }
}
