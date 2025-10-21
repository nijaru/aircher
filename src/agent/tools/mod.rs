use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

pub mod file_ops;
pub mod enhanced_read_file;
pub mod enhanced_write_file;
pub mod enhanced_edit_file;
pub mod enhanced_list_files;
pub mod safe_file_ops;
pub mod code_analysis;
pub mod system_ops;
pub mod permission_channel;
pub mod lsp_tools;
pub mod git_tools;
pub mod web_tools;
pub mod build_tools;
pub mod approved_file_ops;
pub mod approval_registry;
pub mod strategy_tools;
pub mod real_analyze_errors;

#[cfg(test)]
mod tests;

pub use file_ops::{ReadFileTool, WriteFileTool, EditFileTool, ListFilesTool};
pub use enhanced_read_file::EnhancedReadFileTool;
pub use enhanced_write_file::EnhancedWriteFileTool;
pub use enhanced_edit_file::EnhancedEditFileTool;
pub use enhanced_list_files::EnhancedListFilesTool;
pub use safe_file_ops::SafeWriteFileTool;
pub use code_analysis::{SearchCodeTool, FindDefinitionTool};
pub use system_ops::RunCommandTool;
pub use web_tools::{WebBrowsingTool, WebSearchTool};
pub use build_tools::BuildSystemTool;
pub use permission_channel::{PermissionRequest, PermissionResponse, PermissionRequestSender, PermissionRequestReceiver, create_permission_channel};

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub success: bool,
    pub result: Value,
    pub error: Option<String>,
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub parameters: Value,
}

#[async_trait]
pub trait AgentTool: Send + Sync {
    /// Unique name for the tool
    fn name(&self) -> &str;
    
    /// Description of what the tool does
    fn description(&self) -> &str;
    
    /// JSON schema for the tool's parameters
    fn parameters_schema(&self) -> Value;
    
    /// Execute the tool with given parameters
    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError>;
}

/// Registry for all available tools
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn AgentTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, tool: Box<dyn AgentTool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }
    
    pub fn get(&self, name: &str) -> Option<&Box<dyn AgentTool>> {
        self.tools.get(name)
    }
    
    pub fn list_tools(&self) -> Vec<ToolInfo> {
        self.tools.values().map(|tool| ToolInfo {
            name: tool.name().to_string(),
            description: tool.description().to_string(),
            parameters: tool.parameters_schema(),
        }).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        
        // Register default file operation tools
        registry.register(Box::new(ReadFileTool::new()));
        // Use SafeWriteFileTool to prevent overwriting critical files
        let workspace = std::env::current_dir().ok();
        registry.register(Box::new(SafeWriteFileTool::new(workspace.clone())));
        registry.register(Box::new(EditFileTool::new()));
        registry.register(Box::new(ListFilesTool::new()));
        
        // Register code analysis tools
        registry.register(Box::new(SearchCodeTool::new()));
        
        // Register system operation tools
        registry.register(Box::new(RunCommandTool::new()));

        // Register web browsing tools
        registry.register(Box::new(WebBrowsingTool::new()));
        registry.register(Box::new(WebSearchTool::new()));

        // Register build system tool
        if let Ok(workspace) = std::env::current_dir() {
            registry.register(Box::new(BuildSystemTool::new(workspace)));
        }
        
        // Register LSP tools if workspace is available
        if let Ok(workspace) = std::env::current_dir() {
            registry.register(Box::new(lsp_tools::CodeCompletionTool::new(workspace.clone())));
            registry.register(Box::new(lsp_tools::HoverTool::new(workspace.clone())));
            registry.register(Box::new(lsp_tools::GoToDefinitionTool::new(workspace.clone())));
            registry.register(Box::new(lsp_tools::FindReferencesTool::new(workspace.clone())));
            registry.register(Box::new(lsp_tools::RenameSymbolTool::new(workspace.clone())));
            registry.register(Box::new(lsp_tools::DiagnosticsTool::new(workspace.clone())));
            registry.register(Box::new(lsp_tools::FormatCodeTool::new(workspace.clone())));
            
            // Register Git workflow tools
            registry.register(Box::new(git_tools::SmartCommitTool::new(workspace.clone())));
            registry.register(Box::new(git_tools::CreatePRTool::new(workspace.clone())));
            registry.register(Box::new(git_tools::BranchManagementTool::new(workspace.clone())));
            registry.register(Box::new(git_tools::TestRunnerTool::new(workspace)));
        }

        // Register strategy support tools (fallback implementations)
        strategy_tools::register_strategy_tools(&mut registry);

        registry
    }
}