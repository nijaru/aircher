/// Agent Pipeline Test - Test complete agent-to-tool execution
///
/// This test verifies the full pipeline: User → Agent → LLM → Tools → Results

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

use aircher::auth::AuthManager;
use aircher::client::local::LocalClient;
use aircher::config::ConfigManager;
use aircher::providers::ProviderManager;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔗 AGENT PIPELINE TEST");
    println!("=======================\n");

    // Create test client
    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let provider_manager = Arc::new(ProviderManager::new(&config, auth_manager.clone()).await?);

    let mut client = LocalClient::new(&config, auth_manager, provider_manager).await?;
    client.init_session().await?;

    println!("✅ Client created and session initialized");

    // Test 1: Simple message (no tools)
    println!("\n1. Testing simple conversation...");
    match tokio::time::timeout(
        Duration::from_secs(10),
        client.send_message("What is 2 + 2?")
    ).await {
        Ok(Ok(response)) => {
            println!("   ✅ Response: {}", response.content.chars().take(100).collect::<String>());
        }
        Ok(Err(e)) => {
            println!("   ❌ Response error: {}", e);
        }
        Err(_) => {
            println!("   ⚠️  Response timed out");
        }
    }

    // Test 2: Tool-requiring message (if Ollama supports tools)
    println!("\n2. Testing tool execution request...");
    match tokio::time::timeout(
        Duration::from_secs(15),
        client.send_message("Please create a test file at /tmp/agent_test.txt with the content 'Agent pipeline works!'")
    ).await {
        Ok(Ok(response)) => {
            println!("   📝 Response: {}", response.content.chars().take(200).collect::<String>());

            // Check if file was actually created
            if std::path::Path::new("/tmp/agent_test.txt").exists() {
                println!("   ✅ File was actually created!");

                // Read it back to verify
                if let Ok(content) = std::fs::read_to_string("/tmp/agent_test.txt") {
                    println!("   📄 File content: {}", content.trim());
                }

                // Clean up
                let _ = std::fs::remove_file("/tmp/agent_test.txt");
            } else {
                println!("   ⚠️  File was not created (tool calling may not be working)");
            }
        }
        Ok(Err(e)) => {
            println!("   ❌ Tool request error: {}", e);
        }
        Err(_) => {
            println!("   ⚠️  Tool request timed out");
        }
    }

    // Test 3: Multi-turn conversation
    println!("\n3. Testing multi-turn conversation...");
    match tokio::time::timeout(
        Duration::from_secs(10),
        client.send_message("Remember this number: 42")
    ).await {
        Ok(Ok(_)) => {
            // Second turn
            match tokio::time::timeout(
                Duration::from_secs(10),
                client.send_message("What number did I ask you to remember?")
            ).await {
                Ok(Ok(response)) => {
                    if response.content.contains("42") {
                        println!("   ✅ Multi-turn context maintained");
                    } else {
                        println!("   ⚠️  Context not maintained: {}", response.content.chars().take(100).collect::<String>());
                    }
                }
                Ok(Err(e)) => {
                    println!("   ❌ Second turn error: {}", e);
                }
                Err(_) => {
                    println!("   ⚠️  Second turn timed out");
                }
            }
        }
        Ok(Err(e)) => {
            println!("   ❌ First turn error: {}", e);
        }
        Err(_) => {
            println!("   ⚠️  First turn timed out");
        }
    }

    println!("\n🎯 PIPELINE TEST COMPLETE");
    println!("💡 Note: Tool execution depends on LLM understanding and calling tools correctly");

    Ok(())
}