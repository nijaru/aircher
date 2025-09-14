use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

use crate::config::{ConfigManager, ModelConfig};
use crate::cost::{CostTracker, QualityTier};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelectionConfig {
    pub auto_select: bool,
    pub prefer_cost_efficient: bool,
    pub fallback_to_cheaper: bool,
    pub task_mappings: HashMap<String, String>, // task -> model
    pub provider_priorities: Vec<String>,       // ordered list of providers to try
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskType {
    CommitMessages,
    QuickQuestions,
    Summaries,
    Documentation,
    CodeReview,
    Debugging,
    Refactoring,
    CodeGeneration,
    ArchitectureReview,
    General,
}

#[derive(Debug, Clone)]
pub struct ModelRecommendation {
    pub provider: String,
    pub model: String,
    pub reasoning: String,
    pub estimated_cost: f64,
    pub quality_score: f32,      // 0.0-1.0
    pub cost_efficiency: f32,    // 0.0-1.0
    pub alternative_models: Vec<ModelAlternative>,
}

#[derive(Debug, Clone)]
pub struct ModelAlternative {
    pub provider: String,
    pub model: String,
    pub cost_savings: f64,
    pub quality_difference: f32, // negative means lower quality
    pub reason: String,
}

impl Default for ModelSelectionConfig {
    fn default() -> Self {
        let mut task_mappings = HashMap::new();
        
        // Ultra-cheap tasks - use most cost-efficient models
        task_mappings.insert("commit_messages".to_string(), "gpt-4o-mini".to_string()); // Actually cheaper than gpt-3.5-turbo!
        task_mappings.insert("quick_questions".to_string(), "gpt-4o-mini".to_string());
        task_mappings.insert("summaries".to_string(), "claude-3-5-haiku".to_string());
        
        // Balanced cost/quality tasks
        task_mappings.insert("documentation".to_string(), "claude-3-5-haiku".to_string());
        
        // CRITICAL TASKS - Always use SOTA models regardless of cost
        task_mappings.insert("code_review".to_string(), "claude-3-5-sonnet-20241022".to_string()); // Best for code analysis
        task_mappings.insert("debugging".to_string(), "gpt-4o".to_string()); // Strong reasoning needed
        task_mappings.insert("refactoring".to_string(), "claude-3-5-sonnet-20241022".to_string()); // Architectural understanding
        task_mappings.insert("code_generation".to_string(), "gpt-4o".to_string()); // High-quality code output
        task_mappings.insert("architecture_review".to_string(), "claude-3-5-sonnet-20241022".to_string()); // Critical decisions

        Self {
            auto_select: true,
            prefer_cost_efficient: true, // Except for critical tasks
            fallback_to_cheaper: true,
            task_mappings,
            provider_priorities: vec![
                "ollama".to_string(),    // Free first (for non-critical tasks)
                "gemini".to_string(),    // Very cost-effective
                "claude".to_string(),    // Best reasoning
                "openai".to_string(),    // Good general performance
            ],
        }
    }
}

impl TaskType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "commit" | "commit_messages" | "commit_message" => TaskType::CommitMessages,
            "question" | "quick_questions" | "quick_question" | "q&a" => TaskType::QuickQuestions,
            "summary" | "summaries" | "summarize" => TaskType::Summaries,
            "docs" | "documentation" | "document" => TaskType::Documentation,
            "review" | "code_review" | "code_reviews" => TaskType::CodeReview,
            "debug" | "debugging" | "troubleshoot" => TaskType::Debugging,
            "refactor" | "refactoring" | "refactorings" => TaskType::Refactoring,
            "generate" | "code_generation" | "coding" => TaskType::CodeGeneration,
            "architecture" | "architecture_review" | "design" => TaskType::ArchitectureReview,
            _ => TaskType::General,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            TaskType::CommitMessages => "commit_messages".to_string(),
            TaskType::QuickQuestions => "quick_questions".to_string(),
            TaskType::Summaries => "summaries".to_string(),
            TaskType::Documentation => "documentation".to_string(),
            TaskType::CodeReview => "code_review".to_string(),
            TaskType::Debugging => "debugging".to_string(),
            TaskType::Refactoring => "refactoring".to_string(),
            TaskType::CodeGeneration => "code_generation".to_string(),
            TaskType::ArchitectureReview => "architecture_review".to_string(),
            TaskType::General => "general".to_string(),
        }
    }

    /// Returns minimum quality tier required for this task type
    pub fn minimum_quality_tier(&self) -> QualityTier {
        match self {
            TaskType::CommitMessages => QualityTier::Basic,        // Simple text generation
            TaskType::QuickQuestions => QualityTier::Standard,     // Need decent understanding
            TaskType::Summaries => QualityTier::Standard,          // Need good comprehension
            TaskType::Documentation => QualityTier::Premium,       // Need clear communication
            
            // CRITICAL TASKS - Always require flagship models
            TaskType::CodeReview => QualityTier::Flagship,         // Cannot miss bugs
            TaskType::Debugging => QualityTier::Flagship,          // Complex reasoning required
            TaskType::Refactoring => QualityTier::Flagship,        // Architectural changes are risky
            TaskType::CodeGeneration => QualityTier::Flagship,     // Must produce correct code
            TaskType::ArchitectureReview => QualityTier::Flagship, // Business-critical decisions
            
            TaskType::General => QualityTier::Premium,             // Default to good quality
        }
    }

    /// Returns whether this task type should override cost preferences
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            TaskType::CodeReview 
            | TaskType::Debugging 
            | TaskType::Refactoring 
            | TaskType::CodeGeneration 
            | TaskType::ArchitectureReview
        )
    }

    /// Returns typical token usage for this task type
    pub fn typical_token_usage(&self) -> (u32, u32) {
        // (input_tokens, output_tokens)
        match self {
            TaskType::CommitMessages => (200, 100),      // Short input, short output
            TaskType::QuickQuestions => (300, 200),      // Short input, short output
            TaskType::Summaries => (2000, 300),         // Long input, medium output
            TaskType::Documentation => (1000, 1000),    // Medium input, long output
            TaskType::CodeReview => (3000, 1500),       // Long input, long output
            TaskType::Debugging => (2000, 1000),        // Medium input, medium output
            TaskType::Refactoring => (4000, 2000),      // Long input, long output
            TaskType::CodeGeneration => (1500, 2000),   // Medium input, long output
            TaskType::ArchitectureReview => (3000, 2000), // Long input, long output
            TaskType::General => (1000, 1000),          // Default medium
        }
    }
}

pub struct IntelligentModelSelector {
    config: ModelSelectionConfig,
    available_models: HashMap<String, Vec<ModelConfig>>, // provider -> models
}

impl IntelligentModelSelector {
    pub fn new(config_manager: &ConfigManager) -> Self {
        let mut available_models = HashMap::new();
        
        // Extract available models from each provider
        for (provider_name, provider_config) in &config_manager.providers {
            available_models.insert(provider_name.clone(), provider_config.models.clone());
        }

        Self {
            config: ModelSelectionConfig::default(),
            available_models,
        }
    }

    pub fn with_config(mut self, config: ModelSelectionConfig) -> Self {
        self.config = config;
        self
    }

    /// Select the best model for a given task and budget constraints
    pub fn select_model(
        &self,
        task_type: Option<TaskType>,
        cost_tracker: &CostTracker,
        max_cost: Option<f64>,
        user_message: &str,
    ) -> Result<ModelRecommendation> {
        let task = task_type.unwrap_or_else(|| self.infer_task_type(user_message));
        
        debug!("Selecting model for task type: {:?}", task);

        // Get task-specific mapping if available
        let preferred_model = self.config.task_mappings.get(&task.to_string());
        
        let mut candidates = self.get_model_candidates(&task, preferred_model)?;
        
        // Filter by budget constraints
        if let Some(max_cost) = max_cost {
            candidates.retain(|c| c.estimated_cost <= max_cost);
        }

        // Filter by current budget status
        let (input_tokens, output_tokens) = task.typical_token_usage();
        candidates.retain(|c| {
            let estimate = cost_tracker.estimate_cost(
                &c.provider,
                &c.model,
                input_tokens,
                output_tokens,
                Some((
                    self.get_model_config(&c.provider, &c.model)
                        .map(|m| m.input_cost_per_1m)
                        .unwrap_or(0.0),
                    self.get_model_config(&c.provider, &c.model)
                        .map(|m| m.output_cost_per_1m)
                        .unwrap_or(0.0),
                )),
            );
            !estimate.will_exceed_daily && !estimate.will_exceed_monthly
        });

        if candidates.is_empty() {
            // Fallback to free models if budget constraints are too tight
            candidates = self.get_free_model_fallbacks()?;
        }

        // Sort by cost efficiency if prefer_cost_efficient is enabled
        if self.config.prefer_cost_efficient {
            candidates.sort_by(|a, b| {
                a.cost_efficiency
                    .partial_cmp(&b.cost_efficiency)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .reverse()
            });
        } else {
            // Sort by quality
            candidates.sort_by(|a, b| {
                a.quality_score
                    .partial_cmp(&b.quality_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .reverse()
            });
        }

        let best_candidate = candidates
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No suitable models found for task"))?;

        info!(
            "Selected model: {} ({}) for task {:?} - Cost: ${:.4}",
            best_candidate.model, best_candidate.provider, task, best_candidate.estimated_cost
        );

        Ok(best_candidate)
    }

    /// Infer task type from user message content
    fn infer_task_type(&self, message: &str) -> TaskType {
        let message_lower = message.to_lowercase();

        // Check for question patterns first (most specific)
        if message_lower.contains("?") || message_lower.starts_with("how") ||
           message_lower.starts_with("what") || message_lower.starts_with("why") ||
           message_lower.starts_with("where") || message_lower.starts_with("when") {
            return TaskType::QuickQuestions;
        }

        // Check for summary patterns
        if message_lower.contains("summarize") || message_lower.contains("summary") ||
           message_lower.contains("tldr") || message_lower.contains("brief") {
            return TaskType::Summaries;
        }

        // Check for documentation patterns
        if message_lower.contains("document") || message_lower.contains("readme") ||
           message_lower.contains("docs") || message_lower.contains("explain") {
            return TaskType::Documentation;
        }

        // Check for code review patterns
        if message_lower.contains("review") || message_lower.contains("check") ||
           message_lower.contains("analyze") || message_lower.contains("audit") {
            return TaskType::CodeReview;
        }

        // Check for debugging patterns
        if message_lower.contains("debug") || message_lower.contains("error") ||
           message_lower.contains("problem") || message_lower.contains("issue") ||
           message_lower.contains("troubleshoot") {
            return TaskType::Debugging;
        }

        // Check for refactoring patterns
        if message_lower.contains("refactor") || message_lower.contains("improve") ||
           message_lower.contains("optimize") || message_lower.contains("restructure") {
            return TaskType::Refactoring;
        }

        // Check for code generation patterns
        if message_lower.contains("write") || message_lower.contains("create") ||
           message_lower.contains("implement") || message_lower.contains("generate") ||
           message_lower.contains("build") {
            return TaskType::CodeGeneration;
        }

        // Check for architecture patterns
        if message_lower.contains("architecture") || message_lower.contains("design") ||
           message_lower.contains("pattern") || message_lower.contains("structure") {
            return TaskType::ArchitectureReview;
        }

        // Check for commit message patterns
        if message_lower.contains("commit") || message_lower.starts_with("add") ||
           message_lower.starts_with("fix") || message_lower.starts_with("update") {
            return TaskType::CommitMessages;
        }

        // Short messages without specific patterns might be commit messages
        if message_lower.len() < 30 && !message_lower.contains("?") && !message_lower.contains("please") {
            return TaskType::CommitMessages;
        }

        TaskType::General
    }

    fn get_model_candidates(
        &self,
        task: &TaskType,
        preferred_model: Option<&String>,
    ) -> Result<Vec<ModelRecommendation>> {
        let mut candidates = Vec::new();
        let quality_requirement = task.minimum_quality_tier();
        let (input_tokens, output_tokens) = task.typical_token_usage();

        for provider in &self.config.provider_priorities {
            if let Some(models) = self.available_models.get(provider) {
                for model in models {
                    let quality_score = self.calculate_quality_score(provider, &model.name);
                    
                    // Skip models that don't meet quality requirements
                    let required_score = match quality_requirement {
                        QualityTier::Basic => 0.3,
                        QualityTier::Standard => 0.6,
                        QualityTier::Premium => 0.8,
                        QualityTier::Flagship => 0.9,
                    };
                    if quality_score < required_score {
                        continue;
                    }

                    let estimated_cost = (input_tokens as f64 / 1_000_000.0) * model.input_cost_per_1m +
                                       (output_tokens as f64 / 1_000_000.0) * model.output_cost_per_1m;

                    let cost_efficiency = if estimated_cost > 0.0 {
                        quality_score / (estimated_cost as f32)
                    } else {
                        f32::INFINITY // Free models have infinite cost efficiency
                    };

                    let reasoning = if Some(&model.name) == preferred_model {
                        format!("Task-optimized model for {}", task.to_string())
                    } else if estimated_cost == 0.0 {
                        "Free local model".to_string()
                    } else if cost_efficiency > 100.0 {
                        "Excellent cost efficiency".to_string()
                    } else if quality_score > 0.9 {
                        "Premium quality model".to_string()
                    } else {
                        "Balanced cost and quality".to_string()
                    };

                    let alternatives = self.generate_alternatives(provider, &model.name, estimated_cost, quality_score);

                    candidates.push(ModelRecommendation {
                        provider: provider.clone(),
                        model: model.name.clone(),
                        reasoning,
                        estimated_cost,
                        quality_score,
                        cost_efficiency,
                        alternative_models: alternatives,
                    });
                }
            }
        }

        Ok(candidates)
    }

    fn get_free_model_fallbacks(&self) -> Result<Vec<ModelRecommendation>> {
        let mut fallbacks = Vec::new();
        
        if let Some(ollama_models) = self.available_models.get("ollama") {
            for model in ollama_models {
                fallbacks.push(ModelRecommendation {
                    provider: "ollama".to_string(),
                    model: model.name.clone(),
                    reasoning: "Budget constraint fallback - free local model".to_string(),
                    estimated_cost: 0.0,
                    quality_score: 0.7, // Assume decent quality for Ollama models
                    cost_efficiency: f32::INFINITY,
                    alternative_models: vec![],
                });
            }
        }

        Ok(fallbacks)
    }

    fn calculate_quality_score(&self, provider: &str, model: &str) -> f32 {
        // Model quality scoring based on known capabilities
        match (provider, model) {
            // Premium models
            ("openai", "gpt-4") => 1.0,
            ("openai", "gpt-4-turbo") => 0.98,
            ("openai", "o1-preview") => 0.95,
            ("claude", "claude-3-opus-20240229") => 0.97,
            ("claude", "claude-3-5-sonnet-20241022") => 0.95,
            
            // High-quality models
            ("openai", "gpt-4o") => 0.92,
            ("openai", "o1-mini") => 0.88,
            ("claude", "claude-3-5-haiku-20241022") => 0.85,
            ("gemini", "gemini-1.5-pro") => 0.87,
            ("gemini", "gemini-2.0-flash-exp") => 0.83,
            
            // Mid-range models
            ("openai", "gpt-4o-mini") => 0.75,
            ("openai", "gpt-3.5-turbo") => 0.65,
            ("gemini", "gemini-1.5-flash") => 0.70,
            
            // Local models (variable quality, assume decent)
            ("ollama", "llama3.3") => 0.75,
            ("ollama", "llama3.1") => 0.70,
            ("ollama", "codellama") => 0.65,
            ("ollama", "mistral") => 0.60,
            ("ollama", "phi3") => 0.55,
            ("ollama", "qwen2.5") => 0.60,
            
            // Default scoring
            _ => 0.50,
        }
    }

    fn generate_alternatives(&self, current_provider: &str, current_model: &str, current_cost: f64, current_quality: f32) -> Vec<ModelAlternative> {
        let mut alternatives = Vec::new();

        // Find cheaper alternatives with acceptable quality loss
        for provider in &self.config.provider_priorities {
            if let Some(models) = self.available_models.get(provider) {
                for model in models {
                    if provider == current_provider && model.name == current_model {
                        continue; // Skip the current model
                    }

                    let model_cost = model.input_cost_per_1m + model.output_cost_per_1m; // Simplified cost estimate
                    let model_quality = self.calculate_quality_score(provider, &model.name);
                    
                    if model_cost < current_cost || model_cost == 0.0 {
                        let cost_savings = current_cost - model_cost;
                        let quality_difference = model_quality - current_quality;
                        
                        let reason = if model_cost == 0.0 {
                            "Free local alternative".to_string()
                        } else if cost_savings > current_cost * 0.5 {
                            "Significant cost savings".to_string()
                        } else {
                            "Moderate cost savings".to_string()
                        };

                        alternatives.push(ModelAlternative {
                            provider: provider.clone(),
                            model: model.name.clone(),
                            cost_savings,
                            quality_difference,
                            reason,
                        });
                    }
                }
            }
        }

        // Sort by cost savings
        alternatives.sort_by(|a, b| b.cost_savings.partial_cmp(&a.cost_savings).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return top 3 alternatives
        alternatives.truncate(3);
        alternatives
    }

    fn get_model_config(&self, provider: &str, model: &str) -> Option<&ModelConfig> {
        self.available_models
            .get(provider)?
            .iter()
            .find(|m| m.name == model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_type_inference() {
        let selector = IntelligentModelSelector::new(&ConfigManager::default());
        
        assert_eq!(selector.infer_task_type("How do I implement async functions?"), TaskType::QuickQuestions);
        assert_eq!(selector.infer_task_type("Fix memory leak in allocation"), TaskType::CommitMessages);
        assert_eq!(selector.infer_task_type("Please summarize this code"), TaskType::Summaries);
        assert_eq!(selector.infer_task_type("Review my implementation for bugs"), TaskType::CodeReview);
        assert_eq!(selector.infer_task_type("Debug this error message"), TaskType::Debugging);
        assert_eq!(selector.infer_task_type("Refactor this function to be cleaner"), TaskType::Refactoring);
        assert_eq!(selector.infer_task_type("Write a function that parses JSON"), TaskType::CodeGeneration);
    }

    #[test]
    fn test_quality_requirements() {
        assert_eq!(TaskType::CommitMessages.minimum_quality_tier(), QualityTier::Basic);
        assert_eq!(TaskType::ArchitectureReview.minimum_quality_tier(), QualityTier::Flagship);
        assert!(TaskType::CodeReview.minimum_quality_tier() >= TaskType::QuickQuestions.minimum_quality_tier());
    }
}