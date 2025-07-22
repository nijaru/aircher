use super::{AgentTool, ToolError, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct ReadFileTool {
    workspace_root: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct ReadFileParams {
    path: String,
    #[serde(default)]
    start_line: Option<usize>,
    #[serde(default)]
    end_line: Option<usize>,
}

impl ReadFileTool {
    pub fn new() -> Self {
        Self {
            workspace_root: std::env::current_dir().ok(),
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
impl AgentTool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }
    
    fn description(&self) -> &str {
        "Read the contents of a file with optional line range"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                },
                "start_line": {
                    "type": "integer",
                    "description": "Starting line number (1-indexed)"
                },
                "end_line": {
                    "type": "integer",
                    "description": "Ending line number (inclusive)"
                }
            },
            "required": ["path"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: ReadFileParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;
        
        let path = self.resolve_path(&params.path)?;
        
        let content = fs::read_to_string(&path).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read file: {}", e)))?;
        
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        
        let (start, end) = match (params.start_line, params.end_line) {
            (Some(s), Some(e)) => (s.saturating_sub(1), e.min(total_lines)),
            (Some(s), None) => (s.saturating_sub(1), total_lines),
            (None, Some(e)) => (0, e.min(total_lines)),
            (None, None) => (0, total_lines),
        };
        
        let selected_lines: Vec<String> = lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:4} │ {}", start + i + 1, line))
            .collect();
        
        Ok(ToolOutput {
            success: true,
            result: json!({
                "path": path.display().to_string(),
                "content": selected_lines.join("\n"),
                "total_lines": total_lines,
                "displayed_lines": [start + 1, end]
            }),
            error: None,
            usage: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct WriteFileTool {
    workspace_root: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct WriteFileParams {
    path: String,
    content: String,
    #[serde(default)]
    create_dirs: bool,
}

impl WriteFileTool {
    pub fn new() -> Self {
        Self {
            workspace_root: std::env::current_dir().ok(),
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
impl AgentTool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }
    
    fn description(&self) -> &str {
        "Write content to a file, creating it if it doesn't exist"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path where to write the file"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                },
                "create_dirs": {
                    "type": "boolean",
                    "description": "Create parent directories if they don't exist",
                    "default": true
                }
            },
            "required": ["path", "content"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: WriteFileParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;
        
        let path = self.resolve_path(&params.path);
        
        if params.create_dirs {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create directories: {}", e)))?;
            }
        }
        
        fs::write(&path, &params.content).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write file: {}", e)))?;
        
        Ok(ToolOutput {
            success: true,
            result: json!({
                "path": path.display().to_string(),
                "bytes_written": params.content.len(),
                "created": !path.exists()
            }),
            error: None,
            usage: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct EditFileTool {
    workspace_root: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct EditFileParams {
    path: String,
    search: String,
    replace: String,
    #[serde(default)]
    all_occurrences: bool,
}

impl EditFileTool {
    pub fn new() -> Self {
        Self {
            workspace_root: std::env::current_dir().ok(),
        }
    }
}

#[async_trait]
impl AgentTool for EditFileTool {
    fn name(&self) -> &str {
        "edit_file"
    }
    
    fn description(&self) -> &str {
        "Edit a file by searching and replacing text"
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
                    "description": "Replace all occurrences or just the first one",
                    "default": false
                }
            },
            "required": ["path", "search", "replace"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: EditFileParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;
        
        let read_tool = ReadFileTool::new();
        let path = read_tool.resolve_path(&params.path)?;
        
        let content = fs::read_to_string(&path).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read file: {}", e)))?;
        
        let (new_content, replacements) = if params.all_occurrences {
            let count = content.matches(&params.search).count();
            (content.replace(&params.search, &params.replace), count)
        } else {
            let new = content.replacen(&params.search, &params.replace, 1);
            let count = if new != content { 1 } else { 0 };
            (new, count)
        };
        
        if replacements == 0 {
            return Err(ToolError::NotFound(format!("Search text not found: '{}'", params.search)));
        }
        
        fs::write(&path, &new_content).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write file: {}", e)))?;
        
        Ok(ToolOutput {
            success: true,
            result: json!({
                "path": path.display().to_string(),
                "replacements": replacements,
                "search": params.search,
                "replace": params.replace
            }),
            error: None,
            usage: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ListFilesTool {
    workspace_root: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct ListFilesParams {
    #[serde(default = "default_path")]
    path: String,
    #[serde(default)]
    recursive: bool,
    #[serde(default)]
    include_hidden: bool,
    #[serde(default)]
    pattern: Option<String>,
}

fn default_path() -> String {
    ".".to_string()
}

impl ListFilesTool {
    pub fn new() -> Self {
        Self {
            workspace_root: std::env::current_dir().ok(),
        }
    }
}

#[async_trait]
impl AgentTool for ListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }
    
    fn description(&self) -> &str {
        "List files in a directory with optional filtering"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to list (default: current directory)",
                    "default": "."
                },
                "recursive": {
                    "type": "boolean",
                    "description": "List files recursively",
                    "default": false
                },
                "include_hidden": {
                    "type": "boolean",
                    "description": "Include hidden files (starting with .)",
                    "default": false
                },
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern to filter files (e.g., '*.rs')"
                }
            }
        })
    }
    
    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: ListFilesParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;
        
        let read_tool = ReadFileTool::new();
        let path = read_tool.resolve_path(&params.path)?;
        
        if !path.is_dir() {
            return Err(ToolError::InvalidParameters(format!("Not a directory: {}", path.display())));
        }
        
        let mut files = Vec::new();
        let mut dirs = Vec::new();
        
        if params.recursive {
            use walkdir::WalkDir;
            let walker = WalkDir::new(&path)
                .follow_links(true)
                .max_depth(if params.recursive { 10 } else { 1 });
            
            for entry in walker {
                if let Ok(entry) = entry {
                    let relative = entry.path().strip_prefix(&path).unwrap_or(entry.path());
                    let name = relative.to_string_lossy().to_string();
                    
                    if !params.include_hidden && name.contains("/.") {
                        continue;
                    }
                    
                    if let Some(pattern) = &params.pattern {
                        if !glob::Pattern::new(pattern)
                            .map(|p| p.matches(&name))
                            .unwrap_or(false) {
                            continue;
                        }
                    }
                    
                    if entry.file_type().is_file() {
                        files.push(name);
                    } else if entry.file_type().is_dir() && entry.path() != path {
                        dirs.push(name);
                    }
                }
            }
        } else {
            let mut entries = fs::read_dir(&path).await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read directory: {}", e)))?;
            
            while let Some(entry) = entries.next_entry().await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read entry: {}", e)))? {
                let name = entry.file_name().to_string_lossy().to_string();
                
                if !params.include_hidden && name.starts_with('.') {
                    continue;
                }
                
                if let Some(pattern) = &params.pattern {
                    if !glob::Pattern::new(pattern)
                        .map(|p| p.matches(&name))
                        .unwrap_or(false) {
                        continue;
                    }
                }
                
                let file_type = entry.file_type().await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get file type: {}", e)))?;
                
                if file_type.is_file() {
                    files.push(name);
                } else if file_type.is_dir() {
                    dirs.push(name);
                }
            }
        }
        
        files.sort();
        dirs.sort();
        
        Ok(ToolOutput {
            success: true,
            result: json!({
                "path": path.display().to_string(),
                "files": files,
                "directories": dirs,
                "total_files": files.len(),
                "total_directories": dirs.len()
            }),
            error: None,
            usage: None,
        })
    }
}