// Enhanced ListFiles Tool - Production Implementation
// Week 1 Sprint: Real tool with filtering, metadata, and recursive traversal

use super::{AgentTool, ToolError, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::fs;
use walkdir::WalkDir;
use glob::Pattern;

// Common patterns to exclude
const DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    ".git/**",
    "node_modules/**",
    "target/**",
    "build/**",
    "dist/**",
    ".idea/**",
    ".vscode/**",
    "__pycache__/**",
    "*.pyc",
    ".DS_Store",
    "Thumbs.db",
];

#[derive(Debug, Clone)]
pub struct EnhancedListFilesTool {
    workspace_root: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct ListFilesParams {
    path: String,
    #[serde(default)]
    recursive: bool,
    #[serde(default)]
    include_hidden: bool,
    #[serde(default)]
    include_dirs: bool,
    #[serde(default)]
    max_depth: Option<usize>,
    #[serde(default)]
    pattern: Option<String>,
    #[serde(default)]
    extensions: Vec<String>,
    #[serde(default)]
    exclude_patterns: Vec<String>,
    #[serde(default)]
    include_metadata: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileEntry {
    path: String,
    name: String,
    is_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extension: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListResult {
    directory: String,
    entries: Vec<FileEntry>,
    total_files: usize,
    total_dirs: usize,
    total_size: u64,
    truncated: bool,
    max_entries_reached: bool,
}

impl EnhancedListFilesTool {
    pub fn new() -> Self {
        Self {
            workspace_root: std::env::current_dir().ok(),
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

    fn should_exclude(&self, path: &Path, exclude_patterns: &[String]) -> bool {
        let path_str = path.to_string_lossy();

        // Check custom exclude patterns
        for pattern_str in exclude_patterns {
            if let Ok(pattern) = Pattern::new(pattern_str) {
                if pattern.matches(&path_str) {
                    return true;
                }
            }
        }

        // Check default ignore patterns
        for pattern_str in DEFAULT_IGNORE_PATTERNS {
            if let Ok(pattern) = Pattern::new(pattern_str) {
                if pattern.matches(&path_str) {
                    return true;
                }
            }
        }

        false
    }

    fn matches_pattern(&self, path: &Path, pattern: &Option<String>) -> bool {
        if let Some(pattern_str) = pattern {
            if let Ok(pattern) = Pattern::new(pattern_str) {
                return pattern.matches(&path.to_string_lossy());
            }
        }
        true
    }

    fn matches_extension(&self, path: &Path, extensions: &[String]) -> bool {
        if extensions.is_empty() {
            return true;
        }

        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            return extensions.iter().any(|e| e.to_lowercase() == ext_str);
        }

        false
    }

    async fn get_file_metadata(&self, path: &Path) -> Result<(Option<u64>, Option<String>), ToolError> {
        match fs::metadata(path).await {
            Ok(metadata) => {
                let size = if metadata.is_file() {
                    Some(metadata.len())
                } else {
                    None
                };

                let modified = metadata.modified()
                    .ok()
                    .and_then(|time| {
                        use std::time::SystemTime;
                        time.duration_since(SystemTime::UNIX_EPOCH).ok()
                    })
                    .map(|duration| {
                        use chrono::{DateTime, Utc};
                        let datetime = DateTime::<Utc>::from_timestamp(duration.as_secs() as i64, 0);
                        datetime.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    })
                    .flatten();

                Ok((size, modified))
            }
            Err(e) => {
                // Don't fail entirely, just skip metadata
                eprintln!("Warning: Failed to get metadata for {:?}: {}", path, e);
                Ok((None, None))
            }
        }
    }

    async fn list_directory(
        &self,
        path: &Path,
        params: &ListFilesParams,
    ) -> Result<ListResult, ToolError> {
        if !path.exists() {
            return Err(ToolError::NotFound(format!("Directory not found: {}", path.display())));
        }

        if !path.is_dir() {
            return Err(ToolError::InvalidParameters(format!("Path is not a directory: {}", path.display())));
        }

        let mut entries = Vec::new();
        let mut total_files = 0;
        let mut total_dirs = 0;
        let mut total_size = 0u64;
        let max_entries = 1000; // Prevent overwhelming output
        let mut max_entries_reached = false;

        if params.recursive {
            // Recursive traversal using walkdir
            let mut walker = WalkDir::new(path)
                .follow_links(false);

            if let Some(max_depth) = params.max_depth {
                walker = walker.max_depth(max_depth);
            }

            for entry_result in walker {
                if entries.len() >= max_entries {
                    max_entries_reached = true;
                    break;
                }

                let entry = match entry_result {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("Warning: Failed to read entry: {}", e);
                        continue;
                    }
                };

                let entry_path = entry.path();

                // Skip if excluded
                if self.should_exclude(entry_path, &params.exclude_patterns) {
                    continue;
                }

                // Skip hidden files unless requested
                if !params.include_hidden {
                    if let Some(file_name) = entry_path.file_name() {
                        if file_name.to_string_lossy().starts_with('.') {
                            continue;
                        }
                    }
                }

                let is_dir = entry_path.is_dir();

                // Skip directories unless requested
                if is_dir && !params.include_dirs {
                    continue;
                }

                // Apply filters
                if !is_dir {
                    if !self.matches_pattern(entry_path, &params.pattern) {
                        continue;
                    }

                    if !self.matches_extension(entry_path, &params.extensions) {
                        continue;
                    }
                }

                // Get metadata if requested
                let (size, modified) = if params.include_metadata {
                    self.get_file_metadata(entry_path).await?
                } else {
                    (None, None)
                };

                if let Some(s) = size {
                    total_size += s;
                }

                if is_dir {
                    total_dirs += 1;
                } else {
                    total_files += 1;
                }

                let file_entry = FileEntry {
                    path: entry_path.display().to_string(),
                    name: entry_path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    is_dir,
                    size,
                    modified,
                    extension: entry_path.extension()
                        .map(|e| e.to_string_lossy().to_string()),
                };

                entries.push(file_entry);
            }
        } else {
            // Non-recursive: just list immediate children
            let mut read_dir = fs::read_dir(path).await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read directory: {}", e)))?;

            while let Some(entry) = read_dir.next_entry().await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read entry: {}", e)))? {

                if entries.len() >= max_entries {
                    max_entries_reached = true;
                    break;
                }

                let entry_path = entry.path();

                // Skip if excluded
                if self.should_exclude(&entry_path, &params.exclude_patterns) {
                    continue;
                }

                // Skip hidden files unless requested
                if !params.include_hidden {
                    if let Some(file_name) = entry_path.file_name() {
                        if file_name.to_string_lossy().starts_with('.') {
                            continue;
                        }
                    }
                }

                let is_dir = entry_path.is_dir();

                // Skip directories unless requested
                if is_dir && !params.include_dirs {
                    continue;
                }

                // Apply filters
                if !is_dir {
                    if !self.matches_pattern(&entry_path, &params.pattern) {
                        continue;
                    }

                    if !self.matches_extension(&entry_path, &params.extensions) {
                        continue;
                    }
                }

                // Get metadata if requested
                let (size, modified) = if params.include_metadata {
                    self.get_file_metadata(&entry_path).await?
                } else {
                    (None, None)
                };

                if let Some(s) = size {
                    total_size += s;
                }

                if is_dir {
                    total_dirs += 1;
                } else {
                    total_files += 1;
                }

                let file_entry = FileEntry {
                    path: entry_path.display().to_string(),
                    name: entry_path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    is_dir,
                    size,
                    modified,
                    extension: entry_path.extension()
                        .map(|e| e.to_string_lossy().to_string()),
                };

                entries.push(file_entry);
            }
        }

        // Sort entries: directories first, then alphabetically
        entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        Ok(ListResult {
            directory: path.display().to_string(),
            entries,
            total_files,
            total_dirs,
            total_size,
            truncated: max_entries_reached,
            max_entries_reached,
        })
    }
}

#[async_trait]
impl AgentTool for EnhancedListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }

    fn description(&self) -> &str {
        "List files and directories with filtering, metadata, and recursive traversal support. \
        Supports glob patterns, extension filtering, and automatic exclusion of common build/cache directories."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to list (absolute or relative to workspace)"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Recursively list subdirectories (default: false)",
                    "default": false
                },
                "include_hidden": {
                    "type": "boolean",
                    "description": "Include hidden files (starting with .) (default: false)",
                    "default": false
                },
                "include_dirs": {
                    "type": "boolean",
                    "description": "Include directories in results (default: false)",
                    "default": false
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum recursion depth (only applies if recursive=true)",
                    "minimum": 1
                },
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern to filter files (e.g., '*.rs', 'test_*.py')"
                },
                "extensions": {
                    "type": "array",
                    "description": "List of file extensions to include (e.g., ['rs', 'toml'])",
                    "items": {"type": "string"}
                },
                "exclude_patterns": {
                    "type": "array",
                    "description": "Glob patterns to exclude (in addition to default ignores)",
                    "items": {"type": "string"}
                },
                "include_metadata": {
                    "type": "boolean",
                    "description": "Include file metadata (size, modified time, etc.) (default: false)",
                    "default": false
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: ListFilesParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        let path = self.resolve_path(&params.path)?;

        let result = self.list_directory(&path, &params).await?;

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
    async fn test_list_files_non_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create test structure
        fs::write(base_path.join("file1.txt"), "content1").await.unwrap();
        fs::write(base_path.join("file2.rs"), "content2").await.unwrap();
        fs::create_dir(base_path.join("subdir")).await.unwrap();
        fs::write(base_path.join("subdir/file3.txt"), "content3").await.unwrap();

        let tool = EnhancedListFilesTool::new();
        let params = json!({
            "path": base_path.to_str().unwrap(),
            "recursive": false,
            "include_dirs": false,
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);

        let list_result: ListResult = serde_json::from_value(output.result).unwrap();
        assert_eq!(list_result.total_files, 2); // Only file1.txt and file2.rs
        assert_eq!(list_result.total_dirs, 0); // include_dirs=false
    }

    #[tokio::test]
    async fn test_list_files_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create nested structure
        fs::write(base_path.join("file1.txt"), "content1").await.unwrap();
        fs::create_dir(base_path.join("subdir")).await.unwrap();
        fs::write(base_path.join("subdir/file2.txt"), "content2").await.unwrap();
        fs::create_dir(base_path.join("subdir/nested")).await.unwrap();
        fs::write(base_path.join("subdir/nested/file3.txt"), "content3").await.unwrap();

        let tool = EnhancedListFilesTool::new();
        let params = json!({
            "path": base_path.to_str().unwrap(),
            "recursive": true,
            "include_dirs": false,
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        let list_result: ListResult = serde_json::from_value(output.result).unwrap();
        assert_eq!(list_result.total_files, 3); // All three txt files
    }

    #[tokio::test]
    async fn test_extension_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        fs::write(base_path.join("file1.txt"), "content1").await.unwrap();
        fs::write(base_path.join("file2.rs"), "content2").await.unwrap();
        fs::write(base_path.join("file3.md"), "content3").await.unwrap();

        let tool = EnhancedListFilesTool::new();
        let params = json!({
            "path": base_path.to_str().unwrap(),
            "extensions": ["rs", "md"],
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        let list_result: ListResult = serde_json::from_value(output.result).unwrap();
        assert_eq!(list_result.total_files, 2); // Only .rs and .md files
    }

    #[tokio::test]
    async fn test_pattern_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        fs::write(base_path.join("test_one.txt"), "content1").await.unwrap();
        fs::write(base_path.join("test_two.txt"), "content2").await.unwrap();
        fs::write(base_path.join("other.txt"), "content3").await.unwrap();

        let tool = EnhancedListFilesTool::new();
        let params = json!({
            "path": base_path.to_str().unwrap(),
            "pattern": "test_*.txt",
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        let list_result: ListResult = serde_json::from_value(output.result).unwrap();
        assert_eq!(list_result.total_files, 2); // Only test_* files
    }

    #[tokio::test]
    async fn test_hidden_files() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        fs::write(base_path.join("visible.txt"), "content1").await.unwrap();
        fs::write(base_path.join(".hidden.txt"), "content2").await.unwrap();

        let tool = EnhancedListFilesTool::new();

        // Without include_hidden
        let params = json!({
            "path": base_path.to_str().unwrap(),
            "include_hidden": false,
        });

        let result = tool.execute(params).await.unwrap();
        let list_result: ListResult = serde_json::from_value(result.result).unwrap();
        assert_eq!(list_result.total_files, 1); // Only visible.txt

        // With include_hidden
        let params = json!({
            "path": base_path.to_str().unwrap(),
            "include_hidden": true,
        });

        let result = tool.execute(params).await.unwrap();
        let list_result: ListResult = serde_json::from_value(result.result).unwrap();
        assert_eq!(list_result.total_files, 2); // Both files
    }

    #[tokio::test]
    async fn test_include_directories() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        fs::write(base_path.join("file.txt"), "content").await.unwrap();
        fs::create_dir(base_path.join("dir1")).await.unwrap();
        fs::create_dir(base_path.join("dir2")).await.unwrap();

        let tool = EnhancedListFilesTool::new();
        let params = json!({
            "path": base_path.to_str().unwrap(),
            "include_dirs": true,
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        let list_result: ListResult = serde_json::from_value(output.result).unwrap();
        assert_eq!(list_result.total_files, 1);
        assert_eq!(list_result.total_dirs, 2);

        // Directories should be sorted first
        assert!(list_result.entries[0].is_dir);
        assert!(list_result.entries[1].is_dir);
        assert!(!list_result.entries[2].is_dir);
    }

    #[tokio::test]
    async fn test_metadata_inclusion() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        fs::write(base_path.join("file.txt"), "content").await.unwrap();

        let tool = EnhancedListFilesTool::new();
        let params = json!({
            "path": base_path.to_str().unwrap(),
            "include_metadata": true,
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        let list_result: ListResult = serde_json::from_value(output.result).unwrap();

        assert_eq!(list_result.entries.len(), 1);
        let entry = &list_result.entries[0];
        assert!(entry.size.is_some());
        assert!(entry.modified.is_some());
        assert_eq!(entry.size.unwrap(), 7); // "content" is 7 bytes
    }

    #[tokio::test]
    async fn test_max_depth() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create deep nesting
        fs::write(base_path.join("file0.txt"), "level0").await.unwrap();
        fs::create_dir(base_path.join("level1")).await.unwrap();
        fs::write(base_path.join("level1/file1.txt"), "level1").await.unwrap();
        fs::create_dir(base_path.join("level1/level2")).await.unwrap();
        fs::write(base_path.join("level1/level2/file2.txt"), "level2").await.unwrap();

        let tool = EnhancedListFilesTool::new();
        let params = json!({
            "path": base_path.to_str().unwrap(),
            "recursive": true,
            "max_depth": 2, // Only go 2 levels deep
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        let list_result: ListResult = serde_json::from_value(output.result).unwrap();
        assert_eq!(list_result.total_files, 2); // file0.txt and file1.txt only (not file2.txt)
    }
}
