/// Test deepseek-r1 vs gpt-oss for multi-tool calling
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
    println!("🔍 COMPARING MODELS FOR MULTI-TOOL EXECUTION");
    println!("============================================\n");

    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let db_manager = DatabaseManager::new(&config).await?;
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
        let request = "List the files in src/agent/tools/, then read the first 10 lines of mod.rs from that directory";

        // Test gpt-oss
        println!("Testing gpt-oss:");
        println!("-----------------");
        let start = std::time::Instant::now();

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(20),
            agent.process_message(request, provider, "gpt-oss")
        ).await;

        match result {
            Ok(Ok((response, status_messages))) => {
                let duration = start.elapsed();
                println!("✅ Completed in {:.1}s", duration.as_secs_f64());
                println!("Tool calls: {}", status_messages.len());

                let has_list = status_messages.iter().any(|m| m.contains("list_files"));
                let has_read = status_messages.iter().any(|m| m.contains("read_file"));

                println!("  list_files: {}", has_list);
                println!("  read_file: {}", has_read);
                println!("  Result: {}", if has_list && has_read { "✅ BOTH" } else { "❌ ONLY ONE" });
            }
            Ok(Err(e)) => println!("❌ Error: {}", e),
            Err(_) => println!("❌ TIMEOUT"),
        }

        println!("\n\nTesting llama3.1:");
        println!("-----------------");
        let start = std::time::Instant::now();

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            agent.process_message(request, provider, "llama3.1")
        ).await;

        match result {
            Ok(Ok((response, status_messages))) => {
                let duration = start.elapsed();
                println!("✅ Completed in {:.1}s", duration.as_secs_f64());
                println!("Tool calls: {}", status_messages.len());

                let has_list = status_messages.iter().any(|m| m.contains("list_files"));
                let has_read = status_messages.iter().any(|m| m.contains("read_file"));

                println!("  list_files: {}", has_list);
                println!("  read_file: {}", has_read);
                println!("  Result: {}", if has_list && has_read { "✅ BOTH" } else { "❌ ONLY ONE" });

                // Show a bit of the response to see what it generates
                println!("\nResponse preview (200 chars):");
                println!("{}", response.chars().take(200).collect::<String>());
            }
            Ok(Err(e)) => println!("❌ Error: {}", e),
            Err(_) => println!("❌ TIMEOUT"),
        }
    } else {
        println!("❌ No Ollama provider available");
    }

    Ok(())
}