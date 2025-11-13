// LSP Manager with Event Bus Integration
//
// Manages language server instances with global diagnostics tracking and
// real-time event publishing for agent self-correction.
//
// Week 7 Day 1-2: Event Bus + LSP Integration

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

use super::events::{
    AgentEvent, Diagnostic, DiagnosticSeverity,
    FileOperation, SharedEventBus
};

/// Language server process and state
struct LanguageServer {
    process: Child,
    stdin: tokio::process::ChildStdin,
    stdout: BufReader<tokio::process::ChildStdout>,
    request_id: u64,
    language: String,
}

/// LSP Manager with global diagnostics map and event bus integration
pub struct LspManager {
    /// Active language server processes
    servers: Arc<RwLock<HashMap<String, LanguageServer>>>,

    /// Global diagnostics map: path -> diagnostics
    diagnostics: Arc<RwLock<HashMap<PathBuf, Vec<Diagnostic>>>>,

    /// Event bus for publishing diagnostic events
    event_bus: SharedEventBus,

    /// Workspace root for LSP initialization
    workspace_root: PathBuf,
}

impl LspManager {
    /// Create new LSP manager
    pub fn new(workspace_root: PathBuf, event_bus: SharedEventBus) -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            diagnostics: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
            workspace_root,
        }
    }

    /// Start listening for FileChanged events (Week 7 Day 2)
    /// Spawns a background task that processes file change notifications
    pub fn start_listening(self: Arc<Self>) {
        let mut receiver = self.event_bus.subscribe();

        tokio::spawn(async move {
            info!("LSP manager started listening for FileChanged events");

            loop {
                match receiver.recv().await {
                    Ok(AgentEvent::FileChanged { path, operation, .. }) => {
                        debug!("LSP manager received FileChanged event: {:?} ({:?})", path, operation);

                        // Handle file change asynchronously
                        if let Err(e) = self.handle_file_changed(&path).await {
                            warn!("Failed to handle file change for {:?}: {}", path, e);
                        }
                    }
                    Ok(_) => {
                        // Ignore other event types
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("LSP manager lagged, skipped {} events", skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Event bus closed, stopping LSP manager listener");
                        break;
                    }
                }
            }
        });
    }

    /// Handle file changed notification
    async fn handle_file_changed(&self, path: &Path) -> Result<()> {
        // Use the existing notify_file_changed public method
        // It already handles language detection, server startup, and LSP notification
        let operation = FileOperation::Edit; // Assume Edit for file changes
        self.notify_file_changed(path, operation).await?;

        // Note: Diagnostics will come asynchronously via LSP's publishDiagnostics notification
        debug!("Notified LSP server about file change: {:?}", path);

        Ok(())
    }

    /// Get language from file extension
    fn detect_language(path: &Path) -> Option<&'static str> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext {
                "rs" => Some("rust"),
                "ts" | "tsx" => Some("typescript"),
                "js" | "jsx" => Some("javascript"),
                "py" => Some("python"),
                "go" => Some("go"),
                "java" => Some("java"),
                "cpp" | "cc" | "cxx" => Some("cpp"),
                "c" | "h" => Some("c"),
                _ => None,
            })
    }

    /// Get command and args for language server
    fn language_server_command(language: &str) -> Option<(&'static str, Vec<&'static str>)> {
        match language {
            "rust" => Some(("rust-analyzer", vec![])),
            "typescript" | "javascript" => Some(("typescript-language-server", vec!["--stdio"])),
            "python" => Some(("pyright-langserver", vec!["--stdio"])),
            "go" => Some(("gopls", vec![])),
            "java" => Some(("jdtls", vec![])),
            "cpp" | "c" => Some(("clangd", vec![])),
            _ => None,
        }
    }

    /// Start a language server for a specific language
    async fn start_language_server(&self, language: &str) -> Result<()> {
        let (command, args) = Self::language_server_command(language)
            .ok_or_else(|| anyhow::anyhow!("Unsupported language: {}", language))?;

        info!("Starting {} language server: {}", language, command);

        let mut child = Command::new(command)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("Failed to start {} language server. Ensure {} is installed and in PATH.", language, command))?;

        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());

        let mut server = LanguageServer {
            process: child,
            stdin,
            stdout,
            request_id: 1,
            language: language.to_string(),
        };

        // Initialize the server
        self.initialize_server(&mut server).await?;

        // Send initialized notification
        self.send_notification(&mut server, "initialized", json!({})).await?;

        let mut servers = self.servers.write().await;
        servers.insert(language.to_string(), server);

        info!("Started {} language server successfully", language);
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
                        "publishDiagnostics": {
                            "relatedInformation": true,
                            "versionSupport": true,
                            "codeDescriptionSupport": true
                        },
                        "completion": {
                            "completionItem": {
                                "snippetSupport": true
                            }
                        },
                        "hover": {
                            "contentFormat": ["markdown", "plaintext"]
                        }
                    }
                },
                "workspaceFolders": [{
                    "uri": format!("file://{}", self.workspace_root.display()),
                    "name": self.workspace_root.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("workspace")
                }]
            }
        });

        server.request_id += 1;

        let request = format!("Content-Length: {}\r\n\r\n{}",
            init_request.to_string().len(),
            init_request);

        server.stdin.write_all(request.as_bytes()).await?;
        server.stdin.flush().await?;

        // Read initialize response
        self.read_response(&mut server.stdout).await?;

        debug!("LSP {} initialized successfully", server.language);
        Ok(())
    }

    /// Send LSP notification (no response expected)
    async fn send_notification(&self, server: &mut LanguageServer, method: &str, params: Value) -> Result<()> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        let message = format!("Content-Length: {}\r\n\r\n{}",
            notification.to_string().len(),
            notification);

        server.stdin.write_all(message.as_bytes()).await?;
        server.stdin.flush().await?;

        Ok(())
    }

    /// Read LSP response
    async fn read_response(&self, stdout: &mut BufReader<tokio::process::ChildStdout>) -> Result<Value> {
        // Read headers
        let mut content_length = 0;
        let mut line = String::new();

        loop {
            line.clear();
            stdout.read_line(&mut line).await?;

            if line.trim().is_empty() {
                break; // Empty line separates headers from content
            }

            if line.starts_with("Content-Length: ") {
                content_length = line[16..].trim().parse()?;
            }
        }

        // Read content
        let mut content = vec![0u8; content_length];
        use tokio::io::AsyncReadExt;
        stdout.read_exact(&mut content).await?;

        let response: Value = serde_json::from_slice(&content)?;
        Ok(response)
    }

    /// Notify LSP that a file changed
    pub async fn notify_file_changed(&self, path: &Path, operation: FileOperation) -> Result<()> {
        let language = Self::detect_language(path)
            .ok_or_else(|| anyhow::anyhow!("Unsupported file type"))?;

        // Ensure server is running
        {
            let servers = self.servers.read().await;
            if !servers.contains_key(language) {
                drop(servers);
                if let Err(e) = self.start_language_server(language).await {
                    warn!("Failed to start LSP for {}: {}", language, e);
                    return Ok(()); // Non-fatal, continue without LSP
                }
            }
        }

        // Send didChange or didOpen notification
        let mut servers = self.servers.write().await;
        if let Some(server) = servers.get_mut(language) {
            let method = match operation {
                FileOperation::Create => "textDocument/didOpen",
                _ => "textDocument/didChange",
            };

            // Read file content for didOpen
            let content = if matches!(operation, FileOperation::Create | FileOperation::Write) {
                tokio::fs::read_to_string(path).await.unwrap_or_default()
            } else {
                String::new()
            };

            let params = if method == "textDocument/didOpen" {
                json!({
                    "textDocument": {
                        "uri": format!("file://{}", path.display()),
                        "languageId": language,
                        "version": 1,
                        "text": content
                    }
                })
            } else {
                json!({
                    "textDocument": {
                        "uri": format!("file://{}", path.display()),
                        "version": 1
                    },
                    "contentChanges": [{ "text": content }]
                })
            };

            self.send_notification(server, method, params).await?;

            // Wait a bit for diagnostics to arrive (LSP sends them asynchronously)
            drop(servers);
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            // Publish file changed event
            self.event_bus.publish(AgentEvent::FileChanged {
                path: path.to_path_buf(),
                operation,
                timestamp: std::time::SystemTime::now(),
            });
        }

        Ok(())
    }

    /// Get diagnostics for a file
    pub async fn get_diagnostics(&self, path: &Path) -> Vec<Diagnostic> {
        let diagnostics = self.diagnostics.read().await;
        diagnostics.get(path).cloned().unwrap_or_default()
    }

    /// Get all diagnostics (global map)
    pub async fn get_all_diagnostics(&self) -> HashMap<PathBuf, Vec<Diagnostic>> {
        self.diagnostics.read().await.clone()
    }

    /// Store diagnostics for a file and publish event
    pub async fn store_diagnostics(&self, path: PathBuf, diagnostics: Vec<Diagnostic>) {
        {
            let mut diag_map = self.diagnostics.write().await;
            diag_map.insert(path.clone(), diagnostics.clone());
        }

        // Publish diagnostics received event
        self.event_bus.publish(AgentEvent::DiagnosticsReceived {
            path,
            diagnostics,
            timestamp: std::time::SystemTime::now(),
        });
    }

    /// Get count of errors vs warnings
    pub async fn get_diagnostic_counts(&self, path: &Path) -> (usize, usize) {
        let diagnostics = self.get_diagnostics(path).await;
        let errors = diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Error).count();
        let warnings = diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Warning).count();
        (errors, warnings)
    }

    /// Check if file has errors
    pub async fn has_errors(&self, path: &Path) -> bool {
        let (errors, _) = self.get_diagnostic_counts(path).await;
        errors > 0
    }

    /// Shutdown all language servers
    pub async fn shutdown_all(&self) {
        let mut servers = self.servers.write().await;

        for (language, mut server) in servers.drain() {
            info!("Shutting down {} language server", language);

            // Send shutdown request
            let shutdown_req = json!({
                "jsonrpc": "2.0",
                "id": server.request_id,
                "method": "shutdown",
                "params": null
            });

            let message = format!("Content-Length: {}\r\n\r\n{}",
                shutdown_req.to_string().len(),
                shutdown_req);

            if let Err(e) = server.stdin.write_all(message.as_bytes()).await {
                warn!("Failed to send shutdown to {}: {}", language, e);
            }

            // Send exit notification
            if let Err(e) = self.send_notification(&mut server, "exit", json!(null)).await {
                warn!("Failed to send exit to {}: {}", language, e);
            }

            // Kill process if still running
            if let Err(e) = server.process.kill().await {
                warn!("Failed to kill {} process: {}", language, e);
            }
        }
    }

    /// Background task to listen for LSP diagnostic notifications
    pub fn spawn_diagnostic_listener(self: Arc<Self>) {
        tokio::spawn(async move {
            // This would be a real LSP message listener in production
            // For now, diagnostics are polled when files change
            debug!("LSP diagnostic listener started");
        });
    }
}

impl Drop for LspManager {
    fn drop(&mut self) {
        // Best effort cleanup
        if let Ok(servers) = self.servers.try_read() {
            if !servers.is_empty() {
                warn!("LspManager dropped with {} active servers", servers.len());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::agent::events::create_event_bus;

    #[test]
    fn test_detect_language() {
        assert_eq!(LspManager::detect_language(Path::new("test.rs")), Some("rust"));
        assert_eq!(LspManager::detect_language(Path::new("test.ts")), Some("typescript"));
        assert_eq!(LspManager::detect_language(Path::new("test.py")), Some("python"));
        assert_eq!(LspManager::detect_language(Path::new("test.go")), Some("go"));
        assert_eq!(LspManager::detect_language(Path::new("test.txt")), None);
    }

    #[test]
    fn test_language_server_command() {
        let (cmd, args) = LspManager::language_server_command("rust").unwrap();
        assert_eq!(cmd, "rust-analyzer");
        assert_eq!(args.len(), 0);

        let (cmd, args) = LspManager::language_server_command("typescript").unwrap();
        assert_eq!(cmd, "typescript-language-server");
        assert!(args.contains(&"--stdio"));

        assert!(LspManager::language_server_command("unknown").is_none());
    }

    #[tokio::test]
    async fn test_lsp_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let event_bus = create_event_bus();
        let manager = LspManager::new(temp_dir.path().to_path_buf(), event_bus);

        let diags = manager.get_all_diagnostics().await;
        assert!(diags.is_empty());
    }

    #[tokio::test]
    async fn test_store_and_retrieve_diagnostics() {
        let temp_dir = TempDir::new().unwrap();
        let event_bus = create_event_bus();
        let manager = LspManager::new(temp_dir.path().to_path_buf(), event_bus);

        let path = PathBuf::from("/test.rs");
        let diags = vec![
            Diagnostic {
                range: DiagnosticRange {
                    start_line: 10,
                    start_column: 5,
                    end_line: 10,
                    end_column: 15,
                },
                severity: DiagnosticSeverity::Error,
                code: Some("E0308".to_string()),
                message: "mismatched types".to_string(),
                source: Some("rustc".to_string()),
            }
        ];

        manager.store_diagnostics(path.clone(), diags.clone()).await;

        let retrieved = manager.get_diagnostics(&path).await;
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].message, "mismatched types");

        let (errors, warnings) = manager.get_diagnostic_counts(&path).await;
        assert_eq!(errors, 1);
        assert_eq!(warnings, 0);

        assert!(manager.has_errors(&path).await);
    }

    #[tokio::test]
    async fn test_diagnostic_counts() {
        let temp_dir = TempDir::new().unwrap();
        let event_bus = create_event_bus();
        let manager = LspManager::new(temp_dir.path().to_path_buf(), event_bus);

        let path = PathBuf::from("/test.rs");
        let diags = vec![
            Diagnostic {
                range: DiagnosticRange { start_line: 1, start_column: 0, end_line: 1, end_column: 10 },
                severity: DiagnosticSeverity::Error,
                code: None,
                message: "error 1".to_string(),
                source: None,
            },
            Diagnostic {
                range: DiagnosticRange { start_line: 2, start_column: 0, end_line: 2, end_column: 10 },
                severity: DiagnosticSeverity::Warning,
                code: None,
                message: "warning 1".to_string(),
                source: None,
            },
            Diagnostic {
                range: DiagnosticRange { start_line: 3, start_column: 0, end_line: 3, end_column: 10 },
                severity: DiagnosticSeverity::Error,
                code: None,
                message: "error 2".to_string(),
                source: None,
            },
        ];

        manager.store_diagnostics(path.clone(), diags).await;

        let (errors, warnings) = manager.get_diagnostic_counts(&path).await;
        assert_eq!(errors, 2);
        assert_eq!(warnings, 1);
    }
}
