use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;

/// XDG Base Directory Specification compliant directory resolution
/// 
/// This module ensures consistent behavior across platforms by always using XDG spec,
/// rather than platform-specific conventions (like macOS ~/Library/Application Support/).
pub struct XdgDirs;

impl XdgDirs {
    /// Get XDG config directory: $XDG_CONFIG_HOME or ~/.config
    /// 
    /// Used for: configuration files, settings, API keys
    /// Example: ~/.config/aircher/
    pub fn config_dir() -> Result<PathBuf> {
        if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
            Ok(PathBuf::from(xdg_config_home))
        } else {
            let home = dirs::home_dir()
                .context("Could not determine home directory")?;
            Ok(home.join(".config"))
        }
    }
    
    /// Get XDG data directory: $XDG_DATA_HOME or ~/.local/share
    /// 
    /// Used for: databases, logs, persistent application data
    /// Example: ~/.local/share/aircher/
    pub fn data_dir() -> Result<PathBuf> {
        if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
            Ok(PathBuf::from(xdg_data_home))
        } else {
            let home = dirs::home_dir()
                .context("Could not determine home directory")?;
            Ok(home.join(".local").join("share"))
        }
    }
    
    /// Get XDG cache directory: $XDG_CACHE_HOME or ~/.cache
    /// 
    /// Used for: temporary files, caches, performance data
    /// Example: ~/.cache/aircher/
    pub fn cache_dir() -> Result<PathBuf> {
        if let Ok(xdg_cache_home) = env::var("XDG_CACHE_HOME") {
            Ok(PathBuf::from(xdg_cache_home))
        } else {
            let home = dirs::home_dir()
                .context("Could not determine home directory")?;
            Ok(home.join(".cache"))
        }
    }
    
    /// Get aircher config directory with app subdirectory
    /// 
    /// Returns: ~/.config/aircher/
    pub fn aircher_config_dir() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("aircher"))
    }
    
    /// Get aircher data directory with app subdirectory
    /// 
    /// Returns: ~/.local/share/aircher/
    pub fn aircher_data_dir() -> Result<PathBuf> {
        Ok(Self::data_dir()?.join("aircher"))
    }
    
    /// Get aircher cache directory with app subdirectory
    /// 
    /// Returns: ~/.cache/aircher/
    pub fn aircher_cache_dir() -> Result<PathBuf> {
        Ok(Self::cache_dir()?.join("aircher"))
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
            AircherFileType::Config => Self::aircher_config_dir()?,
            AircherFileType::Data => Self::aircher_data_dir()?,
            AircherFileType::Cache => Self::aircher_cache_dir()?,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_xdg_config_dir() {
        // Test with XDG_CONFIG_HOME set
        env::set_var("XDG_CONFIG_HOME", "/tmp/test-config");
        let config_dir = XdgDirs::config_dir().unwrap();
        assert_eq!(config_dir, PathBuf::from("/tmp/test-config"));
        
        // Test with XDG_CONFIG_HOME unset (fallback to ~/.config)
        env::remove_var("XDG_CONFIG_HOME");
        let config_dir = XdgDirs::config_dir().unwrap();
        let expected = dirs::home_dir().unwrap().join(".config");
        assert_eq!(config_dir, expected);
    }
    
    #[test]
    fn test_aircher_directories() {
        let config_dir = XdgDirs::aircher_config_dir().unwrap();
        let data_dir = XdgDirs::aircher_data_dir().unwrap();
        let cache_dir = XdgDirs::aircher_cache_dir().unwrap();
        
        // All should end with 'aircher'
        assert!(config_dir.ends_with("aircher"));
        assert!(data_dir.ends_with("aircher"));
        assert!(cache_dir.ends_with("aircher"));
        
        // Should be in appropriate parent directories
        assert!(config_dir.parent().unwrap().ends_with(".config"));
        assert!(data_dir.parent().unwrap().ends_with("share"));
        assert!(cache_dir.parent().unwrap().ends_with(".cache"));
    }
    
    #[test]
    fn test_file_path_generation() {
        let auth_file = XdgDirs::get_file_path(AircherFileType::Config, "auth.json").unwrap();
        let db_file = XdgDirs::get_file_path(AircherFileType::Data, "sessions.db").unwrap();
        let cache_file = XdgDirs::get_file_path(AircherFileType::Cache, "index.cache").unwrap();
        
        assert!(auth_file.ends_with("aircher/auth.json"));
        assert!(db_file.ends_with("aircher/sessions.db"));
        assert!(cache_file.ends_with("aircher/index.cache"));
    }
}