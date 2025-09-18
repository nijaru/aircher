/// Test two-tool execution - find where it breaks
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
    println!("🔍 TESTING TWO-TOOL EXECUTION");
    println!("=============================\n");

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
        println!("Test: Two-step task");
        println!("-------------------\n");

        // This SHOULD trigger two tool calls
        let request = "List the files in src/providers/, then read the first 5 lines of mod.rs from that directory";
        println!("Request: {}\n", request);
        println!("Expected: list_files, then read_file\n");

        let start = std::time::Instant::now();

        // Add timeout to prevent hanging
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            agent.process_message(request, provider, "qwen2.5-coder:7b")
        ).await;

        match result {
            Ok(Ok((response, status_messages))) => {
                let duration = start.elapsed();
                println!("✅ Completed in {:.1}s", duration.as_secs_f64());
                println!("Tool calls made: {}", status_messages.len());

                println!("\n📋 Tool execution sequence:");
                for (i, msg) in status_messages.iter().enumerate() {
                    println!("  {}: {}", i + 1, msg);
                }

                // Analyze what happened
                let has_list = status_messages.iter().any(|m| m.contains("list_files"));
                let has_read = status_messages.iter().any(|m| m.contains("read_file"));

                println!("\n🔍 Analysis:");
                println!("  list_files called: {}", has_list);
                println!("  read_file called: {}", has_read);
                println!("  Both tools executed: {}", has_list && has_read);

                if has_list && has_read {
                    println!("\n✅ SUCCESS: Multi-tool execution works!");
                } else if has_list || has_read {
                    println!("\n⚠️ PARTIAL: Only one tool executed");
                    println!("This is the PROBLEM - agent doesn't continue after first tool");
                } else {
                    println!("\n❌ FAILURE: No tools executed");
                }

                println!("\n📄 Response (first 500 chars):");
                println!("{}", response.chars().take(500).collect::<String>());
            }
            Ok(Err(e)) => {
                println!("❌ Error: {}", e);
            }
            Err(_) => {
                let duration = start.elapsed();
                println!("❌ TIMEOUT after {:.1}s", duration.as_secs_f64());
                println!("\nThis confirms multi-turn execution is broken!");
                println!("The agent likely gets stuck in:");
                println!("1. Infinite loop in execute_multi_turn_loop");
                println!("2. Waiting forever for LLM response");
                println!("3. Deadlock in conversation mutex");
            }
        }
    } else {
        println!("❌ No Ollama provider available");
    }

    Ok(())
}