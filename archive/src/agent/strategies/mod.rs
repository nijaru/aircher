/// Research-based reasoning strategies for AI agents
/// Based on papers: ReAct (Google), Reflexion (Shinn et al), Tree of Thoughts (Princeton)
/// And industry implementations: Devin (Cognition), Claude (Anthropic), Cursor

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStrategy {
    pub name: String,
    pub description: String,
    pub phases: Vec<StrategyPhase>,
    #[serde(default)]
    pub max_iterations: usize,
    #[serde(default)]
    pub loop_until: Vec<String>,
    #[serde(default)]
    pub search_algorithm: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPhase {
    pub name: String,
    pub description: String,
    pub actions: Vec<PhaseAction>,
    pub success_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseAction {
    #[serde(rename = "type")]
    pub action_type: String,
    pub description: String,
    #[serde(default)]
    pub parameters: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionRule {
    pub condition: String,
    pub strategy: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub strategies: HashMap<String, ReasoningStrategy>,
    pub selection_rules: Vec<SelectionRule>,
    #[serde(default)]
    pub benchmarks: HashMap<String, HashMap<String, f64>>,
}

/// Strategy selector that picks the best strategy based on task characteristics
pub struct StrategySelector {
    config: StrategyConfig,
}

impl StrategySelector {
    /// Load strategies from YAML configuration
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: StrategyConfig = serde_yaml::from_str(&contents)?;
        Ok(Self { config })
    }

    /// Load default strategies from embedded YAML
    pub fn default() -> Result<Self> {
        const DEFAULT_STRATEGIES: &str = include_str!("reasoning_strategies.yaml");
        let config: StrategyConfig = serde_yaml::from_str(DEFAULT_STRATEGIES)?;
        Ok(Self { config })
    }

    /// Select the best strategy based on task characteristics
    pub fn select_strategy(&self, task_description: &str) -> &ReasoningStrategy {
        let task_type = self.classify_task(task_description);

        // Try to find a matching rule
        for rule in &self.config.selection_rules {
            if self.evaluate_condition(&rule.condition, &task_type, task_description) {
                if let Some(strategy) = self.config.strategies.get(&rule.strategy) {
                    tracing::info!(
                        "Selected strategy '{}' for task type '{}': {}",
                        strategy.name,
                        task_type,
                        rule.reason
                    );
                    return strategy;
                }
            }
        }

        // Fall back to ReAct as default
        self.config.strategies.get("react")
            .expect("ReAct strategy should always be present")
    }

    /// Classify task based on keywords and patterns
    fn classify_task(&self, description: &str) -> String {
        let desc_lower = description.to_lowercase();

        if desc_lower.contains("fix") || desc_lower.contains("bug") || desc_lower.contains("error")
            || desc_lower.contains("failing") || desc_lower.contains("broken") {
            return "debugging".to_string();
        }

        if desc_lower.contains("understand") || desc_lower.contains("how") || desc_lower.contains("explore")
            || desc_lower.contains("what") || desc_lower.contains("explain") {
            return "exploration".to_string();
        }

        if desc_lower.contains("refactor") || desc_lower.contains("improve") || desc_lower.contains("optimize")
            || desc_lower.contains("clean") {
            return "refactoring".to_string();
        }

        if desc_lower.contains("generate") || desc_lower.contains("create") || desc_lower.contains("implement")
            || desc_lower.contains("write") || desc_lower.contains("build") {
            return "code_generation".to_string();
        }

        if desc_lower.contains("puzzle") || desc_lower.contains("solve") || desc_lower.contains("algorithm") {
            return "complex_reasoning".to_string();
        }

        if desc_lower.contains("workflow") || desc_lower.contains("pipeline") || desc_lower.contains("process") {
            return "workflow".to_string();
        }

        "general".to_string()
    }

    /// Evaluate a selection rule condition
    fn evaluate_condition(&self, condition: &str, task_type: &str, _description: &str) -> bool {
        // Simple condition evaluation for now
        // In production, this could use a proper expression evaluator

        if condition == "default" {
            return true;
        }

        // Check task type conditions
        if condition.contains("task_type") {
            if condition.contains(&format!("'{}'", task_type))
                || condition.contains(&format!("\"{}\"", task_type)) {
                return true;
            }
        }

        // Check for OR conditions
        if condition.contains(" or ") {
            let parts: Vec<&str> = condition.split(" or ").collect();
            for part in parts {
                if part.contains(&format!("'{}'", task_type))
                    || part.contains(&format!("\"{}\"", task_type)) {
                    return true;
                }
            }
        }

        false
    }

    /// Get strategy by name
    pub fn get_strategy(&self, name: &str) -> Option<&ReasoningStrategy> {
        self.config.strategies.get(name)
    }

    /// List all available strategies
    pub fn list_strategies(&self) -> Vec<String> {
        self.config.strategies.keys().cloned().collect()
    }

    /// Get benchmark data for a strategy
    pub fn get_benchmarks(&self, strategy_name: &str) -> Option<&HashMap<String, f64>> {
        self.config.benchmarks.get(strategy_name)
    }
}

/// Convert strategy phases into our existing ReasoningPlan format
pub fn strategy_to_reasoning_phases(strategy: &ReasoningStrategy) -> Vec<crate::agent::multi_turn_reasoning::ReasoningPhase> {
    use crate::agent::multi_turn_reasoning::{ReasoningPhase, PlannedAction, ActionType};

    strategy.phases.iter().map(|phase| {
        let actions = phase.actions.iter().map(|action| {
            let action_type = match action.action_type.as_str() {
                "explore" | "search" | "index" => ActionType::Explore,
                "analyze" | "evaluate" | "classify" => ActionType::Analyze,
                "test" | "validate" | "run_tests" => ActionType::Test,
                "execute" | "implement" | "edit" | "generate" => ActionType::Implement,
                _ => ActionType::Explore,
            };

            PlannedAction {
                action_type,
                description: action.description.clone(),
                tool: determine_tool_for_action(&action.action_type),
                parameters: serde_json::Value::Object(serde_json::Map::new()),
                expected_outcome: phase.success_criteria.first()
                    .unwrap_or(&"Action completed".to_string())
                    .clone(),
                retry_count: 0,
                max_retries: 2,
            }
        }).collect();

        ReasoningPhase {
            name: phase.name.clone(),
            description: phase.description.clone(),
            actions,
            success_criteria: phase.success_criteria.clone(),
            completed: false,
            results: None,
        }
    }).collect()
}

/// Map action types to appropriate tools
fn determine_tool_for_action(action_type: &str) -> String {
    match action_type {
        "search" | "search_symbols" => "search_code".to_string(),
        "explore" | "index" => "list_files".to_string(),
        "analyze" | "parse" | "read" => "read_file".to_string(),
        "execute" | "run" | "test" | "run_tests" => "run_command".to_string(),
        "edit" | "implement" | "fix" => "edit_file".to_string(),
        "generate" | "create" => "write_file".to_string(),
        _ => "read_file".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_loading() {
        let selector = StrategySelector::default().expect("Should load default strategies");

        // Check that key strategies are present
        assert!(selector.get_strategy("react").is_some());
        assert!(selector.get_strategy("reflexion").is_some());
        assert!(selector.get_strategy("tree_of_thoughts").is_some());
        assert!(selector.get_strategy("interactive_planning").is_some());
    }

    #[test]
    fn test_task_classification() {
        let selector = StrategySelector::default().expect("Should load default strategies");

        // Test debugging task selection
        let strategy = selector.select_strategy("Fix the authentication bug in login.rs");
        assert_eq!(strategy.name, "SWE-bench Strategy");

        // Test exploration task selection
        let strategy = selector.select_strategy("Help me understand how the database connection works");
        assert_eq!(strategy.name, "ReAct");

        // Test refactoring task selection
        let strategy = selector.select_strategy("Refactor the user service to improve performance");
        assert_eq!(strategy.name, "Reflexion");
    }

    #[test]
    fn test_strategy_phases() {
        let selector = StrategySelector::default().expect("Should load default strategies");
        let react = selector.get_strategy("react").expect("ReAct should exist");

        assert_eq!(react.phases.len(), 3);
        assert_eq!(react.phases[0].name, "Thought");
        assert_eq!(react.phases[1].name, "Action");
        assert_eq!(react.phases[2].name, "Observation");
    }
}
