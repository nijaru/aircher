/// Performance bottleneck analysis - find where the 8+ seconds are spent
use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;

use aircher::auth::AuthManager;
use aircher::client::local::LocalClient;
use aircher::config::ConfigManager;
use aircher::providers::ProviderManager;

#[tokio::main]
async fn main() -> Result<()> {
    println!("⏱️  PERFORMANCE BOTTLENECK ANALYSIS");
    println!("====================================\n");

    // Phase 1: Initialization timing
    println!("📊 Phase 1: Initialization");
    let init_start = Instant::now();

    let config_start = Instant::now();
    let config = ConfigManager::default();
    println!("   Config: {:?}", config_start.elapsed());

    let auth_start = Instant::now();
    let auth_manager = Arc::new(AuthManager::new()?);
    println!("   Auth: {:?}", auth_start.elapsed());

    let provider_start = Instant::now();
    let provider_manager = Arc::new(ProviderManager::new(&config, auth_manager.clone()).await?);
    println!("   Provider Manager: {:?}", provider_start.elapsed());

    let client_start = Instant::now();
    let mut client = LocalClient::new(&config, auth_manager, provider_manager.clone()).await?;
    println!("   Client Creation: {:?}", client_start.elapsed());

    let session_start = Instant::now();
    client.init_session().await?;
    println!("   Session Init: {:?}", session_start.elapsed());

    println!("   📊 Total Init: {:?}", init_start.elapsed());

    // Phase 2: Direct Ollama API timing
    println!("\n📊 Phase 2: Direct Ollama Communication");
    let ollama_start = Instant::now();

    let client_http = reqwest::Client::new();
    let payload = serde_json::json!({
        "model": "gpt-oss",
        "messages": [{"role": "user", "content": "Say 'test'"}],
        "stream": false,
        "options": {"num_predict": 10}
    });

    let request_start = Instant::now();
    match client_http.post("http://localhost:11434/api/chat")
        .json(&payload)
        .send()
        .await {
        Ok(response) => {
            println!("   HTTP Request: {:?}", request_start.elapsed());
            let text_start = Instant::now();
            let _text = response.text().await?;
            println!("   Response Parse: {:?}", text_start.elapsed());
        }
        Err(e) => println!("   ❌ Ollama error: {}", e),
    }
    println!("   📊 Total Ollama Direct: {:?}", ollama_start.elapsed());

    // Phase 3: Provider abstraction timing
    println!("\n📊 Phase 3: Provider Abstraction");
    if let Some(provider) = provider_manager.get_provider("ollama") {
        let provider_test_start = Instant::now();

        let messages = vec![aircher::providers::Message {
            id: "test".to_string(),
            role: aircher::providers::MessageRole::User,
            content: "Say 'quick'".to_string(),
            timestamp: chrono::Utc::now(),
            tokens_used: None,
            cost: None,
        }];

        let request = aircher::providers::ChatRequest {
            messages,
            model: "gpt-oss".to_string(),
            temperature: Some(0.1),
            max_tokens: Some(10),
            stream: false,
            tools: None,
        };

        let chat_start = Instant::now();
        match provider.chat(&request).await {
            Ok(_) => println!("   Provider.chat(): {:?}", chat_start.elapsed()),
            Err(e) => println!("   ❌ Provider error: {}", e),
        }
        println!("   📊 Total Provider: {:?}", provider_test_start.elapsed());
    }

    // Phase 4: Full agent pipeline timing (detailed)
    println!("\n📊 Phase 4: Full Agent Pipeline (Detailed)");

    // Test simple message
    let simple_start = Instant::now();
    println!("   Testing: 'What is 1+1?'");

    match client.send_message("What is 1+1?").await {
        Ok(response) => {
            let duration = simple_start.elapsed();
            println!("   ✅ Response received: {:?}", duration);
            println!("   Content length: {} chars", response.content.len());
            println!("   Tool calls: {}", response.tool_calls.len());
        }
        Err(e) => println!("   ❌ Error: {}", e),
    }

    // Phase 5: Intelligence system overhead
    println!("\n📊 Phase 5: Intelligence System Overhead");

    // Test with minimal request to isolate intelligence overhead
    let intel_start = Instant::now();
    match client.send_message("hi").await {
        Ok(_) => {
            println!("   Minimal request ('hi'): {:?}", intel_start.elapsed());
        }
        Err(e) => println!("   ❌ Error: {}", e),
    }

    // Phase 6: Tool execution timing
    println!("\n📊 Phase 6: Tool Execution Timing");

    let tool_start = Instant::now();
    match client.send_message("Create /tmp/perf_test.txt with 'test'").await {
        Ok(response) => {
            println!("   Tool request: {:?}", tool_start.elapsed());
            if std::path::Path::new("/tmp/perf_test.txt").exists() {
                println!("   ✅ Tool executed successfully");
                let _ = std::fs::remove_file("/tmp/perf_test.txt");
            } else {
                println!("   ⚠️ Tool may not have executed");
            }
        }
        Err(e) => println!("   ❌ Error: {}", e),
    }

    // Phase 7: Context manager overhead
    println!("\n📊 Phase 7: Breakdown Analysis");

    // Test requests of increasing complexity
    let sizes = vec![
        ("Tiny", "1"),
        ("Small", "What is 2+2?"),
        ("Medium", "Explain how to write a function"),
        ("Large", "Write a detailed explanation of async programming in Rust with examples"),
    ];

    for (label, request) in sizes {
        let start = Instant::now();
        match client.send_message(request).await {
            Ok(response) => {
                let duration = start.elapsed();
                println!("   {} request ({} chars): {:?}", label, request.len(), duration);
                println!("      Response: {} chars", response.content.len());
            }
            Err(e) => println!("   {} failed: {}", label, e),
        }
    }

    println!("\n🎯 BOTTLENECK ANALYSIS COMPLETE");

    Ok(())
}