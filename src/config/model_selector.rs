use anyhow::Result;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::config::{ConfigManager, MultiProviderConfig, ModelPreference, TaskPreference, FallbackStrategy};
use crate::providers::{LLMProvider, ProviderManager};
use crate::cost::CostTracker;

#[derive(Debug, Clone)]
pub struct ModelSelector {
    config: MultiProviderConfig,
}

#[derive(Debug, Clone)]
pub struct ProviderSelection {
    pub provider: String,
    pub model_name: String,
    pub cost_estimate: Option<f64>,
    pub reasoning: String,
}

#[derive(Debug, Clone)]
pub struct TaskSelection {
    pub provider: String,
    pub model: String,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub system_prompt_override: Option<String>,
    pub reasoning: String,
}

#[derive(Debug, Clone)]
pub struct SelectionContext {
    pub message_content: String,
    pub estimated_input_tokens: u32,
    pub estimated_output_tokens: u32,
    pub max_cost: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskType {
    AgentCoding,
    AgentPlanning,  
    AgentAnalysis,
    GeneralChat,
    CreativeWriting,
    QuickQuestions,
    CodeReview,
    Documentation,
    Summarization,
}

impl TaskType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "agent_coding" => Some(TaskType::AgentCoding),
            "agent_planning" => Some(TaskType::AgentPlanning),
            "agent_analysis" => Some(TaskType::AgentAnalysis), 
            "general_chat" => Some(TaskType::GeneralChat),
            "creative_writing" => Some(TaskType::CreativeWriting),
            "quick_questions" => Some(TaskType::QuickQuestions),
            "code_review" => Some(TaskType::CodeReview),
            "documentation" => Some(TaskType::Documentation),
            "summarization" => Some(TaskType::Summarization),
            _ => None,
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            TaskType::AgentCoding => "agent_coding".to_string(),
            TaskType::AgentPlanning => "agent_planning".to_string(),
            TaskType::AgentAnalysis => "agent_analysis".to_string(),
            TaskType::GeneralChat => "general_chat".to_string(),
            TaskType::CreativeWriting => "creative_writing".to_string(),
            TaskType::QuickQuestions => "quick_questions".to_string(),
            TaskType::CodeReview => "code_review".to_string(),
            TaskType::Documentation => "documentation".to_string(),
            TaskType::Summarization => "summarization".to_string(),
        }
    }
    
    /// Infer task type from message content
    pub fn infer_from_message(content: &str) -> Self {
        let content_lower = content.to_lowercase();
        
        // Agent-related patterns
        if content_lower.contains("read file") || content_lower.contains("list files") || 
           content_lower.contains("search code") || content_lower.contains("run command") {
            return TaskType::AgentCoding;
        }
        
        // Code-related patterns
        if content_lower.contains("function") || content_lower.contains("class") ||
           content_lower.contains("implementation") || content_lower.contains("debug") ||
           content_lower.contains("refactor") || content_lower.contains("optimize") {
            return TaskType::CodeReview;
        }
        
        // Documentation patterns
        if content_lower.contains("document") || content_lower.contains("explain") ||
           content_lower.contains("readme") || content_lower.contains("guide") {
            return TaskType::Documentation;
        }
        
        // Summarization patterns
        if content_lower.contains("summarize") || content_lower.contains("tldr") ||
           content_lower.contains("brief") || content_lower.contains("overview") {
            return TaskType::Summarization;
        }
        
        // Creative patterns
        if content_lower.contains("write a story") || content_lower.contains("creative") ||
           content_lower.contains("poem") || content_lower.contains("narrative") {
            return TaskType::CreativeWriting;
        }
        
        // Quick question patterns (short messages)
        if content.len() < 50 && (content_lower.contains("?") || 
           content_lower.starts_with("what") || content_lower.starts_with("how") ||
           content_lower.starts_with("why") || content_lower.starts_with("when")) {
            return TaskType::QuickQuestions;
        }
        
        // Default to general chat
        TaskType::GeneralChat
    }
}

impl SelectionContext {
    pub fn from_message(content: &str) -> Self {
        let estimated_input_tokens = (content.len() / 4) as u32; // Rough token estimation
        let estimated_output_tokens = match content.len() {
            0..=100 => 200,      // Short messages get moderate responses
            101..=500 => 800,    // Medium messages get detailed responses
            _ => 1500,           // Long messages get comprehensive responses
        };
        
        Self {
            message_content: content.to_string(),
            estimated_input_tokens,
            estimated_output_tokens,
            max_cost: None,
        }
    }
}

impl ModelSelector {
    pub fn new(config: &ConfigManager) -> Self {
        Self {
            config: config.multi_provider.clone(),
        }
    }
    
    /// Select the best provider for a specific model
    pub async fn select_provider_for_model(
        &self,
        model: &str,
        providers: &ProviderManager,
        context: &SelectionContext,
    ) -> Result<ProviderSelection> {
        debug!("Selecting provider for model: {}", model);
        
        // Get model preferences or fallback to default provider selection
        let model_prefs = match self.config.model_preferences.get(model) {
            Some(prefs) => prefs.clone(),
            None => {
                // No specific preferences - try all available providers
                return self.select_fallback_provider(model, providers, context).await;
            }
        };
        
        // Sort preferences by priority
        let mut sorted_prefs = model_prefs;
        sorted_prefs.sort_by_key(|p| p.priority);
        
        // Try each provider in priority order
        for pref in sorted_prefs {
            if let Some(provider) = providers.get_provider_or_host(&pref.provider) {
                // Check if provider is healthy
                if let Ok(healthy) = provider.health_check().await {
                    if !healthy {
                        debug!("Provider {} is unhealthy, skipping", pref.provider);
                        continue;
                    }
                }
                
                // Check conditions if specified
                if let Some(conditions) = &pref.conditions {
                    if !self.check_provider_conditions(provider, conditions).await {
                        debug!("Provider {} doesn't meet conditions, skipping", pref.provider);
                        continue;
                    }
                }
                
                // Calculate cost estimate
                let cost_estimate = provider.calculate_cost(
                    context.estimated_input_tokens,
                    context.estimated_output_tokens,
                ).map(|cost| cost * pref.cost_multiplier);
                
                // Check if cost is acceptable
                if let Some(max_cost) = context.max_cost {
                    if let Some(estimated_cost) = cost_estimate {
                        if estimated_cost > max_cost {
                            debug!("Provider {} cost ${:.4} exceeds max ${:.4}", 
                                  pref.provider, estimated_cost, max_cost);
                            continue;
                        }
                    }
                }
                
                return Ok(ProviderSelection {
                    provider: pref.provider.clone(),
                    model_name: model.to_string(),
                    cost_estimate,
                    reasoning: format!(
                        "Selected {} (priority {}) for {} - cost: ${:.4}",
                        pref.provider, pref.priority, model,
                        cost_estimate.unwrap_or(0.0)
                    ),
                });
            }
        }
        
        // No preferences worked, try fallback
        self.select_fallback_provider(model, providers, context).await
    }
    
    /// Select provider and model for a specific task type
    pub async fn select_for_task(
        &self,
        task: TaskType,
        providers: &ProviderManager,
        context: &SelectionContext,
    ) -> Result<TaskSelection> {
        debug!("Selecting provider for task: {:?}", task);
        
        // Get task preferences or use default
        let task_pref = self.config.task_preferences
            .get(&task.to_string())
            .or_else(|| self.config.task_preferences.get("general_chat"))
            .cloned()
            .unwrap_or_else(|| TaskPreference {
                model: "claude-3-5-sonnet-20241022".to_string(),
                temperature: Some(0.7),
                max_tokens: Some(2000),
                system_prompt_override: None,
            });
        
        // Select provider for the preferred model
        let provider_selection = self.select_provider_for_model(
            &task_pref.model,
            providers,
            context,
        ).await?;
        
        Ok(TaskSelection {
            provider: provider_selection.provider,
            model: provider_selection.model_name,
            temperature: task_pref.temperature,
            max_tokens: task_pref.max_tokens,
            system_prompt_override: task_pref.system_prompt_override,
            reasoning: format!(
                "{} (task: {:?})",
                provider_selection.reasoning,
                task
            ),
        })
    }
    
    /// Select provider when no specific preferences exist
    async fn select_fallback_provider(
        &self,
        model: &str,
        providers: &ProviderManager,
        context: &SelectionContext,
    ) -> Result<ProviderSelection> {
        debug!("Using fallback provider selection for model: {}", model);
        
        let all_providers = providers.list_all();
        let mut candidates = Vec::new();
        
        // Check each provider to see if it supports the model
        for provider_name in all_providers {
            if let Some(provider) = providers.get_provider_or_host(&provider_name) {
                // For now, assume all providers support the model
                // In a real implementation, you'd check provider.supports_model(model)
                let cost_estimate = provider.calculate_cost(
                    context.estimated_input_tokens,
                    context.estimated_output_tokens,
                );
                
                candidates.push((provider_name, cost_estimate));
            }
        }
        
        if candidates.is_empty() {
            return Err(anyhow::anyhow!("No providers available for model: {}", model));
        }
        
        // Apply fallback strategy
        let selected = match self.config.fallback_strategy.strategy.as_str() {
            "cheapest" => {
                candidates.sort_by(|a, b| {
                    a.1.unwrap_or(f64::MAX).partial_cmp(&b.1.unwrap_or(f64::MAX)).unwrap()
                });
                candidates.into_iter().next().unwrap()
            }
            "fastest" => {
                // For now, just pick the first one
                // In a real implementation, you'd track response times
                candidates.into_iter().next().unwrap()
            }
            _ => { // "cost_aware" or "quality"
                // Sort by cost but prefer known good providers
                candidates.sort_by(|a, b| {
                    a.1.unwrap_or(f64::MAX).partial_cmp(&b.1.unwrap_or(f64::MAX)).unwrap()
                });
                candidates.into_iter().next().unwrap()
            }
        };
        
        Ok(ProviderSelection {
            provider: selected.0,
            model_name: model.to_string(),
            cost_estimate: selected.1,
            reasoning: format!(
                "Fallback selection: {} for {} using {} strategy",
                selected.0, model, self.config.fallback_strategy.strategy
            ),
        })
    }
    
    /// Check if a provider meets the specified conditions
    async fn check_provider_conditions(
        &self,
        provider: &dyn LLMProvider,
        conditions: &crate::config::ProviderConditions,
    ) -> bool {
        // Check usage percentage for subscription providers
        if let Some(max_usage) = conditions.max_usage_percent {
            if let Ok(Some(usage_info)) = provider.get_usage_info().await {
                if usage_info.usage_percentage > max_usage {
                    return false;
                }
            }
        }
        
        // Check required features
        for feature in &conditions.required_features {
            match feature.as_str() {
                "streaming" => {
                    // Check if provider supports streaming
                    // For now, assume all providers support streaming
                }
                "tools" => {
                    if !provider.supports_tools() {
                        return false;
                    }
                }
                "vision" => {
                    if !provider.supports_vision() {
                        return false;
                    }
                }
                _ => {
                    warn!("Unknown feature requirement: {}", feature);
                }
            }
        }
        
        // Check time restrictions
        if let Some(time_restrictions) = &conditions.time_restrictions {
            let now = chrono::Utc::now();
            let hour = now.hour() as u8;
            let weekday = now.weekday().num_days_from_sunday() as u8;
            
            if hour < time_restrictions.start_hour || hour > time_restrictions.end_hour {
                return false;
            }
            
            if !time_restrictions.days.contains(&weekday) {
                return false;
            }
        }
        
        true
    }
    
    /// Get available models for a task type
    pub fn get_available_models_for_task(&self, task: TaskType) -> Vec<String> {
        let task_key = task.to_string();
        
        // Get task-specific model if configured
        if let Some(task_pref) = self.config.task_preferences.get(&task_key) {
            vec![task_pref.model.clone()]
        } else {
            // Return all models with preferences
            self.config.model_preferences.keys().cloned().collect()
        }
    }
    
    /// Update config with new preferences
    pub fn update_config(&mut self, config: MultiProviderConfig) {
        self.config = config;
        info!("Model selector config updated");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_type_inference() {
        assert_eq!(TaskType::infer_from_message("read file main.rs"), TaskType::AgentCoding);
        assert_eq!(TaskType::infer_from_message("what is rust?"), TaskType::QuickQuestions);
        assert_eq!(TaskType::infer_from_message("write a story about dragons"), TaskType::CreativeWriting);
        assert_eq!(TaskType::infer_from_message("explain this function"), TaskType::Documentation);
        assert_eq!(TaskType::infer_from_message("summarize this code"), TaskType::Summarization);
        assert_eq!(TaskType::infer_from_message("hello there!"), TaskType::GeneralChat);
    }
    
    #[test]
    fn test_selection_context() {
        let context = SelectionContext::from_message("short message");
        assert!(context.estimated_input_tokens < 10);
        assert_eq!(context.estimated_output_tokens, 200);
        
        let context = SelectionContext::from_message(&"a".repeat(300));
        assert!(context.estimated_input_tokens > 50);
        assert_eq!(context.estimated_output_tokens, 800);
    }
}