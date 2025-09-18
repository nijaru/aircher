/// Debug why we get empty responses when tools aren't provided
use anyhow::Result;
use std::sync::Arc;

use aircher::auth::AuthManager;
use aircher::config::ConfigManager;
use aircher::providers::{ProviderManager, ChatRequest, Message, MessageRole};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 DEBUGGING EMPTY RESPONSE ISSUE");
    println!("=================================\n");

    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let provider_manager = Arc::new(ProviderManager::new(&config, auth_manager.clone()).await?);

    if let Some(provider) = provider_manager.get_provider("ollama") {
        // Test 1: Direct request WITH tools (should work)
        println!("Test 1: Request WITH tools");
        let request_with_tools = ChatRequest {
            model: "gpt-oss".to_string(),
            messages: vec![
                Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: MessageRole::System,
                    content: "You are a helpful assistant.".to_string(),
                    timestamp: chrono::Utc::now(),
                    tokens_used: None,
                    cost: None,
                },
                Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: MessageRole::User,
                    content: "Create a simple hello world function in Rust".to_string(),
                    timestamp: chrono::Utc::now(),
                    tokens_used: None,
                    cost: None,
                },
            ],
            tools: Some(vec![]), // Empty tools list
            temperature: Some(0.7),
            max_tokens: Some(2000),
            stream: false,
        };

        match provider.chat(&request_with_tools).await {
            Ok(response) => {
                println!("✅ Response WITH tools:");
                println!("  Length: {} characters", response.content.len());
                println!("  First 200 chars: {}", response.content.chars().take(200).collect::<String>());
                println!("  Tool calls: {:?}", response.tool_calls);
            }
            Err(e) => println!("❌ Error with tools: {}", e),
        }

        println!("\n{}\n", "=".repeat(50));

        // Test 2: Direct request WITHOUT tools (might be empty)
        println!("Test 2: Request WITHOUT tools");
        let request_no_tools = ChatRequest {
            model: "gpt-oss".to_string(),
            messages: vec![
                Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: MessageRole::System,
                    content: "You are a helpful assistant. Generate code directly.".to_string(),
                    timestamp: chrono::Utc::now(),
                    tokens_used: None,
                    cost: None,
                },
                Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: MessageRole::User,
                    content: "Create a simple hello world function in Rust".to_string(),
                    timestamp: chrono::Utc::now(),
                    tokens_used: None,
                    cost: None,
                },
            ],
            tools: None, // NO tools
            temperature: Some(0.7),
            max_tokens: Some(2000),
            stream: false,
        };

        match provider.chat(&request_no_tools).await {
            Ok(response) => {
                println!("✅ Response WITHOUT tools:");
                println!("  Length: {} characters", response.content.len());
                if response.content.is_empty() {
                    println!("  ⚠️ EMPTY RESPONSE!");
                    println!("  Tokens used: {}", response.tokens_used);
                    println!("  Finish reason: {:?}", response.finish_reason);
                } else {
                    println!("  First 200 chars: {}", response.content.chars().take(200).collect::<String>());
                }
                println!("  Tool calls: {:?}", response.tool_calls);
            }
            Err(e) => println!("❌ Error without tools: {}", e),
        }

        println!("\n{}\n", "=".repeat(50));

        // Test 3: Try with explicit instruction to not use tools
        println!("Test 3: Explicit NO TOOLS instruction");
        let request_explicit = ChatRequest {
            model: "gpt-oss".to_string(),
            messages: vec![
                Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: MessageRole::System,
                    content: "You are a helpful assistant. Generate code directly without using any tools.".to_string(),
                    timestamp: chrono::Utc::now(),
                    tokens_used: None,
                    cost: None,
                },
                Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: MessageRole::User,
                    content: "Create a simple hello world function in Rust. Do not use tools, just write the code.".to_string(),
                    timestamp: chrono::Utc::now(),
                    tokens_used: None,
                    cost: None,
                },
            ],
            tools: None,
            temperature: Some(0.7),
            max_tokens: Some(2000),
            stream: false,
        };

        match provider.chat(&request_explicit).await {
            Ok(response) => {
                println!("✅ Response with EXPLICIT no-tools instruction:");
                println!("  Length: {} characters", response.content.len());
                if response.content.is_empty() {
                    println!("  ⚠️ EMPTY RESPONSE!");
                    println!("  Tokens used: {}", response.tokens_used);
                    println!("  Finish reason: {:?}", response.finish_reason);
                } else {
                    println!("  First 200 chars: {}", response.content.chars().take(200).collect::<String>());
                }
                println!("  Tool calls: {:?}", response.tool_calls);
            }
            Err(e) => println!("❌ Error with explicit instruction: {}", e),
        }
    } else {
        println!("❌ No Ollama provider available");
    }

    Ok(())
}