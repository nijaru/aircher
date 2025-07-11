use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app;
mod config;
mod intelligence;
mod providers;
mod storage;
mod ui;
mod utils;

use app::ArcherApp;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aircher=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Aircher - AI Development Terminal");

    // Initialize application
    let mut app = ArcherApp::new().await?;

    // Run the application
    app.run().await?;

    tracing::info!("Aircher terminated successfully");
    Ok(())
}
