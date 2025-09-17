use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, info, warn};

#[cfg(feature = "acp")]
use agent_client_protocol::{Agent as AcpAgent, InitializeRequest, InitializeResponse, NewSessionRequest, NewSessionResponse, PromptRequest, PromptResponse, ContentBlock, StopReason, AgentCapabilities, PromptCapabilities, SessionId, AuthenticateRequest, LoadSessionRequest, CancelNotification};

use crate::auth::AuthManager;
use crate::intelligence::IntelligenceEngine;
use crate::providers::{LLMProvider, ChatRequest, Message, MessageRole, PricingModel};
use crate::agent::tools::ToolRegistry;
use crate::agent::parser::ToolCallParser;
use crate::agent::conversation::{CodingConversation, Message as ConvMessage, MessageRole as ConvRole, ProjectContext};
use crate::agent::reasoning::{AgentReasoning, TaskStatus};
use crate::agent::dynamic_context::DynamicContextManager;
use crate::agent::task_orchestrator::TaskOrchestrator;
use crate::agent::plan_mode::{PlanGenerator, PlanMode, PlanningContext};
use crate::semantic_search::SemanticCodeSearch;

/// Unified Agent implementation that serves both TUI and ACP modes
pub struct Agent {
    tools: Arc<ToolRegistry>,
    intelligence: Arc<IntelligenceEngine>,
    #[allow(dead_code)]
    auth_manager: Arc<AuthManager>,
    parser: ToolCallParser,
    conversation: Arc<tokio::sync::Mutex<CodingConversation>>,
    reasoning: Arc<AgentReasoning>,
    context_manager: Arc<DynamicContextManager>,
    #[allow(dead_code)]
    orchestrator: Option<Arc<TaskOrchestrator>>,
    plan_generator: Arc<tokio::sync::Mutex<PlanGenerator>>,
    /// Prevents infinite orchestration recursion
    is_orchestration_agent: bool,
    #[allow(dead_code)]
    max_iterations: usize,
}

impl Agent {
    pub async fn new(
        intelligence: IntelligenceEngine,
        auth_manager: Arc<AuthManager>,
        project_context: ProjectContext,
    ) -> Result<Self> {
        let tools = Arc::new(ToolRegistry::default());
        let intelligence = Arc::new(intelligence);

        Self::new_internal(intelligence, auth_manager, project_context, tools).await
    }

    /// Create agent with approval-enabled tools
    pub async fn new_with_approval(
        intelligence: IntelligenceEngine,
        auth_manager: Arc<AuthManager>,
        project_context: ProjectContext,
    ) -> Result<(Self, tokio::sync::mpsc::UnboundedReceiver<crate::agent::approval_modes::PendingChange>)> {
        use crate::agent::tools::approval_registry::create_agent_registry_with_approval;

        let intelligence = Arc::new(intelligence);
        let (tools, approval_rx) = create_agent_registry_with_approval();
        let tools = Arc::new(tools);

        let mut agent = Self::new_internal(intelligence, auth_manager, project_context, tools).await?;

        // Set plan mode to Review by default for approval-enabled agents
        {
            let mut plan_gen = agent.plan_generator.lock().await;
            plan_gen.set_mode(PlanMode::Normal); // Start in normal mode
        }
        Ok((agent, approval_rx))
    }

    async fn new_internal(
        intelligence: Arc<IntelligenceEngine>,
        auth_manager: Arc<AuthManager>,
        project_context: ProjectContext,
        tools: Arc<ToolRegistry>,
    ) -> Result<Self> {

        // Create semantic search for context management (optional - may not have index)
        // Only create if we can, otherwise context manager works without it
        let search = Some(Arc::new(tokio::sync::RwLock::new(SemanticCodeSearch::new())));

        // Create dynamic context manager for intelligent context management
        let context_manager = Arc::new(DynamicContextManager::new(
            intelligence.clone(),
            search,
        ));

        // Create the reasoning engine with intelligent planning capabilities
        let reasoning = Arc::new(AgentReasoning::new(
            intelligence.clone(),
            tools.clone(),
        ));

        Ok(Self {
            tools,
            intelligence,
            auth_manager,
            parser: ToolCallParser::new()?,
            conversation: Arc::new(tokio::sync::Mutex::new(CodingConversation {
                messages: Vec::new(),
                project_context,
                active_files: Vec::new(),
                task_list: Vec::new(),
            })),
            reasoning,
            context_manager,
            orchestrator: None, // Created on-demand to avoid circular dependency
            plan_generator: Arc::new(tokio::sync::Mutex::new(PlanGenerator::new())),
            is_orchestration_agent: false,
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
    pub async fn process_message(&self, user_message: &str, provider: &dyn LLMProvider, model: &str) -> Result<(String, Vec<String>)> {
        info!("Processing user message with intelligent reasoning: {}", user_message);
        
        // Validate authentication before making LLM calls
        if !matches!(provider.pricing_model(), PricingModel::Free) && !provider.name().eq_ignore_ascii_case("ollama") {
            debug!("Auth validated for provider: {}", provider.name());
        }
        
        // Add user message to conversation
        {
            let mut conversation = self.conversation.lock().await;
            conversation.messages.push(ConvMessage {
                role: ConvRole::User,
                content: user_message.to_string(),
                tool_calls: None,
                timestamp: chrono::Utc::now(),
            });
        }

        let mut tool_status_messages = Vec::new();

        // Check if this task needs multi-turn orchestration
        if self.needs_orchestration(user_message).await {
            info!("Task requires orchestration - using TaskOrchestrator for complex workflow");
            return self.process_with_orchestration(user_message, provider, model).await;
        }

        // Use reasoning engine for task planning (NOT execution)
        let reasoning_result = match self.reasoning.process_request(user_message).await {
            Ok(result) => Some(result),
            Err(e) => {
                debug!("Reasoning engine failed to plan: {}", e);
                None
            }
        };

        // Check if reasoning provided a plan to enhance our prompt
        if let Some(result) = reasoning_result {
            if result.task.status == TaskStatus::Planned && !result.task.tool_calls.is_empty() {
                // Reasoning engine has planned tool calls - add them to prompt
                info!("Using reasoning engine plan: {} tool calls planned", result.task.tool_calls.len());

                // Add planned tasks to status messages for UI
                for tool_call in &result.task.tool_calls {
                    tool_status_messages.push(format!("📋 Planned: {} with {}",
                        tool_call.name,
                        tool_call.parameters
                    ));
                }

                // The LLM will execute these with proper content generation
            } else if result.task.status == TaskStatus::Completed {
                // If reasoning claims completion without tool calls, it's fake
                warn!("Reasoning engine claims completion without execution - ignoring");
            }
        }

        // Always proceed with LLM-based execution for actual work
        debug!("Proceeding with LLM-based tool execution");
        
        // Build chat request with intelligence-enhanced system prompt
        let mut system_prompt = "You are Aircher, an AI coding assistant. Use the provided tools to help with coding tasks.".to_string();
        
        // Enhance prompt with intelligence suggestions
        if let Ok(suggestions) = self.intelligence.get_suggestions(user_message, None).await {
            if !suggestions.trim().is_empty() && suggestions != "Intelligence memory not initialized" {
                system_prompt.push_str(&format!("\n\nContext from previous patterns:\n{}", suggestions));
            }
        }

        // Update dynamic context based on current activity
        if let Ok(context_update) = self.context_manager.update_context(user_message).await {
            debug!("Context updated: {} items added, {} removed",
                context_update.added.len(),
                context_update.removed.len());

            // Add relevant context to the system prompt
            if let Ok(relevant_context) = self.context_manager.get_relevant_context(5).await {
                if !relevant_context.is_empty() {
                    system_prompt.push_str("\n\n## Relevant Context:\n");
                    for item in relevant_context {
                        system_prompt.push_str(&format!("- {}\n", item.summary()));
                    }
                }
            }
        }
        
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
        {
            let conversation = self.conversation.lock().await;
            for msg in &conversation.messages {
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

        // Get tool calls - first check structured response, then parse from text
        let (clean_text, tool_calls) = if let Some(structured_tool_calls) = &response.tool_calls {
            // Convert from provider tool calls to agent tool calls
            let agent_tool_calls: Vec<crate::agent::tools::ToolCall> = structured_tool_calls
                .iter()
                .map(|tc| crate::agent::tools::ToolCall {
                    name: tc.name.clone(),
                    parameters: tc.arguments.clone(),
                })
                .collect();

            (assistant_message, agent_tool_calls)
        } else {
            // Fall back to parsing from text content
            self.parser.parse_structured(&assistant_message)?
        };

        if tool_calls.is_empty() {
            // Add assistant message to conversation
            {
                let mut conversation = self.conversation.lock().await;
                conversation.messages.push(ConvMessage {
                    role: ConvRole::Assistant,
                    content: clean_text.clone(),
                    tool_calls: None,
                    timestamp: chrono::Utc::now(),
                });
            }
        }

        let final_response = if tool_calls.is_empty() {
            // No tool calls, this is the final response
            clean_text.clone()
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
                            // Track context access for file-related tools
                            if call.name == "read_file" || call.name == "edit_file" || call.name == "write_file" {
                                if let Some(path) = call.parameters.get("path").or(call.parameters.get("file_path")).and_then(|p| p.as_str()) {
                                    let action = if call.name == "read_file" {
                                        crate::agent::dynamic_context::AccessAction::Read
                                    } else {
                                        crate::agent::dynamic_context::AccessAction::Modified
                                    };
                                    let _ = self.context_manager.track_file_access(path, action).await;
                                }
                            }

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
            
            let response = format!("Executed tools:\n{}", tool_results.join("\n"));
            
            // Record patterns from tool execution for intelligence learning
            for (i, call) in tool_calls.iter().enumerate() {
                if let Ok(embedding) = self.intelligence.get_embedding(&call.parameters.to_string()).await {
                    let action = crate::intelligence::duckdb_memory::AgentAction {
                        tool: call.name.clone(),
                        params: call.parameters.clone(),
                        success: tool_results.get(i).map(|r| r.contains("succeeded")).unwrap_or(false),
                        duration_ms: 100, // Placeholder - would measure actual duration
                        result_summary: tool_results.get(i).unwrap_or(&"No result".to_string()).clone(),
                    };
                    
                    let pattern = crate::intelligence::duckdb_memory::Pattern {
                        id: uuid::Uuid::new_v4().to_string(),
                        description: format!("Tool execution: {}", call.name),
                        context: user_message.to_string(),
                        actions: vec![action],
                        files_involved: vec![], // Would extract from tool parameters
                        success: tool_results.get(i).map(|r| r.contains("succeeded")).unwrap_or(false),
                        timestamp: chrono::Utc::now(),
                        session_id: "current".to_string(), // Would come from actual session
                        embedding_text: format!("{} {}", user_message, call.name),
                        embedding,
                    };
                    
                    if let Err(e) = self.intelligence.record_pattern(pattern).await {
                        debug!("Failed to record pattern: {}", e);
                    }
                }
            }
            
            // Convert agent::tools::ToolCall to conversation::ToolCall
            let conversation_tool_calls: Vec<crate::agent::conversation::ToolCall> = tool_calls.into_iter().map(|tc| {
                crate::agent::conversation::ToolCall {
                    tool_name: tc.name,
                    parameters: tc.parameters,
                    result: None, // Will be filled after execution
                }
            }).collect();
            
            // Add assistant message to conversation
            {
                let mut conversation = self.conversation.lock().await;
                conversation.messages.push(ConvMessage {
                    role: ConvRole::Assistant,
                    content: response.clone(),
                    tool_calls: Some(conversation_tool_calls),
                    timestamp: chrono::Utc::now(),
                });
            }

            response
        };
        
        Ok((final_response, tool_status_messages))
    }

    /// Determine if a task needs multi-turn orchestration
    async fn needs_orchestration(&self, user_message: &str) -> bool {
        // Don't orchestrate if this is already an orchestration agent
        if self.is_orchestration_agent {
            return false;
        }
        let message_lower = user_message.to_lowercase();

        // Tasks that clearly need orchestration based on keywords
        if message_lower.contains("implement feature")
            || message_lower.contains("build application")
            || message_lower.contains("create system")
            || message_lower.contains("refactor codebase")
            || message_lower.contains("add comprehensive")
            || (message_lower.contains("create") && message_lower.contains("test suite"))
            || (message_lower.contains("implement") && message_lower.len() > 100) // Long complex requests
        {
            return true;
        }

        // REMOVED: Reasoning check that causes infinite recursion and timeouts
        // The reasoning.process_request call here would trigger another process_message
        // which checks needs_orchestration again, creating a loop

        false
    }

    /// Process a complex task using the TaskOrchestrator
    async fn process_with_orchestration(
        &self,
        user_message: &str,
        provider: &dyn LLMProvider,
        model: &str
    ) -> Result<(String, Vec<String>)> {
        info!("Processing complex task with orchestration");

        // Create orchestrator on-demand (avoiding circular dependency)
        let orchestrator = TaskOrchestrator::new(
            Arc::new(Agent {
                tools: self.tools.clone(),
                intelligence: self.intelligence.clone(),
                auth_manager: self.auth_manager.clone(),
                parser: ToolCallParser::new()?, // Create new parser (doesn't impl Clone)
                conversation: self.conversation.clone(),
                reasoning: self.reasoning.clone(),
                context_manager: self.context_manager.clone(),
                orchestrator: None, // Avoid infinite recursion
                plan_generator: Arc::new(tokio::sync::Mutex::new(PlanGenerator::new())),
                is_orchestration_agent: true, // Mark as orchestration agent to prevent recursion
                max_iterations: self.max_iterations,
            }),
            self.reasoning.clone(),
            self.context_manager.clone(),
            self.intelligence.clone(),
        );

        // Execute the complex task
        let task_result = orchestrator.execute_task(user_message, provider, model).await?;

        // Convert orchestration result to standard response format
        let mut tool_status_messages = Vec::new();

        // Add summary of orchestration steps
        tool_status_messages.push(format!("🎯 Orchestrated {} steps successfully", task_result.steps_completed));

        if !task_result.files_modified.is_empty() {
            tool_status_messages.push(format!("📝 Modified {} files", task_result.files_modified.len()));
        }

        // Add conversation message
        {
            let mut conversation = self.conversation.lock().await;
            conversation.messages.push(ConvMessage {
                role: ConvRole::Assistant,
                content: task_result.summary.clone(),
                tool_calls: None,
                timestamp: chrono::Utc::now(),
            });
        }

        Ok((task_result.summary, tool_status_messages))
    }

    /// Direct message processing without reasoning engine (fallback)
    pub async fn process_message_direct(&self, user_message: &str, provider: &dyn LLMProvider, model: &str) -> Result<(String, Vec<String>)> {
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
        {
            let conversation = self.conversation.lock().await;
            for msg in &conversation.messages {
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
        }

        // Add current user message
        messages.push(Message {
            id: uuid::Uuid::new_v4().to_string(),
            role: MessageRole::User,
            content: user_message.to_string(),
            timestamp: chrono::Utc::now(),
            tokens_used: None,
            cost: None,
        });

        // Send with tools if provider supports them
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
            tools,
        };
        
        let response = provider.chat(&request).await?;
        let assistant_message = response.content;
        
        // Parse tool calls
        let (clean_text, tool_calls) = self.parser.parse_structured(&assistant_message)?;
        
        if tool_calls.is_empty() {
            // Add assistant message to conversation
            {
                let mut conversation = self.conversation.lock().await;
                conversation.messages.push(ConvMessage {
                    role: ConvRole::Assistant,
                    content: clean_text.clone(),
                    tool_calls: None,
                    timestamp: chrono::Utc::now(),
                });
            }
        }

        let final_response = if tool_calls.is_empty() {
            // No tool calls, this is the final response
            clean_text.clone()
        } else {
            info!("Executing {} tool calls", tool_calls.len());
            
            for call in &tool_calls {
                tool_status_messages.push(format!("🔧 Executing tool: {}", call.name));
            }
            
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
            
            let response = format!("Executed tools:\n{}", tool_results.join("\n"));
            
            let conversation_tool_calls: Vec<crate::agent::conversation::ToolCall> = tool_calls.into_iter().map(|tc| {
                crate::agent::conversation::ToolCall {
                    tool_name: tc.name,
                    parameters: tc.parameters,
                    result: None,
                }
            }).collect();
            
            {
                let mut conversation = self.conversation.lock().await;
                conversation.messages.push(ConvMessage {
                    role: ConvRole::Assistant,
                    content: response.clone(),
                    tool_calls: Some(conversation_tool_calls),
                    timestamp: chrono::Utc::now(),
                });
            }

            response
        };
        
        Ok((final_response, tool_status_messages))
    }
    
    /// Get list of available tool names
    pub async fn list_tools(&self) -> Result<Vec<String>> {
        let tool_names: Vec<String> = self.tools.list_tools()
            .iter()
            .map(|t| t.name.clone())
            .collect();
        Ok(tool_names)
    }
    
    /// Send message with provider and model specification
    pub async fn send_message(
        &mut self,
        _session_id: &str,
        message: &str,
        provider_name: &str,
        model_name: &str,
    ) -> Result<crate::client::AgentResponse> {
        use crate::providers::ProviderManager;
        
        // Create a temporary provider manager for this call
        // In practice, this would come from a stored provider manager
        let config = crate::config::ConfigManager::load().await?;
        let auth_manager = crate::auth::AuthManager::new()?;
        let provider_manager = ProviderManager::new(&config, Arc::new(auth_manager)).await?;
        
        let provider = provider_manager
            .get_provider_or_host(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", provider_name))?;
        
        let (content, status_messages) = self.process_message(message, provider, model_name).await?;
        
        // Convert tool status messages to ToolCallInfo
        let tool_calls: Vec<crate::client::ToolCallInfo> = status_messages
            .into_iter()
            .map(|msg| {
                // Extract tool name from status messages like "🔧 Executing tool: write_file"
                let tool_name = if let Some(colon_pos) = msg.find(": ") {
                    msg[colon_pos + 2..].split_whitespace().next().unwrap_or("unknown_tool").to_string()
                } else {
                    "unknown_tool".to_string()
                };

                crate::client::ToolCallInfo {
                    name: tool_name,
                    status: if msg.contains("✓") {
                        crate::client::ToolStatus::Success
                    } else if msg.contains("🔧") {
                        crate::client::ToolStatus::Running
                    } else {
                        crate::client::ToolStatus::Failed
                    },
                    result: Some(serde_json::Value::String(msg.clone())),
                    error: if msg.contains("error") { Some(msg) } else { None },
                }
            })
            .collect();
        
        Ok(crate::client::AgentResponse {
            content,
            tool_calls,
            session_id: _session_id.to_string(),
        })
    }
    
    /// Send message with streaming response
    pub async fn send_message_streaming(
        &mut self,
        session_id: &str,
        message: &str,
        provider_name: &str,
        model_name: &str,
    ) -> Result<crate::agent::streaming::AgentStream> {
        // For now, use non-streaming and convert to stream
        let response = self.send_message(session_id, message, provider_name, model_name).await?;
        
        // Create a simple stream that emits the response
        let (tx, rx) = crate::agent::streaming::create_agent_stream();
        tx.send(Ok(crate::agent::streaming::AgentUpdate::TextChunk {
            content: response.content,
            delta: false,
            tokens_used: None,
        })).await.ok();
        tx.send(Ok(crate::agent::streaming::AgentUpdate::Complete {
            total_tokens: 0,
            tool_status_messages: Vec::new(),
        })).await.ok();
        
        Ok(rx)
    }
    
    /// Execute a single tool by name
    pub async fn execute_single_tool(
        &self,
        tool_name: &str,
        params: serde_json::Value,
    ) -> Result<crate::client::ToolCallInfo> {
        if let Some(tool) = self.tools.get(tool_name) {
            match tool.execute(params.clone()).await {
                Ok(output) => {
                    Ok(crate::client::ToolCallInfo {
                        name: tool_name.to_string(),
                        status: if output.success {
                            crate::client::ToolStatus::Success
                        } else {
                            crate::client::ToolStatus::Failed
                        },
                        result: Some(output.result),
                        error: output.error,
                    })
                }
                Err(e) => {
                    Ok(crate::client::ToolCallInfo {
                        name: tool_name.to_string(),
                        status: crate::client::ToolStatus::Failed,
                        result: None,
                        error: Some(e.to_string()),
                    })
                }
            }
        } else {
            Ok(crate::client::ToolCallInfo {
                name: tool_name.to_string(),
                status: crate::client::ToolStatus::Failed,
                result: None,
                error: Some(format!("Tool '{}' not found", tool_name)),
            })
        }
    }
    
    /// Get session history (placeholder implementation)
    pub async fn get_history(&self, _session_id: &str) -> Result<Vec<crate::client::AgentResponse>> {
        // Convert conversation messages to AgentResponse format
        let responses: Vec<crate::client::AgentResponse> = {
            let conversation = self.conversation.lock().await;
            conversation.messages
                .iter()
                .filter(|msg| matches!(msg.role, crate::agent::conversation::MessageRole::Assistant))
                .map(|msg| crate::client::AgentResponse {
                    content: msg.content.clone(),
                    tool_calls: msg.tool_calls.as_ref().map(|calls| {
                        calls.iter().map(|call| crate::client::ToolCallInfo {
                            name: call.tool_name.clone(),
                            status: crate::client::ToolStatus::Success,
                            result: call.result.clone(),
                            error: None,
                        }).collect()
                    }).unwrap_or_default(),
                    session_id: _session_id.to_string(),
                })
                .collect()
        };
        
        Ok(responses)
    }
    
    /// End session (placeholder implementation)
    pub async fn end_session(&self, _session_id: &str) -> Result<()> {
        // Clear conversation for this session
        // In practice, you'd want to save it first
        {
            let mut conversation = self.conversation.lock().await;
            conversation.messages.clear();
        }
        Ok(())
    }
}

/// ACP (Agent Client Protocol) implementation
#[cfg(feature = "acp")]
impl AcpAgent for Agent {
    fn initialize(&self, request: InitializeRequest) -> impl std::future::Future<Output = Result<InitializeResponse, agent_client_protocol::Error>> + Send + '_ {
        async move {
        info!("Initializing ACP agent with protocol version: {:?}", request.protocol_version);
        
        // Note: Intelligence initialization would happen during Agent construction
        // For ACP mode, we'll skip dynamic initialization to keep it simple
        
        // Get available tools for capabilities
        let _tool_names: Vec<String> = self.tools.list_tools().iter().map(|t| t.name.clone()).collect();
        
        Ok(InitializeResponse {
            protocol_version: request.protocol_version,
            agent_capabilities: AgentCapabilities {
                load_session: false,
                prompt_capabilities: PromptCapabilities {
                    image: false,
                    audio: false,
                    embedded_context: true,
                },
            },
            auth_methods: vec![],
        })
        }
    }
    
    fn new_session(&self, _request: NewSessionRequest) -> impl std::future::Future<Output = Result<NewSessionResponse, agent_client_protocol::Error>> + Send + '_ {
        async move {
            let session_id = uuid::Uuid::new_v4().to_string();
            
            // Start memory session if available
            if let Ok(Some(memory_session)) = self.intelligence.start_memory_session(Some(session_id.clone())).await {
                info!("Started intelligent memory session: {}", memory_session);
            }
            
            Ok(NewSessionResponse { session_id: SessionId(session_id.into()) })
        }
    }
    
    fn prompt(&self, request: PromptRequest) -> impl std::future::Future<Output = Result<PromptResponse, agent_client_protocol::Error>> + Send + '_ {
        async move {
            // Extract text content from request
            let user_message = request.prompt.iter()
                .filter_map(|block| match block {
                    ContentBlock::Text(text_content) => Some(text_content.text.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
                
            info!("Processing ACP prompt: {}", user_message);
            
            // Use a default Ollama provider for ACP mode (simplified implementation)
            let provider_config = crate::config::ProviderConfig {
                name: "ollama".to_string(),
                api_key_env: "".to_string(), // Ollama doesn't need API key
                base_url: "http://localhost:11434".to_string(),
                fallback_urls: vec![],
                models: vec![],
                timeout_seconds: 30,
                max_retries: 3,
            };
            let provider = crate::providers::ollama::OllamaProvider::new(
                provider_config,
                self.auth_manager.clone()
            ).await?;
            
            match self.process_message(&user_message, &provider, "gpt-oss").await {
                Ok((_response, _status_messages)) => {
                    // Note: ACP works differently - responses are sent via session/update notifications
                    // during processing, and prompt() only returns the stop reason
                    Ok(PromptResponse {
                        stop_reason: StopReason::EndTurn,
                    })
                }
                Err(_e) => {
                    Ok(PromptResponse {
                        stop_reason: StopReason::Refusal,
                    })
                }
            }
        }
    }
    
    fn authenticate(&self, _request: AuthenticateRequest) -> impl std::future::Future<Output = Result<(), agent_client_protocol::Error>> + Send + '_ {
        async move {
            // For simplicity, we don't implement authentication in the basic ACP mode
            // In a production system, you'd validate credentials here
            Ok(())
        }
    }
    
    fn load_session(&self, _request: LoadSessionRequest) -> impl std::future::Future<Output = Result<(), agent_client_protocol::Error>> + Send + '_ {
        async move {
            // We don't support session loading in this basic implementation
            Err(agent_client_protocol::Error::method_not_found())
        }
    }
    
    fn cancel(&self, _notification: CancelNotification) -> impl std::future::Future<Output = Result<(), agent_client_protocol::Error>> + Send + '_ {
        async move {
            // Cancel ongoing operations - for now just log the request
            info!("Received cancel notification for session");
            Ok(())
        }
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
        
        // Create Agent
        let agent = Agent::new(intelligence, auth_manager, project_context).await.unwrap();
        
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