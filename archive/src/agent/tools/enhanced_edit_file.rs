// Enhanced EditFile Tool - Production Implementation
// Week 1 Sprint: Real tool with line-based editing, context-aware changes, diff preview

use super::{AgentTool, ToolError, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::fs;
use similar::{ChangeTag, TextDiff};

#[derive(Debug, Clone)]
pub struct EnhancedEditFileTool {
    workspace_root: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EditFileParams {
    SearchReplace {
        path: String,
        search: String,
        replace: String,
        #[serde(default)]
        all_occurrences: bool,
    },
    LineBased {
        path: String,
        #[serde(default)]
        edits: Vec<LineEdit>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct LineEdit {
    /// Line number (1-indexed)
    line: usize,
    /// Operation type: "replace", "insert_before", "insert_after", "delete"
    operation: String,
    /// New content (for replace, insert_before, insert_after)
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Serialize)]
struct EditResult {
    path: String,
    changes_made: usize,
    diff: String,
    backed_up: bool,
    backup_path: Option<String>,
}

impl EnhancedEditFileTool {
    pub fn new() -> Self {
        Self {
            workspace_root: std::env::current_dir().ok(),
        }
    }

    fn resolve_path(&self, path: &str) -> Result<PathBuf, ToolError> {
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

        if !resolved.exists() {
            return Err(ToolError::NotFound(format!("File not found: {}", resolved.display())));
        }

        Ok(resolved)
    }

    async fn create_backup(&self, path: &Path) -> Result<PathBuf, ToolError> {
        let backup_path = path.with_extension("backup");
        fs::copy(path, &backup_path).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create backup: {}", e)))?;
        Ok(backup_path)
    }

    fn generate_diff(&self, original: &str, modified: &str) -> String {
        let diff = TextDiff::from_lines(original, modified);
        let mut result = String::new();

        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            result.push_str(&format!("{} {}", sign, change));
        }

        result
    }

    async fn apply_search_replace(
        &self,
        content: &str,
        search: &str,
        replace: &str,
        all_occurrences: bool,
    ) -> Result<(String, usize), ToolError> {
        let (new_content, replacements) = if all_occurrences {
            let count = content.matches(search).count();
            (content.replace(search, replace), count)
        } else {
            let new = content.replacen(search, replace, 1);
            let count = if new != content { 1 } else { 0 };
            (new, count)
        };

        if replacements == 0 {
            return Err(ToolError::NotFound(format!("Search text not found: '{}'", search)));
        }

        Ok((new_content, replacements))
    }

    async fn apply_line_edits(
        &self,
        content: &str,
        edits: Vec<LineEdit>,
    ) -> Result<(String, usize), ToolError> {
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut changes_made = 0;

        // Sort edits by line number (descending) to avoid index shifting issues
        let mut sorted_edits = edits;
        sorted_edits.sort_by(|a, b| b.line.cmp(&a.line));

        for edit in sorted_edits {
            // Convert to 0-indexed
            let line_idx = edit.line.saturating_sub(1);

            match edit.operation.as_str() {
                "replace" => {
                    if line_idx >= lines.len() {
                        return Err(ToolError::InvalidParameters(format!(
                            "Line {} out of range (file has {} lines)",
                            edit.line,
                            lines.len()
                        )));
                    }
                    if let Some(content) = edit.content {
                        lines[line_idx] = content;
                        changes_made += 1;
                    } else {
                        return Err(ToolError::InvalidParameters(
                            "replace operation requires content".to_string()
                        ));
                    }
                }
                "insert_before" => {
                    if line_idx > lines.len() {
                        return Err(ToolError::InvalidParameters(format!(
                            "Line {} out of range (file has {} lines)",
                            edit.line,
                            lines.len()
                        )));
                    }
                    if let Some(content) = edit.content {
                        lines.insert(line_idx, content);
                        changes_made += 1;
                    } else {
                        return Err(ToolError::InvalidParameters(
                            "insert_before operation requires content".to_string()
                        ));
                    }
                }
                "insert_after" => {
                    if line_idx >= lines.len() {
                        return Err(ToolError::InvalidParameters(format!(
                            "Line {} out of range (file has {} lines)",
                            edit.line,
                            lines.len()
                        )));
                    }
                    if let Some(content) = edit.content {
                        lines.insert(line_idx + 1, content);
                        changes_made += 1;
                    } else {
                        return Err(ToolError::InvalidParameters(
                            "insert_after operation requires content".to_string()
                        ));
                    }
                }
                "delete" => {
                    if line_idx >= lines.len() {
                        return Err(ToolError::InvalidParameters(format!(
                            "Line {} out of range (file has {} lines)",
                            edit.line,
                            lines.len()
                        )));
                    }
                    lines.remove(line_idx);
                    changes_made += 1;
                }
                _ => {
                    return Err(ToolError::InvalidParameters(format!(
                        "Unknown operation: {}. Valid operations: replace, insert_before, insert_after, delete",
                        edit.operation
                    )));
                }
            }
        }

        let new_content = lines.join("\n");
        if !content.is_empty() && !new_content.ends_with('\n') {
            // Preserve trailing newline if original had one
            if content.ends_with('\n') {
                Ok((format!("{}\n", new_content), changes_made))
            } else {
                Ok((new_content, changes_made))
            }
        } else {
            Ok((new_content, changes_made))
        }
    }
}

#[async_trait]
impl AgentTool for EnhancedEditFileTool {
    fn name(&self) -> &str {
        "edit_file"
    }

    fn description(&self) -> &str {
        "Edit a file using either search/replace or line-based operations. \
        Supports multiple editing modes: \
        1. Search/Replace: Find and replace text (all occurrences or first match) \
        2. Line-based: Precise line operations (replace, insert_before, insert_after, delete) \
        Automatically creates backups and generates diffs showing changes."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "oneOf": [
                {
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
                            "description": "Replace all occurrences or just the first one (default: false)",
                            "default": false
                        }
                    },
                    "required": ["path", "search", "replace"]
                },
                {
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file to edit"
                        },
                        "edits": {
                            "type": "array",
                            "description": "Array of line-based edit operations",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "line": {
                                        "type": "integer",
                                        "description": "Line number (1-indexed)"
                                    },
                                    "operation": {
                                        "type": "string",
                                        "enum": ["replace", "insert_before", "insert_after", "delete"],
                                        "description": "Operation type"
                                    },
                                    "content": {
                                        "type": "string",
                                        "description": "New content (required for replace/insert operations)"
                                    }
                                },
                                "required": ["line", "operation"]
                            }
                        }
                    },
                    "required": ["path", "edits"]
                }
            ]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: EditFileParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        let (path_str, original_content, new_content, changes_made) = match params {
            EditFileParams::SearchReplace {
                path,
                search,
                replace,
                all_occurrences,
            } => {
                let resolved_path = self.resolve_path(&path)?;
                let content = fs::read_to_string(&resolved_path).await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read file: {}", e)))?;

                let (new_content, changes) = self.apply_search_replace(&content, &search, &replace, all_occurrences).await?;
                (path, content, new_content, changes)
            }
            EditFileParams::LineBased { path, edits } => {
                let resolved_path = self.resolve_path(&path)?;
                let content = fs::read_to_string(&resolved_path).await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read file: {}", e)))?;

                let (new_content, changes) = self.apply_line_edits(&content, edits).await?;
                (path, content, new_content, changes)
            }
        };

        let resolved_path = self.resolve_path(&path_str)?;

        // Create backup
        let backup_path = self.create_backup(&resolved_path).await.ok();

        // Generate diff
        let diff = self.generate_diff(&original_content, &new_content);

        // Write new content
        fs::write(&resolved_path, &new_content).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write file: {}", e)))?;

        // Build result
        let result = EditResult {
            path: resolved_path.display().to_string(),
            changes_made,
            diff,
            backed_up: backup_path.is_some(),
            backup_path: backup_path.map(|p| p.display().to_string()),
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
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_search_replace_single() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello World\nHello Rust\nGoodbye World").unwrap();

        let tool = EnhancedEditFileTool::new();
        let params = json!({
            "path": temp_file.path().to_str().unwrap(),
            "search": "Hello",
            "replace": "Hi",
            "all_occurrences": false,
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.result["changes_made"].as_u64().unwrap(), 1);

        let content = fs::read_to_string(temp_file.path()).await.unwrap();
        assert!(content.starts_with("Hi World"));
        assert!(content.contains("Hello Rust")); // Second occurrence unchanged
    }

    #[tokio::test]
    async fn test_search_replace_all() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello World\nHello Rust\nGoodbye World").unwrap();

        let tool = EnhancedEditFileTool::new();
        let params = json!({
            "path": temp_file.path().to_str().unwrap(),
            "search": "Hello",
            "replace": "Hi",
            "all_occurrences": true,
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.result["changes_made"].as_u64().unwrap(), 2);

        let content = fs::read_to_string(temp_file.path()).await.unwrap();
        assert!(content.contains("Hi World"));
        assert!(content.contains("Hi Rust"));
        assert!(!content.contains("Hello"));
    }

    #[tokio::test]
    async fn test_line_replace() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1\nLine 2\nLine 3").unwrap();

        let tool = EnhancedEditFileTool::new();
        let params = json!({
            "path": temp_file.path().to_str().unwrap(),
            "edits": [
                {
                    "line": 2,
                    "operation": "replace",
                    "content": "Modified Line 2"
                }
            ]
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(temp_file.path()).await.unwrap();
        assert!(content.contains("Line 1"));
        assert!(content.contains("Modified Line 2"));
        assert!(content.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_line_insert_before() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1\nLine 2").unwrap();

        let tool = EnhancedEditFileTool::new();
        let params = json!({
            "path": temp_file.path().to_str().unwrap(),
            "edits": [
                {
                    "line": 2,
                    "operation": "insert_before",
                    "content": "New Line"
                }
            ]
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(temp_file.path()).await.unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[1], "New Line");
    }

    #[tokio::test]
    async fn test_line_delete() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1\nLine 2\nLine 3").unwrap();

        let tool = EnhancedEditFileTool::new();
        let params = json!({
            "path": temp_file.path().to_str().unwrap(),
            "edits": [
                {
                    "line": 2,
                    "operation": "delete"
                }
            ]
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(temp_file.path()).await.unwrap();
        assert!(!content.contains("Line 2"));
        assert!(content.contains("Line 1"));
        assert!(content.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_multiple_edits() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1\nLine 2\nLine 3").unwrap();

        let tool = EnhancedEditFileTool::new();
        let params = json!({
            "path": temp_file.path().to_str().unwrap(),
            "edits": [
                {
                    "line": 1,
                    "operation": "replace",
                    "content": "Modified Line 1"
                },
                {
                    "line": 3,
                    "operation": "insert_after",
                    "content": "Line 4"
                }
            ]
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(temp_file.path()).await.unwrap();
        assert!(content.contains("Modified Line 1"));
        assert!(content.contains("Line 4"));
    }

    #[tokio::test]
    async fn test_diff_generation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1\nLine 2\nLine 3").unwrap();

        let tool = EnhancedEditFileTool::new();
        let params = json!({
            "path": temp_file.path().to_str().unwrap(),
            "search": "Line 2",
            "replace": "Modified Line 2",
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        let diff = output.result["diff"].as_str().unwrap();
        assert!(diff.contains("-"));
        assert!(diff.contains("+"));
    }
}
