use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::{
    McpConfig, McpClient, McpResult, McpConnectionStatus,
    ToolInfo, ResourceInfo, ResourceContent, HealthStatus
};
use super::client::MockMcpClient;

/// Manager for multiple MCP clients
pub struct McpClientManager {
    config: McpConfig,
    clients: Arc<RwLock<HashMap<String, Box<dyn McpClient>>>>,
    connection_stats: Arc<RwLock<HashMap<String, ConnectionStats>>>,
}

/// Statistics for MCP client connections
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub failed_connections: usize,
    pub total_operations: usize,
    pub successful_operations: usize,
    pub average_response_time_ms: f64,
    pub last_successful_connection: Option<chrono::DateTime<chrono::Utc>>,
    pub last_error: Option<String>,
}

impl Default for ConnectionStats {
    fn default() -> Self {
        Self {
            total_connections: 0,
            failed_connections: 0,
            total_operations: 0,
            successful_operations: 0,
            average_response_time_ms: 0.0,
            last_successful_connection: None,
            last_error: None,
        }
    }
}

/// Summary of all MCP operations across clients
#[derive(Debug, Clone)]
pub struct McpOperationSummary {
    pub server_name: String,
    pub tools_count: usize,
    pub resources_count: usize,
    pub prompts_count: usize,
    pub connection_status: McpConnectionStatus,
    pub stats: ConnectionStats,
}

impl McpClientManager {
    /// Create a new MCP client manager
    pub async fn new(config: McpConfig) -> Result<Self> {
        info!("Creating MCP client manager with {} configured servers", config.servers.len());

        Ok(Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
            connection_stats: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get reference to the configuration
    pub fn config(&self) -> &McpConfig {
        &self.config
    }

    /// Discover and connect to all enabled MCP servers
    pub async fn discover_and_connect(&mut self) -> Result<()> {
        info!("Discovering and connecting to MCP servers");

        // Collect server configs to avoid borrowing issues
        let server_configs: Vec<_> = self.config.servers.values()
            .filter(|config| config.enabled)
            .cloned()
            .collect();

        for server_config in server_configs {
            let server_name = server_config.name.clone();
            info!("Connecting to MCP server: {}", server_name);

            match self.add_client(server_config).await {
                Ok(_) => info!("Successfully added MCP client: {}", server_name),
                Err(e) => {
                    error!("Failed to add MCP client {}: {}", server_name, e);
                    // Update stats for failed connection
                    let mut stats = self.connection_stats.write().await;
                    let server_stats = stats.entry(server_name).or_default();
                    server_stats.failed_connections += 1;
                    server_stats.last_error = Some(e.to_string());
                }
            }
        }

        info!("MCP server discovery completed");
        Ok(())
    }

    /// Add a new MCP client
    pub async fn add_client(&mut self, server_config: super::McpServerConfig) -> Result<()> {
        let server_name = server_config.name.clone();
        info!("Adding MCP client for server: {}", server_name);

        // Create client based on server type (currently using mock for development)
        let mut client = self.create_client(server_config).await?;

        // Attempt to connect
        match client.connect().await {
            Ok(_) => {
                info!("Successfully connected to MCP server: {}", server_name);

                // Update connection stats
                let mut stats = self.connection_stats.write().await;
                let server_stats = stats.entry(server_name.clone()).or_default();
                server_stats.total_connections += 1;
                server_stats.last_successful_connection = Some(chrono::Utc::now());

                // Store the connected client
                let mut clients = self.clients.write().await;
                clients.insert(server_name, client);

                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to MCP server {}: {}", server_name, e);

                // Update stats for failed connection
                let mut stats = self.connection_stats.write().await;
                let server_stats = stats.entry(server_name).or_default();
                server_stats.failed_connections += 1;
                server_stats.last_error = Some(e.to_string());

                Err(e)
            }
        }
    }

    /// Remove an MCP client
    pub async fn remove_client(&mut self, server_name: &str) -> Result<()> {
        info!("Removing MCP client: {}", server_name);

        let mut clients = self.clients.write().await;
        if let Some(mut client) = clients.remove(server_name) {
            // Gracefully disconnect
            if let Err(e) = client.disconnect().await {
                warn!("Error disconnecting from MCP server {}: {}", server_name, e);
            }
        }

        Ok(())
    }

    /// List all available tools across all connected clients
    pub async fn list_all_tools(&self) -> Result<HashMap<String, Vec<ToolInfo>>> {
        let mut all_tools = HashMap::new();
        let clients = self.clients.read().await;

        for (server_name, client) in clients.iter() {
            if !client.is_connected() {
                continue;
            }

            match client.list_tools().await {
                result if result.is_ok() => {
                    if let Ok(tools) = result.result {
                        all_tools.insert(server_name.clone(), tools);
                    }
                }
                result => {
                    warn!("Failed to list tools from server {}: {:?}", server_name, result.result.err());
                }
            }
        }

        Ok(all_tools)
    }

    /// List all available resources across all connected clients
    pub async fn list_all_resources(&self) -> Result<HashMap<String, Vec<ResourceInfo>>> {
        let mut all_resources = HashMap::new();
        let clients = self.clients.read().await;

        for (server_name, client) in clients.iter() {
            if !client.is_connected() {
                continue;
            }

            match client.list_resources().await {
                result if result.is_ok() => {
                    if let Ok(resources) = result.result {
                        all_resources.insert(server_name.clone(), resources);
                    }
                }
                result => {
                    warn!("Failed to list resources from server {}: {:?}", server_name, result.result.err());
                }
            }
        }

        Ok(all_resources)
    }

    /// Call a tool on a specific server
    pub async fn call_tool(&self, server_name: &str, tool_name: &str, parameters: Value) -> Result<McpResult<Value>> {
        let clients = self.clients.read().await;

        let client = clients.get(server_name)
            .ok_or_else(|| anyhow::anyhow!("MCP server not found: {}", server_name))?;

        if !client.is_connected() {
            return Err(anyhow::anyhow!("MCP server not connected: {}", server_name));
        }

        info!("Calling tool '{}' on server '{}'", tool_name, server_name);
        let result = client.call_tool(tool_name, parameters).await;

        // Update operation stats
        self.update_operation_stats(server_name, &result).await;

        Ok(result)
    }

    /// Get a resource from a specific server
    pub async fn get_resource(&self, server_name: &str, uri: &str) -> Result<McpResult<ResourceContent>> {
        let clients = self.clients.read().await;

        let client = clients.get(server_name)
            .ok_or_else(|| anyhow::anyhow!("MCP server not found: {}", server_name))?;

        if !client.is_connected() {
            return Err(anyhow::anyhow!("MCP server not connected: {}", server_name));
        }

        info!("Getting resource '{}' from server '{}'", uri, server_name);
        let result = client.get_resource(uri).await;

        // Update operation stats
        self.update_operation_stats(server_name, &result).await;

        Ok(result)
    }

    /// Execute a prompt on a specific server
    pub async fn execute_prompt(&self, server_name: &str, prompt_name: &str, arguments: Value) -> Result<McpResult<Value>> {
        let clients = self.clients.read().await;

        let client = clients.get(server_name)
            .ok_or_else(|| anyhow::anyhow!("MCP server not found: {}", server_name))?;

        if !client.is_connected() {
            return Err(anyhow::anyhow!("MCP server not connected: {}", server_name));
        }

        info!("Executing prompt '{}' on server '{}'", prompt_name, server_name);
        let result = client.execute_prompt(prompt_name, arguments).await;

        // Update operation stats
        self.update_operation_stats(server_name, &result).await;

        Ok(result)
    }

    /// Get comprehensive status of all MCP clients
    pub async fn get_status(&self) -> Result<Vec<McpOperationSummary>> {
        let clients = self.clients.read().await;
        let stats = self.connection_stats.read().await;
        let mut summaries = Vec::new();

        for (server_name, client) in clients.iter() {
            let connection_status = client.connection_status();
            let server_stats = stats.get(server_name).cloned().unwrap_or_default();

            // Get capabilities if connected
            let (tools_count, resources_count, prompts_count) = if client.is_connected() {
                let tools = client.list_tools().await.result.unwrap_or_default().len();
                let resources = client.list_resources().await.result.unwrap_or_default().len();
                let prompts = client.list_prompts().await.result.unwrap_or_default().len();
                (tools, resources, prompts)
            } else {
                (0, 0, 0)
            };

            summaries.push(McpOperationSummary {
                server_name: server_name.clone(),
                tools_count,
                resources_count,
                prompts_count,
                connection_status,
                stats: server_stats,
            });
        }

        Ok(summaries)
    }

    /// Health check all connected servers
    pub async fn health_check_all(&self) -> Result<HashMap<String, HealthStatus>> {
        let mut health_results = HashMap::new();
        let clients = self.clients.read().await;

        for (server_name, client) in clients.iter() {
            if client.is_connected() {
                match client.health_check().await {
                    result if result.is_ok() => {
                        if let Ok(health) = result.result {
                            health_results.insert(server_name.clone(), health);
                        }
                    }
                    result => {
                        warn!("Health check failed for server {}: {:?}", server_name, result.result.err());
                    }
                }
            }
        }

        Ok(health_results)
    }

    /// Find tools by name across all servers
    pub async fn find_tools_by_name(&self, tool_name: &str) -> Result<Vec<(String, ToolInfo)>> {
        let all_tools = self.list_all_tools().await?;
        let mut matching_tools = Vec::new();

        for (server_name, tools) in all_tools {
            for tool in tools {
                if tool.name == tool_name || tool.name.contains(tool_name) {
                    matching_tools.push((server_name.clone(), tool));
                }
            }
        }

        Ok(matching_tools)
    }

    /// Get servers that support a specific capability
    pub async fn servers_with_tag(&self, tag: &str) -> Vec<String> {
        self.config.servers_with_tag(tag)
            .into_iter()
            .filter(|server| server.enabled)
            .map(|server| server.name.clone())
            .collect()
    }

    /// Connect to a specific server
    pub async fn connect_server(&mut self, server_name: &str) -> Result<()> {
        let server_config = self.config.servers.get(server_name)
            .ok_or_else(|| anyhow::anyhow!("Server '{}' not found in configuration", server_name))?
            .clone();

        self.add_client(server_config).await
    }

    /// Disconnect from a specific server
    pub async fn disconnect_server(&mut self, server_name: &str) -> Result<()> {
        info!("Disconnecting from MCP server: {}", server_name);

        let mut clients = self.clients.write().await;
        if let Some(mut client) = clients.remove(server_name) {
            client.disconnect().await?;
            info!("Disconnected from server: {}", server_name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Server '{}' not connected", server_name))
        }
    }

    /// Get connection status for a specific server
    pub async fn get_connection_status(&self, server_name: &str) -> Option<McpConnectionStatus> {
        let clients = self.clients.read().await;
        clients.get(server_name).map(|c| c.connection_status())
    }

    /// List all connected servers
    pub async fn list_connected_servers(&self) -> Vec<String> {
        let clients = self.clients.read().await;
        clients.iter()
            .filter(|(_, c)| c.is_connected())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// List tools from a specific server
    pub async fn list_tools(&self, server_name: &str) -> Result<Option<Vec<ToolInfo>>> {
        let clients = self.clients.read().await;

        if let Some(client) = clients.get(server_name) {
            if client.is_connected() {
                let result = client.list_tools().await;
                Ok(result.result.ok())
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// List resources from a specific server
    pub async fn list_resources(&self, server_name: &str) -> Result<Option<Vec<ResourceInfo>>> {
        let clients = self.clients.read().await;

        if let Some(client) = clients.get(server_name) {
            if client.is_connected() {
                let result = client.list_resources().await;
                Ok(result.result.ok())
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Disconnect all clients
    pub async fn disconnect_all(&mut self) -> Result<()> {
        info!("Disconnecting all MCP clients");

        let mut clients = self.clients.write().await;
        for (server_name, client) in clients.iter_mut() {
            info!("Disconnecting from MCP server: {}", server_name);
            if let Err(e) = client.disconnect().await {
                warn!("Error disconnecting from server {}: {}", server_name, e);
            }
        }

        clients.clear();
        Ok(())
    }

    /// Create an MCP client for the given server configuration
    async fn create_client(&self, server_config: super::McpServerConfig) -> Result<Box<dyn McpClient>> {
        #[cfg(debug_assertions)]
        {
            // Use mock clients in debug builds for development
            info!("Creating mock MCP client for server: {} (debug mode)", server_config.name);
            Ok(Box::new(MockMcpClient::new(server_config)))
        }

        #[cfg(not(debug_assertions))]
        {
            // Use real clients in release builds
            info!("Creating real MCP client for server: {}", server_config.name);
            use super::real_client::RealMcpClient;
            let client = RealMcpClient::new(server_config)?;
            Ok(Box::new(client))
        }
    }

    /// Update operation statistics for a server
    async fn update_operation_stats<T>(&self, server_name: &str, result: &McpResult<T>) {
        let mut stats = self.connection_stats.write().await;
        let server_stats = stats.entry(server_name.to_string()).or_default();

        server_stats.total_operations += 1;

        if result.is_ok() {
            server_stats.successful_operations += 1;
        } else if let Err(ref e) = result.result {
            server_stats.last_error = Some(e.to_string());
        }

        // Update average response time
        let response_time_ms = result.duration.as_millis() as f64;
        if server_stats.total_operations == 1 {
            server_stats.average_response_time_ms = response_time_ms;
        } else {
            // Running average
            server_stats.average_response_time_ms =
                (server_stats.average_response_time_ms * (server_stats.total_operations - 1) as f64 + response_time_ms)
                / server_stats.total_operations as f64;
        }
    }
}

impl Drop for McpClientManager {
    fn drop(&mut self) {
        // Note: We can't call async functions in Drop, so we just log
        debug!("McpClientManager is being dropped - clients should be disconnected explicitly");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::{McpConfig, McpServerConfig};

    #[tokio::test]
    async fn test_manager_creation() {
        let config = McpConfig::default();
        let manager = McpClientManager::new(config).await.unwrap();
        assert_eq!(manager.config().enabled, true);
    }

    #[tokio::test]
    async fn test_add_and_remove_client() {
        let config = McpConfig::default();
        let mut manager = McpClientManager::new(config).await.unwrap();

        let server_config = McpServerConfig::local("test", "test-command");

        // Add client
        manager.add_client(server_config).await.unwrap();

        // Verify it was added
        let clients = manager.clients.read().await;
        assert!(clients.contains_key("test"));
        drop(clients);

        // Remove client
        manager.remove_client("test").await.unwrap();

        let clients = manager.clients.read().await;
        assert!(!clients.contains_key("test"));
    }

    #[tokio::test]
    async fn test_tool_operations() {
        let config = McpConfig::default();
        let mut manager = McpClientManager::new(config).await.unwrap();

        let server_config = McpServerConfig::local("test", "test-command");
        manager.add_client(server_config).await.unwrap();

        // Test tool call
        let result = manager.call_tool("test", "ping", serde_json::json!({})).await;
        assert!(result.is_ok());

        let mcp_result = result.unwrap();
        assert!(mcp_result.is_ok());
    }
}
