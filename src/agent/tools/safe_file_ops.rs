use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use super::{AgentTool, ToolOutput, ToolError, TokenUsage};

/// Safe file writing tool that prevents overwriting critical files
pub struct SafeWriteFileTool {
    workspace_root: Option<PathBuf>,
    /// Protected files that should never be overwritten
    protected_patterns: Vec<String>,
}

impl SafeWriteFileTool {
    pub fn new(workspace_root: Option<PathBuf>) -> Self {
        Self {
            workspace_root,
            protected_patterns: vec![
                // Critical project files
                "lib.rs".to_string(),
                "main.rs".to_string(),
                "Cargo.toml".to_string(),
                "package.json".to_string(),
                "pyproject.toml".to_string(),
                ".gitignore".to_string(),
                "README.md".to_string(),

                // Configuration files
                ".env".to_string(),
                "config.toml".to_string(),
                "settings.json".to_string(),

                // Build outputs
                "Cargo.lock".to_string(),
                "package-lock.json".to_string(),
                "yarn.lock".to_string(),
            ],
        }
    }

    fn is_protected_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check if this is a critical file
        for pattern in &self.protected_patterns {
            if path_str.ends_with(pattern) || path_str.contains(&format!("/{}", pattern)) {
                return true;
            }
        }

        // Check for system directories
        if path_str.starts_with("/etc") ||
           path_str.starts_with("/usr") ||
           path_str.starts_with("/bin") ||
           path_str.starts_with("/sbin") ||
           path_str.starts_with("/System") ||
           path_str.contains("/.git/") {
            return true;
        }

        false
    }

    fn suggest_safe_path(&self, original_path: &Path) -> PathBuf {
        let file_name = original_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("generated");

        // Suggest a safe location for generated code
        if let Some(workspace) = &self.workspace_root {
            // Create a generated/ directory for new files
            let generated_dir = workspace.join("generated");
            generated_dir.join(format!("{}.generated", file_name))
        } else {
            // Use temp directory as fallback
            std::env::temp_dir().join(format!("{}.generated", file_name))
        }
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        let path = Path::new(path);

        // If no path specified or just a filename, use safe generated location
        if path.parent().is_none() || path.parent() == Some(Path::new("")) {
            return self.suggest_safe_path(path);
        }

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
impl AgentTool for SafeWriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file safely, preventing overwriting of critical files"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path where to write the file (will be made safe automatically)"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                },
                "force": {
                    "type": "boolean",
                    "description": "Force overwrite even if file exists (still respects protection)",
                    "default": false
                }
            },
            "required": ["content"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let path_str = params.get("path")
            .and_then(|p| p.as_str())
            .unwrap_or("generated_code.txt");

        let content = params.get("content")
            .and_then(|c| c.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'content' parameter".to_string()))?;

        let force = params.get("force")
            .and_then(|f| f.as_bool())
            .unwrap_or(false);

        let mut target_path = self.resolve_path(path_str);

        // Check if this would overwrite a protected file
        if self.is_protected_file(&target_path) {
            let safe_path = self.suggest_safe_path(&target_path);
            warn!("Attempted to write to protected file: {}. Redirecting to safe location: {}",
                  target_path.display(), safe_path.display());

            target_path = safe_path;
        }

        // Check if file exists and we're not forcing
        if target_path.exists() && !force {
            // Generate a unique name
            let stem = target_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            let ext = target_path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("txt");

            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let new_name = format!("{}_{}.{}", stem, timestamp, ext);

            if let Some(parent) = target_path.parent() {
                target_path = parent.join(new_name);
            }

            info!("File exists, created new file: {}", target_path.display());
        }

        // Create parent directories if needed
        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create parent directories: {}", e)))?;
        }

        // Write the file
        tokio::fs::write(&target_path, content).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write file {}: {}", target_path.display(), e)))?;

        info!("Safely wrote {} bytes to {}", content.len(), target_path.display());

        Ok(ToolOutput {
            success: true,
            result: json!({
                "message": format!("File written successfully to {}", target_path.display()),
                "path": target_path.to_string_lossy(),
                "bytes_written": content.len(),
                "was_redirected": target_path != self.resolve_path(path_str)
            }),
            error: None,
            usage: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_prevents_lib_rs_overwrite() {
        let dir = tempdir().unwrap();
        let tool = SafeWriteFileTool::new(Some(dir.path().to_path_buf()));

        let params = json!({
            "path": "lib.rs",
            "content": "fn main() {}"
        });

        let result = tool.execute(params).await.unwrap();
        assert!(result.success);

        // Should have redirected to safe location
        let result_obj = result.result.as_object().unwrap();
        assert_eq!(result_obj.get("was_redirected").unwrap(), true);

        let written_path = result_obj.get("path").unwrap().as_str().unwrap();
        assert!(written_path.contains("generated"));
    }

    #[tokio::test]
    async fn test_allows_new_safe_file() {
        let dir = tempdir().unwrap();
        let tool = SafeWriteFileTool::new(Some(dir.path().to_path_buf()));

        let params = json!({
            "path": "src/my_new_module.rs",
            "content": "pub fn hello() {}"
        });

        let result = tool.execute(params).await.unwrap();
        assert!(result.success);

        // Should write to the requested location
        let written_path = dir.path().join("src/my_new_module.rs");
        assert!(written_path.exists());
    }
}