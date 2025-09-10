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
        let available_tools: Vec<String> = self.agent.tools.list_tools()
            .into_iter()
            .map(|t| t.name)
            .collect();
        
        Ok(AgentInfo {
            name: "Aircher Agent".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: vec![
                "tools".to_string(),
                "streaming".to_string(),
                "multi-provider".to_string(),
                "intelligence".to_string(),
            ],
            available_tools,
        })
    }
    
    async fn create_session(&self) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        self.agent.create_session(session_id.clone()).await?;
        Ok(session_id)
    }
    
    async fn send_prompt(
        &self,
        session_id: &str,
        message: String,
        provider: Option<String>,
        model: Option<String>,
    ) -> Result<AgentResponse> {
        // Direct call to agent's process_prompt
        let response_content = self.agent.process_prompt(
            session_id,
            message,
            provider.as_deref(),
            model.as_deref(),
        ).await?;
        
        // Parse any tool calls from the response
        let tool_calls = self.agent.parser.parse(&response_content);
        
        // Convert to client format
        let tool_call_info: Vec<ToolCallInfo> = tool_calls.iter().map(|tc| {
            ToolCallInfo {
                name: tc.name.clone(),
                status: ToolStatus::Success, // TODO: Track actual status
                result: Some(tc.parameters.clone()),
                error: None,
            }
        }).collect();
        
        Ok(AgentResponse {
            content: response_content,
            tool_calls: tool_call_info,
            session_id: session_id.to_string(),
        })
    }
    
    async fn execute_tool(
        &self,
        tool_name: String,
        params: Value,
    ) -> Result<ToolCallInfo> {
        let tool_call = ToolCall {
            name: tool_name.clone(),
            parameters: params,
        };
        
        match self.agent.execute_tool(&tool_call).await {
            Ok(result) => Ok(ToolCallInfo {
                name: tool_name,
                status: ToolStatus::Success,
                result: Some(result),
                error: None,
            }),
            Err(e) => Ok(ToolCallInfo {
                name: tool_name,
                status: ToolStatus::Failed,
                result: None,
                error: Some(e.to_string()),
            }),
        }
    }
    
    async fn get_session_history(&self, session_id: &str) -> Result<Vec<AgentResponse>> {
        let sessions = self.agent.sessions.read().await;
        
        if let Some(session) = sessions.get(session_id) {
            // Convert conversation messages to client format
            let mut responses = Vec::new();
            
            for message in &session.conversation.messages {
                if message.role == crate::agent::conversation::MessageRole::Assistant {
                    let tool_calls = if let Some(ref tcs) = message.tool_calls {
                        tcs.iter().map(|tc| ToolCallInfo {
                            name: tc.tool_name.clone(),
                            status: ToolStatus::Success,
                            result: tc.result.clone(),
                            error: None,
                        }).collect()
                    } else {
                        Vec::new()
                    };
                    
                    responses.push(AgentResponse {
                        content: message.content.clone(),
                        tool_calls,
                        session_id: session_id.to_string(),
                    });
                }
            }
            
            Ok(responses)
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn send_prompt_streaming(
        &self,
        session_id: &str,
        message: String,
        provider: Option<String>,
        model: Option<String>,
    ) -> Result<AgentStream> {
        // Create streaming channel
        let (tx, rx) = create_agent_stream();
        
        // Process with streaming through UnifiedAgent
        self.agent.process_prompt_streaming(
            session_id,
            message,
            provider.as_deref(),
            model.as_deref(),
            tx,
        ).await?;
        
        Ok(rx)
    }
    
    async fn end_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.agent.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.active = false;
        }
        Ok(())
    }
}