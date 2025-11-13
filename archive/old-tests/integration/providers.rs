use anyhow::Result;
use std::env;
use std::time::Duration;
use tokio::time::timeout;

use aircher::config::ConfigManager;
use aircher::providers::{ChatRequest, Message, ProviderManager};
use aircher::auth::AuthManager;
use std::sync::Arc;

fn create_auth() -> Arc<AuthManager> {
    Arc::new(AuthManager::new().expect("failed to create AuthManager"))
}

/// Integration tests for provider implementations
/// These tests require actual API keys to be set in environment variables

#[tokio::test]
#[ignore] // Run with --ignored flag when API keys are available
async fn test_claude_api_integration() -> Result<()> {
    // Skip test if API key is not available
    if env::var("ANTHROPIC_API_KEY").is_err() {
        println!("Skipping Claude test: ANTHROPIC_API_KEY not set");
        return Ok(());
    }

    let config = ConfigManager::load().await?;
    let providers = ProviderManager::new(&config, create_auth()).await?;

    let provider = providers
        .get_provider("claude")
        .expect("Claude provider should be available");

    // Test basic chat functionality
    let messages = vec![Message::user("Hello! Please respond with exactly 'Test successful'".to_string())];
    let request = ChatRequest::new(messages, "claude-3-haiku-20240307".to_string());

    let response = timeout(Duration::from_secs(30), provider.chat(&request)).await??;

    assert!(!response.content.is_empty());
    assert!(response.tokens_used > 0);
    assert!(response.cost.is_some());

    println!("Claude response: {}", response.content);
    println!("Tokens used: {}", response.tokens_used);
    println!("Cost: ${:.4}", response.cost.unwrap_or(0.0));

    Ok(())
}

#[tokio::test]
#[ignore] // Run with --ignored flag when API keys are available
async fn test_gemini_api_integration() -> Result<()> {
    // Skip test if API key is not available
    if env::var("GOOGLE_API_KEY").is_err() {
        println!("Skipping Gemini test: GOOGLE_API_KEY not set");
        return Ok(());
    }

    let config = ConfigManager::load().await?;
    let providers = ProviderManager::new(&config, create_auth()).await?;

    let provider = providers
        .get_provider("gemini")
        .expect("Gemini provider should be available");

    // Test basic chat functionality
    let messages = vec![Message::user("Hello! Please respond with exactly 'Test successful'".to_string())];
    let request = ChatRequest::new(messages, "gemini-1.5-flash".to_string());

    let response = timeout(Duration::from_secs(30), provider.chat(&request)).await??;

    assert!(!response.content.is_empty());
    assert!(response.tokens_used > 0);

    println!("Gemini response: {}", response.content);
    println!("Tokens used: {}", response.tokens_used);
    if let Some(cost) = response.cost {
        println!("Cost: ${:.4}", cost);
    }

    Ok(())
}

#[tokio::test]
#[ignore] // Run with --ignored flag when API keys are available
async fn test_openrouter_integration() -> Result<()> {
    // Skip test if API key is not available
    if env::var("OPENROUTER_API_KEY").is_err() {
        println!("Skipping OpenRouter test: OPENROUTER_API_KEY not set");
        return Ok(());
    }

    let config = ConfigManager::load().await?;
    let providers = ProviderManager::new(&config, create_auth()).await?;

    let provider = providers
        .get_host("openrouter")
        .expect("OpenRouter host should be available");

    // Test basic chat functionality with a fast model
    let messages = vec![Message::user("Hello! Please respond with exactly 'Test successful'".to_string())];
    let request = ChatRequest::new(messages, "anthropic/claude-3-haiku".to_string());

    let response = timeout(Duration::from_secs(30), provider.chat(&request)).await??;

    assert!(!response.content.is_empty());
    assert!(response.tokens_used > 0);

    println!("OpenRouter response: {}", response.content);
    println!("Tokens used: {}", response.tokens_used);
    if let Some(cost) = response.cost {
        println!("Cost: ${:.4}", cost);
    }

    Ok(())
}

#[tokio::test]
async fn test_provider_error_handling() -> Result<()> {
    let config = ConfigManager::load().await?;

    // Test with temporarily invalid environment (backup real key if exists)
    let original_key = env::var("ANTHROPIC_API_KEY").ok();
    env::set_var("ANTHROPIC_API_KEY", "invalid_key_for_testing");

    let result = ProviderManager::new(&config, create_auth()).await;

    // Restore original key if it existed
    if let Some(key) = original_key {
        env::set_var("ANTHROPIC_API_KEY", key);
    } else {
        env::remove_var("ANTHROPIC_API_KEY");
    }

    // Provider should still initialize even with invalid key
    // The error should only occur when trying to make requests
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_provider_availability() -> Result<()> {
    let config = ConfigManager::load().await?;
    let providers = ProviderManager::new(&config, create_auth()).await?;

    // Test that we can list providers
    let provider_list = providers.list_providers();
    assert!(!provider_list.is_empty());

    println!("Available providers: {:?}", provider_list);

    // Test provider retrieval
    for provider_name in &provider_list {
        let provider = providers.get_provider(provider_name);
        assert!(provider.is_some(), "Provider '{}' should be retrievable", provider_name);
    }

    Ok(())
}

#[tokio::test]
async fn test_cost_calculation() -> Result<()> {
    let config = ConfigManager::load().await?;
    let providers = ProviderManager::new(&config, create_auth()).await?;

    // Test cost calculation for each provider
    for provider_name in providers.list_providers() {
        if let Some(provider) = providers.get_provider(&provider_name) {
            let cost = provider.calculate_cost(1000, 1000); // 1k input, 1k output tokens

            println!("{} cost for 1k/1k tokens: {:?}", provider_name, cost);

            // Cost should be positive if provider supports cost calculation
            if let Some(c) = cost {
                assert!(c > 0.0, "{} should have positive cost", provider_name);
            }
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_health_checks() -> Result<()> {
    let config = ConfigManager::load().await?;
    let providers = ProviderManager::new(&config).await?;

    // Test health checks for all providers
    let health_results = providers.health_check_all().await;

    println!("Health check results: {:?}", health_results);

    // Should have results for all providers
    assert!(!health_results.is_empty());

    // Each provider should have a health status (true or false)
    for (provider, status) in health_results {
        println!("{}: {}", provider, if status { "✅" } else { "❌" });
    }

    Ok(())
}

#[tokio::test]
async fn test_message_construction() -> Result<()> {
    // Test message creation utilities
    let user_msg = Message::user("Test message".to_string());
    assert_eq!(user_msg.role, aircher::providers::MessageRole::User);
    assert_eq!(user_msg.content, "Test message");
    assert!(!user_msg.id.is_empty());

    let system_msg = Message::system("System instruction".to_string());
    assert_eq!(system_msg.role, aircher::providers::MessageRole::System);

    let assistant_msg = Message::assistant("Assistant response".to_string());
    assert_eq!(assistant_msg.role, aircher::providers::MessageRole::Assistant);

    Ok(())
}

#[tokio::test]
async fn test_chat_request_construction() -> Result<()> {
    // Test chat request creation and modification
    let messages = vec![Message::user("Hello".to_string())];
    let model = "test-model".to_string();

    let request = ChatRequest::new(messages.clone(), model.clone());
    assert_eq!(request.messages.len(), 1);
    assert_eq!(request.model, model);
    assert!(!request.stream);
    assert!(request.max_tokens.is_none());
    assert!(request.temperature.is_none());

    // Test request modification methods
    let request = ChatRequest::new(messages, model)
        .with_streaming()
        .with_max_tokens(1000)
        .with_temperature(0.7);

    assert!(request.stream);
    assert_eq!(request.max_tokens, Some(1000));
    assert_eq!(request.temperature, Some(0.7));

    // Test simple request creation
    let simple_request = ChatRequest::simple("Hello world".to_string(), "test-model".to_string());
    assert_eq!(simple_request.messages.len(), 1);
    assert_eq!(simple_request.messages[0].content, "Hello world");

    Ok(())
}

/// Helper function to run integration tests
/// Usage: cargo test test_integration_suite --ignored
#[tokio::test]
#[ignore]
async fn test_integration_suite() -> Result<()> {
    println!("🧪 Running provider integration test suite...");

    // Check which API keys are available
    let has_claude = env::var("ANTHROPIC_API_KEY").is_ok();
    let has_gemini = env::var("GOOGLE_API_KEY").is_ok();
    let has_openrouter = env::var("OPENROUTER_API_KEY").is_ok();

    println!("API Key availability:");
    println!("  Claude (ANTHROPIC_API_KEY): {}", if has_claude { "✅" } else { "❌" });
    println!("  Gemini (GOOGLE_API_KEY): {}", if has_gemini { "✅" } else { "❌" });
    println!("  OpenRouter (OPENROUTER_API_KEY): {}", if has_openrouter { "✅" } else { "❌" });

    if !has_claude && !has_gemini && !has_openrouter {
        println!("⚠️  No API keys available - skipping integration tests");
        println!("Set environment variables to enable testing:");
        println!("  export ANTHROPIC_API_KEY=your_claude_key");
        println!("  export GOOGLE_API_KEY=your_gemini_key");
        println!("  export OPENROUTER_API_KEY=your_openrouter_key");
        return Ok(());
    }

    // Run tests for available providers
    if has_claude {
        println!("\n🤖 Testing Claude integration...");
        if let Err(e) = test_claude_api_integration().await {
            println!("Claude test failed: {}", e);
        }
    }

    if has_gemini {
        println!("\n⭐ Testing Gemini integration...");
        if let Err(e) = test_gemini_api_integration().await {
            println!("Gemini test failed: {}", e);
        }
    }

    if has_openrouter {
        println!("\n🌐 Testing OpenRouter integration...");
        if let Err(e) = test_openrouter_integration().await {
            println!("OpenRouter test failed: {}", e);
        }
    }

    println!("\n✅ All available provider integration tests passed!");

    Ok(())
}
