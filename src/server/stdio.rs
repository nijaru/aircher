//! Stdio-based ACP server for editor integration

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use serde_json;

use agent_client_protocol::{Agent as AcpAgent, InitializeRequest, NewSessionRequest, PromptRequest, ContentBlock};
use crate::agent::core::Agent;
use crate::intelligence::IntelligenceEngine;
use crate::auth::AuthManager;
use crate::config::ConfigManager;
use crate::storage::DatabaseManager;
use crate::agent::conversation::{ProjectContext, ProgrammingLanguage};

/// Session timeout (30 minutes of inactivity)
const SESSION_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// Session state tracking conversation history and metadata
#[derive(Debug, Clone)]
struct SessionState {
    /// Unique session identifier
    session_id: String,
    /// Conversation history (user messages + assistant responses)
    messages: Vec<ConversationMessage>,
    /// Last activity timestamp
    last_activity: Instant,
    /// Session creation timestamp
    created_at: Instant,
}

/// A single message in the conversation history
#[derive(Debug, Clone)]
struct ConversationMessage {
    /// Role: "user" or "assistant"
    role: String,
    /// Message content
    content: String,
    /// Timestamp when message was added
    timestamp: Instant,
}

impl SessionState {
    /// Create a new session
    fn new(session_id: String) -> Self {
        let now = Instant::now();
        Self {
            session_id,
            messages: Vec::new(),
            last_activity: now,
            created_at: now,
        }
    }

    /// Add a user message to conversation history
    fn add_user_message(&mut self, content: String) {
        self.messages.push(ConversationMessage {
            role: "user".to_string(),
            content,
            timestamp: Instant::now(),
        });
        self.last_activity = Instant::now();
    }

    /// Add an assistant response to conversation history
    fn add_assistant_message(&mut self, content: String) {
        self.messages.push(ConversationMessage {
            role: "assistant".to_string(),
            content,
            timestamp: Instant::now(),
        });
        self.last_activity = Instant::now();
    }

    /// Check if session has timed out (30 minutes of inactivity)
    fn is_expired(&self) -> bool {
        self.last_activity.elapsed() > SESSION_TIMEOUT
    }

    /// Get conversation history for context
    fn get_history(&self, last_n: Option<usize>) -> Vec<(String, String)> {
        let messages = if let Some(n) = last_n {
            self.messages.iter().rev().take(n).rev().collect()
        } else {
            self.messages.iter().collect()
        };

        messages
            .iter()
            .map(|m| (m.role.clone(), m.content.clone()))
            .collect()
    }
}

/// ACP Server that communicates over stdio (JSON-RPC)
pub struct AcpServer {
    agent: Arc<tokio::sync::Mutex<Agent>>,
    /// Active sessions (SessionId -> SessionState)
    sessions: Arc<tokio::sync::Mutex<HashMap<String, SessionState>>>,
}

impl AcpServer {
    /// Create a new ACP server with Agent
    pub async fn new() -> Result<Self> {
        let config = ConfigManager::load().await?;
        let db_manager = DatabaseManager::new(&config).await?;
        let intelligence = IntelligenceEngine::new(&config, &db_manager).await?;
        let auth_manager = Arc::new(AuthManager::new()?);

        let project_context = ProjectContext {
            root_path: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/")),
            language: ProgrammingLanguage::Other("Mixed".to_string()), // Auto-detected later
            framework: None, // Will be detected
            recent_changes: Vec::new(),
        };

        let agent = Agent::new(intelligence, auth_manager, project_context).await?;

        Ok(Self {
            agent: Arc::new(tokio::sync::Mutex::new(agent)),
            sessions: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        })
    }

    /// Create a new session and store it
    async fn create_session(&self, session_id: String) -> Result<()> {
        let session_state = SessionState::new(session_id.clone());
        let mut sessions = self.sessions.lock().await;
        sessions.insert(session_id.clone(), session_state);
        info!("Created new session: {} (total sessions: {})", session_id, sessions.len());
        Ok(())
    }

    /// Get session state (if exists and not expired)
    async fn get_session(&self, session_id: &str) -> Option<SessionState> {
        let sessions = self.sessions.lock().await;
        sessions.get(session_id).filter(|s| !s.is_expired()).cloned()
    }

    /// Update session with new message
    async fn add_message_to_session(&self, session_id: &str, role: &str, content: String) -> Result<()> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(session_id) {
            if role == "user" {
                session.add_user_message(content);
            } else if role == "assistant" {
                session.add_assistant_message(content);
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }

    /// Clean up expired sessions (30 min timeout)
    async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.lock().await;
        let initial_count = sessions.len();
        sessions.retain(|id, session| {
            if session.is_expired() {
                info!("Removing expired session: {} (idle for {:?})", id, session.last_activity.elapsed());
                false
            } else {
                true
            }
        });
        let removed = initial_count - sessions.len();
        if removed > 0 {
            info!("Cleaned up {} expired sessions ({} remaining)", removed, sessions.len());
        }
    }
    
    /// Run the ACP server over stdio (for editor integration)
    pub async fn run_stdio(self) -> Result<()> {
        info!("🚀 Starting Aircher ACP server on stdio");
        
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();
        
        loop {
            line.clear();
            
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("Stdin closed, shutting down ACP server");
                    break;
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    
                    debug!("Received ACP message: {}", trimmed);
                    
                    match self.handle_message(trimmed).await {
                        Ok(response) => {
                            if let Some(response_str) = response {
                                debug!("Sending ACP response: {}", response_str);
                                if let Err(e) = stdout.write_all(response_str.as_bytes()).await {
                                    error!("Failed to write response: {}", e);
                                    break;
                                }
                                if let Err(e) = stdout.write_all(b"\n").await {
                                    error!("Failed to write newline: {}", e);
                                    break;
                                }
                                if let Err(e) = stdout.flush().await {
                                    error!("Failed to flush stdout: {}", e);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to handle ACP message: {}", e);
                            let error_response = serde_json::json!({
                                "jsonrpc": "2.0",
                                "error": {
                                    "code": -32603,
                                    "message": format!("Internal error: {}", e)
                                },
                                "id": null
                            });
                            if let Err(write_err) = stdout.write_all(error_response.to_string().as_bytes()).await {
                                error!("Failed to write error response: {}", write_err);
                                break;
                            }
                            if let Err(write_err) = stdout.write_all(b"\n").await {
                                error!("Failed to write error newline: {}", write_err);
                                break;
                            }
                            if let Err(flush_err) = stdout.flush().await {
                                error!("Failed to flush error response: {}", flush_err);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read from stdin: {}", e);
                    break;
                }
            }
        }
        
        info!("🛑 ACP server shutting down");
        Ok(())
    }
    
    /// Handle a JSON-RPC message
    async fn handle_message(&self, message: &str) -> Result<Option<String>> {
        let request: serde_json::Value = serde_json::from_str(message)?;

        let method = request["method"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing method in request"))?;
        let params = &request["params"];
        let id = &request["id"];

        debug!("Handling ACP method: {}", method);

        // Periodically clean up expired sessions
        self.cleanup_expired_sessions().await;

        let agent = self.agent.lock().await;

        let result = match method {
            "initialize" => {
                let init_request: InitializeRequest = serde_json::from_value(params.clone())?;
                let response = agent.initialize(init_request).await?;
                serde_json::to_value(response)?
            }
            "session/new" => {
                let session_request: NewSessionRequest = serde_json::from_value(params.clone())?;
                let response = agent.new_session(session_request).await?;

                // Create session state for tracking
                let session_id = response.session_id.0.to_string();
                drop(agent); // Release lock before async call
                self.create_session(session_id).await?;

                serde_json::to_value(response)?
            }
            "session/prompt" => {
                let prompt_request: PromptRequest = serde_json::from_value(params.clone())?;
                let session_id = prompt_request.session_id.0.to_string();

                // Extract user message from prompt
                let user_message = prompt_request.prompt.iter()
                    .filter_map(|block| match block {
                        ContentBlock::Text(text_content) => Some(text_content.text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                // Check if session exists and is not expired
                drop(agent); // Release lock before session check
                if self.get_session(&session_id).await.is_none() {
                    warn!("Session not found or expired: {}", session_id);
                    return Ok(Some(serde_json::json!({
                        "jsonrpc": "2.0",
                        "error": {
                            "code": -32000,
                            "message": format!("Session not found or expired: {}", session_id)
                        },
                        "id": id
                    }).to_string()));
                }

                // Add user message to session history
                self.add_message_to_session(&session_id, "user", user_message.clone()).await?;

                // Get session history for context
                let session = self.get_session(&session_id).await.unwrap();
                let history = session.get_history(Some(10)); // Last 10 messages
                info!("Processing prompt for session {} with {} messages in history", session_id, history.len());

                // Re-acquire agent lock for processing
                let agent = self.agent.lock().await;
                let response = agent.prompt(prompt_request).await?;

                // Add assistant response to session (simplified - in real impl would capture actual response)
                drop(agent); // Release lock
                self.add_message_to_session(&session_id, "assistant", "Response processed".to_string()).await?;

                serde_json::to_value(response)?
            }
            _ => {
                warn!("Unknown ACP method: {}", method);
                return Ok(Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {}", method)
                    },
                    "id": id
                }).to_string()));
            }
        };

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": id
        });

        Ok(Some(response.to_string()))
    }
}