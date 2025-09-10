use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use agent_client_protocol::SessionId;

use crate::auth::AuthManager;
use crate::config::ConfigManager;
use crate::agent::{AgentController, conversation::ProjectContext};
use crate::intelligence::IntelligenceEngine;
use crate::providers::ProviderManager;

/// Session manager for ACP mode
/// 
/// Manages multiple concurrent agent sessions, each with their own
/// conversation state and context. This allows the same Aircher agent
/// to handle multiple simultaneous sessions from the editor.
pub struct AcpSessionManager {
    sessions: HashMap<SessionId, AgentController>,
}

impl AcpSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Get existing session or create new one
    pub async fn get_or_create_session(
        &mut self,
        session_id: &SessionId,
        config: &ConfigManager,
        auth_manager: Arc<AuthManager>,
        providers: Option<Arc<ProviderManager>>,
        project_context: ProjectContext,
    ) -> Result<&mut AgentController> {
        if !self.sessions.contains_key(session_id) {
            // Create new session
            let intelligence = IntelligenceEngine::new()?;
            let agent_controller = AgentController::new(
                intelligence,
                auth_manager,
                project_context,
            )?;

            self.sessions.insert(session_id.clone(), agent_controller);
        }

        Ok(self.sessions.get_mut(session_id).unwrap())
    }

    /// Remove a session (cleanup when editor closes)
    pub fn remove_session(&mut self, session_id: &SessionId) -> Option<AgentController> {
        self.sessions.remove(session_id)
    }

    /// Get active session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get all session IDs
    pub fn session_ids(&self) -> Vec<&SessionId> {
        self.sessions.keys().collect()
    }
}