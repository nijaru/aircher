use super::{AgentTool, ToolError, ToolOutput};
use crate::agent::approval_modes::{PendingChange, ChangeType};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs;

/// Tool output that includes pending changes for approval
#[derive(Debug, Clone)]
pub enum ApprovalToolOutput {
    /// Immediate result (for read operations)
    Immediate(ToolOutput),
    /// Pending approval required
    PendingApproval(PendingChange),
    /// Multiple changes pending approval
    BatchPendingApproval(Vec<PendingChange>),
}

/// Channel for sending pending changes to the UI
pub type ApprovalChannel = Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<PendingChange>>>>;

/// Base trait for tools that require approval
#[async_trait]
pub trait ApprovalTool: AgentTool {
    async fn execute_with_approval(&self, params: Value) -> Result<ApprovalToolOutput, ToolError>;
}

/// Write file tool with approval workflow
#[derive(Debug, Clone)]
pub struct ApprovedWriteFileTool {
    workspace_root: Option<PathBuf>,
    approval_channel: ApprovalChannel,
}

#[derive(Debug, Deserialize)]
struct WriteFileParams {
    path: String,
    content: String,
    #[serde(default)]
    create_dirs: bool,
}

impl ApprovedWriteFileTool {
    pub fn new(approval_channel: ApprovalChannel) -> Self {
        Self {
            workspace_root: std::env::current_dir().ok(),
            approval_channel,
        }
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        let path = Path::new(path);
        if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(root) = &self.workspace_root {
            root.join(path)
        } else {
            path.to_path_buf()
        }
    }
}

#[async_trait]
impl AgentTool for ApprovedWriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file (requires approval)"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                },
                "create_dirs": {
                    "type": "boolean",
                    "description": "Create parent directories if they don't exist",
                    "default": false
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: WriteFileParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        let path = self.resolve_path(&params.path);

        // Create pending change
        let description = if params.create_dirs {
            format!("Write {} bytes to {} (creating directories)", params.content.len(), path.display())
        } else {
            format!("Write {} bytes to {}", params.content.len(), path.display())
        };

        let change = PendingChange::new(
            ChangeType::CreateFile {
                path: path.clone(),
                content: params.content.clone(),
            },
            "write_file".to_string(),
            description,
        );

        // Send to approval channel if available
        if let Some(sender) = self.approval_channel.lock().await.as_ref() {
            let _ = sender.send(change.clone());
        }

        // Return preview of what would be written
        Ok(ToolOutput {
            success: true,
            result: json!({
                "status": "pending_approval",
                "path": path.display().to_string(),
                "bytes": params.content.len(),
                "preview": if params.content.len() > 100 {
                    format!("{}...", &params.content[..100])
                } else {
                    params.content
                },
                "change_id": change.id,
            }),
            error: None,
            usage: None,
        })
    }
}

/// Edit file tool with approval workflow
#[derive(Debug, Clone)]
pub struct ApprovedEditFileTool {
    workspace_root: Option<PathBuf>,
    approval_channel: ApprovalChannel,
}

#[derive(Debug, Deserialize)]
struct EditFileParams {
    path: String,
    search: String,
    replace: String,
    #[serde(default)]
    all_occurrences: bool,
}

impl ApprovedEditFileTool {
    pub fn new(approval_channel: ApprovalChannel) -> Self {
        Self {
            workspace_root: std::env::current_dir().ok(),
            approval_channel,
        }
    }

    fn resolve_path(&self, path: &str) -> Result<PathBuf, ToolError> {
        let path = Path::new(path);
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(root) = &self.workspace_root {
            root.join(path)
        } else {
            path.to_path_buf()
        };

        if !resolved.exists() {
            return Err(ToolError::NotFound(format!("File not found: {}", path.display())));
        }

        Ok(resolved)
    }
}

#[async_trait]
impl AgentTool for ApprovedEditFileTool {
    fn name(&self) -> &str {
        "edit_file"
    }

    fn description(&self) -> &str {
        "Edit a file by searching and replacing text (requires approval)"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "search": {
                    "type": "string",
                    "description": "Text to search for"
                },
                "replace": {
                    "type": "string",
                    "description": "Text to replace with"
                },
                "all_occurrences": {
                    "type": "boolean",
                    "description": "Replace all occurrences (default: false)",
                    "default": false
                }
            },
            "required": ["path", "search", "replace"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: EditFileParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        let path = self.resolve_path(&params.path)?;

        // Read current content
        let old_content = fs::read_to_string(&path).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read file: {}", e)))?;

        // Apply replacements
        let (new_content, replacements) = if params.all_occurrences {
            let count = old_content.matches(&params.search).count();
            (old_content.replace(&params.search, &params.replace), count)
        } else {
            let new = old_content.replacen(&params.search, &params.replace, 1);
            let count = if new != old_content { 1 } else { 0 };
            (new, count)
        };

        if replacements == 0 {
            return Err(ToolError::NotFound(format!("Search text not found: '{}'", params.search)));
        }

        // Create pending change
        let change = PendingChange::new(
            ChangeType::ModifyFile {
                path: path.clone(),
                old_content: old_content.clone(),
                new_content: new_content.clone(),
            },
            "edit_file".to_string(),
            format!("Replace {} occurrence(s) in {}", replacements, path.display()),
        );

        // Send to approval channel
        if let Some(sender) = self.approval_channel.lock().await.as_ref() {
            let _ = sender.send(change.clone());
        }

        // Return preview
        Ok(ToolOutput {
            success: true,
            result: json!({
                "status": "pending_approval",
                "path": path.display().to_string(),
                "replacements": replacements,
                "change_id": change.id,
                "diff_preview": generate_simple_diff(&params.search, &params.replace, replacements),
            }),
            error: None,
            usage: None,
        })
    }
}

/// Delete file tool with approval workflow
#[derive(Debug, Clone)]
pub struct ApprovedDeleteFileTool {
    workspace_root: Option<PathBuf>,
    approval_channel: ApprovalChannel,
}

#[derive(Debug, Deserialize)]
struct DeleteFileParams {
    path: String,
}

impl ApprovedDeleteFileTool {
    pub fn new(approval_channel: ApprovalChannel) -> Self {
        Self {
            workspace_root: std::env::current_dir().ok(),
            approval_channel,
        }
    }

    fn resolve_path(&self, path: &str) -> Result<PathBuf, ToolError> {
        let path = Path::new(path);
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(root) = &self.workspace_root {
            root.join(path)
        } else {
            path.to_path_buf()
        };

        if !resolved.exists() {
            return Err(ToolError::NotFound(format!("File not found: {}", path.display())));
        }

        Ok(resolved)
    }
}

#[async_trait]
impl AgentTool for ApprovedDeleteFileTool {
    fn name(&self) -> &str {
        "delete_file"
    }

    fn description(&self) -> &str {
        "Delete a file (requires approval)"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to delete"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: DeleteFileParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        let path = self.resolve_path(&params.path)?;

        // Get file size for info
        let metadata = fs::metadata(&path).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read file metadata: {}", e)))?;

        // Create pending change
        let change = PendingChange::new(
            ChangeType::DeleteFile {
                path: path.clone(),
            },
            "delete_file".to_string(),
            format!("Delete {} ({} bytes)", path.display(), metadata.len()),
        );

        // Send to approval channel
        if let Some(sender) = self.approval_channel.lock().await.as_ref() {
            let _ = sender.send(change.clone());
        }

        // Return preview
        Ok(ToolOutput {
            success: true,
            result: json!({
                "status": "pending_approval",
                "path": path.display().to_string(),
                "size": metadata.len(),
                "change_id": change.id,
                "warning": "This operation cannot be undone",
            }),
            error: None,
            usage: None,
        })
    }
}

/// Generate a simple diff preview
fn generate_simple_diff(search: &str, replace: &str, count: usize) -> String {
    format!(
        "Will replace {} occurrence(s):\n- {}\n+ {}",
        count, search, replace
    )
}

/// Factory function to create approved file tools
pub fn create_approved_file_tools(approval_channel: ApprovalChannel) -> Vec<Box<dyn AgentTool>> {
    vec![
        Box::new(ApprovedWriteFileTool::new(approval_channel.clone())),
        Box::new(ApprovedEditFileTool::new(approval_channel.clone())),
        Box::new(ApprovedDeleteFileTool::new(approval_channel)),
    ]
}
