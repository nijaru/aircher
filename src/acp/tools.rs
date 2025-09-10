/// ACP-compatible tool implementations
/// 
/// These tools work via client requests rather than direct file system access.
/// The ACP client (editor) handles the actual file operations, providing
/// better security and integration with the editor's file management.

use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use crate::agent::tools::{AgentTool, ToolOutput, ToolError};

/// ACP client connection for making requests to the editor
/// This will be provided by the ACP connection when tools are executed
#[async_trait]
pub trait AcpClient: Send + Sync {
    async fn read_file(&self, path: &str) -> Result<String>;
    async fn write_file(&self, path: &str, content: &str) -> Result<()>;
    async fn list_directory(&self, path: &str) -> Result<Vec<String>>;
    async fn run_command(&self, command: &str, args: Vec<String>) -> Result<String>;
}

/// Read file tool that works via ACP client requests
pub struct AcpReadFileTool {
    client: Arc<dyn AcpClient>,
}

impl AcpReadFileTool {
    pub fn new(client: Arc<dyn AcpClient>) -> Self {
        Self { client }
    }
}

#[derive(Deserialize)]
struct ReadFileParams {
    path: String,
    #[serde(default)]
    line_start: Option<usize>,
    #[serde(default)]
    line_end: Option<usize>,
}

#[async_trait]
impl AgentTool for AcpReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to read"
                },
                "line_start": {
                    "type": "integer",
                    "description": "Optional starting line number (1-indexed)"
                },
                "line_end": {
                    "type": "integer", 
                    "description": "Optional ending line number (1-indexed)"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: ReadFileParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        match self.client.read_file(&params.path).await {
            Ok(content) => {
                let final_content = if let (Some(start), Some(end)) = (params.line_start, params.line_end) {
                    // Extract specific line range
                    let lines: Vec<&str> = content.lines().collect();
                    let start_idx = (start.saturating_sub(1)).min(lines.len());
                    let end_idx = end.min(lines.len());
                    
                    lines[start_idx..end_idx].join("\n")
                } else {
                    content
                };

                Ok(ToolOutput {
                    success: true,
                    result: serde_json::json!({
                        "content": final_content,
                        "path": params.path,
                        "lines": final_content.lines().count()
                    }),
                    error: None,
                    usage: None,
                })
            }
            Err(e) => Ok(ToolOutput {
                success: false,
                result: serde_json::Value::Null,
                error: Some(format!("Failed to read file {}: {}", params.path, e)),
                usage: None,
            })
        }
    }
}

/// Write file tool that works via ACP client requests
pub struct AcpWriteFileTool {
    client: Arc<dyn AcpClient>,
}

impl AcpWriteFileTool {
    pub fn new(client: Arc<dyn AcpClient>) -> Self {
        Self { client }
    }
}

#[derive(Deserialize)]
struct WriteFileParams {
    path: String,
    content: String,
}

#[async_trait]
impl AgentTool for AcpWriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file"
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: WriteFileParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        match self.client.write_file(&params.path, &params.content).await {
            Ok(()) => Ok(ToolOutput {
                success: true,
                result: serde_json::json!({
                    "path": params.path,
                    "bytes_written": params.content.len()
                }),
                error: None,
                usage: None,
            }),
            Err(e) => Ok(ToolOutput {
                success: false,
                result: serde_json::Value::Null,
                error: Some(format!("Failed to write file {}: {}", params.path, e)),
                usage: None,
            })
        }
    }
}

/// Command execution tool that works via ACP client requests
pub struct AcpRunCommandTool {
    client: Arc<dyn AcpClient>,
}

impl AcpRunCommandTool {
    pub fn new(client: Arc<dyn AcpClient>) -> Self {
        Self { client }
    }
}

#[derive(Deserialize)]
struct RunCommandParams {
    command: String,
    #[serde(default)]
    args: Vec<String>,
}

#[async_trait]
impl AgentTool for AcpRunCommandTool {
    fn name(&self) -> &str {
        "run_command"
    }

    fn description(&self) -> &str {
        "Execute a shell command"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command to execute"
                },
                "args": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Command arguments"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: RunCommandParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        match self.client.run_command(&params.command, params.args).await {
            Ok(output) => Ok(ToolOutput {
                success: true,
                result: serde_json::json!({
                    "output": output,
                    "command": params.command
                }),
                error: None,
                usage: None,
            }),
            Err(e) => Ok(ToolOutput {
                success: false,
                result: serde_json::Value::Null,
                error: Some(format!("Command failed: {}", e)),
                usage: None,
            })
        }
    }
}

/// Create ACP-compatible tool registry
pub fn create_acp_tool_registry(client: Arc<dyn AcpClient>) -> crate::agent::tools::ToolRegistry {
    let mut registry = crate::agent::tools::ToolRegistry::new();

    // Register ACP-compatible tools
    registry.register(Box::new(AcpReadFileTool::new(client.clone())));
    registry.register(Box::new(AcpWriteFileTool::new(client.clone())));
    registry.register(Box::new(AcpRunCommandTool::new(client)));

    registry
}

// TODO: Add more ACP tools:
// - AcpListFilesTool
// - AcpEditFileTool  
// - AcpSearchCodeTool (via client search capabilities)