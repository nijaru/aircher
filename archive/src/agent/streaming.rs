use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentUpdate {
    /// Tool execution status update
    ToolStatus(String),
    /// Streaming text chunk from LLM
    TextChunk {
        content: String,
        delta: bool,
        tokens_used: Option<u32>,
    },
    /// Agent processing complete
    Complete {
        total_tokens: u32,
        tool_status_messages: Vec<String>,
    },
    /// Error occurred
    Error(String),
}

pub type AgentStream = mpsc::Receiver<Result<AgentUpdate>>;
pub type AgentStreamSender = mpsc::Sender<Result<AgentUpdate>>;

/// Create an agent stream channel
pub fn create_agent_stream() -> (AgentStreamSender, AgentStream) {
    mpsc::channel(100)
}
