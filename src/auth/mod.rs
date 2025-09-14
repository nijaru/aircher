use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::sync::RwLock;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use crate::config::ConfigManager;

pub mod storage;
pub mod cli;
pub mod testing;
pub mod oauth;

#[derive(Debug)]
pub struct AuthManager {
    storage: RwLock<storage::AuthStorage>,
    event_tx: broadcast::Sender<AuthEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthStatus {
    Authenticated,
    NotConfigured,
    Invalid,
    Expired,
    RateLimited,
    NetworkError,
}

#[derive(Debug, Clone)]
pub struct ProviderAuthInfo {
    pub provider: String,
    pub status: AuthStatus,
    pub masked_key: Option<String>,
    pub last_validated: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
    pub usage_info: Option<ProviderUsageInfo>,
}

#[derive(Debug, Clone)]
pub struct ProviderUsageInfo {
    pub requests_used: Option<u32>,
    pub requests_limit: Option<u32>,
    pub tokens_used: Option<u64>,
    pub tokens_limit: Option<u64>,
    pub cost_used: Option<f64>,
    pub cost_limit: Option<f64>,
    pub reset_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub enum AuthEvent {
    /// A provider's authentication was added or updated
    ProviderAuthenticated { provider: String },
    /// A provider's authentication was removed
    ProviderUnauthenticated { provider: String },
    /// A provider's authentication failed validation
    ProviderAuthFailed { provider: String, error: String },
    /// All authentication was cleared
    AllAuthCleared,
}

impl AuthManager {
    pub fn new() -> Result<Self> {
        let storage = storage::AuthStorage::new()?;
        let (event_tx, _) = broadcast::channel(100); // Buffer up to 100 events
        
        Ok(Self {
            storage: RwLock::new(storage),
            event_tx,
        })
    }

    /// Subscribe to authentication events
    pub fn subscribe_events(&self) -> broadcast::Receiver<AuthEvent> {
        self.event_tx.subscribe()
    }

    /// Broadcast an authentication event
    fn broadcast_event(&self, event: AuthEvent) {
        if let Err(e) = self.event_tx.send(event.clone()) {
            debug!("Failed to broadcast auth event {:?}: {}", event, e);
        } else {
            debug!("Broadcasted auth event: {:?}", event);
        }
    }

    /// Get authentication status for a specific provider
    pub async fn get_provider_status(&self, provider: &str, config: &ConfigManager) -> ProviderAuthInfo {
        debug!("Checking auth status for provider: {}", provider);

        let provider_config = match config.get_provider(provider) {
            Some(config) => config,
            None => {
                return ProviderAuthInfo {
                    provider: provider.to_string(),
                    status: AuthStatus::NotConfigured,
                    masked_key: None,
                    last_validated: None,
                    error_message: Some("Provider not configured".to_string()),
                    usage_info: None,
                };
            }
        };

        // Check if API key is needed
        if provider_config.api_key_env.is_empty() {
            // Empty api_key_env could be local (Ollama) or OAuth (anthropic-pro)
            if provider == "ollama" {
                // Local providers like Ollama need to be checked for availability
                match self.check_ollama_availability().await {
                    Ok(true) => {
                        return ProviderAuthInfo {
                            provider: provider.to_string(),
                            status: AuthStatus::Authenticated,
                            masked_key: None,
                            last_validated: Some(chrono::Utc::now()),
                            error_message: None,
                            usage_info: None,
                        };
                    }
                    Ok(false) => {
                        return ProviderAuthInfo {
                            provider: provider.to_string(),
                            status: AuthStatus::NetworkError,
                            masked_key: None,
                            last_validated: None,
                            error_message: Some("Ollama service not found or not running".to_string()),
                            usage_info: None,
                        };
                    }
                    Err(e) => {
                        return ProviderAuthInfo {
                            provider: provider.to_string(),
                            status: AuthStatus::NetworkError,
                            masked_key: None,
                            last_validated: None,
                            error_message: Some(format!("Failed to check Ollama: {}", e)),
                            usage_info: None,
                        };
                    }
                }
            } else {
                // OAuth providers like anthropic-pro need separate authentication
                if provider == "anthropic-pro" || provider == "anthropic-max" {
                    // Check if we have stored OAuth token
                    if let Ok(Some(token)) = self.get_oauth_token(provider).await {
                        return ProviderAuthInfo {
                            provider: provider.to_string(),
                            status: AuthStatus::Authenticated,
                            masked_key: Some(self.mask_api_key(&token)),
                            last_validated: Some(chrono::Utc::now()),
                            error_message: None,
                            usage_info: None,
                        };
                    } else {
                        return ProviderAuthInfo {
                            provider: provider.to_string(),
                            status: AuthStatus::NotConfigured,
                            masked_key: None,
                            last_validated: None,
                            error_message: Some("OAuth authentication required".to_string()),
                            usage_info: None,
                        };
                    }
                } else {
                    // Unknown OAuth provider
                    return ProviderAuthInfo {
                        provider: provider.to_string(),
                        status: AuthStatus::NotConfigured,
                        masked_key: None,
                        last_validated: None,
                        error_message: Some("OAuth authentication not supported for this provider".to_string()),
                        usage_info: None,
                    };
                }
            }
        }

        // Try to get API key from environment or storage
        let api_key = match self.get_api_key_with_env(provider, &provider_config.api_key_env).await {
            Ok(Some(key)) => key,
            Ok(None) => {
                return ProviderAuthInfo {
                    provider: provider.to_string(),
                    status: AuthStatus::NotConfigured,
                    masked_key: None,
                    last_validated: None,
                    error_message: Some("API key not found".to_string()),
                    usage_info: None,
                };
            }
            Err(e) => {
                return ProviderAuthInfo {
                    provider: provider.to_string(),
                    status: AuthStatus::NetworkError,
                    masked_key: None,
                    last_validated: None,
                    error_message: Some(e.to_string()),
                    usage_info: None,
                };
            }
        };

        let masked_key = Some(self.mask_api_key(&api_key));

        // TODO: Add actual API validation here
        // For now, assume key exists = authenticated
        ProviderAuthInfo {
            provider: provider.to_string(),
            status: AuthStatus::Authenticated,
            masked_key,
            last_validated: Some(chrono::Utc::now()),
            error_message: None,
            usage_info: None,
        }
    }

    /// Get all provider statuses
    pub async fn get_all_provider_statuses(&self, config: &ConfigManager) -> HashMap<String, ProviderAuthInfo> {
        let mut statuses = HashMap::new();
        
        for provider_name in config.providers.keys() {
            let status = self.get_provider_status(provider_name, config).await;
            statuses.insert(provider_name.clone(), status);
        }

        statuses
    }

    /// Test API key validity for a provider using lightweight auth testing
    pub async fn test_provider_auth(&self, provider: &str, config: &ConfigManager) -> Result<ProviderAuthInfo> {
        info!("Testing authentication for provider: {}", provider);

        let mut auth_info = self.get_provider_status(provider, config).await;

        // Get the API key for testing
        if let Ok(api_key) = self.get_api_key(provider).await {
            let auth_tester = testing::AuthTester::new();
            
            match auth_tester.test_api_key(provider, &api_key).await {
                Ok(true) => {
                    auth_info.status = AuthStatus::Authenticated;
                    auth_info.last_validated = Some(chrono::Utc::now());
                    auth_info.error_message = None;
                    info!("✓ Provider {} authentication successful", provider);
                }
                Ok(false) => {
                    auth_info.status = AuthStatus::Invalid;
                    auth_info.error_message = Some("API key validation failed".to_string());
                    warn!("✗ Provider {} API key validation failed", provider);
                    
                    // Broadcast authentication failure event
                    self.broadcast_event(AuthEvent::ProviderAuthFailed { 
                        provider: provider.to_string(),
                        error: "API key validation failed".to_string()
                    });
                }
                Err(e) => {
                    auth_info.status = AuthStatus::NetworkError;
                    auth_info.error_message = Some(e.to_string());
                    warn!("✗ Provider {} network error: {}", provider, e);
                    
                    // Broadcast authentication failure event
                    self.broadcast_event(AuthEvent::ProviderAuthFailed { 
                        provider: provider.to_string(),
                        error: e.to_string()
                    });
                }
            }
        } else {
            // No API key available - this is handled by get_provider_status
            debug!("No API key available for provider: {}", provider);
        }

        Ok(auth_info)
    }

    /// Store API key for a provider
    pub async fn store_api_key(&self, provider: &str, api_key: &str) -> Result<()> {
        info!("Storing API key for provider: {}", provider);
        self.storage.write().unwrap().store_api_key(provider, api_key).await
            .context("Failed to store API key")?;
        
        // Broadcast authentication event
        self.broadcast_event(AuthEvent::ProviderAuthenticated { 
            provider: provider.to_string() 
        });
        
        info!("✓ API key stored for provider: {}", provider);
        Ok(())
    }

    /// Remove API key for a provider  
    pub async fn remove_api_key(&self, provider: &str) -> Result<()> {
        info!("Removing API key for provider: {}", provider);
        self.storage.write().unwrap().remove_api_key(provider).await
            .context("Failed to remove API key")?;
        
        // Broadcast unauthentication event
        self.broadcast_event(AuthEvent::ProviderUnauthenticated { 
            provider: provider.to_string() 
        });
        
        info!("✓ API key removed for provider: {}", provider);
        Ok(())
    }
    
    /// Clear all stored API keys
    pub async fn clear_all(&self) -> Result<()> {
        self.storage.write().unwrap().clear_all().await?;
        
        // Broadcast clear all event
        self.broadcast_event(AuthEvent::AllAuthCleared);
        
        Ok(())
    }
    
    /// Get API key for a provider (public interface)
    pub async fn get_api_key(&self, provider: &str) -> Result<String> {
        let env_var = match provider {
            "claude" | "anthropic" => "ANTHROPIC_API_KEY",
            "gemini" | "google" => "GOOGLE_API_KEY",
            "openai" => "OPENAI_API_KEY",
            "openrouter" => "OPENROUTER_API_KEY",
            "ollama" => return Err(anyhow::anyhow!("Ollama doesn't require an API key")),
            _ => return Err(anyhow::anyhow!("Unknown provider: {}", provider)),
        };
        
        self.get_api_key_with_env(provider, env_var).await?
            .ok_or_else(|| anyhow::anyhow!("No API key found for provider {}", provider))
    }

    /// Get API key for a provider (from storage or environment)
    async fn get_api_key_with_env(&self, provider: &str, env_var: &str) -> Result<Option<String>> {
        // First try environment variable (highest priority)
        if let Ok(key) = env::var(env_var) {
            if !key.is_empty() {
                debug!("Found API key for {} in environment variable {}", provider, env_var);
                return Ok(Some(key));
            }
        }

        // Then try storage - clone to avoid holding lock across await
        let storage_clone = {
            let storage_guard = self.storage.read().unwrap();
            storage_guard.clone()
        };
        
        match storage_clone.get_api_key(provider).await? {
            Some(key) => {
                debug!("Found API key for {} in storage", provider);
                Ok(Some(key))
            }
            None => {
                debug!("No API key found for {} in storage or environment", provider);
                Ok(None)
            }
        }
    }

    /// Create a masked version of an API key for display
    fn mask_api_key(&self, key: &str) -> String {
        if key.len() <= 8 {
            "*".repeat(key.len())
        } else {
            format!("{}...{}", &key[..4], &key[key.len()-4..])
        }
    }

    /// Get a summary of authentication status
    pub async fn get_auth_summary(&self, config: &ConfigManager) -> String {
        let statuses = self.get_all_provider_statuses(config).await;
        let mut summary_lines = Vec::new();

        for (provider, info) in statuses {
            let status_icon = match info.status {
                AuthStatus::Authenticated => "✓",
                AuthStatus::NotConfigured => "○",
                AuthStatus::Invalid => "✗",
                AuthStatus::Expired => "⚠",
                AuthStatus::RateLimited => "⚠",
                AuthStatus::NetworkError => "⚠",
            };

            let status_text = match info.status {
                AuthStatus::Authenticated => "authenticated",
                AuthStatus::NotConfigured => "not configured",
                AuthStatus::Invalid => "invalid",
                AuthStatus::Expired => "expired",
                AuthStatus::RateLimited => "rate limited",
                AuthStatus::NetworkError => "network error",
            };

            let key_info = match info.masked_key {
                Some(key) => format!(" ({})", key),
                None => String::new(),
            };

            summary_lines.push(format!("{} {}: {}{}", status_icon, provider, status_text, key_info));
        }

        if summary_lines.is_empty() {
            "No providers configured".to_string()
        } else {
            summary_lines.join("\n")
        }
    }

    /// Check if Ollama is available by trying to connect to it
    async fn check_ollama_availability(&self) -> Result<bool> {
        // Simple HTTP request to Ollama's version endpoint
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;
            
        // Check OLLAMA_HOST environment variable first, then fallback to localhost
        let base_url = if let Ok(ollama_host) = std::env::var("OLLAMA_HOST") {
            if ollama_host.starts_with("http") {
                ollama_host
            } else {
                format!("http://{}:11434", ollama_host)
            }
        } else {
            "http://localhost:11434".to_string()
        };
        
        let url = format!("{}/api/version", base_url);
        
        match client.get(url).send().await {
            Ok(response) => {
                Ok(response.status().is_success())
            }
            Err(e) => {
                debug!("Ollama health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Store OAuth token for a provider
    pub async fn store_oauth_token(&self, provider: &str, token: &str) -> Result<()> {
        info!("Storing OAuth token for provider: {}", provider);
        // Use the same storage mechanism but with a different key format
        let oauth_key = format!("{}_oauth_token", provider);
        self.storage.write().unwrap().store_api_key(&oauth_key, token).await
            .context("Failed to store OAuth token")?;
        
        // Broadcast authentication event
        self.broadcast_event(AuthEvent::ProviderAuthenticated { 
            provider: provider.to_string() 
        });
        
        info!("✓ OAuth token stored for provider: {}", provider);
        Ok(())
    }

    /// Get OAuth token for a provider
    pub async fn get_oauth_token(&self, provider: &str) -> Result<Option<String>> {
        let oauth_key = format!("{}_oauth_token", provider);
        self.storage.read().unwrap().get_api_key(&oauth_key).await
    }

    /// Remove OAuth token for a provider
    pub async fn remove_oauth_token(&self, provider: &str) -> Result<()> {
        info!("Removing OAuth token for provider: {}", provider);
        let oauth_key = format!("{}_oauth_token", provider);
        self.storage.write().unwrap().remove_api_key(&oauth_key).await
            .context("Failed to remove OAuth token")?;
        
        // Broadcast unauthentication event
        self.broadcast_event(AuthEvent::ProviderUnauthenticated { 
            provider: provider.to_string() 
        });
        
        info!("✓ OAuth token removed for provider: {}", provider);
        Ok(())
    }

    /// Start OAuth flow for a provider
    pub async fn start_oauth_flow(&self, provider: &str) -> Result<String> {
        use self::oauth::OAuthHandler;
        
        match provider {
            "anthropic-pro" | "anthropic-max" => {
                let oauth_handler = OAuthHandler::new_anthropic_pro();
                
                // Check if we're in SSH session
                if OAuthHandler::is_ssh_session() {
                    info!("📋 SSH session detected - manual authentication required");
                    let auth_url = oauth_handler.start_auth_flow().await?;
                    return Ok(auth_url);
                }
                
                // Start the OAuth flow
                let auth_url = oauth_handler.start_auth_flow().await?;
                
                // Note: OAuth callback handling would typically be done here
                // For now, users will need to manually complete the OAuth flow
                
                Ok(auth_url)
            }
            _ => Err(anyhow::anyhow!("OAuth not supported for provider: {}", provider))
        }
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new().expect("Failed to create AuthManager")
    }
}

impl Clone for AuthManager {
    fn clone(&self) -> Self {
        // Create a new AuthManager with the same storage path
        Self::new().expect("Failed to clone AuthManager")
    }
}