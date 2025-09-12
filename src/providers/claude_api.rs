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
    ChatRequest, ChatResponse, FinishReason, LLMProvider, Message, MessageRole, PricingInfo,
    PricingModel, ResponseStream, StreamChunk, UsageInfo,
};
use crate::config::ProviderConfig;
use crate::auth::AuthManager;
use std::sync::Arc;

pub struct ClaudeApiProvider {
    client: Client,
    config: ProviderConfig,
    _api_key: String,
}

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    id: String,
    #[serde(rename = "type")]
    _response_type: String,
    _role: String,
    content: Vec<ClaudeContent>,
    model: String,
    stop_reason: Option<String>,
    _stop_sequence: Option<String>,
    usage: ClaudeUsage,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ClaudeStreamEvent {
    #[serde(rename = "type")]
    _event_type: String,
    #[serde(default)]
    _message: Option<serde_json::Value>,
    #[serde(default)]
    _content_block: Option<serde_json::Value>,
    #[serde(default)]
    delta: Option<ClaudeStreamDelta>,
    #[serde(default)]
    _usage: Option<ClaudeUsage>,
}

#[derive(Debug, Deserialize)]
struct ClaudeStreamDelta {
    #[serde(rename = "type")]
    _delta_type: String,
    text: Option<String>,
    _stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeError {
    #[serde(rename = "type")]
    _error_type: String,
    message: String,
}

impl ClaudeApiProvider {
    pub async fn new(config: ProviderConfig, auth_manager: Arc<AuthManager>) -> Result<Self> {
        // Try to get API key from auth manager first, fall back to environment variable
        let api_key = auth_manager.get_api_key("claude")
            .await
            .or_else(|_| env::var(&config.api_key_env))
            .with_context(|| format!("No API key found for Claude provider (checked auth storage and {})", config.api_key_env))?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))
                .context("Invalid API key format")?,
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            config,
            _api_key: api_key,
        })
    }

    fn convert_messages(&self, messages: &[Message]) -> (Option<String>, Vec<ClaudeMessage>) {
        let mut system_message = None;
        let mut claude_messages = Vec::new();

        for message in messages {
            match message.role {
                MessageRole::System => {
                    system_message = Some(message.content.clone());
                }
                MessageRole::User => {
                    claude_messages.push(ClaudeMessage {
                        role: "user".to_string(),
                        content: message.content.clone(),
                    });
                }
                MessageRole::Assistant => {
                    claude_messages.push(ClaudeMessage {
                        role: "assistant".to_string(),
                        content: message.content.clone(),
                    });
                }
                MessageRole::Tool => {
                    // For now, treat tool messages as user messages
                    // TODO: Implement proper tool support
                    claude_messages.push(ClaudeMessage {
                        role: "user".to_string(),
                        content: format!("Tool response: {}", message.content),
                    });
                }
            }
        }

        (system_message, claude_messages)
    }

    fn calculate_cost_for_model(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Option<f64> {
        let model_config = self.config.models.iter().find(|m| m.name == model)?;

        let input_cost = (input_tokens as f64 / 1_000_000.0) * model_config.input_cost_per_1m;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * model_config.output_cost_per_1m;

        Some(input_cost + output_cost)
    }

    fn map_finish_reason(&self, stop_reason: Option<&str>) -> FinishReason {
        match stop_reason {
            Some("end_turn") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::Length,
            Some("tool_use") => FinishReason::ToolCalls,
            Some("stop_sequence") => FinishReason::Stop,
            _ => FinishReason::Stop,
        }
    }
}

#[async_trait]
impl LLMProvider for ClaudeApiProvider {
    fn name(&self) -> &str {
        "Claude API"
    }

    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse> {
        debug!("Making Claude API request for model: {}", req.model);

        let (system, messages) = self.convert_messages(&req.messages);

        let claude_req = ClaudeRequest {
            model: req.model.clone(),
            max_tokens: req.max_tokens.unwrap_or(4096),
            messages,
            system,
            temperature: req.temperature,
            stream: false,
            tools: req.tools.as_ref().map(|tools| {
                tools.iter().map(|tool| {
                    serde_json::json!({
                        "name": tool.name,
                        "description": tool.description,
                        "input_schema": tool.parameters,
                    })
                }).collect()
            }),
        };

        let url = format!("{}/messages", self.config.base_url);

        let response = self
            .client
            .post(&url)
            .json(&claude_req)
            .send()
            .await
            .context("Failed to send request to Claude API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            // Try to parse as Claude error
            if let Ok(claude_error) = serde_json::from_str::<ClaudeError>(&error_text) {
                return Err(anyhow!("Claude API error: {}", claude_error.message));
            }

            return Err(anyhow!(
                "Claude API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let claude_response: ClaudeResponse = response
            .json()
            .await
            .context("Failed to parse Claude API response")?;

        // Extract text content and tool calls
        let mut content = String::new();
        let mut tool_calls = Vec::new();
        
        for c in &claude_response.content {
            match c.content_type.as_str() {
                "text" => {
                    if let Some(text) = &c.text {
                        content.push_str(text);
                    }
                }
                "tool_use" => {
                    if let (Some(id), Some(name), Some(input)) = (&c.id, &c.name, &c.input) {
                        tool_calls.push(super::ToolCall {
                            id: id.clone(),
                            name: name.clone(),
                            arguments: input.clone(),
                        });
                    }
                }
                _ => {} // Ignore unknown content types
            }
        }

        let total_tokens = claude_response.usage.input_tokens + claude_response.usage.output_tokens;
        let cost = self.calculate_cost_for_model(
            &req.model,
            claude_response.usage.input_tokens,
            claude_response.usage.output_tokens,
        );

        Ok(ChatResponse {
            id: claude_response.id,
            content,
            role: MessageRole::Assistant,
            model: claude_response.model,
            tokens_used: total_tokens,
            cost,
            finish_reason: self.map_finish_reason(claude_response.stop_reason.as_deref()),
            tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
        })
    }

    async fn stream(&self, req: &ChatRequest) -> Result<ResponseStream> {
        debug!(
            "Making streaming Claude API request for model: {}",
            req.model
        );

        let (system, messages) = self.convert_messages(&req.messages);

        let claude_req = ClaudeRequest {
            model: req.model.clone(),
            max_tokens: req.max_tokens.unwrap_or(4096),
            messages,
            system,
            temperature: req.temperature,
            stream: true,
            tools: req.tools.as_ref().map(|tools| {
                tools.iter().map(|tool| {
                    serde_json::json!({
                        "name": tool.name,
                        "description": tool.description,
                        "input_schema": tool.parameters,
                    })
                }).collect()
            }),
        };

        let url = format!("{}/messages", self.config.base_url);

        let response = self
            .client
            .post(&url)
            .json(&claude_req)
            .send()
            .await
            .context("Failed to send streaming request to Claude API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Claude streaming request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let (tx, rx) = mpsc::channel(32);
        let stream = response.bytes_stream();

        tokio::spawn(async move {
            use eventsource_stream::Eventsource;
            use futures::StreamExt;

            let mut event_stream = stream.eventsource();
            let mut accumulated_content = String::new();
            let response_id = Uuid::new_v4().to_string();

            while let Some(event) = event_stream.next().await {
                match event {
                    Ok(event) => {
                        if event.event == "message_start" || event.event == "content_block_start" {
                            continue;
                        }

                        if event.event == "content_block_delta" {
                            if let Ok(stream_event) =
                                serde_json::from_str::<ClaudeStreamEvent>(&event.data)
                            {
                                if let Some(delta) = stream_event.delta {
                                    if let Some(text) = delta.text {
                                        accumulated_content.push_str(&text);

                                        let chunk = StreamChunk {
                                            id: response_id.clone(),
                                            content: text,
                                            delta: true,
                                            tokens_used: None,
                                            finish_reason: None,
                                        };

                                        if tx.send(Ok(chunk)).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        if event.event == "message_stop" {
                            let chunk = StreamChunk {
                                id: response_id.clone(),
                                content: accumulated_content.clone(),
                                delta: false,
                                tokens_used: None, // TODO: Extract from final event
                                finish_reason: Some(FinishReason::Stop),
                            };

                            let _ = tx.send(Ok(chunk)).await;
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Stream error: {}", e);
                        let _ = tx.send(Err(anyhow!("Stream error: {}", e))).await;
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    fn supports_tools(&self) -> bool {
        true // Claude supports tools, but not implemented yet
    }

    fn supports_vision(&self) -> bool {
        true // Claude supports vision, but not implemented yet
    }

    fn context_window(&self) -> u32 {
        200_000 // Claude 3.5 Sonnet context window
    }

    fn pricing_model(&self) -> PricingModel {
        PricingModel::PerToken {
            input_cost_per_1m: 3.0, // Claude 3.5 Sonnet pricing
            output_cost_per_1m: 15.0,
            currency: "USD".to_string(),
        }
    }

    fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> Option<f64> {
        let input_cost = (input_tokens as f64 / 1_000_000.0) * 3.0;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * 15.0;
        Some(input_cost + output_cost)
    }

    fn get_pricing(&self) -> Option<PricingInfo> {
        Some(PricingInfo {
            input_cost_per_1m: 3.0,
            output_cost_per_1m: 15.0,
            currency: "USD".to_string(),
        })
    }

    async fn get_usage_info(&self) -> Result<Option<UsageInfo>> {
        // Claude API doesn't provide usage info for API keys
        // This would be implemented for Claude Pro/Max subscription
        Ok(None)
    }

    fn usage_warning_threshold(&self) -> Option<f64> {
        None // No usage limits for API access
    }

    async fn health_check(&self) -> Result<bool> {
        // Make a minimal request to check if the API is accessible
        let test_messages = vec![ClaudeMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];

        let test_req = ClaudeRequest {
            model: "claude-3-5-haiku-20241022".to_string(), // Use cheapest model for health check
            max_tokens: 10,
            messages: test_messages,
            system: None,
            temperature: None,
            stream: false,
            tools: None, // No tools needed for health check
        };

        let url = format!("{}/messages", self.config.base_url);

        match self.client.post(&url).json(&test_req).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    async fn list_available_models(&self) -> Result<Vec<String>> {
        // Anthropic doesn't have a public models endpoint yet, so we use our configured models
        // but could fallback to hardcoded current models
        let models = self.config.models.iter()
            .map(|m| m.name.clone())
            .collect();
        
        debug!("Anthropic available models: {:?}", models);
        Ok(models)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
