use anyhow::Result;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn, debug};

/// Available embedding models with clear licensing
#[derive(Debug, Clone, PartialEq)]
pub enum EmbeddingModel {
    // Apache 2.0 Licensed (Commercial OK)
    MiniLMv2 {
        name: &'static str,
        size_mb: u32,
        license: &'static str,
        quality: &'static str,
        hf_repo: &'static str,
    },
    MPNetBase {
        name: &'static str,
        size_mb: u32,
        license: &'static str,
        quality: &'static str,
        hf_repo: &'static str,
    },
    
    // MIT Licensed (Commercial OK)  
    BGESmall {
        name: &'static str,
        size_mb: u32,
        license: &'static str,
        quality: &'static str,
        hf_repo: &'static str,
    },
    
    // CC BY-NC 4.0 (Non-Commercial Only)
    SweRankEmbed {
        name: &'static str,
        size_mb: u32,
        license: &'static str,
        quality: &'static str,
        warning: &'static str,
        hf_repo: &'static str,
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
                hf_repo: "sentence-transformers/all-MiniLM-L6-v2",
            },
            
            // Premium commercial option
            Self::MPNetBase {
                name: "gte-large",
                size_mb: 670,
                license: "Apache 2.0", 
                quality: "Excellent - High quality (~800ms), commercial-safe",
                hf_repo: "thenlper/gte-large",
            },
            
            // Premier private option
            Self::SweRankEmbed {
                name: "SweRankEmbed-Small",
                size_mb: 260,
                license: "CC BY-NC 4.0 (NON-COMMERCIAL ONLY)",
                quality: "Premier - Code-specific training (~400ms), private use only", 
                warning: "⚠️ This model is for non-commercial use only!",
                hf_repo: "SalesforceAIResearch/SweRank",
            },
        ]
    }
    
    pub fn is_commercial_compatible(&self) -> bool {
        !matches!(self, Self::SweRankEmbed { .. })
    }
    
    pub fn display_info(&self) -> String {
        match self {
            Self::MiniLMv2 { name, size_mb, license, quality, .. } => {
                format!("{:<20} {:>6} MB  License: {:<15} {}", name, size_mb, license, quality)
            }
            Self::MPNetBase { name, size_mb, license, quality, .. } => {
                format!("{:<20} {:>6} MB  License: {:<15} {}", name, size_mb, license, quality)
            }
            Self::BGESmall { name, size_mb, license, quality, .. } => {
                format!("{:<20} {:>6} MB  License: {:<15} {}", name, size_mb, license, quality)
            }
            Self::SweRankEmbed { name, size_mb, license, quality, warning, .. } => {
                format!("{:<20} {:>6} MB  License: {:<15} {}\n{}", name, size_mb, license, quality, warning)
            }
            Self::None => "No model (text search only)".to_string(),
        }
    }
    
    pub fn hf_repo(&self) -> Option<&str> {
        match self {
            Self::MiniLMv2 { hf_repo, .. } => Some(hf_repo),
            Self::MPNetBase { hf_repo, .. } => Some(hf_repo),
            Self::BGESmall { hf_repo, .. } => Some(hf_repo),
            Self::SweRankEmbed { hf_repo, .. } => Some(hf_repo),
            Self::None => None,
        }
    }
    
    pub fn cache_dir_name(&self) -> Option<&str> {
        match self {
            Self::MiniLMv2 { name, .. } => Some(name),
            Self::MPNetBase { name, .. } => Some(name),
            Self::BGESmall { name, .. } => Some(name),
            Self::SweRankEmbed { name, .. } => Some(name),
            Self::None => None,
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
            return Ok(());
        }
        _ => {}
    }
    
    let Some(repo_id) = model.hf_repo() else {
        warn!("No HuggingFace repository configured for model");
        return Ok(());
    };
    
    let Some(model_name) = model.cache_dir_name() else {
        warn!("No cache directory name configured for model");
        return Ok(());
    };
    
    // Get cache directory
    let cache_dir = crate::config::ArcherConfig::cache_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get cache directory: {}", e))?
        .join("models")
        .join(model_name);
    
    // Create cache directory
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }
    
    let model_file = cache_dir.join("model.safetensors");
    
    // Check if model already exists
    if model_file.exists() {
        info!("Model already exists: {}", model_file.display());
        return Ok(());
    }
    
    info!("Downloading {} from HuggingFace...", model_name);
    
    // Initialize HuggingFace API
    let api = hf_hub::api::tokio::Api::new()?;
    let repo = api.model(repo_id.to_string());
    
    // Download model files with progress indication
    println!("📥 Downloading {}...", model_name);
    
    // Show a simple progress indicator
    let progress_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut progress_idx = 0;
    
    // Download the model file
    match repo.get("model.safetensors").await {
        Ok(model_path) => {
            // Copy from HF cache to our cache
            let hf_model_path = model_path;
            fs::copy(&hf_model_path, &model_file)?;
            
            println!("\r✅ Model downloaded successfully!");
            info!("Model cached at: {}", model_file.display());
            
            // Save model info
            let info_file = cache_dir.join(".model_info");
            let model_info = serde_json::json!({
                "name": model_name,
                "repo": repo_id,
                "downloaded_at": chrono::Utc::now().to_rfc3339(),
                "file_size": fs::metadata(&model_file)?.len()
            });
            fs::write(info_file, serde_json::to_string_pretty(&model_info)?)?;
        }
        Err(e) => {
            // Try alternative files for different model formats
            let alternative_files = ["pytorch_model.bin", "model.bin", "model.onnx"];
            let mut downloaded = false;
            
            for alt_file in &alternative_files {
                if let Ok(alt_path) = repo.get(alt_file).await {
                    fs::copy(&alt_path, cache_dir.join(alt_file))?;
                    downloaded = true;
                    info!("Downloaded alternative model file: {}", alt_file);
                    break;
                }
            }
            
            if !downloaded {
                return Err(anyhow::anyhow!("Failed to download model: {}", e));
            }
        }
    }
    
    // Download config files if available
    let config_files = ["config.json", "tokenizer.json", "tokenizer_config.json"];
    for config_file in &config_files {
        if let Ok(config_path) = repo.get(config_file).await {
            let dest = cache_dir.join(config_file);
            if let Err(e) = fs::copy(&config_path, &dest) {
                debug!("Could not copy {}: {}", config_file, e);
            }
        }
    }
    
    println!("\n🎉 Model setup complete! Ready for semantic search.");
    Ok(())
}