use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::fs;
use dirs;

use crate::utils::xdg_dirs::{XdgDirs, AircherFileType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsConfig {
    /// Commands that have been approved to run without asking
    #[serde(default)]
    pub approved_commands: HashSet<String>,
    
    /// Patterns for commands that are pre-approved (e.g., "npm test", "cargo build")
    #[serde(default)]
    pub approved_patterns: Vec<String>,
    
    /// Directories where file operations are allowed without asking
    #[serde(default)]
    pub allowed_directories: Vec<PathBuf>,
    
    /// File extensions that can be read/written without asking
    #[serde(default)]
    pub allowed_extensions: Vec<String>,
}

impl Default for PermissionsConfig {
    fn default() -> Self {
        Self {
            approved_commands: HashSet::new(),
            approved_patterns: vec![
                "ls".to_string(),
                "pwd".to_string(),
                "echo".to_string(),
                "cat".to_string(),
                "grep".to_string(),
                "find".to_string(),
            ],
            allowed_directories: vec![PathBuf::from(".")],
            allowed_extensions: vec![
                "rs".to_string(),
                "toml".to_string(),
                "json".to_string(),
                "md".to_string(),
                "txt".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
            ],
        }
    }
}

#[derive(Clone)]
pub struct PermissionsManager {
    config: PermissionsConfig,
    config_path: PathBuf,
}

impl PermissionsManager {
    pub fn new() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        let config_path = config_dir.join("permissions.toml");
        
        // Load existing config or create default
        let config = if config_path.exists() {
            let contents = fs::read_to_string(&config_path)?;
            toml::from_str(&contents)?
        } else {
            // Create config directory if it doesn't exist
            fs::create_dir_all(&config_dir)?;
            let default_config = PermissionsConfig::default();
            let toml_str = toml::to_string_pretty(&default_config)?;
            fs::write(&config_path, toml_str)?;
            default_config
        };
        
        Ok(Self {
            config,
            config_path,
        })
    }
    
    /// Get the config directory for permissions - local project or XDG global
    fn get_config_dir() -> Result<PathBuf> {
        // First try local .aircher directory (project-specific)
        let current_dir = std::env::current_dir()?;
        let local_aircher = current_dir.join(".aircher");
        
        if local_aircher.exists() && local_aircher.is_dir() {
            return Ok(local_aircher);
        }
        
        // Fall back to XDG config directory
        XdgDirs::aircher_config_dir()
    }
    
    /// Check if a command is approved
    pub fn is_command_approved(&self, command: &str) -> bool {
        // Check exact matches
        if self.config.approved_commands.contains(command) {
            return true;
        }
        
        // Check patterns
        for pattern in &self.config.approved_patterns {
            if command.starts_with(pattern) {
                return true;
            }
        }
        
        false
    }
    
    /// Add an approved command
    pub fn approve_command(&mut self, command: String) -> Result<()> {
        self.config.approved_commands.insert(command);
        self.save_config()
    }
    
    /// Add a command pattern for future approval
    pub fn approve_pattern(&mut self, pattern: String) -> Result<()> {
        if !self.config.approved_patterns.contains(&pattern) {
            self.config.approved_patterns.push(pattern);
            self.save_config()?;
        }
        Ok(())
    }
    
    /// Check if a file path is allowed for operations
    pub fn is_path_allowed(&self, path: &Path) -> bool {
        // Check if path is under an allowed directory
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .unwrap_or_default()
                .join(path)
        };
        
        // Always allow current directory and subdirectories
        if let Ok(current) = std::env::current_dir() {
            if abs_path.starts_with(&current) {
                return true;
            }
        }
        
        // Check allowed directories
        for allowed_dir in &self.config.allowed_directories {
            let allowed_abs = if allowed_dir.is_absolute() {
                allowed_dir.clone()
            } else {
                std::env::current_dir()
                    .unwrap_or_default()
                    .join(allowed_dir)
            };
            
            if abs_path.starts_with(&allowed_abs) {
                return true;
            }
        }
        
        // Check file extension
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return self.config.allowed_extensions.contains(&ext_str.to_string());
            }
        }
        
        false
    }
    
    /// Save the configuration
    fn save_config(&self) -> Result<()> {
        let toml_str = toml::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, toml_str)?;
        Ok(())
    }
    
    /// Get a pattern from a command (e.g., "npm test" -> "npm")
    pub fn get_command_pattern(command: &str) -> String {
        command.split_whitespace()
            .next()
            .unwrap_or(command)
            .to_string()
    }
}