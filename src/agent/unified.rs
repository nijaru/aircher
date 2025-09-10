/// Unified agent implementation that serves both TUI and ACP modes
/// 
/// This is the single source of truth for agent behavior. The TUI accesses
/// this through LocalClient, while editors access it through ACP protocol.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[cfg(feature = "acp")]
use agent_client_protocol::{
    Agent, Error as AcpError,
    InitializeRequest, InitializeResponse, AgentCapabilities,
    NewSessionRequest, NewSessionResponse, SessionId,
    LoadSessionRequest, 
    PromptRequest, PromptResponse, StopReason,
    AuthenticateRequest, CancelNotification,
};

use crate::agent::tools::{ToolRegistry, ToolCall};
use crate::agent::streaming::{AgentStreamSender, AgentUpdate};
use crate::agent::parser::ToolCallParser;
use crate::agent::conversation::{CodingConversation, ProjectContext};
use crate::intelligence::IntelligenceEngine;
use crate::intelligence::tools::IntelligenceTools;
use crate::providers::ProviderManager;
use crate::auth::AuthManager;

/// Session state for a conversation
pub struct Session {
    pub id: String,
    pub conversation: CodingConversation,
    pub active: bool,
}

/// The unified agent that implements core logic once
pub struct UnifiedAgent {
    /// Tool registry for executing tools
    pub tools: Arc<ToolRegistry>,
    
    /// Intelligence engine for enhanced responses
    pub intelligence: Arc<IntelligenceEngine>,
    
    /// Provider manager for LLM access
    pub providers: Arc<RwLock<Option<ProviderManager>>>,
    
    /// Authentication manager
    pub auth_manager: Arc<AuthManager>,
    
    /// Tool call parser
    pub parser: ToolCallParser,
    
    /// Active sessions
    pub sessions: Arc<RwLock<HashMap<String, Session>>>,
    
    /// Project context
    pub project_context: ProjectContext,
}

impl UnifiedAgent {
    pub async fn new(
        intelligence: IntelligenceEngine,
        auth_manager: Arc<AuthManager>,
        project_context: ProjectContext,
    ) -> Result<Self> {
        Ok(Self {
            tools: Arc::new(ToolRegistry::default()),
            intelligence: Arc::new(intelligence),
            providers: Arc::new(RwLock::new(None)),
            auth_manager,
            parser: ToolCallParser::new()?,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            project_context,
        })
    }
    
    /// Set the provider manager (called after initialization)
    pub async fn set_provider_manager(&self, provider_manager: ProviderManager) {
        let mut providers = self.providers.write().await;
        *providers = Some(provider_manager);
    }
    
    /// Create a new session
    pub async fn create_session(&self, session_id: String) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        let session = Session {
            id: session_id.clone(),
            conversation: CodingConversation {
                messages: Vec::new(),
                project_context: self.project_context.clone(),
                active_files: Vec::new(),
                task_list: Vec::new(),
            },
            active: true,
        };
        
        sessions.insert(session_id, session);
        Ok(())
    }
    
    /// Process a prompt in a session
    pub async fn process_prompt(
        &self,
        session_id: &str,
        prompt: String,
        provider_name: Option<&str>,
        model_name: Option<&str>,
    ) -> Result<String> {
        info!("Processing prompt for session {}: {}", session_id, prompt);
        
        // Get or create session
        {
            let sessions = self.sessions.read().await;
            if !sessions.contains_key(session_id) {
                drop(sessions);
                self.create_session(session_id.to_string()).await?;
            }
        }
        
        // Add user message to conversation
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.conversation.messages.push(crate::agent::conversation::Message {
                    role: crate::agent::conversation::MessageRole::User,
                    content: prompt.clone(),
                    tool_calls: None,
                    timestamp: chrono::Utc::now(),
                });
            }
        }
        
        // Check for tool calls in the prompt
        let tool_calls = self.parser.parse(&prompt);
        
        if !tool_calls.is_empty() {
            // Execute tools
            let mut results = Vec::new();
            for tool_call in &tool_calls {
                debug!("Executing tool: {}", tool_call.name);
                if let Some(tool) = self.tools.get(&tool_call.name) {
                    match tool.execute(tool_call.parameters.clone()).await {
                        Ok(result) => {
                            results.push(format!("Tool '{}' executed successfully: {:?}", 
                                tool_call.name, result));
                        }
                        Err(e) => {
                            warn!("Tool execution failed: {}", e);
                            results.push(format!("Tool '{}' failed: {}", tool_call.name, e));
                        }
                    }
                } else {
                    results.push(format!("Tool '{}' not found", tool_call.name));
                }
            }
            
            // Add assistant response with tool results
            let response = results.join("\n");
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                // Convert agent::tools::ToolCall to agent::conversation::ToolCall
                let conversation_tool_calls: Vec<crate::agent::conversation::ToolCall> = tool_calls.iter().map(|tc| {
                    crate::agent::conversation::ToolCall {
                        tool_name: tc.name.clone(),
                        parameters: tc.parameters.clone(),
                        result: None, // Will be populated after execution
                    }
                }).collect();
                
                session.conversation.messages.push(crate::agent::conversation::Message {
                    role: crate::agent::conversation::MessageRole::Assistant,
                    content: response.clone(),
                    tool_calls: Some(conversation_tool_calls),
                    timestamp: chrono::Utc::now(),
                });
            }
            
            return Ok(response);
        }
        
        // If no tools, generate an intelligence-enhanced response
        let response = self.create_intelligence_enhanced_response(
            &prompt, 
            provider_name, 
            model_name
        ).await?;
        
        // Add assistant response
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.conversation.messages.push(crate::agent::conversation::Message {
                    role: crate::agent::conversation::MessageRole::Assistant,
                    content: response.clone(),
                    tool_calls: None,
                    timestamp: chrono::Utc::now(),
                });
            }
        }
        
        Ok(response)
    }
    
    /// Execute a tool directly
    pub async fn execute_tool(&self, tool_call: &ToolCall) -> Result<serde_json::Value> {
        if let Some(tool) = self.tools.get(&tool_call.name) {
            tool.execute(tool_call.parameters.clone()).await
                .map(|output| output.result)
                .map_err(|e| anyhow::anyhow!("Tool execution failed: {}", e))
        } else {
            Err(anyhow::anyhow!("Tool '{}' not found", tool_call.name))
        }
    }
    
    /// Process a prompt with streaming updates
    pub async fn process_prompt_streaming(
        &self,
        session_id: &str,
        prompt: String,
        provider_name: Option<&str>,
        model_name: Option<&str>,
        tx: AgentStreamSender,
    ) -> Result<()> {
        info!("Processing streaming prompt for session {}: {}", session_id, prompt);
        
        // Get or create session
        {
            let sessions = self.sessions.read().await;
            if !sessions.contains_key(session_id) {
                drop(sessions);
                self.create_session(session_id.to_string()).await?;
            }
        }
        
        // Add user message to conversation
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.conversation.messages.push(crate::agent::conversation::Message {
                    role: crate::agent::conversation::MessageRole::User,
                    content: prompt.clone(),
                    tool_calls: None,
                    timestamp: chrono::Utc::now(),
                });
            }
        }
        
        // Check for tool calls in the prompt
        let tool_calls = self.parser.parse(&prompt);
        
        if !tool_calls.is_empty() {
            // Execute tools with streaming updates
            let mut results = Vec::new();
            for tool_call in &tool_calls {
                debug!("Executing tool: {}", tool_call.name);
                
                // Send tool status update
                let _ = tx.send(Ok(AgentUpdate::ToolStatus(
                    format!("🔧 {} — running...", tool_call.name)
                ))).await;
                
                if let Some(tool) = self.tools.get(&tool_call.name) {
                    match tool.execute(tool_call.parameters.clone()).await {
                        Ok(_result) => {
                            let summary = format!("✓ {} — completed", tool_call.name);
                            let _ = tx.send(Ok(AgentUpdate::ToolStatus(summary.clone()))).await;
                            results.push(summary);
                        }
                        Err(e) => {
                            warn!("Tool execution failed: {}", e);
                            let error_msg = format!("✗ {} — failed: {}", tool_call.name, e);
                            let _ = tx.send(Ok(AgentUpdate::ToolStatus(error_msg.clone()))).await;
                            results.push(error_msg);
                        }
                    }
                } else {
                    let error_msg = format!("✗ {} — not found", tool_call.name);
                    let _ = tx.send(Ok(AgentUpdate::ToolStatus(error_msg.clone()))).await;
                    results.push(error_msg);
                }
            }
            
            // Add assistant response with tool results
            let response = results.join("\n");
            {
                let mut sessions = self.sessions.write().await;
                if let Some(session) = sessions.get_mut(session_id) {
                    // Convert agent::tools::ToolCall to agent::conversation::ToolCall
                    let conversation_tool_calls: Vec<crate::agent::conversation::ToolCall> = tool_calls.iter().map(|tc| {
                        crate::agent::conversation::ToolCall {
                            tool_name: tc.name.clone(),
                            parameters: tc.parameters.clone(),
                            result: None, // Will be populated after execution
                        }
                    }).collect();
                    
                    session.conversation.messages.push(crate::agent::conversation::Message {
                        role: crate::agent::conversation::MessageRole::Assistant,
                        content: response.clone(),
                        tool_calls: Some(conversation_tool_calls),
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
            
            // Send final text chunk
            let _ = tx.send(Ok(AgentUpdate::TextChunk {
                content: response,
                delta: false,
                tokens_used: Some(0), // TODO: Track actual tokens
            })).await;
            
            // Send completion
            let _ = tx.send(Ok(AgentUpdate::Complete {
                total_tokens: 0, // TODO: Track actual tokens
                tool_status_messages: results,
            })).await;
            
            return Ok(());
        }
        
        // If no tools, generate an intelligence-enhanced response (streaming)
        let response = self.create_intelligence_enhanced_response(
            &prompt,
            provider_name,
            model_name
        ).await?;
        
        // Send response as text chunk
        let _ = tx.send(Ok(AgentUpdate::TextChunk {
            content: response.clone(),
            delta: false,
            tokens_used: Some(0), // TODO: Track actual tokens
        })).await;
        
        // Add assistant response
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.conversation.messages.push(crate::agent::conversation::Message {
                    role: crate::agent::conversation::MessageRole::Assistant,
                    content: response.clone(),
                    tool_calls: None,
                    timestamp: chrono::Utc::now(),
                });
            }
        }
        
        // Send completion
        let _ = tx.send(Ok(AgentUpdate::Complete {
            total_tokens: 0, // TODO: Track actual tokens
            tool_status_messages: vec![],
        })).await;
        
        Ok(())
    }
    
    /// Create an intelligence-enhanced response using the IntelligenceEngine
    async fn create_intelligence_enhanced_response(
        &self,
        prompt: &str,
        provider_name: Option<&str>,
        model_name: Option<&str>,
    ) -> Result<String> {
        // Use intelligence to get contextual insights
        let context = self.intelligence.get_development_context(prompt).await;
        
        // Create enhanced response with intelligence insights
        let mut response = String::new();
        
        // Add intelligent analysis
        if !context.active_story.is_empty() {
            response.push_str(&format!("📋 **Current Focus**: {}\\n\\n", context.active_story));
        }
        
        if !context.development_phase.is_empty() {
            response.push_str(&format!("🔄 **Development Phase**: {}\\n\\n", context.development_phase));
        }
        
        // Add relevant files context
        if !context.key_files.is_empty() {
            response.push_str("📁 **Relevant Files**:\\n");
            for file in context.key_files.iter().take(3) {  // Show top 3 relevant files
                response.push_str(&format!("- `{}`\\n", file.path));
            }
            response.push_str("\\n");
        }
        
        // Add suggested actions
        if !context.suggested_next_actions.is_empty() {
            response.push_str("🎯 **Suggested Next Actions**:\\n");
            for action in context.suggested_next_actions.iter().take(3) {  // Show top 3 actions
                response.push_str(&format!("- {}\\n", action.description));
            }
            response.push_str("\\n");
        }
        
        // Add recent patterns if available
        if !context.recent_patterns.is_empty() {
            response.push_str("🧠 **Learned Patterns**:\\n");
            for pattern in context.recent_patterns.iter().take(2) {  // Show top 2 patterns
                response.push_str(&format!("- {}\\n", pattern.description));
            }
            response.push_str("\\n");
        }
        
        // Add confidence indicator
        let confidence_emoji = if context.confidence > 0.8 {
            "🎯"
        } else if context.confidence > 0.6 {
            "📊"  
        } else {
            "🤔"
        };
        
        response.push_str(&format!(
            "{} **Intelligence Analysis** (confidence: {:.1}%)\\n\\n",
            confidence_emoji,
            context.confidence * 100.0
        ));
        
        // Add original query context
        response.push_str(&format!("**Your Query**: {}\\n\\n", prompt));
        
        // Add provider info if available
        if let Some(provider) = provider_name {
            response.push_str(&format!(
                "Ready to process with **{}** using model **{}**.\\n",
                provider,
                model_name.unwrap_or("default")
            ));
        } else {
            response.push_str("Ready to assist with your development work.\\n");
        }
        
        Ok(response)
    }
}

// ACP Agent trait implementation (only when ACP feature is enabled)
#[cfg(feature = "acp")]
#[async_trait]
impl Agent for UnifiedAgent {
    async fn authenticate(&self, _request: AuthenticateRequest) -> Result<(), AcpError> {
        debug!("ACP authenticate request");
        Ok(())
    }
    
    async fn initialize(&self, _request: InitializeRequest) -> Result<InitializeResponse, AcpError> {
        info!("ACP initialize request");
        
        Ok(InitializeResponse {
            agent_capabilities: AgentCapabilities::default(),
            auth_methods: Vec::new(),
            protocol_version: agent_client_protocol::V1,
        })
    }
    
    async fn new_session(&self, _request: NewSessionRequest) -> Result<NewSessionResponse, AcpError> {
        info!("ACP new_session request");
        
        let session_id = uuid::Uuid::new_v4().to_string();
        self.create_session(session_id.clone()).await
            .map_err(|e| AcpError {
                code: agent_client_protocol::ErrorCode::INTERNAL_ERROR,
                message: e.to_string(),
                data: None,
            })?;
        
        Ok(NewSessionResponse {
            session_id: SessionId(session_id.into()),
        })
    }
    
    async fn load_session(&self, _request: LoadSessionRequest) -> Result<(), AcpError> {
        info!("ACP load_session request");
        // TODO: Implement session persistence
        Ok(())
    }
    
    async fn prompt(&self, request: PromptRequest) -> Result<PromptResponse, AcpError> {
        info!("ACP prompt request for session: {:?}", request.session_id);
        
        // Convert ACP prompt to string
        // TODO: Handle different content types properly
        let prompt_text = format!("ACP prompt for session {:?}", request.session_id);
        
        // Process through unified logic
        let response = self.process_prompt(
            &request.session_id.0,
            prompt_text,
            None,
            None,
        ).await.map_err(|e| AcpError {
            code: agent_client_protocol::ErrorCode::INTERNAL_ERROR,
            message: e.to_string(),
            data: None,
        })?;
        
        debug!("Generated response: {}", response);
        
        Ok(PromptResponse {
            stop_reason: StopReason::EndTurn,
        })
    }
    
    async fn cancel(&self, _notification: CancelNotification) -> Result<(), AcpError> {
        info!("ACP cancel notification");
        // TODO: Implement cancellation
        Ok(())
    }
}