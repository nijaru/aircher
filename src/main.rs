use anyhow::Result;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use aircher::cli::CliApp;

#[tokio::main]
async fn main() -> Result<()> {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Check if running in TUI mode (no arguments or help/model/search/etc subcommands)
    let is_tui_mode = args.len() == 1 || (args.len() > 1 && !args[1].starts_with('-'));
    
    // Initialize logging
    if is_tui_mode {
        // For TUI mode, suppress all logging to avoid display corruption
        // Errors will be shown in the TUI interface itself
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "aircher=off".into())
            )
            .init();
    } else {
        // Normal logging to stderr for CLI commands
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "aircher=warn".into())
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    // Initialize and run CLI application
    let mut app = CliApp::new().await?;
    app.run(args).await?;

    Ok(())
}
