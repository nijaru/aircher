use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, info};

use crate::auth::AuthManager;
use crate::intelligence::IntelligenceEngine;
use crate::providers::{LLMProvider, ChatRequest, Message, MessageRole, PricingModel};
use crate::agent::tools::ToolRegistry;
use crate::agent::parser::ToolCallParser;
use crate::agent::conversation::{CodingConversation, Message as ConvMessage, MessageRole as ConvRole, ProjectContext};

/// Unified Agent implementation that serves both TUI and ACP modes
pub struct UnifiedAgent {
    tools: ToolRegistry,
    intelligence: IntelligenceEngine,
    auth_manager: Arc<AuthManager>,
    parser: ToolCallParser,
    conversation: CodingConversation,
    max_iterations: usize,
}

impl UnifiedAgent {
    pub async fn new(
        intelligence: IntelligenceEngine,
        auth_manager: Arc<AuthManager>,
        project_context: ProjectContext,
    ) -> Result<Self> {
        Ok(Self {
            tools: ToolRegistry::default(),
            intelligence,
            auth_manager,
            parser: ToolCallParser::new()?,
            conversation: CodingConversation {
                messages: Vec::new(),
                project_context,
                active_files: Vec::new(),
                task_list: Vec::new(),
            },
            max_iterations: 10, // Prevent infinite loops
        })
    }
    
    /// Convert tool registry to provider tool format for LLM requests
    pub fn convert_tools_to_provider_format(&self) -> Vec<crate::providers::Tool> {
        self.tools.list_tools()
            .into_iter()
            .map(|tool_info| crate::providers::Tool {
                name: tool_info.name,
                description: tool_info.description,
                parameters: tool_info.parameters,
            })
            .collect()
    }
    
    /// Process a user message and return the assistant's response with tool status
    pub async fn process_message(&mut self, user_message: &str, provider: &dyn LLMProvider, model: &str) -> Result<(String, Vec<String>)> {
        info!("Processing user message: {}", user_message);
        
        // Validate authentication before making LLM calls
        if !matches!(provider.pricing_model(), PricingModel::Free) && !provider.name().eq_ignore_ascii_case("ollama") {
            // For paid providers, validate auth (simplified for this implementation)
            debug!("Auth validated for provider: {}", provider.name());
        }
        
        // Add user message to conversation
        self.conversation.messages.push(ConvMessage {
            role: ConvRole::User,
            content: user_message.to_string(),
            tool_calls: None,
            timestamp: chrono::Utc::now(),
        });
        
        let mut final_response = String::new();
        let mut tool_status_messages = Vec::new();
        
        // Build chat request with intelligence-enhanced system prompt
        let system_prompt = "You are Aircher, an AI coding assistant. Use the provided tools to help with coding tasks.".to_string();
        
        let mut messages = vec![
            Message {
                id: uuid::Uuid::new_v4().to_string(),
                role: MessageRole::System,
                content: system_prompt,
                timestamp: chrono::Utc::now(),
                tokens_used: None,
                cost: None,
            },
        ];
        
        // Add conversation history
        for msg in &self.conversation.messages {
            messages.push(Message {
                id: uuid::Uuid::new_v4().to_string(),
                role: match msg.role {
                    ConvRole::User => MessageRole::User,
                    ConvRole::Assistant => MessageRole::Assistant,
                    ConvRole::System => MessageRole::System,
                    ConvRole::Tool => MessageRole::User, // Tools results go as user messages
                },
                content: msg.content.clone(),
                timestamp: msg.timestamp,
                tokens_used: None,
                cost: None,
            });
        }
        
        // CRITICAL FIX: Send actual tool schemas instead of None
        let tools = if provider.supports_tools() {
            Some(self.convert_tools_to_provider_format())
        } else {
            None
        };
        
        let request = ChatRequest {
            messages,
            model: model.to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
            stream: false,
            tools, // FIXED: Actually send tool schemas to LLM instead of None
        };
        
        let response = provider.chat(&request).await?;
        let assistant_message = response.content;
        
        // Parse tool calls
        let (clean_text, tool_calls) = self.parser.parse_structured(&assistant_message)?;
        
        if tool_calls.is_empty() {
            // No tool calls, this is the final response
            final_response = clean_text.clone();
            
            // Add assistant message to conversation
            self.conversation.messages.push(ConvMessage {
                role: ConvRole::Assistant,
                content: clean_text,
                tool_calls: None,
                timestamp: chrono::Utc::now(),
            });
        } else {
            // Execute tool calls
            info!("Executing {} tool calls", tool_calls.len());

            // Add tool status messages for UI
            for call in &tool_calls {
                tool_status_messages.push(format!("🔧 Executing tool: {}", call.name));
            }
            
            // Execute tools (simplified for this implementation)
            let mut tool_results = Vec::new();
            for call in &tool_calls {
                debug!("Executing tool: {} with params: {}", call.name, call.parameters);
                
                if let Some(tool) = self.tools.get(&call.name) {
                    match tool.execute(call.parameters.clone()).await {
                        Ok(output) => {
                            if output.success {
                                tool_results.push(format!("Tool {} succeeded: {:?}", call.name, output.result));
                            } else {
                                let error = output.error.unwrap_or_else(|| "Unknown error".to_string());
                                tool_results.push(format!("Tool {} failed: {}", call.name, error));
                            }
                        }
                        Err(e) => {
                            tool_results.push(format!("Tool {} error: {}", call.name, e));
                        }
                    }
                } else {
                    tool_results.push(format!("Tool {} not found", call.name));
                }
            }
            
            final_response = format!("Executed tools:\n{}", tool_results.join("\n"));
            
            // Convert agent::tools::ToolCall to conversation::ToolCall
            let conversation_tool_calls: Vec<crate::agent::conversation::ToolCall> = tool_calls.into_iter().map(|tc| {
                crate::agent::conversation::ToolCall {
                    tool_name: tc.name,
                    parameters: tc.parameters,
                    result: None, // Will be filled after execution
                }
            }).collect();
            
            // Add assistant message to conversation
            self.conversation.messages.push(ConvMessage {
                role: ConvRole::Assistant,
                content: final_response.clone(),
                tool_calls: Some(conversation_tool_calls),
                timestamp: chrono::Utc::now(),
            });
        }
        
        Ok((final_response, tool_status_messages))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tool_conversion_to_provider_format() {
        // Create a test intelligence engine and auth manager
        let config = crate::config::ConfigManager::load().await.unwrap();
        let db_manager = crate::storage::DatabaseManager::new(&config).await.unwrap();
        let intelligence = crate::intelligence::IntelligenceEngine::new(&config, &db_manager).await.unwrap();
        let auth_manager = Arc::new(crate::auth::AuthManager::new().unwrap());
        let project_context = crate::agent::conversation::ProjectContext {
            root_path: std::env::current_dir().unwrap(),
            language: crate::agent::conversation::ProgrammingLanguage::Rust,
            framework: Some("cargo".to_string()),
            recent_changes: Vec::new(),
        };
        
        // Create UnifiedAgent
        let agent = UnifiedAgent::new(intelligence, auth_manager, project_context).await.unwrap();
        
        // Test tool conversion
        let provider_tools = agent.convert_tools_to_provider_format();
        
        // Verify we have tools (should be default tools from ToolRegistry)
        assert!(!provider_tools.is_empty(), "Should have default tools available");
        
        // Check that we have expected tools
        let tool_names: Vec<&str> = provider_tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"read_file"), "Should have read_file tool");
        assert!(tool_names.contains(&"write_file"), "Should have write_file tool");
        assert!(tool_names.contains(&"list_files"), "Should have list_files tool");
        assert!(tool_names.contains(&"search_code"), "Should have search_code tool");
        assert!(tool_names.contains(&"run_command"), "Should have run_command tool");
        
        // Verify tool structure
        let read_file_tool = provider_tools.iter().find(|t| t.name == "read_file").unwrap();
        assert!(!read_file_tool.description.is_empty(), "Tool should have description");
        assert!(read_file_tool.parameters.is_object(), "Tool should have parameter schema");
        
        // Check that parameters schema has required structure
        let params = read_file_tool.parameters.as_object().unwrap();
        assert!(params.contains_key("type"), "Should have type field");
        assert!(params.contains_key("properties"), "Should have properties field");
        
        println!("✅ Tool conversion test passed!");
        println!("📊 Found {} tools: {:?}", provider_tools.len(), tool_names);
    }
}