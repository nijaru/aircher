/// Test direct provider communication to isolate timeout issues
use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};

use aircher::auth::AuthManager;
use aircher::config::ConfigManager;
use aircher::providers::{ProviderManager, ChatRequest, Message, MessageRole};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔌 PROVIDER DIRECT TEST");
    println!("=======================\n");

    // Create managers
    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let provider_manager = Arc::new(ProviderManager::new(&config, auth_manager.clone()).await?);

    // Test 1: Get provider
    println!("1. Getting Ollama provider...");
    let start = Instant::now();
    match provider_manager.get_provider("ollama") {
        Some(provider) => {
            println!("   ✅ Provider obtained ({:?})", start.elapsed());

            // Test 2: Check if it supports tools
            println!("   🔧 Tools supported: {}", provider.supports_tools());

            // Test 3: Simple chat request
            println!("\n2. Testing simple chat request...");
            let start = Instant::now();

            let messages = vec![Message {
                id: "test-1".to_string(),
                role: MessageRole::User,
                content: "Say 'hello' - just one word".to_string(),
                timestamp: chrono::Utc::now(),
                tokens_used: None,
                cost: None,
            }];

            let request = ChatRequest {
                messages,
                model: "gpt-oss".to_string(),
                temperature: Some(0.1),
                max_tokens: Some(10),
                stream: false,
                tools: None,
            };

            // Set timeout explicitly
            match tokio::time::timeout(Duration::from_secs(30), provider.chat(&request)).await {
                Ok(Ok(response)) => {
                    println!("   ✅ Chat response received ({:?})", start.elapsed());
                    println!("   📝 Content: {}", response.content.chars().take(100).collect::<String>());
                    println!("   🛠️ Tool calls: {:?}", response.tool_calls.as_ref().map(|t| t.len()));
                }
                Ok(Err(e)) => {
                    println!("   ❌ Chat error: {}", e);
                }
                Err(_) => {
                    println!("   ⏰ Chat timed out after 30 seconds");
                }
            }
        }
        None => {
            println!("   ❌ No ollama provider found");
        }
    }

    // Test 4: Simple Ollama API call via reqwest
    println!("\n3. Testing direct Ollama API call...");
    let start = Instant::now();

    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "model": "gpt-oss",
        "prompt": "Say hello",
        "stream": false,
        "options": {
            "num_predict": 10
        }
    });

    match tokio::time::timeout(
        Duration::from_secs(15),
        client.post("http://localhost:11434/api/generate")
            .json(&payload)
            .send()
    ).await {
        Ok(Ok(response)) => {
            println!("   ✅ Direct API call succeeded ({:?})", start.elapsed());
            if response.status().is_success() {
                if let Ok(text) = response.text().await {
                    println!("   📝 Raw response: {}", text.chars().take(200).collect::<String>());
                }
            } else {
                println!("   ⚠️ HTTP status: {}", response.status());
            }
        }
        Ok(Err(e)) => {
            println!("   ❌ Direct API error: {}", e);
        }
        Err(_) => {
            println!("   ⏰ Direct API timed out");
        }
    }

    Ok(())
}