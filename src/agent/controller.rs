use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info, warn};
use std::collections::HashSet;

use crate::auth::AuthManager;
use crate::intelligence::IntelligenceEngine;
use crate::providers::{LLMProvider, ChatRequest, Message, MessageRole, PricingModel};
use crate::agent::tools::{ToolRegistry, ToolCall};
use crate::agent::parser::{ToolCallParser, format_tool_results};
use crate::agent::tool_formatter::{format_tool_status, format_tool_result, format_tool_batch};
use crate::agent::streaming::{AgentUpdate, AgentStream, create_agent_stream};
use crate::agent::conversation::{CodingConversation, Message as ConvMessage, MessageRole as ConvRole, ProjectContext};

pub struct AgentController {
    tools: ToolRegistry,
    _intelligence: IntelligenceEngine,
    auth_manager: Arc<AuthManager>,
    parser: ToolCallParser,
    conversation: CodingConversation,
    max_iterations: usize,
}

impl AgentController {
    pub fn new(
        intelligence: IntelligenceEngine,
        auth_manager: Arc<AuthManager>,
        project_context: ProjectContext,
    ) -> Result<Self> {
        Ok(Self {
            tools: ToolRegistry::default(),
            _intelligence: intelligence,
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
    
    /// Validate that authentication is configured for the active provider/model
    async fn validate_auth_for_request(&self, provider: &dyn LLMProvider, model: &str) -> Result<()> {
        // Skip auth for local/free providers (e.g., Ollama)
        if matches!(provider.pricing_model(), PricingModel::Free) || provider.name().eq_ignore_ascii_case("ollama") {
            return Ok(());
        }

        // Map common provider names to auth keys
        let pname = provider.name().to_lowercase();
        let (auth_key, env_var) = if pname.contains("anthropic") || pname.contains("claude") {
            ("anthropic", "ANTHROPIC_API_KEY")
        } else if pname.contains("openai") {
            ("openai", "OPENAI_API_KEY")
        } else if pname.contains("gemini") || pname.contains("google") {
            ("gemini", "GOOGLE_API_KEY")
        } else {
            // Unknown provider auth handled at provider construction time
            return Ok(());
        };

        match self.auth_manager.get_api_key(auth_key).await {
            Ok(_) => {
                debug!("Auth validated for provider: {}", provider.name());
                Ok(())
            }
            Err(e) => {
                let error_msg = format!(
                    "Authentication required for provider '{}' (model: '{}'). Run /auth or set {}.",
                    provider.name(), model, env_var
                );
                warn!("Auth validation failed: {}", e);
                Err(anyhow::anyhow!(error_msg))
            }
        }
    }
    
    /// Process a user message and return the assistant's response with tool status
    pub async fn process_message(&mut self, user_message: &str, provider: &dyn LLMProvider, model: &str) -> Result<(String, Vec<String>)> {
        info!("Processing user message: {}", user_message);
        
        // Validate authentication before making LLM calls
        self.validate_auth_for_request(provider, model).await?;
        
        // Add user message to conversation
        self.conversation.messages.push(ConvMessage {
            role: ConvRole::User,
            content: user_message.to_string(),
            tool_calls: None,
            timestamp: chrono::Utc::now(),
        });
        
        let mut iterations = 0;
        let mut final_response = String::new();
        let mut tool_status_messages = Vec::new();
        let mut seen_tool_batches: HashSet<String> = HashSet::new();
        
        loop {
            iterations += 1;
            if iterations > self.max_iterations {
                warn!("Max iterations reached, stopping tool execution");
                break;
            }
            
            // Build chat request with system prompt
            let mut messages = vec![
                Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: MessageRole::System,
                    content: self.build_system_prompt(),
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
            
            // Get response from LLM
            let request = ChatRequest {
                messages,
                model: model.to_string(),
                temperature: Some(0.7),
                max_tokens: Some(2000),
                stream: false, // TODO: Implement streaming support in agent
                tools: None,
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
                
                break;
            } else {
                // Execute tool calls
                info!("Executing {} tool calls", tool_calls.len());

                // Detect repeated tool-call batch
                let sig = Self::signature_for_calls(&tool_calls);
                if seen_tool_batches.contains(&sig) {
                    warn!("Detected repeated tool calls; stopping after {} iterations", iterations);
                    final_response = "Stopped due to repeated tool calls. Please adjust your approach.".to_string();
                    self.conversation.messages.push(ConvMessage {
                        role: ConvRole::System,
                        content: "Detected repeated tool calls; halting to prevent loops.".to_string(),
                        tool_calls: None,
                        timestamp: chrono::Utc::now(),
                    });
                    break;
                }
                seen_tool_batches.insert(sig);
                
                // Add tool status messages for UI
                for call in &tool_calls {
                    tool_status_messages.push(format_tool_status(&call.name, &call.parameters, true));
                }
                
                let tool_results = self.execute_tools(&tool_calls).await;
                
                // Add result status messages
                for (tool_name, result) in &tool_results {
                    tool_status_messages.push(format_tool_result(tool_name, result));
                }
                
                // Add assistant message with tool calls
                self.conversation.messages.push(ConvMessage {
                    role: ConvRole::Assistant,
                    content: clean_text,
                    tool_calls: Some(tool_calls.iter().map(|tc| crate::agent::conversation::ToolCall {
                        tool_name: tc.name.clone(),
                        parameters: tc.parameters.clone(),
                        result: None,
                    }).collect()),
                    timestamp: chrono::Utc::now(),
                });
                
                // Format and add tool results
                let formatted_results = format_tool_results(&tool_results);
                self.conversation.messages.push(ConvMessage {
                    role: ConvRole::Tool,
                    content: formatted_results,
                    tool_calls: None,
                    timestamp: chrono::Utc::now(),
                });
            }
        }
        
        Ok((final_response, tool_status_messages))
    }
    
    /// Process a user message with streaming support
    pub async fn process_message_streaming(&mut self, user_message: &str, provider: &dyn LLMProvider, model: &str) -> Result<AgentStream> {
        let (tx, rx) = create_agent_stream();
        
        // Add user message to conversation immediately
        self.conversation.messages.push(ConvMessage {
            role: ConvRole::User,
            content: user_message.to_string(),
            tool_calls: None,
            timestamp: chrono::Utc::now(),
        });
        
        // Process directly without spawning (simpler approach)
        match self.process_message_internal(user_message, provider, model, &tx).await {
            Ok(_) => {
                let _ = tx.send(Ok(AgentUpdate::Complete {
                    total_tokens: 0, // TODO: Track actual tokens
                    tool_status_messages: vec![],
                })).await;
            }
            Err(e) => {
                let _ = tx.send(Ok(AgentUpdate::Error(e.to_string()))).await;
            }
        }
        
        Ok(rx)
    }
    
    /// Internal processing method that can send streaming updates
    async fn process_message_internal(&mut self, user_message: &str, provider: &dyn LLMProvider, model: &str, tx: &crate::agent::streaming::AgentStreamSender) -> Result<()> {
        info!("Processing user message with streaming: {}", user_message);
        
        // Validate authentication before making LLM calls
        self.validate_auth_for_request(provider, model).await?;
        
        let mut iterations = 0;
        let mut tool_status_messages = Vec::new();
        let mut seen_tool_batches: HashSet<String> = HashSet::new();
        
        loop {
            iterations += 1;
            if iterations > self.max_iterations {
                warn!("Max iterations reached, stopping tool execution");
                break;
            }
            
            // Build chat request with system prompt
            let mut messages = vec![
                Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: MessageRole::System,
                    content: self.build_system_prompt(),
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
                        ConvRole::Tool => MessageRole::User,
                    },
                    content: msg.content.clone(),
                    timestamp: msg.timestamp,
                    tokens_used: None,
                    cost: None,
                });
            }
            
            // Create streaming request
            let request = ChatRequest {
                messages,
                model: model.to_string(),
                temperature: Some(0.7),
                max_tokens: Some(2000),
                stream: true, // Enable streaming
                tools: None,
            };
            
            // Use streaming for the response
            match provider.stream(&request).await {
                Ok(mut stream) => {
                    let mut response_content = String::new();
                    let mut _total_tokens = 0;
                    
                    // Process streaming chunks
                    while let Some(chunk_result) = stream.recv().await {
                        match chunk_result {
                            Ok(chunk) => {
                                response_content.push_str(&chunk.content);
                                if let Some(tokens) = chunk.tokens_used {
                                    _total_tokens = tokens;
                                }
                                
                                // Send streaming update to UI
                                let _ = tx.send(Ok(AgentUpdate::TextChunk {
                                    content: chunk.content,
                                    delta: chunk.delta,
                                    tokens_used: chunk.tokens_used,
                                })).await;
                                
                                // Check if stream is complete
                                if chunk.finish_reason.is_some() {
                                    break;
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(Ok(AgentUpdate::Error(e.to_string()))).await;
                                return Err(anyhow::anyhow!("Streaming error: {}", e));
                            }
                        }
                    }
                    
                    // Parse tool calls from complete response
                    let (clean_text, tool_calls) = self.parser.parse_structured(&response_content)?;
                    
                    if tool_calls.is_empty() {
                        // No tool calls, we're done
                        self.conversation.messages.push(ConvMessage {
                            role: ConvRole::Assistant,
                            content: clean_text,
                            tool_calls: None,
                            timestamp: chrono::Utc::now(),
                        });
                        break;
                    } else {
                        // Execute tool calls
                        info!("Executing {} tool calls", tool_calls.len());
                        // Detect repeated tool-call batch
                        let sig = Self::signature_for_calls(&tool_calls);
                        if seen_tool_batches.contains(&sig) {
                            warn!("Detected repeated tool calls; stopping after {} iterations", iterations);
                            let _ = tx.send(Ok(AgentUpdate::ToolStatus("✗ Repeated tool calls detected; stopping.".to_string()))).await;
                            self.conversation.messages.push(ConvMessage {
                                role: ConvRole::System,
                                content: "Detected repeated tool calls; halting to prevent loops.".to_string(),
                                tool_calls: None,
                                timestamp: chrono::Utc::now(),
                            });
                            break;
                        }
                        seen_tool_batches.insert(sig);
                        
                        // Send batch header then per-tool status updates
                        let batch = format_tool_batch(&tool_calls);
                        if !batch.is_empty() {
                            let _ = tx.send(Ok(AgentUpdate::ToolStatus(batch.clone()))).await;
                            tool_status_messages.push(batch);
                        }
                        // Send tool status updates
                        for call in &tool_calls {
                            let status = format_tool_status(&call.name, &call.parameters, true);
                            tool_status_messages.push(status.clone());
                            let _ = tx.send(Ok(AgentUpdate::ToolStatus(status))).await;
                        }
                        
                        let tool_results = self.execute_tools(&tool_calls).await;
                        
                        // Send tool result updates
                        for (tool_name, result) in &tool_results {
                            let status = format_tool_result(tool_name, result);
                            tool_status_messages.push(status.clone());
                            let _ = tx.send(Ok(AgentUpdate::ToolStatus(status))).await;
                        }
                        
                        // Add assistant message with tool calls
                        self.conversation.messages.push(ConvMessage {
                            role: ConvRole::Assistant,
                            content: clean_text,
                            tool_calls: Some(tool_calls.iter().map(|tc| crate::agent::conversation::ToolCall {
                                tool_name: tc.name.clone(),
                                parameters: tc.parameters.clone(),
                                result: None,
                            }).collect()),
                            timestamp: chrono::Utc::now(),
                        });
                        
                        // Add tool results
                        let formatted_results = format_tool_results(&tool_results);
                        self.conversation.messages.push(ConvMessage {
                            role: ConvRole::Tool,
                            content: formatted_results,
                            tool_calls: None,
                            timestamp: chrono::Utc::now(),
                        });
                        
                        // Continue the loop for next iteration
                    }
                }
                Err(e) => {
                    let _ = tx.send(Ok(AgentUpdate::Error(e.to_string()))).await;
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }

    /// Create a stable signature for a batch of tool calls
    fn signature_for_calls(calls: &[ToolCall]) -> String {
        let mut parts: Vec<String> = calls.iter().map(|c| {
            let mut args = c.parameters.clone();
            // Best-effort canonicalization: sort object keys if present
            if let Some(obj) = args.as_object_mut() {
                let mut entries: Vec<(String, Value)> = obj.iter().map(|(k,v)| (k.clone(), v.clone())).collect();
                entries.sort_by(|a, b| a.0.cmp(&b.0));
                let mut new = serde_json::Map::new();
                for (k, v) in entries { new.insert(k, v); }
                args = Value::Object(new);
            }
            format!("{}:{}", c.name, args)
        }).collect();
        parts.sort();
        parts.join("|")
    }
    
    /// Execute a list of tool calls
    async fn execute_tools(&self, tool_calls: &[ToolCall]) -> Vec<(String, Result<Value, String>)> {
        let mut results = Vec::new();
        
        for call in tool_calls {
            debug!("Executing tool: {} with params: {}", call.name, call.parameters);
            
            if let Some(tool) = self.tools.get(&call.name) {
                let start = std::time::Instant::now();
                match tool.execute(call.parameters.clone()).await {
                    Ok(output) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        if output.success {
                            // Try to inject duration_ms into result if it's an object
                            let mut result_value = output.result;
                            if let Some(obj) = result_value.as_object_mut() {
                                obj.insert("duration_ms".to_string(), serde_json::json!(duration_ms));
                            }
                            results.push((call.name.clone(), Ok(result_value)));
                        } else {
                            let error = output.error.unwrap_or_else(|| "Unknown error".to_string());
                            results.push((call.name.clone(), Err(error)));
                        }
                    }
                    Err(e) => {
                        results.push((call.name.clone(), Err(e.to_string())));
                    }
                }
            } else {
                results.push((call.name.clone(), Err(format!("Unknown tool: {}", call.name))));
            }
        }
        
        results
    }
    
    /// Build the system prompt for the coding assistant
    fn build_system_prompt(&self) -> String {
        format!(r#"You are an AI coding assistant helping with a {} project at {}.

You have access to the following tools:
{}

To use a tool, wrap your tool calls in <tool_use> blocks:
<tool_use>
<tool>tool_name</tool><params>{{"param": "value"}}</params>
</tool_use>

Guidelines:
1. Always read files before editing them
2. Use search_code to find relevant code across the project
3. Make focused, specific edits rather than rewriting entire files
4. Verify your changes by reading the file again after editing
5. Run tests or build commands to ensure changes work
6. Explain your reasoning and the changes you're making

Current context:
- Project root: {}
- Active files: {:?}
- Recent tasks: {} pending, {} completed

Be concise but thorough. Focus on solving the user's problem effectively."#,
            self.conversation.project_context.language.to_string(),
            self.conversation.project_context.root_path.display(),
            self.format_tool_list(),
            self.conversation.project_context.root_path.display(),
            self.conversation.active_files,
            self.conversation.task_list.iter().filter(|t| matches!(t.status, crate::agent::conversation::TaskStatus::Pending)).count(),
            self.conversation.task_list.iter().filter(|t| matches!(t.status, crate::agent::conversation::TaskStatus::Completed)).count(),
        )
    }
    
    /// Format the list of available tools
    fn format_tool_list(&self) -> String {
        let tools = self.tools.list_tools();
        tools.iter()
            .map(|info| format!("- {}: {}", info.name, info.description))
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Add a tool to the registry
    pub fn register_tool(&mut self, tool: Box<dyn crate::agent::tools::AgentTool>) {
        self.tools.register(tool);
    }
    
    /// Get the current conversation
    pub fn conversation(&self) -> &CodingConversation {
        &self.conversation
    }
    
    /// Clear the conversation history
    pub fn clear_conversation(&mut self) {
        self.conversation.messages.clear();
    }
}
