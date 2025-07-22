use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use super::{McpTransport, TransportInfo, JsonRpcMessage, generate_request_id};
use crate::mcp::McpServerConfig;

/// Stdio transport implementation for local MCP servers
pub struct StdioTransport {
    config: McpServerConfig,
    process: Option<Child>,
    connected: Arc<AtomicBool>,
    _request_counter: AtomicU64,
    pending_requests: Arc<RwLock<HashMap<Value, oneshot::Sender<Result<Value>>>>>,
    stdin_tx: Option<mpsc::UnboundedSender<String>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl StdioTransport {
    pub fn new(config: McpServerConfig) -> Self {
        Self {
            config,
            process: None,
            connected: Arc::new(AtomicBool::new(false)),
            _request_counter: AtomicU64::new(1),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            stdin_tx: None,
            shutdown_tx: None,
        }
    }
    
    /// Get the command and args for starting the MCP server
    fn get_command_info(&self) -> Result<(String, Vec<String>, Option<PathBuf>, HashMap<String, String>)> {
        match &self.config.server_type {
            crate::mcp::McpServerType::Local { command, args, working_directory, env } => {
                Ok((command.clone(), args.clone(), working_directory.clone(), env.clone()))
            }
            crate::mcp::McpServerType::Docker { image, args, env, volumes } => {
                // Convert Docker configuration to docker run command
                let mut docker_args = vec![
                    "run".to_string(),
                    "--rm".to_string(),
                    "-i".to_string(), // Interactive for stdin
                ];
                
                // Add volume mounts
                for volume in volumes {
                    docker_args.push("-v".to_string());
                    docker_args.push(volume.clone());
                }
                
                // Add environment variables
                for (key, value) in env {
                    docker_args.push("-e".to_string());
                    docker_args.push(format!("{}={}", key, value));
                }
                
                // Add image and args
                docker_args.push(image.clone());
                docker_args.extend(args.clone());
                
                Ok(("docker".to_string(), docker_args, None, HashMap::new()))
            }
            _ => Err(anyhow!("Stdio transport only supports Local and Docker server types")),
        }
    }
    
    /// Spawn the MCP server process and set up communication channels
    async fn spawn_process(&mut self) -> Result<()> {
        let (command, args, working_dir, env_vars) = self.get_command_info()?;
        
        info!("Starting MCP server: {} {:?}", command, args);
        
        let mut cmd = Command::new(&command);
        cmd.args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        
        if let Some(dir) = &working_dir {
            cmd.current_dir(dir);
        }
        
        for (key, value) in &env_vars {
            cmd.env(key, value);
        }
        
        let mut child = cmd.spawn()
            .map_err(|e| anyhow!("Failed to spawn MCP server process '{}': {}", command, e))?;
        
        let stdin = child.stdin.take()
            .ok_or_else(|| anyhow!("Failed to get stdin handle for MCP server"))?;
        let stdout = child.stdout.take()
            .ok_or_else(|| anyhow!("Failed to get stdout handle for MCP server"))?;
        let stderr = child.stderr.take()
            .ok_or_else(|| anyhow!("Failed to get stderr handle for MCP server"))?;
        
        // Set up communication channels
        let (stdin_tx, stdin_rx) = mpsc::unbounded_channel::<String>();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        
        self.stdin_tx = Some(stdin_tx);
        self.shutdown_tx = Some(shutdown_tx);
        self.process = Some(child);
        
        // Start I/O tasks
        let connected = Arc::clone(&self.connected);
        let pending_requests = Arc::clone(&self.pending_requests);
        let server_name = self.config.name.clone();
        
        // Stdin handler task
        tokio::spawn(Self::handle_stdin(stdin, stdin_rx, connected.clone(), server_name.clone()));
        
        // Stdout reader task  
        tokio::spawn(Self::handle_stdout(stdout, pending_requests.clone(), connected.clone(), server_name.clone()));
        
        // Stderr logger task
        tokio::spawn(Self::handle_stderr(stderr, server_name.clone()));
        
        // Process monitor task
        tokio::spawn(Self::monitor_process(shutdown_rx, connected.clone(), server_name));
        
        self.connected.store(true, Ordering::SeqCst);
        info!("MCP server '{}' started successfully", self.config.name);
        
        Ok(())
    }
    
    /// Handle writing to the server's stdin
    async fn handle_stdin(
        mut stdin: tokio::process::ChildStdin,
        mut rx: mpsc::UnboundedReceiver<String>,
        connected: Arc<AtomicBool>,
        server_name: String,
    ) {
        while let Some(message) = rx.recv().await {
            if !connected.load(Ordering::SeqCst) {
                break;
            }
            
            debug!("Sending to MCP server '{}': {}", server_name, message);
            
            if let Err(e) = stdin.write_all(message.as_bytes()).await {
                error!("Failed to write to MCP server '{}' stdin: {}", server_name, e);
                connected.store(false, Ordering::SeqCst);
                break;
            }
            
            if let Err(e) = stdin.write_all(b"\n").await {
                error!("Failed to write newline to MCP server '{}' stdin: {}", server_name, e);
                connected.store(false, Ordering::SeqCst);
                break;
            }
            
            if let Err(e) = stdin.flush().await {
                error!("Failed to flush MCP server '{}' stdin: {}", server_name, e);
                connected.store(false, Ordering::SeqCst);
                break;
            }
        }
        
        debug!("Stdin handler for MCP server '{}' shutting down", server_name);
    }
    
    /// Handle reading from the server's stdout
    async fn handle_stdout(
        stdout: tokio::process::ChildStdout,
        pending_requests: Arc<RwLock<HashMap<Value, oneshot::Sender<Result<Value>>>>>,
        connected: Arc<AtomicBool>,
        server_name: String,
    ) {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        
        while connected.load(Ordering::SeqCst) {
            line.clear();
            
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    // EOF reached
                    debug!("EOF reached for MCP server '{}' stdout", server_name);
                    break;
                }
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    
                    debug!("Received from MCP server '{}': {}", server_name, line);
                    
                    // Parse JSON-RPC message
                    match serde_json::from_str::<JsonRpcMessage>(line) {
                        Ok(message) => {
                            Self::handle_message(message, &pending_requests, &server_name).await;
                        }
                        Err(e) => {
                            warn!("Failed to parse message from MCP server '{}': {} (message: {})", server_name, e, line);
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading from MCP server '{}' stdout: {}", server_name, e);
                    break;
                }
            }
        }
        
        connected.store(false, Ordering::SeqCst);
        debug!("Stdout handler for MCP server '{}' shutting down", server_name);
    }
    
    /// Handle reading from the server's stderr for logging
    async fn handle_stderr(stderr: tokio::process::ChildStderr, server_name: String) {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        
        loop {
            line.clear();
            
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let line = line.trim();
                    if !line.is_empty() {
                        debug!("MCP server '{}' stderr: {}", server_name, line);
                    }
                }
                Err(e) => {
                    warn!("Error reading from MCP server '{}' stderr: {}", server_name, e);
                    break;
                }
            }
        }
    }
    
    /// Monitor the process and handle shutdown
    async fn monitor_process(
        mut shutdown_rx: oneshot::Receiver<()>,
        connected: Arc<AtomicBool>,
        server_name: String,
    ) {
        tokio::select! {
            _ = shutdown_rx => {
                debug!("Shutdown signal received for MCP server '{}'", server_name);
            }
        }
        
        connected.store(false, Ordering::SeqCst);
    }
    
    /// Handle incoming JSON-RPC messages
    async fn handle_message(
        message: JsonRpcMessage,
        pending_requests: &Arc<RwLock<HashMap<Value, oneshot::Sender<Result<Value>>>>>,
        server_name: &str,
    ) {
        match message {
            JsonRpcMessage::Response { id, result, error, .. } => {
                let mut pending = pending_requests.write().await;
                if let Some(sender) = pending.remove(&id) {
                    let response = if let Some(error) = error {
                        Err(anyhow!("MCP server error: {} ({})", error.message, error.code))
                    } else {
                        Ok(result.unwrap_or(Value::Null))
                    };
                    
                    if sender.send(response).is_err() {
                        warn!("Failed to send response for request {} to MCP server '{}'", id, server_name);
                    }
                } else {
                    warn!("Received response for unknown request {} from MCP server '{}'", id, server_name);
                }
            }
            JsonRpcMessage::Notification { method, params, .. } => {
                debug!("Received notification from MCP server '{}': {} {:?}", server_name, method, params);
                // Handle notifications (e.g., log messages, status updates)
            }
            JsonRpcMessage::Request { .. } => {
                warn!("Received unexpected request from MCP server '{}' - servers should not send requests", server_name);
            }
        }
    }
    
    /// Send a JSON-RPC message to the server
    async fn send_message(&self, message: JsonRpcMessage) -> Result<()> {
        if !self.is_connected() {
            return Err(anyhow!("Not connected to MCP server"));
        }
        
        let json = serde_json::to_string(&message)
            .map_err(|e| anyhow!("Failed to serialize message: {}", e))?;
        
        if let Some(ref tx) = self.stdin_tx {
            tx.send(json)
                .map_err(|_| anyhow!("Failed to send message to stdin handler"))?;
        } else {
            return Err(anyhow!("Stdin channel not available"));
        }
        
        Ok(())
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn connect(&mut self) -> Result<()> {
        if self.is_connected() {
            return Ok(());
        }
        
        self.spawn_process().await?;
        
        // Wait a moment for the process to start up
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        if !self.is_connected() {
            return Err(anyhow!("Process started but connection not established"));
        }
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        if !self.is_connected() {
            return Ok(());
        }
        
        info!("Disconnecting from MCP server '{}'", self.config.name);
        
        self.connected.store(false, Ordering::SeqCst);
        
        // Send shutdown signal
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        
        // Close stdin channel
        self.stdin_tx = None;
        
        // Kill the process if it's still running
        if let Some(mut process) = self.process.take() {
            if let Err(e) = process.kill().await {
                warn!("Failed to kill MCP server process '{}': {}", self.config.name, e);
            }
            
            // Wait for the process to exit
            if let Err(e) = timeout(Duration::from_secs(5), process.wait()).await {
                error!("Timeout waiting for MCP server '{}' to exit: {}", self.config.name, e);
            }
        }
        
        // Clear pending requests
        let mut pending = self.pending_requests.write().await;
        for (_, sender) in pending.drain() {
            let _ = sender.send(Err(anyhow!("Connection closed")));
        }
        
        info!("Disconnected from MCP server '{}'", self.config.name);
        Ok(())
    }
    
    async fn send_request(&self, method: &str, params: Value) -> Result<Value> {
        let id = generate_request_id();
        let message = JsonRpcMessage::request(id.clone(), method.to_string(), Some(params));
        
        let (response_tx, response_rx) = oneshot::channel();
        
        // Register the pending request
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(id, response_tx);
        }
        
        // Send the request
        self.send_message(message).await?;
        
        // Wait for the response with timeout
        let timeout_duration = Duration::from_secs(
            self.config.timeout_seconds.unwrap_or(30)
        );
        
        match timeout(timeout_duration, response_rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(anyhow!("Response channel closed")),
            Err(_) => {
                // Remove the timed-out request
                let mut pending = self.pending_requests.write().await;
                pending.remove(&generate_request_id());
                Err(anyhow!("Request timed out after {:?}", timeout_duration))
            }
        }
    }
    
    async fn send_notification(&self, method: &str, params: Value) -> Result<()> {
        let message = JsonRpcMessage::notification(method.to_string(), Some(params));
        self.send_message(message).await
    }
    
    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }
    
    fn transport_info(&self) -> TransportInfo {
        let connection_details = match &self.config.server_type {
            crate::mcp::McpServerType::Local { command, args, .. } => {
                format!("{} {}", command, args.join(" "))
            }
            crate::mcp::McpServerType::Docker { image, .. } => {
                format!("docker: {}", image)
            }
            _ => "unknown".to_string(),
        };
        
        TransportInfo {
            transport_type: "stdio".to_string(),
            connection_details,
            supports_notifications: true,
            max_concurrent_requests: None, // No inherent limit for stdio
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::McpServerConfig;
    
    #[tokio::test]
    async fn test_stdio_transport_creation() {
        let config = McpServerConfig::local("test", "echo");
        let transport = StdioTransport::new(config);
        
        assert!(!transport.is_connected());
        assert_eq!(transport.transport_info().transport_type, "stdio");
    }
    
    #[tokio::test]
    async fn test_command_info_extraction() {
        let config = McpServerConfig::local("test", "node")
            .with_arg("server.js")
            .with_arg("--port=3000")
            .with_env("NODE_ENV", "test");
            
        let transport = StdioTransport::new(config);
        let (command, args, _working_dir, env) = transport.get_command_info().unwrap();
        
        assert_eq!(command, "node");
        assert_eq!(args, vec!["server.js", "--port=3000"]);
        assert_eq!(env.get("NODE_ENV"), Some(&"test".to_string()));
    }
}