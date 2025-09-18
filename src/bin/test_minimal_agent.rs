/// Minimal test of agent without semantic search
use anyhow::Result;
use std::sync::Arc;

use aircher::auth::AuthManager;
use aircher::config::ConfigManager;
use aircher::storage::DatabaseManager;
use aircher::intelligence::IntelligenceEngine;
use aircher::agent::{
    core::Agent,
    conversation::{ProjectContext, ProgrammingLanguage},
};
use aircher::providers::ProviderManager;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 MINIMAL AGENT TEST - NO SEMANTIC SEARCH");
    println!("==========================================\n");

    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let db_manager = DatabaseManager::new(&config).await?;

    // Create intelligence engine WITHOUT semantic search
    let intelligence = IntelligenceEngine::new(&config, &db_manager).await?;
    let provider_manager = Arc::new(ProviderManager::new(&config, auth_manager.clone()).await?);

    let project_context = ProjectContext {
        root_path: std::env::current_dir()?,
        language: ProgrammingLanguage::Rust,
        framework: None,
        recent_changes: Vec::new(),
    };

    let agent = Agent::new(intelligence, auth_manager, project_context).await?;

    if let Some(provider) = provider_manager.get_provider("ollama") {
        println!("🧠 Testing agent without semantic search...\n");

        // Simple code generation request
        let request = "Create a simple hello world function in Rust";

        println!("Request: {}", request);
        println!("\n⏱️ Processing...");

        match agent.process_message(request, provider, "gpt-oss").await {
            Ok((response, status_messages)) => {
                println!("\n📊 RESULT:");
                println!("Response length: {} characters", response.len());
                println!("Status messages: {} items", status_messages.len());

                if response.is_empty() {
                    println!("❌ EMPTY RESPONSE!");
                } else {
                    println!("✅ Got response!");
                    println!("\n📄 Response:");
                    println!("---START---");
                    println!("{}", response);
                    println!("---END---");
                }

                if !status_messages.is_empty() {
                    println!("\nStatus messages:");
                    for msg in status_messages {
                        println!("  - {}", msg);
                    }
                }
            }
            Err(e) => {
                println!("❌ Error: {}", e);
                println!("Error chain: {:?}", e.chain().collect::<Vec<_>>());
            }
        }
    } else {
        println!("❌ No Ollama provider available");
    }

    Ok(())
}