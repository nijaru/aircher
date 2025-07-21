use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::config::ConfigManager;
use crate::intelligence::IntelligenceEngine;
use crate::providers::ProviderManager;
use crate::storage::DatabaseManager;
use crate::ui::TuiManager;

pub struct ArcherApp {
    #[allow(dead_code)]
    config: ConfigManager,
    providers: ProviderManager,
    ui: TuiManager,
    storage: DatabaseManager,
    #[allow(dead_code)]
    intelligence: IntelligenceEngine,
    running: Arc<RwLock<bool>>,
}

impl ArcherApp {
    pub async fn new() -> Result<Self> {
        info!("Initializing Aircher application");

        // Load configuration
        let config = ConfigManager::load().await?;
        info!("Configuration loaded");

        // Initialize database storage
        let storage = DatabaseManager::new(&config).await?;
        info!("Database initialized");

        // Initialize provider manager
        let providers = ProviderManager::new(&config).await?;
        info!("Provider manager initialized");

        // Initialize intelligence engine
        let intelligence = IntelligenceEngine::new(&config, &storage).await?;
        info!("Intelligence engine initialized");

        // Initialize TUI
        let ui = TuiManager::new(&config, &providers).await?;
        info!("TUI initialized");

        Ok(Self {
            config,
            providers,
            ui,
            storage,
            intelligence,
            running: Arc::new(RwLock::new(true)),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting main application loop");

        // Start the TUI
        self.ui.run(&self.providers).await?;

        info!("Application loop completed");
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Aircher");

        // Set running flag to false
        *self.running.write().await = false;

        // Close database connections
        self.storage.close().await?;

        info!("Shutdown complete");
        Ok(())
    }
}
