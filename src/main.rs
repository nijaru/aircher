use anyhow::Result;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use aircher::cli::CliApp;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging (quieter for CLI usage)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aircher=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Initialize and run CLI application
    let mut app = CliApp::new().await?;
    app.run(args).await?;

    Ok(())
}
