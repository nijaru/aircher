use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info};
use uuid::Uuid;

use super::{
    ChatRequest, ChatResponse, FinishReason, LLMProvider, Message, MessageRole, PricingInfo,
    PricingModel, StreamChunk, UsageInfo,
};
use crate::config::ProviderConfig;

pub struct OllamaProvider {
    client: Client,
    config: ProviderConfig,
    base_url: String,
    available_models: Vec<String>,
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    model: String,
    created_at: String,
    message: OllamaMessage,
    done: bool,
    #[serde(default)]
    total_duration: Option<u64>,
    #[serde(default)]
    load_duration: Option<u64>,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    prompt_eval_duration: Option<u64>,
    #[serde(default)]
    eval_count: Option<u32>,
    #[serde(default)]
    eval_duration: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OllamaStreamResponse {
    model: String,
    created_at: String,
    message: OllamaMessage,
    done: bool,
    #[serde(default)]
    total_duration: Option<u64>,
    #[serde(default)]
    load_duration: Option<u64>,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    prompt_eval_duration: Option<u64>,
    #[serde(default)]
    eval_count: Option<u32>,
    #[serde(default)]
    eval_duration: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelInfo {
    name: String,
    modified_at: String,
    size: u64,
    digest: String,
}

#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OllamaVersionResponse {
    version: String,
}

#[derive(Debug, Deserialize)]
struct OllamaError {
    error: String,
}

type OllamaResponseStream = mpsc::Receiver<Result<StreamChunk>>;

impl OllamaProvider {
    pub async fn new(config: ProviderConfig) -> Result<Self> {
        let base_url = if config.base_url.is_empty() {
            "http://localhost:11434".to_string()
        } else {
            config.base_url.clone()
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        let mut provider = Self {
            client,
            config,
            base_url,
            available_models: Vec::new(),
        };

        // Initialize available models
        provider.refresh_models().await?;

        Ok(provider)
    }

    async fn refresh_models(&mut self) -> Result<()> {
        match self.get_available_models().await {
            Ok(models) => {
                self.available_models = models;
                info!("Discovered {} Ollama models", self.available_models.len());
            }
            Err(e) => {
                error!("Failed to refresh Ollama models: {}", e);
                // Continue with empty models list - models can be added later
            }
        }
        Ok(())
    }

    async fn get_available_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch available models")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to get models: HTTP {}",
                response.status()
            ));
        }

        let models_response: OllamaModelsResponse = response
            .json()
            .await
            .context("Failed to parse models response")?;

        let models = models_response
            .models
            .into_iter()
            .map(|model| model.name)
            .collect();

        Ok(models)
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/version", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(version_response) = response.json::<OllamaVersionResponse>().await {
                        debug!("Ollama version: {}", version_response.version);
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Err(e) => {
                debug!("Ollama health check failed: {}", e);
                Ok(false)
            }
        }
    }

    pub fn get_models(&self) -> &[String] {
        &self.available_models
    }

    pub async fn pull_model(&self, model: &str) -> Result<()> {
        let url = format!("{}/api/pull", self.base_url);
        
        let request = serde_json::json!({
            "name": model
        });

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to pull model")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to pull model {}: HTTP {}",
                model,
                response.status()
            ));
        }

        info!("Successfully pulled model: {}", model);
        Ok(())
    }

    pub fn convert_messages(&self, messages: &[Message]) -> Vec<OllamaMessage> {
        messages
            .iter()
            .map(|msg| OllamaMessage {
                role: match msg.role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::System => "system".to_string(),
                    MessageRole::Tool => "tool".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect()
    }

    async fn handle_streaming_response(&self, response: reqwest::Response) -> Result<mpsc::Receiver<Result<StreamChunk>>> {
        let (tx, rx) = mpsc::channel(100);
        let mut stream = response.bytes_stream();
        
        tokio::spawn(async move {
            use futures::StreamExt;
            
            let mut buffer = String::new();
            let mut input_tokens = 0u32;
            let mut output_tokens = 0u32;
            
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        let chunk_str = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&chunk_str);
                        
                        // Process complete JSON lines
                        while let Some(newline_pos) = buffer.find('\n') {
                            let line = buffer[..newline_pos].trim().to_string();
                            buffer = buffer[newline_pos + 1..].to_string();
                            
                            if line.is_empty() {
                                continue;
                            }
                            
                            match serde_json::from_str::<OllamaStreamResponse>(&line) {
                                Ok(stream_response) => {
                                    // Update token counts from response
                                    if let Some(count) = stream_response.prompt_eval_count {
                                        input_tokens = count;
                                    }
                                    if let Some(count) = stream_response.eval_count {
                                        output_tokens = count;
                                    }
                                    
                                    let chunk = StreamChunk {
                                        id: Uuid::new_v4().to_string(),
                                        content: stream_response.message.content,
                                        delta: !stream_response.done,
                                        tokens_used: if stream_response.done {
                                            Some(input_tokens + output_tokens)
                                        } else {
                                            None
                                        },
                                        finish_reason: if stream_response.done {
                                            Some(FinishReason::Stop)
                                        } else {
                                            None
                                        },
                                    };
                                    
                                    if tx.send(Ok(chunk)).await.is_err() {
                                        break;
                                    }
                                    
                                    if stream_response.done {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to parse streaming response: {}", e);
                                    if tx.send(Err(anyhow!("Failed to parse streaming response: {}", e))).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Stream error: {}", e);
                        if tx.send(Err(anyhow!("Stream error: {}", e))).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Ok(rx)
    }
}


#[async_trait]
impl LLMProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/api/chat", self.base_url);

        let ollama_request = OllamaRequest {
            model: request.model.clone(),
            messages: self.convert_messages(&request.messages),
            options: if request.temperature.is_some() {
                Some(OllamaOptions {
                    temperature: request.temperature,
                    num_predict: request.max_tokens,
                })
            } else {
                None
            },
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .context("Failed to send chat request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Ollama API error: HTTP {} - {}",
                status,
                error_text
            ));
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .context("Failed to parse response")?;

        Ok(ChatResponse {
            id: Uuid::new_v4().to_string(),
            content: ollama_response.message.content,
            role: MessageRole::Assistant,
            model: ollama_response.model,
            tokens_used: ollama_response.prompt_eval_count.unwrap_or(0) + ollama_response.eval_count.unwrap_or(0),
            cost: Some(0.0), // Free for local models
            finish_reason: FinishReason::Stop,
            tool_calls: None,
        })
    }

    async fn stream(&self, request: &ChatRequest) -> Result<mpsc::Receiver<Result<StreamChunk>>> {
        let url = format!("{}/api/chat", self.base_url);

        let ollama_request = OllamaRequest {
            model: request.model.clone(),
            messages: self.convert_messages(&request.messages),
            options: if request.temperature.is_some() {
                Some(OllamaOptions {
                    temperature: request.temperature,
                    num_predict: request.max_tokens,
                })
            } else {
                None
            },
            stream: true,
        };

        let response = self
            .client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .context("Failed to send streaming request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Ollama API error: HTTP {} - {}",
                status,
                error_text
            ));
        }

        let stream = self.handle_streaming_response(response).await?;
        Ok(stream)
    }

    fn supports_tools(&self) -> bool {
        false // Ollama doesn't support function calling yet
    }

    fn supports_vision(&self) -> bool {
        true // Some Ollama models support vision
    }

    fn context_window(&self) -> u32 {
        // Default context window, varies by model
        4096
    }

    fn pricing_model(&self) -> PricingModel {
        PricingModel::Free
    }

    fn calculate_cost(&self, _input_tokens: u32, _output_tokens: u32) -> Option<f64> {
        Some(0.0) // Local models are free
    }

    fn get_pricing(&self) -> Option<PricingInfo> {
        Some(PricingInfo {
            input_cost_per_1m: 0.0,
            output_cost_per_1m: 0.0,
            currency: "USD".to_string(),
        })
    }

    async fn get_usage_info(&self) -> Result<Option<UsageInfo>> {
        // Local models don't have usage limits
        Ok(None)
    }

    fn usage_warning_threshold(&self) -> Option<f64> {
        None // No usage limits for local models
    }

    async fn health_check(&self) -> Result<bool> {
        self.health_check().await
    }
}