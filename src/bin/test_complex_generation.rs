/// Test why complex code generation is failing
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
    println!("🔍 DEBUGGING COMPLEX CODE GENERATION FAILURE");
    println!("============================================\n");

    // Enable debug logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

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

    println!("Testing with complex request that should generate code...\n");

    if let Some(provider) = provider_manager.get_provider("ollama") {
        // Pre-warm the model with keep_alive
        println!("📦 Pre-warming model with keep_alive...");
        let warmup_request = aircher::providers::ChatRequest {
            messages: vec![aircher::providers::ChatMessage {
                role: "user".to_string(),
                content: "hello".to_string(),
            }],
            model: Some("gpt-oss".to_string()),
            temperature: Some(0.7),
            max_tokens: None,
            stream: false,
            tools: None,
            tool_choice: None,
        };

        // Add keep_alive to the request (if Ollama supports it)
        match provider.chat(&warmup_request).await {
            Ok(response) => println!("✅ Model pre-warmed. Response: {}", response.content),
            Err(e) => println!("⚠️  Pre-warm failed: {}", e),
        }

        println!("\n📝 Sending complex code generation request...");

        // Try the same request that failed in validation
        let complex_request = "Create a new Rust function called calculate_fibonacci that takes a number and returns the fibonacci sequence up to that number. Include error handling and tests.";

        match agent.process_message(complex_request, provider, "gpt-oss").await {
            Ok((response, status)) => {
                println!("\n📊 Response Analysis:");
                println!("   Length: {} chars", response.len());
                println!("   Status messages: {:?}", status);

                // Check for expected content
                let has_function = response.contains("fn ") || response.contains("def ") || response.contains("function ");
                let has_fibonacci_mention = response.to_lowercase().contains("fibonacci");
                let has_code_block = response.contains("```");
                let has_error_handling = response.contains("Result") || response.contains("Error") || response.contains("?");
                let has_tests = response.contains("#[test]") || response.contains("mod tests");

                println!("\n✅ Content Checks:");
                println!("   Has function syntax: {}", has_function);
                println!("   Mentions fibonacci: {}", has_fibonacci_mention);
                println!("   Has code blocks: {}", has_code_block);
                println!("   Has error handling: {}", has_error_handling);
                println!("   Has tests: {}", has_tests);

                println!("\n📄 Full Response:");
                println!("---");
                println!("{}", response);
                println!("---");

                if has_function && has_fibonacci_mention {
                    println!("\n✅ SUCCESS: Generated proper code!");
                } else {
                    println!("\n❌ FAILURE: Response doesn't contain expected code");

                    // Debug: Check if it's trying to use tools
                    if response.contains("tool") || response.contains("function_call") {
                        println!("⚠️  Model may be trying to call tools instead of generating code");
                    }
                }
            }
            Err(e) => {
                println!("❌ Error: {}", e);
                println!("   Error chain: {:?}", e.chain().collect::<Vec<_>>());
            }
        }
    } else {
        println!("❌ No Ollama provider available");
        println!("\n🔍 Checking available providers...");

        for provider_name in ["anthropic", "openai", "gemini", "ollama"] {
            if provider_manager.get_provider(provider_name).is_some() {
                println!("   ✅ {} available", provider_name);
            } else {
                println!("   ❌ {} not available", provider_name);
            }
        }
    }

    Ok(())
}