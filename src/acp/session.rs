use agent_client_protocol::SessionId;

/// Simple session manager for ACP mode
/// 
/// For now, just tracks session IDs. More sophisticated session
/// management will be added as we develop the ACP integration.
pub struct AcpSessionManager {
    _sessions: Vec<SessionId>,
}

impl AcpSessionManager {
    pub fn new() -> Self {
        Self {
            _sessions: Vec::new(),
        }
    }

    /// Track a new session
    pub fn add_session(&mut self, session_id: SessionId) {
        self._sessions.push(session_id);
    }

    /// Get active session count
    pub fn session_count(&self) -> usize {
        self._sessions.len()
    }
}