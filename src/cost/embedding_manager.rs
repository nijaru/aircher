use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;
use tracing::{debug, info, warn};

use super::swerank_integration::{SweRankEmbedModel, ModelInfo as SweRankModelInfo};

/// Embedding models optimized for AI agent coding tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModel {
    pub name: String,
    pub provider: String,
    pub size_mb: u32,
    pub description: String,
    pub optimized_for: Vec<String>,
    pub download_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub preferred_model: String,
    pub fallback_model: Option<String>,
    pub auto_download: bool,
    pub use_ollama_if_available: bool,
    pub max_model_size_mb: u32,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            preferred_model: "nomic-embed-text".to_string(), // Best for code
            fallback_model: Some("all-MiniLM-L6-v2".to_string()), // Smaller fallback
            auto_download: true,
            use_ollama_if_available: true,
            max_model_size_mb: 1000, // 1GB limit
        }
    }
}

pub struct EmbeddingManager {
    config: EmbeddingConfig,
    available_models: Vec<EmbeddingModel>,
    ollama_available: Option<bool>,
    swerank_model: Option<SweRankEmbedModel>,
}

impl EmbeddingManager {
    pub fn new(config: EmbeddingConfig) -> Self {
        let available_models = Self::get_coding_optimized_models();
        
        Self {
            config,
            available_models,
            ollama_available: None,
            swerank_model: None,
        }
    }

    /// Get models specifically optimized for AI agent coding tasks
    pub fn get_coding_optimized_models() -> Vec<EmbeddingModel> {
        vec![
            EmbeddingModel {
                name: "swerank-embed-small".to_string(),
                provider: "embedded".to_string(),
                size_mb: 137,
                description: "SOTA code embeddings (74.45% on SWE-Bench-Lite) - zero dependencies".to_string(),
                optimized_for: vec![
                    "issue_localization".to_string(),
                    "code_understanding".to_string(),
                    "semantic_search".to_string(),
                    "zero_dependency".to_string(),
                ],
                download_url: Some("sentence-transformers/all-MiniLM-L6-v2".to_string()), // Placeholder URL
            },
            EmbeddingModel {
                name: "nomic-embed-text".to_string(),
                provider: "ollama".to_string(),
                size_mb: 274,
                description: "Code-optimized, excellent for semantic code search".to_string(),
                optimized_for: vec![
                    "code_search".to_string(),
                    "function_similarity".to_string(),
                    "documentation".to_string(),
                ],
                download_url: None, // Ollama handles this
            },
            EmbeddingModel {
                name: "mxbai-embed-large".to_string(),
                provider: "ollama".to_string(),
                size_mb: 669,
                description: "High-quality embeddings, best accuracy for complex code".to_string(),
                optimized_for: vec![
                    "complex_reasoning".to_string(),
                    "architecture_analysis".to_string(),
                    "cross_language".to_string(),
                ],
                download_url: None,
            },
            EmbeddingModel {
                name: "bge-m3".to_string(),
                provider: "ollama".to_string(),
                size_mb: 1200,
                description: "Multilingual, excellent for mixed-language codebases".to_string(),
                optimized_for: vec![
                    "multilingual_code".to_string(),
                    "documentation".to_string(),
                    "comments".to_string(),
                ],
                download_url: None,
            },
            EmbeddingModel {
                name: "all-MiniLM-L6-v2".to_string(),
                provider: "huggingface".to_string(),
                size_mb: 90,
                description: "Lightweight, fast, good enough for basic code search".to_string(),
                optimized_for: vec![
                    "quick_search".to_string(),
                    "basic_similarity".to_string(),
                ],
                download_url: Some("sentence-transformers/all-MiniLM-L6-v2".to_string()),
            },
        ]
    }

    /// Check if Ollama is available and running
    pub async fn check_ollama_availability(&mut self) -> bool {
        if let Some(available) = self.ollama_available {
            return available;
        }

        debug!("Checking Ollama availability...");
        
        let output = Command::new("ollama")
            .arg("list")
            .output()
            .await;

        let available = match output {
            Ok(output) => {
                if output.status.success() {
                    info!("Ollama is available");
                    true
                } else {
                    debug!("Ollama command failed: {}", String::from_utf8_lossy(&output.stderr));
                    false
                }
            }
            Err(e) => {
                debug!("Ollama not found: {}", e);
                false
            }
        };

        self.ollama_available = Some(available);
        available
    }

    /// Get the best embedding model for AI coding tasks
    pub async fn get_recommended_model(&mut self) -> Result<EmbeddingModel> {
        // First priority: Embedded SweRankEmbed model (zero dependencies)
        if self.is_embedded_model_available().await {
            if let Some(model) = self.available_models.iter()
                .find(|m| m.name == "swerank-embed-small" && m.provider == "embedded") {
                info!("Using embedded SweRankEmbed-Small model (SOTA, zero dependencies)");
                return Ok(model.clone());
            }
        }

        // Second priority: If Ollama is available and configured, prefer Ollama models
        if self.config.use_ollama_if_available && self.check_ollama_availability().await {
            // Check if preferred model is available
            if let Some(model) = self.available_models.iter()
                .find(|m| m.name == self.config.preferred_model && m.provider == "ollama") {
                
                if self.is_ollama_model_available(&model.name).await? {
                    info!("Using available Ollama model: {}", model.name);
                    return Ok(model.clone());
                }
            }

            // Try to auto-download the preferred model
            if self.config.auto_download {
                if let Some(model) = self.available_models.iter()
                    .find(|m| m.name == self.config.preferred_model && m.provider == "ollama") {
                    
                    if model.size_mb <= self.config.max_model_size_mb {
                        info!("Auto-downloading recommended model: {}", model.name);
                        self.download_ollama_model(&model.name).await?;
                        return Ok(model.clone());
                    }
                }
            }

            // Find the best available model in Ollama
            for model in &self.available_models {
                if model.provider == "ollama" && model.size_mb <= self.config.max_model_size_mb {
                    if self.is_ollama_model_available(&model.name).await? {
                        info!("Using available Ollama model: {}", model.name);
                        return Ok(model.clone());
                    }
                }
            }
        }

        // Fallback to non-Ollama models or prompt for download
        if let Some(fallback_name) = &self.config.fallback_model {
            if let Some(model) = self.available_models.iter()
                .find(|m| m.name == *fallback_name) {
                
                warn!("Using fallback embedding model: {}", model.name);
                return Ok(model.clone());
            }
        }

        // Default to the smallest available model
        let smallest = self.available_models.iter()
            .filter(|m| m.size_mb <= self.config.max_model_size_mb)
            .min_by_key(|m| m.size_mb)
            .context("No suitable embedding model found")?;

        Ok(smallest.clone())
    }

    /// Check if an Ollama model is already downloaded
    async fn is_ollama_model_available(&self, model_name: &str) -> Result<bool> {
        let output = Command::new("ollama")
            .arg("list")
            .output()
            .await
            .context("Failed to run ollama list")?;

        if !output.status.success() {
            return Ok(false);
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(output_str.contains(model_name))
    }

    /// Download an Ollama model
    async fn download_ollama_model(&self, model_name: &str) -> Result<()> {
        info!("Downloading Ollama model: {}", model_name);
        
        let output = Command::new("ollama")
            .arg("pull")
            .arg(model_name)
            .output()
            .await
            .context("Failed to download Ollama model")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to download {}: {}", model_name, error));
        }

        info!("Successfully downloaded: {}", model_name);
        Ok(())
    }

    /// Generate user-friendly model selection prompt
    pub async fn generate_selection_prompt(&mut self) -> String {
        let mut prompt = String::new();
        
        prompt.push_str("🧠 Embedding Model Selection for AI Code Agent\n\n");
        
        if self.check_ollama_availability().await {
            prompt.push_str("✅ Ollama detected - recommended for best performance\n\n");
            
            prompt.push_str("📋 Available models for code analysis:\n");
            for (i, model) in self.available_models.iter()
                .filter(|m| m.provider == "ollama")
                .enumerate() {
                
                let available = self.is_ollama_model_available(&model.name).await
                    .unwrap_or(false);
                
                let status = if available { "✅ Installed" } else { "⬇️  Download" };
                let size = if model.size_mb < 1000 {
                    format!("{}MB", model.size_mb)
                } else {
                    format!("{:.1}GB", model.size_mb as f32 / 1000.0)
                };
                
                prompt.push_str(&format!(
                    "  {}. {} - {} ({}) {}\n     {}\n",
                    i + 1, 
                    model.name, 
                    model.description,
                    size,
                    status,
                    model.optimized_for.join(", ")
                ));
            }
            
            prompt.push_str("\n💡 Recommendation for AI coding:\n");
            prompt.push_str("  Default: nomic-embed-text (274MB) - best balance of size/performance\n");
            prompt.push_str("  Power:   mxbai-embed-large (669MB) - highest quality\n");
            prompt.push_str("  Light:   Skip embedding features for now\n");
            
        } else {
            prompt.push_str("⚠️  Ollama not detected - limited embedding options\n\n");
            prompt.push_str("To get the best experience:\n");
            prompt.push_str("1. Install Ollama: https://ollama.ai\n");
            prompt.push_str("2. Or continue with basic embedding support\n");
        }
        
        prompt
    }

    /// Interactive model selection
    pub async fn prompt_for_model_selection(&mut self) -> Result<Option<EmbeddingModel>> {
        println!("{}", self.generate_selection_prompt().await);
        
        if self.check_ollama_availability().await {
            println!("Choose an option:");
            println!("1. Default (nomic-embed-text) - Recommended");
            println!("2. Large (mxbai-embed-large) - Best quality");
            println!("3. Skip embedding features");
            println!("4. Auto-select based on system");
            
            // In a real implementation, this would read from stdin
            // For now, return the recommended default
            let recommended = self.get_recommended_model().await?;
            return Ok(Some(recommended));
        } else {
            println!("Continue without advanced embedding features? (y/n)");
            // For now, return None (skip embeddings)
            return Ok(None);
        }
    }

    /// Auto-select the best model without prompting
    pub async fn auto_select_model(&mut self) -> Result<Option<EmbeddingModel>> {
        if !self.config.auto_download {
            return Ok(None);
        }

        if self.check_ollama_availability().await {
            match self.get_recommended_model().await {
                Ok(model) => {
                    info!("Auto-selected embedding model: {} ({}MB)", model.name, model.size_mb);
                    Ok(Some(model))
                }
                Err(e) => {
                    warn!("Failed to auto-select embedding model: {}", e);
                    Ok(None)
                }
            }
        } else {
            debug!("Ollama not available, skipping embedding features");
            Ok(None)
        }
    }

    /// Get embedding model status summary
    pub async fn get_status_summary(&mut self) -> String {
        let mut summary = String::new();
        
        summary.push_str("🧠 Embedding Models Status:\n");
        
        if self.check_ollama_availability().await {
            summary.push_str("✅ Ollama: Available\n");
            
            for model in &self.available_models {
                if model.provider == "ollama" {
                    let available = self.is_ollama_model_available(&model.name).await
                        .unwrap_or(false);
                    
                    let status = if available { "✅" } else { "❌" };
                    summary.push_str(&format!("  {} {}: {}\n", status, model.name, model.description));
                }
            }
        } else {
            summary.push_str("❌ Ollama: Not available\n");
            summary.push_str("   Install Ollama for advanced code embeddings\n");
        }
        
        summary.push_str(&format!("\nCurrent config: {}\n", self.config.preferred_model));
        summary
    }

    /// Initialize SweRankEmbed model if needed
    async fn ensure_swerank_model(&mut self) -> Result<()> {
        if self.swerank_model.is_none() {
            info!("🚀 Initializing SweRankEmbed-Small model...");
            let model = SweRankEmbedModel::new().await?;
            self.swerank_model = Some(model);
            info!("✅ SweRankEmbed-Small model initialized");
        }
        Ok(())
    }

    /// Check if embedded model is available
    pub async fn is_embedded_model_available(&self) -> bool {
        SweRankEmbedModel::is_available().await
    }

    /// Generate embeddings for the given text using the best available method
    pub async fn generate_embeddings(&mut self, text: &str) -> Result<Vec<f32>> {
        // Fast path: Try bundled model first to avoid Ollama timeouts
        if !self.config.use_ollama_if_available {
            match self.ensure_swerank_model().await {
                Ok(_) => {
                    let model = self.swerank_model.as_ref()
                        .context("SweRankEmbed model not initialized")?;
                    return model.generate_embeddings(text).await;
                },
                Err(e) => {
                    warn!("Bundled model failed, trying Ollama: {}", e);
                }
            }
        }
        
        let model = self.get_recommended_model().await?;
        self.generate_embeddings_with_model(text, &model.name).await
    }

    /// Generate embeddings using a specific model
    pub async fn generate_embeddings_with_model(&mut self, text: &str, model_name: &str) -> Result<Vec<f32>> {
        // Always try embedded SweRankEmbed model first for fast search
        if model_name == "swerank-embed-small" || !self.config.use_ollama_if_available {
            match self.ensure_swerank_model().await {
                Ok(_) => {
                    let model = self.swerank_model.as_ref()
                        .context("SweRankEmbed model not initialized")?;
                    return model.generate_embeddings(text).await;
                },
                Err(e) => {
                    warn!("SweRankEmbed not available, falling back: {}", e);
                }
            }
        }

        // Otherwise, use Ollama
        use serde_json::json;
        
        if !self.check_ollama_availability().await {
            anyhow::bail!("Ollama not available");
        }

        let client = reqwest::Client::new();
        let request_body = json!({
            "model": model_name,
            "prompt": text
        });

        let response = client
            .post("http://localhost:11434/api/embeddings")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama API error {}: {}", status, error_text);
        }

        let response_data: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        let embeddings: Vec<f32> = response_data["embedding"]
            .as_array()
            .context("No embedding field in response")?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        debug!("Generated {} dimensional embedding for text of {} chars", 
               embeddings.len(), text.len());

        Ok(embeddings)
    }

    /// Generate embeddings for multiple texts efficiently
    pub async fn generate_batch_embeddings(&mut self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let model = self.get_recommended_model().await?;
        
        info!("Generating embeddings for {} texts using {}", texts.len(), model.name);
        
        // Use SweRankEmbed's batch processing if available
        if model.name == "swerank-embed-small" {
            self.ensure_swerank_model().await?;
            let swerank_model = self.swerank_model.as_ref()
                .context("SweRankEmbed model not initialized")?;
            
            let texts_owned: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
            return swerank_model.generate_batch_embeddings(&texts_owned).await;
        }
        
        // Otherwise, process sequentially with Ollama
        let mut results = Vec::with_capacity(texts.len());
        for (i, text) in texts.iter().enumerate() {
            let embedding = self.generate_embeddings_with_model(text, &model.name).await?;
            results.push(embedding);
            
            if i % 10 == 0 && i > 0 {
                debug!("Processed {}/{} texts", i, texts.len());
            }
        }
        
        info!("Completed batch embedding generation");
        Ok(results)
    }

    /// Calculate cosine similarity between two embeddings
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        dot_product / (norm_a * norm_b)
    }

    /// Find the most similar texts to a query
    pub async fn find_similar_texts(&mut self, query: &str, candidates: &[&str], top_k: usize) -> Result<Vec<(usize, f32)>> {
        let query_embedding = self.generate_embeddings(query).await?;
        let candidate_embeddings = self.generate_batch_embeddings(candidates).await?;
        
        let mut similarities: Vec<(usize, f32)> = candidate_embeddings
            .iter()
            .enumerate()
            .map(|(idx, embedding)| {
                let similarity = Self::cosine_similarity(&query_embedding, embedding);
                (idx, similarity)
            })
            .collect();
        
        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return top k results
        similarities.truncate(top_k);
        Ok(similarities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_models() {
        let models = EmbeddingManager::get_coding_optimized_models();
        assert!(!models.is_empty());
        
        // Should have nomic-embed-text as a code-optimized model
        let nomic = models.iter().find(|m| m.name == "nomic-embed-text");
        assert!(nomic.is_some());
        assert!(nomic.unwrap().optimized_for.contains(&"code_search".to_string()));
    }

    #[test]
    fn test_config_defaults() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.preferred_model, "nomic-embed-text");
        assert!(config.auto_download);
        assert!(config.use_ollama_if_available);
    }

    #[tokio::test]
    async fn test_embedding_manager() {
        let config = EmbeddingConfig::default();
        let mut manager = EmbeddingManager::new(config);
        
        // Should work even if Ollama is not available
        let status = manager.get_status_summary().await;
        assert!(status.contains("Embedding Models Status"));
    }
}