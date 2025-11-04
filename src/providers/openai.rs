use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error};
use uuid::Uuid;

use super::{
    ChatRequest, ChatResponse, FinishReason, LLMProvider, MessageRole, PricingInfo,
    PricingModel, ResponseStream, StreamChunk, ToolCall, UsageInfo,
};
use crate::config::ProviderConfig;
use crate::auth::AuthManager;
use std::sync::Arc;

pub struct OpenAIProvider {
    client: Client,
    config: ProviderConfig,
    api_key: String,
}

impl std::fmt::Debug for OpenAIProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAIProvider")
            .field("config", &self.config)
            .field("api_key", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    #[serde(deserialize_with = "deserialize_content")]
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_content: Option<String>,
    // vLLM-specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    refusal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    annotations: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audio: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<serde_json::Value>,
}

fn deserialize_content<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIFunction,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
    // vLLM-specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_logprobs: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_token_ids: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kv_transfer_params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: Option<String>,
    // vLLM-specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_ids: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logprobs: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
    // vLLM-specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_tokens_details: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamEvent {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    index: u32,
    delta: OpenAIStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamDelta {
    role: Option<String>,
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIError {
    error: OpenAIErrorDetails,
}

#[derive(Debug, Deserialize)]
struct OpenAIErrorDetails {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
    code: Option<String>,
}

impl OpenAIProvider {
    pub fn new(config: ProviderConfig, auth_manager: Arc<AuthManager>) -> Result<Self> {
        // Try to get API key from auth manager first, fall back to environment variable
        let api_key = futures::executor::block_on(auth_manager.get_api_key("openai"))
            .or_else(|_| env::var("OPENAI_API_KEY"))
            .context("No API key found for OpenAI provider (checked auth storage and OPENAI_API_KEY)")?;

        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            config,
            api_key,
        })
    }

    fn create_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .context("Failed to create authorization header")?,
        );
        Ok(headers)
    }

    fn get_base_url(&self) -> &str {
        if self.config.base_url.is_empty() {
            "https://api.openai.com/v1"
        } else {
            &self.config.base_url
        }
    }

    fn convert_request(&self, req: &ChatRequest) -> OpenAIRequest {
        let mut messages = Vec::new();
        
        for msg in &req.messages {
            let role = match msg.role {
                MessageRole::System => "system",
                MessageRole::User => "user", 
                MessageRole::Assistant => "assistant",
                MessageRole::Tool => "tool",
            };
            
            messages.push(OpenAIMessage {
                role: role.to_string(),
                content: msg.content.clone(),
                tool_calls: None, // Tool calls are only in responses, not requests
                reasoning_content: None,
                refusal: None,
                annotations: None,
                audio: None,
                function_call: None,
            });
        }

        OpenAIRequest {
            model: req.model.clone(),
            messages,
            max_tokens: req.max_tokens,
            temperature: req.temperature,
            stream: req.stream,
            tools: req.tools.as_ref().map(|tools| {
                tools.iter().map(|tool| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": tool.name,
                            "description": tool.description,
                            "parameters": tool.parameters,
                        }
                    })
                }).collect()
            }),
        }
    }

    fn convert_response(&self, resp: OpenAIResponse) -> Result<ChatResponse> {
        let choice = resp.choices.first()
            .ok_or_else(|| anyhow!("No choices in OpenAI response"))?;

        let finish_reason = match choice.finish_reason.as_deref() {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("content_filter") => FinishReason::ContentFilter,
            Some("tool_calls") => FinishReason::ToolCalls,
            _ => FinishReason::Error,
        };


        // Parse tool calls if present
        let tool_calls = choice.message.tool_calls.as_ref().map(|calls| {
            calls.iter().map(|call| {
                ToolCall {
                    id: call.id.clone(),
                    name: call.function.name.clone(),
                    arguments: serde_json::from_str(&call.function.arguments)
                        .unwrap_or_else(|_| serde_json::Value::Null),
                }
            }).collect()
        });

        // Prefer reasoning_content if content is empty (vLLM compatibility)
        let content = if choice.message.content.is_empty() {
            choice.message.reasoning_content.clone().unwrap_or_default()
        } else {
            choice.message.content.clone()
        };

        Ok(ChatResponse {
            id: resp.id,
            content,
            role: MessageRole::Assistant,
            model: resp.model,
            tokens_used: resp.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0),
            cost: resp.usage.as_ref().and_then(|u| {
                self.calculate_cost(u.prompt_tokens, u.completion_tokens)
            }),
            finish_reason,
            tool_calls,
        })
    }


}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "OpenAI"
    }

    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse> {
        let openai_req = self.convert_request(req);
        let headers = self.create_headers()?;

        debug!("Sending OpenAI API request for model: {}", req.model);

        let url = format!("{}/chat/completions", self.get_base_url());
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&openai_req)
            .send()
            .await
            .context("Failed to send request to OpenAI API")?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            match serde_json::from_str::<OpenAIError>(&response_text) {
                Ok(error_resp) => {
                    return Err(anyhow!(
                        "OpenAI API error: {} ({})",
                        error_resp.error.message,
                        status
                    ));
                }
                Err(_) => {
                    return Err(anyhow!(
                        "OpenAI API error: {} - {}",
                        status,
                        response_text
                    ));
                }
            }
        }

        let openai_response: OpenAIResponse = serde_json::from_str(&response_text)
            .context("Failed to parse OpenAI response")?;

        self.convert_response(openai_response)
    }

    async fn stream(&self, req: &ChatRequest) -> Result<ResponseStream> {
        let mut openai_req = self.convert_request(req);
        openai_req.stream = true;

        let headers = self.create_headers()?;

        debug!("Starting OpenAI streaming for model: {}", req.model);

        let url = format!("{}/chat/completions", self.get_base_url());
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&openai_req)
            .send()
            .await
            .context("Failed to send streaming request to OpenAI API")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI streaming failed: {}", error_text));
        }

        let (tx, rx) = mpsc::channel(1000); // Buffer up to 1000 chunks
        let chunk_id = Uuid::new_v4().to_string();

        tokio::spawn(async move {
            use futures::StreamExt;
            
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if let Ok(text) = String::from_utf8(chunk.to_vec()) {
                            buffer.push_str(&text);
                            
                            // Process complete lines
                            while let Some(line_end) = buffer.find('\n') {
                                let line = buffer[..line_end].trim().to_string();
                                buffer = buffer[line_end + 1..].to_string();
                                
                                if line.starts_with("data: ") {
                                    let data = &line[6..]; // Remove "data: " prefix
                                    if data == "[DONE]" {
                                        let _ = tx.send(Ok(StreamChunk {
                                            id: chunk_id.clone(),
                                            content: String::new(),
                                            delta: false,
                                            tokens_used: None,
                                            finish_reason: Some(FinishReason::Stop),
                                        })).await;
                                        return;
                                    }

                                    if let Ok(event) = serde_json::from_str::<OpenAIStreamEvent>(data) {
                                        if let Some(choice) = event.choices.first() {
                                            if let Some(content) = &choice.delta.content {
                                                if tx.send(Ok(StreamChunk {
                                                    id: chunk_id.clone(),
                                                    content: content.clone(),
                                                    delta: true,
                                                    tokens_used: None,
                                                    finish_reason: None,
                                                })).await.is_err() {
                                                    return;
                                                }
                                            }
                                            
                                            if let Some(reason) = &choice.finish_reason {
                                                let finish_reason = match reason.as_str() {
                                                    "stop" => FinishReason::Stop,
                                                    "length" => FinishReason::Length,
                                                    "content_filter" => FinishReason::ContentFilter,
                                                    "tool_calls" => FinishReason::ToolCalls,
                                                    _ => FinishReason::Error,
                                                };
                                                let _ = tx.send(Ok(StreamChunk {
                                                    id: chunk_id.clone(),
                                                    content: String::new(),
                                                    delta: false,
                                                    tokens_used: None,
                                                    finish_reason: Some(finish_reason),
                                                })).await;
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading OpenAI stream: {}", e);
                        let _ = tx.send(Err(anyhow!("Stream error: {}", e))).await;
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    fn supports_tools(&self) -> bool {
        true // OpenAI supports function calling
    }

    fn supports_vision(&self) -> bool {
        // Only certain models support vision
        // For now, we'll check if the first model supports vision
        if let Some(model) = self.config.models.first() {
            matches!(
                model.name.as_str(),
                "gpt-4o" | "gpt-4o-mini" | "gpt-4-turbo" | "gpt-4-vision-preview"
            )
        } else {
            false
        }
    }

    fn context_window(&self) -> u32 {
        // Return the context window of the first model, or default
        self.config.models.first()
            .map(|m| m.context_window)
            .unwrap_or(4_096)
    }

    fn pricing_model(&self) -> PricingModel {
        if let Some(model) = self.config.models.first() {
            PricingModel::PerToken {
                input_cost_per_1m: model.input_cost_per_1m,
                output_cost_per_1m: model.output_cost_per_1m,
                currency: "USD".to_string(),
            }
        } else {
            PricingModel::PerToken {
                input_cost_per_1m: 1.0,
                output_cost_per_1m: 3.0,
                currency: "USD".to_string(),
            }
        }
    }

    fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> Option<f64> {
        if let Some(model) = self.config.models.first() {
            let input_cost = (input_tokens as f64 / 1_000_000.0) * model.input_cost_per_1m;
            let output_cost = (output_tokens as f64 / 1_000_000.0) * model.output_cost_per_1m;
            Some(input_cost + output_cost)
        } else {
            None
        }
    }

    fn get_pricing(&self) -> Option<PricingInfo> {
        if let Some(model) = self.config.models.first() {
            Some(PricingInfo {
                input_cost_per_1m: model.input_cost_per_1m,
                output_cost_per_1m: model.output_cost_per_1m,
                currency: "USD".to_string(),
            })
        } else {
            None
        }
    }

    async fn get_usage_info(&self) -> Result<Option<UsageInfo>> {
        // OpenAI doesn't provide a direct usage API in their standard tier
        // This would require implementing billing API access for organizations
        Ok(None)
    }

    fn usage_warning_threshold(&self) -> Option<f64> {
        // No usage warnings for OpenAI since we can't track usage easily
        None
    }

    async fn health_check(&self) -> Result<bool> {
        // Simple health check by making a minimal API call
        let headers = self.create_headers()?;
        
        let test_request = OpenAIRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: "test".to_string(),
                tool_calls: None,
                reasoning_content: None,
                refusal: None,
                annotations: None,
                audio: None,
                function_call: None,
            }],
            max_tokens: Some(1),
            temperature: Some(0.0),
            stream: false,
            tools: None,
        };

        let url = format!("{}/chat/completions", self.get_base_url());
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&test_request)
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn list_available_models(&self) -> Result<Vec<String>> {
        let headers = self.create_headers()?;
        
        debug!("Fetching available models from OpenAI API");

        let url = format!("{}/models", self.get_base_url());
        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to fetch models from OpenAI API")?;

        if !response.status().is_success() {
            debug!("OpenAI models API failed, using configured models");
            return Ok(self.config.models.iter().map(|m| m.name.clone()).collect());
        }

        let models_response: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse OpenAI models response")?;

        let mut models = Vec::new();
        if let Some(data) = models_response.get("data").and_then(|d| d.as_array()) {
            for model in data {
                if let Some(id) = model.get("id").and_then(|id| id.as_str()) {
                    // Filter to only chat models (exclude embeddings, whisper, etc.)
                    if id.starts_with("gpt-") || id.starts_with("o1-") || id.starts_with("o3-") {
                        models.push(id.to_string());
                    }
                }
            }
        }

        // Sort models by preference (newer models first)
        models.sort_by(|a, b| {
            let a_priority = get_model_priority(a);
            let b_priority = get_model_priority(b);
            a_priority.cmp(&b_priority)
        });

        debug!("OpenAI available models: {:?}", models);
        
        if models.is_empty() {
            // Fallback to configured models if API doesn't return any
            Ok(self.config.models.iter().map(|m| m.name.clone()).collect())
        } else {
            Ok(models)
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Helper function to prioritize models
fn get_model_priority(model: &str) -> i32 {
    match model {
        // Latest models first
        m if m.starts_with("o3") => 1,
        m if m.starts_with("o1") => 2,
        m if m.starts_with("gpt-4o") => 3,
        m if m.starts_with("gpt-4-turbo") => 4,
        m if m.starts_with("gpt-4") => 5,
        m if m.starts_with("gpt-3.5") => 6,
        _ => 99, // Everything else at the end
    }
}