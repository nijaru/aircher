/// Simple test focusing only on code generation quality without complex intelligence
use anyhow::Result;
use std::sync::Arc;

use aircher::auth::AuthManager;
use aircher::config::ConfigManager;
use aircher::providers::ProviderManager;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 SIMPLE CODE GENERATION TEST");
    println!("===============================\n");

    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let provider_manager = Arc::new(ProviderManager::new(&config, auth_manager.clone()).await?);

    if let Some(provider) = provider_manager.get_provider("ollama") {
        println!("📝 Testing fibonacci generation directly with provider...");

        // Create a simple chat request directly to the provider
        let request = aircher::providers::ChatRequest {
            model: "gpt-oss".to_string(),
            messages: vec![aircher::providers::Message {
                id: uuid::Uuid::new_v4().to_string(),
                role: aircher::providers::MessageRole::User,
                content: "Create a Rust function called calculate_fibonacci that takes a number and returns the fibonacci sequence up to that number. Include proper error handling with Result<Vec<u64>, String> and comprehensive tests.".to_string(),
                timestamp: chrono::Utc::now(),
                tokens_used: None,
                cost: None,
            }],
            tools: None, // No tools needed for this test
            temperature: Some(0.7),
            max_tokens: Some(2000), // Ensure we allow enough tokens for comprehensive response
            stream: false,
        };

        println!("Request: {}", request.messages[0].content);
        println!("\n⏱️ Processing...");

        match provider.chat(&request).await {
            Ok(response) => {
                println!("\n📊 RESPONSE ANALYSIS:");
                println!("Response length: {} characters", response.content.len());
                println!("Finish reason: {:?}", response.finish_reason);
                println!("Token usage: {}", response.tokens_used);

                println!("\n📄 FULL RESPONSE:");
                println!("---START---");
                println!("{}", response.content);
                println!("---END---");

                // Analyze the response content
                let has_function = response.content.contains("fn ") || response.content.contains("function");
                let has_fibonacci = response.content.to_lowercase().contains("fibonacci");
                let has_result_type = response.content.contains("Result<") || response.content.contains("Result ");
                let has_error_handling = response.content.contains("Error") || response.content.contains("Err(");
                let has_tests = response.content.contains("#[test]") || response.content.contains("mod test");
                let has_code_block = response.content.contains("```");

                println!("\n🔍 CONTENT ANALYSIS:");
                println!("  Function syntax: {}", has_function);
                println!("  Mentions fibonacci: {}", has_fibonacci);
                println!("  Result type: {}", has_result_type);
                println!("  Error handling: {}", has_error_handling);
                println!("  Tests: {}", has_tests);
                println!("  Code blocks: {}", has_code_block);

                if response.content.len() < 500 {
                    println!("\n❌ PROBLEM: Response too short for comprehensive code generation");
                    println!("Expected: 1000+ characters for function + error handling + tests");
                    println!("Actual: {} characters", response.content.len());
                }

                if response.content.len() > 1000 && has_function && has_fibonacci {
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
                println!("❌ Error with provider request: {}", e);
                println!("Error chain: {:?}", e.chain().collect::<Vec<_>>());
            }
        }
    } else {
        println!("❌ No Ollama provider available");
    }

    Ok(())
}