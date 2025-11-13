// Enhanced ReadFile Tool - Production Implementation
// Week 1 Sprint: Real tool with syntax highlighting, context extraction, smart truncation

use super::{AgentTool, ToolError, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::fs;
use crate::search_display::SyntaxHighlighter;
use crate::intelligence::ast_analysis::ASTAnalyzer;

const DEFAULT_MAX_LINES: usize = 1000;
const DEFAULT_CONTEXT_LINES: usize = 5;

#[derive(Debug)]
pub struct EnhancedReadFileTool {
    workspace_root: Option<PathBuf>,
    max_lines: usize,
}

#[derive(Debug, Deserialize)]
struct ReadFileParams {
    path: String,
    #[serde(default)]
    start_line: Option<usize>,
    #[serde(default)]
    end_line: Option<usize>,
    /// Enable syntax highlighting (default: true)
    #[serde(default = "default_true")]
    syntax_highlight: bool,
    /// Extract surrounding context for functions/classes (default: false)
    #[serde(default)]
    extract_context: bool,
    /// Maximum lines to return (default: 1000)
    #[serde(default = "default_max_lines")]
    max_lines: usize,
}

#[derive(Debug, Serialize)]
struct FileMetadata {
    path: String,
    size_bytes: u64,
    modified: String,
    permissions: String,
    language: String,
    total_lines: usize,
    displayed_lines: (usize, usize),
}

#[derive(Debug, Serialize)]
struct ContextInfo {
    context_type: String, // "function", "class", "method"
    name: String,
    start_line: usize,
    end_line: usize,
}

fn default_true() -> bool {
    true
}

fn default_max_lines() -> usize {
    DEFAULT_MAX_LINES
}

impl EnhancedReadFileTool {
    pub fn new() -> Self {
        Self {
            workspace_root: std::env::current_dir().ok(),
            max_lines: DEFAULT_MAX_LINES,
        }
    }

    fn resolve_path(&self, path: &str) -> Result<PathBuf, ToolError> {
        // Handle paths that look like they should be absolute but are missing leading slash
        let corrected_path = if path.starts_with("tmp/") || path.starts_with("var/") || path.starts_with("etc/") {
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

        // Check if it's a regular file (not directory or special file)
        if !resolved.is_file() {
            return Err(ToolError::InvalidParameters(format!("Not a regular file: {}", resolved.display())));
        }

        Ok(resolved)
    }

    async fn get_metadata(&self, path: &Path) -> Result<FileMetadata, ToolError> {
        let metadata = fs::metadata(path).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read metadata: {}", e)))?;

        let modified = metadata.modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| {
                let secs = d.as_secs();
                chrono::DateTime::from_timestamp(secs as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        #[cfg(unix)]
        let permissions = {
            use std::os::unix::fs::PermissionsExt;
            format!("{:o}", metadata.permissions().mode() & 0o777)
        };

        #[cfg(not(unix))]
        let permissions = if metadata.permissions().readonly() {
            "read-only".to_string()
        } else {
            "read-write".to_string()
        };

        let language = self.detect_language(path);
        let content = fs::read_to_string(path).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read file: {}", e)))?;

        Ok(FileMetadata {
            path: path.display().to_string(),
            size_bytes: metadata.len(),
            modified,
            permissions,
            language: language.clone(),
            total_lines: content.lines().count(),
            displayed_lines: (0, 0), // Will be updated later
        })
    }

    fn detect_language(&self, path: &Path) -> String {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| match ext {
                "rs" => "rust",
                "py" => "python",
                "js" => "javascript",
                "ts" => "typescript",
                "go" => "go",
                "c" | "h" => "c",
                "cpp" | "cc" | "cxx" | "hpp" => "cpp",
                "java" => "java",
                "json" => "json",
                "sh" | "bash" => "bash",
                "rb" => "ruby",
                "php" => "php",
                "cs" => "csharp",
                "kt" => "kotlin",
                "swift" => "swift",
                "md" => "markdown",
                "yaml" | "yml" => "yaml",
                "toml" => "toml",
                "sql" => "sql",
                _ => "text",
            })
            .unwrap_or("text")
            .to_string()
    }

    async fn extract_context_info(&self, path: &Path, line_number: usize) -> Result<Vec<ContextInfo>> {
        let mut analyzer = ASTAnalyzer::new()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create AST analyzer: {}", e)))?;

        let analysis = analyzer.analyze_file(path).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to analyze file: {}", e)))?;

        let mut contexts = Vec::new();

        if let Some(analysis) = analysis {
            // Check if line is within a function
            for func in &analysis.functions {
                if line_number >= func.start_line && line_number <= func.end_line {
                    contexts.push(ContextInfo {
                        context_type: if func.is_async { "async function".to_string() } else { "function".to_string() },
                        name: func.name.clone(),
                        start_line: func.start_line,
                        end_line: func.end_line,
                    });
                }
            }

            // Check if line is within a class
            for class in &analysis.classes {
                if line_number >= class.start_line && line_number <= class.end_line {
                    contexts.push(ContextInfo {
                        context_type: "class".to_string(),
                        name: class.name.clone(),
                        start_line: class.start_line,
                        end_line: class.end_line,
                    });

                    // Check if within a method of this class
                    for method in &class.methods {
                        if line_number >= method.start_line && line_number <= method.end_line {
                            contexts.push(ContextInfo {
                                context_type: "method".to_string(),
                                name: format!("{}.{}", class.name, method.name),
                                start_line: method.start_line,
                                end_line: method.end_line,
                            });
                        }
                    }
                }
            }
        }

        Ok(contexts)
    }

    fn smart_truncate(&self, lines: &[&str], max_lines: usize) -> (Vec<String>, bool) {
        if lines.len() <= max_lines {
            return (lines.iter().map(|s| s.to_string()).collect(), false);
        }

        // Take first and last portions, indicate truncation
        let half = max_lines / 2;
        let mut result = Vec::new();
        result.extend(lines[..half].iter().map(|s| s.to_string()));
        result.extend(lines[lines.len() - half..].iter().map(|s| s.to_string()));
        (result, true)
    }
}

#[async_trait]
impl AgentTool for EnhancedReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read file contents with syntax highlighting, context extraction, and smart truncation. \
        Automatically detects language, extracts surrounding function/class context, and handles \
        large files intelligently."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read (absolute or relative to workspace)"
                },
                "start_line": {
                    "type": "integer",
                    "description": "Starting line number (1-indexed, optional)"
                },
                "end_line": {
                    "type": "integer",
                    "description": "Ending line number (inclusive, optional)"
                },
                "syntax_highlight": {
                    "type": "boolean",
                    "description": "Enable syntax highlighting using tree-sitter (default: true)",
                    "default": true
                },
                "extract_context": {
                    "type": "boolean",
                    "description": "Extract surrounding function/class context (default: false)",
                    "default": false
                },
                "max_lines": {
                    "type": "integer",
                    "description": "Maximum lines to return, truncates large files (default: 1000)",
                    "default": 1000
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: ReadFileParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        let path = self.resolve_path(&params.path)?;

        // Get metadata
        let mut metadata = self.get_metadata(&path).await?;

        // Read file content
        let content = fs::read_to_string(&path).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read file: {}", e)))?;

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        // Determine line range
        let (start, end) = match (params.start_line, params.end_line) {
            (Some(s), Some(e)) => (s.saturating_sub(1), e.min(total_lines)),
            (Some(s), None) => (s.saturating_sub(1), total_lines),
            (None, Some(e)) => (0, e.min(total_lines)),
            (None, None) => (0, total_lines),
        };

        // Extract requested lines
        let selected_lines: Vec<&str> = lines[start..end].to_vec();

        // Smart truncation if needed
        let (final_lines, truncated) = if selected_lines.len() > params.max_lines {
            self.smart_truncate(&selected_lines, params.max_lines)
        } else {
            (selected_lines.iter().map(|s| s.to_string()).collect(), false)
        };

        // Format with line numbers
        let formatted_lines: Vec<String> = final_lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if truncated && i == params.max_lines / 2 {
                    format!("{:4} │ ... ({} lines truncated) ...", "", total_lines - params.max_lines)
                } else {
                    format!("{:4} │ {}", start + i + 1, line)
                }
            })
            .collect();

        let content_str = formatted_lines.join("\n");

        // Apply syntax highlighting if requested
        let final_content = if params.syntax_highlight {
            let mut highlighter = SyntaxHighlighter::new();
            highlighter.highlight_code(&content_str, &metadata.language)
        } else {
            content_str
        };

        // Extract context if requested
        let context = if params.extract_context && params.start_line.is_some() {
            let line_num = params.start_line.unwrap();
            self.extract_context_info(&path, line_num).await.ok()
        } else {
            None
        };

        // Update metadata with displayed range
        metadata.displayed_lines = (start + 1, end);

        // Build result
        let mut result = json!({
            "metadata": metadata,
            "content": final_content,
            "truncated": truncated,
        });

        if let Some(context) = context {
            result["context"] = json!(context);
        }

        Ok(ToolOutput {
            success: true,
            result,
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
    async fn test_read_simple_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1\nLine 2\nLine 3").unwrap();

        let tool = EnhancedReadFileTool::new();
        let params = json!({
            "path": temp_file.path().to_str().unwrap(),
            "syntax_highlight": false,
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.result["content"].as_str().unwrap().contains("Line 1"));
        assert!(output.result["content"].as_str().unwrap().contains("Line 2"));
    }

    #[tokio::test]
    async fn test_read_with_line_range() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1\nLine 2\nLine 3\nLine 4\nLine 5").unwrap();

        let tool = EnhancedReadFileTool::new();
        let params = json!({
            "path": temp_file.path().to_str().unwrap(),
            "start_line": 2,
            "end_line": 4,
            "syntax_highlight": false,
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        let content = output.result["content"].as_str().unwrap();
        assert!(content.contains("Line 2"));
        assert!(content.contains("Line 3"));
        assert!(content.contains("Line 4"));
        assert!(!content.contains("Line 1"));
        assert!(!content.contains("Line 5"));
    }

    #[tokio::test]
    async fn test_file_not_found() {
        let tool = EnhancedReadFileTool::new();
        let params = json!({
            "path": "/nonexistent/file.txt",
        });

        let result = tool.execute(params).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_metadata_extraction() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "fn main() {{}}").unwrap();

        let tool = EnhancedReadFileTool::new();
        let params = json!({
            "path": temp_file.path().to_str().unwrap(),
            "syntax_highlight": false,
        });

        let result = tool.execute(params).await.unwrap();
        let metadata = &result.result["metadata"];

        assert!(metadata["path"].as_str().is_some());
        assert!(metadata["size_bytes"].as_u64().unwrap() > 0);
        assert!(metadata["total_lines"].as_u64().unwrap() > 0);
    }
}
