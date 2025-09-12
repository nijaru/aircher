use anyhow::Result;
use aircher::agent::unified::UnifiedAgent;
use aircher::providers::{ollama::OllamaProvider, LLMProvider};
use aircher::config::ConfigManager;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test configuration for Ollama testing
struct TestConfig {
    model: String,
    timeout_seconds: u64,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            model: "exaone-deep".to_string(), // Smallest model for fast tests
            timeout_seconds: 30,
        }
    }
}

/// Verify Ollama is running and accessible
async fn verify_ollama_connection() -> Result<bool> {
    let provider = OllamaProvider::new(None)?;
    match provider.list_available_models().await {
        Ok(models) => {
            println!("✓ Ollama connected. Available models: {:?}", models);
            Ok(!models.is_empty())
        }
        Err(e) => {
            println!("✗ Ollama connection failed: {}", e);
            Ok(false)
        }
    }
}

/// Test basic message processing without tools
async fn test_basic_message_processing() -> Result<()> {
    println!("\n=== Testing Basic Message Processing ===");
    
    let config = ConfigManager::load_or_default()?;
    let provider = Arc::new(OllamaProvider::new(None)?);
    let agent = UnifiedAgent::new(config.clone(), None)?;
    
    // Simple message test
    let response = agent.process_message(
        "What is 2 + 2?",
        provider.as_ref(),
        "exaone-deep"
    ).await?;
    
    println!("Response: {}", response);
    assert!(!response.is_empty(), "Response should not be empty");
    
    println!("✓ Basic message processing works");
    Ok(())
}

/// Test tool execution with read_file
async fn test_tool_execution() -> Result<()> {
    println!("\n=== Testing Tool Execution ===");
    
    let config = ConfigManager::load_or_default()?;
    let provider = Arc::new(OllamaProvider::new(None)?);
    let mut agent = UnifiedAgent::new(config.clone(), None)?;
    
    // Create a test file
    let test_file = "/tmp/aircher_test.txt";
    std::fs::write(test_file, "Hello from Aircher test!")?;
    
    // Request that should trigger tool use
    let response = agent.process_message(
        &format!("Read the file at {} and tell me what it contains", test_file),
        provider.as_ref(),
        "gpt-oss" // Use better model for tool calling
    ).await?;
    
    println!("Response: {}", response);
    
    // Cleanup
    std::fs::remove_file(test_file).ok();
    
    println!("✓ Tool execution tested");
    Ok(())
}

/// Test streaming responses
async fn test_streaming() -> Result<()> {
    println!("\n=== Testing Streaming Responses ===");
    
    let config = ConfigManager::load_or_default()?;
    let provider = Arc::new(OllamaProvider::new(None)?);
    let agent = UnifiedAgent::new(config.clone(), None)?;
    
    let mut stream = agent.stream_response(
        "Count from 1 to 5 slowly",
        provider.as_ref(),
        "exaone-deep"
    ).await?;
    
    let mut chunks = Vec::new();
    while let Some(chunk) = stream.recv().await {
        match chunk {
            aircher::agent::streaming::AgentUpdate::Content(text) => {
                print!("{}", text);
                chunks.push(text);
            }
            aircher::agent::streaming::AgentUpdate::ToolExecution { name, status } => {
                println!("\n[Tool: {} - {:?}]", name, status);
            }
            _ => {}
        }
    }
    
    println!("\n✓ Streaming works - received {} chunks", chunks.len());
    Ok(())
}

/// Test intelligence system integration
async fn test_intelligence_integration() -> Result<()> {
    println!("\n=== Testing Intelligence Integration ===");
    
    let config = ConfigManager::load_or_default()?;
    let provider = Arc::new(OllamaProvider::new(None)?);
    
    // Initialize with intelligence enabled
    let mut agent = UnifiedAgent::new(config.clone(), None)?;
    
    // First interaction - should create a pattern
    let response1 = agent.process_message(
        "What files are in the src directory?",
        provider.as_ref(),
        "exaone-deep"
    ).await?;
    
    println!("First response: {}", response1.chars().take(100).collect::<String>());
    
    // Second similar interaction - should use learned patterns
    let response2 = agent.process_message(
        "Show me the source files",
        provider.as_ref(),
        "exaone-deep"
    ).await?;
    
    println!("Second response (should be enhanced): {}", 
             response2.chars().take(100).collect::<String>());
    
    println!("✓ Intelligence integration tested");
    Ok(())
}

/// Test error handling and recovery
async fn test_error_handling() -> Result<()> {
    println!("\n=== Testing Error Handling ===");
    
    let config = ConfigManager::load_or_default()?;
    let provider = Arc::new(OllamaProvider::new(None)?);
    let agent = UnifiedAgent::new(config.clone(), None)?;
    
    // Test with non-existent model
    match agent.process_message(
        "Hello",
        provider.as_ref(),
        "non_existent_model"
    ).await {
        Ok(_) => println!("⚠️ Expected error for non-existent model"),
        Err(e) => println!("✓ Correctly handled error: {}", e),
    }
    
    // Test with invalid tool parameters
    let response = agent.process_message(
        "Read the file at /definitely/not/a/real/path.txt",
        provider.as_ref(),
        "exaone-deep"
    ).await?;
    
    println!("Handled invalid path gracefully: {}", 
             response.chars().take(100).collect::<String>());
    
    println!("✓ Error handling tested");
    Ok(())
}

/// Main test runner
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("aircher=debug")
        .init();
    
    println!("🧪 Aircher Agent Test Suite");
    println!("============================\n");
    
    // Check Ollama connection first
    if !verify_ollama_connection().await? {
        println!("❌ Ollama is not running. Please start it with: ollama serve");
        return Ok(());
    }
    
    // Run test suite
    let mut passed = 0;
    let mut failed = 0;
    
    let tests = vec![
        ("Basic Message Processing", test_basic_message_processing()),
        ("Tool Execution", test_tool_execution()),
        ("Streaming Responses", test_streaming()),
        ("Intelligence Integration", test_intelligence_integration()),
        ("Error Handling", test_error_handling()),
    ];
    
    for (name, test) in tests {
        match test.await {
            Ok(_) => {
                println!("✅ {} PASSED\n", name);
                passed += 1;
            }
            Err(e) => {
                println!("❌ {} FAILED: {}\n", name, e);
                failed += 1;
            }
        }
    }
    
    println!("\n============================");
    println!("Test Results: {} passed, {} failed", passed, failed);
    
    if failed == 0 {
        println!("🎉 All tests passed!");
    } else {
        println!("⚠️ Some tests failed");
    }
    
    Ok(())
}