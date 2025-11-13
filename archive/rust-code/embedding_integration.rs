//! Example of integrating the embedding system with Aircher's AI code agent
//!
//! This demonstrates:
//! - Auto-detecting and setting up embedding models
//! - Using embeddings for semantic code search
//! - Integrating with the cost tracking system
//! - Providing fallbacks when embeddings aren't available

use anyhow::Result;
use aircher::cost::{EmbeddingManager, EmbeddingConfig, ModelConfiguration};
// quick_embedding_setup has been removed; keep the example self-contained

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("🤖 Aircher AI Code Agent - Embedding Integration Demo\n");

    // 1. Quick setup for new users (answers user's question about prompting)
    demonstrate_setup_strategies().await?;

    // 2. Show integration with model configuration
    demonstrate_model_integration().await?;

    // 3. Show practical usage in AI coding scenarios
    demonstrate_coding_scenarios().await?;

    Ok(())
}

/// Demonstrates different setup strategies based on user preferences
async fn demonstrate_setup_strategies() -> Result<()> {
    println!("📋 Embedding Setup Strategies:\n");

    // Strategy 1: Auto-setup (minimal friction)
    println!("1️⃣ Auto-Setup (Recommended for most users):");
    // Auto-setup example stub (demonstrative only)
    println!("   (auto-setup) Checking for local embedding models…");
    println!("   ⚠️  No embedding model available (demo)");
    println!();

    // Strategy 2: Configurable with sensible defaults
    println!("2️⃣ Configurable Setup:");
    let config = EmbeddingConfig {
        preferred_model: "nomic-embed-text".to_string(), // Best for AI coding
        auto_download: true, // Don't prompt by default
        use_ollama_if_available: true,
        max_model_size_mb: 700, // Allow up to mxbai-embed-large
        ..Default::default()
    };

    let mut manager = EmbeddingManager::new(config);
    match manager.auto_select_model().await? {
        Some(model) => {
            println!("   ✅ Selected: {} ({}MB)", model.name, model.size_mb);
            println!("   📊 Optimized for: {}", model.optimized_for.join(", "));
        }
        None => println!("   ⚠️  Setup required"),
    }
    println!();

    // Strategy 3: Show what interactive would look like
    println!("3️⃣ Interactive Setup (for power users):");
    println!("   Would prompt with options:");
    println!("   • Default: nomic-embed-text (274MB) - balanced");
    println!("   • Large: mxbai-embed-large (669MB) - highest quality");
    println!("   • Skip: Continue without embeddings");
    println!();

    Ok(())
}

/// Show how embedding models integrate with the 3-tier model system
async fn demonstrate_model_integration() -> Result<()> {
    println!("🔗 Integration with Model Configuration:\n");

    let model_config = ModelConfiguration::default();
    let embedding_config = EmbeddingConfig::default();
    let mut embedding_manager = EmbeddingManager::new(embedding_config);

    // Show how embeddings enhance each tier
    println!("Integration with 3-tier system:");

    if let Some(claude_models) = model_config.get_provider_models("claude") {
        println!("📍 Claude Provider:");
        println!("   Planning: {} + semantic analysis", claude_models.planning);
        println!("   Main: {} + code search", claude_models.main);
        println!("   Light: {} + quick lookup", claude_models.light);
    }

    if let Some(ollama_models) = model_config.get_provider_models("ollama") {
        println!("📍 Ollama Provider:");
        println!("   Planning: {} + nomic-embed-text", ollama_models.planning);
        println!("   Main: {} + nomic-embed-text", ollama_models.main);
        println!("   Light: {} + basic search", ollama_models.light);
        if let Some(ref embedding) = ollama_models.embedding {
            println!("   Embedding: {} ✅", embedding);
        }
    }

    // Show cost implications
    println!("\n💰 Cost Impact:");
    println!("   • Ollama models + embeddings: Free");
    println!("   • Cloud models: Enhanced context for better results");
    println!("   • Hybrid: Use embeddings to pre-filter, then cloud models");
    println!();

    Ok(())
}

/// Demonstrate practical usage scenarios for AI coding
async fn demonstrate_coding_scenarios() -> Result<()> {
    println!("🛠️ AI Coding Scenarios with Embeddings:\n");

    let config = EmbeddingConfig::default();
    let mut manager = EmbeddingManager::new(config);

    match manager.get_recommended_model().await {
        Ok(model) => {
            println!("Using embedding model: {} for the following scenarios:\n", model.name);

            // Scenario 1: Code search and context
            println!("1️⃣ Semantic Code Search:");
            println!("   User: 'Find functions that handle user authentication'");
            println!("   Process:");
            println!("   • Use {} to embed query", model.name);
            println!("   • Search codebase for similar function embeddings");
            println!("   • Provide relevant context to main model");
            println!("   • Main model gives focused, contextual response");
            println!();

            // Scenario 2: Architecture analysis
            println!("2️⃣ Architecture Analysis:");
            println!("   User: 'How do these components interact?'");
            println!("   Process:");
            println!("   • Embed code structure and relationships");
            println!("   • Find related patterns across files");
            println!("   • Use planning model with rich context");
            println!("   • Generate comprehensive architecture insights");
            println!();

            // Scenario 3: Bug hunting
            println!("3️⃣ Bug Pattern Detection:");
            println!("   User: 'Find potential null pointer issues'");
            println!("   Process:");
            println!("   • Embed known bug patterns");
            println!("   • Scan codebase for similar patterns");
            println!("   • Use main model to analyze findings");
            println!("   • Suggest fixes with context");
            println!();

            // Show model selection logic
            println!("🎯 Model Selection Logic:");
            println!("   • Light tasks: Basic keyword search + light model");
            println!("   • Main tasks: Embedding search + main model");
            println!("   • Planning tasks: Deep embedding analysis + planning model");
            println!();

        }
        Err(_) => {
            println!("❌ No embedding model available");
            println!("AI coding will use text-based analysis only");
            println!("Install Ollama for enhanced semantic capabilities");
        }
    }

    // Answer the user's question about which is best for AI agent coding
    println!("🏆 Best Models for AI Agent Coding:\n");
    println!("For AI agents specifically:");
    println!("   1. nomic-embed-text (274MB) - Designed for code");
    println!("      • Optimized for function similarity");
    println!("      • Excellent code/documentation boundary detection");
    println!("      • Fast inference for real-time suggestions");
    println!();
    println!("   2. mxbai-embed-large (669MB) - Highest quality");
    println!("      • Best for complex architectural analysis");
    println!("      • Superior cross-language understanding");
    println!("      • Worth the size for professional development");
    println!();
    println!("   3. bge-m3 (1.2GB) - For multilingual codebases");
    println!("      • Best if working with mixed languages");
    println!("      • Excellent documentation understanding");
    println!("      • Good for international team codebases");
    println!();

    println!("💡 Recommendation: Start with nomic-embed-text, upgrade to mxbai-embed-large if needed");

    Ok(())
}
