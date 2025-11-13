use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// Real-time pricing information fetched from providers or pricing APIs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivePricingData {
    pub provider: String,
    pub model: String,
    pub input_cost_per_1m: f64,
    pub output_cost_per_1m: f64,
    pub last_updated: DateTime<Utc>,
    pub context_window: u32,
    pub quality_tier: QualityTier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QualityTier {
    Basic,     // gpt-3.5-turbo level
    Standard,  // gpt-4o-mini level
    Premium,   // gpt-4o level
    Flagship,  // gpt-4, claude-3.5-sonnet level
}

#[derive(Debug, Clone)]
pub struct ModelComparison {
    pub model: String,
    pub provider: String,
    pub cost_per_1k_tokens: f64, // Estimated for typical input/output mix
    pub quality_tier: QualityTier,
    pub cost_efficiency_score: f64, // Quality/Cost ratio
    pub is_recommended: bool,
}

pub struct PricingEngine {
    pricing_cache: HashMap<String, LivePricingData>, // model_id -> pricing
    cache_duration_hours: u64,
}

impl PricingEngine {
    pub fn new() -> Self {
        Self {
            pricing_cache: HashMap::new(),
            cache_duration_hours: 24, // Refresh daily
        }
    }

    /// Get current pricing for all available models, with freshness check
    pub async fn get_current_pricing(&mut self) -> Result<Vec<LivePricingData>> {
        // Check if cache is stale
        if self.is_cache_stale() {
            self.refresh_pricing().await?;
        }

        Ok(self.pricing_cache.values().cloned().collect())
    }

    /// Compare models for a specific task, sorted by cost-efficiency
    pub async fn compare_models_for_task(
        &mut self,
        minimum_quality: QualityTier,
        estimated_input_tokens: u32,
        estimated_output_tokens: u32,
    ) -> Result<Vec<ModelComparison>> {
        let pricing_data = self.get_current_pricing().await?;
        let mut comparisons = Vec::new();

        for pricing in pricing_data {
            // Skip models below minimum quality requirement
            if pricing.quality_tier < minimum_quality {
                continue;
            }

            let estimated_cost = self.calculate_estimated_cost(
                &pricing,
                estimated_input_tokens,
                estimated_output_tokens,
            );

            let cost_efficiency_score = self.calculate_cost_efficiency(
                &pricing.quality_tier,
                estimated_cost,
            );

            comparisons.push(ModelComparison {
                model: pricing.model.clone(),
                provider: pricing.provider.clone(),
                cost_per_1k_tokens: estimated_cost * 1000.0, // Show per 1k for readability
                quality_tier: pricing.quality_tier.clone(),
                cost_efficiency_score,
                is_recommended: false, // Will be set after sorting
            });
        }

        // Sort by cost efficiency (higher is better)
        comparisons.sort_by(|a, b| {
            b.cost_efficiency_score
                .partial_cmp(&a.cost_efficiency_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Mark top 3 as recommended
        for (i, comparison) in comparisons.iter_mut().enumerate() {
            if i < 3 {
                comparison.is_recommended = true;
            }
        }

        Ok(comparisons)
    }

    /// Get the best model for critical tasks (code review, architecture)
    pub async fn get_flagship_model(&mut self) -> Result<Option<ModelComparison>> {
        let comparisons = self
            .compare_models_for_task(QualityTier::Flagship, 2000, 1000)
            .await?;

        // For critical tasks, prioritize quality over cost
        let best_flagship = comparisons
            .into_iter()
            .filter(|c| c.quality_tier == QualityTier::Flagship)
            .min_by(|a, b| a.cost_per_1k_tokens.partial_cmp(&b.cost_per_1k_tokens).unwrap_or(std::cmp::Ordering::Equal));

        Ok(best_flagship)
    }

    /// Get real-time pricing from provider APIs or pricing services
    async fn refresh_pricing(&mut self) -> Result<()> {
        debug!("Refreshing model pricing data...");

        // In a real implementation, this would fetch from:
        // - OpenAI API pricing endpoints
        // - Anthropic pricing pages
        // - Third-party pricing aggregators
        // - Provider documentation

        // For now, use current known pricing (should be replaced with API calls)
        let current_pricing = vec![
            // OpenAI Models (as of latest known pricing)
            LivePricingData {
                provider: "openai".to_string(),
                model: "gpt-4o".to_string(),
                input_cost_per_1m: 5.0,
                output_cost_per_1m: 15.0,
                last_updated: Utc::now(),
                context_window: 128_000,
                quality_tier: QualityTier::Flagship,
            },
            LivePricingData {
                provider: "openai".to_string(),
                model: "gpt-4o-mini".to_string(),
                input_cost_per_1m: 0.15,
                output_cost_per_1m: 0.6,
                last_updated: Utc::now(),
                context_window: 128_000,
                quality_tier: QualityTier::Standard,
            },
            LivePricingData {
                provider: "openai".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                input_cost_per_1m: 0.5,   // Actually more expensive than gpt-4o-mini!
                output_cost_per_1m: 1.5,
                last_updated: Utc::now(),
                context_window: 16_385,
                quality_tier: QualityTier::Basic,
            },

            // Claude Models
            LivePricingData {
                provider: "claude".to_string(),
                model: "claude-3-5-sonnet-20241022".to_string(),
                input_cost_per_1m: 3.0,
                output_cost_per_1m: 15.0,
                last_updated: Utc::now(),
                context_window: 200_000,
                quality_tier: QualityTier::Flagship,
            },
            LivePricingData {
                provider: "claude".to_string(),
                model: "claude-3-5-haiku-20241022".to_string(),
                input_cost_per_1m: 0.25,
                output_cost_per_1m: 1.25,
                last_updated: Utc::now(),
                context_window: 200_000,
                quality_tier: QualityTier::Premium,
            },

            // Gemini Models
            LivePricingData {
                provider: "gemini".to_string(),
                model: "gemini-2.0-flash-exp".to_string(),
                input_cost_per_1m: 0.075,
                output_cost_per_1m: 0.30,
                last_updated: Utc::now(),
                context_window: 1_000_000,
                quality_tier: QualityTier::Premium,
            },

            // Ollama (Free)
            LivePricingData {
                provider: "ollama".to_string(),
                model: "llama3.3".to_string(),
                input_cost_per_1m: 0.0,
                output_cost_per_1m: 0.0,
                last_updated: Utc::now(),
                context_window: 128_000,
                quality_tier: QualityTier::Standard,
            },
        ];

        // Update cache
        self.pricing_cache.clear();
        for pricing in current_pricing {
            let key = format!("{}:{}", pricing.provider, pricing.model);
            self.pricing_cache.insert(key, pricing);
        }

        Ok(())
    }

    fn is_cache_stale(&self) -> bool {
        if self.pricing_cache.is_empty() {
            return true;
        }

        // Check if any pricing data is older than cache_duration_hours
        let stale_threshold = Utc::now() - chrono::Duration::hours(self.cache_duration_hours as i64);

        self.pricing_cache.values().any(|pricing| pricing.last_updated < stale_threshold)
    }

    fn calculate_estimated_cost(
        &self,
        pricing: &LivePricingData,
        input_tokens: u32,
        output_tokens: u32,
    ) -> f64 {
        (input_tokens as f64 / 1_000_000.0) * pricing.input_cost_per_1m +
        (output_tokens as f64 / 1_000_000.0) * pricing.output_cost_per_1m
    }

    fn calculate_cost_efficiency(&self, quality: &QualityTier, cost: f64) -> f64 {
        let quality_score = match quality {
            QualityTier::Basic => 1.0,
            QualityTier::Standard => 2.0,
            QualityTier::Premium => 3.0,
            QualityTier::Flagship => 4.0,
        };

        if cost > 0.0 {
            quality_score / cost
        } else {
            f64::INFINITY // Free models have infinite cost efficiency
        }
    }

    /// Generate a pricing report for transparency
    pub async fn generate_pricing_report(&mut self) -> Result<String> {
        let pricing_data = self.get_current_pricing().await?;
        let mut report = String::new();

        report.push_str("📊 Current Model Pricing Report\n");
        report.push_str("================================\n\n");

        // Group by quality tier
        let mut by_tier: HashMap<QualityTier, Vec<&LivePricingData>> = HashMap::new();
        for pricing in &pricing_data {
            by_tier.entry(pricing.quality_tier.clone()).or_default().push(pricing);
        }

        for (tier, models) in by_tier {
            report.push_str(&format!("🏆 {:?} Models:\n", tier));
            for model in models {
                let est_cost = self.calculate_estimated_cost(model, 1000, 500);
                report.push_str(&format!(
                    "  {} ({}): ${:.4}/1.5k tokens | {}k context\n",
                    model.model,
                    model.provider,
                    est_cost,
                    model.context_window / 1000
                ));
            }
            report.push('\n');
        }

        // Show surprising findings
        report.push_str("💡 Key Insights:\n");

        // Find if gpt-3.5-turbo is more expensive than newer models
        if let (Some(gpt35), Some(gpt4mini)) = (
            pricing_data.iter().find(|p| p.model.contains("gpt-3.5-turbo")),
            pricing_data.iter().find(|p| p.model.contains("gpt-4o-mini"))
        ) {
            let gpt35_cost = self.calculate_estimated_cost(gpt35, 1000, 500);
            let gpt4mini_cost = self.calculate_estimated_cost(gpt4mini, 1000, 500);

            if gpt35_cost > gpt4mini_cost {
                report.push_str(&format!(
                    "  ⚠️  gpt-3.5-turbo (${:.4}) is MORE expensive than gpt-4o-mini (${:.4})!\n",
                    gpt35_cost, gpt4mini_cost
                ));
            }
        }

        // Find best cost-efficiency models
        let mut efficiency_sorted = pricing_data.clone();
        efficiency_sorted.sort_by(|a, b| {
            let a_eff = self.calculate_cost_efficiency(&a.quality_tier, self.calculate_estimated_cost(a, 1000, 500));
            let b_eff = self.calculate_cost_efficiency(&b.quality_tier, self.calculate_estimated_cost(b, 1000, 500));
            b_eff.partial_cmp(&a_eff).unwrap_or(std::cmp::Ordering::Equal)
        });

        report.push_str(&format!(
            "  🏅 Best cost-efficiency: {} ({})\n",
            efficiency_sorted[0].model, efficiency_sorted[0].provider
        ));

        report.push_str(&format!("\nLast updated: {}\n", Utc::now().format("%Y-%m-%d %H:%M UTC")));

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pricing_refresh() {
        let mut engine = PricingEngine::new();
        assert!(engine.is_cache_stale());

        engine.refresh_pricing().await.unwrap();
        assert!(!engine.is_cache_stale());
        assert!(!engine.pricing_cache.is_empty());
    }

    #[tokio::test]
    async fn test_model_comparison() {
        let mut engine = PricingEngine::new();
        engine.refresh_pricing().await.unwrap();

        let comparisons = engine
            .compare_models_for_task(QualityTier::Standard, 1000, 500)
            .await
            .unwrap();

        assert!(!comparisons.is_empty());

        // Should be sorted by cost efficiency
        for i in 1..comparisons.len() {
            assert!(comparisons[i-1].cost_efficiency_score >= comparisons[i].cost_efficiency_score);
        }
    }

    #[tokio::test]
    async fn test_gpt35_vs_gpt4mini_pricing() {
        let mut engine = PricingEngine::new();
        engine.refresh_pricing().await.unwrap();

        let gpt35 = engine.pricing_cache.values()
            .find(|p| p.model.contains("gpt-3.5-turbo"))
            .unwrap();
        let gpt4mini = engine.pricing_cache.values()
            .find(|p| p.model.contains("gpt-4o-mini"))
            .unwrap();

        let gpt35_cost = engine.calculate_estimated_cost(gpt35, 1000, 500);
        let gpt4mini_cost = engine.calculate_estimated_cost(gpt4mini, 1000, 500);

        // This test documents the surprising fact that gpt-3.5-turbo is more expensive
        assert!(gpt35_cost > gpt4mini_cost,
            "gpt-3.5-turbo (${:.4}) should be more expensive than gpt-4o-mini (${:.4})",
            gpt35_cost, gpt4mini_cost);
    }
}
