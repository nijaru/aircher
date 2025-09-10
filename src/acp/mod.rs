/// Agent Client Protocol integration module
/// 
/// This module provides ACP (Agent Client Protocol) support for Aircher,
/// allowing it to work as a coding agent in editors like Zed, VS Code, and others
/// that support the ACP standard.
/// 
/// The ACP mode runs alongside our existing TUI mode, sharing the same core
/// agent logic but adapting the interface for editor integration.

pub mod agent;
pub mod session;
pub mod tools;

pub use agent::AircherAcpAgent;
pub use session::AcpSessionManager;

use anyhow::Result;
use std::sync::Arc;
use tokio::io::{stdin, stdout};

use crate::auth::AuthManager;
use crate::config::ConfigManager;
use crate::intelligence::IntelligenceEngine;
use crate::agent::conversation::ProjectContext;

/// Entry point for ACP mode
/// Runs Aircher as an Agent Client Protocol agent over stdin/stdout
pub async fn acp_main(
    config: ConfigManager,
    auth_manager: Arc<AuthManager>,
) -> Result<()> {
    // Initialize project context
    let project_context = ProjectContext {
        root_path: std::env::current_dir()?,
        name: "ACP Project".to_string(), // ACP clients provide project context
        language: crate::agent::conversation::ProgrammingLanguage::Mixed,
        framework: None,
        dependencies: Vec::new(),
    };

    // Initialize intelligence engine  
    let intelligence = IntelligenceEngine::new()?;

    // Create ACP agent
    let agent = AircherAcpAgent::new(
        config,
        auth_manager,
        intelligence,
        project_context,
    ).await?;

    // Connect to stdin/stdout for JSON-RPC communication
    let stdin = stdin();
    let stdout = stdout();

    // Run ACP agent (this will block until the connection is closed)
    agent.run(stdin, stdout).await?;

    Ok(())
}