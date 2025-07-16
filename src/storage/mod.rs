use crate::config::ConfigManager;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Clone)]
pub struct DatabaseManager {
    config: ConfigManager,
}

impl DatabaseManager {
    pub async fn new(config: &ConfigManager) -> Result<Self> {
        // TODO: Initialize SQLite databases
        Ok(Self {
            config: config.clone(),
        })
    }

    pub async fn close(&self) -> Result<()> {
        // TODO: Close database connections
        Ok(())
    }

    /// Get the path to the sessions database
    pub fn get_sessions_db_path(&self) -> &PathBuf {
        &self.config.database.sessions_db
    }

    /// Get the path to the conversations database
    pub fn get_conversations_db_path(&self) -> &PathBuf {
        &self.config.database.conversations_db
    }

    /// Get the path to the knowledge database
    pub fn get_knowledge_db_path(&self) -> &PathBuf {
        &self.config.database.knowledge_db
    }

    /// Get the path to the file index database
    pub fn get_file_index_db_path(&self) -> &PathBuf {
        &self.config.database.file_index_db
    }
}
