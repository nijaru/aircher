/// Agent integration module for TUI
/// 
/// This module bridges the TUI with the UnifiedAgent through the LocalClient,
/// replacing the old AgentController approach.

use anyhow::Result;
use std::sync::Arc;

use crate::agent::unified::UnifiedAgent;
use crate::client::local::LocalClient;
use crate::client::{AgentClient, AgentResponse};
use crate::agent::streaming::AgentStream;
use crate::auth::AuthManager;
use crate::config::ConfigManager;
use crate::intelligence::IntelligenceEngine;
use crate::providers::ProviderManager;
use crate::agent::conversation::ProjectContext;
use crate::storage::DatabaseManager;

/// Manages the agent connection for the TUI
pub struct AgentIntegration {
    /// The client used to communicate with the agent
    client: Arc<dyn AgentClient>,
    
    /// Current session ID
    session_id: String,
    
    /// The underlying agent (kept for provider updates)
    agent: Arc<UnifiedAgent>,
}

impl AgentIntegration {
    /// Create a new agent integration for the TUI
    pub async fn new(
        config: &ConfigManager,
        auth_manager: Arc<AuthManager>,
        project_context: ProjectContext,
    ) -> Result<Self> {
        // Initialize the database manager
        let db_manager = DatabaseManager::new(config).await?;
        
        // Create the intelligence engine
        let intelligence = IntelligenceEngine::new(config, &db_manager).await?;
        
        // Create the unified agent (single implementation for all modes)
        let agent = Arc::new(
            UnifiedAgent::new(intelligence, auth_manager.clone(), project_context).await?
        );
        
        // Create the local client for TUI (direct access, no JSON-RPC)
        let client = Arc::new(LocalClient::new(agent.clone()));
        
        // Initialize the agent
        let _info = client.initialize().await?;
        
        // Create a session
        let session_id = client.create_session().await?;
        
        Ok(Self {
            client,
            session_id,
            agent,
        })
    }
    
    /// Set the provider manager (called after providers are initialized)
    pub async fn set_provider_manager(&self, provider_manager: ProviderManager) {
        self.agent.set_provider_manager(provider_manager).await;
    }
    
    /// Send a message to the agent
    pub async fn send_message(
        &self,
        message: String,
        provider: Option<String>,
        model: Option<String>,
    ) -> Result<AgentResponse> {
        self.client.send_prompt(
            &self.session_id,
            message,
            provider,
            model,
        ).await
    }
    
    /// Send a message to the agent with streaming response
    pub async fn send_message_streaming(
        &self,
        message: String,
        provider: Option<String>,
        model: Option<String>,
    ) -> Result<AgentStream> {
        self.client.send_prompt_streaming(
            &self.session_id,
            message,
            provider,
            model,
        ).await
    }
    
    /// Get the current session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
    
    /// Get session history
    pub async fn get_history(&self) -> Result<Vec<AgentResponse>> {
        self.client.get_session_history(&self.session_id).await
    }
    
    /// End the current session
    pub async fn end_session(&self) -> Result<()> {
        self.client.end_session(&self.session_id).await
    }
}

/// Helper to migrate from old AgentController to new AgentIntegration
pub mod migration {
    use super::*;
    
    /// Convert old AgentController initialization to new AgentIntegration
    pub async fn from_agent_controller_params(
        config: &ConfigManager,
        auth_manager: Arc<AuthManager>,
        project_context: ProjectContext,
    ) -> Result<AgentIntegration> {
        // This is what TuiManager would call instead of creating AgentController
        AgentIntegration::new(config, auth_manager, project_context).await
    }
}