/// Test tool calling with explicit tool request to see LLM behavior
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

use aircher::auth::AuthManager;
use aircher::client::{local::LocalClient, AgentClient};
use aircher::config::ConfigManager;
use aircher::providers::ProviderManager;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔧 TOOL CALLING SPECIFIC TEST");
    println!("==============================\n");

    // Create test client
    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let provider_manager = Arc::new(ProviderManager::new(&config, auth_manager.clone()).await?);

    let mut client = LocalClient::new(&config, auth_manager, provider_manager).await?;
    client.init_session().await?;

    println!("✅ Client created and session initialized");

    // Test 1: Very explicit tool request
    println!("\n1. Testing explicit tool calling request...");
    let tool_request = "Please use the write_file tool to create a file at /tmp/test_explicit.txt with content 'Tool calling test'. You must use tools to complete this task.";

    match tokio::time::timeout(
        Duration::from_secs(30),
        client.send_message(tool_request)
    ).await {
        Ok(Ok(response)) => {
            println!("   📝 Response: {}", response.content.chars().take(300).collect::<String>());
            println!("   🛠️ Tool calls received: {}", response.tool_calls.len());

            for (i, tool_call) in response.tool_calls.iter().enumerate() {
                println!("   🔧 Tool call {}: {}", i + 1, tool_call.name);
                println!("      Status: {:?}", tool_call.status);
                if let Some(result) = &tool_call.result {
                    println!("      Result: {}", serde_json::to_string_pretty(result)?);
                }
            }

            // Check if file was actually created
            if std::path::Path::new("/tmp/test_explicit.txt").exists() {
                println!("   ✅ File was created successfully!");
                if let Ok(content) = std::fs::read_to_string("/tmp/test_explicit.txt") {
                    println!("   📄 File content: '{}'", content.trim());
                }
                // Clean up
                let _ = std::fs::remove_file("/tmp/test_explicit.txt");
            } else {
                println!("   ❌ File was not created");
            }
        }
        Ok(Err(e)) => {
            println!("   ❌ Tool request error: {}", e);
        }
        Err(_) => {
            println!("   ⏰ Tool request timed out");
        }
    }

    // Test 2: Check what tools are available
    println!("\n2. Checking available tools...");
    match client.initialize().await {
        Ok(agent_info) => {
            println!("   🔧 Available tools:");
            for tool in &agent_info.available_tools {
                println!("      - {}", tool);
            }
            println!("   📊 Total tools: {}", agent_info.available_tools.len());
        }
        Err(e) => {
            println!("   ❌ Failed to get agent info: {}", e);
        }
    }

    // Test 3: Direct tool execution test
    println!("\n3. Testing direct tool execution...");
    match client.execute_tool(
        "write_file",
        serde_json::json!({
            "path": "/tmp/test_direct.txt",
            "content": "Direct tool execution test"
        })
    ).await {
        Ok(result) => {
            println!("   ✅ Direct tool execution succeeded");
            println!("   📄 Result: {}", serde_json::to_string_pretty(&result)?);

            // Verify file exists
            if std::path::Path::new("/tmp/test_direct.txt").exists() {
                println!("   ✅ File created by direct execution");
                let _ = std::fs::remove_file("/tmp/test_direct.txt");
            }
        }
        Err(e) => {
            println!("   ❌ Direct tool execution failed: {}", e);
        }
    }

    println!("\n🎯 TOOL CALLING TEST COMPLETE");

    Ok(())
}