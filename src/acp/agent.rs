use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{debug, info, warn};

use agent_client_protocol::{
    Agent, AgentSideConnection, 
    InitializeRequest, InitializeResponse,
    PromptRequest, PromptResponse,
    SessionId, ToolCall as AcpToolCall, ToolResult,
};

use crate::auth::AuthManager;
use crate::config::ConfigManager;
use crate::intelligence::IntelligenceEngine;
use crate::agent::{AgentController, conversation::ProjectContext};
use crate::providers::ProviderManager;
use super::session::AcpSessionManager;

/// Aircher Agent Client Protocol implementation
/// 
/// This agent can run in any ACP-compatible editor (Zed, VS Code future, etc.)
/// while maintaining our core advantages: model selection transparency,
/// multi-provider support, and intelligent tool execution.
pub struct AircherAcpAgent {
    config: ConfigManager,
    auth_manager: Arc<AuthManager>,
    providers: Option<Arc<ProviderManager>>,
    intelligence: IntelligenceEngine,
    session_manager: AcpSessionManager,
    project_context: ProjectContext,
}

impl AircherAcpAgent {
    pub async fn new(
        config: ConfigManager,
        auth_manager: Arc<AuthManager>,
        intelligence: IntelligenceEngine,
        project_context: ProjectContext,
    ) -> Result<Self> {
        // Initialize provider manager if auth is available
        let providers = match ProviderManager::new(&config, auth_manager.clone()).await {
            Ok(pm) => Some(Arc::new(pm)),
            Err(e) => {
                warn!("Failed to initialize providers in ACP mode: {}", e);
                None // Still work in demo mode
            }
        };

        let session_manager = AcpSessionManager::new();

        Ok(Self {
            config,
            auth_manager,
            providers,
            intelligence,
            session_manager,
            project_context,
        })
    }

    pub async fn run<R, W>(&self, reader: R, writer: W) -> Result<()> 
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        info!("Starting Aircher ACP agent");

        // Clone self for the connection (ACP requires owned values)
        let agent = AircherAcpAgentRunner {
            inner: self.clone_for_runner().await?,
        };

        let connection = AgentSideConnection::new(reader, writer, agent);
        connection.run().await?;

        Ok(())
    }

    async fn clone_for_runner(&self) -> Result<AircherAcpAgent> {
        Ok(AircherAcpAgent {
            config: self.config.clone(),
            auth_manager: self.auth_manager.clone(),
            providers: self.providers.clone(),
            intelligence: IntelligenceEngine::new()?, // Fresh instance for ACP
            session_manager: AcpSessionManager::new(),
            project_context: self.project_context.clone(),
        })
    }
}

/// Wrapper for ACP Agent trait implementation
/// (Needed because ACP requires owned values in some places)
struct AircherAcpAgentRunner {
    inner: AircherAcpAgent,
}

#[async_trait]
impl Agent for AircherAcpAgentRunner {
    async fn initialize(&mut self, request: InitializeRequest) -> Result<InitializeResponse> {
        info!("ACP Initialize request: {:?}", request);

        // TODO: Process initialization parameters
        // - Model preferences from editor
        // - Project context
        // - User settings

        Ok(InitializeResponse {
            protocol_version: "0.1.1".to_string(),
            capabilities: vec![
                "model_selection".to_string(),
                "multi_provider".to_string(),
                "cost_tracking".to_string(),
                "tool_execution".to_string(),
            ],
            server_info: Some(agent_client_protocol::ServerInfo {
                name: "Aircher".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            }),
        })
    }

    async fn prompt(&mut self, request: PromptRequest) -> Result<PromptResponse> {
        info!("ACP Prompt request for session: {:?}", request.session_id);
        debug!("Prompt content: {}", request.prompt);

        // Get or create session
        let session_id = request.session_id;
        let agent_controller = self.inner.session_manager
            .get_or_create_session(
                &session_id,
                &self.inner.config,
                self.inner.auth_manager.clone(),
                self.inner.providers.clone(),
                self.inner.project_context.clone(),
            ).await?;

        // Process the prompt using our existing agent logic
        // TODO: Implement streaming support for ACP
        let (response_text, tool_results) = agent_controller
            .process_message(
                &request.prompt,
                // TODO: Get provider from session preferences or config
                todo!("Get provider from session/config"),
                // TODO: Get model from session preferences or config  
                todo!("Get model from session/config"),
            ).await?;

        // Convert our tool results to ACP format
        let acp_tool_calls: Vec<AcpToolCall> = tool_results.into_iter()
            .enumerate()
            .map(|(i, tool_name)| {
                // TODO: Convert from our internal tool format
                AcpToolCall {
                    id: format!("call_{}", i),
                    name: tool_name,
                    parameters: serde_json::Value::Object(serde_json::Map::new()),
                }
            })
            .collect();

        Ok(PromptResponse {
            response: response_text,
            tool_calls: if acp_tool_calls.is_empty() { None } else { Some(acp_tool_calls) },
            tool_results: None, // TODO: Implement tool result passing
        })
    }

    async fn tool_result(&mut self, session_id: SessionId, tool_result: ToolResult) -> Result<()> {
        info!("ACP Tool result for session: {:?}", session_id);
        
        // TODO: Pass tool result back to agent controller
        // This would continue the conversation loop after tool execution

        Ok(())
    }
}