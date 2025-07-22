use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

pub mod config;
pub mod client;
pub mod manager;
pub mod transport;
pub mod real_client;

pub use config::{McpConfig, StdioServerConfig, HttpServerConfig};
pub use client::{McpClient, ToolInfo, ResourceInfo, ResourceContent, PromptInfo, ServerInfo, HealthStatus};
pub use manager::McpClientManager;

/// MCP server configuration types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpServerType {
    /// Local process using stdio transport
    Local {
        command: String,
        args: Vec<String>,
        working_directory: Option<PathBuf>,
        env: HashMap<String, String>,
    },
    /// Docker container using stdio transport  
    Docker {
        image: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        volumes: Vec<String>,
    },
    /// Remote HTTP server using SSE transport
    Remote {
        url: String,
        auth_type: AuthType,
        headers: HashMap<String, String>,
    },
}

/// Authentication types for MCP servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    None,
    Bearer { token: String },
    OAuth { 
        client_id: String,
        scopes: Vec<String>,
        token_endpoint: Option<String>,
    },
    ApiKey {
        header: String,
        value: String,
    },
}

/// MCP server configuration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub enabled: bool,
    pub server_type: McpServerType,
    pub timeout_seconds: Option<u64>,
    pub auto_reconnect: bool,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

impl McpServerConfig {
    /// Create a new local MCP server configuration
    pub fn local(name: &str, command: &str) -> Self {
        Self {
            name: name.to_string(),
            enabled: true,
            server_type: McpServerType::Local {
                command: command.to_string(),
                args: Vec::new(),
                working_directory: None,
                env: HashMap::new(),
            },
            timeout_seconds: Some(30),
            auto_reconnect: true,
            description: None,
            tags: Vec::new(),
        }
    }
    
    /// Create a new Docker MCP server configuration
    pub fn docker(name: &str, image: &str) -> Self {
        Self {
            name: name.to_string(),
            enabled: true,
            server_type: McpServerType::Docker {
                image: image.to_string(),
                args: Vec::new(),
                env: HashMap::new(),
                volumes: Vec::new(),
            },
            timeout_seconds: Some(60),
            auto_reconnect: true,
            description: None,
            tags: Vec::new(),
        }
    }
    
    /// Create a new remote MCP server configuration
    pub fn remote(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            enabled: true,
            server_type: McpServerType::Remote {
                url: url.to_string(),
                auth_type: AuthType::None,
                headers: HashMap::new(),
            },
            timeout_seconds: Some(30),
            auto_reconnect: true,
            description: None,
            tags: Vec::new(),
        }
    }
    
    /// Builder pattern: Add environment variable
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        match &mut self.server_type {
            McpServerType::Local { env, .. } | McpServerType::Docker { env, .. } => {
                env.insert(key.to_string(), value.to_string());
            }
            _ => {}
        }
        self
    }
    
    /// Builder pattern: Add command argument
    pub fn with_arg(mut self, arg: &str) -> Self {
        match &mut self.server_type {
            McpServerType::Local { args, .. } | McpServerType::Docker { args, .. } => {
                args.push(arg.to_string());
            }
            _ => {}
        }
        self
    }
    
    /// Builder pattern: Set description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    /// Builder pattern: Add tag
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }
}

/// MCP connection status
#[derive(Debug, Clone, PartialEq)]
pub enum McpConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed { error: String, retry_count: usize },
}

/// MCP operation result with enhanced error information
#[derive(Debug)]
pub struct McpResult<T> {
    pub result: Result<T>,
    pub server_name: String,
    pub operation: String,
    pub duration: std::time::Duration,
}

impl<T> McpResult<T> {
    pub fn new(result: Result<T>, server_name: String, operation: String, duration: std::time::Duration) -> Self {
        Self {
            result,
            server_name,
            operation,
            duration,
        }
    }
    
    pub fn is_ok(&self) -> bool {
        self.result.is_ok()
    }
    
    pub fn is_err(&self) -> bool {
        self.result.is_err()
    }
}

/// Default MCP server configurations for common development tools
pub fn default_mcp_servers() -> Vec<McpServerConfig> {
    vec![
        McpServerConfig::local("filesystem", "mcp-server-filesystem")
            .with_description("Local filesystem access for reading and writing files")
            .with_tag("filesystem")
            .with_tag("core"),
            
        McpServerConfig::docker("github", "ghcr.io/modelcontextprotocol/server-github")
            .with_description("GitHub repository and issue management")
            .with_tag("git")
            .with_tag("github")
            .with_tag("productivity"),
            
        McpServerConfig::docker("postgres", "crystaldba/postgres-mcp")
            .with_arg("--access-mode=restricted")
            .with_description("PostgreSQL database analysis and query assistance")
            .with_tag("database")
            .with_tag("postgresql")
            .with_tag("analysis"),
    ]
}

/// Initialize MCP system with default configuration
pub async fn initialize_mcp() -> Result<McpClientManager> {
    info!("Initializing MCP (Model Context Protocol) subsystem");
    
    // Load configuration
    let config = McpConfig::load().await?;
    
    // Create client manager
    let mut manager = McpClientManager::new(config).await?;
    
    // Auto-discover and connect to enabled servers
    if manager.config().auto_discover {
        manager.discover_and_connect().await?;
    }
    
    info!("MCP subsystem initialized successfully");
    Ok(manager)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_server_config_builder() {
        let config = McpServerConfig::local("test", "test-command")
            .with_arg("--verbose")
            .with_env("DEBUG", "1")
            .with_description("Test server")
            .with_tag("test");
            
        assert_eq!(config.name, "test");
        assert_eq!(config.description, Some("Test server".to_string()));
        assert_eq!(config.tags, vec!["test"]);
        
        match config.server_type {
            McpServerType::Local { command, args, env, .. } => {
                assert_eq!(command, "test-command");
                assert_eq!(args, vec!["--verbose"]);
                assert_eq!(env.get("DEBUG"), Some(&"1".to_string()));
            }
            _ => panic!("Expected Local server type"),
        }
    }
    
    #[test]
    fn test_default_server_configs() {
        let defaults = default_mcp_servers();
        assert_eq!(defaults.len(), 3);
        
        let filesystem = &defaults[0];
        assert_eq!(filesystem.name, "filesystem");
        assert!(filesystem.tags.contains(&"core".to_string()));
        
        let github = &defaults[1];
        assert_eq!(github.name, "github");
        assert!(github.tags.contains(&"github".to_string()));
    }
}