use anyhow::Result;
use clap::{Args, Subcommand};
use tracing::{info, debug};

use crate::cost::{EmbeddingManager, EmbeddingConfig, EmbeddingLifecycleManager, AutoSelectionEngine, SelectionCriteria};

#[derive(Debug, Args)]
pub struct EmbeddingArgs {
    #[command(subcommand)]
    pub command: Option<EmbeddingCommand>,
}

#[derive(Debug, Subcommand)]
pub enum EmbeddingCommand {
    /// List all available embedding models with current selection marked
    List,
    
    /// Set embedding model (auto for intelligent selection, or specific model name)
    Set {
        /// Model name or 'auto' for intelligent selection
        model: String,
    },
    
    /// Verify current embedding model is working
    Verify {
        /// Optional sample text to verify with
        text: Option<String>,
    },
    
    /// Update embedding models to latest versions
    Update {
        /// Check for updates without installing
        #[arg(long)]
        check_only: bool,
    },
    
    /// Clean up unused models and stale indices
    Clean {
        /// Remove unused model versions
        #[arg(long)]
        models: bool,
        /// Remove stale search indices
        #[arg(long)]
        indices: bool,
        /// Remove everything (nuclear option)
        #[arg(long)]
        all: bool,
    },
    
    /// Show storage usage and cleanup recommendations
    Status,
}

pub async fn handle_embedding_command(args: EmbeddingArgs) -> Result<()> {
    match args.command {
        // Default: show list with current marked (git branch pattern)
        None => list_embedding_models().await,
        
        Some(EmbeddingCommand::List) => list_embedding_models().await,
        Some(EmbeddingCommand::Set { model }) => set_embedding_model(&model).await,
        Some(EmbeddingCommand::Verify { text }) => verify_embedding_model(text.as_deref()).await,
        Some(EmbeddingCommand::Update { check_only }) => update_embedding_models(check_only).await,
        Some(EmbeddingCommand::Clean { models, indices, all }) => clean_embedding_storage(models, indices, all).await,
        Some(EmbeddingCommand::Status) => show_embedding_status().await,
    }
}

async fn show_current_embedding() -> Result<()> {
    let lifecycle = EmbeddingLifecycleManager::new()?;
    
    if let Some(current) = lifecycle.get_current_model() {
        println!("Current embedding model:");
        println!("  {} v{}", current.model.name, current.version);
        println!("  Provider: {}", current.model.provider);
        println!("  Size: {}MB", current.model.size_mb);
        println!("  Description: {}", current.model.description);
        
        let days_since_use = (chrono::Utc::now() - current.last_used).num_days();
        if days_since_use == 0 {
            println!("  Last used: Today");
        } else {
            println!("  Last used: {} days ago", days_since_use);
        }
    } else {
        println!("❌ No embedding model configured");
        println!("💡 Run 'aircher embedding set auto' to set up automatically");
    }
    
    Ok(())
}

async fn list_embedding_models() -> Result<()> {
    let lifecycle = EmbeddingLifecycleManager::new()?;
    let models = lifecycle.list_models();
    
    if models.is_empty() {
        println!("No embedding models installed.");
        println!("💡 Run 'aircher embedding set auto' to install SweRankEmbed-Small");
        return Ok(());
    }
    
    println!("Available embedding models:");
    println!();
    
    for (_key, entry, is_current) in models {
        let marker = if is_current { " ← current" } else { "" };
        let provider_info = match entry.model.provider.as_str() {
            "embedded" => "🔹 Embedded".to_string(),
            "ollama" => "🏠 Ollama".to_string(), 
            "huggingface" => "🌐 HuggingFace".to_string(),
            _ => format!("📡 {}", entry.model.provider),
        };
        
        println!("  {} v{} ({}MB) {}{}", 
                entry.model.name, 
                entry.version,
                entry.model.size_mb,
                provider_info,
                marker);
        println!("    {}", entry.model.description);
        
        if !entry.model.optimized_for.is_empty() {
            println!("    Optimized for: {}", entry.model.optimized_for.join(", "));
        }
        println!();
    }
    
    println!("💡 Use 'aircher embedding set <model>' to switch models");
    println!("💡 Use 'aircher embedding set auto' for intelligent selection");
    
    Ok(())
}

async fn set_embedding_model(model: &str) -> Result<()> {
    let mut lifecycle = EmbeddingLifecycleManager::new()?;
    
    if model == "auto" {
        // Intelligent auto-selection
        println!("🧠 Selecting optimal embedding model...");
        
        let embedding_config = EmbeddingConfig::default();
        let mut embedding_manager = EmbeddingManager::new(embedding_config);
        let mut auto_engine = AutoSelectionEngine::new();
        let criteria = SelectionCriteria::default();
        
        match auto_engine.select_optimal_model(&mut embedding_manager, &criteria).await {
            Ok(selected_model) => {
                println!("✅ Selected: {} ({}MB)", selected_model.name, selected_model.size_mb);
                println!("  {}", selected_model.description);
                
                // Install if not already installed
                lifecycle.install_model(&selected_model, "1.0").await?;
                let model_key = format!("{}-v1.0", selected_model.name);
                lifecycle.set_default_model(&model_key)?;
                
                println!("🎯 Embedding model configured successfully!");
            },
            Err(e) => {
                println!("❌ Auto-selection failed: {}", e);
                println!("💡 Try 'aircher embedding list' to see available models");
            }
        }
    } else {
        // Set specific model
        // Collect available model keys first
        let available_keys: Vec<String> = lifecycle.list_models()
            .into_iter()
            .map(|(key, _, _)| key.clone())
            .collect();
        
        // Find matching key
        let found_key = available_keys.iter()
            .find(|key| key.starts_with(model) || key.contains(model));
        
        if let Some(model_key) = found_key {
            lifecycle.set_default_model(model_key)?;
            // Get the model info after setting it
            if let Some(current) = lifecycle.get_current_model() {
                println!("✅ Set embedding model to: {} v{}", current.model.name, current.version);
            }
        } else {
            println!("❌ Model '{}' not found", model);
            println!("💡 Use 'aircher embedding list' to see available models");
            println!("💡 Use 'aircher embedding set auto' for automatic selection");
        }
    }
    
    Ok(())
}

async fn verify_embedding_model(test_text: Option<&str>) -> Result<()> {
    let lifecycle = EmbeddingLifecycleManager::new()?;
    
    let current = lifecycle.get_current_model()
        .ok_or_else(|| anyhow::anyhow!("No embedding model configured"))?;
    
    let sample_text = test_text.unwrap_or("function authenticate(username, password) { return validateCredentials(username, password); }");
    
    println!("✅ Verifying embedding model: {} v{}", current.model.name, current.version);
    println!("📝 Sample text: {}", sample_text);
    
    // Initialize embedding manager and verify
    let embedding_config = EmbeddingConfig::default();
    let mut embedding_manager = EmbeddingManager::new(embedding_config);
    
    let start_time = std::time::Instant::now();
    
    match embedding_manager.generate_embeddings_with_model(sample_text, &current.model.name).await {
        Ok(embeddings) => {
            let elapsed = start_time.elapsed();
            
            println!("✅ Verification successful!");
            println!("  📊 Generated {} dimensions", embeddings.len());
            println!("  ⏱️  Response time: {:.0}ms", elapsed.as_millis());
            
            // Show a sample of the embedding vector
            if embeddings.len() >= 5 {
                println!("  🔢 Sample values: [{:.3}, {:.3}, {:.3}, {:.3}, {:.3}...]", 
                        embeddings[0], embeddings[1], embeddings[2], embeddings[3], embeddings[4]);
            }
        },
        Err(e) => {
            println!("❌ Verification failed: {}", e);
            println!("💡 Try 'aircher embedding set auto' to configure a working model");
        }
    }
    
    Ok(())
}

async fn update_embedding_models(check_only: bool) -> Result<()> {
    let mut lifecycle = EmbeddingLifecycleManager::new()?;
    
    println!("🔍 Checking for embedding model updates...");
    
    let updates = lifecycle.check_for_updates().await?;
    
    if updates.is_empty() {
        println!("✅ All embedding models are up to date");
        return Ok(());
    }
    
    println!("📦 Found {} available updates:", updates.len());
    println!();
    
    for update in &updates {
        println!("  {} v{} → v{}", 
                update.model_name, 
                update.current_version,
                update.latest_version);
        println!("    📈 {}", update.improvement_description);
        println!("    📥 Download size: {}MB", update.download_size / 1_000_000);
        if update.breaking_change {
            println!("    ⚠️  Breaking change - will require re-indexing");
        }
        println!();
    }
    
    if check_only {
        println!("💡 Run 'aircher embedding update' to install updates");
    } else {
        println!("🚀 Installing updates...");
        
        for update in updates {
            println!("📦 Updating {}...", update.model_name);
            
            // Create a dummy model for installation
            let model = crate::cost::EmbeddingModel {
                name: update.model_name.clone(),
                provider: "embedded".to_string(),
                size_mb: (update.download_size / 1_000_000) as u32,
                description: "Updated model".to_string(),
                optimized_for: vec!["code".to_string()],
                download_url: None,
            };
            
            lifecycle.install_model(&model, &update.latest_version).await?;
            println!("✅ {} updated to v{}", update.model_name, update.latest_version);
        }
        
        println!("🎉 All updates completed successfully!");
    }
    
    Ok(())
}

async fn clean_embedding_storage(clean_models: bool, clean_indices: bool, clean_all: bool) -> Result<()> {
    let mut lifecycle = EmbeddingLifecycleManager::new()?;
    
    if clean_all {
        println!("🧹 Performing complete cleanup...");
    } else if !clean_models && !clean_indices {
        // Default: show what would be cleaned
        let storage_info = lifecycle.get_storage_info()?;
        
        println!("📊 Storage usage:");
        println!("  Total size: {:.1}MB", storage_info.total_size as f64 / 1_000_000.0);
        println!("  Models: {}", storage_info.model_count);
        println!("  Indices: {}", storage_info.index_count);
        
        if !storage_info.unused_models.is_empty() {
            println!("\n🗑️  Unused models (can be cleaned):");
            for model in &storage_info.unused_models {
                println!("    {}", model);
            }
        }
        
        if !storage_info.stale_indices.is_empty() {
            println!("\n📁 Stale indices (can be cleaned):");
            for index in &storage_info.stale_indices {
                println!("    {}", index);
            }
        }
        
        if storage_info.unused_models.is_empty() && storage_info.stale_indices.is_empty() {
            println!("\n✨ Nothing to clean - storage is optimized!");
        } else {
            println!("\n💡 Use 'aircher embedding clean --models --indices' to clean up");
        }
        
        return Ok(());
    }
    
    let freed_bytes = lifecycle.cleanup(
        clean_models || clean_all,
        clean_indices || clean_all
    ).await?;
    
    if freed_bytes > 0 {
        println!("✅ Cleanup completed!");
        println!("  Freed: {:.1}MB", freed_bytes as f64 / 1_000_000.0);
    } else {
        println!("✨ Nothing to clean - storage is already optimized!");
    }
    
    Ok(())
}

async fn show_embedding_status() -> Result<()> {
    let lifecycle = EmbeddingLifecycleManager::new()?;
    let storage_info = lifecycle.get_storage_info()?;
    
    println!("📊 Embedding Storage Status:");
    println!();
    
    // Current model
    if let Some(current) = lifecycle.get_current_model() {
        println!("Current model:");
        println!("  {} v{} ({}MB)", current.model.name, current.version, current.model.size_mb);
        println!("  Installed: {}", current.installed_at.format("%Y-%m-%d"));
        println!("  Last used: {}", current.last_used.format("%Y-%m-%d"));
        println!();
    }
    
    // Storage breakdown
    println!("Storage usage:");
    println!("  Total: {:.1}MB", storage_info.total_size as f64 / 1_000_000.0);
    println!("  Models: {} ({} unused)", storage_info.model_count, storage_info.unused_models.len());
    println!("  Indices: {} ({} stale)", storage_info.index_count, storage_info.stale_indices.len());
    println!();
    
    // Recommendations
    let total_cleanable = storage_info.unused_models.len() + storage_info.stale_indices.len();
    if total_cleanable > 0 {
        println!("🧹 Cleanup recommendations:");
        if !storage_info.unused_models.is_empty() {
            println!("  {} unused model versions can be removed", storage_info.unused_models.len());
        }
        if !storage_info.stale_indices.is_empty() {
            println!("  {} stale indices can be removed", storage_info.stale_indices.len());
        }
        println!("  💡 Run 'aircher embedding clean' to see details");
    } else {
        println!("✨ Storage is optimized - no cleanup needed!");
    }
    
    Ok(())
}

/// Quick embedding setup for new users (called internally)
pub async fn quick_embedding_setup() -> Result<Option<String>> {
    info!("Setting up embedding model automatically...");
    
    let embedding_config = EmbeddingConfig::default();
    let mut embedding_manager = EmbeddingManager::new(embedding_config);
    let mut auto_engine = AutoSelectionEngine::new();
    let criteria = SelectionCriteria::default();
    
    match auto_engine.select_optimal_model(&mut embedding_manager, &criteria).await {
        Ok(selected_model) => {
            let mut lifecycle = EmbeddingLifecycleManager::new()?;
            lifecycle.install_model(&selected_model, "1.0").await?;
            
            let model_key = format!("{}-v1.0", selected_model.name);
            lifecycle.set_default_model(&model_key)?;
            
            info!("Auto-configured embedding model: {}", selected_model.name);
            Ok(Some(selected_model.name))
        },
        Err(e) => {
            debug!("Auto-setup failed: {}", e);
            Ok(None)
        }
    }
}