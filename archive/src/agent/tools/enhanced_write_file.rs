// Enhanced WriteFile Tool - Production Implementation
// Week 1 Sprint: Real tool with automatic backups, atomic writes, protected file detection

use super::{AgentTool, ToolError, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::fs;
use chrono::Utc;

const BACKUP_DIR_NAME: &str = ".aircher_backups";

// Protected files that should not be overwritten without explicit confirmation
const PROTECTED_FILES: &[&str] = &[
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "CMakeLists.txt",
    "Makefile",
    "lib.rs",
    "main.rs",
    "mod.rs",
    "__init__.py",
    "setup.py",
    "main.go",
    "index.js",
    "index.ts",
    "App.tsx",
    "App.jsx",
];

// System directories that should never be written to
const PROTECTED_DIRS: &[&str] = &[
    "/bin",
    "/sbin",
    "/usr/bin",
    "/usr/sbin",
    "/etc",
    "/sys",
    "/proc",
    "/dev",
    "/boot",
];

#[derive(Debug, Clone)]
pub struct EnhancedWriteFileTool {
    workspace_root: Option<PathBuf>,
    backup_enabled: bool,
}

#[derive(Debug, Deserialize)]
struct WriteFileParams {
    path: String,
    content: String,
    #[serde(default = "default_true")]
    create_dirs: bool,
    #[serde(default = "default_true")]
    backup: bool,
    #[serde(default)]
    force_overwrite_protected: bool,
}

#[derive(Debug, Serialize)]
struct WriteResult {
    path: String,
    bytes_written: usize,
    created: bool,
    backed_up: bool,
    backup_path: Option<String>,
    protected_override: bool,
}

fn default_true() -> bool {
    true
}

impl EnhancedWriteFileTool {
    pub fn new() -> Self {
        Self {
            workspace_root: std::env::current_dir().ok(),
            backup_enabled: true,
        }
    }

    fn resolve_path(&self, path: &str) -> Result<PathBuf, ToolError> {
        // Handle paths that look like they should be absolute but are missing leading slash
        let corrected_path = if path.starts_with("tmp/") || path.starts_with("var/") {
            format!("/{}", path)
        } else if path.starts_with("Users/") && cfg!(target_os = "macos") {
            format!("/{}", path)
        } else if path.starts_with("home/") && cfg!(unix) {
            format!("/{}", path)
        } else {
            path.to_string()
        };

        let path = Path::new(&corrected_path);
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(root) = &self.workspace_root {
            root.join(path)
        } else {
            path.to_path_buf()
        };

        Ok(resolved)
    }

    fn is_protected_file(&self, path: &Path) -> bool {
        // Check if it's a protected system directory
        let path_str = path.to_string_lossy();
        for protected_dir in PROTECTED_DIRS {
            if path_str.starts_with(protected_dir) {
                return true;
            }
        }

        // Check if filename matches protected list
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();
            for protected in PROTECTED_FILES {
                if filename_str == *protected {
                    return true;
                }
            }
        }

        false
    }

    async fn create_backup(&self, path: &Path) -> Result<PathBuf, ToolError> {
        if !path.exists() {
            return Err(ToolError::ExecutionFailed("Cannot backup non-existent file".to_string()));
        }

        // Create backup directory
        let backup_dir = if let Some(parent) = path.parent() {
            parent.join(BACKUP_DIR_NAME)
        } else {
            PathBuf::from(BACKUP_DIR_NAME)
        };

        fs::create_dir_all(&backup_dir).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create backup directory: {}", e)))?;

        // Generate backup filename with timestamp
        let filename = path.file_name()
            .ok_or_else(|| ToolError::ExecutionFailed("Invalid filename".to_string()))?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("{}.backup_{}", filename.to_string_lossy(), timestamp);
        let backup_path = backup_dir.join(backup_filename);

        // Copy file to backup location
        fs::copy(path, &backup_path).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create backup: {}", e)))?;

        Ok(backup_path)
    }

    async fn atomic_write(&self, path: &Path, content: &str) -> Result<(), ToolError> {
        // Write to temporary file first
        let temp_path = path.with_extension("tmp");

        fs::write(&temp_path, content).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write temporary file: {}", e)))?;

        // Verify the write succeeded
        let written_content = fs::read_to_string(&temp_path).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to verify write: {}", e)))?;

        if written_content != content {
            // Rollback - delete temp file
            let _ = fs::remove_file(&temp_path).await;
            return Err(ToolError::ExecutionFailed("Write verification failed - content mismatch".to_string()));
        }

        // Atomic rename (overwrites destination if it exists)
        fs::rename(&temp_path, path).await
            .map_err(|e| {
                // Try to clean up temp file on failure
                let _ = std::fs::remove_file(&temp_path);
                ToolError::ExecutionFailed(format!("Failed to finalize write: {}", e))
            })?;

        Ok(())
    }

    async fn verify_write(&self, path: &Path, expected_content: &str) -> Result<bool, ToolError> {
        let actual_content = fs::read_to_string(path).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to verify write: {}", e)))?;

        Ok(actual_content == expected_content)
    }
}

#[async_trait]
impl AgentTool for EnhancedWriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file with automatic backups, atomic writes, and protected file detection. \
        Creates parent directories if needed. Automatically backs up existing files before overwriting. \
        Protects critical files (Cargo.toml, package.json, etc.) from accidental overwrite."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path where to write the file (absolute or relative to workspace)"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                },
                "create_dirs": {
                    "type": "boolean",
                    "description": "Create parent directories if they don't exist (default: true)",
                    "default": true
                },
                "backup": {
                    "type": "boolean",
                    "description": "Create backup of existing file before overwriting (default: true)",
                    "default": true
                },
                "force_overwrite_protected": {
                    "type": "boolean",
                    "description": "Force overwrite of protected files (Cargo.toml, package.json, etc.). Use with caution! (default: false)",
                    "default": false
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: WriteFileParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        let path = self.resolve_path(&params.path)?;

        // Check if file exists (for backup and created flag)
        let file_exists = path.exists();
        let is_protected = self.is_protected_file(&path);

        // Protected file check
        if file_exists && is_protected && !params.force_overwrite_protected {
            return Err(ToolError::PermissionDenied(format!(
                "Refusing to overwrite protected file: {}. \
                Protected files include: Cargo.toml, package.json, lib.rs, main.rs, etc. \
                Set force_overwrite_protected=true to override this safety check.",
                path.display()
            )));
        }

        // Create parent directories if requested
        if params.create_dirs {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create directories: {}", e)))?;
            }
        }

        // Create backup if file exists and backup is enabled
        let backup_path = if file_exists && params.backup && self.backup_enabled {
            match self.create_backup(&path).await {
                Ok(backup) => Some(backup),
                Err(e) => {
                    // Log warning but don't fail the write
                    eprintln!("Warning: Failed to create backup: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Perform atomic write
        self.atomic_write(&path, &params.content).await?;

        // Verify the write succeeded
        let write_verified = self.verify_write(&path, &params.content).await?;
        if !write_verified {
            // Attempt rollback if backup exists
            if let Some(ref backup) = backup_path {
                let _ = fs::copy(backup, &path).await;
                return Err(ToolError::ExecutionFailed(
                    "Write verification failed - rolled back to backup".to_string()
                ));
            }
            return Err(ToolError::ExecutionFailed("Write verification failed".to_string()));
        }

        // Build result
        let result = WriteResult {
            path: path.display().to_string(),
            bytes_written: params.content.len(),
            created: !file_exists,
            backed_up: backup_path.is_some(),
            backup_path: backup_path.map(|p| p.display().to_string()),
            protected_override: is_protected && params.force_overwrite_protected,
        };

        Ok(ToolOutput {
            success: true,
            result: json!(result),
            error: None,
            usage: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_write_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let tool = EnhancedWriteFileTool::new();
        let params = json!({
            "path": file_path.to_str().unwrap(),
            "content": "Hello, World!",
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.result["created"].as_bool().unwrap(), true);
        assert_eq!(output.result["backed_up"].as_bool().unwrap(), false);

        // Verify file contents
        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[tokio::test]
    async fn test_write_with_backup() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create initial file
        fs::write(&file_path, "Original content").await.unwrap();

        let tool = EnhancedWriteFileTool::new();
        let params = json!({
            "path": file_path.to_str().unwrap(),
            "content": "New content",
            "backup": true,
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.result["created"].as_bool().unwrap(), false);
        assert_eq!(output.result["backed_up"].as_bool().unwrap(), true);
        assert!(output.result["backup_path"].as_str().is_some());

        // Verify new content
        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "New content");

        // Verify backup exists and has original content
        let backup_path = output.result["backup_path"].as_str().unwrap();
        let backup_content = fs::read_to_string(backup_path).await.unwrap();
        assert_eq!(backup_content, "Original content");
    }

    #[tokio::test]
    async fn test_protected_file_rejection() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("Cargo.toml");

        // Create the protected file
        fs::write(&file_path, "Original").await.unwrap();

        let tool = EnhancedWriteFileTool::new();
        let params = json!({
            "path": file_path.to_str().unwrap(),
            "content": "New content",
        });

        let result = tool.execute(params).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::PermissionDenied(_)));

        // Verify original content unchanged
        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "Original");
    }

    #[tokio::test]
    async fn test_protected_file_force_override() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("Cargo.toml");

        // Create the protected file
        fs::write(&file_path, "Original").await.unwrap();

        let tool = EnhancedWriteFileTool::new();
        let params = json!({
            "path": file_path.to_str().unwrap(),
            "content": "New content",
            "force_overwrite_protected": true,
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.result["protected_override"].as_bool().unwrap(), true);

        // Verify new content
        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "New content");
    }

    #[tokio::test]
    async fn test_create_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("deep").join("nested").join("test.txt");

        let tool = EnhancedWriteFileTool::new();
        let params = json!({
            "path": file_path.to_str().unwrap(),
            "content": "Nested file",
            "create_dirs": true,
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        // Verify file exists in nested directory
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "Nested file");
    }

    #[tokio::test]
    async fn test_atomic_write_verification() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let tool = EnhancedWriteFileTool::new();
        let content = "Test content with verification";
        let params = json!({
            "path": file_path.to_str().unwrap(),
            "content": content,
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        // Verify exact content match
        let written_content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(written_content, content);

        // Verify no temporary files left behind
        let temp_path = file_path.with_extension("tmp");
        assert!(!temp_path.exists());
    }
}
