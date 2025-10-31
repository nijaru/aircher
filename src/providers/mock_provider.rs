/// Mock LLM Provider for testing strategies without requiring real API connections
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::collections::VecDeque;
use tokio::sync::mpsc;

use super::{
    ChatRequest, ChatResponse, FinishReason, LLMProvider, MessageRole,
    PricingInfo, PricingModel, ResponseStream, StreamChunk, SubscriptionTier, ToolCall, UsageInfo,
};

/// MockProvider simulates LLM responses for testing
/// Can be configured with pre-programmed responses for deterministic testing
pub struct MockProvider {
    responses: VecDeque<MockResponse>,
    default_response: String,
    model_name: String,
}

#[derive(Debug, Clone)]
pub struct MockResponse {
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub finish_reason: FinishReason,
}

impl MockProvider {
    /// Create a new MockProvider with default responses
    pub fn new() -> Self {
        Self {
            responses: VecDeque::new(),
            default_response: "I understand the task and will proceed step by step.".to_string(),
            model_name: "mock-gpt-4".to_string(),
        }
    }

    /// Create a MockProvider with pre-programmed responses
    pub fn with_responses(responses: Vec<MockResponse>) -> Self {
        Self {
            responses: responses.into(),
            default_response: "Default mock response".to_string(),
            model_name: "mock-gpt-4".to_string(),
        }
    }

    /// Create a strategy-aware MockProvider that responds appropriately to strategy phases
    pub fn for_strategy_testing() -> Self {
        let responses = vec![
            // Think phase response
            MockResponse {
                content: "I need to analyze the current situation and plan my next action.".to_string(),
                tool_calls: Some(vec![ToolCall {
                    id: "call_1".to_string(),
                    name: "reflect".to_string(),
                    arguments: json!({
                        "context": "analyzing task requirements",
                        "iteration": 1
                    }),
                }]),
                finish_reason: FinishReason::ToolCalls,
            },
            // Act phase response
            MockResponse {
                content: "Based on my analysis, I'll search for TODO comments in the codebase.".to_string(),
                tool_calls: Some(vec![ToolCall {
                    id: "call_2".to_string(),
                    name: "search_code".to_string(),
                    arguments: json!({
                        "query": "TODO",
                        "file_pattern": "src/**/*.rs"
                    }),
                }]),
                finish_reason: FinishReason::ToolCalls,
            },
            // Observe phase response
            MockResponse {
                content: "I found several TODO comments. Let me validate the results.".to_string(),
                tool_calls: Some(vec![ToolCall {
                    id: "call_3".to_string(),
                    name: "run_command".to_string(),
                    arguments: json!({
                        "command": "grep -r 'TODO' src/",
                        "timeout_seconds": 10
                    }),
                }]),
                finish_reason: FinishReason::ToolCalls,
            },
            // Final summary response
            MockResponse {
                content: "Task completed successfully. I found multiple TODO comments in the source code and validated the results.".to_string(),
                tool_calls: None,
                finish_reason: FinishReason::Stop,
            },
        ];

        Self::with_responses(responses)
    }

    /// Add a response to the queue
    pub fn add_response(&mut self, response: MockResponse) {
        self.responses.push_back(response);
    }

    /// Get the next response, or return default if queue is empty
    fn get_next_response(&mut self) -> MockResponse {
        self.responses.pop_front().unwrap_or_else(|| MockResponse {
            content: self.default_response.clone(),
            tool_calls: None,
            finish_reason: FinishReason::Stop,
        })
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LLMProvider for MockProvider {
    fn name(&self) -> &str {
        "mock"
    }

    async fn chat(&self, _request: &ChatRequest) -> Result<ChatResponse> {
        // Create a mutable copy to pop responses
        let mut provider = self.clone();
        let mock_response = provider.get_next_response();

        Ok(ChatResponse {
            id: "mock_response_id".to_string(),
            content: mock_response.content,
            role: MessageRole::Assistant,
            model: self.model_name.clone(),
            tokens_used: 150,
            cost: Some(0.0), // Free for testing
            finish_reason: mock_response.finish_reason,
            tool_calls: mock_response.tool_calls,
        })
    }

    async fn stream(&self, request: &ChatRequest) -> Result<ResponseStream> {
        let (tx, rx) = mpsc::channel(10);

        // Get the response that would be returned by chat()
        let response = self.chat(request).await?;

        // Send the content as streaming chunks
        let content = response.content;

        tokio::spawn(async move {
            // Split content into chunks for realistic streaming
            let words: Vec<&str> = content.split_whitespace().collect();

            for (i, word) in words.iter().enumerate() {
                let chunk_content = if i == words.len() - 1 {
                    format!("{}", word)
                } else {
                    format!("{} ", word)
                };

                let chunk = StreamChunk {
                    id: "mock_chunk".to_string(),
                    content: chunk_content,
                    delta: true,
                    tokens_used: if i == words.len() - 1 { Some(150) } else { None },
                    finish_reason: if i == words.len() - 1 {
                        Some(FinishReason::Stop)
                    } else {
                        None
                    },
                };

                if tx.send(Ok(chunk)).await.is_err() {
                    break;
                }

                // Small delay to simulate real streaming
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        });

        Ok(rx)
    }

    fn supports_tools(&self) -> bool {
        true
    }

    fn supports_vision(&self) -> bool {
        false
    }

    fn context_window(&self) -> u32 {
        128000 // Mock large context window
    }

    fn pricing_model(&self) -> PricingModel {
        PricingModel::PerToken {
            input_cost_per_1m: 0.0,
            output_cost_per_1m: 0.0,
            currency: "USD".to_string(),
        }
    }

    fn calculate_cost(&self, _input_tokens: u32, _output_tokens: u32) -> Option<f64> {
        Some(0.0) // Free for testing
    }

    fn get_pricing(&self) -> Option<PricingInfo> {
        Some(PricingInfo {
            input_cost_per_1m: 0.0,
            output_cost_per_1m: 0.0,
            currency: "USD".to_string(),
        })
    }

    async fn get_usage_info(&self) -> Result<Option<UsageInfo>> {
        Ok(Some(UsageInfo {
            current_usage: 0,
            limit: u64::MAX,
            reset_date: chrono::Utc::now() + chrono::Duration::days(30),
            usage_percentage: 0.0,
            tier: SubscriptionTier::Pro,
            approaching_limit: false,
        }))
    }

    fn usage_warning_threshold(&self) -> Option<f64> {
        Some(0.8)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true) // Mock provider is always healthy
    }

    async fn list_available_models(&self) -> Result<Vec<String>> {
        Ok(vec![
            "mock-gpt-4".to_string(),
            "mock-claude-3".to_string(),
            "mock-gemini-pro".to_string(),
        ])
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Make MockProvider cloneable for mutable operations
impl Clone for MockProvider {
    fn clone(&self) -> Self {
        Self {
            responses: self.responses.clone(),
            default_response: self.default_response.clone(),
            model_name: self.model_name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_basic_response() {
        let provider = MockProvider::new();
        let request = ChatRequest {
            messages: vec![Message {
                id: "test_msg_1".to_string(),
                role: MessageRole::User,
                content: "Hello".to_string(),
                timestamp: chrono::Utc::now(),
                tokens_used: None,
                cost: None,
            }],
            model: "mock-gpt-4".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: false,
            tools: None,
        };

        let response = provider.chat(&request).await.unwrap();
        assert_eq!(response.content, "I understand the task and will proceed step by step.");
        assert!(response.tool_calls.is_none());
    }

    #[tokio::test]
    async fn test_mock_provider_with_custom_responses() {
        let responses = vec![
            MockResponse {
                content: "First response".to_string(),
                tool_calls: None,
                finish_reason: FinishReason::Stop,
            },
            MockResponse {
                content: "Second response".to_string(),
                tool_calls: Some(vec![ToolCall {
                    id: "test_call".to_string(),
                    name: "test_tool".to_string(),
                    arguments: json!({"param": "value"}),
                }]),
                finish_reason: FinishReason::ToolCalls,
            },
        ];

        let provider = MockProvider::with_responses(responses);
        let request = ChatRequest {
            messages: vec![Message {
                id: "test_msg_2".to_string(),
                role: MessageRole::User,
                content: "Test".to_string(),
                timestamp: chrono::Utc::now(),
                tokens_used: None,
                cost: None,
            }],
            model: "mock-gpt-4".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: false,
            tools: None,
        };

        let response1 = provider.chat(&request).await.unwrap();
        assert_eq!(response1.content, "First response");

        let response2 = provider.chat(&request).await.unwrap();
        assert_eq!(response2.content, "Second response");
        assert_eq!(response2.tool_calls.as_ref().unwrap().len(), 1);
        assert_eq!(response2.tool_calls.as_ref().unwrap()[0].name, "test_tool");
    }

    #[tokio::test]
    async fn test_strategy_testing_provider() {
        let provider = MockProvider::for_strategy_testing();
        let request = ChatRequest {
            messages: vec![Message {
                id: "test_msg_3".to_string(),
                role: MessageRole::User,
                content: "Find TODO comments".to_string(),
                timestamp: chrono::Utc::now(),
                tokens_used: None,
                cost: None,
            }],
            model: "mock-gpt-4".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: false,
            tools: None,
        };

        // First response should be a think phase with reflect tool
        let response1 = provider.chat(&request).await.unwrap();
        assert!(response1.content.contains("analyze"));
        assert_eq!(response1.tool_calls.as_ref().unwrap().len(), 1);
        assert_eq!(response1.tool_calls.as_ref().unwrap()[0].name, "reflect");

        // Second response should be an act phase with search_code tool
        let response2 = provider.chat(&request).await.unwrap();
        assert!(response2.content.contains("search"));
        assert_eq!(response2.tool_calls.as_ref().unwrap().len(), 1);
        assert_eq!(response2.tool_calls.as_ref().unwrap()[0].name, "search_code");
    }

    #[tokio::test]
    async fn test_streaming_functionality() {
        let provider = MockProvider::new();
        let request = ChatRequest {
            messages: vec![Message {
                id: "test_msg_4".to_string(),
                role: MessageRole::User,
                content: "Test streaming".to_string(),
                timestamp: chrono::Utc::now(),
                tokens_used: None,
                cost: None,
            }],
            model: "mock-gpt-4".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: true,
            tools: None,
        };

        let mut stream = provider.stream(&request).await.unwrap();
        let mut chunks = Vec::new();

        while let Some(chunk_result) = stream.recv().await {
            let chunk = chunk_result.unwrap();
            chunks.push(chunk);
        }

        assert!(!chunks.is_empty());

        // Reconstruct the message from chunks
        let full_content: String = chunks.iter().map(|c| c.content.clone()).collect();
        assert_eq!(full_content, "I understand the task and will proceed step by step.");
    }
}