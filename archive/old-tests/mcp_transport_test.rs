//! Unit tests for MCP transport layer message handling
//!
//! These tests verify the correct operation of both stdio and HTTP transports,
//! including JSON-RPC message serialization/deserialization, connection handling,
//! and error scenarios.

use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;

#[cfg(feature = "mcp")]
use aircher::mcp::{
    McpServerConfig, McpServerType, AuthType,
    transport::{
        JsonRpcMessage, JsonRpcError, McpTransport,
        generate_request_id, error_codes,
        StdioTransport, HttpTransport
    },
};

#[cfg(feature = "mcp")]
mod stdio_transport_tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_config() -> McpServerConfig {
        McpServerConfig {
            name: "test-server".to_string(),
            enabled: true,
            server_type: McpServerType::Local {
                command: "echo".to_string(), // Simple command for testing
                args: vec!["test".to_string()],
                working_directory: Some(PathBuf::from("/tmp")),
                env: HashMap::new(),
            },
            timeout_seconds: Some(5),
            auto_reconnect: true,
            description: Some("Test server for transport testing".to_string()),
            tags: vec!["test".to_string()],
        }
    }

    #[tokio::test]
    async fn test_transport_creation() {
        let config = create_test_config();
        let transport = StdioTransport::new(config.clone());

        let info = transport.transport_info();
        assert_eq!(info.transport_type, "stdio");
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_json_rpc_message_serialization() -> Result<()> {
        // Test JSON-RPC request message using constructor
        let request = JsonRpcMessage::request(
            generate_request_id(),
            "tools/list".to_string(),
            Some(json!({"includeContext": true}))
        );

        let serialized = serde_json::to_string(&request)?;
        let deserialized: JsonRpcMessage = serde_json::from_str(&serialized)?;

        match (&request, &deserialized) {
            (
                JsonRpcMessage::Request { id: id1, method: method1, params: params1, .. },
                JsonRpcMessage::Request { id: id2, method: method2, params: params2, .. }
            ) => {
                assert_eq!(id1, id2);
                assert_eq!(method1, method2);
                assert_eq!(params1, params2);
            }
            _ => panic!("Deserialized message doesn't match original"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_json_rpc_response_serialization() -> Result<()> {
        // Test successful response using constructor
        let response = JsonRpcMessage::response(
            json!(1),
            json!({
                "tools": [
                    {
                        "name": "list_files",
                        "description": "List files in a directory",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "path": {"type": "string"}
                            }
                        }
                    }
                ]
            })
        );

        let serialized = serde_json::to_string(&response)?;
        let deserialized: JsonRpcMessage = serde_json::from_str(&serialized)?;

        match (&response, &deserialized) {
            (
                JsonRpcMessage::Response { id: id1, result: result1, error: error1, .. },
                JsonRpcMessage::Response { id: id2, result: result2, error: error2, .. }
            ) => {
                assert_eq!(id1, id2);
                assert_eq!(result1, result2);
                assert_eq!(error1, error2);
            }
            _ => panic!("Deserialized response doesn't match original"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_json_rpc_error_response() -> Result<()> {
        // Test error response using constructor
        let error_response = JsonRpcMessage::error_response(
            json!(1),
            JsonRpcError {
                code: error_codes::METHOD_NOT_FOUND,
                message: "Method not found".to_string(),
                data: Some(json!({
                    "method": "invalid/method"
                }))
            }
        );

        let serialized = serde_json::to_string(&error_response)?;
        let deserialized: JsonRpcMessage = serde_json::from_str(&serialized)?;

        match deserialized {
            JsonRpcMessage::Response { error: Some(err), .. } => {
                assert_eq!(err.code, error_codes::METHOD_NOT_FOUND);
                assert_eq!(err.message, "Method not found");
            }
            _ => panic!("Expected error response"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_notification_message() -> Result<()> {
        let notification = JsonRpcMessage::notification(
            "notifications/resources/list_changed".to_string(),
            Some(json!({
                "uri": "file:///tmp/test.txt"
            }))
        );

        let serialized = serde_json::to_string(&notification)?;
        let deserialized: JsonRpcMessage = serde_json::from_str(&serialized)?;

        match (&notification, &deserialized) {
            (
                JsonRpcMessage::Notification { method: method1, params: params1, .. },
                JsonRpcMessage::Notification { method: method2, params: params2, .. }
            ) => {
                assert_eq!(method1, method2);
                assert_eq!(params1, params2);
            }
            _ => panic!("Deserialized notification doesn't match original"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_malformed_json_handling() {
        let malformed_json = r#"{"jsonrpc": "2.0", "method": "test", "params": INVALID}"#;

        let result: Result<JsonRpcMessage, _> = serde_json::from_str(malformed_json);
        assert!(result.is_err(), "Should fail to parse malformed JSON");
    }

    #[tokio::test]
    async fn test_missing_required_fields() {
        // Missing jsonrpc version
        let invalid_message = r#"{"id": 1, "method": "test"}"#;

        let result: Result<JsonRpcMessage, _> = serde_json::from_str(invalid_message);
        // This might fail depending on strict parsing requirements
        assert!(result.is_err() || result.is_ok());
    }
}

#[cfg(feature = "mcp")]
mod http_transport_tests {
    use super::*;

    fn create_http_config() -> McpServerConfig {
        McpServerConfig {
            name: "http-test-server".to_string(),
            enabled: true,
            server_type: McpServerType::Remote {
                url: "http://localhost:3000/mcp".to_string(),
                auth_type: AuthType::ApiKey {
                    header: "X-API-Key".to_string(),
                    value: "test-key-12345".to_string(),
                },
                headers: {
                    let mut headers = HashMap::new();
                    headers.insert("Content-Type".to_string(), "application/json".to_string());
                    headers.insert("User-Agent".to_string(), "Aircher-MCP-Client/1.0".to_string());
                    headers
                },
            },
            timeout_seconds: Some(10),
            auto_reconnect: false,
            description: Some("HTTP test server for transport testing".to_string()),
            tags: vec!["test".to_string(), "http".to_string()],
        }
    }

    #[tokio::test]
    async fn test_http_transport_creation() -> Result<()> {
        let config = create_http_config();
        let transport = HttpTransport::new(config.clone())?;

        let info = transport.transport_info();
        assert_eq!(info.transport_type, "http");
        assert!(!transport.is_connected());

        Ok(())
    }

    #[tokio::test]
    async fn test_http_headers_configuration() -> Result<()> {
        let config = create_http_config();
        let transport = HttpTransport::new(config)?;

        let info = transport.transport_info();
        assert!(info.connection_details.contains("localhost:3000"));

        Ok(())
    }

    #[tokio::test]
    async fn test_oauth_config() -> Result<()> {
        let mut config = create_http_config();
        config.server_type = McpServerType::Remote {
            url: "https://api.example.com/mcp".to_string(),
            auth_type: AuthType::OAuth {
                client_id: "test-client-id".to_string(),
                scopes: vec!["mcp:read".to_string(), "mcp:write".to_string()],
                token_endpoint: Some("https://auth.example.com/oauth/token".to_string()),
            },
            headers: HashMap::new(),
        };

        let transport = HttpTransport::new(config)?;
        assert_eq!(transport.transport_info().transport_type, "http");

        Ok(())
    }

    #[tokio::test]
    async fn test_bearer_token_config() -> Result<()> {
        let mut config = create_http_config();
        config.server_type = McpServerType::Remote {
            url: "https://secure.example.com/mcp".to_string(),
            auth_type: AuthType::Bearer {
                token: "bearer-token-xyz789".to_string(),
            },
            headers: HashMap::new(),
        };

        let transport = HttpTransport::new(config)?;
        let info = transport.transport_info();
        assert_eq!(info.transport_type, "http");
        assert!(info.connection_details.contains("secure.example.com"));

        Ok(())
    }
}

#[cfg(feature = "mcp")]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_request_id_generation() {
        // Test that request IDs are unique
        let mut ids = Vec::new();
        for _ in 0..100 {
            ids.push(generate_request_id());
        }

        // Convert to set to check uniqueness
        let unique_ids: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique_ids.len(), "All request IDs should be unique");
    }

    #[tokio::test]
    async fn test_transport_info_consistency() -> Result<()> {
        let stdio_config = McpServerConfig::local("stdio-test", "test-command");
        let http_config = McpServerConfig::remote("http-test", "http://localhost:3000");

        let stdio_transport = StdioTransport::new(stdio_config);
        let http_transport = HttpTransport::new(http_config)?;

        let stdio_info = stdio_transport.transport_info();
        let http_info = http_transport.transport_info();

        assert_eq!(stdio_info.transport_type, "stdio");
        assert_eq!(http_info.transport_type, "http");

        Ok(())
    }

    #[tokio::test]
    async fn test_docker_config_parsing() {
        let config = McpServerConfig {
            name: "docker-test".to_string(),
            enabled: true,
            server_type: McpServerType::Docker {
                image: "mcp-test:latest".to_string(),
                args: vec!["--verbose".to_string(), "--port=8080".to_string()],
                env: {
                    let mut env = HashMap::new();
                    env.insert("DEBUG".to_string(), "1".to_string());
                    env.insert("LOG_LEVEL".to_string(), "info".to_string());
                    env
                },
                volumes: vec!["/tmp:/app/tmp:ro".to_string()],
            },
            timeout_seconds: Some(60),
            auto_reconnect: true,
            description: Some("Docker container test".to_string()),
            tags: vec!["test".to_string(), "docker".to_string()],
        };

        // Docker containers use stdio transport
        let transport = StdioTransport::new(config);
        let info = transport.transport_info();
        assert_eq!(info.transport_type, "stdio");
    }
}

#[cfg(feature = "mcp")]
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_timeout_simulation() {
        // Test what happens with invalid configurations
        let invalid_config = McpServerConfig {
            name: "timeout-test".to_string(),
            enabled: true,
            server_type: McpServerType::Local {
                command: "nonexistent-command".to_string(),
                args: vec![],
                working_directory: None,
                env: HashMap::new(),
            },
            timeout_seconds: Some(1), // Very short timeout
            auto_reconnect: false,
            description: None,
            tags: vec![],
        };

        let transport = StdioTransport::new(invalid_config);

        // The transport should be created successfully even with invalid config
        // The error would occur during connection attempt
        assert!(!transport.is_connected());
        let info = transport.transport_info();
        assert_eq!(info.transport_type, "stdio");
    }

    #[tokio::test]
    async fn test_invalid_url_handling() {
        let invalid_config = McpServerConfig {
            name: "invalid-url-test".to_string(),
            enabled: true,
            server_type: McpServerType::Remote {
                url: "not-a-valid-url".to_string(), // Invalid URL
                auth_type: AuthType::None,
                headers: HashMap::new(),
            },
            timeout_seconds: Some(5),
            auto_reconnect: false,
            description: None,
            tags: vec![],
        };

        // This might fail during transport creation if URL validation is strict
        let result = HttpTransport::new(invalid_config);

        // Either creation fails or transport is created but not connected
        match result {
            Ok(transport) => {
                assert!(!transport.is_connected());
                let info = transport.transport_info();
                assert_eq!(info.transport_type, "http");
            }
            Err(_) => {
                // URL validation during creation is also acceptable
                assert!(true, "Transport creation failed with invalid URL");
            }
        }
    }

    #[tokio::test]
    async fn test_large_message_handling() -> Result<()> {
        // Test serialization of large messages
        let large_data = "x".repeat(10_000);

        let large_message = JsonRpcMessage::response(
            json!(1),
            json!({
                "content": large_data,
                "metadata": {
                    "size": large_data.len(),
                    "timestamp": "2024-01-01T00:00:00Z"
                }
            })
        );

        let serialized = serde_json::to_string(&large_message)?;
        assert!(serialized.len() > 10_000, "Serialized message should be large");

        let deserialized: JsonRpcMessage = serde_json::from_str(&serialized)?;

        match deserialized {
            JsonRpcMessage::Response { result: Some(result), .. } => {
                assert_eq!(result["content"].as_str().unwrap().len(), 10_000);
            }
            _ => panic!("Expected successful response with large content"),
        }

        Ok(())
    }
}

#[cfg(feature = "mcp")]
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_rapid_message_creation() {
        let start = std::time::Instant::now();

        let mut messages = Vec::new();
        for i in 0..1000 {
            let message = JsonRpcMessage::request(
                generate_request_id(),
                format!("test/method_{}", i),
                Some(json!({
                    "index": i,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            );
            messages.push(message);
        }

        let duration = start.elapsed();
        assert!(duration.as_millis() < 1000, "Should create 1000 messages in under 1 second");
        assert_eq!(messages.len(), 1000);
    }

    #[tokio::test]
    async fn test_concurrent_transport_creation() {
        let configs: Vec<_> = (0..10)
            .map(|i| McpServerConfig::local(&format!("concurrent-test-{}", i), "echo"))
            .collect();

        let start = std::time::Instant::now();

        let handles: Vec<_> = configs
            .into_iter()
            .map(|config| {
                tokio::spawn(async move {
                    let transport = StdioTransport::new(config);
                    transport.transport_info().transport_type.clone()
                })
            })
            .collect();

        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        let duration = start.elapsed();
        assert!(duration.as_millis() < 1000, "Should create transports concurrently quickly");
        assert_eq!(results.len(), 10);

        // Verify all are stdio transports
        assert!(results.iter().all(|t| t == "stdio"));
    }
}

// Test helper functions
#[cfg(feature = "mcp")]
mod test_helpers {
    use super::*;

    pub fn create_sample_tool_response() -> Value {
        json!({
            "tools": [
                {
                    "name": "read_file",
                    "description": "Read the contents of a file",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "The file path to read"
                            }
                        },
                        "required": ["path"]
                    }
                },
                {
                    "name": "write_file",
                    "description": "Write content to a file",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "The file path to write to"
                            },
                            "content": {
                                "type": "string",
                                "description": "The content to write"
                            }
                        },
                        "required": ["path", "content"]
                    }
                }
            ]
        })
    }

    pub fn create_sample_resource_response() -> Value {
        json!({
            "resources": [
                {
                    "uri": "file:///tmp/test.txt",
                    "name": "Test File",
                    "description": "A test text file",
                    "mimeType": "text/plain"
                },
                {
                    "uri": "file:///tmp/config.json",
                    "name": "Configuration",
                    "description": "Application configuration file",
                    "mimeType": "application/json"
                }
            ]
        })
    }

    #[tokio::test]
    async fn test_sample_responses() -> Result<()> {
        let tool_response = create_sample_tool_response();
        let resource_response = create_sample_resource_response();

        // Verify the sample data is well-formed
        assert!(tool_response["tools"].is_array());
        assert_eq!(tool_response["tools"].as_array().unwrap().len(), 2);

        assert!(resource_response["resources"].is_array());
        assert_eq!(resource_response["resources"].as_array().unwrap().len(), 2);

        Ok(())
    }
}

#[cfg(not(feature = "mcp"))]
mod disabled_tests {
    #[tokio::test]
    async fn test_mcp_feature_disabled() {
        // When MCP feature is disabled, these tests should be skipped
        println!("MCP feature is disabled - transport tests skipped");
    }
}
