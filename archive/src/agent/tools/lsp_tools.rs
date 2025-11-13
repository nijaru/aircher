use super::{AgentTool, ToolError, ToolOutput};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tracing::{debug, info};
use std::sync::Arc;

/// LSP client for managing language server connections
pub struct LspClient {
    servers: Arc<RwLock<HashMap<String, LanguageServer>>>,
    workspace_root: PathBuf,
}

struct LanguageServer {
    #[allow(dead_code)]
    process: Child,
    stdin: tokio::process::ChildStdin,
    stdout: BufReader<tokio::process::ChildStdout>,
    request_id: u64,
    #[allow(dead_code)]
    capabilities: ServerCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerCapabilities {
    completion: bool,
    hover: bool,
    definition: bool,
    references: bool,
    rename: bool,
    formatting: bool,
    code_actions: bool,
    diagnostics: bool,
}

impl LspClient {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            workspace_root,
        }
    }

    /// Start a language server for a specific language
    async fn start_language_server(&self, language: &str) -> Result<()> {
        let (command, args) = match language {
            "rust" => ("rust-analyzer", vec![]),
            "typescript" | "javascript" => ("typescript-language-server", vec!["--stdio"]),
            "python" => ("pylsp", vec![]),
            "go" => ("gopls", vec![]),
            "java" => ("jdtls", vec![]),
            "cpp" | "c" => ("clangd", vec![]),
            _ => return Err(anyhow::anyhow!("Unsupported language: {}", language)),
        };

        let mut child = Command::new(command)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("Failed to start {} language server", language))?;

        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());

        let mut server = LanguageServer {
            process: child,
            stdin,
            stdout,
            request_id: 1,
            capabilities: ServerCapabilities {
                completion: true,
                hover: true,
                definition: true,
                references: true,
                rename: true,
                formatting: true,
                code_actions: true,
                diagnostics: true,
            },
        };

        // Initialize the server
        self.initialize_server(&mut server).await?;

        let mut servers = self.servers.write().await;
        servers.insert(language.to_string(), server);

        info!("Started {} language server", language);
        Ok(())
    }

    /// Initialize LSP server with workspace
    async fn initialize_server(&self, server: &mut LanguageServer) -> Result<()> {
        let init_request = json!({
            "jsonrpc": "2.0",
            "id": server.request_id,
            "method": "initialize",
            "params": {
                "processId": std::process::id(),
                "rootUri": format!("file://{}", self.workspace_root.display()),
                "capabilities": {
                    "textDocument": {
                        "completion": {
                            "completionItem": {
                                "snippetSupport": true
                            }
                        },
                        "hover": {
                            "contentFormat": ["markdown", "plaintext"]
                        }
                    }
                }
            }
        });

        server.request_id += 1;

        let request = format!("Content-Length: {}\r\n\r\n{}",
            init_request.to_string().len(),
            init_request);

        server.stdin.write_all(request.as_bytes()).await?;
        server.stdin.flush().await?;

        // Read response (simplified - real implementation needs proper LSP message parsing)
        let mut response = String::new();
        server.stdout.read_line(&mut response).await?;

        debug!("LSP initialized: {}", response);
        Ok(())
    }

    /// Send a request to the language server
    async fn send_request(&self, language: &str, method: &str, params: Value) -> Result<Value> {
        let mut servers = self.servers.write().await;

        if !servers.contains_key(language) {
            drop(servers);
            self.start_language_server(language).await?;
            servers = self.servers.write().await;
        }

        let server = servers.get_mut(language)
            .ok_or_else(|| anyhow::anyhow!("Language server not available for {}", language))?;

        let request = json!({
            "jsonrpc": "2.0",
            "id": server.request_id,
            "method": method,
            "params": params
        });

        server.request_id += 1;

        let request_str = format!("Content-Length: {}\r\n\r\n{}",
            request.to_string().len(),
            request);

        server.stdin.write_all(request_str.as_bytes()).await?;
        server.stdin.flush().await?;

        // Read response (simplified)
        let mut response = String::new();
        server.stdout.read_line(&mut response).await?;

        Ok(serde_json::from_str(&response).unwrap_or(json!({})))
    }

    fn detect_language(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext {
                "rs" => "rust",
                "ts" | "tsx" => "typescript",
                "js" | "jsx" => "javascript",
                "py" => "python",
                "go" => "go",
                "java" => "java",
                "cpp" | "cc" | "cxx" => "cpp",
                "c" | "h" => "c",
                _ => "unknown",
            })
            .filter(|lang| *lang != "unknown")
            .map(String::from)
    }
}

/// Tool for getting code completions via LSP
#[derive(Clone)]
pub struct CodeCompletionTool {
    lsp_client: Arc<LspClient>,
}

#[derive(Debug, Deserialize)]
struct CompletionParams {
    file_path: String,
    line: u32,
    column: u32,
    #[serde(default)]
    trigger_character: Option<String>,
}

impl CodeCompletionTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            lsp_client: Arc::new(LspClient::new(workspace_root)),
        }
    }
}

#[async_trait]
impl AgentTool for CodeCompletionTool {
    fn name(&self) -> &str {
        "code_completion"
    }

    fn description(&self) -> &str {
        "Get intelligent code completions at a specific position"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file"
                },
                "line": {
                    "type": "integer",
                    "description": "Line number (0-indexed)"
                },
                "column": {
                    "type": "integer",
                    "description": "Column number (0-indexed)"
                },
                "trigger_character": {
                    "type": "string",
                    "description": "Optional trigger character (e.g., '.', '::')"
                }
            },
            "required": ["file_path", "line", "column"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: CompletionParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        let path = Path::new(&params.file_path);
        let language = LspClient::detect_language(path)
            .ok_or_else(|| ToolError::InvalidParameters("Unsupported file type".to_string()))?;

        let lsp_params = json!({
            "textDocument": {
                "uri": format!("file://{}", path.display())
            },
            "position": {
                "line": params.line,
                "character": params.column
            },
            "context": {
                "triggerKind": if params.trigger_character.is_some() { 2 } else { 1 },
                "triggerCharacter": params.trigger_character
            }
        });

        match self.lsp_client.send_request(&language, "textDocument/completion", lsp_params).await {
            Ok(response) => {
                Ok(ToolOutput {
                    success: true,
                    result: json!({
                        "completions": response["result"]["items"],
                        "file_path": params.file_path,
                        "position": {
                            "line": params.line,
                            "column": params.column
                        }
                    }),
                    error: None,
                    usage: None,
                })
            }
            Err(e) => {
                Ok(ToolOutput {
                    success: false,
                    result: json!({
                        "error": e.to_string()
                    }),
                    error: Some(e.to_string()),
                    usage: None,
                })
            }
        }
    }
}

/// Tool for getting hover information (documentation) via LSP
#[derive(Clone)]
pub struct HoverTool {
    lsp_client: Arc<LspClient>,
}

#[derive(Debug, Deserialize)]
struct HoverParams {
    file_path: String,
    line: u32,
    column: u32,
}

impl HoverTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            lsp_client: Arc::new(LspClient::new(workspace_root)),
        }
    }
}

#[async_trait]
impl AgentTool for HoverTool {
    fn name(&self) -> &str {
        "hover_info"
    }

    fn description(&self) -> &str {
        "Get documentation and type information for a symbol"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file"
                },
                "line": {
                    "type": "integer",
                    "description": "Line number (0-indexed)"
                },
                "column": {
                    "type": "integer",
                    "description": "Column number (0-indexed)"
                }
            },
            "required": ["file_path", "line", "column"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: HoverParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        let path = Path::new(&params.file_path);
        let language = LspClient::detect_language(path)
            .ok_or_else(|| ToolError::InvalidParameters("Unsupported file type".to_string()))?;

        let lsp_params = json!({
            "textDocument": {
                "uri": format!("file://{}", path.display())
            },
            "position": {
                "line": params.line,
                "character": params.column
            }
        });

        match self.lsp_client.send_request(&language, "textDocument/hover", lsp_params).await {
            Ok(response) => {
                Ok(ToolOutput {
                    success: true,
                    result: json!({
                        "hover": response["result"]["contents"],
                        "file_path": params.file_path,
                        "position": {
                            "line": params.line,
                            "column": params.column
                        }
                    }),
                    error: None,
                    usage: None,
                })
            }
            Err(e) => {
                Ok(ToolOutput {
                    success: false,
                    result: json!({
                        "error": e.to_string()
                    }),
                    error: Some(e.to_string()),
                    usage: None,
                })
            }
        }
    }
}

/// Tool for going to definition via LSP
#[derive(Clone)]
pub struct GoToDefinitionTool {
    lsp_client: Arc<LspClient>,
}

impl GoToDefinitionTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            lsp_client: Arc::new(LspClient::new(workspace_root)),
        }
    }
}

#[async_trait]
impl AgentTool for GoToDefinitionTool {
    fn name(&self) -> &str {
        "go_to_definition"
    }

    fn description(&self) -> &str {
        "Find the definition of a symbol across files"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file containing the symbol"
                },
                "line": {
                    "type": "integer",
                    "description": "Line number (0-indexed)"
                },
                "column": {
                    "type": "integer",
                    "description": "Column number (0-indexed)"
                }
            },
            "required": ["file_path", "line", "column"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: HoverParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        let path = Path::new(&params.file_path);
        let language = LspClient::detect_language(path)
            .ok_or_else(|| ToolError::InvalidParameters("Unsupported file type".to_string()))?;

        let lsp_params = json!({
            "textDocument": {
                "uri": format!("file://{}", path.display())
            },
            "position": {
                "line": params.line,
                "character": params.column
            }
        });

        match self.lsp_client.send_request(&language, "textDocument/definition", lsp_params).await {
            Ok(response) => {
                Ok(ToolOutput {
                    success: true,
                    result: json!({
                        "definitions": response["result"],
                        "source": {
                            "file_path": params.file_path,
                            "line": params.line,
                            "column": params.column
                        }
                    }),
                    error: None,
                    usage: None,
                })
            }
            Err(e) => {
                Ok(ToolOutput {
                    success: false,
                    result: json!({
                        "error": e.to_string()
                    }),
                    error: Some(e.to_string()),
                    usage: None,
                })
            }
        }
    }
}

/// Tool for finding all references to a symbol
#[derive(Clone)]
pub struct FindReferencesTool {
    lsp_client: Arc<LspClient>,
}

impl FindReferencesTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            lsp_client: Arc::new(LspClient::new(workspace_root)),
        }
    }
}

#[async_trait]
impl AgentTool for FindReferencesTool {
    fn name(&self) -> &str {
        "find_references"
    }

    fn description(&self) -> &str {
        "Find all references to a symbol across the codebase"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file containing the symbol"
                },
                "line": {
                    "type": "integer",
                    "description": "Line number (0-indexed)"
                },
                "column": {
                    "type": "integer",
                    "description": "Column number (0-indexed)"
                },
                "include_declaration": {
                    "type": "boolean",
                    "description": "Include the declaration in results",
                    "default": true
                }
            },
            "required": ["file_path", "line", "column"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let file_path = params["file_path"].as_str()
            .ok_or_else(|| ToolError::InvalidParameters("Missing file_path".to_string()))?;
        let line = params["line"].as_u64()
            .ok_or_else(|| ToolError::InvalidParameters("Missing line".to_string()))? as u32;
        let column = params["column"].as_u64()
            .ok_or_else(|| ToolError::InvalidParameters("Missing column".to_string()))? as u32;
        let include_declaration = params["include_declaration"].as_bool().unwrap_or(true);

        let path = Path::new(file_path);
        let language = LspClient::detect_language(path)
            .ok_or_else(|| ToolError::InvalidParameters("Unsupported file type".to_string()))?;

        let lsp_params = json!({
            "textDocument": {
                "uri": format!("file://{}", path.display())
            },
            "position": {
                "line": line,
                "character": column
            },
            "context": {
                "includeDeclaration": include_declaration
            }
        });

        match self.lsp_client.send_request(&language, "textDocument/references", lsp_params).await {
            Ok(response) => {
                Ok(ToolOutput {
                    success: true,
                    result: json!({
                        "references": response["result"],
                        "source": {
                            "file_path": file_path,
                            "line": line,
                            "column": column
                        }
                    }),
                    error: None,
                    usage: None,
                })
            }
            Err(e) => {
                Ok(ToolOutput {
                    success: false,
                    result: json!({
                        "error": e.to_string()
                    }),
                    error: Some(e.to_string()),
                    usage: None,
                })
            }
        }
    }
}

/// Tool for renaming symbols across the codebase
#[derive(Clone)]
pub struct RenameSymbolTool {
    lsp_client: Arc<LspClient>,
}

#[derive(Debug, Deserialize)]
struct RenameParams {
    file_path: String,
    line: u32,
    column: u32,
    new_name: String,
}

impl RenameSymbolTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            lsp_client: Arc::new(LspClient::new(workspace_root)),
        }
    }
}

#[async_trait]
impl AgentTool for RenameSymbolTool {
    fn name(&self) -> &str {
        "rename_symbol"
    }

    fn description(&self) -> &str {
        "Rename a symbol across all its usages in the codebase"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file containing the symbol"
                },
                "line": {
                    "type": "integer",
                    "description": "Line number (0-indexed)"
                },
                "column": {
                    "type": "integer",
                    "description": "Column number (0-indexed)"
                },
                "new_name": {
                    "type": "string",
                    "description": "New name for the symbol"
                }
            },
            "required": ["file_path", "line", "column", "new_name"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: RenameParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        let path = Path::new(&params.file_path);
        let language = LspClient::detect_language(path)
            .ok_or_else(|| ToolError::InvalidParameters("Unsupported file type".to_string()))?;

        let lsp_params = json!({
            "textDocument": {
                "uri": format!("file://{}", path.display())
            },
            "position": {
                "line": params.line,
                "character": params.column
            },
            "newName": params.new_name
        });

        match self.lsp_client.send_request(&language, "textDocument/rename", lsp_params).await {
            Ok(response) => {
                Ok(ToolOutput {
                    success: true,
                    result: json!({
                        "workspace_edit": response["result"],
                        "renamed": {
                            "old_name": "unknown", // Would need to fetch this
                            "new_name": params.new_name,
                            "file_path": params.file_path,
                            "position": {
                                "line": params.line,
                                "column": params.column
                            }
                        }
                    }),
                    error: None,
                    usage: None,
                })
            }
            Err(e) => {
                Ok(ToolOutput {
                    success: false,
                    result: json!({
                        "error": e.to_string()
                    }),
                    error: Some(e.to_string()),
                    usage: None,
                })
            }
        }
    }
}

/// Tool for getting diagnostics (errors, warnings) from LSP
#[derive(Clone)]
pub struct DiagnosticsTool {
    lsp_client: Arc<LspClient>,
}

#[derive(Debug, Deserialize)]
struct DiagnosticsParams {
    file_path: String,
    #[serde(default)]
    severity: Option<String>, // error, warning, info, hint
}

impl DiagnosticsTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            lsp_client: Arc::new(LspClient::new(workspace_root)),
        }
    }
}

#[async_trait]
impl AgentTool for DiagnosticsTool {
    fn name(&self) -> &str {
        "get_diagnostics"
    }

    fn description(&self) -> &str {
        "Get code diagnostics (errors, warnings) for a file"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to check"
                },
                "severity": {
                    "type": "string",
                    "enum": ["error", "warning", "info", "hint"],
                    "description": "Filter by severity level"
                }
            },
            "required": ["file_path"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: DiagnosticsParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        let path = Path::new(&params.file_path);
        let language = LspClient::detect_language(path)
            .ok_or_else(|| ToolError::InvalidParameters("Unsupported file type".to_string()))?;

        // Note: Real implementation would need to handle LSP's push-based diagnostics
        // This is a simplified version that requests diagnostics
        let lsp_params = json!({
            "textDocument": {
                "uri": format!("file://{}", path.display())
            }
        });

        match self.lsp_client.send_request(&language, "textDocument/diagnostic", lsp_params).await {
            Ok(response) => {
                let diagnostics = response["result"]["items"].as_array()
                    .map(|items| {
                        if let Some(severity_filter) = &params.severity {
                            items.iter()
                                .filter(|d| {
                                    let severity = d["severity"].as_u64().unwrap_or(0);
                                    match severity_filter.as_str() {
                                        "error" => severity == 1,
                                        "warning" => severity == 2,
                                        "info" => severity == 3,
                                        "hint" => severity == 4,
                                        _ => true,
                                    }
                                })
                                .cloned()
                                .collect()
                        } else {
                            items.clone()
                        }
                    })
                    .unwrap_or_default();

                Ok(ToolOutput {
                    success: true,
                    result: json!({
                        "diagnostics": diagnostics,
                        "file_path": params.file_path,
                        "count": diagnostics.len(),
                        "severity_filter": params.severity
                    }),
                    error: None,
                    usage: None,
                })
            }
            Err(e) => {
                Ok(ToolOutput {
                    success: false,
                    result: json!({
                        "error": e.to_string()
                    }),
                    error: Some(e.to_string()),
                    usage: None,
                })
            }
        }
    }
}

/// Tool for formatting code via LSP
#[derive(Clone)]
pub struct FormatCodeTool {
    lsp_client: Arc<LspClient>,
}

#[derive(Debug, Deserialize)]
struct FormatParams {
    file_path: String,
    #[serde(default)]
    tab_size: Option<u32>,
    #[serde(default)]
    insert_spaces: Option<bool>,
}

impl FormatCodeTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            lsp_client: Arc::new(LspClient::new(workspace_root)),
        }
    }
}

#[async_trait]
impl AgentTool for FormatCodeTool {
    fn name(&self) -> &str {
        "format_code"
    }

    fn description(&self) -> &str {
        "Format code according to language-specific style rules"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to format"
                },
                "tab_size": {
                    "type": "integer",
                    "description": "Number of spaces for a tab",
                    "default": 4
                },
                "insert_spaces": {
                    "type": "boolean",
                    "description": "Use spaces instead of tabs",
                    "default": true
                }
            },
            "required": ["file_path"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: FormatParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        let path = Path::new(&params.file_path);
        let language = LspClient::detect_language(path)
            .ok_or_else(|| ToolError::InvalidParameters("Unsupported file type".to_string()))?;

        let lsp_params = json!({
            "textDocument": {
                "uri": format!("file://{}", path.display())
            },
            "options": {
                "tabSize": params.tab_size.unwrap_or(4),
                "insertSpaces": params.insert_spaces.unwrap_or(true)
            }
        });

        match self.lsp_client.send_request(&language, "textDocument/formatting", lsp_params).await {
            Ok(response) => {
                Ok(ToolOutput {
                    success: true,
                    result: json!({
                        "edits": response["result"],
                        "file_path": params.file_path,
                        "formatted": true
                    }),
                    error: None,
                    usage: None,
                })
            }
            Err(e) => {
                Ok(ToolOutput {
                    success: false,
                    result: json!({
                        "error": e.to_string()
                    }),
                    error: Some(e.to_string()),
                    usage: None,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_language_detection() {
        assert_eq!(LspClient::detect_language(Path::new("test.rs")), Some("rust".to_string()));
        assert_eq!(LspClient::detect_language(Path::new("test.ts")), Some("typescript".to_string()));
        assert_eq!(LspClient::detect_language(Path::new("test.py")), Some("python".to_string()));
        assert_eq!(LspClient::detect_language(Path::new("test.go")), Some("go".to_string()));
        assert_eq!(LspClient::detect_language(Path::new("test.txt")), None);
    }

    #[tokio::test]
    async fn test_code_completion_tool() {
        let temp_dir = TempDir::new().unwrap();
        let tool = CodeCompletionTool::new(temp_dir.path().to_path_buf());

        assert_eq!(tool.name(), "code_completion");
        assert!(tool.description().contains("completion"));

        let schema = tool.parameters_schema();
        assert!(schema["properties"]["file_path"].is_object());
        assert!(schema["required"].as_array().unwrap().contains(&json!("file_path")));
    }

    #[tokio::test]
    async fn test_hover_tool() {
        let temp_dir = TempDir::new().unwrap();
        let tool = HoverTool::new(temp_dir.path().to_path_buf());

        assert_eq!(tool.name(), "hover_info");
        assert!(tool.description().contains("documentation"));
    }

    #[tokio::test]
    async fn test_diagnostics_tool() {
        let temp_dir = TempDir::new().unwrap();
        let tool = DiagnosticsTool::new(temp_dir.path().to_path_buf());

        assert_eq!(tool.name(), "get_diagnostics");
        assert!(tool.description().contains("errors"));

        let schema = tool.parameters_schema();
        assert!(schema["properties"]["severity"]["enum"].is_array());
    }
}
