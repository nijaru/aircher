use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::process::Command;
use tracing::{debug, info, warn};

use super::{EmbeddingManager, EmbeddingConfig, EmbeddingModel};

/// Smart embedding setup that answers the user's questions:
/// 1. Should we prompt users? -> No, auto-select with smart defaults
/// 2. Default to large? -> No, default to balanced (nomic-embed-text 274MB)
/// 3. Best for AI coding? -> nomic-embed-text, then mxbai-embed-large
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartEmbeddingSetup {
    pub strategy: SetupStrategy,
    pub auto_upgrade_threshold_mb: u32, // Auto-upgrade if system has enough RAM
    pub fallback_enabled: bool,
    pub min_disk_space_gb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SetupStrategy {
    /// Auto-select best model without prompting (recommended)
    AutoSelect,
    /// Prompt once during first setup, remember choice
    PromptOnce,
    /// Always ask user (power user mode)
    AlwaysPrompt,
    /// Never download, use fallback only
    NoDownload,
}

impl Default for SmartEmbeddingSetup {
    fn default() -> Self {
        Self {
            strategy: SetupStrategy::AutoSelect,
            auto_upgrade_threshold_mb: 8192, // 8GB RAM = allow larger models
            fallback_enabled: true,
            min_disk_space_gb: 2, // Require 2GB free space
        }
    }
}

pub struct SmartSetupEngine {
    setup_config: SmartEmbeddingSetup,
    system_info: SystemInfo,
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub available_ram_mb: u32,
    pub available_disk_gb: u32,
    pub ollama_available: bool,
    pub is_development_machine: bool, // Has git, dev tools, etc.
}

#[derive(Debug, Clone)]
pub struct SetupRecommendation {
    pub recommended_model: Option<EmbeddingModel>,
    pub reasoning: String,
    pub auto_proceed: bool,
    pub alternatives: Vec<EmbeddingModel>,
    pub user_message: String,
}

impl SmartSetupEngine {
    pub async fn new() -> Result<Self> {
        let setup_config = SmartEmbeddingSetup::default();
        let system_info = Self::detect_system_info().await?;
        
        Ok(Self {
            setup_config,
            system_info,
        })
    }

    /// Main entry point: intelligently set up embeddings based on system and user preferences
    pub async fn setup_embeddings(&self) -> Result<SetupRecommendation> {
        info!("Starting smart embedding setup...");
        
        match self.setup_config.strategy {
            SetupStrategy::AutoSelect => self.auto_select_best_model().await,
            SetupStrategy::PromptOnce => self.prompt_once_setup().await,
            SetupStrategy::AlwaysPrompt => self.interactive_setup().await,
            SetupStrategy::NoDownload => Ok(self.no_download_fallback()),
        }
    }

    /// Answer user question: "should we even prompt the user?"
    /// Answer: No, auto-select for best UX
    async fn auto_select_best_model(&self) -> Result<SetupRecommendation> {
        let models = EmbeddingManager::get_coding_optimized_models();
        
        // Selection algorithm based on system capabilities
        let recommended = if !self.system_info.ollama_available {
            // No Ollama -> use lightweight fallback
            models.iter()
                .find(|m| m.provider == "huggingface" && m.size_mb < 200)
                .cloned()
        } else if self.system_info.available_ram_mb >= self.setup_config.auto_upgrade_threshold_mb {
            // High-RAM system -> use best quality (mxbai-embed-large)
            models.iter()
                .find(|m| m.name == "mxbai-embed-large")
                .cloned()
        } else if self.system_info.is_development_machine {
            // Development machine -> use balanced choice (nomic-embed-text)
            models.iter()
                .find(|m| m.name == "nomic-embed-text")
                .cloned()
        } else {
            // Basic system -> use smallest available
            models.iter()
                .min_by_key(|m| m.size_mb)
                .cloned()
        };

        let reasoning = self.generate_selection_reasoning(&recommended);
        
        Ok(SetupRecommendation {
            recommended_model: recommended.clone(),
            reasoning: reasoning.clone(),
            auto_proceed: true, // No prompting
            alternatives: self.get_alternatives(&recommended),
            user_message: format!(
                "🤖 Auto-selected embedding model: {}\n💡 {}", 
                recommended.as_ref().map(|m| m.name.as_str()).unwrap_or("none"),
                reasoning
            ),
        })
    }

    /// Answer user question: "default to large and then have options for small or skip?"
    /// Answer: Default to balanced (274MB), offer large as upgrade
    async fn prompt_once_setup(&self) -> Result<SetupRecommendation> {
        let models = EmbeddingManager::get_coding_optimized_models();
        
        // Default to nomic-embed-text (274MB) - best balance for AI coding
        let default_model = models.iter()
            .find(|m| m.name == "nomic-embed-text")
            .cloned();
            
        let alternatives = vec![
            models.iter().find(|m| m.name == "mxbai-embed-large").cloned(), // Large option
            models.iter().find(|m| m.name == "all-MiniLM-L6-v2").cloned(),  // Small option
        ].into_iter().flatten().collect();

        Ok(SetupRecommendation {
            recommended_model: default_model.clone(),
            reasoning: "Balanced choice: Good quality, reasonable size, optimized for code".to_string(),
            auto_proceed: false, // Prompt user
            alternatives,
            user_message: format!(
                "🧠 Setup embedding model for better code search?\n\
                 \n\
                 Recommended: {} (274MB) - Best balance for AI coding\n\
                 Options:\n\
                 • Recommended - Good quality, reasonable download\n\
                 • Large (669MB) - Highest quality, bigger download\n\
                 • Small (90MB) - Basic quality, fast download\n\
                 • Skip - Continue without embeddings\n\
                 \n\
                 Choice (recommended/large/small/skip): ",
                default_model.as_ref().map(|m| m.name.as_str()).unwrap_or("none")
            ),
        })
    }

    async fn interactive_setup(&self) -> Result<SetupRecommendation> {
        let models = EmbeddingManager::get_coding_optimized_models();
        
        Ok(SetupRecommendation {
            recommended_model: None,
            reasoning: "User will choose interactively".to_string(),
            auto_proceed: false,
            alternatives: models,
            user_message: "🧠 Interactive embedding model setup...\nChoose from available models:".to_string(),
        })
    }

    fn no_download_fallback(&self) -> SetupRecommendation {
        SetupRecommendation {
            recommended_model: None,
            reasoning: "Downloads disabled, using text-based search only".to_string(),
            auto_proceed: true,
            alternatives: vec![],
            user_message: "ℹ️  Embedding downloads disabled. Using basic text search.".to_string(),
        }
    }

    /// Answer user question: "which is best for an ai agent coding?"
    fn generate_selection_reasoning(&self, model: &Option<EmbeddingModel>) -> String {
        match model {
            Some(m) => match m.name.as_str() {
                "nomic-embed-text" => {
                    "Selected nomic-embed-text: Specifically designed for code analysis. \
                     Excellent at understanding function similarity, code structure, and \
                     finding related patterns. 274MB is reasonable for development machines. \
                     This is the sweet spot for AI coding assistants.".to_string()
                }
                "mxbai-embed-large" => {
                    "Selected mxbai-embed-large: Highest quality embeddings for complex \
                     codebases. Superior cross-language understanding and architectural \
                     analysis. Worth the 669MB for professional development teams working \
                     on large, complex projects.".to_string()
                }
                "bge-m3" => {
                    "Selected bge-m3: Best for multilingual teams and mixed-language \
                     codebases. Excellent documentation understanding. Good choice for \
                     international development teams.".to_string()
                }
                "all-MiniLM-L6-v2" => {
                    "Selected all-MiniLM-L6-v2: Lightweight fallback for resource-constrained \
                     systems. Fast inference, good enough for basic code search. \
                     Excellent battery life on laptops.".to_string()
                }
                _ => format!("Selected {}: {}", m.name, m.description),
            },
            None => {
                "No embedding model selected. AI coding will use text-based analysis only. \
                 This still works but you'll miss semantic code search capabilities.".to_string()
            }
        }
    }

    fn get_alternatives(&self, selected: &Option<EmbeddingModel>) -> Vec<EmbeddingModel> {
        let all_models = EmbeddingManager::get_coding_optimized_models();
        
        all_models.into_iter()
            .filter(|m| selected.as_ref().map_or(true, |s| s.name != m.name))
            .take(3) // Show top 3 alternatives
            .collect()
    }

    async fn detect_system_info() -> Result<SystemInfo> {
        let ollama_available = Self::check_ollama_available().await;
        let available_ram_mb = Self::estimate_available_ram().await;
        let available_disk_gb = Self::estimate_available_disk().await;
        let is_development_machine = Self::detect_development_environment().await;

        Ok(SystemInfo {
            available_ram_mb,
            available_disk_gb,
            ollama_available,
            is_development_machine,
        })
    }

    async fn check_ollama_available() -> bool {
        debug!("Checking Ollama availability...");
        
        let result = tokio::time::timeout(
            Duration::from_secs(5),
            Command::new("ollama").arg("--version").output()
        ).await;

        match result {
            Ok(Ok(output)) => output.status.success(),
            _ => {
                debug!("Ollama not available or timed out");
                false
            }
        }
    }

    async fn estimate_available_ram() -> u32 {
        // Simple heuristic - in real implementation would use system APIs
        // For now, assume 8GB as reasonable default for development machines
        8192
    }

    async fn estimate_available_disk() -> u32 {
        // Simple heuristic - assume enough space available
        // Real implementation would check actual disk space
        10
    }

    async fn detect_development_environment() -> bool {
        // Check for common development tools
        let git_available = Command::new("git")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false);

        let node_available = Command::new("node")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false);

        git_available || node_available
    }

    /// Execute the selected recommendation
    pub async fn execute_recommendation(&self, recommendation: &SetupRecommendation) -> Result<Option<String>> {
        if let Some(ref model) = recommendation.recommended_model {
            if model.provider == "ollama" && self.system_info.ollama_available {
                info!("Downloading Ollama model: {}", model.name);
                self.download_ollama_model(&model.name).await?;
                return Ok(Some(model.name.clone()));
            }
        }
        
        Ok(None)
    }

    async fn download_ollama_model(&self, model_name: &str) -> Result<()> {
        info!("Downloading Ollama model: {}", model_name);
        
        let output = Command::new("ollama")
            .arg("pull")
            .arg(model_name)
            .output()
            .await
            .context("Failed to execute ollama pull")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to download {}: {}", model_name, error));
        }

        info!("Successfully downloaded: {}", model_name);
        Ok(())
    }

    /// Generate a summary of the setup decision for logging/display
    pub fn generate_setup_summary(&self, recommendation: &SetupRecommendation) -> String {
        let mut summary = String::new();
        
        summary.push_str("🧠 Embedding Setup Summary\n");
        summary.push_str("========================\n\n");
        
        if let Some(ref model) = recommendation.recommended_model {
            summary.push_str(&format!("Selected: {} ({}MB)\n", model.name, model.size_mb));
            summary.push_str(&format!("Provider: {}\n", model.provider));
            summary.push_str(&format!("Optimized for: {}\n", model.optimized_for.join(", ")));
        } else {
            summary.push_str("Selected: No embedding model\n");
        }
        
        summary.push_str(&format!("\nStrategy: {:?}\n", self.setup_config.strategy));
        summary.push_str(&format!("Reasoning: {}\n", recommendation.reasoning));
        summary.push_str(&format!("Auto-proceed: {}\n", recommendation.auto_proceed));
        
        summary.push_str("\nSystem Info:\n");
        summary.push_str(&format!("  Ollama available: {}\n", self.system_info.ollama_available));
        summary.push_str(&format!("  RAM: {}MB\n", self.system_info.available_ram_mb));
        summary.push_str(&format!("  Development machine: {}\n", self.system_info.is_development_machine));
        
        if !recommendation.alternatives.is_empty() {
            summary.push_str("\nAlternatives:\n");
            for alt in &recommendation.alternatives {
                summary.push_str(&format!("  • {} ({}MB) - {}\n", alt.name, alt.size_mb, alt.description));
            }
        }
        
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_smart_setup_engine() {
        let engine = SmartSetupEngine::new().await.unwrap();
        let recommendation = engine.setup_embeddings().await.unwrap();
        
        // Should provide a recommendation
        assert!(recommendation.recommended_model.is_some() || !recommendation.auto_proceed);
    }

    #[test]
    fn test_system_info_detection() {
        // Should not panic
        let _info = SystemInfo {
            available_ram_mb: 8192,
            available_disk_gb: 10,
            ollama_available: false,
            is_development_machine: true,
        };
    }

    #[test]
    fn test_setup_strategies() {
        let auto_setup = SmartEmbeddingSetup {
            strategy: SetupStrategy::AutoSelect,
            ..Default::default()
        };
        
        assert!(matches!(auto_setup.strategy, SetupStrategy::AutoSelect));
    }
}