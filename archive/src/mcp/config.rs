use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

use crate::config::ArcherConfig;
use super::{McpServerConfig, default_mcp_servers};

// Re-export the parent module types
pub use super::McpServerType;

// Define stub types that were expected
pub struct StdioServerConfig {
    pub command: String,
    pub args: Vec<String>,
}

pub struct HttpServerConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub bearer_token: Option<String>,
}

/// MCP subsystem configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Enable MCP functionality
    pub enabled: bool,

    /// Auto-discover and connect to configured servers on startup
    pub auto_discover: bool,

    /// Default timeout for MCP operations (seconds)
    pub default_timeout_seconds: u64,

    /// Maximum number of concurrent MCP operations
    pub max_concurrent_operations: usize,

    /// Retry configuration for failed connections
    pub retry_config: RetryConfig,

    /// Security settings
    pub security: SecurityConfig,

    /// Configured MCP servers
    pub servers: HashMap<String, McpServerConfig>,

    /// Directory for MCP-related cache and temporary files
    pub cache_dir: Option<PathBuf>,

    /// Logging configuration for MCP operations
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts for failed connections
    pub max_retries: usize,

    /// Base delay between retries in milliseconds
    pub base_delay_ms: u64,

    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,

    /// Use exponential backoff for retries
    pub exponential_backoff: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Require explicit user confirmation for potentially dangerous operations
    pub require_confirmation: bool,

    /// List of allowed MCP server domains (for remote servers)
    pub allowed_domains: Vec<String>,

    /// Maximum response size for MCP operations (bytes)
    pub max_response_size_bytes: usize,

    /// Enable audit logging for all MCP operations
    pub audit_logging: bool,

    /// Directory for storing OAuth tokens securely
    pub token_storage_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log all MCP operations for debugging
    pub log_operations: bool,

    /// Log MCP server responses (potentially sensitive)
    pub log_responses: bool,

    /// Maximum length of logged content
    pub max_log_length: usize,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_discover: true,
            default_timeout_seconds: 30,
            max_concurrent_operations: 10,
            retry_config: RetryConfig::default(),
            security: SecurityConfig::default(),
            servers: HashMap::new(),
            cache_dir: None,
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            exponential_backoff: true,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            require_confirmation: false,
            allowed_domains: vec![
                "api.github.com".to_string(),
                "localhost".to_string(),
                "127.0.0.1".to_string(),
            ],
            max_response_size_bytes: 50 * 1024 * 1024, // 50MB
            audit_logging: true,
            token_storage_dir: None,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_operations: true,
            log_responses: false, // Disabled by default for security
            max_log_length: 1000,
        }
    }
}

impl McpConfig {
    /// Load MCP configuration from disk, creating default if not found
    pub async fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if config_path.exists() {
            info!("Loading MCP configuration from {}", config_path.display());
            let content = fs::read_to_string(&config_path).await?;
            let mut config: McpConfig = toml::from_str(&content)?;

            // Ensure cache directory is set
            if config.cache_dir.is_none() {
                config.cache_dir = Some(Self::default_cache_dir()?);
            }

            // Ensure token storage directory is set
            if config.security.token_storage_dir.is_none() {
                config.security.token_storage_dir = Some(Self::default_token_dir()?);
            }

            Ok(config)
        } else {
            info!("No MCP configuration found, creating default");
            let config = Self::create_default_config().await?;
            config.save().await?;
            Ok(config)
        }
    }

    /// Save current configuration to disk
    pub async fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content).await?;

        info!("MCP configuration saved to {}", config_path.display());
        Ok(())
    }

    /// Create default configuration with common MCP servers
    async fn create_default_config() -> Result<Self> {
        let mut config = Self::default();
        config.cache_dir = Some(Self::default_cache_dir()?);
        config.security.token_storage_dir = Some(Self::default_token_dir()?);

        // Add default server configurations
        for server_config in default_mcp_servers() {
            config.servers.insert(server_config.name.clone(), server_config);
        }

        // Ensure directories exist
        if let Some(cache_dir) = &config.cache_dir {
            fs::create_dir_all(cache_dir).await?;
        }

        if let Some(token_dir) = &config.security.token_storage_dir {
            fs::create_dir_all(token_dir).await?;
        }

        Ok(config)
    }

    /// Get path to MCP configuration file
    fn config_file_path() -> Result<PathBuf> {
        let config_dir = ArcherConfig::config_dir()?;
        Ok(config_dir.join("mcp.toml"))
    }

    /// Get default cache directory for MCP
    fn default_cache_dir() -> Result<PathBuf> {
        let cache_dir = ArcherConfig::cache_dir()?;
        Ok(cache_dir.join("mcp"))
    }

    /// Get default token storage directory
    fn default_token_dir() -> Result<PathBuf> {
        let config_dir = ArcherConfig::config_dir()?;
        Ok(config_dir.join("mcp_tokens"))
    }

    /// Add a new MCP server configuration
    pub fn add_server(&mut self, server_config: McpServerConfig) {
        info!("Adding MCP server configuration: {}", server_config.name);
        self.servers.insert(server_config.name.clone(), server_config);
    }

    /// Remove an MCP server configuration
    pub fn remove_server(&mut self, server_name: &str) -> Option<McpServerConfig> {
        info!("Removing MCP server configuration: {}", server_name);
        self.servers.remove(server_name)
    }

    /// Get server configuration by name
    pub fn get_server(&self, server_name: &str) -> Option<&McpServerConfig> {
        self.servers.get(server_name)
    }

    /// List all enabled server configurations
    pub fn enabled_servers(&self) -> impl Iterator<Item = &McpServerConfig> {
        self.servers.values().filter(|server| server.enabled)
    }

    /// List servers by tag
    pub fn servers_with_tag(&self, tag: &str) -> Vec<&McpServerConfig> {
        let tag_string = tag.to_string();
        self.servers.values()
            .filter(|server| server.tags.contains(&tag_string))
            .collect()
    }

    /// Update security settings
    pub fn update_security(&mut self, security: SecurityConfig) {
        info!("Updating MCP security configuration");
        self.security = security;
    }

    /// Check if a domain is allowed for remote MCP servers
    pub fn is_domain_allowed(&self, domain: &str) -> bool {
        self.security.allowed_domains.contains(&domain.to_string()) ||
        self.security.allowed_domains.iter().any(|allowed| {
            // Support wildcard matching
            allowed.starts_with('*') && domain.ends_with(&allowed[1..])
        })
    }

    /// Get effective timeout for a server
    pub fn effective_timeout(&self, server_config: &McpServerConfig) -> u64 {
        server_config.timeout_seconds.unwrap_or(self.default_timeout_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_config() {
        let config = McpConfig::default();
        assert!(config.enabled);
        assert!(config.auto_discover);
        assert_eq!(config.default_timeout_seconds, 30);
        assert!(config.security.audit_logging);
    }

    #[tokio::test]
    async fn test_domain_validation() {
        let config = McpConfig::default();

        assert!(config.is_domain_allowed("api.github.com"));
        assert!(config.is_domain_allowed("localhost"));
        assert!(!config.is_domain_allowed("evil.com"));
    }

    #[tokio::test]
    async fn test_server_management() {
        let mut config = McpConfig::default();

        let server = McpServerConfig::local("test", "test-command");
        config.add_server(server.clone());

        assert_eq!(config.servers.len(), 1);
        assert!(config.get_server("test").is_some());

        let removed = config.remove_server("test");
        assert!(removed.is_some());
        assert_eq!(config.servers.len(), 0);
    }

    #[tokio::test]
    async fn test_server_filtering() {
        let mut config = McpConfig::default();

        let server1 = McpServerConfig::local("server1", "cmd1").with_tag("database");
        let server2 = McpServerConfig::local("server2", "cmd2").with_tag("filesystem");
        let server3 = McpServerConfig::local("server3", "cmd3").with_tag("database");

        config.add_server(server1);
        config.add_server(server2);
        config.add_server(server3);

        let database_servers = config.servers_with_tag("database");
        assert_eq!(database_servers.len(), 2);

        let filesystem_servers = config.servers_with_tag("filesystem");
        assert_eq!(filesystem_servers.len(), 1);
    }
}
