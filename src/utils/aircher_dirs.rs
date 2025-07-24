use anyhow::{Context, Result};
use std::path::PathBuf;

/// Aircher directory management using ~/.aircher convention
/// 
/// This replaces the XDG implementation with a simpler, more conventional approach
/// similar to other developer tools (e.g., ~/.ssh, ~/.aws, ~/.cargo)
pub struct AircherDirs;

impl AircherDirs {
    /// Get the base aircher directory: ~/.aircher
    pub fn base_dir() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Could not determine home directory")?;
        Ok(home.join(".aircher"))
    }
    
    /// Get aircher config directory: ~/.aircher/config
    /// Used for additional config files like permissions.toml
    pub fn config_dir() -> Result<PathBuf> {
        Ok(Self::base_dir()?.join("config"))
    }
    
    /// Get main config file path: ~/.aircher/config.toml
    pub fn main_config_path() -> Result<PathBuf> {
        Ok(Self::base_dir()?.join("config.toml"))
    }
    
    /// Get auth file path: ~/.aircher/auth.json  
    pub fn auth_path() -> Result<PathBuf> {
        Ok(Self::base_dir()?.join("auth.json"))
    }
    
    /// Get aircher data directory: ~/.aircher/data
    pub fn data_dir() -> Result<PathBuf> {
        Ok(Self::base_dir()?.join("data"))
    }
    
    /// Get aircher cache directory: ~/.aircher/cache
    pub fn cache_dir() -> Result<PathBuf> {
        Ok(Self::base_dir()?.join("cache"))
    }
    
    /// Get aircher logs directory: ~/.aircher/logs
    pub fn logs_dir() -> Result<PathBuf> {
        Ok(Self::base_dir()?.join("logs"))
    }
    
    /// Ensure a directory exists, creating parent directories as needed
    pub fn ensure_dir_exists(path: &PathBuf) -> Result<()> {
        if !path.exists() {
            std::fs::create_dir_all(path)
                .with_context(|| format!("Failed to create directory: {}", path.display()))?;
        }
        Ok(())
    }
    
    /// Get the recommended location for different types of aircher files
    pub fn get_file_path(file_type: AircherFileType, filename: &str) -> Result<PathBuf> {
        let dir = match file_type {
            AircherFileType::Config => Self::config_dir()?,
            AircherFileType::Data => Self::data_dir()?,
            AircherFileType::Cache => Self::cache_dir()?,
            AircherFileType::Logs => Self::logs_dir()?,
        };
        
        Self::ensure_dir_exists(&dir)?;
        Ok(dir.join(filename))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AircherFileType {
    /// Configuration files (auth.json, config.toml, permissions.toml)
    Config,
    /// Data files (databases, logs, sessions)  
    Data,
    /// Cache files (indexes, temporary data)
    Cache,
    /// Log files
    Logs,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_base_directory() {
        let base_dir = AircherDirs::base_dir().unwrap();
        assert!(base_dir.ends_with(".aircher"));
    }
    
    #[test]
    fn test_subdirectories() {
        let config_dir = AircherDirs::config_dir().unwrap();
        let data_dir = AircherDirs::data_dir().unwrap();
        let cache_dir = AircherDirs::cache_dir().unwrap();
        let logs_dir = AircherDirs::logs_dir().unwrap();
        
        assert!(config_dir.ends_with(".aircher/config"));
        assert!(data_dir.ends_with(".aircher/data"));
        assert!(cache_dir.ends_with(".aircher/cache"));
        assert!(logs_dir.ends_with(".aircher/logs"));
    }
    
    #[test]
    fn test_specific_paths() {
        let config_file = AircherDirs::main_config_path().unwrap();
        let auth_file = AircherDirs::auth_path().unwrap();
        
        assert!(config_file.ends_with(".aircher/config.toml"));
        assert!(auth_file.ends_with(".aircher/auth.json"));
    }
    
    #[test]
    fn test_file_paths() {
        let permissions_file = AircherDirs::get_file_path(AircherFileType::Config, "permissions.toml").unwrap();
        let db_file = AircherDirs::get_file_path(AircherFileType::Data, "sessions.db").unwrap();
        let cache_file = AircherDirs::get_file_path(AircherFileType::Cache, "index.cache").unwrap();
        let log_file = AircherDirs::get_file_path(AircherFileType::Logs, "aircher.log").unwrap();
        
        assert!(permissions_file.ends_with(".aircher/config/permissions.toml"));
        assert!(db_file.ends_with(".aircher/data/sessions.db"));
        assert!(cache_file.ends_with(".aircher/cache/index.cache"));
        assert!(log_file.ends_with(".aircher/logs/aircher.log"));
    }
}