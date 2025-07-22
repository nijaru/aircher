use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

pub mod stdio;
pub mod http;

pub use stdio::StdioTransport;
pub use http::HttpTransport;

/// Transport layer abstraction for MCP communication
#[async_trait]
pub trait McpTransport: Send + Sync {
    /// Initialize the transport connection
    async fn connect(&mut self) -> Result<()>;
    
    /// Close the transport connection
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Send a JSON-RPC request and wait for response
    async fn send_request(&self, method: &str, params: Value) -> Result<Value>;
    
    /// Send a JSON-RPC notification (no response expected)
    async fn send_notification(&self, method: &str, params: Value) -> Result<()>;
    
    /// Check if transport is currently connected
    fn is_connected(&self) -> bool;
    
    /// Get transport-specific configuration info
    fn transport_info(&self) -> TransportInfo;
}

/// Information about a transport implementation
#[derive(Debug, Clone)]
pub struct TransportInfo {
    pub transport_type: String,
    pub connection_details: String,
    pub supports_notifications: bool,
    pub max_concurrent_requests: Option<usize>,
}

/// JSON-RPC message types for MCP protocol
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request {
        jsonrpc: String,
        id: Value,
        method: String,
        params: Option<Value>,
    },
    Response {
        jsonrpc: String,
        id: Value,
        result: Option<Value>,
        error: Option<JsonRpcError>,
    },
    Notification {
        jsonrpc: String,
        method: String,
        params: Option<Value>,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

impl JsonRpcMessage {
    pub fn request(id: Value, method: String, params: Option<Value>) -> Self {
        Self::Request {
            jsonrpc: "2.0".to_string(),
            id,
            method,
            params,
        }
    }
    
    pub fn response(id: Value, result: Value) -> Self {
        Self::Response {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }
    
    pub fn error_response(id: Value, error: JsonRpcError) -> Self {
        Self::Response {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
    
    pub fn notification(method: String, params: Option<Value>) -> Self {
        Self::Notification {
            jsonrpc: "2.0".to_string(),
            method,
            params,
        }
    }
}

/// Common JSON-RPC error codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    
    // MCP-specific error codes
    pub const TRANSPORT_ERROR: i32 = -32001;
    pub const TIMEOUT_ERROR: i32 = -32002;
    pub const CONNECTION_ERROR: i32 = -32003;
}

/// Utility functions for transport implementations
pub fn generate_request_id() -> Value {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    Value::Number(serde_json::Number::from(COUNTER.fetch_add(1, Ordering::SeqCst)))
}

pub fn is_notification(message: &JsonRpcMessage) -> bool {
    matches!(message, JsonRpcMessage::Notification { .. })
}

pub fn is_response(message: &JsonRpcMessage) -> bool {
    matches!(message, JsonRpcMessage::Response { .. })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_json_rpc_message_creation() {
        let req = JsonRpcMessage::request(
            Value::Number(serde_json::Number::from(1u64)),
            "test_method".to_string(),
            Some(serde_json::json!({"param": "value"}))
        );
        
        match req {
            JsonRpcMessage::Request { jsonrpc, id, method, params } => {
                assert_eq!(jsonrpc, "2.0");
                assert_eq!(method, "test_method");
                assert!(params.is_some());
            }
            _ => panic!("Expected Request variant"),
        }
    }
    
    #[test]
    fn test_request_id_generation() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_message_type_checks() {
        let notif = JsonRpcMessage::notification("test".to_string(), None);
        let resp = JsonRpcMessage::response(Value::Number(serde_json::Number::from(1u64)), serde_json::json!("ok"));
        
        assert!(is_notification(&notif));
        assert!(!is_notification(&resp));
        assert!(is_response(&resp));
        assert!(!is_response(&notif));
    }
}