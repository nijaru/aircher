# LLM Providers Architecture with Sophisticated Fallback Chains

## Overview

Based on Warp's successful achievement of 71% on SWE-bench Verified and 52% on Terminal-Bench, sophisticated model fallback chains are critical for reliable AI coding performance. This specification outlines our LLM provider architecture with Warp-inspired fallback strategies and retry mechanisms.

## Core Architecture Principles

### 1. **Warp-Validated Fallback Chain**
Warp's most effective chain that achieved 71% SWE-bench success:
1. **Primary**: Anthropic Claude 4 Sonnet (execution and implementation)
2. **Secondary**: Anthropic Claude 3.7 Sonnet (proven reliability fallback)  
3. **Tertiary**: Google Gemini 2.5 Pro (alternative architecture)
4. **Emergency**: OpenAI GPT-4.1 (final fallback)

### 2. **Retry Strategy**
- **Failed tool calls**: Retry with different model (not same model)
- **Invalid responses**: Move to next model in chain immediately
- **Rate limits**: Circuit breaker with exponential backoff
- **Provider outages**: Automatic failover with health monitoring

### 3. **Context Preservation**
- **Same conversation context**: Maintain conversation state across model switches
- **Tool call history**: Preserve previous attempts for learning
- **Error context**: Pass failure reasons to next model in chain

## Provider Interface Design

### Universal LLM Provider Trait

```rust
use async_trait::async_trait;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait LLMProvider: Send + Sync + std::fmt::Debug {
    // Core chat functionality
    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse, ProviderError>;
    async fn stream_chat(&self, req: &ChatRequest) -> Result<impl Stream<Item = ChatChunk>, ProviderError>;
    
    // Capability detection
    fn supports_functions(&self) -> bool;
    fn supports_system_messages(&self) -> bool;
    fn supports_images(&self) -> bool;
    fn supports_thinking(&self) -> bool;
    
    // Token and cost management
    fn get_token_limit(&self, model: &str) -> Option<usize>;
    async fn count_tokens(&self, content: &str) -> Result<usize, ProviderError>;
    fn calculate_cost(&self, input_tokens: usize, output_tokens: usize, model: &str) -> Result<f64, ProviderError>;
    
    // Provider metadata
    fn name(&self) -> &str;
    fn models(&self) -> Vec<ModelInfo>;
    fn authentication_methods(&self) -> Vec<AuthMethod>;
    
    // Health and reliability
    async fn health_check(&self) -> Result<HealthStatus, ProviderError>;
    fn get_rate_limit_info(&self) -> Option<RateLimitInfo>;
    fn reliability_tier(&self) -> ReliabilityTier;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_length: usize,
    pub cost_per_input_token: f64,
    pub cost_per_output_token: f64,
    pub capabilities: ModelCapabilities,
    pub reliability_tier: ReliabilityTier,
    pub fallback_priority: u8,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReliabilityTier {
    Primary,   // claude-4-sonnet - primary execution model
    Fallback,  // claude-3.7-sonnet, gemini-2.5-pro - proven reliability
    Emergency, // gpt-4.1, deepseek-r1 - last resort
}

#[derive(Debug, Clone)]
pub struct ModelCapabilities {
    pub max_tokens: usize,
    pub supports_tools: bool,
    pub supports_images: bool,
    pub supports_thinking: bool,
    pub supports_streaming: bool,
    pub code_execution: bool,
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub latency_ms: u64,
    pub error_rate: f64,
    pub last_check: std::time::SystemTime,
    pub status_message: String,
}

#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub requests_per_minute: u32,
    pub tokens_per_minute: u32,
    pub requests_remaining: Option<u32>,
    pub reset_time: Option<std::time::SystemTime>,
}
```

### Sophisticated Fallback Chain Manager

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct FallbackChainManager {
    providers: HashMap<String, Arc<dyn LLMProvider>>,
    fallback_chains: HashMap<String, FallbackChain>,
    health_monitor: Arc<HealthMonitor>,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    retry_config: RetryConfig,
}

impl FallbackChainManager {
    pub fn new_with_warp_strategy() -> Self {
        let mut chains = HashMap::new();
        
        // Warp's proven fallback chain
        chains.insert("coding".to_string(), FallbackChain {
            name: "coding".to_string(),
            models: vec![
                FallbackModel {
                    provider: "anthropic".to_string(),
                    model: "claude-4-sonnet".to_string(),
                    max_retries: 2,
                    timeout: Duration::from_secs(30),
                    conditions: vec![FailureCondition::ToolCallFailure, FailureCondition::InvalidResponse],
                },
                FallbackModel {
                    provider: "anthropic".to_string(), 
                    model: "claude-3.7-sonnet".to_string(),
                    max_retries: 2,
                    timeout: Duration::from_secs(45),
                    conditions: vec![FailureCondition::ToolCallFailure, FailureCondition::RateLimit],
                },
                FallbackModel {
                    provider: "google".to_string(),
                    model: "gemini-2.5-pro".to_string(), 
                    max_retries: 1,
                    timeout: Duration::from_secs(60),
                    conditions: vec![FailureCondition::ProviderOutage],
                },
                FallbackModel {
                    provider: "openai".to_string(),
                    model: "gpt-4.1".to_string(),
                    max_retries: 1,
                    timeout: Duration::from_secs(90),
                    conditions: vec![FailureCondition::AllOthersFailed],
                },
            ],
        });
        
        Self {
            providers: HashMap::new(),
            fallback_chains: chains,
            health_monitor: Arc::new(HealthMonitor::new()),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            retry_config: RetryConfig::default(),
        }
    }
    
    pub async fn execute_with_fallback(&self, request: ChatRequest, chain_name: &str) -> Result<ChatResponse, FallbackError> {
        let chain = self.fallback_chains.get(chain_name)
            .ok_or_else(|| FallbackError::ChainNotFound(chain_name.to_string()))?;
        
        let mut last_error = None;
        let mut attempt_history = Vec::new();
        
        for (index, fallback_model) in chain.models.iter().enumerate() {
            // Check circuit breaker
            if self.is_circuit_breaker_open(&fallback_model.provider).await {
                continue;
            }
            
            let provider = match self.providers.get(&fallback_model.provider) {
                Some(p) => p,
                None => {
                    last_error = Some(FallbackError::ProviderNotFound(fallback_model.provider.clone()));
                    continue;
                }
            };
            
            // Execute request with retries
            match self.execute_with_retries(provider.clone(), &request, fallback_model).await {
                Ok(response) => {
                    // Success! Log the attempt and return
                    attempt_history.push(AttemptResult {
                        provider: fallback_model.provider.clone(),
                        model: fallback_model.model.clone(),
                        attempt_index: index,
                        success: true,
                        error: None,
                        latency: response.latency,
                    });
                    
                    self.record_success(&fallback_model.provider).await;
                    
                    return Ok(ChatResponse {
                        content: response.content,
                        model_used: fallback_model.model.clone(),
                        provider_used: fallback_model.provider.clone(),
                        fallback_chain_used: Some(chain_name.to_string()),
                        attempt_history,
                        tokens_used: response.tokens_used,
                        cost: response.cost,
                        latency: response.latency,
                    });
                }
                Err(error) => {
                    // Log failed attempt
                    attempt_history.push(AttemptResult {
                        provider: fallback_model.provider.clone(),
                        model: fallback_model.model.clone(),
                        attempt_index: index,
                        success: false,
                        error: Some(error.to_string()),
                        latency: Duration::from_millis(0),
                    });
                    
                    self.record_failure(&fallback_model.provider, &error).await;
                    last_error = Some(FallbackError::ProviderError(error));
                    
                    // Check if we should continue to next provider based on error type
                    if !self.should_continue_chain(&error, fallback_model) {
                        break;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or(FallbackError::AllProvidersFailed))
    }
    
    async fn execute_with_retries(&self, provider: Arc<dyn LLMProvider>, 
                                  request: &ChatRequest, 
                                  fallback_model: &FallbackModel) -> Result<ChatResponse, ProviderError> {
        let mut last_error = None;
        
        for attempt in 0..fallback_model.max_retries {
            match tokio::time::timeout(fallback_model.timeout, provider.chat(request)).await {
                Ok(Ok(response)) => return Ok(response),
                Ok(Err(error)) => {
                    last_error = Some(error.clone());
                    
                    // Warp's key insight: Don't retry same model for tool call failures
                    if matches!(error, ProviderError::ToolCallFailure(_)) {
                        break;
                    }
                    
                    // Exponential backoff for retries
                    if attempt < fallback_model.max_retries - 1 {
                        let delay = Duration::from_millis(100 * 2_u64.pow(attempt));
                        tokio::time::sleep(delay).await;
                    }
                }
                Err(_) => {
                    last_error = Some(ProviderError::Timeout(fallback_model.timeout));
                    break;
                }
            }
        }
        
        Err(last_error.unwrap_or(ProviderError::MaxRetriesExceeded))
    }
    
    fn should_continue_chain(&self, error: &ProviderError, model: &FallbackModel) -> bool {
        match error {
            ProviderError::ToolCallFailure(_) => model.conditions.contains(&FailureCondition::ToolCallFailure),
            ProviderError::InvalidResponse(_) => model.conditions.contains(&FailureCondition::InvalidResponse),
            ProviderError::RateLimit(_) => model.conditions.contains(&FailureCondition::RateLimit),
            ProviderError::ProviderOutage => model.conditions.contains(&FailureCondition::ProviderOutage),
            _ => true, // Continue for other errors
        }
    }
}

#[derive(Debug, Clone)]
pub struct FallbackChain {
    pub name: String,
    pub models: Vec<FallbackModel>,
}

#[derive(Debug, Clone)]
pub struct FallbackModel {
    pub provider: String,
    pub model: String,
    pub max_retries: u32,
    pub timeout: Duration,
    pub conditions: Vec<FailureCondition>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FailureCondition {
    ToolCallFailure,
    InvalidResponse,
    RateLimit,
    ProviderOutage,
    AllOthersFailed,
}

#[derive(Debug, Clone)]
pub struct AttemptResult {
    pub provider: String,
    pub model: String,
    pub attempt_index: usize,
    pub success: bool,
    pub error: Option<String>,
    pub latency: Duration,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub model_used: String,
    pub provider_used: String,
    pub fallback_chain_used: Option<String>,
    pub attempt_history: Vec<AttemptResult>,
    pub tokens_used: TokenUsage,
    pub cost: f64,
    pub latency: Duration,
}
```

### Circuit Breaker Implementation

```rust
use std::time::{Duration, Instant};

pub struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,    // Normal operation
    Open,      // Failing, reject requests
    HalfOpen,  // Testing if service recovered
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_threshold,
            success_threshold: 3, // Need 3 successes to close
            timeout,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
        }
    }
    
    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        self.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }
    
    pub fn record_success(&mut self) {
        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count = 0;
            }
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.state = CircuitBreakerState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitBreakerState::Open => {} // Should not happen
        }
    }
    
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());
        
        if self.failure_count >= self.failure_threshold {
            self.state = CircuitBreakerState::Open;
        }
    }
}
```

### Provider Implementations

```rust
// Anthropic Claude Provider (Primary)
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    rate_limiter: Arc<RateLimiter>,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://api.anthropic.com".to_string(),
            rate_limiter: Arc::new(RateLimiter::new(5000, Duration::from_secs(60))), // 5000 requests/minute
        }
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse, ProviderError> {
        // Wait for rate limit
        self.rate_limiter.wait().await;
        
        let start_time = Instant::now();
        
        // Prepare request
        let request_body = serde_json::json!({
            "model": req.model,
            "messages": req.messages,
            "max_tokens": req.max_tokens.unwrap_or(4096),
            "temperature": req.temperature.unwrap_or(0.7),
            "tools": req.tools,
        });
        
        // Make API call
        let response = self.client
            .post(&format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;
        
        let latency = start_time.elapsed();
        
        // Handle response
        if response.status().is_success() {
            let response_body: serde_json::Value = response.json().await
                .map_err(|e| ProviderError::ParseError(e.to_string()))?;
            
            // Parse response and extract content
            let content = self.extract_content(&response_body)?;
            let tokens_used = self.extract_token_usage(&response_body)?;
            
            Ok(ChatResponse {
                content,
                model_used: req.model.clone(),
                provider_used: "anthropic".to_string(),
                fallback_chain_used: None,
                attempt_history: vec![],
                tokens_used,
                cost: self.calculate_cost(tokens_used.input, tokens_used.output, &req.model)?,
                latency,
            })
        } else {
            let error_body: serde_json::Value = response.json().await
                .map_err(|e| ProviderError::ParseError(e.to_string()))?;
            
            Err(self.parse_error(&error_body, response.status().as_u16()))
        }
    }
    
    fn reliability_tier(&self) -> ReliabilityTier {
        ReliabilityTier::Primary // Claude is our primary provider
    }
    
    fn models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "claude-4-sonnet".to_string(),
                name: "Claude 4 Sonnet".to_string(),
                context_length: 200000,
                cost_per_input_token: 0.000003,
                cost_per_output_token: 0.000015,
                capabilities: ModelCapabilities {
                    max_tokens: 4096,
                    supports_tools: true,
                    supports_images: true,
                    supports_thinking: true,
                    supports_streaming: true,
                    code_execution: false,
                },
                reliability_tier: ReliabilityTier::Primary,
                fallback_priority: 1,
                max_retries: 2,
            },
            ModelInfo {
                id: "claude-3.7-sonnet".to_string(),
                name: "Claude 3.7 Sonnet".to_string(),
                context_length: 200000,
                cost_per_input_token: 0.000003,
                cost_per_output_token: 0.000015,
                capabilities: ModelCapabilities {
                    max_tokens: 4096,
                    supports_tools: true,
                    supports_images: true,
                    supports_thinking: false,
                    supports_streaming: true,
                    code_execution: false,
                },
                reliability_tier: ReliabilityTier::Fallback,
                fallback_priority: 2,
                max_retries: 2,
            },
        ]
    }
}

// Google Gemini Provider (Tertiary)
pub struct GeminiProvider {
    client: reqwest::Client,
    api_key: String,
    rate_limiter: Arc<RateLimiter>,
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    fn reliability_tier(&self) -> ReliabilityTier {
        ReliabilityTier::Fallback
    }
    
    fn models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "gemini-2.5-pro".to_string(),
                name: "Gemini 2.5 Pro".to_string(),
                context_length: 1000000,
                cost_per_input_token: 0.00000125,
                cost_per_output_token: 0.000005,
                capabilities: ModelCapabilities {
                    max_tokens: 8192,
                    supports_tools: true,
                    supports_images: true,
                    supports_thinking: false,
                    supports_streaming: true,
                    code_execution: true,
                },
                reliability_tier: ReliabilityTier::Fallback,
                fallback_priority: 3,
                max_retries: 1,
            },
        ]
    }
    
    // Implementation details...
}

// OpenAI Provider (Emergency)
pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    rate_limiter: Arc<RateLimiter>,
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    fn reliability_tier(&self) -> ReliabilityTier {
        ReliabilityTier::Emergency
    }
    
    fn models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "gpt-4.1".to_string(),
                name: "GPT-4.1".to_string(),
                context_length: 128000,
                cost_per_input_token: 0.00001,
                cost_per_output_token: 0.00003,
                capabilities: ModelCapabilities {
                    max_tokens: 4096,
                    supports_tools: true,
                    supports_images: true,
                    supports_thinking: false,
                    supports_streaming: true,
                    code_execution: false,
                },
                reliability_tier: ReliabilityTier::Emergency,
                fallback_priority: 4,
                max_retries: 1,
            },
        ]
    }
    
    // Implementation details...
}
```

## Configuration

```toml
[providers]
default_chain = "coding"

[providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"
base_url = "https://api.anthropic.com"
timeout = "30s"
max_retries = 2

[providers.google]
api_key_env = "GOOGLE_API_KEY"
timeout = "60s"
max_retries = 1

[providers.openai]
api_key_env = "OPENAI_API_KEY"
timeout = "90s"
max_retries = 1

[fallback_chains.coding]
name = "coding"
models = [
    { provider = "anthropic", model = "claude-4-sonnet", max_retries = 2, timeout = "30s" },
    { provider = "anthropic", model = "claude-3.7-sonnet", max_retries = 2, timeout = "45s" },
    { provider = "google", model = "gemini-2.5-pro", max_retries = 1, timeout = "60s" },
    { provider = "openai", model = "gpt-4.1", max_retries = 1, timeout = "90s" },
]

[circuit_breaker]
failure_threshold = 5
timeout = "5m"
success_threshold = 3

[rate_limiting]
requests_per_minute = 5000
tokens_per_minute = 1000000
```

This sophisticated provider system with Warp-validated fallback chains ensures maximum reliability while maintaining the performance characteristics that enabled Warp's benchmark success.