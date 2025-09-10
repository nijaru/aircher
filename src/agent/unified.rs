/// Unified agent implementation that serves both TUI and ACP modes
/// 
/// This is the single source of truth for agent behavior. The TUI accesses
/// this through LocalClient, while editors access it through ACP protocol.

use anyhow::Result;
use async_trait::async_trait;
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
use crate::agent::parser::ToolCallParser;
use crate::agent::conversation::{CodingConversation, ProjectContext};
use crate::intelligence::IntelligenceEngine;
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
        let tool_calls = self.parser.parse_tool_calls(&prompt)?;
        
        if !tool_calls.is_empty() {
            // Execute tools
            let mut results = Vec::new();
            for tool_call in tool_calls {
                debug!("Executing tool: {}", tool_call.name);
                match self.tools.execute(&tool_call.name, tool_call.params.clone()).await {
                    Ok(result) => {
                        results.push(format!("Tool '{}' executed successfully: {:?}", 
                            tool_call.name, result));
                    }
                    Err(e) => {
                        warn!("Tool execution failed: {}", e);
                        results.push(format!("Tool '{}' failed: {}", tool_call.name, e));
                    }
                }
            }
            
            // Add assistant response with tool results
            let response = results.join("\n");
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.conversation.messages.push(crate::agent::conversation::Message {
                    role: crate::agent::conversation::MessageRole::Assistant,
                    content: response.clone(),
                    tool_calls: Some(tool_calls),
                    timestamp: chrono::Utc::now(),
                });
            }
            
            return Ok(response);
        }
        
        // If no tools, generate a response
        // TODO: Use provider and model to generate actual LLM response
        let response = if let Some(provider) = provider_name {
            format!("Processing with provider '{}' and model '{}':\n{}", 
                provider, 
                model_name.unwrap_or("default"),
                prompt)
        } else {
            format!("Echo response: {}", prompt)
        };
        
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
        self.tools.execute(&tool_call.name, tool_call.params.clone()).await
            .map(|output| output.result)
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