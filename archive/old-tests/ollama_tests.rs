use aircher::config::ProviderConfig;
use aircher::providers::ollama::OllamaProvider;
use aircher::auth::AuthManager;
use aircher::providers::{ChatRequest, LLMProvider, Message, MessageRole, PricingModel};
use anyhow::Result;
use std::time::Duration;
use tokio::time::timeout;
use std::sync::Arc;

fn create_auth() -> Arc<AuthManager> {
    Arc::new(AuthManager::new().expect("failed to create AuthManager"))
}

#[tokio::test]
async fn test_ollama_provider_creation() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "http://localhost:11434".to_string(),
        fallback_urls: vec![],
        models: vec![],
        timeout_seconds: 30,
        max_retries: 3,
    };

    let provider = OllamaProvider::new(config, create_auth()).await?;
    assert_eq!(provider.name(), "ollama");
    assert_eq!(provider.pricing_model(), PricingModel::Free);
    assert_eq!(provider.calculate_cost(1000, 1000), Some(0.0));
    assert!(provider.supports_tools()); // Modern Ollama models support tools
    assert!(provider.supports_vision());
    assert_eq!(provider.context_window(), 4096);

    Ok(())
}

#[tokio::test]
async fn test_ollama_provider_with_custom_base_url() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "http://192.168.1.100:11434".to_string(),
        fallback_urls: vec![],
        models: vec![],
        timeout_seconds: 30,
        max_retries: 3,
    };

    let provider = OllamaProvider::new(config, create_auth()).await?;
    assert_eq!(provider.name(), "ollama");

    // Should handle custom base URL for Tailscale or remote Ollama
    Ok(())
}

#[tokio::test]
async fn test_ollama_provider_with_empty_base_url() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "".to_string(), // Empty base URL should use default
        fallback_urls: vec![],
        models: vec![],
        timeout_seconds: 30,
        max_retries: 3,
    };

    let provider = OllamaProvider::new(config, create_auth()).await?;
    assert_eq!(provider.name(), "ollama");

    // Should default to localhost:11434 when base_url is empty
    Ok(())
}

#[tokio::test]
async fn test_ollama_health_check_with_timeout() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "http://localhost:11434".to_string(),
        fallback_urls: vec![],
        models: vec![],
        timeout_seconds: 5,
        max_retries: 1,
    };

    let provider = OllamaProvider::new(config, create_auth()).await?;

    // Test health check with timeout (should not hang)
    let health_check_result = timeout(Duration::from_secs(3), provider.health_check()).await?;

    // Result can be true or false, but shouldn't timeout
    match health_check_result {
        Ok(true) => println!("Ollama is running and healthy"),
        Ok(false) => println!("Ollama is not running or unhealthy"),
        Err(e) => println!("Health check failed: {}", e),
    }

    Ok(())
}

#[tokio::test]
async fn test_ollama_get_models() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "http://localhost:11434".to_string(),
        fallback_urls: vec![],
        models: vec![],
        timeout_seconds: 30,
        max_retries: 3,
    };

    let provider = OllamaProvider::new(config, create_auth()).await?;
    let models = provider.get_models();

    // Should return empty list if Ollama is not running
    // or list of available models if it is running
    println!("Available models: {:?}", models);

    Ok(())
}

#[tokio::test]
async fn test_ollama_pricing_info() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "http://localhost:11434".to_string(),
        fallback_urls: vec![],
        models: vec![],
        timeout_seconds: 30,
        max_retries: 3,
    };

    let provider = OllamaProvider::new(config, create_auth()).await?;

    // Test pricing info
    let pricing = provider.get_pricing();
    assert!(pricing.is_some());

    let pricing_info = pricing.unwrap();
    assert_eq!(pricing_info.input_cost_per_1m, 0.0);
    assert_eq!(pricing_info.output_cost_per_1m, 0.0);
    assert_eq!(pricing_info.currency, "USD");

    // Test usage info
    let usage = provider.get_usage_info().await?;
    assert!(usage.is_none()); // Local models don't have usage limits

    // Test warning threshold
    assert!(provider.usage_warning_threshold().is_none());

    Ok(())
}

#[tokio::test]
async fn test_ollama_message_conversion() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "http://localhost:11434".to_string(),
        fallback_urls: vec![],
        models: vec![],
        timeout_seconds: 30,
        max_retries: 3,
    };

    let provider = OllamaProvider::new(config, create_auth()).await?;

    let messages = vec![
        Message::new(MessageRole::System, "You are a helpful assistant.".to_string()),
        Message::new(MessageRole::User, "Hello!".to_string()),
        Message::new(MessageRole::Assistant, "Hi there!".to_string()),
        Message::new(MessageRole::Tool, "Tool result".to_string()),
    ];

    let request = ChatRequest {
        messages,
        model: "llama3.3".to_string(),
        max_tokens: Some(1000),
        temperature: Some(0.7),
        stream: false,
        tools: None,
    };

    // Test that message conversion works without errors
    // (This tests internal convert_messages method indirectly)
    let _converted_messages = provider.convert_messages(&request.messages);

    Ok(())
}

// Integration test that requires actual Ollama instance running
#[tokio::test]
#[ignore] // Use `cargo test -- --ignored` to run this test
async fn test_ollama_chat_integration() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "http://localhost:11434".to_string(),
        fallback_urls: vec![],
        models: vec![],
        timeout_seconds: 30,
        max_retries: 3,
    };

    let provider = OllamaProvider::new(config, create_auth()).await?;

    // First check if Ollama is running
    let health = provider.health_check().await?;
    if !health {
        println!("Ollama is not running, skipping integration test");
        return Ok(());
    }

    // Check available models
    let models = provider.get_models();
    if models.is_empty() {
        println!("No models available in Ollama, skipping integration test");
        return Ok(());
    }

    let model = models[0].clone();
    println!("Testing with model: {}", model);

    let messages = vec![
        Message::new(MessageRole::User, "Hello! Please respond with just 'Hi there!'".to_string()),
    ];

    let request = ChatRequest {
        messages,
        model,
        max_tokens: Some(50),
        temperature: Some(0.1),
        stream: false,
        tools: None,
    };

    // Test chat request
    let response = provider.chat(&request).await?;
    println!("Response: {}", response.content);

    assert!(!response.content.is_empty());
    assert!(response.tokens_used > 0);
    assert_eq!(response.cost, Some(0.0));

    Ok(())
}

// Integration test for streaming
#[tokio::test]
#[ignore] // Use `cargo test -- --ignored` to run this test
async fn test_ollama_streaming_integration() -> Result<()> {
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "http://localhost:11434".to_string(),
        fallback_urls: vec![],
        models: vec![],
        timeout_seconds: 30,
        max_retries: 3,
    };

    let provider = OllamaProvider::new(config, create_auth()).await?;

    // First check if Ollama is running
    let health = provider.health_check().await?;
    if !health {
        println!("Ollama is not running, skipping streaming integration test");
        return Ok(());
    }

    // Check available models
    let models = provider.get_models();
    if models.is_empty() {
        println!("No models available in Ollama, skipping streaming integration test");
        return Ok(());
    }

    let model = models[0].clone();
    println!("Testing streaming with model: {}", model);

    let messages = vec![
        Message::new(MessageRole::User, "Count from 1 to 5".to_string()),
    ];

    let request = ChatRequest {
        messages,
        model,
        max_tokens: Some(100),
        temperature: Some(0.1),
        stream: true,
        tools: None,
    };

    // Test streaming
    let mut stream = provider.stream(&request).await?;
    let mut chunk_count = 0;
    let mut total_content = String::new();

    while let Some(chunk) = stream.recv().await {
        match chunk {
            Ok(chunk) => {
                chunk_count += 1;
                total_content.push_str(&chunk.content);
                println!("Chunk {}: {}", chunk_count, chunk.content);

                if chunk.finish_reason.is_some() {
                    break;
                }
            }
            Err(e) => {
                println!("Stream error: {}", e);
                break;
            }
        }
    }

    println!("Total chunks: {}", chunk_count);
    println!("Total content: {}", total_content);

    assert!(chunk_count > 0);
    assert!(!total_content.is_empty());

    Ok(())
}

// Test with Tailscale/remote Ollama setup
#[tokio::test]
#[ignore] // Use `cargo test -- --ignored` to run this test
async fn test_ollama_tailscale_integration() -> Result<()> {
    // This test is designed for the user's Tailscale setup
    // Set OLLAMA_TAILSCALE_URL environment variable to test
    let tailscale_url = std::env::var("OLLAMA_TAILSCALE_URL")
        .unwrap_or_else(|_| "http://100.64.0.1:11434".to_string());

    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: tailscale_url.clone(),
        fallback_urls: vec![],
        models: vec![],
        timeout_seconds: 30,
        max_retries: 3,
    };

    let provider = OllamaProvider::new(config, create_auth()).await?;

    println!("Testing Tailscale Ollama at: {}", tailscale_url);

    // Test health check
    let health = timeout(Duration::from_secs(10), provider.health_check()).await??;

    if health {
        println!("Tailscale Ollama is healthy!");

        // Test model discovery
        let models = provider.get_models();
        println!("Available models: {:?}", models);

        // If models are available, test a simple chat
        if !models.is_empty() {
            let messages = vec![
                Message::new(MessageRole::User, "Say hello!".to_string()),
            ];

            let request = ChatRequest {
                messages,
                model: models[0].clone(),
                max_tokens: Some(20),
                temperature: Some(0.1),
                stream: false,
                tools: None,
            };

            let response = provider.chat(&request).await?;
            println!("Tailscale Ollama response: {}", response.content);

            assert!(!response.content.is_empty());
            assert_eq!(response.cost, Some(0.0));
        }
    } else {
        println!("Tailscale Ollama is not healthy or not accessible");
    }

    Ok(())
}

// Test error handling
#[tokio::test]
async fn test_ollama_error_handling() -> Result<()> {
    // Test with invalid URL
    let config = ProviderConfig {
        name: "Ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "http://invalid-url:11434".to_string(),
        fallback_urls: vec![],
        models: vec![],
        timeout_seconds: 5,
        max_retries: 1,
    };

    let provider = OllamaProvider::new(config, create_auth()).await?;

    // Health check should return false for invalid URL
    let health = provider.health_check().await?;
    assert_eq!(health, false);

    // Chat should return error for invalid URL
    let messages = vec![
        Message::new(MessageRole::User, "Test".to_string()),
    ];

    let request = ChatRequest {
        messages,
        model: "test-model".to_string(),
        max_tokens: Some(50),
        temperature: Some(0.1),
        stream: false,
        tools: None,
    };

    let chat_result = provider.chat(&request).await;
    assert!(chat_result.is_err());

    // Stream should return error for invalid URL
    let stream_result = provider.stream(&request).await;
    assert!(stream_result.is_err());

    Ok(())
}
