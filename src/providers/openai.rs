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
    PricingModel, ResponseStream, StreamChunk, UsageInfo,
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
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    _object: String,
    _created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    _index: u32,
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamEvent {
    _id: String,
    _object: String,
    _created: u64,
    _model: String,
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    _index: u32,
    delta: OpenAIStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamDelta {
    _role: Option<String>,
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
    _error_type: Option<String>,
    _code: Option<String>,
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
            });
        }

        OpenAIRequest {
            model: req.model.clone(),
            messages,
            max_tokens: req.max_tokens,
            temperature: req.temperature,
            stream: req.stream,
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


        Ok(ChatResponse {
            id: resp.id,
            content: choice.message.content.clone(),
            role: MessageRole::Assistant,
            model: resp.model,
            tokens_used: resp.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0),
            cost: resp.usage.as_ref().and_then(|u| {
                self.calculate_cost(u.prompt_tokens, u.completion_tokens)
            }),
            finish_reason,
            tool_calls: None, // TODO: Implement tool calls when needed
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

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
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

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
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
            }],
            max_tokens: Some(1),
            temperature: Some(0.0),
            stream: false,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .headers(headers)
            .json(&test_request)
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}