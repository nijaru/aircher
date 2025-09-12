// TEMPORARY: Commented out until UnifiedAgent is implemented

/*
/// Local client implementation for direct access to UnifiedAgent
/// 
/// This is used by the TUI for optimal performance - no serialization
/// overhead, direct function calls to the agent.

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use serde_json::Value;

use super::{AgentClient, AgentInfo, AgentResponse, ToolCallInfo, ToolStatus};
use crate::agent::unified::UnifiedAgent;
use crate::agent::tools::ToolCall;
use crate::agent::streaming::{AgentStream, create_agent_stream};

/// Local client that directly calls the UnifiedAgent
pub struct LocalClient {
    agent: Arc<UnifiedAgent>,
}

impl LocalClient {
    pub fn new(agent: Arc<UnifiedAgent>) -> Self {
        Self { agent }
    }
}

#[async_trait]
impl AgentClient for LocalClient {
    async fn initialize(&self) -> Result<AgentInfo> {
        // Get agent capabilities directly
        let tools = self.agent.list_available_tools().await?;
        
        Ok(AgentInfo {
            name: "Aircher Agent".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: vec!["tool_calling".to_string(), "streaming".to_string()],
            available_tools: tools,
        })
    }
    
    async fn create_session(&self) -> Result<String> {
        // For local client, we can create sessions directly
        self.agent.create_session().await
    }
    
    async fn send_message(
        &self,
        session_id: String,
        message: String,
    ) -> Result<AgentResponse> {
        // Direct call to agent, no serialization overhead
        let response = self.agent.process_message(&session_id, &message).await?;
        
        Ok(AgentResponse {
            content: response.content,
            tool_calls: response.tool_calls.unwrap_or_default(),
            session_id,
            finish_reason: "completed".to_string(),
        })
    }
    
    async fn stream_message(
        &self,
        session_id: String,
        message: String,
    ) -> Result<AgentStream> {
        // Create streaming response
        let stream = self.agent.stream_process_message(&session_id, &message).await?;
        Ok(stream)
    }
    
    async fn execute_tool(
        &self,
        session_id: String,
        tool_call: ToolCall,
    ) -> Result<Value> {
        // Direct tool execution
        let result = self.agent.execute_tool(&session_id, tool_call).await?;
        Ok(result)
    }
    
    async fn get_tool_status(&self, session_id: String, tool_call_id: String) -> Result<ToolStatus> {
        // Get tool execution status
        let status = self.agent.get_tool_status(&session_id, &tool_call_id).await?;
        Ok(status)
    }
    
    async fn list_tools(&self) -> Result<Vec<ToolCallInfo>> {
        // List available tools with their schemas
        let tools = self.agent.list_available_tools().await?;
        let tool_infos = tools.into_iter().map(|tool| {
            // Map tool info to ToolCallInfo structure
            let tcs: Option<Vec<_>> = None; // This causes the compilation error - need to fix
            tcs.iter().map(|tc| ToolCallInfo {
                id: tool.name.clone(),
                name: tool.name,
                description: tool.description,
                parameters: tool.parameters,
                status: ToolStatus::Available,
            }).collect::<Vec<_>>()
        }).flatten().collect();
        
        Ok(tool_infos)
    }
}
*/