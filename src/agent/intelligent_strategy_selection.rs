/// Intelligent Strategy Selection and Adaptation
///
/// Combines our IntelligenceEngine with research-based strategies for optimal performance.
/// Intelligence enhances strategy selection, adapts parameters, and learns from execution.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, debug};

use crate::agent::strategies::{ReasoningStrategy, StrategySelector};
use crate::intelligence::{IntelligenceEngine, UserIntent, ContextItem};

/// Intelligence-enhanced strategy selector
pub struct IntelligentStrategySelector {
    strategy_selector: StrategySelector,
    intelligence: Arc<IntelligenceEngine>,
    execution_history: Vec<StrategyExecution>,
    strategy_performance: HashMap<String, StrategyMetrics>,
}

#[derive(Debug, Clone)]
pub struct StrategyExecution {
    pub task: String,
    pub strategy_name: String,
    pub success: bool,
    pub execution_time_ms: u64,
    pub phases_completed: usize,
    pub learned_patterns: Vec<String>,
    pub confidence_score: f32,
}

#[derive(Debug, Clone, Default)]
pub struct StrategyMetrics {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub average_time_ms: u64,
    pub average_confidence: f32,
    pub common_failure_patterns: Vec<String>,
    pub success_patterns: Vec<String>,
}

/// Enhanced strategy with intelligence parameters
pub struct AdaptedStrategy {
    pub base_strategy: ReasoningStrategy,
    pub confidence_threshold: f32,
    pub max_exploration_depth: usize,
    pub should_use_reflection: bool,
    pub should_use_tree_search: bool,
    pub context_window_percentage: f32,
    pub adapted_parameters: HashMap<String, serde_json::Value>,
}

impl IntelligentStrategySelector {
    pub fn new(intelligence: Arc<IntelligenceEngine>) -> Result<Self> {
        Ok(Self {
            strategy_selector: StrategySelector::default()?,
            intelligence,
            execution_history: Vec::new(),
            strategy_performance: HashMap::new(),
        })
    }

    /// Select and adapt strategy using intelligence
    pub async fn select_intelligent_strategy(
        &self,
        task: &str,
        context: &[ContextItem],
    ) -> Result<AdaptedStrategy> {
        // Step 1: Intelligence analyzes the task
        let analysis = self.intelligence.analyze_task(task, context).await?;
        let intent = analysis.intent;
        let complexity = analysis.complexity_score;
        let confidence = analysis.confidence_score;

        info!(
            "Intelligence analysis: intent={:?}, complexity={:.2}, confidence={:.2}",
            intent, complexity, confidence
        );

        // Step 2: Get relevant patterns from past executions
        let relevant_patterns = self.get_relevant_patterns(task, &intent);

        // Step 3: Select base strategy with intelligence input
        let base_strategy = self.select_with_intelligence(task, &intent, complexity, &relevant_patterns)?;

        // Step 4: Adapt strategy parameters based on intelligence
        let adapted = self.adapt_strategy(base_strategy, complexity, confidence, &analysis);

        info!(
            "Selected strategy '{}' with adaptations: confidence_threshold={:.2}, max_depth={}, reflection={}, tree_search={}",
            adapted.base_strategy.name,
            adapted.confidence_threshold,
            adapted.max_exploration_depth,
            adapted.should_use_reflection,
            adapted.should_use_tree_search
        );

        Ok(adapted)
    }

    /// Select strategy with intelligence enhancement
    fn select_with_intelligence(
        &self,
        task: &str,
        intent: &UserIntent,
        complexity: f32,
        patterns: &[String],
    ) -> Result<ReasoningStrategy> {
        // Check if we have successful patterns for this type of task
        let pattern_match = self.find_successful_pattern_match(intent, patterns);

        if let Some(strategy_name) = pattern_match {
            if let Some(strategy) = self.strategy_selector.get_strategy(&strategy_name) {
                info!("Using previously successful strategy '{}' based on pattern match", strategy_name);
                return Ok(strategy.clone());
            }
        }

        // Intelligent strategy selection based on intent and complexity
        let strategy_name = match (intent, complexity) {
            // High complexity tasks need sophisticated strategies
            (_, c) if c > 0.8 => {
                if task.contains("fix") || task.contains("bug") {
                    "swe_bench_strategy"
                } else {
                    "tree_of_thoughts"  // For complex reasoning
                }
            },

            // Improvement tasks benefit from reflection
            (UserIntent::Refactor, _) | (UserIntent::Optimize, _) => "reflexion",

            // Debugging needs systematic exploration
            (UserIntent::Debug, _) | (UserIntent::Fix, _) => "swe_bench_strategy",

            // Understanding tasks use ReAct
            (UserIntent::Explain, _) | (UserIntent::Analyze, _) => "react",

            // Code generation with planning
            (UserIntent::Generate, c) if c > 0.5 => "interactive_planning",

            // Workflow for well-defined tasks
            (UserIntent::Execute, _) => "workflow_orchestration",

            // Default to ReAct
            _ => "react",
        };

        let strategy = self.strategy_selector.get_strategy(strategy_name)
            .ok_or_else(|| anyhow::anyhow!("Strategy '{}' not found", strategy_name))?
            .clone();

        Ok(strategy)
    }

    /// Adapt strategy parameters based on intelligence analysis
    fn adapt_strategy(
        &self,
        base_strategy: ReasoningStrategy,
        complexity: f32,
        confidence: f32,
        analysis: &crate::intelligence::TaskAnalysis,
    ) -> AdaptedStrategy {
        // Determine adaptations based on analysis
        let should_use_reflection =
            complexity > 0.6 ||
            confidence < 0.5 ||
            base_strategy.name.to_lowercase().contains("reflexion");

        let should_use_tree_search =
            complexity > 0.7 &&
            analysis.has_multiple_solution_paths;

        let max_exploration_depth = if complexity > 0.8 {
            10  // Deep exploration for complex tasks
        } else if complexity > 0.5 {
            5   // Moderate exploration
        } else {
            3   // Quick exploration for simple tasks
        };

        // Adjust confidence threshold based on task criticality
        let confidence_threshold = if analysis.is_critical {
            0.9  // High confidence required for critical tasks
        } else if confidence < 0.5 {
            0.6  // Lower threshold when we're uncertain
        } else {
            0.7  // Standard threshold
        };

        // Context window management based on complexity
        let context_window_percentage = if complexity > 0.7 {
            0.4  // Use 40% of context for complex tasks (more room for reasoning)
        } else {
            0.6  // Use 60% for simpler tasks
        };

        let mut adapted_parameters = HashMap::new();

        // Add intelligence-specific parameters
        adapted_parameters.insert(
            "use_semantic_search".to_string(),
            serde_json::json!(analysis.requires_codebase_search),
        );

        adapted_parameters.insert(
            "parallel_exploration".to_string(),
            serde_json::json!(complexity > 0.6),
        );

        adapted_parameters.insert(
            "max_retries".to_string(),
            serde_json::json!(if confidence < 0.5 { 5 } else { 3 }),
        );

        AdaptedStrategy {
            base_strategy,
            confidence_threshold,
            max_exploration_depth,
            should_use_reflection,
            should_use_tree_search,
            context_window_percentage,
            adapted_parameters,
        }
    }

    /// Learn from strategy execution
    pub async fn learn_from_execution(&mut self, execution: StrategyExecution) -> Result<()> {
        // Record execution
        self.execution_history.push(execution.clone());

        // Update strategy metrics
        let metrics = self.strategy_performance
            .entry(execution.strategy_name.clone())
            .or_default();

        metrics.total_executions += 1;
        if execution.success {
            metrics.successful_executions += 1;
            for pattern in &execution.learned_patterns {
                if !metrics.success_patterns.contains(pattern) {
                    metrics.success_patterns.push(pattern.clone());
                }
            }
        } else {
            for pattern in &execution.learned_patterns {
                if !metrics.common_failure_patterns.contains(pattern) {
                    metrics.common_failure_patterns.push(pattern.clone());
                }
            }
        }

        // Update averages
        let total = metrics.total_executions as u64;
        metrics.average_time_ms =
            (metrics.average_time_ms * (total - 1) + execution.execution_time_ms) / total;
        metrics.average_confidence =
            (metrics.average_confidence * (total as f32 - 1.0) + execution.confidence_score) / total as f32;

        // Tell intelligence engine about the outcome
        self.intelligence.learn_from_execution(
            &execution.task,
            &execution.strategy_name,
            execution.success,
            execution.learned_patterns.clone(),
        ).await?;

        info!(
            "Learned from execution: strategy='{}', success={}, patterns={}",
            execution.strategy_name,
            execution.success,
            execution.learned_patterns.len()
        );

        Ok(())
    }

    /// Get relevant patterns from history
    fn get_relevant_patterns(&self, task: &str, intent: &UserIntent) -> Vec<String> {
        let mut patterns = Vec::new();

        // Look for similar past executions
        for execution in &self.execution_history {
            // Simple similarity check (in production, use embeddings)
            if execution.task.to_lowercase().contains(&task.to_lowercase()[..task.len().min(10)]) ||
               self.intents_similar(&self.classify_intent(&execution.task), intent) {
                patterns.extend(execution.learned_patterns.clone());
            }
        }

        patterns
    }

    /// Find successful strategy for similar patterns
    fn find_successful_pattern_match(&self, intent: &UserIntent, patterns: &[String]) -> Option<String> {
        let mut best_match: Option<(String, f32)> = None;

        for (strategy_name, metrics) in &self.strategy_performance {
            if metrics.total_executions == 0 {
                continue;
            }

            let success_rate = metrics.successful_executions as f32 / metrics.total_executions as f32;

            // Check if this strategy has been successful with similar patterns
            let pattern_match_score = self.calculate_pattern_match_score(
                patterns,
                &metrics.success_patterns,
            );

            let combined_score = success_rate * 0.7 + pattern_match_score * 0.3;

            if combined_score > 0.6 {
                if best_match.is_none() || best_match.as_ref().unwrap().1 < combined_score {
                    best_match = Some((strategy_name.clone(), combined_score));
                }
            }
        }

        best_match.map(|(name, _)| name)
    }

    /// Calculate pattern matching score
    fn calculate_pattern_match_score(&self, patterns: &[String], success_patterns: &[String]) -> f32 {
        if patterns.is_empty() || success_patterns.is_empty() {
            return 0.0;
        }

        let mut matches = 0;
        for pattern in patterns {
            if success_patterns.iter().any(|sp| sp.contains(pattern) || pattern.contains(sp)) {
                matches += 1;
            }
        }

        matches as f32 / patterns.len() as f32
    }

    /// Simple intent classification (placeholder)
    fn classify_intent(&self, task: &str) -> UserIntent {
        let task_lower = task.to_lowercase();

        if task_lower.contains("fix") || task_lower.contains("bug") {
            UserIntent::Fix
        } else if task_lower.contains("debug") {
            UserIntent::Debug
        } else if task_lower.contains("refactor") || task_lower.contains("improve") {
            UserIntent::Refactor
        } else if task_lower.contains("generate") || task_lower.contains("create") {
            UserIntent::Generate
        } else if task_lower.contains("explain") || task_lower.contains("understand") {
            UserIntent::Explain
        } else {
            UserIntent::Unknown
        }
    }

    /// Check if intents are similar
    fn intents_similar(&self, intent1: &UserIntent, intent2: &UserIntent) -> bool {
        use UserIntent::*;

        match (intent1, intent2) {
            (Fix, Fix) | (Fix, Debug) | (Debug, Fix) | (Debug, Debug) => true,
            (Refactor, Refactor) | (Refactor, Optimize) | (Optimize, Refactor) => true,
            (Generate, Generate) | (Generate, Create) | (Create, Generate) => true,
            (Explain, Explain) | (Explain, Analyze) | (Analyze, Explain) => true,
            _ => false,
        }
    }

    /// Get strategy recommendation with reasoning
    pub fn get_strategy_recommendation(&self, task: &str) -> String {
        let mut recommendation = String::new();

        // Check historical performance
        if !self.execution_history.is_empty() {
            recommendation.push_str("Based on historical data:\n");

            for (strategy, metrics) in &self.strategy_performance {
                if metrics.total_executions > 0 {
                    let success_rate = metrics.successful_executions as f32 / metrics.total_executions as f32;
                    recommendation.push_str(&format!(
                        "- {}: {:.1}% success rate ({}/{} executions)\n",
                        strategy,
                        success_rate * 100.0,
                        metrics.successful_executions,
                        metrics.total_executions
                    ));
                }
            }
        }

        // Add intelligence insights
        recommendation.push_str("\nIntelligence recommendations:\n");
        recommendation.push_str("- Use reflection for uncertain tasks (confidence < 50%)\n");
        recommendation.push_str("- Enable tree search for complex problems (complexity > 70%)\n");
        recommendation.push_str("- Increase exploration depth for critical tasks\n");

        recommendation
    }
}