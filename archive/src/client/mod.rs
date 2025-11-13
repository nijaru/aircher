/// Client abstraction layer for accessing the UnifiedAgent
///
/// This allows different frontends (TUI, CLI, tests) to access the agent
/// through a consistent interface, whether locally or remotely.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::agent::streaming::AgentStream;

pub mod local;
// pub mod remote; // Future: for network access

/// Information about the agent's capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub available_tools: Vec<String>,
}

/// Response from the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCallInfo>,
    pub session_id: String,
}

/// Information about a tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub name: String,
    pub status: ToolStatus,
    pub result: Option<Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolStatus {
    Pending,
    Running,
    Success,
    Failed,
}

/// Client trait for accessing the agent
#[async_trait]
pub trait AgentClient: Send + Sync {
    /// Initialize connection to the agent
    async fn initialize(&self) -> Result<AgentInfo>;

    /// Create a new conversation session
    async fn create_session(&self) -> Result<String>;

    /// Send a prompt to the agent
    async fn send_prompt(
        &self,
        session_id: &str,
        message: String,
        provider: Option<String>,
        model: Option<String>,
    ) -> Result<AgentResponse>;

    /// Send a prompt with streaming response
    async fn send_prompt_streaming(
        &self,
        session_id: &str,
        message: String,
        provider: Option<String>,
        model: Option<String>,
    ) -> Result<AgentStream>;

    /// Execute a specific tool
    async fn execute_tool(
        &self,
        tool_name: String,
        params: Value,
    ) -> Result<ToolCallInfo>;

    /// Get session history
    async fn get_session_history(&self, session_id: &str) -> Result<Vec<AgentResponse>>;

    /// End a session
    async fn end_session(&self, session_id: &str) -> Result<()>;
}
