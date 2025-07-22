use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use tracing::{debug, info};

use super::{
    ChatRequest, ChatResponse, FinishReason, LLMProvider, Message, MessageRole, PricingInfo,
    PricingModel, ResponseStream, StreamChunk, UsageInfo,
};
use crate::config::HostConfig;

pub struct OpenRouterHost {
    client: Client,
    config: HostConfig,
    _api_key: String,
    model_cache: HashMap<String, OpenRouterModel>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    id: String,
    choices: Vec<OpenRouterChoice>,
    usage: Option<OpenRouterUsage>,
    model: String,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMessage,
    finish_reason: Option<String>,
    _index: u32,
}

#[derive(Debug, Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterModel {
    id: String,
    _name: String,
    _description: Option<String>,
    context_length: u32,
    pricing: OpenRouterPricing,
    _top_provider: Option<OpenRouterProvider>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterPricing {
    prompt: String,
    completion: String,
}

#[derive(Debug, Deserialize)]
struct OpenRouterProvider {
    _name: String,
    _max_completion_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModel>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterError {
    error: OpenRouterErrorDetails,
}

#[derive(Debug, Deserialize)]
struct OpenRouterErrorDetails {
    message: String,
    #[serde(rename = "type")]
    _error_type: Option<String>,
    _code: Option<u32>,
}

impl OpenRouterHost {
    pub async fn new(config: HostConfig) -> Result<Self> {
        let api_key = env::var(&config.api_key_env)
            .with_context(|| format!("Environment variable {} not found", config.api_key_env))?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {api_key}"))
                .context("Failed to create authorization header")?,
        );

        // Add OpenRouter-specific headers
        headers.insert(
            "HTTP-Referer",
            HeaderValue::from_static("https://github.com/nijaru/aircher"),
        );
        headers.insert(
            "X-Title",
            HeaderValue::from_static("Aircher Terminal Assistant"),
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(120)) // OpenRouter can be slower
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;

        let mut host = Self {
            client,
            config,
            _api_key: api_key,
            model_cache: HashMap::new(),
        };

        // Load available models on initialization
        if let Err(e) = host.refresh_models().await {
            debug!("Failed to load OpenRouter models on initialization: {}", e);
        }

        Ok(host)
    }

    async fn refresh_models(&mut self) -> Result<()> {
        info!("Refreshing OpenRouter model list");

        let url = format!("{}/models", self.config.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch OpenRouter models")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "OpenRouter models request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let models_response: OpenRouterModelsResponse = response
            .json()
            .await
            .context("Failed to parse OpenRouter models response")?;

        self.model_cache.clear();
        for model in models_response.data {
            self.model_cache.insert(model.id.clone(), model);
        }

        info!("Loaded {} OpenRouter models", self.model_cache.len());
        Ok(())
    }

    fn convert_messages(&self, messages: &[Message]) -> Vec<OpenRouterMessage> {
        messages
            .iter()
            .map(|msg| OpenRouterMessage {
                role: match msg.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::Tool => "user".to_string(), // Treat tool as user message
                },
                content: if msg.role == MessageRole::Tool {
                    format!("Tool response: {}", msg.content)
                } else {
                    msg.content.clone()
                },
            })
            .collect()
    }

    fn map_finish_reason(&self, finish_reason: Option<&str>) -> FinishReason {
        match finish_reason {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("content_filter") => FinishReason::ContentFilter,
            Some("tool_calls") => FinishReason::ToolCalls,
            Some("error") => FinishReason::Error,
            _ => FinishReason::Stop,
        }
    }

    fn calculate_cost_for_model(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Option<f64> {
        let model_info = self.model_cache.get(model)?;

        // Parse pricing strings (format: "0.000001" per token)
        let input_cost_per_token: f64 = model_info.pricing.prompt.parse().ok()?;
        let output_cost_per_token: f64 = model_info.pricing.completion.parse().ok()?;

        let input_cost = input_tokens as f64 * input_cost_per_token;
        let output_cost = output_tokens as f64 * output_cost_per_token;

        Some(input_cost + output_cost)
    }

    pub fn get_available_models(&self) -> Vec<String> {
        self.model_cache.keys().cloned().collect()
    }

    pub fn get_model_info(&self, model: &str) -> Option<&OpenRouterModel> {
        self.model_cache.get(model)
    }

    pub async fn get_model_pricing(&self, model: &str) -> Option<PricingInfo> {
        let model_info = self.model_cache.get(model)?;

        let input_cost_per_token: f64 = model_info.pricing.prompt.parse().ok()?;
        let output_cost_per_token: f64 = model_info.pricing.completion.parse().ok()?;

        Some(PricingInfo {
            input_cost_per_1m: input_cost_per_token * 1_000_000.0,
            output_cost_per_1m: output_cost_per_token * 1_000_000.0,
            currency: "USD".to_string(),
        })
    }

    pub async fn compare_costs(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64> {
        self.calculate_cost_for_model(model, input_tokens, output_tokens)
            .ok_or_else(|| anyhow!("Model {} not found in OpenRouter", model))
    }
}

#[async_trait]
impl LLMProvider for OpenRouterHost {
    fn name(&self) -> &str {
        "OpenRouter"
    }

    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse> {
        debug!("Making OpenRouter API request for model: {}", req.model);

        let messages = self.convert_messages(&req.messages);

        let openrouter_req = OpenRouterRequest {
            model: req.model.clone(),
            messages,
            max_tokens: req.max_tokens,
            temperature: req.temperature,
            stream: Some(false),
            provider: None, // Let OpenRouter choose best provider
        };

        let url = format!("{}/chat/completions", self.config.base_url);

        let response = self
            .client
            .post(&url)
            .json(&openrouter_req)
            .send()
            .await
            .context("Failed to send request to OpenRouter API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            // Try to parse as OpenRouter error
            if let Ok(openrouter_error) = serde_json::from_str::<OpenRouterError>(&error_text) {
                return Err(anyhow!(
                    "OpenRouter API error: {}",
                    openrouter_error.error.message
                ));
            }

            return Err(anyhow!(
                "OpenRouter API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let openrouter_response: OpenRouterResponse = response
            .json()
            .await
            .context("Failed to parse OpenRouter API response")?;

        // Extract text content from first choice
        if openrouter_response.choices.is_empty() {
            return Err(anyhow!("No choices in OpenRouter response"));
        }

        let choice = &openrouter_response.choices[0];
        let content = choice.message.content.clone();

        let input_tokens = openrouter_response
            .usage
            .as_ref()
            .map(|u| u.prompt_tokens)
            .unwrap_or(0);
        let output_tokens = openrouter_response
            .usage
            .as_ref()
            .map(|u| u.completion_tokens)
            .unwrap_or(0);
        let total_tokens = input_tokens + output_tokens;

        let cost = self.calculate_cost_for_model(&req.model, input_tokens, output_tokens);

        Ok(ChatResponse {
            id: openrouter_response.id,
            content,
            role: MessageRole::Assistant,
            model: openrouter_response.model,
            tokens_used: total_tokens,
            cost,
            finish_reason: self.map_finish_reason(choice.finish_reason.as_deref()),
            tool_calls: None, // TODO: Implement tool calls
        })
    }

    async fn stream(&self, req: &ChatRequest) -> Result<ResponseStream> {
        debug!(
            "Making streaming OpenRouter API request for model: {}",
            req.model
        );

        let messages = self.convert_messages(&req.messages);

        let openrouter_req = OpenRouterRequest {
            model: req.model.clone(),
            messages,
            max_tokens: req.max_tokens,
            temperature: req.temperature,
            stream: Some(true),
            provider: None, // Let OpenRouter choose best provider
        };

        let url = format!("{}/chat/completions", self.config.base_url);

        let response = self
            .client
            .post(&url)
            .json(&openrouter_req)
            .send()
            .await
            .context("Failed to send streaming request to OpenRouter API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "OpenRouter streaming request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let mut stream = response.bytes_stream();

        tokio::spawn(async move {
            use futures::StreamExt;
            use std::str;

            let mut accumulated_content = String::new();
            let mut response_id = String::new();
            let mut buffer = Vec::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        buffer.extend_from_slice(&chunk);

                        // Try to parse complete SSE events
                        if let Ok(content) = str::from_utf8(&buffer) {
                            let lines: Vec<&str> = content.lines().collect();
                            let mut processed_lines = 0;

                            for line in &lines {
                                if line.trim().is_empty() {
                                    processed_lines += 1;
                                    continue;
                                }

                                if line.starts_with("data: ") {
                                    let data = &line[6..]; // Remove "data: " prefix

                                    if data == "[DONE]" {
                                        // Send final chunk
                                        let final_chunk = StreamChunk {
                                            id: response_id.clone(),
                                            content: accumulated_content.clone(),
                                            delta: false,
                                            tokens_used: None,
                                            finish_reason: Some(FinishReason::Stop),
                                        };
                                        let _ = tx.send(Ok(final_chunk)).await;
                                        return;
                                    }

                                    match serde_json::from_str::<OpenRouterResponse>(data) {
                                        Ok(response) => {
                                            if response_id.is_empty() {
                                                response_id = response.id.clone();
                                            }

                                            if let Some(choice) = response.choices.first() {
                                                let delta_content = choice.message.content.clone();
                                                if !delta_content.is_empty() {
                                                    accumulated_content.push_str(&delta_content);

                                                    let chunk = StreamChunk {
                                                        id: response_id.clone(),
                                                        content: delta_content,
                                                        delta: true,
                                                        tokens_used: None,
                                                        finish_reason: None,
                                                    };

                                                    if tx.send(Ok(chunk)).await.is_err() {
                                                        return;
                                                    }
                                                }

                                                // Check for finish reason
                                                if let Some(finish_reason) = &choice.finish_reason {
                                                    let final_chunk = StreamChunk {
                                                        id: response_id.clone(),
                                                        content: accumulated_content.clone(),
                                                        delta: false,
                                                        tokens_used: response
                                                            .usage
                                                            .map(|u| u.total_tokens),
                                                        finish_reason: Some(
                                                            match finish_reason.as_str() {
                                                                "stop" => FinishReason::Stop,
                                                                "length" => FinishReason::Length,
                                                                "content_filter" => {
                                                                    FinishReason::ContentFilter
                                                                }
                                                                _ => FinishReason::Error,
                                                            },
                                                        ),
                                                    };
                                                    let _ = tx.send(Ok(final_chunk)).await;
                                                    return;
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            // Incomplete JSON, continue buffering
                                            continue;
                                        }
                                    }
                                }
                                processed_lines += 1;
                            }

                            // Keep unprocessed content in buffer
                            if processed_lines < lines.len() {
                                let remaining = lines[processed_lines..].join("\n");
                                buffer = remaining.as_bytes().to_vec();
                            } else {
                                buffer.clear();
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(anyhow!("Stream error: {}", e))).await;
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    fn supports_tools(&self) -> bool {
        true // OpenRouter supports tools for compatible models
    }

    fn supports_vision(&self) -> bool {
        true // OpenRouter supports vision for compatible models
    }

    fn context_window(&self) -> u32 {
        // Return max context from cached models or default
        self.model_cache
            .values()
            .map(|m| m.context_length)
            .max()
            .unwrap_or(128_000)
    }

    fn pricing_model(&self) -> PricingModel {
        PricingModel::PerToken {
            input_cost_per_1m: 0.50,  // Average estimate
            output_cost_per_1m: 1.50, // Average estimate
            currency: "USD".to_string(),
        }
    }

    fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> Option<f64> {
        // Generic calculation - should use specific model pricing
        let input_cost = (input_tokens as f64 / 1_000_000.0) * 0.50;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * 1.50;
        Some(input_cost + output_cost)
    }

    fn get_pricing(&self) -> Option<PricingInfo> {
        Some(PricingInfo {
            input_cost_per_1m: 0.50,  // Average estimate
            output_cost_per_1m: 1.50, // Average estimate
            currency: "USD".to_string(),
        })
    }

    async fn get_usage_info(&self) -> Result<Option<UsageInfo>> {
        // OpenRouter doesn't provide usage info for API keys
        Ok(None)
    }

    fn usage_warning_threshold(&self) -> Option<f64> {
        None // No usage limits for API access
    }

    async fn health_check(&self) -> Result<bool> {
        // Make a minimal request to check if OpenRouter is accessible
        let url = format!("{}/models", self.config.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
