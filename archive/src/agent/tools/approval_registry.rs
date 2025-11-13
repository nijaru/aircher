use super::ToolRegistry;
use crate::agent::approval_modes::PendingChange;
use crate::agent::tools::approved_file_ops::{ApprovedWriteFileTool, ApprovedEditFileTool, ApprovedDeleteFileTool};
use crate::agent::tools::file_ops::{ReadFileTool, ListFilesTool};
use crate::agent::tools::code_analysis::SearchCodeTool;
use crate::agent::tools::system_ops::RunCommandTool;
use crate::agent::tools::web_tools::{WebBrowsingTool, WebSearchTool};
use crate::agent::tools::build_tools::BuildSystemTool;
use crate::agent::tools::lsp_tools;
use crate::agent::tools::git_tools::{SmartCommitTool, CreatePRTool, BranchManagementTool, TestRunnerTool};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

/// Channel for sending pending changes to the UI
pub type ApprovalChannel = Arc<Mutex<Option<mpsc::UnboundedSender<PendingChange>>>>;

/// Create an approval-integrated tool registry
pub fn create_approval_registry(approval_sender: mpsc::UnboundedSender<PendingChange>) -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // Wrap sender in the expected type
    let approval_channel: ApprovalChannel = Arc::new(Mutex::new(Some(approval_sender)));

    // Register read-only tools (no approval needed)
    registry.register(Box::new(ReadFileTool::new()));
    registry.register(Box::new(ListFilesTool::new()));
    registry.register(Box::new(SearchCodeTool::new()));

    // Register APPROVAL-REQUIRED file tools
    registry.register(Box::new(ApprovedWriteFileTool::new(approval_channel.clone())));
    registry.register(Box::new(ApprovedEditFileTool::new(approval_channel.clone())));
    registry.register(Box::new(ApprovedDeleteFileTool::new(approval_channel.clone())));

    // Register system tools (commands need approval via existing system)
    registry.register(Box::new(RunCommandTool::new()));

    // Register web tools
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
    }

    // Register Git tools
    if let Ok(workspace) = std::env::current_dir() {
        registry.register(Box::new(SmartCommitTool::new(workspace.clone())));
        registry.register(Box::new(CreatePRTool::new(workspace.clone())));
        registry.register(Box::new(BranchManagementTool::new(workspace.clone())));
        registry.register(Box::new(TestRunnerTool::new(workspace.clone())));
    }

    registry
}

/// Create default registry (for non-approval modes)
pub fn create_default_registry() -> ToolRegistry {
    ToolRegistry::default()
}

/// Create registry with approval integration for agent
pub fn create_agent_registry_with_approval() -> (ToolRegistry, mpsc::UnboundedReceiver<PendingChange>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let registry = create_approval_registry(tx);
    (registry, rx)
}
