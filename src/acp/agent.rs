use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{info, warn};

use agent_client_protocol::{
    Agent, AgentSideConnection, Error as AcpError,
    InitializeRequest, InitializeResponse, AgentCapabilities,
    PromptRequest, PromptResponse, StopReason,
    NewSessionRequest, NewSessionResponse,
    LoadSessionRequest, CancelNotification,
    AuthenticateRequest, SessionId,
};

use crate::auth::AuthManager;
use crate::config::ConfigManager;
use crate::intelligence::IntelligenceEngine;
use crate::agent::conversation::ProjectContext;
use crate::providers::ProviderManager;

/// Simple Aircher ACP Agent implementation
pub struct AircherAcpAgent {
    _config: ConfigManager,
    _auth_manager: Arc<AuthManager>,
    _providers: Option<Arc<ProviderManager>>,
    _intelligence: IntelligenceEngine,
    _project_context: ProjectContext,
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
                None
            }
        };

        Ok(Self {
            _config: config,
            _auth_manager: auth_manager,
            _providers: providers,
            _intelligence: intelligence,
            _project_context: project_context,
        })
    }

    pub async fn run<R, W>(&self, reader: R, writer: W) -> Result<()> 
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        info!("Starting Aircher ACP agent");
        
        let (connection, _task) = AgentSideConnection::new(self, writer);
        connection.run(reader).await.map_err(|e| anyhow::anyhow!("ACP connection error: {}", e))?;
        
        Ok(())
    }
}

#[async_trait]
impl Agent for AircherAcpAgent {
    async fn authenticate(&self, _request: AuthenticateRequest) -> Result<(), AcpError> {
        info!("ACP authenticate request");
        // TODO: Implement authentication
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
        
        let session_id = SessionId(uuid::Uuid::new_v4().to_string().into());
        
        Ok(NewSessionResponse {
            session_id,
        })
    }

    async fn load_session(&self, _request: LoadSessionRequest) -> Result<(), AcpError> {
        info!("ACP load_session request");
        // TODO: Implement session loading
        Ok(())
    }

    async fn prompt(&self, request: PromptRequest) -> Result<PromptResponse, AcpError> {
        info!("ACP prompt request for session: {:?}", request.session_id);
        
        // Simple implementation - just acknowledge receipt for now
        // TODO: Implement actual prompt processing
        let _response_text = format!("Aircher received prompt for session: {:?}", request.session_id);

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