use anyhow::Result;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use aircher::config::ConfigManager;
use aircher::providers::{ChatRequest, Message, ProviderManager};

/// Test error handling scenarios without requiring valid API keys
#[tokio::test]
async fn test_invalid_api_key_handling() -> Result<()> {
    // Backup original keys
    let claude_backup = env::var("ANTHROPIC_API_KEY").ok();
    let gemini_backup = env::var("GOOGLE_API_KEY").ok();
    let openrouter_backup = env::var("OPENROUTER_API_KEY").ok();
    
    // Set invalid keys
    env::set_var("ANTHROPIC_API_KEY", "invalid_key_test");
    env::set_var("GOOGLE_API_KEY", "invalid_key_test");
    env::set_var("OPENROUTER_API_KEY", "invalid_key_test");
    
    let config = ConfigManager::load().await?;
    let providers = ProviderManager::new(&config).await?;
    
    // Test that providers can be created even with invalid keys
    assert!(providers.get_provider("claude").is_some());
    assert!(providers.get_provider("gemini").is_some());
    assert!(providers.get_host("openrouter").is_some());
    
    // Test actual request fails gracefully (don't wait too long)
    if let Some(provider) = providers.get_provider("claude") {
        let messages = vec![Message::user("test".to_string())];
        let request = ChatRequest::new(messages, "claude-3-haiku-20240307".to_string());
        
        let result = timeout(Duration::from_secs(10), provider.chat(&request)).await;
        
        match result {
            Ok(chat_result) => {
                // Request completed but should fail due to invalid key
                assert!(chat_result.is_err(), "Should fail with invalid API key");
            }
            Err(_) => {
                // Timeout occurred - this is acceptable for this test
                println!("Request timed out (expected with invalid key)");
            }
        }
    }
    
    // Restore original keys
    if let Some(key) = claude_backup {
        env::set_var("ANTHROPIC_API_KEY", key);
    } else {
        env::remove_var("ANTHROPIC_API_KEY");
    }
    
    if let Some(key) = gemini_backup {
        env::set_var("GOOGLE_API_KEY", key);
    } else {
        env::remove_var("GOOGLE_API_KEY");
    }
    
    if let Some(key) = openrouter_backup {
        env::set_var("OPENROUTER_API_KEY", key);
    } else {
        env::remove_var("OPENROUTER_API_KEY");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_provider_health_checks() -> Result<()> {
    let config = ConfigManager::load().await?;
    let providers = ProviderManager::new(&config).await?;
    
    // Health checks should complete without errors
    let health_results = timeout(Duration::from_secs(5), providers.health_check_all()).await?;
    
    assert!(!health_results.is_empty(), "Should have health check results");
    
    // Print health status for debugging
    for (provider, healthy) in &health_results {
        println!("Provider {}: {}", provider, if *healthy { "✅" } else { "❌" });
    }
    
    Ok(())
}

#[tokio::test]
async fn test_cost_calculation_without_requests() -> Result<()> {
    let config = ConfigManager::load().await?;
    let providers = ProviderManager::new(&config).await?;
    
    // Test cost calculation for different token amounts
    let test_cases = vec![
        (1000, 1000),   // 1k input, 1k output
        (10000, 5000),  // 10k input, 5k output
        (0, 1000),      // No input, 1k output
        (1000, 0),      // 1k input, no output
    ];
    
    for provider_name in providers.list_providers() {
        if let Some(provider) = providers.get_provider(&provider_name) {
            for (input_tokens, output_tokens) in &test_cases {
                let cost = provider.calculate_cost(*input_tokens, *output_tokens);
                
                if let Some(c) = cost {
                    assert!(c >= 0.0, "Cost should be non-negative for {}", provider_name);
                    
                    // Cost should increase with more tokens
                    if *input_tokens > 0 || *output_tokens > 0 {
                        assert!(c > 0.0, "Cost should be positive for non-zero tokens");
                    }
                }
                
                println!("{}: {}/{} tokens = ${:.6}", 
                    provider_name, input_tokens, output_tokens, cost.unwrap_or(0.0));
            }
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_provider_capabilities() -> Result<()> {
    let config = ConfigManager::load().await?;
    let providers = ProviderManager::new(&config).await?;
    
    for provider_name in providers.list_providers() {
        if let Some(provider) = providers.get_provider(&provider_name) {
            // Test capability queries
            let context_window = provider.context_window();
            let supports_tools = provider.supports_tools();
            let supports_vision = provider.supports_vision();
            let pricing_model = provider.pricing_model();
            
            assert!(context_window > 0, "{} should have positive context window", provider_name);
            
            println!("{} capabilities:", provider_name);
            println!("  Context window: {}", context_window);
            println!("  Supports tools: {}", supports_tools);
            println!("  Supports vision: {}", supports_vision);
            println!("  Pricing model: {:?}", pricing_model);
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_model_validation() -> Result<()> {
    let config = ConfigManager::load().await?;
    
    // Test model lookup
    let claude_sonnet = config.get_model("claude", "claude-3-5-sonnet-20241022");
    assert!(claude_sonnet.is_some(), "Should find Claude Sonnet model");
    
    let claude_haiku = config.get_model("claude", "claude-3-5-haiku-20241022");
    assert!(claude_haiku.is_some(), "Should find Claude Haiku model");
    
    // Test non-existent model
    let fake_model = config.get_model("claude", "claude-999-fake");
    assert!(fake_model.is_none(), "Should not find fake model");
    
    // Test non-existent provider
    let fake_provider = config.get_model("fake-provider", "any-model");
    assert!(fake_provider.is_none(), "Should not find fake provider");
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_provider_access() -> Result<()> {
    let config = ConfigManager::load().await?;
    let providers = Arc::new(ProviderManager::new(&config).await?);
    
    // Test concurrent access to provider manager
    let handles: Vec<_> = (0..10).map(|i| {
        let providers_clone = Arc::clone(&providers);
        tokio::spawn(async move {
            let provider_list = providers_clone.list_providers();
            assert!(!provider_list.is_empty());
            
            for provider_name in provider_list {
                let provider = providers_clone.get_provider(&provider_name);
                assert!(provider.is_some());
                
                if let Some(p) = provider {
                    let _context_window = p.context_window();
                    let _cost = p.calculate_cost(100, 100);
                }
            }
            
            i // Return the iteration number
        })
    }).collect();
    
    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await?;
        assert!(result < 10);
    }
    
    Ok(())
}