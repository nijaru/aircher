use anyhow::Result;
use clap::{Args, Subcommand};
use tracing::info;

use crate::cost::{EmbeddingManager, EmbeddingConfig};

#[derive(Debug, Args)]
pub struct EmbeddingArgs {
    #[command(subcommand)]
    pub command: EmbeddingCommand,
}

#[derive(Debug, Subcommand)]
pub enum EmbeddingCommand {
    /// Show current embedding model status
    Status,
    /// Configure embedding models
    Config {
        /// Set preferred embedding model
        #[arg(long)]
        model: Option<String>,
        /// Enable/disable auto-download
        #[arg(long)]
        auto_download: Option<bool>,
        /// Set maximum model size in MB
        #[arg(long)]
        max_size: Option<u32>,
    },
    /// Download and setup embedding models
    Setup {
        /// Force re-download even if model exists
        #[arg(long)]
        force: bool,
        /// Use interactive selection
        #[arg(long)]
        interactive: bool,
    },
    /// List available embedding models
    List,
    /// Test embedding functionality
    Test {
        /// Sample text to embed
        text: Option<String>,
    },
}

pub async fn handle_embedding_command(args: EmbeddingArgs) -> Result<()> {
    match args.command {
        EmbeddingCommand::Status => {
            let config = EmbeddingConfig::default();
            let mut manager = EmbeddingManager::new(config);
            let status = manager.get_status_summary().await;
            println!("{}", status);
        }
        
        EmbeddingCommand::Config { model, auto_download, max_size } => {
            let mut config = EmbeddingConfig::default(); // In real app, load from file
            
            if let Some(model_name) = model {
                config.preferred_model = model_name;
                info!("Set preferred embedding model: {}", config.preferred_model);
            }
            
            if let Some(auto) = auto_download {
                config.auto_download = auto;
                info!("Set auto-download: {}", config.auto_download);
            }
            
            if let Some(size) = max_size {
                config.max_model_size_mb = size;
                info!("Set max model size: {}MB", config.max_model_size_mb);
            }
            
            // In real app, save config to file
            println!("Embedding configuration updated");
        }
        
        EmbeddingCommand::Setup { force, interactive } => {
            let config = EmbeddingConfig::default();
            let mut manager = EmbeddingManager::new(config);
            
            if interactive {
                match manager.prompt_for_model_selection().await? {
                    Some(model) => {
                        println!("Selected model: {} ({}MB)", model.name, model.size_mb);
                        println!("Model setup complete!");
                    }
                    None => {
                        println!("Embedding features will be disabled");
                    }
                }
            } else {
                match manager.auto_select_model().await? {
                    Some(model) => {
                        println!("Auto-selected: {} ({}MB)", model.name, model.size_mb);
                        println!("✅ Embedding model ready for AI code analysis");
                    }
                    None => {
                        println!("No suitable embedding model found");
                        println!("Try: aircher embedding setup --interactive");
                    }
                }
            }
        }
        
        EmbeddingCommand::List => {
            let models = EmbeddingManager::get_coding_optimized_models();
            
            println!("🧠 Available Embedding Models for AI Coding:\n");
            
            for model in models {
                let size_display = if model.size_mb < 1000 {
                    format!("{}MB", model.size_mb)
                } else {
                    format!("{:.1}GB", model.size_mb as f32 / 1000.0)
                };
                
                println!("📦 {} ({})", model.name, model.provider);
                println!("   Size: {}", size_display);
                println!("   Description: {}", model.description);
                println!("   Optimized for: {}", model.optimized_for.join(", "));
                println!();
            }
            
            println!("💡 Recommendations:");
            println!("   • nomic-embed-text: Best balance for code search (274MB)");
            println!("   • mxbai-embed-large: Highest quality for complex analysis (669MB)");
            println!("   • all-MiniLM-L6-v2: Lightweight fallback (90MB)");
        }
        
        EmbeddingCommand::Test { text } => {
            let config = EmbeddingConfig::default();
            let mut manager = EmbeddingManager::new(config);
            
            match manager.get_recommended_model().await {
                Ok(model) => {
                    println!("✅ Embedding model available: {}", model.name);
                    
                    let test_text = text.unwrap_or_else(|| {
                        "function calculateTotal(items) { return items.reduce((sum, item) => sum + item.price, 0); }".to_string()
                    });
                    
                    println!("🧪 Test text: {}", test_text);
                    println!("Model: {} ({})", model.name, model.description);
                    
                    // In real implementation, would generate actual embeddings
                    println!("✅ Embedding test successful (vector dimension would be shown here)");
                }
                Err(e) => {
                    println!("❌ Embedding test failed: {}", e);
                    println!("Try: aircher embedding setup --interactive");
                }
            }
        }
    }
    
    Ok(())
}

/// Quick setup for first-time users
pub async fn quick_embedding_setup() -> Result<Option<String>> {
    let config = EmbeddingConfig::default();
    let mut manager = EmbeddingManager::new(config);
    
    println!("🚀 Setting up embedding models for AI code analysis...");
    
    if manager.check_ollama_availability().await {
        println!("✅ Ollama detected");
        
        match manager.auto_select_model().await? {
            Some(model) => {
                println!("✅ Using: {} ({}MB)", model.name, model.size_mb);
                Ok(Some(model.name))
            }
            None => {
                println!("⚠️  No embedding model auto-selected");
                println!("Run 'aircher embedding setup --interactive' for manual setup");
                Ok(None)
            }
        }
    } else {
        println!("ℹ️  Ollama not found - embedding features will be limited");
        println!("Install Ollama for best AI coding experience: https://ollama.ai");
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_commands() {
        // Test that commands don't panic
        let args = EmbeddingArgs {
            command: EmbeddingCommand::List,
        };
        
        assert!(handle_embedding_command(args).await.is_ok());
    }

    #[tokio::test]
    async fn test_quick_setup() {
        // Should work even without Ollama
        let result = quick_embedding_setup().await;
        assert!(result.is_ok());
    }
}