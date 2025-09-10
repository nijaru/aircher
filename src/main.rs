use anyhow::Result;
use std::{env, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use aircher::cli::CliApp;
use aircher::config::ConfigManager;
use aircher::auth::AuthManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Check for special modes
    let mode = determine_mode(&args);
    
    match mode {
        Mode::Acp => acp_main().await,
        Mode::Tui => tui_main(args).await,
        Mode::Cli => cli_main(args).await,
    }
}

#[derive(Debug)]
enum Mode {
    /// Agent Client Protocol mode (JSON-RPC over stdin/stdout)
    Acp,
    /// Terminal User Interface mode (default, no args)
    Tui,
    /// Command Line Interface mode (specific commands)
    Cli,
}

fn determine_mode(args: &[String]) -> Mode {
    if args.len() > 1 {
        match args[1].as_str() {
            "--acp" => return Mode::Acp,
            _ if args[1].starts_with('-') => return Mode::Cli,
            _ => {} // Fall through to check for CLI commands
        }
    }
    
    // Check if running in TUI mode (no arguments or help/model/search/etc subcommands)
    let is_tui_mode = args.len() == 1 || (args.len() > 1 && !args[1].starts_with('-'));
    
    if is_tui_mode {
        Mode::Tui
    } else {
        Mode::Cli
    }
}

/// Entry point for Agent Client Protocol mode
async fn acp_main() -> Result<()> {
    // Initialize logging for ACP mode (to stderr, won't interfere with JSON-RPC on stdout)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aircher=debug".into())
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr) // Important: use stderr for ACP mode
        )
        .init();

    tracing::info!("Starting Aircher in Agent Client Protocol mode");

    // Initialize configuration and auth
    let config = ConfigManager::new().await?;
    let auth_manager = Arc::new(AuthManager::new(&config).await?);

    // Run ACP agent
    aircher::acp::acp_main(config, auth_manager).await?;

    Ok(())
}

/// Entry point for Terminal User Interface mode  
async fn tui_main(args: Vec<String>) -> Result<()> {
    // For TUI mode, suppress all logging to avoid display corruption
    // Errors will be shown in the TUI interface itself
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aircher=off".into())
        )
        .init();

    // Initialize and run CLI application (which handles TUI mode)
    let mut app = CliApp::new().await?;
    app.run(args).await?;

    Ok(())
}

/// Entry point for Command Line Interface mode
async fn cli_main(args: Vec<String>) -> Result<()> {
    // Normal logging to stderr for CLI commands
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aircher=warn".into())
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize and run CLI application
    let mut app = CliApp::new().await?;
    app.run(args).await?;

    Ok(())
}
