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
    PricingModel, StreamChunk, ToolCall, UsageInfo,
};
use crate::config::ProviderConfig;
use crate::auth::AuthManager;
use std::sync::Arc;

pub struct OllamaProvider {
    client: Client,
    _config: ProviderConfig,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OllamaToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaToolCall {
    function: OllamaFunction,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaFunction {
    name: String,
    arguments: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OllamaResponse {
    model: String,
    created_at: String,
    message: OllamaMessage,
    done: bool,
    done_reason: String,  // CRITICAL FIX: Missing field causing deserialization failure
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
#[allow(dead_code)]
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
    #[serde(skip_deserializing)]
    _model: Option<String>,
    #[serde(rename = "modified_at")]
    _modified_at: Option<String>,
    #[serde(rename = "size")]
    _size: Option<u64>,
    #[serde(rename = "digest")]
    _digest: Option<String>,
    #[serde(skip_deserializing)]
    _details: Option<serde_json::Value>,
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
    _error: String,
}

// Unused type alias
// type OllamaResponseStream = mpsc::Receiver<Result<StreamChunk>>;

impl OllamaProvider {
    pub async fn new(config: ProviderConfig, _auth_manager: Arc<AuthManager>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120)) // Allow time for large models like gpt-oss (13GB)
            .build()
            .context("Failed to create HTTP client")?;

        let base_url = if config.base_url.is_empty() {
            debug!("Ollama config.base_url is empty, discovering URL...");
            Self::discover_ollama_url(&client, &config.fallback_urls).await
                .unwrap_or_else(|| {
                    debug!("Ollama discovery failed, falling back to localhost");
                    "http://localhost:11434".to_string()
                })
        } else {
            debug!("Using configured Ollama base_url: {}", config.base_url);
            config.base_url.clone()
        };

        info!("Using Ollama at: {}", base_url);

        let provider = Self {
            client,
            _config: config,
            base_url,
            available_models: Vec::new(),  // Lazy load models when needed
        };

        // PERFORMANCE FIX: Don't refresh models during init - it's expensive!
        // Models will be loaded on first access if needed

        Ok(provider)
    }
    
    /// Create a stub provider for UI availability when Ollama is not running
    pub fn new_stub(config: crate::config::ProviderConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .context("Failed to create HTTP client")?;
            
        // Use default localhost URL since discovery failed
        let base_url = "http://localhost:11434".to_string();
        
        Ok(Self {
            client,
            _config: config,
            base_url,
            available_models: Vec::new(), // Empty models - will be populated if/when Ollama becomes available
        })
    }

    async fn discover_ollama_url(client: &Client, fallback_urls: &[String]) -> Option<String> {
        let mut candidate_urls = Vec::new();
        
        // 1. First try configured fallback URLs
        debug!("Ollama discovery: {} configured fallback URLs", fallback_urls.len());
        candidate_urls.extend(fallback_urls.iter().cloned());
        
        // 2. Then try auto-discovered URLs
        let discovered_urls = Self::get_candidate_urls().await;
        debug!("Ollama discovery: {} auto-discovered URLs", discovered_urls.len());
        candidate_urls.extend(discovered_urls);
        
        debug!("Ollama discovery: Total {} candidate URLs to try", candidate_urls.len());
        for url in candidate_urls {
            debug!("Trying Ollama at: {}", url);
            
            let version_url = format!("{}/api/version", url);
            if let Ok(response) = client.get(&version_url).send().await {
                if response.status().is_success() {
                    if let Ok(version_info) = response.json::<OllamaVersionResponse>().await {
                        info!("Found Ollama {} at: {}", version_info.version, url);
                        return Some(url);
                    }
                }
            }
        }
        
        None
    }

    async fn get_candidate_urls() -> Vec<String> {
        let mut urls = Vec::new();
        
        // 1. Check OLLAMA_HOST environment variable first (highest priority)
        if let Ok(ollama_host) = std::env::var("OLLAMA_HOST") {
            let host_url = if ollama_host.starts_with("http") {
                ollama_host
            } else {
                format!("http://{}:11434", ollama_host)
            };
            debug!("Found OLLAMA_HOST env var: {}", host_url);
            urls.push(host_url);
        }
        
        // 2. Always try localhost
        urls.push("http://localhost:11434".to_string());
        urls.push("http://127.0.0.1:11434".to_string());
        
        // 3. Try common Tailscale patterns
        if let Some(tailscale_urls) = Self::get_tailscale_candidates().await {
            urls.extend(tailscale_urls);
        }
        
        // 4. Try Docker default
        urls.push("http://host.docker.internal:11434".to_string());
        
        // 5. Try common local IPs
        urls.push("http://192.168.1.100:11434".to_string());
        urls.push("http://192.168.0.100:11434".to_string());
        urls.push("http://10.0.0.100:11434".to_string());
        
        urls
    }

    async fn get_tailscale_candidates() -> Option<Vec<String>> {
        // Try to detect Tailscale IPs
        let mut candidates = Vec::new();
        
        // Check for tailscale command
        if let Ok(output) = tokio::process::Command::new("tailscale")
            .args(["ip", "-4"])
            .output()
            .await
        {
            if output.status.success() {
                if let Ok(ip_output) = String::from_utf8(output.stdout) {
                    for line in ip_output.lines() {
                        let ip = line.trim();
                        if ip.starts_with("100.") {
                            // This is likely our Tailscale IP, but we want to find other machines
                            // Try common Tailscale IP patterns around our IP
                            if let Some(base_ip) = Self::get_tailscale_network_base(ip) {
                                for i in 1..=254 {
                                    if i % 50 == 0 || i <= 10 || i >= 240 {
                                        // Sample some IPs to avoid too many requests
                                        candidates.push(format!("http://{}.{}:11434", base_ip, i));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Also try some common Tailscale IP patterns
        candidates.push("http://100.64.0.1:11434".to_string());
        candidates.push("http://100.100.100.100:11434".to_string());
        candidates.push("http://100.101.101.101:11434".to_string());
        
        if candidates.is_empty() {
            None
        } else {
            Some(candidates)
        }
    }

    fn get_tailscale_network_base(ip: &str) -> Option<String> {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() == 4 && parts[0] == "100" {
            Some(format!("{}.{}.{}", parts[0], parts[1], parts[2]))
        } else {
            None
        }
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

        let all_models: Vec<String> = models_response
            .models
            .into_iter()
            .map(|model| model.name)
            .collect();
            
        debug!("Ollama raw models from API: {:?}", all_models);
        
        let models: Vec<String> = all_models
            .into_iter()
            .filter(|name| {
                let is_specialized = is_specialized_model(name);
                if is_specialized {
                    debug!("Filtering out specialized model: {}", name);
                }
                !is_specialized
            })
            .collect();
            
        debug!("Ollama filtered models: {:?}", models);

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
                tool_calls: None, // Input messages don't have tool calls
                thinking: None,   // No thinking for input messages
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
                                    
                                    // Build content, appending tool calls on the final chunk so
                                    // upstream agent parsers can detect them in streaming mode.
                                    let mut content = stream_response.message.content;
                                    if stream_response.done {
                                        if let Some(tool_calls) = &stream_response.message.tool_calls {
                                            // Avoid duplicating tool blocks if model already included them
                                            let already_contains = content.contains("<tool_use>");
                                            // Append tool calls using the XML-style wrapper that our parser supports
                                            if !already_contains {
                                                for tc in tool_calls {
                                                    let name = &tc.function.name;
                                                    let args = &tc.function.arguments;
                                                    let params_json = match serde_json::to_string(args) {
                                                        Ok(s) => s,
                                                        Err(_) => String::from("{}"),
                                                    };
                                                    content.push_str("\n<tool_use>\n");
                                                    content.push_str(&format!("<tool>{}</tool><params>{}</params>", name, params_json));
                                                    content.push_str("\n</tool_use>\n");
                                                }
                                            }
                                        }
                                    }

                                    let chunk = StreamChunk {
                                        id: Uuid::new_v4().to_string(),
                                        content,
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
            tools: request.tools.as_ref().map(|tools| {
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

        // Convert Ollama tool calls to our standard format
        let tool_calls = if let Some(ollama_tool_calls) = &ollama_response.message.tool_calls {
            Some(
                ollama_tool_calls
                    .iter()
                    .enumerate()
                    .map(|(i, tool_call)| ToolCall {
                        id: format!("call_{}", i),
                        name: tool_call.function.name.clone(),
                        arguments: tool_call.function.arguments.clone(),
                    })
                    .collect(),
            )
        } else {
            None
        };

        Ok(ChatResponse {
            id: Uuid::new_v4().to_string(),
            content: ollama_response.message.content,
            role: MessageRole::Assistant,
            model: ollama_response.model,
            tokens_used: ollama_response.prompt_eval_count.unwrap_or(0) + ollama_response.eval_count.unwrap_or(0),
            cost: Some(0.0), // Free for local models
            finish_reason: FinishReason::Stop,
            tool_calls,
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
            tools: request.tools.as_ref().map(|tools| {
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
        true // Modern Ollama models support function calling (gpt-oss, qwen2.5-coder, etc.)
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

    async fn list_available_models(&self) -> Result<Vec<String>> {
        // Ollama already has dynamic model fetching - use it
        match self.get_available_models().await {
            Ok(model_names) => {
                debug!("Ollama available models: {:?}", model_names);
                Ok(model_names)
            }
            Err(e) => {
                debug!("Failed to fetch Ollama models: {}", e);
                // Fallback to configured models
                Ok(self.get_models().to_vec())
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Check if a model name indicates it's a specialized model (embedding, vision, etc.)
fn is_specialized_model(model_name: &str) -> bool {
    let name_lower = model_name.to_lowercase();
    
    // Embedding model patterns
    let is_embedding = name_lower.contains("embed") ||
        name_lower.contains("embedding") ||
        name_lower.contains("sentence") ||
        name_lower.contains("bge-") ||
        name_lower.contains("e5-") ||
        name_lower.contains("instructor") ||
        name_lower.contains("all-minilm") ||
        name_lower.contains("all-mpnet") ||
        name_lower.contains("msmarco") ||
        name_lower.contains("paraphrase") ||
        name_lower.contains("retrieval") ||
        name_lower.contains("vector") ||
        name_lower.starts_with("nomic-embed") ||
        name_lower.starts_with("mxbai-embed");
    
    // Vision model patterns
    let is_vision = name_lower.contains("vision") ||
        name_lower.contains("visual") ||
        name_lower.contains("llava") ||
        name_lower.contains("bakllava") ||
        name_lower.contains("moondream") ||
        name_lower.contains("cogvlm") ||
        name_lower.contains("minicpm-v") ||
        name_lower.contains("internvl") ||
        name_lower.contains("bunny") ||
        name_lower.contains("ava-") ||
        name_lower.contains("-vision") ||
        name_lower.ends_with("-v") ||
        name_lower.contains("clip") ||
        name_lower.contains("blip");
    
    is_embedding || is_vision
}
