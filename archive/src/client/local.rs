// LocalClient implementation for direct access to Agent
/// Local client implementation that provides high-performance TUI access to the Agent
///
/// This client:
/// - Creates and manages the core Agent instance
/// - Handles session management
/// - Provides direct function calls (no serialization overhead)
/// - Implements the AgentClient trait for consistency

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use serde_json::Value;

use super::{AgentClient, AgentInfo, AgentResponse, ToolCallInfo};
use crate::agent::core::Agent;
use crate::agent::streaming::AgentStream;
use crate::agent::conversation::{ProjectContext, ProgrammingLanguage};
use crate::auth::AuthManager;
use crate::config::ConfigManager;
use crate::intelligence::IntelligenceEngine;
use crate::providers::ProviderManager;
use crate::storage::DatabaseManager;

/// Local client that directly calls the Agent with session management
pub struct LocalClient {
    agent: Arc<tokio::sync::Mutex<Agent>>,
    session_id: Option<String>,
}

impl LocalClient {
    /// Create a new LocalClient with full Agent setup
    pub async fn new(
        config: &ConfigManager,
        auth_manager: Arc<AuthManager>,
        _provider_manager: Arc<ProviderManager>,
    ) -> Result<Self> {
        // Create intelligence engine
        let db_manager = DatabaseManager::new(config).await?;
        let intelligence = IntelligenceEngine::new(config, &db_manager).await?;

        // Create project context
        let project_context = ProjectContext {
            root_path: std::env::current_dir()?,
            language: ProgrammingLanguage::Rust, // This is a Rust project
            framework: None,
            recent_changes: Vec::new(),
        };

        // Create the core agent
        let agent = Arc::new(tokio::sync::Mutex::new(
            Agent::new(intelligence, auth_manager, project_context).await?
        ));

        Ok(Self {
            agent,
            session_id: None,
        })
    }

    /// Create LocalClient from existing agent (for testing)
    pub fn from_agent(agent: Arc<tokio::sync::Mutex<Agent>>) -> Self {
        Self {
            agent,
            session_id: None,
        }
    }

    /// Create LocalClient with approval system enabled
    pub async fn new_with_approval(
        config: &ConfigManager,
        auth_manager: Arc<AuthManager>,
        _provider_manager: Arc<ProviderManager>,
    ) -> Result<(Self, tokio::sync::mpsc::UnboundedReceiver<crate::agent::approval_modes::PendingChange>)> {
        // Create database manager (needed for intelligence engine)
        use crate::storage::DatabaseManager;
        let storage = DatabaseManager::new(config).await?;

        // Create intelligence engine
        let intelligence = IntelligenceEngine::new(config, &storage).await?;

        // Create project context
        let project_context = ProjectContext {
            root_path: std::env::current_dir().unwrap_or_default(),
            language: crate::agent::conversation::ProgrammingLanguage::Other("Mixed".to_string()),
            framework: None,
            recent_changes: Vec::new(),
        };

        // Create the core agent with approval support
        let (agent, approval_rx) = Agent::new_with_approval(intelligence, auth_manager, project_context).await?;
        let agent = Arc::new(tokio::sync::Mutex::new(agent));

        let client = Self {
            agent,
            session_id: None,
        };

        Ok((client, approval_rx))
    }

    /// Initialize a session for this client
    pub async fn init_session(&mut self) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        self.session_id = Some(session_id.clone());
        Ok(session_id)
    }

    /// Send message using the current session
    pub async fn send_message(&self, message: &str) -> Result<AgentResponse> {
        let session_id = self.session_id.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No active session - call create_session() first"))?;

        self.send_prompt(session_id, message.to_string(), None, None).await
    }

    /// Send streaming message using the current session
    pub async fn stream_message(&self, message: &str) -> Result<AgentStream> {
        let session_id = self.session_id.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No active session - call create_session() first"))?;

        self.send_prompt_streaming(session_id, message.to_string(), None, None).await
    }

    /// Get current session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Execute a single tool directly
    pub async fn execute_tool(&self, tool_name: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let agent = self.agent.lock().await;
        let result = agent.execute_single_tool(tool_name, params).await?;
        Ok(serde_json::json!({
            "status": result.status,
            "result": result.result,
            "error": result.error,
        }))
    }
}

#[async_trait]
impl AgentClient for LocalClient {
    async fn initialize(&self) -> Result<AgentInfo> {
        // Get agent capabilities directly
        let agent = self.agent.lock().await;
        let tools = agent.list_tools().await?;

        Ok(AgentInfo {
            name: "Aircher Agent".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: vec!["tool_calling".to_string(), "streaming".to_string()],
            available_tools: tools,
        })
    }

    async fn create_session(&self) -> Result<String> {
        // For AgentClient trait - just generate UUID
        // The actual session is managed by the mutable methods above
        Ok(uuid::Uuid::new_v4().to_string())
    }

    async fn send_prompt(
        &self,
        session_id: &str,
        message: String,
        provider: Option<String>,
        model: Option<String>,
    ) -> Result<AgentResponse> {
        // Get default provider if none specified
        let provider_name = provider.unwrap_or_else(|| "ollama".to_string());
        let model_name = model.unwrap_or_else(|| "gpt-oss".to_string());

        // Direct call to agent
        let mut agent = self.agent.lock().await;
        let response = agent.send_message(session_id, &message, &provider_name, &model_name).await?;

        Ok(AgentResponse {
            content: response.content,
            tool_calls: response.tool_calls,
            session_id: session_id.to_string(),
        })
    }

    async fn send_prompt_streaming(
        &self,
        session_id: &str,
        message: String,
        provider: Option<String>,
        model: Option<String>,
    ) -> Result<AgentStream> {
        // Get default provider if none specified
        let provider_name = provider.unwrap_or_else(|| "ollama".to_string());
        let model_name = model.unwrap_or_else(|| "gpt-oss".to_string());

        // Create streaming response
        let mut agent = self.agent.lock().await;
        let stream = agent.send_message_streaming(session_id, &message, &provider_name, &model_name).await?;
        Ok(stream)
    }

    async fn execute_tool(
        &self,
        tool_name: String,
        params: Value,
    ) -> Result<ToolCallInfo> {
        // Direct tool execution
        let agent = self.agent.lock().await;
        let result = agent.execute_single_tool(&tool_name, params).await?;
        Ok(result)
    }

    async fn get_session_history(&self, session_id: &str) -> Result<Vec<AgentResponse>> {
        // Get session history
        let agent = self.agent.lock().await;
        agent.get_history(session_id).await
    }

    async fn end_session(&self, session_id: &str) -> Result<()> {
        // End session
        let agent = self.agent.lock().await;
        agent.end_session(session_id).await
    }
}
