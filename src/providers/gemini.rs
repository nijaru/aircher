use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
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

pub struct GeminiProvider {
    client: Client,
    config: ProviderConfig,
    api_key: String,
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(default)]
    usage_metadata: Option<GeminiUsage>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
    finish_reason: Option<String>,
    _safety_ratings: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
struct GeminiUsage {
    prompt_token_count: Option<u32>,
    candidates_token_count: Option<u32>,
    total_token_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct GeminiError {
    error: GeminiErrorDetails,
}

#[derive(Debug, Deserialize)]
struct GeminiErrorDetails {
    _code: u32,
    message: String,
    _status: String,
}

impl GeminiProvider {
    pub async fn new(config: ProviderConfig, auth_manager: Arc<AuthManager>) -> Result<Self> {
        // Try to get API key from auth manager first, fall back to environment variable
        let api_key = auth_manager.get_api_key("gemini")
            .await
            .or_else(|_| env::var(&config.api_key_env))
            .with_context(|| format!("No API key found for Gemini provider (checked auth storage and {})", config.api_key_env))?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            config,
            api_key,
        })
    }

    fn convert_messages(
        &self,
        messages: &[Message],
    ) -> (Option<GeminiContent>, Vec<GeminiContent>) {
        let mut system_instruction = None;
        let mut gemini_contents = Vec::new();

        for message in messages {
            match message.role {
                MessageRole::System => {
                    system_instruction = Some(GeminiContent {
                        role: "user".to_string(), // Gemini uses user role for system instructions
                        parts: vec![GeminiPart {
                            text: message.content.clone(),
                        }],
                    });
                }
                MessageRole::User => {
                    gemini_contents.push(GeminiContent {
                        role: "user".to_string(),
                        parts: vec![GeminiPart {
                            text: message.content.clone(),
                        }],
                    });
                }
                MessageRole::Assistant => {
                    gemini_contents.push(GeminiContent {
                        role: "model".to_string(), // Gemini uses "model" for assistant
                        parts: vec![GeminiPart {
                            text: message.content.clone(),
                        }],
                    });
                }
                MessageRole::Tool => {
                    // For now, treat tool messages as user messages
                    gemini_contents.push(GeminiContent {
                        role: "user".to_string(),
                        parts: vec![GeminiPart {
                            text: format!("Tool response: {}", message.content),
                        }],
                    });
                }
            }
        }

        (system_instruction, gemini_contents)
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

    fn map_finish_reason(&self, finish_reason: Option<&str>) -> FinishReason {
        match finish_reason {
            Some("STOP") => FinishReason::Stop,
            Some("MAX_TOKENS") => FinishReason::Length,
            Some("SAFETY") => FinishReason::ContentFilter,
            Some("RECITATION") => FinishReason::ContentFilter,
            Some("OTHER") => FinishReason::Error,
            _ => FinishReason::Stop,
        }
    }
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    fn name(&self) -> &str {
        "Gemini"
    }

    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse> {
        debug!("Making Gemini API request for model: {}", req.model);

        let (system_instruction, contents) = self.convert_messages(&req.messages);

        let gemini_req = GeminiRequest {
            contents,
            system_instruction,
            generation_config: GenerationConfig {
                temperature: req.temperature,
                max_output_tokens: req.max_tokens,
            },
        };

        // Gemini API URLs include the model and action
        let url = format!(
            "{}:generateContent?key={}",
            self.config.base_url.replace("{model}", &req.model),
            self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&gemini_req)
            .send()
            .await
            .context("Failed to send request to Gemini API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            // Try to parse as Gemini error
            if let Ok(gemini_error) = serde_json::from_str::<GeminiError>(&error_text) {
                return Err(anyhow!("Gemini API error: {}", gemini_error.error.message));
            }

            return Err(anyhow!(
                "Gemini API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .context("Failed to parse Gemini API response")?;

        // Extract text content from first candidate
        if gemini_response.candidates.is_empty() {
            return Err(anyhow!("No candidates in Gemini response"));
        }

        let candidate = &gemini_response.candidates[0];
        let content = candidate
            .content
            .parts
            .iter()
            .map(|part| part.text.clone())
            .collect::<Vec<_>>()
            .join("");

        let input_tokens = gemini_response
            .usage_metadata
            .as_ref()
            .and_then(|u| u.prompt_token_count)
            .unwrap_or(0);
        let output_tokens = gemini_response
            .usage_metadata
            .as_ref()
            .and_then(|u| u.candidates_token_count)
            .unwrap_or(0);
        let total_tokens = input_tokens + output_tokens;

        let cost = self.calculate_cost_for_model(&req.model, input_tokens, output_tokens);

        Ok(ChatResponse {
            id: Uuid::new_v4().to_string(),
            content,
            role: MessageRole::Assistant,
            model: req.model.clone(),
            tokens_used: total_tokens,
            cost,
            finish_reason: self.map_finish_reason(candidate.finish_reason.as_deref()),
            tool_calls: None, // TODO: Implement tool calls
        })
    }

    async fn stream(&self, req: &ChatRequest) -> Result<ResponseStream> {
        debug!(
            "Making streaming Gemini API request for model: {}",
            req.model
        );

        let (system_instruction, contents) = self.convert_messages(&req.messages);

        let gemini_req = GeminiRequest {
            contents,
            system_instruction,
            generation_config: GenerationConfig {
                temperature: req.temperature,
                max_output_tokens: req.max_tokens,
            },
        };

        // Gemini streaming API uses streamGenerateContent
        let url = format!(
            "{}:streamGenerateContent?key={}",
            self.config.base_url.replace("{model}", &req.model),
            self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&gemini_req)
            .send()
            .await
            .context("Failed to send streaming request to Gemini API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Gemini streaming request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let (tx, rx) = mpsc::channel(32);
        let mut stream = response.bytes_stream();

        tokio::spawn(async move {
            use futures::StreamExt;

            let mut accumulated_content = String::new();
            let response_id = Uuid::new_v4().to_string();
            let mut buffer = Vec::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        buffer.extend_from_slice(&chunk);

                        // Try to parse complete JSON objects separated by newlines
                        if let Ok(content) = String::from_utf8(buffer.clone()) {
                            for line in content.lines() {
                                if line.trim().is_empty() {
                                    continue;
                                }

                                match serde_json::from_str::<GeminiResponse>(line) {
                                    Ok(response) => {
                                        if let Some(candidate) = response.candidates.get(0) {
                                            let text = candidate
                                                .content
                                                .parts
                                                .iter()
                                                .map(|part| part.text.clone())
                                                .collect::<Vec<_>>()
                                                .join("");

                                            if !text.is_empty() {
                                                accumulated_content.push_str(&text);

                                                let chunk = StreamChunk {
                                                    id: response_id.clone(),
                                                    content: text,
                                                    delta: true,
                                                    tokens_used: None,
                                                    finish_reason: None,
                                                };

                                                if tx.send(Ok(chunk)).await.is_err() {
                                                    return;
                                                }
                                            }

                                            // Check for finish reason
                                            if let Some(finish_reason) = &candidate.finish_reason {
                                                let final_chunk = StreamChunk {
                                                    id: response_id.clone(),
                                                    content: accumulated_content.clone(),
                                                    delta: false,
                                                    tokens_used: response
                                                        .usage_metadata
                                                        .and_then(|u| u.total_token_count),
                                                    finish_reason: Some(
                                                        match finish_reason.as_str() {
                                                            "STOP" => FinishReason::Stop,
                                                            "MAX_TOKENS" => FinishReason::Length,
                                                            "SAFETY" => FinishReason::ContentFilter,
                                                            "RECITATION" => {
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

                            // Keep the last incomplete line in buffer
                            if let Some(last_newline) = content.rfind('\n') {
                                let remaining = &content[last_newline + 1..];
                                buffer = remaining.as_bytes().to_vec();
                            }
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
        true // Gemini supports function calling
    }

    fn supports_vision(&self) -> bool {
        true // Gemini supports vision
    }

    fn context_window(&self) -> u32 {
        1_000_000 // Gemini 2.0 Flash context window
    }

    fn pricing_model(&self) -> PricingModel {
        PricingModel::PerToken {
            input_cost_per_1m: 0.075, // Gemini 2.0 Flash pricing
            output_cost_per_1m: 0.30,
            currency: "USD".to_string(),
        }
    }

    fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> Option<f64> {
        let input_cost = (input_tokens as f64 / 1_000_000.0) * 0.075;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * 0.30;
        Some(input_cost + output_cost)
    }

    fn get_pricing(&self) -> Option<PricingInfo> {
        Some(PricingInfo {
            input_cost_per_1m: 0.075,
            output_cost_per_1m: 0.30,
            currency: "USD".to_string(),
        })
    }

    async fn get_usage_info(&self) -> Result<Option<UsageInfo>> {
        // Gemini API doesn't provide usage info for API keys
        Ok(None)
    }

    fn usage_warning_threshold(&self) -> Option<f64> {
        None // No usage limits for API access
    }

    async fn health_check(&self) -> Result<bool> {
        // Make a minimal request to check if the API is accessible
        let test_contents = vec![GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart {
                text: "Hello".to_string(),
            }],
        }];

        let test_req = GeminiRequest {
            contents: test_contents,
            system_instruction: None,
            generation_config: GenerationConfig {
                temperature: None,
                max_output_tokens: Some(10),
            },
        };

        let url = format!(
            "{}:generateContent?key={}",
            self.config
                .base_url
                .replace("{model}", "gemini-2.0-flash-exp"),
            self.api_key
        );

        match self.client.post(&url).json(&test_req).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    async fn list_available_models(&self) -> Result<Vec<String>> {
        // For now, return configured models - could implement Gemini models API later
        let models = self.config.models.iter()
            .map(|m| m.name.clone())
            .collect();
        
        debug!("Gemini available models: {:?}", models);
        Ok(models)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
