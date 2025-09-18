/// Simple test of semantic-aware code generation
use anyhow::Result;
use std::sync::Arc;

use aircher::auth::AuthManager;
use aircher::config::ConfigManager;
use aircher::storage::DatabaseManager;
use aircher::intelligence::IntelligenceEngine;
use aircher::semantic_search::SemanticCodeSearch;
use aircher::agent::{
    core::Agent,
    conversation::{ProjectContext, ProgrammingLanguage},
};
use aircher::providers::ProviderManager;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 SIMPLE SEMANTIC CODE GENERATION TEST");
    println!("=====================================\n");

    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let db_manager = DatabaseManager::new(&config).await?;

    // Quick semantic search setup
    let mut semantic_search = SemanticCodeSearch::new();
    if let Ok(_) = semantic_search.load_persisted_index().await {
        println!("✅ Using existing semantic index");
    } else {
        println!("⚠️ No semantic index - will use basic intelligence");
    }

    let intelligence = IntelligenceEngine::with_semantic_search(&config, &db_manager, semantic_search).await?;
    let provider_manager = Arc::new(ProviderManager::new(&config, auth_manager.clone()).await?);

    let project_context = ProjectContext {
        root_path: std::env::current_dir()?,
        language: ProgrammingLanguage::Rust,
        framework: None,
        recent_changes: Vec::new(),
    };

    let agent = Agent::new(intelligence, auth_manager, project_context).await?;

    if let Some(provider) = provider_manager.get_provider("ollama") {
        println!("🧠 Testing semantic-aware code generation...\n");

        // Simple request that should trigger contextual generation without tools
        let request = "Create a Rust function for validating email addresses that follows our project patterns";

        println!("Request: {}", request);
        println!("\n⏱️ Processing...");

        match agent.process_message(request, provider, "gpt-oss").await {
            Ok((response, status_messages)) => {
                println!("\n📊 RESULT:");
                println!("Response length: {} characters", response.len());
                println!("Status messages: {} items", status_messages.len());

                if !status_messages.is_empty() {
                    println!("Status messages: {:?}", status_messages);
                }

                // Check if it generated code directly without using tools
                let used_tools = !status_messages.is_empty();
                let has_code = response.contains("fn ") || response.contains("```rust");
                let has_validation = response.to_lowercase().contains("email") && response.to_lowercase().contains("valid");
                let reasonable_length = response.len() > 500 && response.len() < 50_000;

                println!("\n🔍 ANALYSIS:");
                println!("  Used tools: {}", used_tools);
                println!("  Contains code: {}", has_code);
                println!("  Email validation context: {}", has_validation);
                println!("  Reasonable length: {} ({})", reasonable_length, response.len());

                if !used_tools && has_code && has_validation && reasonable_length {
                    println!("\n✅ SUCCESS: Generated contextual code without unnecessary tools");
                } else {
                    println!("\n❌ ISSUE: Semantic integration needs refinement");
                }

                println!("\n📄 Response preview (first 1000 chars):");
                println!("---START---");
                println!("{}", response.chars().take(1000).collect::<String>());
                if response.len() > 1000 {
                    println!("... (truncated {} more characters)", response.len() - 1000);
                }
                println!("---END---");
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }
    } else {
        println!("❌ No Ollama provider available");
    }

    Ok(())
}