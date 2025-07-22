use super::{AgentTool, ToolError, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct RunCommandTool {
    workspace_root: Option<PathBuf>,
    allowed_commands: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RunCommandParams {
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    working_dir: Option<String>,
    #[serde(default = "default_timeout")]
    timeout_seconds: u64,
}

fn default_timeout() -> u64 {
    30
}

impl RunCommandTool {
    pub fn new() -> Self {
        Self {
            workspace_root: std::env::current_dir().ok(),
            // Default safe commands
            allowed_commands: vec![
                "ls".to_string(),
                "cat".to_string(),
                "grep".to_string(),
                "find".to_string(),
                "git".to_string(),
                "cargo".to_string(),
                "npm".to_string(),
                "python".to_string(),
                "make".to_string(),
                "echo".to_string(),
                "pwd".to_string(),
                "which".to_string(),
                "head".to_string(),
                "tail".to_string(),
                "wc".to_string(),
                "tree".to_string(),
            ],
        }
    }
    
    pub fn with_allowed_commands(mut self, commands: Vec<String>) -> Self {
        self.allowed_commands = commands;
        self
    }
    
    fn is_command_allowed(&self, command: &str) -> bool {
        self.allowed_commands.iter().any(|allowed| command == allowed)
    }
}

#[async_trait]
impl AgentTool for RunCommandTool {
    fn name(&self) -> &str {
        "run_command"
    }
    
    fn description(&self) -> &str {
        "Execute shell commands safely with output capture"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Command to execute"
                },
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Command arguments"
                },
                "working_dir": {
                    "type": "string",
                    "description": "Working directory for the command"
                },
                "timeout_seconds": {
                    "type": "integer",
                    "description": "Command timeout in seconds",
                    "default": 30,
                    "minimum": 1,
                    "maximum": 300
                }
            },
            "required": ["command"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: RunCommandParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;
        
        if !self.is_command_allowed(&params.command) {
            return Err(ToolError::PermissionDenied(
                format!("Command '{}' is not in the allowed list", params.command)
            ));
        }
        
        let mut cmd = Command::new(&params.command);
        cmd.args(&params.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
        
        // Set working directory
        if let Some(working_dir) = params.working_dir {
            cmd.current_dir(working_dir);
        } else if let Some(root) = &self.workspace_root {
            cmd.current_dir(root);
        }
        
        // Execute with timeout
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(params.timeout_seconds),
            cmd.spawn()
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to spawn command: {}", e)))?
                .wait_with_output()
        ).await
            .map_err(|_| ToolError::ExecutionFailed(format!("Command timed out after {} seconds", params.timeout_seconds)))?
            .map_err(|e| ToolError::ExecutionFailed(format!("Command execution failed: {}", e)))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        Ok(ToolOutput {
            success: output.status.success(),
            result: json!({
                "command": params.command,
                "args": params.args,
                "exit_code": output.status.code(),
                "stdout": stdout,
                "stderr": stderr,
                "success": output.status.success()
            }),
            error: if !output.status.success() {
                Some(format!("Command failed with exit code: {:?}", output.status.code()))
            } else {
                None
            },
            usage: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GitStatusTool {
    workspace_root: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct GitStatusParams {
    #[serde(default)]
    include_diff: bool,
    #[serde(default)]
    include_untracked: bool,
}

impl GitStatusTool {
    pub fn new() -> Self {
        Self {
            workspace_root: std::env::current_dir().ok(),
        }
    }
}

#[async_trait]
impl AgentTool for GitStatusTool {
    fn name(&self) -> &str {
        "git_status"
    }
    
    fn description(&self) -> &str {
        "Get git repository status and optionally the diff"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "include_diff": {
                    "type": "boolean",
                    "description": "Include git diff in the output",
                    "default": false
                },
                "include_untracked": {
                    "type": "boolean",
                    "description": "Include untracked files in the output",
                    "default": true
                }
            }
        })
    }
    
    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: GitStatusParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;
        
        let working_dir = self.workspace_root.as_ref()
            .ok_or_else(|| ToolError::ExecutionFailed("No workspace root".to_string()))?;
        
        // Get git status
        let mut status_cmd = Command::new("git");
        status_cmd.arg("status")
            .arg("--porcelain")
            .arg("-b")
            .current_dir(working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        if params.include_untracked {
            status_cmd.arg("-u");
        } else {
            status_cmd.arg("-uno");
        }
        
        let status_output = status_cmd.output().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to run git status: {}", e)))?;
        
        if !status_output.status.success() {
            let stderr = String::from_utf8_lossy(&status_output.stderr);
            return Err(ToolError::ExecutionFailed(format!("git status failed: {}", stderr)));
        }
        
        let status = String::from_utf8_lossy(&status_output.stdout);
        
        // Parse status
        let mut branch = String::new();
        let mut modified_files = Vec::new();
        let mut added_files = Vec::new();
        let mut deleted_files = Vec::new();
        let mut untracked_files = Vec::new();
        
        for line in status.lines() {
            if line.starts_with("## ") {
                branch = line[3..].split("...").next().unwrap_or("").to_string();
            } else if line.len() > 2 {
                let status_code = &line[0..2];
                let file_path = line[3..].trim();
                
                match status_code {
                    " M" | "M " | "MM" => modified_files.push(file_path.to_string()),
                    "A " | "AM" => added_files.push(file_path.to_string()),
                    "D " | " D" => deleted_files.push(file_path.to_string()),
                    "??" => untracked_files.push(file_path.to_string()),
                    _ => {}
                }
            }
        }
        
        let mut result = json!({
            "branch": branch,
            "modified": modified_files,
            "added": added_files,
            "deleted": deleted_files,
            "untracked": untracked_files,
            "total_changes": modified_files.len() + added_files.len() + deleted_files.len()
        });
        
        // Get diff if requested
        if params.include_diff && (modified_files.len() + added_files.len() > 0) {
            let diff_output = Command::new("git")
                .args(&["diff", "--cached", "HEAD"])
                .current_dir(working_dir)
                .output()
                .await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to run git diff: {}", e)))?;
            
            if diff_output.status.success() {
                let diff = String::from_utf8_lossy(&diff_output.stdout);
                result["diff"] = json!(diff.to_string());
            }
        }
        
        Ok(ToolOutput {
            success: true,
            result,
            error: None,
            usage: None,
        })
    }
}