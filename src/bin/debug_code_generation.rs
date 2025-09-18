/// Debug what's happening with code generation
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
    println!("🔍 DEBUGGING CODE GENERATION QUALITY");
    println!("===================================\n");

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
        println!("📝 Testing fibonacci generation...");

        let request = "Create a Rust function called calculate_fibonacci that takes a number and returns the fibonacci sequence up to that number. Include proper error handling with Result<Vec<u64>, String> and comprehensive tests.";

        println!("Request: {}", request);
        println!("\n⏱️ Processing...");

        match agent.process_message(request, provider, "gpt-oss").await {
            Ok((response, status_messages)) => {
                println!("\n📊 RESPONSE ANALYSIS:");
                println!("Response length: {} characters", response.len());
                println!("Status messages: {} items", status_messages.len());

                println!("\n📋 Status Messages:");
                for (i, msg) in status_messages.iter().enumerate() {
                    println!("  {}: {}", i + 1, msg);
                }

                println!("\n📄 FULL RESPONSE:");
                println!("---START---");
                println!("{}", response);
                println!("---END---");

                // Analyze the response content
                let has_function = response.contains("fn ") || response.contains("function");
                let has_fibonacci = response.to_lowercase().contains("fibonacci");
                let has_result_type = response.contains("Result<") || response.contains("Result ");
                let has_error_handling = response.contains("Error") || response.contains("Err(");
                let has_tests = response.contains("#[test]") || response.contains("mod test");
                let has_code_block = response.contains("```");

                println!("\n🔍 CONTENT ANALYSIS:");
                println!("  Function syntax: {}", has_function);
                println!("  Mentions fibonacci: {}", has_fibonacci);
                println!("  Result type: {}", has_result_type);
                println!("  Error handling: {}", has_error_handling);
                println!("  Tests: {}", has_tests);
                println!("  Code blocks: {}", has_code_block);

                if response.len() < 500 {
                    println!("\n❌ PROBLEM: Response too short for comprehensive code generation");
                    println!("Expected: 1000+ characters for function + error handling + tests");
                    println!("Actual: {} characters", response.len());
                }

                if response.len() > 1000 && has_function && has_fibonacci {
                    println!("\n✅ SUCCESS: Generated comprehensive code response");
                } else {
                    println!("\n❌ FAILURE: Poor code generation quality");

                    // Debug potential issues
                    if !has_code_block {
                        println!("  Issue: No code blocks (```) in response");
                    }
                    if !has_function {
                        println!("  Issue: No function syntax detected");
                    }
                    if !has_fibonacci {
                        println!("  Issue: Doesn't mention fibonacci");
                    }
                }
            }
            Err(e) => {
                println!("❌ Error processing request: {}", e);
                println!("Error chain: {:?}", e.chain().collect::<Vec<_>>());
            }
        }
    } else {
        println!("❌ No Ollama provider available");
    }

    Ok(())
}