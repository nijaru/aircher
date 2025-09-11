use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info, warn};
use std::collections::HashSet;
use std::time::Instant;
use chrono::Utc;

use crate::auth::AuthManager;
use crate::intelligence::IntelligenceEngine;
use crate::intelligence::duckdb_memory::AgentAction;
use crate::providers::{LLMProvider, ChatRequest, Message, MessageRole, PricingModel};
use crate::agent::tools::{ToolRegistry, ToolCall};
use crate::agent::parser::{ToolCallParser, format_tool_results};
use crate::agent::tool_formatter::{format_tool_status, format_tool_result, format_tool_batch};
use crate::agent::streaming::{AgentUpdate, AgentStream, create_agent_stream};
use crate::agent::conversation::{CodingConversation, Message as ConvMessage, MessageRole as ConvRole, ProjectContext};

pub struct AgentController {
    tools: ToolRegistry,
    intelligence: IntelligenceEngine,
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
    
    /// Convert tool registry to provider tool format for LLM requests
    fn convert_tools_to_provider_format(&self) -> Vec<crate::providers::Tool> {
        self.tools.list_tools()
            .into_iter()
            .map(|tool_info| crate::providers::Tool {
                name: tool_info.name,
                description: tool_info.description,
                parameters: tool_info.parameters,
            })
            .collect()
    }
    
    /// Build intelligence-enhanced system prompt
    async fn build_intelligent_system_prompt(&self, user_message: &str) -> Result<String> {
        let base_prompt = self.build_system_prompt();
        
        // Get intelligence insights
        let suggestions = self.intelligence.get_suggestions(user_message, None).await?;
        
        // Check for file mentions
        if let Some(file) = extract_file_mention(user_message) {
            match self.intelligence.predict_file_changes(&file).await {
                Ok(related_files) => {
                    return Ok(format!(
                        "{}\n\n## Intelligence Context:\n{}\nRelated files that often change with {}: {:?}",
                        base_prompt, suggestions, file, related_files
                    ));
                }
                Err(_) => {
                    // Fallback if file prediction fails
                    return Ok(format!("{}\n\n## Intelligence Context:\n{}", base_prompt, suggestions));
                }
            }
        }
        
        Ok(format!("{}\n\n## Intelligence Context:\n{}", base_prompt, suggestions))
    }
    
    /// Execute tools with tracking for pattern learning
    async fn execute_tools_with_tracking(&self, tool_calls: &[ToolCall]) -> (Vec<(String, Result<Value, String>)>, Vec<AgentAction>) {
        let mut results = Vec::new();
        let mut actions = Vec::new();
        
        for call in tool_calls {
            debug!("Executing tool: {} with params: {}", call.name, call.parameters);
            
            let start = Instant::now();
            
            if let Some(tool) = self.tools.get(&call.name) {
                match tool.execute(call.parameters.clone()).await {
                    Ok(output) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        let success = output.success;
                        
                        // Track the action for learning
                        actions.push(AgentAction {
                            tool: call.name.clone(),
                            params: call.parameters.clone(),
                            success,
                            duration_ms,
                            result_summary: if success {
                                format!("Success: {}", summarize_value(&output.result))
                            } else {
                                format!("Error: {}", output.error.as_deref().unwrap_or("Unknown error"))
                            },
                        });
                        
                        if success {
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
                        let duration_ms = start.elapsed().as_millis() as u64;
                        let error_msg = e.to_string();
                        
                        // Track the failed action
                        actions.push(AgentAction {
                            tool: call.name.clone(),
                            params: call.parameters.clone(),
                            success: false,
                            duration_ms,
                            result_summary: format!("Error: {}", error_msg),
                        });
                        
                        results.push((call.name.clone(), Err(error_msg)));
                    }
                }
            } else {
                let duration_ms = start.elapsed().as_millis() as u64;
                let error_msg = format!("Tool '{}' not found", call.name);
                
                // Track the failed action
                actions.push(AgentAction {
                    tool: call.name.clone(),
                    params: call.parameters.clone(),
                    success: false,
                    duration_ms,
                    result_summary: error_msg.clone(),
                });
                
                results.push((call.name.clone(), Err(error_msg)));
            }
        }
        
        (results, actions)
    }
    
    /// Record interaction pattern for learning
    async fn record_interaction_pattern(
        &self,
        user_message: &str,
        actions: &[AgentAction],
        response: &str,
        success: bool,
    ) -> Result<()> {
        use crate::intelligence::duckdb_memory::Pattern;
        
        // Extract files mentioned or modified
        let files = extract_files_from_actions(actions);
        
        // Generate embedding for the pattern
        let embedding_text = format!("{} {}", user_message, response);
        let embedding = self.intelligence.get_embedding(&embedding_text).await
            .unwrap_or_else(|e| {
                warn!("Failed to generate embedding: {}", e);
                vec![] // Fallback to empty embedding
            });
        
        // Create pattern for learning
        let pattern = Pattern {
            id: uuid::Uuid::new_v4().to_string(),
            description: user_message.to_string(),
            context: user_message.to_string(),
            actions: actions.to_vec(),
            files_involved: files,
            success,
            timestamp: Utc::now(),
            session_id: self.conversation.project_context.root_path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("aircher")
                .to_string(),
            embedding_text,
            embedding,
        };
        
        self.intelligence.record_pattern(pattern).await?;
        Ok(())
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
        let mut all_actions = Vec::new(); // Track all actions for pattern learning
        
        loop {
            iterations += 1;
            if iterations > self.max_iterations {
                warn!("Max iterations reached, stopping tool execution");
                break;
            }
            
            // Build chat request with intelligence-enhanced system prompt (only on first iteration)
            let system_prompt = if iterations == 1 {
                self.build_intelligent_system_prompt(user_message).await?
            } else {
                self.build_system_prompt()
            };
            
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
            
            // Get response from LLM
            let tools = if provider.supports_tools() {
                Some(self.convert_tools_to_provider_format())
            } else {
                None
            };
            
            debug!("Provider supports tools: {}, sending {} tools", provider.supports_tools(), tools.as_ref().map(|t| t.len()).unwrap_or(0));
            
            let request = ChatRequest {
                messages,
                model: model.to_string(),
                temperature: Some(0.7),
                max_tokens: Some(2000),
                stream: false, // TODO: Implement streaming support in agent
                tools, // FIXED: Actually send tool schemas to LLM instead of None
            };
            
            debug!("Calling provider.chat() with model: {}", model);
            let response = provider.chat(&request).await?;
            debug!("Got response from provider");
            let assistant_message = response.content;
            
            debug!("LLM Response: {}", assistant_message);
            
            // Use tool calls from response if available, otherwise parse from content
            let (clean_text, mut tool_calls) = if let Some(response_tool_calls) = response.tool_calls {
                // Modern providers (like Ollama with gpt-oss) return tool_calls directly
                let tool_calls = response_tool_calls.into_iter().map(|tc| {
                    crate::agent::tools::ToolCall {
                        name: tc.name,
                        parameters: tc.arguments,
                    }
                }).collect();
                (assistant_message.clone(), tool_calls)
            } else {
                // Legacy parsing for providers that embed tool calls in content
                self.parser.parse_structured(&assistant_message)?
            };
            debug!("Parsed - clean_text: '{}', tool_calls: {:?}", clean_text, tool_calls);
            
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
                    let status = format_tool_status(&call.name, &call.parameters, true);
                    tool_status_messages.push(status);
                }
                
                let (tool_results, actions) = self.execute_tools_with_tracking(&tool_calls).await;
                all_actions.extend(actions); // Track actions for pattern learning
                
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
        
        // Record interaction pattern for learning if we had any actions
        if !all_actions.is_empty() {
            let success = !final_response.contains("error") && 
                         !final_response.contains("failed") && 
                         !final_response.contains("Stopped due to repeated tool calls");
            
            if let Err(e) = self.record_interaction_pattern(
                user_message,
                &all_actions,
                &final_response,
                success,
            ).await {
                warn!("Failed to record interaction pattern: {}", e);
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

// Helper functions for intelligence integration
fn extract_file_mention(text: &str) -> Option<String> {
    // Look for file patterns like "main.rs", "src/lib.rs", etc.
    use regex::Regex;
    let file_regex = Regex::new(r"\b[\w/.-]+\.\w+\b").unwrap();
    file_regex.find(text).map(|m| m.as_str().to_string())
}

fn extract_files_from_actions(actions: &[AgentAction]) -> Vec<String> {
    let mut files = Vec::new();
    for action in actions {
        if action.tool == "read_file" || action.tool == "write_file" || action.tool == "edit_file" {
            if let Some(path) = action.params.get("path")
                .or_else(|| action.params.get("file_path"))
                .and_then(|v| v.as_str()) {
                files.push(path.to_string());
            }
        }
    }
    files.dedup();
    files
}

fn summarize_value(value: &Value) -> String {
    match value {
        Value::String(s) => {
            if s.len() > 100 {
                format!("{}...", &s[..100])
            } else {
                s.clone()
            }
        }
        Value::Object(map) => format!("{} fields", map.len()),
        Value::Array(arr) => format!("{} items", arr.len()),
        _ => value.to_string(),
    }
}
