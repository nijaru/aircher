use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::{Client, header::{HeaderMap, HeaderName, HeaderValue}};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use tracing::{debug, error, info};

use super::{McpTransport, TransportInfo, JsonRpcMessage, generate_request_id};
use crate::mcp::{McpServerConfig, AuthType};

/// HTTP+SSE transport implementation for remote MCP servers
pub struct HttpTransport {
    config: McpServerConfig,
    client: Client,
    connected: Arc<AtomicBool>,
    base_url: String,
    headers: HeaderMap,
}

impl HttpTransport {
    pub fn new(config: McpServerConfig) -> Result<Self> {
        let (url, auth_type, custom_headers) = match &config.server_type {
            crate::mcp::McpServerType::Remote { url, auth_type, headers } => {
                (url.clone(), auth_type.clone(), headers.clone())
            }
            _ => return Err(anyhow!("HTTP transport only supports Remote server types")),
        };

        let mut headers = Self::build_headers(&auth_type, &custom_headers)?;
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("Accept", HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds.unwrap_or(30)))
            .user_agent("Aircher MCP Client/1.0")
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            config,
            client,
            connected: Arc::new(AtomicBool::new(false)),
            base_url: url,
            headers,
        })
    }

    /// Build HTTP headers from authentication configuration
    fn build_headers(auth_type: &AuthType, custom_headers: &HashMap<String, String>) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        // Add authentication headers
        match auth_type {
            AuthType::None => {},
            AuthType::Bearer { token } => {
                let header_value = HeaderValue::from_str(&format!("Bearer {}", token))
                    .map_err(|e| anyhow!("Invalid bearer token: {}", e))?;
                headers.insert("Authorization", header_value);
            }
            AuthType::ApiKey { header, value } => {
                let header_name = HeaderName::from_bytes(header.as_bytes())
                    .map_err(|e| anyhow!("Invalid header name '{}': {}", header, e))?;
                let header_value = HeaderValue::from_str(value)
                    .map_err(|e| anyhow!("Invalid header value for '{}': {}", header, e))?;
                headers.insert(header_name, header_value);
            }
            AuthType::OAuth { .. } => {
                // OAuth implementation would require token exchange flow
                return Err(anyhow!("OAuth authentication not yet implemented for HTTP transport"));
            }
        }

        // Add custom headers
        for (name, value) in custom_headers {
            let header_name = HeaderName::from_bytes(name.as_bytes())
                .map_err(|e| anyhow!("Invalid custom header name '{}': {}", name, e))?;
            let header_value = HeaderValue::from_str(value)
                .map_err(|e| anyhow!("Invalid custom header value for '{}': {}", name, e))?;
            headers.insert(header_name, header_value);
        }

        Ok(headers)
    }

    /// Get the endpoint URL for a specific operation
    fn get_endpoint_url(&self, endpoint: &str) -> String {
        if self.base_url.ends_with('/') {
            format!("{}{}", self.base_url, endpoint)
        } else {
            format!("{}/{}", self.base_url, endpoint)
        }
    }

    /// Send an HTTP request to the MCP server
    async fn send_http_request(&self, endpoint: &str, body: Value) -> Result<Value> {
        let url = self.get_endpoint_url(endpoint);

        debug!("Sending HTTP request to MCP server '{}': {} -> {}",
               self.config.name, endpoint, serde_json::to_string(&body).unwrap_or_default());

        let response = self.client
            .post(&url)
            .headers(self.headers.clone())
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("HTTP request failed to '{}': {}", url, e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "HTTP request to '{}' failed with status {}: {}",
                url,
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let response_text = response.text().await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

        debug!("Received HTTP response from MCP server '{}': {}",
               self.config.name, response_text);

        serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse response JSON: {}", e))
    }

    /// Test connection to the MCP server
    async fn test_connection(&self) -> Result<()> {
        // Try to call the server info endpoint or a health check
        let health_request = JsonRpcMessage::request(
            generate_request_id(),
            "initialize".to_string(),
            Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "Aircher",
                    "version": "1.0.0"
                }
            }))
        );

        match self.send_http_request("rpc", serde_json::to_value(&health_request)?).await {
            Ok(response) => {
                debug!("Connection test successful for MCP server '{}': {:?}", self.config.name, response);
                Ok(())
            }
            Err(e) => {
                error!("Connection test failed for MCP server '{}': {}", self.config.name, e);
                Err(e)
            }
        }
    }
}

#[async_trait]
impl McpTransport for HttpTransport {
    async fn connect(&mut self) -> Result<()> {
        if self.is_connected() {
            return Ok(());
        }

        info!("Connecting to MCP server '{}' via HTTP at {}", self.config.name, self.base_url);

        // Test the connection
        self.test_connection().await?;

        self.connected.store(true, Ordering::SeqCst);

        info!("Successfully connected to MCP server '{}' via HTTP", self.config.name);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if !self.is_connected() {
            return Ok(());
        }

        info!("Disconnecting from MCP server '{}'", self.config.name);

        self.connected.store(false, Ordering::SeqCst);

        info!("Disconnected from MCP server '{}'", self.config.name);
        Ok(())
    }

    async fn send_request(&self, method: &str, params: Value) -> Result<Value> {
        if !self.is_connected() {
            return Err(anyhow!("Not connected to MCP server"));
        }

        let request = JsonRpcMessage::request(
            generate_request_id(),
            method.to_string(),
            Some(params)
        );

        let response_value = self.send_http_request("rpc", serde_json::to_value(&request)?).await?;

        // Parse the JSON-RPC response
        match serde_json::from_value::<JsonRpcMessage>(response_value)? {
            JsonRpcMessage::Response { result, error, .. } => {
                if let Some(error) = error {
                    Err(anyhow!("MCP server error: {} ({})", error.message, error.code))
                } else {
                    Ok(result.unwrap_or(Value::Null))
                }
            }
            _ => Err(anyhow!("Unexpected response type from MCP server")),
        }
    }

    async fn send_notification(&self, method: &str, params: Value) -> Result<()> {
        if !self.is_connected() {
            return Err(anyhow!("Not connected to MCP server"));
        }

        let notification = JsonRpcMessage::notification(method.to_string(), Some(params));

        // Send notification - we don't expect a response
        self.send_http_request("notify", serde_json::to_value(&notification)?).await?;

        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    fn transport_info(&self) -> TransportInfo {
        TransportInfo {
            transport_type: "http+sse".to_string(),
            connection_details: self.base_url.clone(),
            supports_notifications: true,
            max_concurrent_requests: Some(100), // Reasonable default for HTTP
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::McpServerConfig;

    #[tokio::test]
    async fn test_http_transport_creation() {
        let config = McpServerConfig::remote("test", "https://api.example.com/mcp");
        let transport = HttpTransport::new(config).unwrap();

        assert!(!transport.is_connected());
        assert_eq!(transport.transport_info().transport_type, "http+sse");
        assert_eq!(transport.base_url, "https://api.example.com/mcp");
    }

    #[test]
    fn test_header_building() {
        let auth = AuthType::Bearer { token: "test_token".to_string() };
        let mut custom_headers = HashMap::new();
        custom_headers.insert("X-Custom".to_string(), "custom_value".to_string());

        let headers = HttpTransport::build_headers(&auth, &custom_headers).unwrap();

        assert!(headers.contains_key("authorization"));
        assert!(headers.contains_key("x-custom"));
    }

    #[test]
    fn test_endpoint_url_construction() {
        let config = McpServerConfig::remote("test", "https://api.example.com/mcp/");
        let transport = HttpTransport::new(config).unwrap();

        assert_eq!(transport.get_endpoint_url("rpc"), "https://api.example.com/mcp/rpc");

        let config2 = McpServerConfig::remote("test2", "https://api.example.com/mcp");
        let transport2 = HttpTransport::new(config2).unwrap();

        assert_eq!(transport2.get_endpoint_url("rpc"), "https://api.example.com/mcp/rpc");
    }
}
