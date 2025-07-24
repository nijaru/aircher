use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, warn};

use crate::utils::aircher_dirs::AircherDirs;

/// Simple file-based auth storage
/// Future: Can be enhanced with keychain/keyring integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStorage {
    auth_file: PathBuf,
    keys: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthFile {
    version: u32,
    keys: HashMap<String, String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl AuthStorage {
    pub fn new() -> Result<Self> {
        let auth_file = Self::get_auth_file_path()?;
        let keys = Self::load_keys_from_file(&auth_file)?;

        Ok(Self {
            auth_file,
            keys,
        })
    }

    /// Store API key for a provider
    pub async fn store_api_key(&mut self, provider: &str, api_key: &str) -> Result<()> {
        debug!("Storing API key for provider: {}", provider);
        
        // Simple encryption/obfuscation (not cryptographically secure)
        let obfuscated_key = Self::obfuscate_key(api_key);
        
        self.keys.insert(provider.to_string(), obfuscated_key);
        self.save_to_file()?;
        
        Ok(())
    }

    /// Get API key for a provider
    pub async fn get_api_key(&self, provider: &str) -> Result<Option<String>> {
        match self.keys.get(provider) {
            Some(obfuscated_key) => {
                let key = Self::deobfuscate_key(obfuscated_key);
                Ok(Some(key))
            }
            None => Ok(None),
        }
    }

    /// Remove API key for a provider
    pub async fn remove_api_key(&mut self, provider: &str) -> Result<()> {
        debug!("Removing API key for provider: {}", provider);
        
        self.keys.remove(provider);
        self.save_to_file()?;
        
        Ok(())
    }

    /// List all providers with stored keys
    pub fn list_providers(&self) -> Vec<String> {
        self.keys.keys().cloned().collect()
    }

    /// Get the auth file path
    fn get_auth_file_path() -> Result<PathBuf> {
        AircherDirs::auth_path()
    }

    /// Load keys from auth file
    fn load_keys_from_file(auth_file: &PathBuf) -> Result<HashMap<String, String>> {
        if !auth_file.exists() {
            debug!("Auth file does not exist, starting with empty keys");
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(auth_file)
            .context("Failed to read auth file")?;

        if content.trim().is_empty() {
            debug!("Auth file is empty, starting with empty keys");
            return Ok(HashMap::new());
        }

        let auth_data: AuthFile = serde_json::from_str(&content)
            .context("Failed to parse auth file")?;

        debug!("Loaded {} API keys from auth file", auth_data.keys.len());
        Ok(auth_data.keys)
    }

    /// Save keys to auth file
    fn save_to_file(&self) -> Result<()> {
        let auth_data = AuthFile {
            version: 1,
            keys: self.keys.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let content = serde_json::to_string_pretty(&auth_data)
            .context("Failed to serialize auth data")?;

        // Ensure parent directory exists
        if let Some(parent) = self.auth_file.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create auth file directory")?;
        }

        fs::write(&self.auth_file, content)
            .context("Failed to write auth file")?;

        debug!("Saved {} API keys to auth file", self.keys.len());
        Ok(())
    }

    /// Simple key obfuscation (NOT cryptographically secure!)
    /// This is just to prevent casual viewing of keys in the file
    /// For production use, we should integrate with OS keychain
    fn obfuscate_key(key: &str) -> String {
        let mut result = String::new();
        for (i, byte) in key.bytes().enumerate() {
            let obfuscated = byte ^ ((i % 256) as u8) ^ 0xAA;
            result.push_str(&format!("{:02x}", obfuscated));
        }
        result
    }

    /// Deobfuscate a key
    fn deobfuscate_key(obfuscated: &str) -> String {
        let mut result = Vec::new();
        let mut chars = obfuscated.chars();
        let mut i = 0;
        
        while let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
            if let (Some(d1), Some(d2)) = (c1.to_digit(16), c2.to_digit(16)) {
                let byte = (d1 * 16 + d2) as u8;
                let deobfuscated = byte ^ ((i % 256) as u8) ^ 0xAA;
                result.push(deobfuscated);
                i += 1;
            }
        }
        
        String::from_utf8(result).unwrap_or_default()
    }

    /// Clear all stored keys (for testing/reset)
    pub async fn clear_all(&mut self) -> Result<()> {
        warn!("Clearing all stored API keys");
        self.keys.clear();
        self.save_to_file()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_obfuscation() {
        let original_key = "sk-1234567890abcdef";
        let obfuscated = AuthStorage::obfuscate_key(original_key);
        let deobfuscated = AuthStorage::deobfuscate_key(&obfuscated);
        
        assert_ne!(original_key, obfuscated);
        assert_eq!(original_key, deobfuscated);
    }

    #[tokio::test]
    async fn test_key_storage() {
        let mut storage = AuthStorage::new().unwrap();
        
        // Clear any existing keys for test
        storage.clear_all().await.unwrap();
        
        // Test storing and retrieving
        storage.store_api_key("test_provider", "test_key_123").await.unwrap();
        
        let retrieved = storage.get_api_key("test_provider").await.unwrap();
        assert_eq!(retrieved, Some("test_key_123".to_string()));
        
        // Test removal
        storage.remove_api_key("test_provider").await.unwrap();
        let after_removal = storage.get_api_key("test_provider").await.unwrap();
        assert_eq!(after_removal, None);
    }
}