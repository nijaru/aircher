//! Stdio-based ACP server for editor integration

use anyhow::Result;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info, warn};
use serde_json;

use agent_client_protocol::{Agent as AcpAgent, InitializeRequest, NewSessionRequest, PromptRequest};
use crate::agent::core::Agent;
use crate::intelligence::IntelligenceEngine;
use crate::auth::AuthManager;
use crate::config::ConfigManager;
use crate::storage::DatabaseManager;
use crate::agent::conversation::{ProjectContext, ProgrammingLanguage};

/// ACP Server that communicates over stdio (JSON-RPC)
pub struct AcpServer {
    agent: Arc<tokio::sync::Mutex<Agent>>,
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
        })
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
                serde_json::to_value(response)?
            }
            "session/prompt" => {
                let prompt_request: PromptRequest = serde_json::from_value(params.clone())?;
                let response = agent.prompt(prompt_request).await?;
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