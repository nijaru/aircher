use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Simplified model selection - just good defaults + easy user override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleModelSelector {
    /// User's preferred model per provider
    pub provider_defaults: HashMap<String, String>,
    /// User's preferred model per task type (optional override)
    pub task_overrides: HashMap<String, String>,
    /// Whether to show model selection reasoning
    pub show_reasoning: bool,
}

impl Default for SimpleModelSelector {
    fn default() -> Self {
        let mut provider_defaults = HashMap::new();

        // Conservative defaults - just pick one good model per provider
        provider_defaults.insert("claude".to_string(), "claude-3-5-sonnet-20241022".to_string());
        provider_defaults.insert("openai".to_string(), "gpt-4o".to_string());
        provider_defaults.insert("gemini".to_string(), "gemini-2.0-flash-exp".to_string());
        provider_defaults.insert("ollama".to_string(), "llama3.3".to_string());
        provider_defaults.insert("openrouter".to_string(), "claude-3-5-sonnet-20241022".to_string());

        Self {
            provider_defaults,
            task_overrides: HashMap::new(),
            show_reasoning: true,
        }
    }
}

impl SimpleModelSelector {
    /// Get the model to use for a provider/task combination
    pub fn select_model(&self, provider: &str, task_hint: Option<&str>) -> (String, String) {
        // Check for task-specific override first
        if let Some(task) = task_hint {
            if let Some(model) = self.task_overrides.get(task) {
                let reason = format!("User override for {} tasks", task);
                return (model.clone(), reason);
            }
        }

        // Fall back to provider default
        if let Some(model) = self.provider_defaults.get(provider) {
            let reason = format!("Default {} model", provider);
            (model.clone(), reason)
        } else {
            // Ultimate fallback - let the provider decide
            let reason = format!("Provider {} default (not configured)", provider);
            ("auto".to_string(), reason)
        }
    }

    /// Set user's preferred model for a provider
    pub fn set_provider_default(&mut self, provider: &str, model: &str) {
        info!("Setting {} default model to {}", provider, model);
        self.provider_defaults.insert(provider.to_string(), model.to_string());
    }

    /// Set user's preferred model for a specific task type
    pub fn set_task_override(&mut self, task: &str, model: &str) {
        info!("Setting {} task override to {}", task, model);
        self.task_overrides.insert(task.to_string(), model.to_string());
    }

    /// Remove a task override
    pub fn remove_task_override(&mut self, task: &str) {
        self.task_overrides.remove(task);
    }

    /// Get current configuration as user-readable text
    pub fn get_config_summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str("🤖 Current Model Selection:\n\n");

        summary.push_str("Provider Defaults:\n");
        for (provider, model) in &self.provider_defaults {
            summary.push_str(&format!("  • {}: {}\n", provider, model));
        }

        if !self.task_overrides.is_empty() {
            summary.push_str("\nTask Overrides:\n");
            for (task, model) in &self.task_overrides {
                summary.push_str(&format!("  • {}: {}\n", task, model));
            }
        }

        summary.push_str(&format!("\nShow reasoning: {}\n", self.show_reasoning));

        summary.push_str("\n💡 To change: aircher config set-model <provider> <model>\n");
        summary.push_str("   Or for tasks: aircher config set-task <task> <model>\n");

        summary
    }

    /// Suggest some common task overrides users might want
    pub fn suggest_optimizations(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Suggest cost optimizations
        if !self.task_overrides.contains_key("commit_messages") {
            suggestions.push("Consider 'gpt-4o-mini' for commit messages (much cheaper)".to_string());
        }

        if !self.task_overrides.contains_key("summaries") {
            suggestions.push("Consider 'claude-3-5-haiku' for summaries (excellent and cheap)".to_string());
        }

        // Suggest quality optimizations
        if !self.task_overrides.contains_key("code_review") {
            suggestions.push("Consider always using 'claude-3-5-sonnet' for code reviews (best reasoning)".to_string());
        }

        // Suggest free alternatives
        suggestions.push("Try 'ollama' provider with 'llama3.3' for free local processing".to_string());

        suggestions
    }
}

/// Simple cost-aware wrapper that shows cost info but doesn't block requests
pub struct CostAwareSelector {
    pub selector: SimpleModelSelector,
    pub show_cost_warnings: bool,
    pub cost_warning_threshold: f64, // USD
}

impl CostAwareSelector {
    pub fn new(selector: SimpleModelSelector) -> Self {
        Self {
            selector,
            show_cost_warnings: true,
            cost_warning_threshold: 0.05, // Warn if estimated cost > 5 cents
        }
    }

    /// Select model and optionally warn about cost
    pub fn select_with_cost_info(
        &self,
        provider: &str,
        task_hint: Option<&str>,
        estimated_tokens: Option<(u32, u32)>, // (input, output)
    ) -> (String, String, Option<String>) {
        let (model, reason) = self.selector.select_model(provider, task_hint);

        let cost_warning = if self.show_cost_warnings {
            if let Some((input_tokens, output_tokens)) = estimated_tokens {
                self.estimate_cost_warning(provider, &model, input_tokens, output_tokens)
            } else {
                None
            }
        } else {
            None
        };

        (model, reason, cost_warning)
    }

    fn estimate_cost_warning(&self, provider: &str, model: &str, input_tokens: u32, output_tokens: u32) -> Option<String> {
        // Rough cost estimation based on known pricing
        let estimated_cost = match (provider, model) {
            ("openai", "gpt-4o") => {
                (input_tokens as f64 / 1_000_000.0) * 5.0 + (output_tokens as f64 / 1_000_000.0) * 15.0
            }
            ("openai", "gpt-4o-mini") => {
                (input_tokens as f64 / 1_000_000.0) * 0.15 + (output_tokens as f64 / 1_000_000.0) * 0.6
            }
            ("claude", "claude-3-5-sonnet-20241022") => {
                (input_tokens as f64 / 1_000_000.0) * 3.0 + (output_tokens as f64 / 1_000_000.0) * 15.0
            }
            ("claude", "claude-3-5-haiku-20241022") => {
                (input_tokens as f64 / 1_000_000.0) * 0.25 + (output_tokens as f64 / 1_000_000.0) * 1.25
            }
            ("ollama", _) => 0.0, // Free
            _ => return None, // Unknown pricing
        };

        if estimated_cost > self.cost_warning_threshold {
            Some(format!("💰 Estimated cost: ${:.3} | Free alternative: ollama", estimated_cost))
        } else if estimated_cost > 0.0 {
            Some(format!("💰 ${:.3}", estimated_cost))
        } else {
            Some("🆓 Free".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_selection() {
        let selector = SimpleModelSelector::default();

        let (model, reason) = selector.select_model("claude", None);
        assert_eq!(model, "claude-3-5-sonnet-20241022");
        assert!(reason.contains("Default claude"));
    }

    #[test]
    fn test_task_override() {
        let mut selector = SimpleModelSelector::default();
        selector.set_task_override("commit_messages", "gpt-4o-mini");

        let (model, reason) = selector.select_model("openai", Some("commit_messages"));
        assert_eq!(model, "gpt-4o-mini");
        assert!(reason.contains("User override"));
    }

    #[test]
    fn test_cost_warning() {
        let selector = SimpleModelSelector::default();
        let cost_aware = CostAwareSelector::new(selector);

        let (model, _reason, warning) = cost_aware.select_with_cost_info(
            "openai",
            None,
            Some((10000, 5000)) // Large token count
        );

        assert_eq!(model, "gpt-4o");
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("$"));
    }
}
