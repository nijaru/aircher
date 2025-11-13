//! Integration tests for MCP (Model Context Protocol) functionality
//!
//! These tests verify that the MCP client manager can properly orchestrate
//! multiple mock MCP servers and provide a working foundation for real MCP integration.

use anyhow::Result;
use serde_json::json;
use tokio::time::Duration;

#[cfg(feature = "mcp")]
use aircher::mcp::{
    initialize_mcp, McpConfig, McpServerConfig, McpClientManager,
    McpConnectionStatus, AuthType,
};

#[cfg(feature = "mcp")]
#[tokio::test]
async fn test_mcp_initialization() -> Result<()> {
    // Test the main initialization function
    let manager = initialize_mcp().await?;

    // Should have default configuration
    assert!(manager.config().enabled);
    assert!(manager.config().auto_discover);

    Ok(())
}

#[cfg(feature = "mcp")]
#[tokio::test]
async fn test_mcp_client_lifecycle() -> Result<()> {
    let config = McpConfig::default();
    let mut manager = McpClientManager::new(config).await?;

    // Create and add a mock filesystem server
    let filesystem_server = McpServerConfig::local("filesystem", "mcp-server-filesystem")
        .with_description("Mock filesystem server for testing")
        .with_tag("filesystem")
        .with_tag("test");

    // Add client should succeed
    manager.add_client(filesystem_server).await?;

    // Verify the client is connected
    let status = manager.get_status().await?;
    assert_eq!(status.len(), 1);
    assert_eq!(status[0].server_name, "filesystem");
    assert_eq!(status[0].connection_status, McpConnectionStatus::Connected);

    // Test tool operations
    let tools = manager.list_all_tools().await?;
    assert!(tools.contains_key("filesystem"));
    assert!(!tools["filesystem"].is_empty());

    // Test calling a tool
    let result = manager.call_tool("filesystem", "ping", json!({})).await?;
    assert!(result.is_ok());

    // Test getting resources
    let resources = manager.list_all_resources().await?;
    assert!(resources.contains_key("filesystem"));

    // Clean up
    manager.remove_client("filesystem").await?;
    let status_after_removal = manager.get_status().await?;
    assert_eq!(status_after_removal.len(), 0);

    Ok(())
}

#[cfg(feature = "mcp")]
#[tokio::test]
async fn test_multiple_mcp_clients() -> Result<()> {
    let mut config = McpConfig::default();

    // Add multiple mock servers to configuration
    let servers = vec![
        McpServerConfig::local("filesystem", "mcp-server-filesystem")
            .with_tag("filesystem"),
        McpServerConfig::docker("github", "ghcr.io/modelcontextprotocol/server-github")
            .with_tag("github"),
        McpServerConfig::local("postgres", "postgres-mcp")
            .with_tag("database"),
    ];

    // Add servers to configuration
    for server in servers {
        config.add_server(server);
    }

    let mut manager = McpClientManager::new(config).await?;

    // Connect to all configured servers
    manager.discover_and_connect().await?;

    // Verify all are connected
    let status = manager.get_status().await?;
    assert_eq!(status.len(), 3);

    // Test operations across all servers
    let all_tools = manager.list_all_tools().await?;

    // Debug output for troubleshooting
    println!("Connected servers: {:?}", all_tools.keys().collect::<Vec<_>>());
    for (server_name, tools) in &all_tools {
        println!("Server '{}' has {} tools: {:?}", server_name, tools.len(), tools.iter().map(|t| &t.name).collect::<Vec<_>>());
    }

    // We expect exactly 3 servers with tools
    assert_eq!(all_tools.len(), 3, "Expected 3 servers with tools, got {}: {:?}", all_tools.len(), all_tools.keys().collect::<Vec<_>>());

    // Each server should have at least the ping tool
    for (server_name, tools) in &all_tools {
        assert!(!tools.is_empty(), "Server {} should have tools", server_name);
        assert!(tools.iter().any(|t| t.name == "ping"), "Server {} should have ping tool", server_name);
    }

    // Test server-specific tools
    assert!(all_tools["filesystem"].iter().any(|t| t.name == "read_file"));
    assert!(all_tools["github"].iter().any(|t| t.name == "get_repository"));
    assert!(all_tools["postgres"].iter().any(|t| t.name == "query_database"));

    // Test tool search functionality
    let ping_tools = manager.find_tools_by_name("ping").await?;
    assert_eq!(ping_tools.len(), 3); // All servers should have ping

    let read_file_tools = manager.find_tools_by_name("read_file").await?;
    assert_eq!(read_file_tools.len(), 1); // Only filesystem server

    // Test server filtering by tags
    let database_servers = manager.servers_with_tag("database").await;
    assert_eq!(database_servers, vec!["postgres"]);

    let github_servers = manager.servers_with_tag("github").await;
    assert_eq!(github_servers, vec!["github"]);

    // Test health checks
    let health_results = manager.health_check_all().await?;
    assert_eq!(health_results.len(), 3);

    for (server_name, health) in health_results {
        assert_eq!(health.status, "healthy", "Server {} should be healthy", server_name);
    }

    Ok(())
}

#[cfg(feature = "mcp")]
#[tokio::test]
async fn test_mcp_error_handling() -> Result<()> {
    let config = McpConfig::default();
    let mut manager = McpClientManager::new(config).await?;

    // Add a client
    let server = McpServerConfig::local("test", "test-command");
    manager.add_client(server).await?;

    // Test calling a non-existent tool
    let result = manager.call_tool("test", "nonexistent_tool", json!({})).await?;
    assert!(result.is_err());

    // Test operations on non-existent server
    let result = manager.call_tool("nonexistent_server", "ping", json!({}));
    assert!(result.await.is_err());

    // Test getting resource from non-existent server
    let result = manager.get_resource("nonexistent_server", "test://uri");
    assert!(result.await.is_err());

    Ok(())
}

#[cfg(feature = "mcp")]
#[tokio::test]
async fn test_mcp_server_config_builders() -> Result<()> {
    // Test local server configuration
    let local_server = McpServerConfig::local("local_test", "test-command")
        .with_arg("--verbose")
        .with_arg("--debug")
        .with_env("DEBUG", "1")
        .with_env("LOG_LEVEL", "trace")
        .with_description("Test local server")
        .with_tag("test")
        .with_tag("development");

    assert_eq!(local_server.name, "local_test");
    assert_eq!(local_server.description, Some("Test local server".to_string()));
    assert_eq!(local_server.tags, vec!["test", "development"]);
    assert!(local_server.enabled);
    assert!(local_server.auto_reconnect);

    // Test Docker server configuration
    let docker_server = McpServerConfig::docker("docker_test", "test/image:latest")
        .with_arg("--config=/app/config.json")
        .with_env("CONTAINER_ENV", "test")
        .with_tag("docker");

    assert_eq!(docker_server.name, "docker_test");
    assert!(docker_server.tags.contains(&"docker".to_string()));

    // Test remote server configuration
    let remote_server = McpServerConfig::remote("remote_test", "https://api.example.com/mcp");

    assert_eq!(remote_server.name, "remote_test");
    match remote_server.server_type {
        aircher::mcp::McpServerType::Remote { url, auth_type, .. } => {
            assert_eq!(url, "https://api.example.com/mcp");
            assert!(matches!(auth_type, AuthType::None));
        },
        _ => panic!("Expected Remote server type"),
    }

    Ok(())
}

#[cfg(feature = "mcp")]
#[tokio::test]
async fn test_mcp_operation_timing_and_stats() -> Result<()> {
    let config = McpConfig::default();
    let mut manager = McpClientManager::new(config).await?;

    // Add a client
    let server = McpServerConfig::local("timing_test", "test-command");
    manager.add_client(server).await?;

    // Perform several operations
    for i in 0..5 {
        let result = manager.call_tool("timing_test", "ping", json!({"iteration": i})).await?;
        assert!(result.is_ok());
        assert!(result.duration > Duration::from_nanos(0));
    }

    // Check operation statistics
    let status = manager.get_status().await?;
    assert_eq!(status.len(), 1);

    let server_status = &status[0];
    assert_eq!(server_status.server_name, "timing_test");
    assert!(server_status.stats.total_operations >= 5);
    assert!(server_status.stats.successful_operations >= 5);
    // Mock operations may complete very quickly, so just check they're not negative
    assert!(server_status.stats.average_response_time_ms >= 0.0);
    assert!(server_status.stats.last_successful_connection.is_some());

    Ok(())
}

#[cfg(not(feature = "mcp"))]
#[test]
fn test_mcp_feature_disabled() {
    // When MCP feature is disabled, this test ensures the feature flag works
    println!("MCP feature is disabled - skipping MCP integration tests");
}
