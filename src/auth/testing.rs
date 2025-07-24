use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use tracing::{debug, warn};

/// Lightweight authentication tester that doesn't depend on ProviderManager
/// This breaks the circular dependency between AuthManager and ProviderManager
pub struct AuthTester {
    client: Client,
}

impl AuthTester {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Test authentication for a provider using direct API calls
    pub async fn test_api_key(&self, provider: &str, api_key: &str) -> Result<bool> {
        debug!("Testing API key for provider: {}", provider);

        match provider {
            "claude" | "anthropic" => self.test_claude_auth(api_key).await,
            "openai" => self.test_openai_auth(api_key).await,
            "gemini" | "google" => self.test_gemini_auth(api_key).await,
            "openrouter" => self.test_openrouter_auth(api_key).await,
            "ollama" => Ok(true), // Ollama doesn't require API keys
            _ => {
                warn!("Unknown provider for auth testing: {}", provider);
                Ok(false)
            }
        }
    }

    /// Test Claude/Anthropic API key with a minimal request
    async fn test_claude_auth(&self, api_key: &str) -> Result<bool> {
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&json!({
                "model": "claude-3-haiku-20240307",
                "max_tokens": 1,
                "messages": [{"role": "user", "content": "Hi"}]
            }))
            .send()
            .await
            .context("Failed to send Claude auth test request")?;

        match response.status().as_u16() {
            200..=299 => {
                debug!("Claude authentication successful");
                Ok(true)
            }
            401 | 403 => {
                debug!("Claude authentication failed: invalid API key");
                Ok(false)
            }
            _ => {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                warn!("Claude auth test unexpected response: {} - {}", status, body);
                Ok(false)
            }
        }
    }

    /// Test OpenAI API key with a minimal request
    async fn test_openai_auth(&self, api_key: &str) -> Result<bool> {
        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("content-type", "application/json")
            .json(&json!({
                "model": "gpt-3.5-turbo",
                "messages": [{"role": "user", "content": "Hi"}],
                "max_tokens": 1
            }))
            .send()
            .await
            .context("Failed to send OpenAI auth test request")?;

        match response.status().as_u16() {
            200..=299 => {
                debug!("OpenAI authentication successful");
                Ok(true)
            }
            401 | 403 => {
                debug!("OpenAI authentication failed: invalid API key");
                Ok(false)
            }
            _ => {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                warn!("OpenAI auth test unexpected response: {} - {}", status, body);
                Ok(false)
            }
        }
    }

    /// Test Gemini API key with a minimal request
    async fn test_gemini_auth(&self, api_key: &str) -> Result<bool> {
        let response = self
            .client
            .post(&format!(
                "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key={}",
                api_key
            ))
            .header("content-type", "application/json")
            .json(&json!({
                "contents": [{"parts": [{"text": "Hi"}]}],
                "generationConfig": {"maxOutputTokens": 1}
            }))
            .send()
            .await
            .context("Failed to send Gemini auth test request")?;

        match response.status().as_u16() {
            200..=299 => {
                debug!("Gemini authentication successful");
                Ok(true)
            }
            400 => {
                // Gemini returns 400 for invalid API keys
                let body = response.text().await.unwrap_or_default();
                if body.contains("API_KEY_INVALID") || body.contains("invalid") {
                    debug!("Gemini authentication failed: invalid API key");
                    Ok(false)
                } else {
                    debug!("Gemini authentication successful (400 with valid key)");
                    Ok(true)
                }
            }
            401 | 403 => {
                debug!("Gemini authentication failed: invalid API key");
                Ok(false)
            }
            _ => {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                warn!("Gemini auth test unexpected response: {} - {}", status, body);
                Ok(false)
            }
        }
    }

    /// Test OpenRouter API key with a minimal request
    async fn test_openrouter_auth(&self, api_key: &str) -> Result<bool> {
        let response = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("content-type", "application/json")
            .json(&json!({
                "model": "openai/gpt-3.5-turbo",
                "messages": [{"role": "user", "content": "Hi"}],
                "max_tokens": 1
            }))
            .send()
            .await
            .context("Failed to send OpenRouter auth test request")?;

        match response.status().as_u16() {
            200..=299 => {
                debug!("OpenRouter authentication successful");
                Ok(true)
            }
            401 | 403 => {
                debug!("OpenRouter authentication failed: invalid API key");
                Ok(false)
            }
            _ => {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                warn!("OpenRouter auth test unexpected response: {} - {}", status, body);
                Ok(false)
            }
        }
    }
}

impl Default for AuthTester {
    fn default() -> Self {
        Self::new()
    }
}