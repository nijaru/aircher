/// Test single tool execution - verify basics work
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
    println!("🔍 TESTING SINGLE TOOL EXECUTION - REALITY CHECK");
    println!("================================================\n");

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
        println!("Test 1: Simple list files");
        println!("-------------------------\n");

        let request = "List the files in the src/agent/tools directory";
        println!("Request: {}\n", request);

        let start = std::time::Instant::now();
        match agent.process_message(request, provider, "gpt-oss").await {
            Ok((response, status_messages)) => {
                let duration = start.elapsed();
                println!("✅ Completed in {:.1}s", duration.as_secs_f64());
                println!("Response length: {} chars", response.len());
                println!("Status messages: {}", status_messages.len());

                // Check what we got
                if response.starts_with("Executed tools:") {
                    println!("❌ PROBLEM: Agent returned raw tool output");
                } else if response.contains(".rs") {
                    println!("✅ SUCCESS: Agent interpreted results");
                } else {
                    println!("⚠️ UNCLEAR: Response doesn't clearly list files");
                }

                println!("\nResponse preview:");
                println!("{}", response.chars().take(500).collect::<String>());
                if response.len() > 500 {
                    println!("... ({} more chars)", response.len() - 500);
                }
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }

        println!("\n\nTest 2: Read specific file");
        println!("---------------------------\n");

        let request2 = "Read the first 20 lines of src/agent/tools/mod.rs";
        println!("Request: {}\n", request2);

        let start = std::time::Instant::now();
        match agent.process_message(request2, provider, "gpt-oss").await {
            Ok((response, status_messages)) => {
                let duration = start.elapsed();
                println!("✅ Completed in {:.1}s", duration.as_secs_f64());
                println!("Response length: {} chars", response.len());

                // Analysis
                if response.starts_with("Executed tools:") {
                    println!("❌ PROBLEM: Raw tool output again");
                } else if response.contains("impl") || response.contains("pub") {
                    println!("✅ SUCCESS: Shows actual code content");
                } else {
                    println!("⚠️ UNCLEAR: Can't determine if content shown");
                }

                println!("\nFirst 300 chars:");
                println!("{}", response.chars().take(300).collect::<String>());
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }

        println!("\n\nTest 3: Non-tool request");
        println!("-------------------------\n");

        let request3 = "Write a simple hello world function in Rust";
        println!("Request: {}\n", request3);

        let start = std::time::Instant::now();
        match agent.process_message(request3, provider, "gpt-oss").await {
            Ok((response, status_messages)) => {
                let duration = start.elapsed();
                println!("✅ Completed in {:.1}s", duration.as_secs_f64());
                println!("Response length: {} chars", response.len());
                println!("Tool calls: {}", status_messages.len());

                if status_messages.is_empty() && response.len() > 100 {
                    println!("✅ SUCCESS: Generated code without tools");
                } else if !status_messages.is_empty() {
                    println!("❌ PROBLEM: Used tools for simple generation");
                } else if response.is_empty() {
                    println!("❌ CRITICAL: Empty response!");
                } else {
                    println!("⚠️ Response too short: {} chars", response.len());
                }

                if !response.is_empty() {
                    println!("\nGenerated code:");
                    println!("{}", response.chars().take(500).collect::<String>());
                }
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