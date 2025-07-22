use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::info;
use uuid::Uuid;

pub mod claude_api;
pub mod gemini;
pub mod hosts;
pub mod ollama;
pub mod openai;

use crate::config::ConfigManager;
use crate::cost::{CostTracker, CostDecision, IntelligentModelSelector, TaskType};

pub struct ProviderManager {
    providers: HashMap<String, Box<dyn LLMProvider>>,
    hosts: HashMap<String, Box<dyn LLMProvider>>,
    _config: ConfigManager,
    cost_tracker: CostTracker,
    model_selector: IntelligentModelSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: bool,
    pub tools: Option<Vec<Tool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub tokens_used: Option<u32>,
    pub cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub content: String,
    pub role: MessageRole,
    pub model: String,
    pub tokens_used: u32,
    pub cost: Option<f64>,
    pub finish_reason: FinishReason,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub id: String,
    pub content: String,
    pub delta: bool, // true for delta updates, false for complete content
    pub tokens_used: Option<u32>,
    pub finish_reason: Option<FinishReason>,
}

pub type ResponseStream = mpsc::Receiver<Result<StreamChunk>>;

#[derive(Debug, Clone, PartialEq)]
pub enum PricingModel {
    PerToken {
        input_cost_per_1m: f64,
        output_cost_per_1m: f64,
        currency: String,
    },
    Subscription {
        current_usage: u64,
        limit: u64,
        reset_date: chrono::DateTime<chrono::Utc>,
        tier: SubscriptionTier,
    },
    Free,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubscriptionTier {
    Pro,
    Max,
    Team,
    Enterprise,
}

#[derive(Debug, Clone)]
pub struct PricingInfo {
    pub input_cost_per_1m: f64,
    pub output_cost_per_1m: f64,
    pub currency: String,
}

#[derive(Debug, Clone)]
pub struct UsageInfo {
    pub current_usage: u64,
    pub limit: u64,
    pub reset_date: chrono::DateTime<chrono::Utc>,
    pub usage_percentage: f64,
    pub tier: SubscriptionTier,
    pub approaching_limit: bool,
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse>;
    async fn stream(&self, req: &ChatRequest) -> Result<ResponseStream>;

    // Capabilities
    fn supports_tools(&self) -> bool;
    fn supports_vision(&self) -> bool;
    fn context_window(&self) -> u32;

    // Pricing model detection
    fn pricing_model(&self) -> PricingModel;

    // Cost management (for API-based providers)
    fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> Option<f64>;
    fn get_pricing(&self) -> Option<PricingInfo>;

    // Usage tracking (for subscription-based providers)
    async fn get_usage_info(&self) -> Result<Option<UsageInfo>>;
    fn usage_warning_threshold(&self) -> Option<f64>;

    // Health check
    async fn health_check(&self) -> Result<bool>;
}

impl ProviderManager {
    pub async fn new(config: &ConfigManager) -> Result<Self> {
        let mut providers: HashMap<String, Box<dyn LLMProvider>> = HashMap::new();
        let mut hosts: HashMap<String, Box<dyn LLMProvider>> = HashMap::new();

        // Initialize Claude API provider
        if let Some(claude_config) = config.get_provider("claude") {
            let claude_provider = claude_api::ClaudeApiProvider::new(claude_config.clone()).await?;
            providers.insert("claude".to_string(), Box::new(claude_provider));
        }

        // Initialize Gemini provider
        if let Some(gemini_config) = config.get_provider("gemini") {
            let gemini_provider = gemini::GeminiProvider::new(gemini_config.clone()).await?;
            providers.insert("gemini".to_string(), Box::new(gemini_provider));
        }

        // Initialize OpenAI provider
        if let Some(openai_config) = config.get_provider("openai") {
            let openai_provider = openai::OpenAIProvider::new(openai_config.clone())?;
            providers.insert("openai".to_string(), Box::new(openai_provider));
        }

        // Initialize Ollama provider
        if let Some(ollama_config) = config.get_provider("ollama") {
            let ollama_provider = ollama::OllamaProvider::new(ollama_config.clone()).await?;
            providers.insert("ollama".to_string(), Box::new(ollama_provider));
        }

        // Initialize OpenRouter host
        if let Some(openrouter_config) = config.get_host("openrouter") {
            let openrouter_host = hosts::OpenRouterHost::new(openrouter_config.clone()).await?;
            hosts.insert("openrouter".to_string(), Box::new(openrouter_host));
        }

        Ok(Self {
            providers,
            hosts,
            _config: config.clone(),
            cost_tracker: CostTracker::new(config.cost.clone()),
            model_selector: IntelligentModelSelector::new(config),
        })
    }

    pub fn get_provider(&self, name: &str) -> Option<&dyn LLMProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    pub fn get_host(&self, name: &str) -> Option<&dyn LLMProvider> {
        self.hosts.get(name).map(|h| h.as_ref())
    }

    pub fn get_provider_or_host(&self, name: &str) -> Option<&dyn LLMProvider> {
        self.get_provider(name).or_else(|| self.get_host(name))
    }

    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub fn list_hosts(&self) -> Vec<String> {
        self.hosts.keys().cloned().collect()
    }

    pub fn list_all(&self) -> Vec<String> {
        let mut all = self.list_providers();
        all.extend(self.list_hosts());
        all
    }

    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();

        // Check providers
        for (name, provider) in &self.providers {
            match provider.health_check().await {
                Ok(healthy) => {
                    results.insert(name.clone(), healthy);
                }
                Err(_) => {
                    results.insert(name.clone(), false);
                }
            }
        }

        // Check hosts
        for (name, host) in &self.hosts {
            match host.health_check().await {
                Ok(healthy) => {
                    results.insert(name.clone(), healthy);
                }
                Err(_) => {
                    results.insert(name.clone(), false);
                }
            }
        }

        results
    }

    /// Cost-aware chat method that includes budget checking and usage tracking
    pub async fn chat_with_cost_tracking(
        &mut self,
        provider_name: &str,
        request: &ChatRequest,
    ) -> Result<ChatResponse> {
        // Get the provider
        let provider = self
            .get_provider_or_host(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", provider_name))?;

        // Get model pricing info
        let pricing = provider.get_pricing();
        let pricing_rates = pricing.map(|p| (p.input_cost_per_1m, p.output_cost_per_1m));

        // Estimate token usage (rough estimation)
        let estimated_input_tokens = request
            .messages
            .iter()
            .map(|m| m.content.len() / 4) // Rough token estimation
            .sum::<usize>() as u32;
        let estimated_output_tokens = request.max_tokens.unwrap_or(1000);

        // Create cost estimate
        let estimate = self.cost_tracker.estimate_cost(
            provider_name,
            &request.model,
            estimated_input_tokens,
            estimated_output_tokens,
            pricing_rates,
        );

        // Check if request is allowed
        match self.cost_tracker.check_request_allowed(&estimate) {
            CostDecision::Allow => {
                // Proceed with request
            }
            CostDecision::RequireConfirmation(reason) => {
                // In a real UI, this would prompt the user
                eprintln!("⚠️  {}", reason);
                eprintln!("Proceeding with request...");
            }
            CostDecision::Deny(reason) => {
                return Err(anyhow::anyhow!("Request denied: {}", reason));
            }
            CostDecision::SuggestAlternative(suggestion) => {
                eprintln!("💡 {}", suggestion);
                eprintln!("Proceeding with request...");
            }
        }

        // Make the actual request
        let response = provider.chat(request).await?;

        // Record actual usage
        if let Some(actual_cost) = response.cost {
            let input_tokens = response.tokens_used.saturating_sub(estimated_output_tokens);
            let output_tokens = response.tokens_used.saturating_sub(input_tokens);

            if let Err(e) = self.cost_tracker.record_usage(
                provider_name,
                &request.model,
                actual_cost,
                input_tokens,
                output_tokens,
            ) {
                eprintln!("Warning: Failed to record usage: {}", e);
            }
        }

        Ok(response)
    }

    /// Get current cost summary
    pub fn get_cost_summary(&self) -> String {
        format!(
            "{}\n{}",
            self.cost_tracker.get_daily_summary(None),
            self.cost_tracker.get_monthly_summary()
        )
    }

    /// Get provider-specific usage
    pub fn get_provider_usage(&self, provider: &str) -> String {
        self.cost_tracker.get_daily_summary(Some(provider))
    }

    /// Intelligently select and chat with the best model for the task
    pub async fn smart_chat(
        &mut self,
        message: &str,
        task_type: Option<TaskType>,
        max_cost: Option<f64>,
        stream: bool,
    ) -> Result<ChatResponse> {
        // Get model recommendation
        let recommendation = self.model_selector.select_model(
            task_type,
            &self.cost_tracker,
            max_cost,
            message,
        )?;

        info!(
            "Smart model selection: {} ({}) - {}",
            recommendation.model, recommendation.provider, recommendation.reasoning
        );

        // Show alternatives if available
        if !recommendation.alternative_models.is_empty() {
            eprintln!("💡 Alternative models available:");
            for alt in &recommendation.alternative_models {
                eprintln!(
                    "   {} ({}) - Save ${:.4} ({})",
                    alt.model, alt.provider, alt.cost_savings, alt.reason
                );
            }
        }

        // Create chat request
        let messages = vec![Message::user(message.to_string())];
        let mut request = ChatRequest::new(messages, recommendation.model.clone());
        request.stream = stream;

        // Use the cost-aware chat method
        self.chat_with_cost_tracking(&recommendation.provider, &request)
            .await
    }

    /// Get model recommendation without executing chat
    pub fn recommend_model(
        &self,
        message: &str,
        task_type: Option<TaskType>,
        max_cost: Option<f64>,
    ) -> Result<crate::cost::ModelRecommendation> {
        self.model_selector.select_model(
            task_type,
            &self.cost_tracker,
            max_cost,
            message,
        )
    }
}

impl Message {
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            content,
            timestamp: chrono::Utc::now(),
            tokens_used: None,
            cost: None,
        }
    }

    pub fn system(content: String) -> Self {
        Self::new(MessageRole::System, content)
    }

    pub fn user(content: String) -> Self {
        Self::new(MessageRole::User, content)
    }

    pub fn assistant(content: String) -> Self {
        Self::new(MessageRole::Assistant, content)
    }

}

impl ChatRequest {
    pub fn new(messages: Vec<Message>, model: String) -> Self {
        Self {
            messages,
            model,
            max_tokens: None,
            temperature: None,
            stream: false,
            tools: None,
        }
    }

    pub fn simple(content: String, model: String) -> Self {
        let messages = vec![Message::user(content)];
        Self::new(messages, model)
    }

    pub fn with_streaming(mut self) -> Self {
        self.stream = true;
        self
    }

    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }
}
