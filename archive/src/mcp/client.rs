use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info};

use super::{McpConnectionStatus, McpResult, McpServerConfig};

/// Information about an MCP tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<Value>, // JSON Schema for parameters
    pub required_permissions: Vec<String>,
    pub examples: Vec<ToolExample>,
}

/// Example usage of an MCP tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExample {
    pub description: String,
    pub parameters: Value,
    pub expected_result: Option<String>,
}

/// Information about an MCP resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    pub annotations: HashMap<String, Value>,
}

/// Content of an MCP resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    pub content: Value,
    pub mime_type: String,
    pub annotations: HashMap<String, Value>,
    pub size_bytes: Option<usize>,
}

/// MCP client trait for standardized server interactions
#[async_trait]
pub trait McpClient: Send + Sync {
    /// Get the server configuration this client is connected to
    fn server_config(&self) -> &McpServerConfig;

    /// Get current connection status
    fn connection_status(&self) -> McpConnectionStatus;

    /// Connect to the MCP server
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from the MCP server
    async fn disconnect(&mut self) -> Result<()>;

    /// Check if client is currently connected
    fn is_connected(&self) -> bool {
        matches!(self.connection_status(), McpConnectionStatus::Connected)
    }

    /// List available tools on the server
    async fn list_tools(&self) -> McpResult<Vec<ToolInfo>>;

    /// List available resources on the server
    async fn list_resources(&self) -> McpResult<Vec<ResourceInfo>>;

    /// List available prompts on the server
    async fn list_prompts(&self) -> McpResult<Vec<PromptInfo>>;

    /// Call a tool with the given parameters
    async fn call_tool(&self, tool_name: &str, parameters: Value) -> McpResult<Value>;

    /// Get a resource by URI
    async fn get_resource(&self, uri: &str) -> McpResult<ResourceContent>;

    /// Execute a prompt template
    async fn execute_prompt(&self, prompt_name: &str, arguments: Value) -> McpResult<Value>;

    /// Get server information and capabilities
    async fn get_server_info(&self) -> McpResult<ServerInfo>;

    /// Perform health check
    async fn health_check(&self) -> McpResult<HealthStatus>;
}

/// Information about an MCP prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptInfo {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<Value>, // JSON Schema
    pub examples: Vec<PromptExample>,
}

/// Example usage of an MCP prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptExample {
    pub description: String,
    pub arguments: Value,
    pub expected_output: Option<String>,
}

/// MCP server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub capabilities: ServerCapabilities,
    pub supported_transports: Vec<String>,
}

/// MCP server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub tools: Option<ToolCapabilities>,
    pub resources: Option<ResourceCapabilities>,
    pub prompts: Option<PromptCapabilities>,
    pub sampling: Option<SamplingCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    pub supports_cancellation: bool,
    pub supports_progress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapabilities {
    pub supports_streaming: bool,
    pub supports_subscriptions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCapabilities {
    pub supports_templates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingCapabilities {
    pub supports_temperature: bool,
    pub supports_top_p: bool,
}

/// Health status of an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String, // "healthy", "degraded", "unhealthy"
    pub checks: HashMap<String, CheckResult>,
    pub uptime_seconds: Option<u64>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub status: String,
    pub message: Option<String>,
    pub duration_ms: Option<u64>,
}

/// Mock MCP client implementation for testing and development
pub struct MockMcpClient {
    config: McpServerConfig,
    status: McpConnectionStatus,
    connected_at: Option<Instant>,
    tools: Vec<ToolInfo>,
    resources: Vec<ResourceInfo>,
    prompts: Vec<PromptInfo>,
}

impl MockMcpClient {
    pub fn new(config: McpServerConfig) -> Self {
        // Create some mock tools based on server type
        let tools = Self::create_mock_tools(&config);
        let resources = Self::create_mock_resources(&config);
        let prompts = Self::create_mock_prompts(&config);

        Self {
            config,
            status: McpConnectionStatus::Disconnected,
            connected_at: None,
            tools,
            resources,
            prompts,
        }
    }

    fn create_mock_tools(config: &McpServerConfig) -> Vec<ToolInfo> {
        let mut tools = vec![
            ToolInfo {
                name: "ping".to_string(),
                description: Some("Test connectivity to the server".to_string()),
                parameters: Some(serde_json::json!({})),
                required_permissions: vec![],
                examples: vec![ToolExample {
                    description: "Basic ping test".to_string(),
                    parameters: serde_json::json!({}),
                    expected_result: Some("pong".to_string()),
                }],
            }
        ];

        // Add server-specific mock tools
        match config.name.as_str() {
            "filesystem" => {
                tools.push(ToolInfo {
                    name: "read_file".to_string(),
                    description: Some("Read contents of a file".to_string()),
                    parameters: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "File path to read"}
                        },
                        "required": ["path"]
                    })),
                    required_permissions: vec!["filesystem.read".to_string()],
                    examples: vec![],
                });

                tools.push(ToolInfo {
                    name: "write_file".to_string(),
                    description: Some("Write content to a file".to_string()),
                    parameters: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "File path to write"},
                            "content": {"type": "string", "description": "Content to write"}
                        },
                        "required": ["path", "content"]
                    })),
                    required_permissions: vec!["filesystem.write".to_string()],
                    examples: vec![],
                });
            },
            "github" => {
                tools.push(ToolInfo {
                    name: "get_repository".to_string(),
                    description: Some("Get information about a GitHub repository".to_string()),
                    parameters: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "owner": {"type": "string", "description": "Repository owner"},
                            "repo": {"type": "string", "description": "Repository name"}
                        },
                        "required": ["owner", "repo"]
                    })),
                    required_permissions: vec!["github.repository.read".to_string()],
                    examples: vec![],
                });
            },
            "postgres" => {
                tools.push(ToolInfo {
                    name: "query_database".to_string(),
                    description: Some("Execute a SQL query on the database".to_string()),
                    parameters: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "query": {"type": "string", "description": "SQL query to execute"}
                        },
                        "required": ["query"]
                    })),
                    required_permissions: vec!["database.query".to_string()],
                    examples: vec![],
                });
            },
            _ => {}
        }

        tools
    }

    fn create_mock_resources(config: &McpServerConfig) -> Vec<ResourceInfo> {
        let mut resources = Vec::new();

        match config.name.as_str() {
            "filesystem" => {
                resources.push(ResourceInfo {
                    uri: "file://./".to_string(),
                    name: "Current Directory".to_string(),
                    description: Some("Contents of the current directory".to_string()),
                    mime_type: Some("application/json".to_string()),
                    annotations: HashMap::new(),
                });
            },
            "github" => {
                resources.push(ResourceInfo {
                    uri: "github://repositories".to_string(),
                    name: "Repositories".to_string(),
                    description: Some("List of accessible repositories".to_string()),
                    mime_type: Some("application/json".to_string()),
                    annotations: HashMap::new(),
                });
            },
            _ => {}
        }

        resources
    }

    fn create_mock_prompts(_config: &McpServerConfig) -> Vec<PromptInfo> {
        vec![
            PromptInfo {
                name: "analyze_code".to_string(),
                description: Some("Analyze code for potential issues".to_string()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "code": {"type": "string", "description": "Code to analyze"},
                        "language": {"type": "string", "description": "Programming language"}
                    },
                    "required": ["code"]
                })),
                examples: vec![],
            }
        ]
    }
}

#[async_trait]
impl McpClient for MockMcpClient {
    fn server_config(&self) -> &McpServerConfig {
        &self.config
    }

    fn connection_status(&self) -> McpConnectionStatus {
        self.status.clone()
    }

    async fn connect(&mut self) -> Result<()> {
        info!("Connecting to mock MCP server: {}", self.config.name);
        self.status = McpConnectionStatus::Connecting;

        // Simulate connection delay
        tokio::time::sleep(Duration::from_millis(100)).await;

        self.status = McpConnectionStatus::Connected;
        self.connected_at = Some(Instant::now());

        info!("Successfully connected to mock MCP server: {}", self.config.name);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from mock MCP server: {}", self.config.name);
        self.status = McpConnectionStatus::Disconnected;
        self.connected_at = None;
        Ok(())
    }

    async fn list_tools(&self) -> McpResult<Vec<ToolInfo>> {
        let start = Instant::now();
        let result = Ok(self.tools.clone());
        let duration = start.elapsed();

        McpResult::new(
            result,
            self.config.name.clone(),
            "list_tools".to_string(),
            duration,
        )
    }

    async fn list_resources(&self) -> McpResult<Vec<ResourceInfo>> {
        let start = Instant::now();
        let result = Ok(self.resources.clone());
        let duration = start.elapsed();

        McpResult::new(
            result,
            self.config.name.clone(),
            "list_resources".to_string(),
            duration,
        )
    }

    async fn list_prompts(&self) -> McpResult<Vec<PromptInfo>> {
        let start = Instant::now();
        let result = Ok(self.prompts.clone());
        let duration = start.elapsed();

        McpResult::new(
            result,
            self.config.name.clone(),
            "list_prompts".to_string(),
            duration,
        )
    }

    async fn call_tool(&self, tool_name: &str, parameters: Value) -> McpResult<Value> {
        let start = Instant::now();
        debug!("Calling mock tool: {} with parameters: {}", tool_name, parameters);

        let result = match tool_name {
            "ping" => Ok(serde_json::json!({"response": "pong", "timestamp": chrono::Utc::now()})),
            "read_file" => {
                if let Some(path) = parameters.get("path").and_then(|v| v.as_str()) {
                    Ok(serde_json::json!({
                        "content": format!("Mock content of file: {}", path),
                        "size": 42,
                        "modified": chrono::Utc::now()
                    }))
                } else {
                    Err(anyhow::anyhow!("Missing 'path' parameter"))
                }
            },
            "write_file" => {
                Ok(serde_json::json!({
                    "success": true,
                    "bytes_written": 100,
                    "timestamp": chrono::Utc::now()
                }))
            },
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
        };

        let duration = start.elapsed();
        McpResult::new(result, self.config.name.clone(), format!("call_tool:{}", tool_name), duration)
    }

    async fn get_resource(&self, uri: &str) -> McpResult<ResourceContent> {
        let start = Instant::now();
        debug!("Getting mock resource: {}", uri);

        let result = Ok(ResourceContent {
            uri: uri.to_string(),
            content: serde_json::json!({
                "type": "mock_resource",
                "uri": uri,
                "data": "Mock resource content"
            }),
            mime_type: "application/json".to_string(),
            annotations: HashMap::new(),
            size_bytes: Some(100),
        });

        let duration = start.elapsed();
        McpResult::new(result, self.config.name.clone(), format!("get_resource:{}", uri), duration)
    }

    async fn execute_prompt(&self, prompt_name: &str, arguments: Value) -> McpResult<Value> {
        let start = Instant::now();
        debug!("Executing mock prompt: {} with arguments: {}", prompt_name, arguments);

        let result = Ok(serde_json::json!({
            "prompt_name": prompt_name,
            "result": "Mock prompt execution result",
            "arguments": arguments
        }));

        let duration = start.elapsed();
        McpResult::new(result, self.config.name.clone(), format!("execute_prompt:{}", prompt_name), duration)
    }

    async fn get_server_info(&self) -> McpResult<ServerInfo> {
        let start = Instant::now();

        let result = Ok(ServerInfo {
            name: self.config.name.clone(),
            version: "1.0.0-mock".to_string(),
            description: self.config.description.clone(),
            author: Some("Aircher Mock Implementation".to_string()),
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
            supported_transports: vec!["stdio".to_string(), "http".to_string()],
        });

        let duration = start.elapsed();
        McpResult::new(result, self.config.name.clone(), "get_server_info".to_string(), duration)
    }

    async fn health_check(&self) -> McpResult<HealthStatus> {
        let start = Instant::now();

        let uptime = self.connected_at.map(|connected_at| connected_at.elapsed().as_secs());

        let mut checks = HashMap::new();
        checks.insert("connectivity".to_string(), CheckResult {
            status: "healthy".to_string(),
            message: Some("Mock connection is healthy".to_string()),
            duration_ms: Some(5),
        });

        let result = Ok(HealthStatus {
            status: "healthy".to_string(),
            checks,
            uptime_seconds: uptime,
            last_error: None,
        });

        let duration = start.elapsed();
        McpResult::new(result, self.config.name.clone(), "health_check".to_string(), duration)
    }
}
