/// Test very simple tool call that definitely won't trigger orchestration
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

use aircher::auth::AuthManager;
use aircher::client::local::LocalClient;
use aircher::config::ConfigManager;
use aircher::providers::ProviderManager;

#[tokio::main]
async fn main() -> Result<()> {
    println!("📝 SIMPLE TOOL CALL TEST");
    println!("=========================\n");

    // Create test client
    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let provider_manager = Arc::new(ProviderManager::new(&config, auth_manager.clone()).await?);

    let mut client = LocalClient::new(&config, auth_manager, provider_manager).await?;
    client.init_session().await?;

    println!("✅ Client created and session initialized");

    // Test 1: Very simple request that should NOT trigger orchestration
    println!("\n1. Testing simple write_file request...");
    let simple_request = "Write 'hello world' to /tmp/simple.txt";

    match tokio::time::timeout(
        Duration::from_secs(30),
        client.send_message(simple_request)
    ).await {
        Ok(Ok(response)) => {
            println!("   📝 Response: {}", response.content.chars().take(200).collect::<String>());
            println!("   🛠️ Tool calls received: {}", response.tool_calls.len());

            for (i, tool_call) in response.tool_calls.iter().enumerate() {
                println!("   🔧 Tool call {}: {}", i + 1, tool_call.name);
                println!("      Status: {:?}", tool_call.status);
            }

            // Check if file was created
            if std::path::Path::new("/tmp/simple.txt").exists() {
                println!("   ✅ File was created!");
                if let Ok(content) = std::fs::read_to_string("/tmp/simple.txt") {
                    println!("   📄 Content: '{}'", content.trim());
                }
                let _ = std::fs::remove_file("/tmp/simple.txt");
            } else {
                println!("   ❌ File was not created");
            }
        }
        Ok(Err(e)) => {
            println!("   ❌ Request error: {}", e);
        }
        Err(_) => {
            println!("   ⏰ Request timed out");
        }
    }

    // Test 2: Even simpler request
    println!("\n2. Testing read current directory...");
    let read_request = "List files in current directory";

    match tokio::time::timeout(
        Duration::from_secs(20),
        client.send_message(read_request)
    ).await {
        Ok(Ok(response)) => {
            println!("   📝 Response: {}", response.content.chars().take(200).collect::<String>());
            println!("   🛠️ Tool calls: {}", response.tool_calls.len());
            for tool_call in &response.tool_calls {
                println!("   🔧 Tool: {} - {:?}", tool_call.name, tool_call.status);
            }
        }
        Ok(Err(e)) => {
            println!("   ❌ Request error: {}", e);
        }
        Err(_) => {
            println!("   ⏰ Request timed out");
        }
    }

    println!("\n🎯 SIMPLE TOOL TEST COMPLETE");

    Ok(())
}