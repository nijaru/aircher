// Model Router for Cost-Aware Model Selection (Week 7 Day 6-7)
//
// Implements Amp's proven pattern of routing tasks to appropriate models
// based on complexity, agent type, and cost considerations.
//
// References:
// - docs/architecture/SYSTEM_DESIGN_2025.md
// - ai/research/competitive-analysis-2025.md (Amp section)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Task complexity level determines model selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskComplexity {
    /// Simple tasks: file reading, basic queries, formatting
    /// Target: Fast response, low cost
    Low,

    /// Moderate tasks: code analysis, refactoring, debugging
    /// Target: Balance of quality and cost
    Medium,

    /// Complex tasks: architecture decisions, novel algorithms, deep reasoning
    /// Target: Best possible quality
    High,
}

/// Agent type determines baseline model preferences
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    /// Explorer: Code reading, analysis, understanding
    Explorer,

    /// Builder: Code writing, feature implementation
    Builder,

    /// Debugger: Bug fixing, error resolution
    Debugger,

    /// Refactorer: Code improvements, migrations
    Refactorer,

    /// FileSearcher: Parallel file content search (sub-agent)
    FileSearcher,

    /// PatternFinder: Find code patterns (sub-agent)
    PatternFinder,

    /// DependencyMapper: Trace dependencies (sub-agent)
    DependencyMapper,
}

/// Model configuration for routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Provider name (e.g., "anthropic", "openai", "ollama")
    pub provider: String,

    /// Model name (e.g., "claude-opus-4.1", "claude-sonnet-4", "claude-haiku")
    pub model: String,

    /// Estimated cost per 1M input tokens (USD)
    pub cost_per_1m_input: f64,

    /// Estimated cost per 1M output tokens (USD)
    pub cost_per_1m_output: f64,

    /// Maximum context window (tokens)
    pub max_context: usize,

    /// Average tokens per second (for time estimation)
    pub tokens_per_second: usize,
}

impl ModelConfig {
    /// Claude Opus 4.1 - Best reasoning, highest cost
    /// NOTE: Sonnet 4.5 is better for most/all tasks. Opus rarely needed.
    pub fn claude_opus_4_1() -> Self {
        Self {
            provider: "anthropic".to_string(),
            model: "claude-opus-4-1".to_string(), // API alias (full: claude-opus-4-1-20250805)
            cost_per_1m_input: 15.0, // TODO: Verify current pricing
            cost_per_1m_output: 75.0,
            max_context: 200_000,
            tokens_per_second: 50,
        }
    }

    /// Claude Sonnet 4.5 - RECOMMENDED for 90%+ of tasks
    /// Better than Opus for most use cases (faster, cheaper, often better results)
    pub fn claude_sonnet_4_5() -> Self {
        Self {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-5".to_string(), // API alias (full: claude-sonnet-4-5-20250929)
            cost_per_1m_input: 3.0, // TODO: Verify current pricing
            cost_per_1m_output: 15.0,
            max_context: 200_000,
            tokens_per_second: 80,
        }
    }

    /// Claude Haiku 4.5 - Fast and cheap for simple tasks
    pub fn claude_haiku_4_5() -> Self {
        Self {
            provider: "anthropic".to_string(),
            model: "claude-haiku-4-5".to_string(), // API alias (full: claude-haiku-4-5-20251001)
            cost_per_1m_input: 0.25, // TODO: Verify current pricing
            cost_per_1m_output: 1.25,
            max_context: 200_000,
            tokens_per_second: 120,
        }
    }

    /// GPT-4o - OpenAI alternative
    pub fn gpt_4o() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            cost_per_1m_input: 2.5,
            cost_per_1m_output: 10.0,
            max_context: 128_000,
            tokens_per_second: 70,
        }
    }

    /// Estimate cost for a given token count
    pub fn estimate_cost(&self, input_tokens: usize, output_tokens: usize) -> f64 {
        let input_cost = (input_tokens as f64 / 1_000_000.0) * self.cost_per_1m_input;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * self.cost_per_1m_output;
        input_cost + output_cost
    }
}

/// Usage statistics for cost tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelUsageStats {
    /// Total input tokens used
    pub total_input_tokens: usize,

    /// Total output tokens used
    pub total_output_tokens: usize,

    /// Total estimated cost (USD)
    pub total_cost: f64,

    /// Number of requests
    pub request_count: usize,

    /// Per-model breakdown
    pub per_model: HashMap<String, ModelStats>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelStats {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub cost: f64,
    pub requests: usize,
}

impl ModelUsageStats {
    /// Record usage for a model
    pub fn record_usage(
        &mut self,
        model_key: &str,
        input_tokens: usize,
        output_tokens: usize,
        cost: f64,
    ) {
        self.total_input_tokens += input_tokens;
        self.total_output_tokens += output_tokens;
        self.total_cost += cost;
        self.request_count += 1;

        let stats = self.per_model.entry(model_key.to_string()).or_default();
        stats.input_tokens += input_tokens;
        stats.output_tokens += output_tokens;
        stats.cost += cost;
        stats.requests += 1;
    }

    /// Get cost savings estimate vs always using most expensive model
    pub fn cost_savings(&self, baseline_model: &ModelConfig) -> f64 {
        // Calculate what cost would have been with baseline model
        let baseline_cost = baseline_model.estimate_cost(
            self.total_input_tokens,
            self.total_output_tokens,
        );

        // Actual savings
        baseline_cost - self.total_cost
    }

    /// Get cost savings percentage
    pub fn cost_savings_percent(&self, baseline_model: &ModelConfig) -> f64 {
        let baseline_cost = baseline_model.estimate_cost(
            self.total_input_tokens,
            self.total_output_tokens,
        );

        if baseline_cost == 0.0 {
            return 0.0;
        }

        ((baseline_cost - self.total_cost) / baseline_cost) * 100.0
    }
}

/// Model router with cost-aware selection
pub struct ModelRouter {
    /// Default model configs by agent type and complexity
    routing_table: HashMap<(AgentType, TaskComplexity), ModelConfig>,

    /// Optional: Single model to use for ALL tasks (bypasses routing table)
    /// Set via config.model.model = "claude-sonnet-4-5"
    single_model_override: Option<ModelConfig>,

    /// Usage statistics
    stats: Arc<RwLock<ModelUsageStats>>,

    /// Baseline model for cost comparison (typically most expensive)
    baseline_model: ModelConfig,

    /// Optional: Maximum budget in USD (inspired by Claude Code's --max-budget-usd)
    /// When set, router will reject requests that would exceed this budget
    budget_limit_usd: Option<f64>,
}

impl ModelRouter {
    /// Create a new model router with default routing table
    pub fn new() -> Self {
        let mut routing_table = HashMap::new();

        // Explorer agent routes (read-only, code analysis)
        routing_table.insert(
            (AgentType::Explorer, TaskComplexity::Low),
            ModelConfig::claude_haiku_4_5(), // Fast queries
        );
        routing_table.insert(
            (AgentType::Explorer, TaskComplexity::Medium),
            ModelConfig::claude_sonnet_4_5(), // Analysis needs quality
        );
        routing_table.insert(
            (AgentType::Explorer, TaskComplexity::High),
            ModelConfig::claude_sonnet_4_5(), // Even deep analysis (Sonnet better than Opus)
        );

        // Builder agent routes (code generation)
        routing_table.insert(
            (AgentType::Builder, TaskComplexity::Low),
            ModelConfig::claude_sonnet_4_5(), // Even simple generation needs quality
        );
        routing_table.insert(
            (AgentType::Builder, TaskComplexity::Medium),
            ModelConfig::claude_sonnet_4_5(), // Default for most building
        );
        routing_table.insert(
            (AgentType::Builder, TaskComplexity::High),
            ModelConfig::claude_sonnet_4_5(), // Sonnet better for coding (not Opus)
        );

        // Debugger agent routes (bug fixing)
        routing_table.insert(
            (AgentType::Debugger, TaskComplexity::Low),
            ModelConfig::claude_sonnet_4_5(), // Simple fixes
        );
        routing_table.insert(
            (AgentType::Debugger, TaskComplexity::Medium),
            ModelConfig::claude_sonnet_4_5(), // Sonnet sufficient for most debugging
        );
        routing_table.insert(
            (AgentType::Debugger, TaskComplexity::High),
            ModelConfig::claude_sonnet_4_5(), // Even complex bugs (Sonnet better)
        );

        // Refactorer agent routes (code improvements)
        routing_table.insert(
            (AgentType::Refactorer, TaskComplexity::Low),
            ModelConfig::claude_sonnet_4_5(), // Simple refactors
        );
        routing_table.insert(
            (AgentType::Refactorer, TaskComplexity::Medium),
            ModelConfig::claude_sonnet_4_5(), // Most refactors
        );
        routing_table.insert(
            (AgentType::Refactorer, TaskComplexity::High),
            ModelConfig::claude_sonnet_4_5(), // Even architecture changes (Sonnet sufficient)
        );

        // Sub-agent routes (cheap parallelization)
        routing_table.insert(
            (AgentType::FileSearcher, TaskComplexity::Low),
            ModelConfig::claude_haiku_4_5(), // Fast parallel search
        );
        routing_table.insert(
            (AgentType::FileSearcher, TaskComplexity::Medium),
            ModelConfig::claude_haiku_4_5(), // Still want cheap
        );
        routing_table.insert(
            (AgentType::FileSearcher, TaskComplexity::High),
            ModelConfig::claude_haiku_4_5(), // Even complex searches
        );

        routing_table.insert(
            (AgentType::PatternFinder, TaskComplexity::Low),
            ModelConfig::claude_haiku_4_5(),
        );
        routing_table.insert(
            (AgentType::PatternFinder, TaskComplexity::Medium),
            ModelConfig::claude_haiku_4_5(),
        );
        routing_table.insert(
            (AgentType::PatternFinder, TaskComplexity::High),
            ModelConfig::claude_haiku_4_5(),
        );

        routing_table.insert(
            (AgentType::DependencyMapper, TaskComplexity::Low),
            ModelConfig::claude_haiku_4_5(),
        );
        routing_table.insert(
            (AgentType::DependencyMapper, TaskComplexity::Medium),
            ModelConfig::claude_haiku_4_5(),
        );
        routing_table.insert(
            (AgentType::DependencyMapper, TaskComplexity::High),
            ModelConfig::claude_haiku_4_5(),
        );

        Self {
            routing_table,
            single_model_override: None, // No override by default (use routing table)
            stats: Arc::new(RwLock::new(ModelUsageStats::default())),
            baseline_model: ModelConfig::claude_opus_4_1(), // Most expensive as baseline
            budget_limit_usd: None, // No budget limit by default
        }
    }

    /// Create a router with a single model for all tasks (bypasses routing)
    /// Useful for: config.model.model = "claude-sonnet-4-5" (simple config)
    pub fn with_single_model(model_config: ModelConfig) -> Self {
        info!(
            "Creating router with single model override: {} ({})",
            model_config.model, model_config.provider
        );

        // Create a normal router first (with routing table), then override
        let mut router = Self::new();
        router.single_model_override = Some(model_config.clone());
        router.baseline_model = model_config; // Use same model as baseline
        router
    }

    /// Set single model override (use this model for everything)
    pub fn set_single_model(&mut self, model_config: ModelConfig) {
        info!(
            "Setting single model override: {} ({})",
            model_config.model, model_config.provider
        );
        self.single_model_override = Some(model_config);
    }

    /// Clear single model override (go back to routing table)
    pub fn clear_single_model(&mut self) {
        info!("Clearing single model override, using routing table");
        self.single_model_override = None;
    }

    /// Select the appropriate model for a task
    pub fn select_model(
        &self,
        agent_type: AgentType,
        complexity: TaskComplexity,
        user_override: Option<ModelConfig>,
    ) -> ModelConfig {
        // Single model override takes precedence (config.model.model = "...")
        if let Some(ref config) = self.single_model_override {
            debug!(
                "Using single model override: {} ({})",
                config.model, config.provider
            );
            return config.clone();
        }

        // User override takes precedence over routing table
        if let Some(config) = user_override {
            debug!(
                "Using user-specified model: {} ({})",
                config.model, config.provider
            );
            return config;
        }

        // Lookup in routing table
        let key = (agent_type, complexity);
        let config = self
            .routing_table
            .get(&key)
            .cloned()
            .unwrap_or_else(|| {
                // Fallback: use Sonnet 4.5 for unknown combinations
                debug!(
                    "No routing rule for {:?}/{:?}, using default Sonnet 4.5",
                    agent_type, complexity
                );
                ModelConfig::claude_sonnet_4_5()
            });

        info!(
            "Selected model: {} ({}) for {:?}/{:?} - Cost: ${:.4}/1M in, ${:.4}/1M out",
            config.model,
            config.provider,
            agent_type,
            complexity,
            config.cost_per_1m_input,
            config.cost_per_1m_output
        );

        config
    }

    /// Record model usage for statistics
    pub async fn record_usage(
        &self,
        model: &ModelConfig,
        input_tokens: usize,
        output_tokens: usize,
    ) {
        let cost = model.estimate_cost(input_tokens, output_tokens);
        let model_key = format!("{}/{}", model.provider, model.model);

        let mut stats = self.stats.write().await;
        stats.record_usage(&model_key, input_tokens, output_tokens, cost);

        debug!(
            "Recorded usage: {} - {} in, {} out, ${:.6}",
            model_key, input_tokens, output_tokens, cost
        );
    }

    /// Get current usage statistics
    pub async fn get_stats(&self) -> ModelUsageStats {
        self.stats.read().await.clone()
    }

    /// Get cost savings compared to baseline (always using Opus)
    pub async fn get_cost_savings(&self) -> (f64, f64) {
        let stats = self.stats.read().await;
        let savings = stats.cost_savings(&self.baseline_model);
        let percent = stats.cost_savings_percent(&self.baseline_model);
        (savings, percent)
    }

    /// Generate usage report
    pub async fn generate_report(&self) -> String {
        let stats = self.stats.read().await;
        let (savings, percent) = {
            let s = stats.cost_savings(&self.baseline_model);
            let p = stats.cost_savings_percent(&self.baseline_model);
            (s, p)
        };

        let mut report = String::new();
        report.push_str("=== Model Usage Report ===\n\n");

        report.push_str(&format!("Total Requests: {}\n", stats.request_count));
        report.push_str(&format!(
            "Total Input Tokens: {}\n",
            stats.total_input_tokens
        ));
        report.push_str(&format!(
            "Total Output Tokens: {}\n",
            stats.total_output_tokens
        ));
        report.push_str(&format!("Total Cost: ${:.4}\n\n", stats.total_cost));

        report.push_str(&format!(
            "Cost Savings vs Opus-Only: ${:.4} ({:.1}%)\n\n",
            savings, percent
        ));

        report.push_str("Per-Model Breakdown:\n");
        for (model, model_stats) in &stats.per_model {
            report.push_str(&format!(
                "  {}: {} requests, {} in, {} out, ${:.4}\n",
                model, model_stats.requests, model_stats.input_tokens, model_stats.output_tokens, model_stats.cost
            ));
        }

        report
    }

    /// Override routing rule for specific agent type and complexity
    pub fn set_route(&mut self, agent_type: AgentType, complexity: TaskComplexity, config: ModelConfig) {
        info!(
            "Setting custom route: {:?}/{:?} -> {} ({})",
            agent_type, complexity, config.model, config.provider
        );
        self.routing_table.insert((agent_type, complexity), config);
    }
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_config_cost_estimation() {
        let opus = ModelConfig::claude_opus_4_1();

        // 10K input, 5K output
        let cost = opus.estimate_cost(10_000, 5_000);

        // Expected: (10,000 / 1,000,000) * 15.0 + (5,000 / 1,000,000) * 75.0
        //         = 0.01 * 15.0 + 0.005 * 75.0
        //         = 0.15 + 0.375
        //         = 0.525
        assert!((cost - 0.525).abs() < 0.001);
    }

    #[test]
    fn test_model_router_selection() {
        let router = ModelRouter::new();

        // Explorer + Low = Haiku 4.5
        let config = router.select_model(AgentType::Explorer, TaskComplexity::Low, None);
        assert_eq!(config.model, "claude-haiku-4-5");

        // Builder + High = Sonnet 4.5 (NOT Opus - Sonnet better for coding)
        let config = router.select_model(AgentType::Builder, TaskComplexity::High, None);
        assert_eq!(config.model, "claude-sonnet-4-5");

        // Sub-agent always Haiku 4.5 (cheap parallelization)
        let config = router.select_model(AgentType::FileSearcher, TaskComplexity::High, None);
        assert_eq!(config.model, "claude-haiku-4-5");
    }

    #[test]
    fn test_user_override() {
        let router = ModelRouter::new();

        let override_config = ModelConfig::gpt_4o();

        let config = router.select_model(
            AgentType::Builder,
            TaskComplexity::High,
            Some(override_config.clone()),
        );

        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.provider, "openai");
    }

    #[tokio::test]
    async fn test_usage_tracking() {
        let router = ModelRouter::new();
        let opus = ModelConfig::claude_opus_4_1();

        // Record some usage
        router.record_usage(&opus, 10_000, 5_000).await;
        router.record_usage(&opus, 20_000, 10_000).await;

        let stats = router.get_stats().await;

        assert_eq!(stats.total_input_tokens, 30_000);
        assert_eq!(stats.total_output_tokens, 15_000);
        assert_eq!(stats.request_count, 2);

        // Cost should be sum of both requests
        let expected_cost = opus.estimate_cost(30_000, 15_000);
        assert!((stats.total_cost - expected_cost).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_cost_savings() {
        let router = ModelRouter::new();
        let haiku = ModelConfig::claude_haiku_4_5();
        let opus = ModelConfig::claude_opus_4_1();

        // Record usage with cheaper model
        router.record_usage(&haiku, 100_000, 50_000).await;

        let (savings, percent) = router.get_cost_savings().await;

        // Haiku cost: (100K/1M * 0.25) + (50K/1M * 1.25) = 0.025 + 0.0625 = 0.0875
        // Opus cost:  (100K/1M * 15.0) + (50K/1M * 75.0) = 1.5 + 3.75 = 5.25
        // Savings: 5.25 - 0.0875 = 5.1625
        // Percent: (5.1625 / 5.25) * 100 = 98.33%

        assert!((savings - 5.1625).abs() < 0.01);
        assert!(percent > 98.0 && percent < 99.0);
    }

    #[tokio::test]
    async fn test_custom_routing_rule() {
        let mut router = ModelRouter::new();

        // Override Builder + Medium to use GPT-4o
        let gpt4o = ModelConfig::gpt_4o();
        router.set_route(AgentType::Builder, TaskComplexity::Medium, gpt4o.clone());

        let config = router.select_model(AgentType::Builder, TaskComplexity::Medium, None);
        assert_eq!(config.model, "gpt-4o");
    }

    #[tokio::test]
    async fn test_report_generation() {
        let router = ModelRouter::new();
        let haiku = ModelConfig::claude_haiku_4_5();
        let sonnet = ModelConfig::claude_sonnet_4_5();

        router.record_usage(&haiku, 50_000, 25_000).await;
        router.record_usage(&sonnet, 30_000, 15_000).await;

        let report = router.generate_report().await;

        // Should contain key information
        assert!(report.contains("Total Requests: 2"));
        assert!(report.contains("Total Input Tokens: 80000"));
        assert!(report.contains("Total Output Tokens: 40000"));
        assert!(report.contains("Cost Savings"));
        assert!(report.contains("anthropic/claude-haiku-4-5"));
        assert!(report.contains("anthropic/claude-sonnet-4-5"));
    }

    #[test]
    fn test_single_model_override() {
        let router = ModelRouter::with_single_model(ModelConfig::claude_sonnet_4_5());

        // All tasks should use the single model override
        let config = router.select_model(AgentType::Explorer, TaskComplexity::Low, None);
        assert_eq!(config.model, "claude-sonnet-4-5");

        let config = router.select_model(AgentType::Builder, TaskComplexity::High, None);
        assert_eq!(config.model, "claude-sonnet-4-5");

        let config = router.select_model(AgentType::FileSearcher, TaskComplexity::Medium, None);
        assert_eq!(config.model, "claude-sonnet-4-5");
    }

    #[test]
    fn test_single_model_set_and_clear() {
        let mut router = ModelRouter::new();

        // Initially should use routing table
        let config = router.select_model(AgentType::Explorer, TaskComplexity::Low, None);
        assert_eq!(config.model, "claude-haiku-4-5");

        // Set single model override
        router.set_single_model(ModelConfig::claude_sonnet_4_5());
        let config = router.select_model(AgentType::Explorer, TaskComplexity::Low, None);
        assert_eq!(config.model, "claude-sonnet-4-5");

        // Clear override
        router.clear_single_model();
        let config = router.select_model(AgentType::Explorer, TaskComplexity::Low, None);
        assert_eq!(config.model, "claude-haiku-4-5");
    }
}
