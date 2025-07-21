use anyhow::Result;
use std::fmt;
use tracing::{info, warn};

/// Available embedding models with clear licensing
#[derive(Debug, Clone, PartialEq)]
pub enum EmbeddingModel {
    // Apache 2.0 Licensed (Commercial OK)
    MiniLMv2 {
        name: &'static str,
        size_mb: u32,
        license: &'static str,
        quality: &'static str,
    },
    MPNetBase {
        name: &'static str,
        size_mb: u32,
        license: &'static str,
        quality: &'static str,
    },
    
    // MIT Licensed (Commercial OK)
    BGESmall {
        name: &'static str,
        size_mb: u32,
        license: &'static str,
        quality: &'static str,
    },
    
    // CC BY-NC 4.0 (Non-Commercial Only)
    SweRankEmbed {
        name: &'static str,
        size_mb: u32,
        license: &'static str,
        quality: &'static str,
        warning: &'static str,
    },
    
    // No model (fallback mode)
    None,
}

impl EmbeddingModel {
    pub fn all_models() -> Vec<Self> {
        vec![
            // Default: Fast and commercial-safe
            Self::MiniLMv2 {
                name: "all-MiniLM-L6-v2",
                size_mb: 90,
                license: "Apache 2.0",
                quality: "Good - Fast startup (~200ms), commercial-safe",
            },
            
            // Premium commercial option
            Self::MPNetBase {
                name: "gte-large",
                size_mb: 670,
                license: "Apache 2.0",
                quality: "Excellent - High quality (~800ms), commercial-safe",
            },
            
            // Premier private option
            Self::SweRankEmbed {
                name: "SweRankEmbed-Small", 
                size_mb: 260,
                license: "CC BY-NC 4.0 (NON-COMMERCIAL ONLY)",
                quality: "Premier - Code-specific training (~400ms), private use only",
                warning: "⚠️ This model is for non-commercial use only!",
            },
        ]
    }
    
    pub fn is_commercial_compatible(&self) -> bool {
        !matches!(self, Self::SweRankEmbed { .. })
    }
    
    pub fn display_info(&self) -> String {
        match self {
            Self::MiniLMv2 { name, size_mb, license, quality } => {
                format!("{:<20} {:>6} MB  License: {:<15} {}", name, size_mb, license, quality)
            }
            Self::MPNetBase { name, size_mb, license, quality } => {
                format!("{:<20} {:>6} MB  License: {:<15} {}", name, size_mb, license, quality)
            }
            Self::BGESmall { name, size_mb, license, quality } => {
                format!("{:<20} {:>6} MB  License: {:<15} {}", name, size_mb, license, quality)
            }
            Self::SweRankEmbed { name, size_mb, license, quality, warning } => {
                format!("{:<20} {:>6} MB  License: {:<15} {}\n{}", name, size_mb, license, quality, warning)
            }
            Self::None => "No model (text search only)".to_string(),
        }
    }
}

/// Interactive model selection on first run
pub async fn select_embedding_model() -> Result<EmbeddingModel> {
    println!("\n🤖 Embedding Model Selection");
    println!("============================\n");
    println!("Aircher uses embedding models for semantic code search.");
    println!("This allows finding code by meaning, not just text matching.\n");
    println!("Performance tiers (startup time → quality):\n");
    
    let models = EmbeddingModel::all_models();
    for (i, model) in models.iter().enumerate() {
        println!("{}. {}", i + 1, model.display_info());
        println!();
    }
    
    println!("4. No model (basic text search only)\n");
    
    println!("📝 Recommendations:");
    println!("   - Default choice: Model 1 (MiniLM) - fast, commercial-safe"); 
    println!("   - High quality: Model 2 (GTE-Large) - slower but excellent, commercial-safe");
    println!("   - Best for code: Model 3 (SweRankEmbed) - premier quality, private use only");
    println!("   - Commercial users: Choose models 1 or 2\n");
    
    loop {
        print!("Select model (1-4) [1]: ");
        std::io::Write::flush(&mut std::io::stdout())?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        // Default to Apache 2.0 licensed model (MiniLM) for commercial compatibility
        let choice = if input.is_empty() { 
            1 
        } else { 
            input.parse::<usize>().unwrap_or(0) 
        };
        
        match choice {
            1 => return Ok(models[0].clone()),
            2 => return Ok(models[1].clone()),
            3 => {
                // Extra confirmation for non-commercial model
                println!("\n⚠️  LICENSE WARNING");
                println!("==================");
                println!("SweRankEmbed is licensed under CC BY-NC 4.0");
                println!("This means NO COMMERCIAL USE is allowed.\n");
                println!("By selecting this model, you agree to:");
                println!("- Use it only for personal/research purposes");
                println!("- Not use it in any commercial context");
                println!("- Not distribute it with commercial software\n");
                
                print!("Do you accept these terms? [y/N]: ");
                std::io::Write::flush(&mut std::io::stdout())?;
                
                let mut confirm = String::new();
                std::io::stdin().read_line(&mut confirm)?;
                
                if confirm.trim().to_lowercase() == "y" {
                    return Ok(models[2].clone());
                } else {
                    println!("Model selection cancelled. Please choose another model.\n");
                }
            }
            4 => {
                println!("\n⚠️  No embedding model selected.");
                println!("Semantic search will be limited to basic text matching.");
                return Ok(EmbeddingModel::None);
            }
            _ => println!("Invalid choice. Please select 1-4."),
        }
    }
}

/// Download selected model with progress
pub async fn download_model(model: &EmbeddingModel) -> Result<()> {
    match model {
        EmbeddingModel::None => {
            info!("No model selected, using text search fallback");
            Ok(())
        }
        _ => {
            info!("Downloading model: {:?}", model);
            // TODO: Implement actual download from HuggingFace
            // - Show progress bar
            // - Verify checksum
            // - Extract to cache directory
            Ok(())
        }
    }
}