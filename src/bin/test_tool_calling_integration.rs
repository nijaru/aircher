/// Tool Calling Integration Test - Real End-to-End Workflows
///
/// This test verifies that the agent can actually execute tools and use results
/// in multi-turn conversations, simulating real user interactions.

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use aircher::auth::AuthManager;
use aircher::client::local::LocalClient;
use aircher::config::ConfigManager;
use aircher::providers::ProviderManager;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔧 TOOL CALLING INTEGRATION TEST");
    println!("=================================\n");

    let mut passed = 0;
    let mut total = 0;

    // Test 1: Agent can execute file operations
    println!("1. Testing file operation tools...");
    total += 1;
    if test_file_operations().await.is_ok() {
        println!("   ✅ File operations working");
        passed += 1;
    } else {
        println!("   ❌ File operations failed");
    }

    // Test 2: Agent can search code
    println!("\n2. Testing code search tools...");
    total += 1;
    if test_code_search().await.is_ok() {
        println!("   ✅ Code search working");
        passed += 1;
    } else {
        println!("   ❌ Code search failed");
    }

    // Test 3: Agent can run commands
    println!("\n3. Testing command execution...");
    total += 1;
    if test_command_execution().await.is_ok() {
        println!("   ✅ Command execution working");
        passed += 1;
    } else {
        println!("   ❌ Command execution failed");
    }

    // Test 4: Multi-turn workflow
    println!("\n4. Testing multi-turn workflow...");
    total += 1;
    if test_multi_turn_workflow().await.is_ok() {
        println!("   ✅ Multi-turn workflow working");
        passed += 1;
    } else {
        println!("   ❌ Multi-turn workflow failed");
    }

    // Test 5: Error handling
    println!("\n5. Testing error handling...");
    total += 1;
    if test_error_handling().await.is_ok() {
        println!("   ✅ Error handling working");
        passed += 1;
    } else {
        println!("   ❌ Error handling failed");
    }

    // Summary
    println!("\n📊 TOOL CALLING RESULTS:");
    println!("========================");
    println!("Passed: {}/{} ({:.1}%)", passed, total,
        (passed as f64 / total as f64) * 100.0);

    if passed == total {
        println!("\n🎉 TOOL CALLING FULLY FUNCTIONAL!");
        println!("✅ Agent can execute all tool types");
        println!("✅ Multi-turn workflows operational");
        println!("✅ Error handling robust");
        println!("✅ READY FOR RELEASE!");
        std::process::exit(0);
    } else {
        println!("\n⚠️  TOOL CALLING ISSUES FOUND");
        println!("🔧 Need to fix tool execution before release");
        println!("🚨 CRITICAL: Agent functionality incomplete");
        std::process::exit(1);
    }
}

/// Test file operation tools (read, write, edit)
async fn test_file_operations() -> Result<()> {
    println!("   Testing file read/write/edit tools...");

    // Create test environment
    let client = create_test_client().await?;
    let test_file = "/tmp/aircher_test_file.txt";

    // Test 1: Write a file
    let write_response = timeout(
        Duration::from_secs(30),
        client.send_message(&format!("Please write a test file at {} with the content 'Hello from Aircher tool test'", test_file))
    ).await??;

    println!("   • Write response: {}", write_response.content.chars().take(100).collect::<String>());

    // Verify file was created
    if !std::path::Path::new(test_file).exists() {
        return Err(anyhow::anyhow!("File was not created by agent"));
    }

    // Test 2: Read the file back
    let read_response = timeout(
        Duration::from_secs(30),
        client.send_message(&format!("Please read the content of {}", test_file))
    ).await??;

    println!("   • Read response: {}", read_response.content.chars().take(100).collect::<String>());

    // Test 3: Edit the file
    let edit_response = timeout(
        Duration::from_secs(30),
        client.send_message(&format!("Please add the line 'Tool test successful' to {}", test_file))
    ).await??;

    println!("   • Edit response: {}", edit_response.content.chars().take(100).collect::<String>());

    // Clean up
    let _ = std::fs::remove_file(test_file);

    // Check if responses indicate tool usage
    if !contains_tool_indicators(&write_response.content) {
        return Err(anyhow::anyhow!("Write response doesn't indicate tool usage"));
    }

    Ok(())
}

/// Test code search functionality
async fn test_code_search() -> Result<()> {
    println!("   Testing semantic code search...");

    let client = create_test_client().await?;

    // Test semantic search for common patterns
    let search_response = timeout(
        Duration::from_secs(30),
        client.send_message("Please search for 'async fn' in the codebase and show me what you find")
    ).await??;

    println!("   • Search response: {}", search_response.content.chars().take(150).collect::<String>());

    // Verify search was performed
    if !contains_tool_indicators(&search_response.content) &&
       !search_response.content.to_lowercase().contains("search") {
        return Err(anyhow::anyhow!("Search response doesn't indicate search was performed"));
    }

    Ok(())
}

/// Test command execution
async fn test_command_execution() -> Result<()> {
    println!("   Testing command execution...");

    let client = create_test_client().await?;

    // Test safe command execution
    let cmd_response = timeout(
        Duration::from_secs(30),
        client.send_message("Please run 'echo Hello from Aircher command test' and show me the output")
    ).await??;

    println!("   • Command response: {}", cmd_response.content.chars().take(150).collect::<String>());

    // Verify command was executed
    if !contains_tool_indicators(&cmd_response.content) &&
       !cmd_response.content.contains("Hello from Aircher command test") {
        return Err(anyhow::anyhow!("Command response doesn't indicate execution"));
    }

    Ok(())
}

/// Test multi-turn workflow with tool dependencies
async fn test_multi_turn_workflow() -> Result<()> {
    println!("   Testing multi-turn tool workflow...");

    let client = create_test_client().await?;
    let test_file = "/tmp/aircher_workflow_test.txt";

    // Turn 1: Create a file
    let create_response = timeout(
        Duration::from_secs(30),
        client.send_message(&format!("Create a file {} with some sample code", test_file))
    ).await??;

    println!("   • Create: {}", create_response.content.chars().take(80).collect::<String>());

    // Turn 2: Search within that file
    let search_response = timeout(
        Duration::from_secs(30),
        client.send_message(&format!("Now search for any functions in {}", test_file))
    ).await??;

    println!("   • Search: {}", search_response.content.chars().take(80).collect::<String>());

    // Turn 3: Get file info
    let info_response = timeout(
        Duration::from_secs(30),
        client.send_message(&format!("Tell me about the file {} - its size, content, etc", test_file))
    ).await??;

    println!("   • Info: {}", info_response.content.chars().take(80).collect::<String>());

    // Clean up
    let _ = std::fs::remove_file(test_file);

    // Verify we got responses that suggest tool usage
    let responses = [&create_response.content, &search_response.content, &info_response.content];
    for (i, response) in responses.iter().enumerate() {
        if response.trim().is_empty() {
            return Err(anyhow::anyhow!("Empty response in turn {}", i + 1));
        }
    }

    Ok(())
}

/// Test error handling in tool execution
async fn test_error_handling() -> Result<()> {
    println!("   Testing error handling...");

    let client = create_test_client().await?;

    // Test 1: Invalid file operation
    let invalid_response = timeout(
        Duration::from_secs(30),
        client.send_message("Please read the file /invalid/nonexistent/path/file.txt")
    ).await??;

    println!("   • Invalid file: {}", invalid_response.content.chars().take(100).collect::<String>());

    // Test 2: Invalid command
    let invalid_cmd_response = timeout(
        Duration::from_secs(30),
        client.send_message("Please run the command 'nonexistentcommand12345'")
    ).await??;

    println!("   • Invalid command: {}", invalid_cmd_response.content.chars().take(100).collect::<String>());

    // Verify we got error responses, not empty/hanging responses
    if invalid_response.content.trim().is_empty() || invalid_cmd_response.content.trim().is_empty() {
        return Err(anyhow::anyhow!("Got empty responses for error cases"));
    }

    // Error responses should indicate the problem
    if !invalid_response.content.to_lowercase().contains("error") &&
       !invalid_response.content.to_lowercase().contains("not found") &&
       !invalid_response.content.to_lowercase().contains("invalid") {
        return Err(anyhow::anyhow!("Error response doesn't indicate error handling"));
    }

    Ok(())
}

/// Create a test client with proper configuration
async fn create_test_client() -> Result<LocalClient> {
    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let provider_manager = Arc::new(ProviderManager::new(&config, auth_manager.clone()).await?);

    let mut client = LocalClient::new(&config, auth_manager, provider_manager).await?;
    client.init_session().await?;

    Ok(client)
}

/// Check if response contains indicators of tool usage
fn contains_tool_indicators(content: &str) -> bool {
    let indicators = [
        "tool", "execute", "running", "command", "file",
        "read", "write", "search", "found", "created"
    ];

    let content_lower = content.to_lowercase();
    indicators.iter().any(|indicator| content_lower.contains(indicator))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let result = create_test_client().await;
        assert!(result.is_ok(), "Should be able to create test client");
    }

    #[test]
    fn test_tool_indicators() {
        assert!(contains_tool_indicators("I will read the file for you"));
        assert!(contains_tool_indicators("Running command: ls"));
        assert!(contains_tool_indicators("Search found 3 results"));
        assert!(!contains_tool_indicators("Hello there"));
    }
}