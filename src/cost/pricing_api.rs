use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{timeout, Duration};
use tracing::{debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsDevResponse {
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub pricing: Option<ModelPricing>,
    pub context_length: Option<u32>,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input: f64,  // per 1M tokens
    pub output: f64, // per 1M tokens
}

pub struct PricingAPI {
    client: reqwest::Client,
    cache: Option<ModelsDevResponse>,
    cache_timestamp: Option<std::time::Instant>,
    cache_duration: Duration,
}

impl PricingAPI {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: None,
            cache_timestamp: None,
            cache_duration: Duration::from_secs(3600), // 1 hour cache
        }
    }

    /// Fetch live pricing data from models.dev API
    pub async fn fetch_pricing(&mut self) -> Result<ModelsDevResponse> {
        // Check cache first
        if let (Some(ref cache), Some(timestamp)) = (&self.cache, self.cache_timestamp) {
            if timestamp.elapsed() < self.cache_duration {
                debug!("Using cached pricing data");
                return Ok(cache.clone());
            }
        }

        debug!("Fetching fresh pricing data from models.dev");

        // Fetch with timeout
        let response = timeout(
            Duration::from_secs(10),
            self.client.get("https://models.dev/api.json").send()
        ).await??;

        if !response.status().is_success() {
            warn!("Failed to fetch pricing: HTTP {}", response.status());
            // Return cached data if available, even if stale
            if let Some(ref cache) = self.cache {
                return Ok(cache.clone());
            }
            return Err(anyhow::anyhow!("Failed to fetch pricing data"));
        }

        let data: ModelsDevResponse = response.json().await?;
        
        // Update cache
        self.cache = Some(data.clone());
        self.cache_timestamp = Some(std::time::Instant::now());
        
        debug!("Cached {} models from API", data.models.len());
        Ok(data)
    }

    /// Get pricing for a specific model
    pub async fn get_model_pricing(&mut self, provider: &str, model: &str) -> Option<String> {
        let data = self.fetch_pricing().await.ok()?;
        
        // Try exact match first
        for model_info in &data.models {
            if model_info.provider.to_lowercase() == provider.to_lowercase() 
                && (model_info.id == model || model_info.name == model) {
                if let Some(ref pricing) = model_info.pricing {
                    return Some(format!("${:.2}/${:.2}", pricing.input, pricing.output));
                }
            }
        }

        // Try fuzzy match for model name
        for model_info in &data.models {
            if model_info.provider.to_lowercase() == provider.to_lowercase() 
                && (model_info.id.contains(model) || model_info.name.contains(model) || model.contains(&model_info.id)) {
                if let Some(ref pricing) = model_info.pricing {
                    return Some(format!("${:.2}/${:.2}", pricing.input, pricing.output));
                }
            }
        }

        None
    }

    /// Get context window for a model
    pub async fn get_context_window(&mut self, provider: &str, model: &str) -> Option<u32> {
        let data = self.fetch_pricing().await.ok()?;
        
        for model_info in &data.models {
            if model_info.provider.to_lowercase() == provider.to_lowercase() 
                && (model_info.id == model || model_info.name == model || 
                    model_info.id.contains(model) || model.contains(&model_info.id)) {
                return model_info.context_length;
            }
        }
        
        None
    }

    /// Check if a model supports tools/function calling
    pub async fn supports_tools(&mut self, provider: &str, model: &str) -> Option<bool> {
        let data = self.fetch_pricing().await.ok()?;
        
        for model_info in &data.models {
            if model_info.provider.to_lowercase() == provider.to_lowercase() 
                && (model_info.id == model || model_info.name == model || 
                    model_info.id.contains(model) || model.contains(&model_info.id)) {
                // Check capabilities for function calling indicators
                let supports = model_info.capabilities.iter().any(|cap| {
                    cap.to_lowercase().contains("function") || 
                    cap.to_lowercase().contains("tool") ||
                    cap.to_lowercase().contains("call")
                });
                return Some(supports);
            }
        }
        
        None
    }

    /// Get all available models for a provider
    pub async fn get_provider_models(&mut self, provider: &str) -> Vec<ModelInfo> {
        let data = match self.fetch_pricing().await {
            Ok(data) => data,
            Err(_) => return vec![],
        };
        
        data.models.iter()
            .filter(|m| m.provider.to_lowercase() == provider.to_lowercase())
            .cloned()
            .collect()
    }

    /// Generate a pricing summary report
    pub async fn generate_summary(&mut self) -> Result<String> {
        let data = self.fetch_pricing().await?;
        let mut summary = String::new();
        
        summary.push_str("📊 Live Model Pricing (models.dev)\n");
        summary.push_str("=====================================\n\n");
        
        // Group by provider
        let mut by_provider: HashMap<String, Vec<&ModelInfo>> = HashMap::new();
        for model in &data.models {
            by_provider.entry(model.provider.clone()).or_default().push(model);
        }
        
        for (provider, models) in by_provider {
            summary.push_str(&format!("🏢 {}:\n", provider));
            
            for model in models.iter().take(5) { // Show top 5 per provider
                if let Some(ref pricing) = model.pricing {
                    let ctx = model.context_length
                        .map(|c| format!("{}k", c / 1000))
                        .unwrap_or_else(|| "?".to_string());
                    
                    summary.push_str(&format!(
                        "  {} - ${:.2}/${:.2} ({})\n",
                        model.name, pricing.input, pricing.output, ctx
                    ));
                }
            }
            summary.push('\n');
        }
        
        summary.push_str(&format!("Last updated: {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")));
        
        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pricing_api_basic() {
        let mut api = PricingAPI::new();
        
        // This test requires network access
        if let Ok(data) = api.fetch_pricing().await {
            assert!(!data.models.is_empty());
            println!("Found {} models", data.models.len());
        }
    }

    #[tokio::test] 
    async fn test_model_lookup() {
        let mut api = PricingAPI::new();
        
        // Test common models if API is available
        if let Some(pricing) = api.get_model_pricing("openai", "gpt-4o").await {
            println!("GPT-4o pricing: {}", pricing);
            assert!(pricing.contains("$"));
        }
    }
}