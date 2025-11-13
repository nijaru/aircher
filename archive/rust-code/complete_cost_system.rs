//! Complete demonstration of Aircher's cost optimization and embedding system
//!
//! This example shows the full integration answering the user's questions:
//! 1. Should we prompt users for model downloads?
//! 2. Default to large models with options for small or skip?
//! 3. Which models are best for AI agent coding?
//! 4. Integration with the 3-tier model system

use anyhow::Result;
use aircher::cost::{
    EmbeddingManager, EmbeddingConfig, ModelConfiguration, ModelTier,
    PricingAPI, CostTracker, CostConfig
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("🤖 Aircher Complete Cost & Embedding System Demo\n");

    // Answer user's questions with practical implementation
    demonstrate_embedding_strategy().await?;
    demonstrate_model_selection().await?;
    demonstrate_cost_integration().await?;

    Ok(())
}

/// Answers: "should we even prompt the user? default to large and then have options for small or skip?"
async fn demonstrate_embedding_strategy() -> Result<()> {
    println!("🧠 Embedding Strategy - User Experience Design\n");

    // Strategy 1: No prompting - auto-select best available
    println!("1️⃣ Recommended Approach: Auto-select with smart defaults");
    let config = EmbeddingConfig {
        preferred_model: "nomic-embed-text".to_string(),
        auto_download: true, // No prompting
        use_ollama_if_available: true,
        max_model_size_mb: 700, // Allow up to mxbai-embed-large
        ..Default::default()
    };

    let mut manager = EmbeddingManager::new(config);

    match manager.auto_select_model().await? {
        Some(model) => {
            println!("   ✅ Auto-selected: {} ({}MB)", model.name, model.size_mb);
            println!("   🎯 Reasoning: {}", model.description);
            println!("   ⚡ Zero user friction - works out of the box");
        }
        None => {
            println!("   ⚠️  No suitable model found");
            println!("   💡 Graceful fallback: Basic text search still works");
        }
    }
    println!();

    // Strategy 2: Minimal prompting with smart options
    println!("2️⃣ Alternative: One-time setup with smart options");
    println!("   Prompt: 'Download embedding model for better code search?'");
    println!("   Options:");
    println!("   • Recommended (274MB) - Best balance");
    println!("   • Advanced (669MB) - Highest quality");
    println!("   • Skip - Continue with basic search");
    println!("   🎯 Default to 'Recommended' if user just hits Enter");
    println!();

    // Why this approach is best for AI coding
    println!("🎯 Why this strategy works for AI agents:");
    println!("   • Most users want 'it just works' experience");
    println!("   • nomic-embed-text is specifically designed for code");
    println!("   • 274MB is reasonable for development machines");
    println!("   • Power users can always upgrade to mxbai-embed-large");
    println!("   • Graceful degradation if no embeddings available");
    println!();

    Ok(())
}

/// Answers: "which is best for an ai agent coding?"
async fn demonstrate_model_selection() -> Result<()> {
    println!("🏆 Best Models for AI Agent Coding - Detailed Analysis\n");

    let models = EmbeddingManager::get_coding_optimized_models();

    println!("📊 Model Comparison for AI Coding Tasks:\n");

    for (rank, model) in models.iter().enumerate() {
        let rank_emoji = match rank {
            0 => "🥇", // Gold
            1 => "🥈", // Silver
            2 => "🥉", // Bronze
            _ => "📦",
        };

        println!("{} {} ({})", rank_emoji, model.name, model.provider);
        println!("   Size: {}MB", model.size_mb);
        println!("   {}", model.description);
        println!("   Best for: {}", model.optimized_for.join(", "));

        // Add specific AI coding insights
        match model.name.as_str() {
            "nomic-embed-text" => {
                println!("   🎯 AI Agent Benefits:");
                println!("      • Understands code structure vs documentation");
                println!("      • Fast enough for real-time suggestions");
                println!("      • Excellent function similarity detection");
                println!("      • Good at finding related code patterns");
            }
            "mxbai-embed-large" => {
                println!("   🎯 AI Agent Benefits:");
                println!("      • Best cross-language understanding");
                println!("      • Superior at complex architectural queries");
                println!("      • Excellent for design pattern recognition");
                println!("      • Worth the size for professional development");
            }
            "bge-m3" => {
                println!("   🎯 AI Agent Benefits:");
                println!("      • Best for international teams");
                println!("      • Handles mixed-language codebases well");
                println!("      • Good documentation/code boundary detection");
            }
            "all-MiniLM-L6-v2" => {
                println!("   🎯 AI Agent Benefits:");
                println!("      • Ultra-fast inference for basic tasks");
                println!("      • Good enough for simple code search");
                println!("      • Excellent battery life on laptops");
            }
            _ => {}
        }
        println!();
    }

    println!("💡 Recommendation Algorithm for AI Agents:");
    println!("   1. Default: nomic-embed-text - specifically designed for code");
    println!("   2. If complex codebase (>100k LOC): mxbai-embed-large");
    println!("   3. If multilingual team: bge-m3");
    println!("   4. If resource constrained: all-MiniLM-L6-v2");
    println!("   5. If Ollama unavailable: Graceful fallback to text search");
    println!();

    Ok(())
}

/// Shows complete integration with cost tracking and 3-tier models
async fn demonstrate_cost_integration() -> Result<()> {
    println!("💰 Complete Cost & Model Integration\n");

    // Set up the complete system
    let model_config = ModelConfiguration::default();
    let cost_config = CostConfig::default();
    let cost_tracker = CostTracker::new(cost_config);
    let embedding_config = EmbeddingConfig::default();
    let mut embedding_manager = EmbeddingManager::new(embedding_config);
    let mut pricing_api = PricingAPI::new();

    println!("🏗️ System Architecture:");
    println!("   3-Tier Models + Embeddings + Cost Tracking + Live Pricing");
    println!();

    // Show how different tasks use different combinations
    let scenarios = vec![
        ("commit_messages", ModelTier::Light, "Simple commit message generation"),
        ("code_search", ModelTier::Main, "Find similar functions using embeddings"),
        ("architecture_review", ModelTier::Planning, "Complex architectural analysis"),
        ("quick_question", ModelTier::Light, "Fast code explanation"),
    ];

    for (task, tier, description) in scenarios {
        println!("🎯 Task: {} ({})", task, description);

        // Get the model for this task
        let (model, selected_tier, reason) = model_config.select_model("claude", Some(task));
        assert_eq!(selected_tier, tier); // Verify our routing works

        // Get pricing info
        let pricing = pricing_api.get_model_pricing("claude", &model).await
            .unwrap_or_else(|| "TBD".to_string());

        // Show embedding usage
        let embedding_model = embedding_manager.get_recommended_model().await
            .map(|m| m.name)
            .unwrap_or_else(|_| "none".to_string());

        println!("   Model: {} ({}) - {}", model, pricing, reason);
        println!("   Embedding: {} for semantic search", embedding_model);

        // Show cost implication
        match tier {
            ModelTier::Light => println!("   Cost: Low - frequent use OK"),
            ModelTier::Main => println!("   Cost: Medium - balanced approach"),
            ModelTier::Planning => println!("   Cost: High - use sparingly for complex tasks"),
        }
        println!();
    }

    // Show cost tracking integration
    println!("📈 Cost Tracking Integration:");
    println!("   Daily usage: {}", cost_tracker.get_daily_summary(None));
    println!("   Monthly usage: {}", cost_tracker.get_monthly_summary());
    println!("   Provider rankings: {:?}", cost_tracker.get_provider_rankings());
    println!();

    // Show the complete user experience
    println!("✨ Complete User Experience:");
    println!("   1. First run: Auto-setup embedding model (no prompting)");
    println!("   2. User asks: 'Find authentication functions'");
    println!("   3. System: Uses embeddings to find relevant code");
    println!("   4. System: Routes to appropriate model tier based on complexity");
    println!("   5. System: Tracks cost and warns if approaching limits");
    println!("   6. System: Provides contextual, cost-efficient response");
    println!();

    println!("🎉 Result: Intelligent, cost-aware AI coding assistant!");

    Ok(())
}

/// Bonus: Show configuration management
#[allow(dead_code)]
async fn demonstrate_configuration() -> Result<()> {
    println!("⚙️ Configuration Management\n");

    // Users can customize via CLI or config file
    println!("CLI customization examples:");
    println!("   aircher config model claude planning claude-3-opus    # Upgrade planning model");
    println!("   aircher config model openai light gpt-4o-mini        # Set light model");
    println!("   aircher embedding config --model mxbai-embed-large   # Upgrade embedding");
    println!("   aircher embedding config --max-size 500              # Limit model size");
    println!();

    // Config file approach
    println!("Config file (aircher.toml):");
    println!(r#"
[models.claude]
planning = "claude-3-opus"           # Best reasoning
main = "claude-3-5-sonnet-20241022"  # Balanced
light = "claude-3-5-haiku-20241022"  # Fast & cheap

[embedding]
preferred = "nomic-embed-text"       # Code-optimized
auto_download = true                 # No prompting
max_size_mb = 700                    # Allow large models

[cost]
monthly_budget = 50.0                # $50/month
daily_limit = 3.0                    # $3/day
alert_threshold = 0.8                # Warn at 80%
"#);

    Ok(())
}
