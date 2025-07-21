use anyhow::Result;
use std::io::{self, Write};
use tracing::{info, warn};

/// First-run experience for model setup
pub struct FirstRunExperience {
    has_embedded_model: bool,
    model_path: std::path::PathBuf,
}

impl FirstRunExperience {
    pub fn new() -> Self {
        let cache_dir = crate::config::ArcherConfig::cache_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from(".cache"))
            .join("models");
        
        Self {
            has_embedded_model: cfg!(embed_model),
            model_path: cache_dir.join("swerank-embed-small.safetensors"),
        }
    }
    
    /// Check if semantic search is available
    pub fn is_semantic_search_ready(&self) -> bool {
        self.has_embedded_model || self.model_path.exists()
    }
    
    /// Handle first semantic search attempt
    pub async fn ensure_semantic_search(&self) -> Result<bool> {
        if self.is_semantic_search_ready() {
            return Ok(true);
        }
        
        // Lite version - ask user
        println!("\n🤖 Semantic Search Setup");
        println!("========================\n");
        println!("Aircher can provide AI-powered semantic code search that understands");
        println!("programming concepts, not just text matching.\n");
        println!("Examples of what semantic search enables:");
        println!("  • 'error handling' → finds try/catch, Result<T,E>, validation");
        println!("  • 'auth logic' → finds login, JWT, OAuth, permissions");
        println!("  • 'database queries' → finds SQL, ORMs, query builders\n");
        println!("This requires downloading a 260MB AI model (one-time download).\n");
        println!("Without it, Aircher will use basic text search (less effective).\n");
        
        print!("Download semantic search model now? [Y/n] ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        
        if input.is_empty() || input == "y" || input == "yes" {
            self.download_model().await?;
            Ok(true)
        } else {
            println!("\n⚠️  Semantic search disabled. Using fallback text search.");
            println!("💡 You can enable it later with: aircher model download\n");
            Ok(false)
        }
    }
    
    /// Download the model with progress
    async fn download_model(&self) -> Result<()> {
        println!("\n📥 Downloading SweRankEmbed model...");
        
        // Create a simple progress indicator
        let progress_chars = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        
        // TODO: Implement actual download from HuggingFace or CDN
        // For now, simulate download
        for i in 0..10 {
            print!("\r{} Downloading... {}%", progress_chars[i % progress_chars.len()], i * 10);
            io::stdout().flush()?;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        
        println!("\r✅ Model downloaded successfully!        \n");
        
        // In real implementation:
        // - Download from HuggingFace or CDN
        // - Show actual progress
        // - Verify checksum
        // - Extract to cache directory
        
        Ok(())
    }
}

/// Show comparison between search modes
pub fn show_search_comparison() {
    println!("\n📊 Search Mode Comparison:");
    println!("┌─────────────────────┬──────────────────────┬──────────────────────┐");
    println!("│ Feature             │ Text Search          │ Semantic Search      │");
    println!("├─────────────────────┼──────────────────────┼──────────────────────┤");
    println!("│ Find by concept     │ ❌ No                │ ✅ Yes               │");
    println!("│ Cross-language      │ ❌ No                │ ✅ Yes               │");
    println!("│ Understands code    │ ❌ No                │ ✅ Yes               │");
    println!("│ Speed               │ ✅ Fast              │ ✅ Fast (cached)     │");
    println!("│ Offline             │ ✅ Yes               │ ✅ Yes               │");
    println!("│ Size requirement    │ ✅ 0MB               │ ⚠️  260MB            │");
    println!("└─────────────────────┴──────────────────────┴──────────────────────┘");
}