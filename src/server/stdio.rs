//! Stdio-based ACP server for editor integration

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::time::{Duration, Instant, timeout};
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

/// Operation timeout (5 minutes for long-running operations)
const OPERATION_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// Maximum retry attempts for transient failures
const MAX_RETRIES: u32 = 3;

/// Base delay for exponential backoff (100ms)
const BASE_RETRY_DELAY: Duration = Duration::from_millis(100);

/// JSON-RPC error codes
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum JsonRpcError {
    /// Parse error (-32700)
    ParseError = -32700,
    /// Invalid request (-32600)
    InvalidRequest = -32600,
    /// Method not found (-32601)
    MethodNotFound = -32601,
    /// Invalid params (-32602)
    InvalidParams = -32602,
    /// Internal error (-32603)
    InternalError = -32603,
    /// Server error (-32000 to -32099)
    ServerError = -32000,
    /// Session not found (-32001)
    SessionNotFound = -32001,
    /// Session expired (-32002)
    SessionExpired = -32002,
    /// Operation timeout (-32003)
    OperationTimeout = -32003,
    /// Rate limit exceeded (-32004)
    RateLimitExceeded = -32004,
}

impl JsonRpcError {
    fn code(self) -> i32 {
        self as i32
    }

    fn default_message(self) -> &'static str {
        match self {
            Self::ParseError => "Parse error",
            Self::InvalidRequest => "Invalid request",
            Self::MethodNotFound => "Method not found",
            Self::InvalidParams => "Invalid params",
            Self::InternalError => "Internal error",
            Self::ServerError => "Server error",
            Self::SessionNotFound => "Session not found",
            Self::SessionExpired => "Session expired",
            Self::OperationTimeout => "Operation timeout",
            Self::RateLimitExceeded => "Rate limit exceeded",
        }
    }
}

/// Error context for better user messages
#[derive(Debug, Clone)]
struct ErrorContext {
    code: i32,
    message: String,
    /// User-friendly description
    user_message: String,
    /// Whether this error is retryable
    retryable: bool,
    /// Suggested action for user
    suggestion: Option<String>,
}

impl ErrorContext {
    fn new(code: i32, message: String, user_message: String) -> Self {
        Self {
            code,
            message,
            user_message,
            retryable: false,
            suggestion: None,
        }
    }

    fn with_retry(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    /// Convert anyhow::Error to ErrorContext with user-friendly message
    fn from_error(err: &anyhow::Error) -> Self {
        let err_str = err.to_string();

        // Parse error type and provide helpful context
        if err_str.contains("timeout") || err_str.contains("timed out") {
            ErrorContext::new(
                JsonRpcError::OperationTimeout.code(),
                err_str.clone(),
                "Operation took too long and was cancelled".to_string()
            )
            .with_retry(true)
            .with_suggestion("Try a simpler request or increase timeout".to_string())
        } else if err_str.contains("connection") || err_str.contains("network") {
            ErrorContext::new(
                JsonRpcError::ServerError.code(),
                err_str.clone(),
                "Network connection issue".to_string()
            )
            .with_retry(true)
            .with_suggestion("Check your internet connection and try again".to_string())
        } else if err_str.contains("Session not found") || err_str.contains("session") {
            ErrorContext::new(
                JsonRpcError::SessionNotFound.code(),
                err_str.clone(),
                "Session not found or expired".to_string()
            )
            .with_retry(false)
            .with_suggestion("Please start a new session".to_string())
        } else if err_str.contains("parse") || err_str.contains("JSON") {
            ErrorContext::new(
                JsonRpcError::ParseError.code(),
                err_str.clone(),
                "Failed to parse request".to_string()
            )
            .with_retry(false)
            .with_suggestion("Check request format".to_string())
        } else {
            // Generic error
            ErrorContext::new(
                JsonRpcError::InternalError.code(),
                err_str.clone(),
                "An unexpected error occurred".to_string()
            )
            .with_retry(true)
            .with_suggestion("Please try again".to_string())
        }
    }
}

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

/// Streaming notification types
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
enum StreamUpdate {
    /// Text chunk from LLM response
    #[serde(rename = "text")]
    Text { content: String },
    /// Tool execution started
    #[serde(rename = "tool_start")]
    ToolStart { tool_name: String, parameters: serde_json::Value },
    /// Tool execution progress
    #[serde(rename = "tool_progress")]
    ToolProgress { tool_name: String, message: String },
    /// Tool execution completed
    #[serde(rename = "tool_complete")]
    ToolComplete { tool_name: String, result: serde_json::Value },
    /// Thinking/reasoning update
    #[serde(rename = "thinking")]
    Thinking { content: String },
}

/// ACP Server that communicates over stdio (JSON-RPC)
pub struct AcpServer {
    agent: Arc<tokio::sync::Mutex<Agent>>,
    /// Active sessions (SessionId -> SessionState)
    sessions: Arc<tokio::sync::Mutex<HashMap<String, SessionState>>>,
    /// Stdout handle for streaming notifications
    stdout: Arc<tokio::sync::Mutex<tokio::io::Stdout>>,
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
            stdout: Arc::new(tokio::sync::Mutex::new(io::stdout())),
        })
    }

    /// Send a streaming notification (JSON-RPC notification with no ID)
    async fn send_notification(&self, session_id: &str, update: StreamUpdate) -> Result<()> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "session/update",
            "params": {
                "session_id": session_id,
                "update": update
            }
        });

        let mut stdout = self.stdout.lock().await;
        stdout.write_all(notification.to_string().as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;

        debug!("Sent streaming notification to session {}: {:?}", session_id, update);
        Ok(())
    }

    /// Send a text chunk notification
    async fn stream_text(&self, session_id: &str, content: &str) -> Result<()> {
        self.send_notification(session_id, StreamUpdate::Text {
            content: content.to_string()
        }).await
    }

    /// Send tool start notification
    async fn stream_tool_start(&self, session_id: &str, tool_name: &str, parameters: serde_json::Value) -> Result<()> {
        self.send_notification(session_id, StreamUpdate::ToolStart {
            tool_name: tool_name.to_string(),
            parameters
        }).await
    }

    /// Send tool progress notification
    async fn stream_tool_progress(&self, session_id: &str, tool_name: &str, message: &str) -> Result<()> {
        self.send_notification(session_id, StreamUpdate::ToolProgress {
            tool_name: tool_name.to_string(),
            message: message.to_string()
        }).await
    }

    /// Send tool complete notification
    async fn stream_tool_complete(&self, session_id: &str, tool_name: &str, result: serde_json::Value) -> Result<()> {
        self.send_notification(session_id, StreamUpdate::ToolComplete {
            tool_name: tool_name.to_string(),
            result
        }).await
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
    
    /// Send a JSON-RPC response
    async fn send_response(&self, response: String) -> Result<()> {
        let mut stdout = self.stdout.lock().await;
        stdout.write_all(response.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
        Ok(())
    }

    /// Send a JSON-RPC error response
    async fn send_error(&self, id: Option<serde_json::Value>, code: i32, message: String) -> Result<()> {
        let error_response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {
                "code": code,
                "message": message
            },
            "id": id
        });
        self.send_response(error_response.to_string()).await
    }

    /// Send enhanced error response with user context
    async fn send_error_context(&self, id: Option<serde_json::Value>, context: ErrorContext) -> Result<()> {
        let mut error_data = serde_json::json!({
            "user_message": context.user_message,
            "retryable": context.retryable,
        });

        if let Some(suggestion) = context.suggestion {
            error_data["suggestion"] = serde_json::Value::String(suggestion);
        }

        let error_response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {
                "code": context.code,
                "message": context.message,
                "data": error_data
            },
            "id": id
        });

        self.send_response(error_response.to_string()).await
    }

    /// Retry an operation with exponential backoff
    async fn retry_operation<F, Fut, T>(&self, operation: F, operation_name: &str) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < MAX_RETRIES {
            match operation().await {
                Ok(result) => {
                    if attempts > 0 {
                        info!("Operation {} succeeded after {} retries", operation_name, attempts);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e);

                    if attempts < MAX_RETRIES {
                        // Exponential backoff: 100ms, 200ms, 400ms
                        let delay = BASE_RETRY_DELAY * 2_u32.pow(attempts - 1);
                        warn!(
                            "Operation {} failed (attempt {}/{}), retrying in {:?}",
                            operation_name, attempts, MAX_RETRIES, delay
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        error!("Operation {} failed after {} attempts", operation_name, MAX_RETRIES);
        Err(last_error.unwrap())
    }

    /// Execute operation with timeout
    async fn with_timeout<F, Fut, T>(&self, operation: F, operation_name: &str) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        match timeout(OPERATION_TIMEOUT, operation()).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                error!("Operation {} timed out after {:?}", operation_name, OPERATION_TIMEOUT);
                Err(anyhow::anyhow!("Operation timed out after {:?}", OPERATION_TIMEOUT))
            }
        }
    }

    /// Execute operation with both timeout and retry
    async fn execute_resilient<F, Fut, T>(&self, operation: F, operation_name: &str) -> Result<T>
    where
        F: Fn() -> Fut + Clone,
        Fut: std::future::Future<Output = Result<T>>,
    {
        self.retry_operation(
            || self.with_timeout(operation.clone(), operation_name),
            operation_name
        ).await
    }

    /// Run the ACP server over stdio (for editor integration)
    pub async fn run_stdio(self) -> Result<()> {
        info!("🚀 Starting Aircher ACP server on stdio");

        let stdin = io::stdin();
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
                                // Retry response sending in case of transient failures
                                if let Err(e) = self.retry_operation(
                                    || async { self.send_response(response_str.clone()).await },
                                    "send_response"
                                ).await {
                                    error!("Failed to send response after retries: {}", e);
                                    // Only break if we can't send responses at all
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to handle ACP message: {}", e);
                            let error_context = ErrorContext::from_error(&e);

                            // Try to send enhanced error with retry
                            if let Err(send_err) = self.retry_operation(
                                || async { self.send_error_context(None, error_context.clone()).await },
                                "send_error_context"
                            ).await {
                                error!("Failed to send error response after retries: {}", send_err);
                                // If we can't even send errors, connection is likely broken
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

                // STREAMING DEMONSTRATION
                // In real implementation, this would integrate with LLM provider's streaming API
                // For now, demonstrate the streaming pattern:

                // 1. Send thinking notification
                self.send_notification(&session_id, StreamUpdate::Thinking {
                    content: "Analyzing request and planning response...".to_string()
                }).await?;

                // 2. Simulate tool execution with progress
                if user_message.contains("search") || user_message.contains("find") {
                    self.stream_tool_start(&session_id, "search_code", serde_json::json!({
                        "query": user_message
                    })).await?;

                    tokio::time::sleep(Duration::from_millis(100)).await;

                    self.stream_tool_progress(&session_id, "search_code", "Scanning codebase...").await?;

                    tokio::time::sleep(Duration::from_millis(100)).await;

                    self.stream_tool_complete(&session_id, "search_code", serde_json::json!({
                        "results_count": 5
                    })).await?;
                }

                // 3. Stream response text in chunks (simulating token-by-token)
                let response_text = "I understand your request. Processing...";
                for chunk in response_text.chars().collect::<Vec<_>>().chunks(5) {
                    let chunk_str: String = chunk.iter().collect();
                    // Graceful degradation: If streaming fails, continue but log
                    if let Err(e) = self.stream_text(&session_id, &chunk_str).await {
                        warn!("Failed to stream text chunk: {}", e);
                        // Continue processing despite streaming failure
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }

                // Re-acquire agent lock for processing with timeout
                let agent = self.agent.lock().await;

                // Execute agent.prompt with timeout and graceful degradation
                let response = match timeout(OPERATION_TIMEOUT, agent.prompt(prompt_request)).await {
                    Ok(Ok(response)) => response,
                    Ok(Err(e)) => {
                        warn!("Agent prompt failed: {}", e);
                        // Graceful degradation: Return error but don't crash
                        drop(agent);
                        self.add_message_to_session(&session_id, "assistant", "Error processing request".to_string()).await?;

                        // Return error response instead of crashing
                        let error_ctx = ErrorContext::from_error(&e);
                        return Ok(Some(serde_json::json!({
                            "jsonrpc": "2.0",
                            "error": {
                                "code": error_ctx.code,
                                "message": error_ctx.message,
                                "data": {
                                    "user_message": error_ctx.user_message,
                                    "retryable": error_ctx.retryable,
                                    "suggestion": error_ctx.suggestion
                                }
                            },
                            "id": id
                        }).to_string()));
                    }
                    Err(_) => {
                        error!("Agent prompt timed out after {:?}", OPERATION_TIMEOUT);
                        drop(agent);
                        self.add_message_to_session(&session_id, "assistant", "Request timed out".to_string()).await?;

                        // Return timeout error
                        return Ok(Some(serde_json::json!({
                            "jsonrpc": "2.0",
                            "error": {
                                "code": JsonRpcError::OperationTimeout.code(),
                                "message": format!("Operation timed out after {:?}", OPERATION_TIMEOUT),
                                "data": {
                                    "user_message": "Request took too long and was cancelled",
                                    "retryable": true,
                                    "suggestion": "Try a simpler request"
                                }
                            },
                            "id": id
                        }).to_string()));
                    }
                };

                // Add assistant response to session
                drop(agent); // Release lock
                self.add_message_to_session(&session_id, "assistant", response_text.to_string()).await?;

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