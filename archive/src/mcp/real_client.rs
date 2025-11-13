use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Instant;
use tracing::{debug, error, info};

use super::{
    McpClient, McpConnectionStatus, McpResult, McpServerConfig,
    ToolInfo, ResourceInfo, ResourceContent, PromptInfo, ServerInfo, HealthStatus,
};
use super::transport::{McpTransport, StdioTransport, HttpTransport};
use super::client::{CheckResult, ServerCapabilities, ToolCapabilities, ResourceCapabilities, PromptCapabilities};

/// Real MCP client implementation using transport layers
pub struct RealMcpClient {
    config: McpServerConfig,
    transport: Arc<Mutex<Box<dyn McpTransport>>>,
    status: Arc<Mutex<McpConnectionStatus>>,
    initialized: Arc<Mutex<bool>>,
}

impl RealMcpClient {
    /// Create a new real MCP client with appropriate transport
    pub fn new(config: McpServerConfig) -> Result<Self> {
        let transport: Box<dyn McpTransport> = match &config.server_type {
            super::McpServerType::Local { .. } | super::McpServerType::Docker { .. } => {
                Box::new(StdioTransport::new(config.clone()))
            }
            super::McpServerType::Remote { .. } => {
                Box::new(HttpTransport::new(config.clone())?)
            }
        };

        Ok(Self {
            config,
            transport: Arc::new(Mutex::new(transport)),
            status: Arc::new(Mutex::new(McpConnectionStatus::Disconnected)),
            initialized: Arc::new(Mutex::new(false)),
        })
    }

    /// Initialize the MCP session with the server
    async fn initialize_session(&self) -> Result<()> {
        debug!("Initializing MCP session with server '{}'", self.config.name);

        let init_params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "roots": {
                    "listChanged": false
                },
                "sampling": {},
                "experimental": {}
            },
            "clientInfo": {
                "name": "Aircher",
                "version": "1.0.0"
            }
        });

        let transport = self.transport.lock().await;
        let response = transport.send_request("initialize", init_params).await?;

        debug!("MCP server '{}' initialize response: {:?}", self.config.name, response);

        // Send initialized notification
        transport.send_notification("initialized", serde_json::json!({})).await?;
        drop(transport);

        let mut initialized = self.initialized.lock().await;
        *initialized = true;

        info!("MCP session initialized with server '{}'", self.config.name);
        Ok(())
    }

    /// Helper method to ensure we're connected and initialized
    async fn ensure_ready(&self) -> Result<()> {
        if !self.is_connected() {
            return Err(anyhow!("Not connected to MCP server"));
        }

        let initialized = *self.initialized.lock().await;
        if !initialized {
            return Err(anyhow!("MCP session not initialized"));
        }

        Ok(())
    }

    /// Convert transport response to MCP result
    fn to_mcp_result<T>(&self, operation: String, start_time: Instant, result: Result<T>) -> McpResult<T> {
        let duration = start_time.elapsed();
        McpResult::new(result, self.config.name.clone(), operation, duration)
    }
}

#[async_trait]
impl McpClient for RealMcpClient {
    fn server_config(&self) -> &McpServerConfig {
        &self.config
    }

    fn connection_status(&self) -> McpConnectionStatus {
        // For synchronous access, we'll use try_lock with a fallback
        match self.status.try_lock() {
            Ok(status) => status.clone(),
            Err(_) => McpConnectionStatus::Disconnected, // Fallback if locked
        }
    }

    async fn connect(&mut self) -> Result<()> {
        let mut status = self.status.lock().await;
        *status = McpConnectionStatus::Connecting;
        drop(status);

        info!("Connecting to MCP server '{}'", self.config.name);

        match self.transport.lock().await.connect().await {
            Ok(_) => {
                let mut status = self.status.lock().await;
                *status = McpConnectionStatus::Connected;
                drop(status);

                // Initialize the MCP session
                match self.initialize_session().await {
                    Ok(_) => {
                        info!("Successfully connected and initialized MCP server '{}'", self.config.name);
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to initialize MCP session with server '{}': {}", self.config.name, e);
                        let mut status = self.status.lock().await;
                        *status = McpConnectionStatus::Failed {
                            error: e.to_string(),
                            retry_count: 0
                        };
                        Err(e)
                    }
                }
            }
            Err(e) => {
                error!("Failed to connect to MCP server '{}': {}", self.config.name, e);
                let mut status = self.status.lock().await;
                *status = McpConnectionStatus::Failed {
                    error: e.to_string(),
                    retry_count: 0
                };
                Err(e)
            }
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from MCP server '{}'", self.config.name);

        let result = self.transport.lock().await.disconnect().await;

        let mut status = self.status.lock().await;
        *status = McpConnectionStatus::Disconnected;
        drop(status);

        let mut initialized = self.initialized.lock().await;
        *initialized = false;

        result
    }

    async fn list_tools(&self) -> McpResult<Vec<ToolInfo>> {
        let start = Instant::now();

        let result = async {
            self.ensure_ready().await?;
            let transport = self.transport.lock().await;
            let response = transport.send_request("tools/list", serde_json::json!({})).await?;

            let tools: Vec<ToolInfo> = serde_json::from_value(
                response.get("tools").unwrap_or(&serde_json::json!([])).clone()
            ).map_err(|e| anyhow!("Failed to parse tools response: {}", e))?;

            Ok(tools)
        }.await;

        self.to_mcp_result("list_tools".to_string(), start, result)
    }

    async fn list_resources(&self) -> McpResult<Vec<ResourceInfo>> {
        let start = Instant::now();

        let result = async {
            self.ensure_ready().await?;
            let transport = self.transport.lock().await;
            let response = transport.send_request("resources/list", serde_json::json!({})).await?;

            let resources: Vec<ResourceInfo> = serde_json::from_value(
                response.get("resources").unwrap_or(&serde_json::json!([])).clone()
            ).map_err(|e| anyhow!("Failed to parse resources response: {}", e))?;

            Ok(resources)
        }.await;

        self.to_mcp_result("list_resources".to_string(), start, result)
    }

    async fn list_prompts(&self) -> McpResult<Vec<PromptInfo>> {
        let start = Instant::now();

        let result = async {
            self.ensure_ready().await?;
            let transport = self.transport.lock().await;
            let response = transport.send_request("prompts/list", serde_json::json!({})).await?;

            let prompts: Vec<PromptInfo> = serde_json::from_value(
                response.get("prompts").unwrap_or(&serde_json::json!([])).clone()
            ).map_err(|e| anyhow!("Failed to parse prompts response: {}", e))?;

            Ok(prompts)
        }.await;

        self.to_mcp_result("list_prompts".to_string(), start, result)
    }

    async fn call_tool(&self, tool_name: &str, parameters: Value) -> McpResult<Value> {
        let start = Instant::now();

        let result = async {
            self.ensure_ready().await?;
            let transport = self.transport.lock().await;

            let params = serde_json::json!({
                "name": tool_name,
                "arguments": parameters
            });

            let response = transport.send_request("tools/call", params).await?;
            Ok(response)
        }.await;

        self.to_mcp_result(format!("call_tool:{}", tool_name), start, result)
    }

    async fn get_resource(&self, uri: &str) -> McpResult<ResourceContent> {
        let start = Instant::now();

        let result = async {
            self.ensure_ready().await?;
            let transport = self.transport.lock().await;

            let params = serde_json::json!({
                "uri": uri
            });

            let response = transport.send_request("resources/read", params).await?;

            let content: ResourceContent = serde_json::from_value(response)
                .map_err(|e| anyhow!("Failed to parse resource content: {}", e))?;

            Ok(content)
        }.await;

        self.to_mcp_result(format!("get_resource:{}", uri), start, result)
    }

    async fn execute_prompt(&self, prompt_name: &str, arguments: Value) -> McpResult<Value> {
        let start = Instant::now();

        let result = async {
            self.ensure_ready().await?;
            let transport = self.transport.lock().await;

            let params = serde_json::json!({
                "name": prompt_name,
                "arguments": arguments
            });

            let response = transport.send_request("prompts/get", params).await?;
            Ok(response)
        }.await;

        self.to_mcp_result(format!("execute_prompt:{}", prompt_name), start, result)
    }

    async fn get_server_info(&self) -> McpResult<ServerInfo> {
        let start = Instant::now();

        let result = async {
            self.ensure_ready().await?;
            // Server info is typically obtained during initialization
            // For now, return a constructed response based on our configuration
            Ok(ServerInfo {
                name: self.config.name.clone(),
                version: "unknown".to_string(),
                description: self.config.description.clone(),
                author: None,
                capabilities: ServerCapabilities {
                    tools: Some(ToolCapabilities {
                        supports_cancellation: false,
                        supports_progress: false,
                    }),
                    resources: Some(ResourceCapabilities {
                        supports_streaming: false,
                        supports_subscriptions: false,
                    }),
                    prompts: Some(PromptCapabilities {
                        supports_templates: true,
                    }),
                    sampling: None,
                },
                supported_transports: vec![
                    "unknown".to_string() // We can't lock synchronously here
                ],
            })
        }.await;

        self.to_mcp_result("get_server_info".to_string(), start, result)
    }

    async fn health_check(&self) -> McpResult<HealthStatus> {
        let start = Instant::now();

        let result = async {
            if !self.is_connected() {
                return Ok(HealthStatus {
                    status: "unhealthy".to_string(),
                    checks: std::collections::HashMap::new(),
                    uptime_seconds: None,
                    last_error: Some("Not connected".to_string()),
                });
            }

            // Try a simple ping-like operation
            let transport = self.transport.lock().await;
            match transport.send_request("ping", serde_json::json!({})).await {
                Ok(_) => {
                    let mut checks = std::collections::HashMap::new();
                    checks.insert("connectivity".to_string(), CheckResult {
                        status: "healthy".to_string(),
                        message: Some("Connection is responsive".to_string()),
                        duration_ms: Some(start.elapsed().as_millis() as u64),
                    });

                    Ok(HealthStatus {
                        status: "healthy".to_string(),
                        checks,
                        uptime_seconds: None,
                        last_error: None,
                    })
                }
                Err(e) => {
                    let mut checks = std::collections::HashMap::new();
                    checks.insert("connectivity".to_string(), CheckResult {
                        status: "unhealthy".to_string(),
                        message: Some(e.to_string()),
                        duration_ms: Some(start.elapsed().as_millis() as u64),
                    });

                    Ok(HealthStatus {
                        status: "unhealthy".to_string(),
                        checks,
                        uptime_seconds: None,
                        last_error: Some(e.to_string()),
                    })
                }
            }
        }.await;

        self.to_mcp_result("health_check".to_string(), start, result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::McpServerConfig;

    #[tokio::test]
    async fn test_real_client_creation() {
        let config = McpServerConfig::local("test", "echo");
        let client = RealMcpClient::new(config).unwrap();

        assert!(!client.is_connected());
        assert_eq!(client.connection_status(), McpConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn test_transport_selection() {
        // Test stdio transport selection
        let local_config = McpServerConfig::local("local", "echo");
        let local_client = RealMcpClient::new(local_config).unwrap();
        assert_eq!(local_client.server_config().name, "local");

        // Test HTTP transport selection
        let remote_config = McpServerConfig::remote("remote", "https://api.example.com/mcp");
        let remote_client = RealMcpClient::new(remote_config).unwrap();
        assert_eq!(remote_client.server_config().name, "remote");
    }
}
