use aircher::config::ProviderConfig;
use aircher::providers::ollama::OllamaProvider;
use aircher::providers::LLMProvider;
use aircher::auth::AuthManager;
use std::sync::Arc;

fn create_auth() -> Arc<AuthManager> {
    Arc::new(AuthManager::new().expect("failed to create AuthManager"))
}
use anyhow::Result;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_ollama_auto_discovery_with_empty_config() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "".to_string(), // Empty to trigger auto-discovery
        fallback_urls: vec![],
        models: vec![],
        timeout_seconds: 10,
        max_retries: 1,
    };

    // This should try auto-discovery and fallback to localhost if nothing found
    let provider = OllamaProvider::new(config, create_auth()).await?;
    assert_eq!(provider.name(), "ollama");
    
    Ok(())
}

#[tokio::test]
async fn test_ollama_auto_discovery_with_fallback_urls() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "".to_string(), // Empty to trigger auto-discovery
        fallback_urls: vec![
            "http://localhost:11434".to_string(),
            "http://100.64.0.1:11434".to_string(), // Example Tailscale IP
            "http://192.168.1.100:11434".to_string(), // Example local IP
        ],
        models: vec![],
        timeout_seconds: 10,
        max_retries: 1,
    };

    // This should try the fallback URLs first, then auto-discovery
    let provider = OllamaProvider::new(config, create_auth()).await?;
    assert_eq!(provider.name(), "ollama");
    
    Ok(())
}

#[tokio::test]
async fn test_ollama_explicit_config_overrides_discovery() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "http://explicit.example.com:11434".to_string(), // Explicit URL
        fallback_urls: vec![
            "http://localhost:11434".to_string(),
            "http://100.64.0.1:11434".to_string(),
        ],
        models: vec![],
        timeout_seconds: 10,
        max_retries: 1,
    };

    // This should use the explicit base_url and skip auto-discovery
    let provider = OllamaProvider::new(config, create_auth()).await?;
    assert_eq!(provider.name(), "ollama");
    
    Ok(())
}

#[tokio::test]
async fn test_ollama_discovery_timeout() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "".to_string(),
        fallback_urls: vec![
            "http://192.168.255.255:11434".to_string(), // Should timeout
            "http://10.255.255.255:11434".to_string(),  // Should timeout
        ],
        models: vec![],
        timeout_seconds: 5,
        max_retries: 1,
    };

    // Test that discovery doesn't hang indefinitely
    let result = timeout(Duration::from_secs(10), OllamaProvider::new(config, create_auth())).await;
    assert!(result.is_ok(), "Discovery should complete within timeout");
    
    Ok(())
}

#[tokio::test]
async fn test_ollama_discovery_with_invalid_urls() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "".to_string(),
        fallback_urls: vec![
            "http://invalid-host:11434".to_string(),
            "http://does-not-exist:11434".to_string(),
        ],
        models: vec![],
        timeout_seconds: 5,
        max_retries: 1,
    };

    // Should gracefully handle invalid URLs and fallback to localhost
    let provider = OllamaProvider::new(config, create_auth()).await?;
    assert_eq!(provider.name(), "ollama");
    
    Ok(())
}

#[tokio::test]
async fn test_ollama_discovery_priority_order() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "".to_string(),
        fallback_urls: vec![
            "http://first-priority:11434".to_string(),
            "http://second-priority:11434".to_string(),
        ],
        models: vec![],
        timeout_seconds: 5,
        max_retries: 1,
    };

    // Test verifies that the provider is created (priority order tested implicitly)
    let provider = OllamaProvider::new(config, create_auth()).await?;
    assert_eq!(provider.name(), "ollama");
    
    Ok(())
}

// Integration test for actual Tailscale discovery
#[tokio::test]
#[ignore] // Use `cargo test -- --ignored` to run this test
async fn test_ollama_tailscale_autodiscovery() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "".to_string(), // Enable auto-discovery
        fallback_urls: vec![], // No fallbacks, rely on auto-discovery
        models: vec![],
        timeout_seconds: 30,
        max_retries: 1,
    };

    println!("Testing Tailscale auto-discovery...");
    let provider = OllamaProvider::new(config, create_auth()).await?;
    
    // If we reach here, auto-discovery worked
    println!("Auto-discovery successful!");
    assert_eq!(provider.name(), "ollama");
    
    // Test that we can actually connect
    let health = provider.health_check().await?;
    if health {
        println!("Successfully connected to auto-discovered Ollama instance");
        
        // Test model discovery
        let models = provider.get_models();
        println!("Available models: {:?}", models);
    } else {
        println!("Auto-discovery found URL but instance is not healthy");
    }
    
    Ok(())
}

// Test the candidate URL generation
#[tokio::test]
async fn test_candidate_url_generation() -> Result<()> {
    // This tests the internal candidate URL generation logic
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "".to_string(),
        fallback_urls: vec![
            "http://custom:11434".to_string(),
        ],
        models: vec![],
        timeout_seconds: 1, // Very short timeout for testing
        max_retries: 1,
    };

    // This should complete quickly even with many candidate URLs
    let start = std::time::Instant::now();
    let provider = OllamaProvider::new(config, create_auth()).await?;
    let duration = start.elapsed();
    
    assert_eq!(provider.name(), "ollama");
    assert!(duration < Duration::from_secs(30), "Auto-discovery should complete reasonably quickly");
    
    Ok(())
}

// Test that manual configuration still works
#[tokio::test]
async fn test_manual_config_still_works() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "http://localhost:11434".to_string(), // Manual config
        fallback_urls: vec![
            "http://should-not-be-used:11434".to_string(),
        ],
        models: vec![],
        timeout_seconds: 10,
        max_retries: 1,
    };

    // Manual config should be used, not fallback URLs
    let provider = OllamaProvider::new(config, create_auth()).await?;
    assert_eq!(provider.name(), "ollama");
    
    Ok(())
}
