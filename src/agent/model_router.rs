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
    pub fn claude_opus_4_1() -> Self {
        Self {
            provider: "anthropic".to_string(),
            model: "claude-opus-4.1".to_string(),
            cost_per_1m_input: 15.0,
            cost_per_1m_output: 75.0,
            max_context: 200_000,
            tokens_per_second: 50,
        }
    }

    /// Claude Sonnet 4 - Good balance
    pub fn claude_sonnet_4() -> Self {
        Self {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4".to_string(),
            cost_per_1m_input: 3.0,
            cost_per_1m_output: 15.0,
            max_context: 200_000,
            tokens_per_second: 80,
        }
    }

    /// Claude Haiku - Fast and cheap
    pub fn claude_haiku() -> Self {
        Self {
            provider: "anthropic".to_string(),
            model: "claude-haiku".to_string(),
            cost_per_1m_input: 0.25,
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

    /// Usage statistics
    stats: Arc<RwLock<ModelUsageStats>>,

    /// Baseline model for cost comparison (typically most expensive)
    baseline_model: ModelConfig,
}

impl ModelRouter {
    /// Create a new model router with default routing table
    pub fn new() -> Self {
        let mut routing_table = HashMap::new();

        // Explorer agent routes (read-only, code analysis)
        routing_table.insert(
            (AgentType::Explorer, TaskComplexity::Low),
            ModelConfig::claude_haiku(), // Fast queries
        );
        routing_table.insert(
            (AgentType::Explorer, TaskComplexity::Medium),
            ModelConfig::claude_sonnet_4(), // Analysis needs quality
        );
        routing_table.insert(
            (AgentType::Explorer, TaskComplexity::High),
            ModelConfig::claude_opus_4_1(), // Deep understanding
        );

        // Builder agent routes (code generation)
        routing_table.insert(
            (AgentType::Builder, TaskComplexity::Low),
            ModelConfig::claude_sonnet_4(), // Even simple generation needs quality
        );
        routing_table.insert(
            (AgentType::Builder, TaskComplexity::Medium),
            ModelConfig::claude_sonnet_4(), // Default for most building
        );
        routing_table.insert(
            (AgentType::Builder, TaskComplexity::High),
            ModelConfig::claude_opus_4_1(), // Complex features need best
        );

        // Debugger agent routes (bug fixing)
        routing_table.insert(
            (AgentType::Debugger, TaskComplexity::Low),
            ModelConfig::claude_sonnet_4(), // Simple fixes
        );
        routing_table.insert(
            (AgentType::Debugger, TaskComplexity::Medium),
            ModelConfig::claude_opus_4_1(), // Most bugs need deep reasoning
        );
        routing_table.insert(
            (AgentType::Debugger, TaskComplexity::High),
            ModelConfig::claude_opus_4_1(), // Complex bugs need best
        );

        // Refactorer agent routes (code improvements)
        routing_table.insert(
            (AgentType::Refactorer, TaskComplexity::Low),
            ModelConfig::claude_sonnet_4(), // Simple refactors
        );
        routing_table.insert(
            (AgentType::Refactorer, TaskComplexity::Medium),
            ModelConfig::claude_sonnet_4(), // Most refactors
        );
        routing_table.insert(
            (AgentType::Refactorer, TaskComplexity::High),
            ModelConfig::claude_opus_4_1(), // Architecture changes
        );

        // Sub-agent routes (cheap parallelization)
        routing_table.insert(
            (AgentType::FileSearcher, TaskComplexity::Low),
            ModelConfig::claude_haiku(), // Fast parallel search
        );
        routing_table.insert(
            (AgentType::FileSearcher, TaskComplexity::Medium),
            ModelConfig::claude_haiku(), // Still want cheap
        );
        routing_table.insert(
            (AgentType::FileSearcher, TaskComplexity::High),
            ModelConfig::claude_haiku(), // Even complex searches
        );

        routing_table.insert(
            (AgentType::PatternFinder, TaskComplexity::Low),
            ModelConfig::claude_haiku(),
        );
        routing_table.insert(
            (AgentType::PatternFinder, TaskComplexity::Medium),
            ModelConfig::claude_haiku(),
        );
        routing_table.insert(
            (AgentType::PatternFinder, TaskComplexity::High),
            ModelConfig::claude_haiku(),
        );

        routing_table.insert(
            (AgentType::DependencyMapper, TaskComplexity::Low),
            ModelConfig::claude_haiku(),
        );
        routing_table.insert(
            (AgentType::DependencyMapper, TaskComplexity::Medium),
            ModelConfig::claude_haiku(),
        );
        routing_table.insert(
            (AgentType::DependencyMapper, TaskComplexity::High),
            ModelConfig::claude_haiku(),
        );

        Self {
            routing_table,
            stats: Arc::new(RwLock::new(ModelUsageStats::default())),
            baseline_model: ModelConfig::claude_opus_4_1(), // Most expensive as baseline
        }
    }

    /// Select the appropriate model for a task
    pub fn select_model(
        &self,
        agent_type: AgentType,
        complexity: TaskComplexity,
        user_override: Option<ModelConfig>,
    ) -> ModelConfig {
        // User override takes precedence
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
                // Fallback: use Sonnet for unknown combinations
                debug!(
                    "No routing rule for {:?}/{:?}, using default Sonnet",
                    agent_type, complexity
                );
                ModelConfig::claude_sonnet_4()
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

        // Explorer + Low = Haiku
        let config = router.select_model(AgentType::Explorer, TaskComplexity::Low, None);
        assert_eq!(config.model, "claude-haiku");

        // Builder + High = Opus
        let config = router.select_model(AgentType::Builder, TaskComplexity::High, None);
        assert_eq!(config.model, "claude-opus-4.1");

        // Sub-agent always Haiku (cheap parallelization)
        let config = router.select_model(AgentType::FileSearcher, TaskComplexity::High, None);
        assert_eq!(config.model, "claude-haiku");
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
        let haiku = ModelConfig::claude_haiku();
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
        let haiku = ModelConfig::claude_haiku();
        let sonnet = ModelConfig::claude_sonnet_4();

        router.record_usage(&haiku, 50_000, 25_000).await;
        router.record_usage(&sonnet, 30_000, 15_000).await;

        let report = router.generate_report().await;

        // Should contain key information
        assert!(report.contains("Total Requests: 2"));
        assert!(report.contains("Total Input Tokens: 80000"));
        assert!(report.contains("Total Output Tokens: 40000"));
        assert!(report.contains("Cost Savings"));
        assert!(report.contains("anthropic/claude-haiku"));
        assert!(report.contains("anthropic/claude-sonnet-4"));
    }
}
