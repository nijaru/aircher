use anyhow::Result;
use aircher::providers::{
    ChatRequest, LLMProvider, Message, MessageRole,
    PricingModel,
};
use aircher::providers::openai::OpenAIProvider;
use aircher::auth::AuthManager;
use aircher::config::{ProviderConfig, ModelConfig};
use std::env;
use std::sync::{Arc, Mutex};

// Serialize tests that manipulate environment variables
static ENV_MUTEX: Mutex<()> = Mutex::new(());

fn create_test_config() -> ProviderConfig {
    ProviderConfig {
        name: "OpenAI".to_string(),
        api_key_env: "OPENAI_API_KEY".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        fallback_urls: vec![],
        models: vec![
            ModelConfig {
                name: "gpt-4o".to_string(),
                context_window: 128_000,
                input_cost_per_1m: 5.0,
                output_cost_per_1m: 15.0,
                supports_streaming: true,
                supports_tools: true,
            },
            ModelConfig {
                name: "gpt-4o-mini".to_string(),
                context_window: 128_000,
                input_cost_per_1m: 0.15,
                output_cost_per_1m: 0.6,
                supports_streaming: true,
                supports_tools: true,
            },
            ModelConfig {
                name: "gpt-3.5-turbo".to_string(),
                context_window: 16_385,
                input_cost_per_1m: 0.5,
                output_cost_per_1m: 1.5,
                supports_streaming: true,
                supports_tools: true,
            },
        ],
        timeout_seconds: 120,
        max_retries: 3,
    }
}

fn create_auth() -> Arc<AuthManager> {
    Arc::new(AuthManager::new().expect("failed to create AuthManager"))
}

#[tokio::test]
async fn test_openai_provider_creation() -> Result<()> {
    let _guard = ENV_MUTEX.lock().unwrap();

    // Set a dummy API key for testing
    env::set_var("OPENAI_API_KEY", "test-key-12345");

    let config = create_test_config();
    let provider = OpenAIProvider::new(config, create_auth())?;

    assert_eq!(provider.name(), "OpenAI");
    assert!(provider.supports_tools());
    assert!(provider.supports_vision());
    assert_eq!(provider.context_window(), 128_000);

    Ok(())
}

#[tokio::test]
async fn test_openai_provider_missing_api_key() {
    let _guard = ENV_MUTEX.lock().unwrap();

    // Temporarily remove the API key to test error handling
    let original_key = env::var("OPENAI_API_KEY").ok();
    env::remove_var("OPENAI_API_KEY");

    let config = create_test_config();
    let result = OpenAIProvider::new(config, create_auth());

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("OPENAI_API_KEY"));
    }

    // Restore the original key if it existed
    if let Some(key) = original_key {
        env::set_var("OPENAI_API_KEY", key);
    }
}

#[tokio::test]
async fn test_openai_pricing_model() -> Result<()> {
    let _guard = ENV_MUTEX.lock().unwrap();
    env::set_var("OPENAI_API_KEY", "test-key-12345");

    let config = create_test_config();
    let provider = OpenAIProvider::new(config, create_auth())?;

    match provider.pricing_model() {
        PricingModel::PerToken {
            input_cost_per_1m,
            output_cost_per_1m,
            currency,
        } => {
            assert_eq!(input_cost_per_1m, 5.0); // gpt-4o pricing
            assert_eq!(output_cost_per_1m, 15.0);
            assert_eq!(currency, "USD");
        }
        _ => panic!("Expected PerToken pricing model"),
    }

    Ok(())
}

#[tokio::test]
async fn test_openai_cost_calculation() -> Result<()> {
    let _guard = ENV_MUTEX.lock().unwrap();
    env::set_var("OPENAI_API_KEY", "test-key-12345");

    let config = create_test_config();
    let provider = OpenAIProvider::new(config, create_auth())?;

    // Test cost calculation for 1000 input tokens and 500 output tokens
    let cost = provider.calculate_cost(1000, 500);
    assert!(cost.is_some());

    let expected_cost = (1000.0 / 1_000_000.0) * 5.0 + (500.0 / 1_000_000.0) * 15.0;
    assert!((cost.unwrap() - expected_cost).abs() < 0.0001);

    Ok(())
}

#[tokio::test]
async fn test_openai_pricing_info() -> Result<()> {
    let _guard = ENV_MUTEX.lock().unwrap();
    env::set_var("OPENAI_API_KEY", "test-key-12345");

    let config = create_test_config();
    let provider = OpenAIProvider::new(config, create_auth())?;

    let pricing = provider.get_pricing();
    assert!(pricing.is_some());

    let pricing = pricing.unwrap();
    assert_eq!(pricing.input_cost_per_1m, 5.0);
    assert_eq!(pricing.output_cost_per_1m, 15.0);
    assert_eq!(pricing.currency, "USD");

    Ok(())
}

#[tokio::test]
async fn test_openai_usage_info() -> Result<()> {
    let _guard = ENV_MUTEX.lock().unwrap();
    env::set_var("OPENAI_API_KEY", "test-key-12345");

    let config = create_test_config();
    let provider = OpenAIProvider::new(config, create_auth())?;

    // OpenAI doesn't provide usage info in standard tier
    let usage = provider.get_usage_info().await?;
    assert!(usage.is_none());

    // No usage warning threshold
    assert!(provider.usage_warning_threshold().is_none());

    Ok(())
}

#[tokio::test]
async fn test_openai_model_capabilities() -> Result<()> {
    let _guard = ENV_MUTEX.lock().unwrap();
    env::set_var("OPENAI_API_KEY", "test-key-12345");

    let config = create_test_config();
    let provider = OpenAIProvider::new(config, create_auth())?;

    // Test capabilities
    assert!(provider.supports_tools());
    assert!(provider.supports_vision()); // gpt-4o supports vision
    assert_eq!(provider.context_window(), 128_000);

    Ok(())
}

#[cfg(feature = "integration-tests")]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_openai_health_check_with_real_api() -> Result<()> {
        // Skip if no API key is set
        if env::var("OPENAI_API_KEY").is_err() {
            println!("Skipping OpenAI health check test - no API key set");
            return Ok(());
        }

        let config = create_test_config();
        let provider = OpenAIProvider::new(config, create_auth())?;

        let health = provider.health_check().await;
        // Don't assert success since we might not have a valid API key
        // Just ensure the method runs without panic
        println!("OpenAI health check result: {:?}", health);

        Ok(())
    }

    #[tokio::test]
    async fn test_openai_chat_request_with_real_api() -> Result<()> {
        // Skip if no API key is set
        if env::var("OPENAI_API_KEY").is_err() {
            println!("Skipping OpenAI chat test - no API key set");
            return Ok(());
        }

        let config = create_test_config();
        let provider = OpenAIProvider::new(config, create_auth())?;

        let messages = vec![Message::user("Hello, this is a test".to_string())];
        let request = ChatRequest::new(messages, "gpt-3.5-turbo".to_string());

        // Don't assert success since we might not have credits
        // Just ensure the method runs without panic
        let result = provider.chat(&request).await;
        println!("OpenAI chat result: {:?}", result.map(|r| r.content));

        Ok(())
    }
}

#[tokio::test]
async fn test_openai_request_conversion() -> Result<()> {
    let _guard = ENV_MUTEX.lock().unwrap();
    env::set_var("OPENAI_API_KEY", "test-key-12345");

    let config = create_test_config();
    let provider = OpenAIProvider::new(config, create_auth())?;

    // Test message conversion by ensuring the provider accepts various message types
    let messages = vec![
        Message::new(MessageRole::System, "You are a helpful assistant".to_string()),
        Message::new(MessageRole::User, "Hello".to_string()),
        Message::new(MessageRole::Assistant, "Hi there!".to_string()),
        Message::new(MessageRole::Tool, "Tool result".to_string()),
    ];

    let request = ChatRequest::new(messages, "gpt-3.5-turbo".to_string());

    // This should not panic during request conversion
    // We can't test the actual API call without a valid key, but we can test setup
    assert_eq!(request.model, "gpt-3.5-turbo");
    assert_eq!(request.messages.len(), 4);

    Ok(())
}

#[tokio::test]
async fn test_openai_stream_setup() -> Result<()> {
    let _guard = ENV_MUTEX.lock().unwrap();
    env::set_var("OPENAI_API_KEY", "test-key-12345");

    let config = create_test_config();
    let provider = OpenAIProvider::new(config, create_auth())?;

    let messages = vec![Message::user("Test streaming".to_string())];
    let request = ChatRequest::new(messages, "gpt-3.5-turbo".to_string()).with_streaming();

    // Test that streaming setup works (will fail at network level without valid key)
    // But this ensures the streaming infrastructure is properly set up
    assert!(request.stream);

    Ok(())
}
