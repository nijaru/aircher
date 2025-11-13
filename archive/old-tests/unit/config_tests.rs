use anyhow::Result;
use std::env;

use aircher::config::{ConfigManager, GlobalConfig, ModelConfig, ProviderConfig};

#[tokio::test]
async fn test_default_config_creation() -> Result<()> {
    let config = ConfigManager::load().await?;

    // Should have default values
    assert_eq!(config.global.default_provider, "claude");
    assert_eq!(config.global.default_model, "claude-3-5-sonnet-20241022");
    assert!(config.global.max_context_tokens > 0);

    // Should have UI defaults
    assert_eq!(config.ui.theme, "default");
    assert_eq!(config.ui.refresh_rate_ms, 100);
    assert!(config.ui.show_token_count);
    assert!(config.ui.show_cost_estimate);

    // Should have intelligence defaults
    assert!(config.intelligence.enable_project_analysis);
    assert!(config.intelligence.enable_file_scoring);
    assert!(config.intelligence.file_scan_depth > 0);

    Ok(())
}

#[tokio::test]
async fn test_provider_configuration() -> Result<()> {
    let config = ConfigManager::load().await?;

    // Should have claude provider configured
    let claude_config = config.get_provider("claude");
    assert!(claude_config.is_some());

    let claude = claude_config.unwrap();
    assert_eq!(claude.name, "Claude");
    assert_eq!(claude.api_key_env, "ANTHROPIC_API_KEY");
    assert!(!claude.models.is_empty());

    // Should have models with valid properties
    for model in &claude.models {
        assert!(!model.name.is_empty());
        assert!(model.context_window > 0);
        assert!(model.input_cost_per_1m >= 0.0);
        assert!(model.output_cost_per_1m >= 0.0);
    }

    Ok(())
}

#[tokio::test]
async fn test_model_lookup() -> Result<()> {
    let config = ConfigManager::load().await?;

    // Test model lookup
    let model = config.get_model("claude", "claude-3-5-sonnet-20241022");
    assert!(model.is_some());

    let model_config = model.unwrap();
    assert_eq!(model_config.name, "claude-3-5-sonnet-20241022");
    assert!(model_config.context_window > 0);
    assert!(model_config.supports_streaming);

    // Test non-existent model
    let missing_model = config.get_model("claude", "non-existent-model");
    assert!(missing_model.is_none());

    Ok(())
}

#[tokio::test]
async fn test_environment_variable_resolution() -> Result<()> {
    // Test that config handles missing environment variables gracefully
    let original_key = env::var("ANTHROPIC_API_KEY").ok();
    env::remove_var("ANTHROPIC_API_KEY");

    let config = ConfigManager::load().await;
    assert!(config.is_ok());

    // Restore original key if it existed
    if let Some(key) = original_key {
        env::set_var("ANTHROPIC_API_KEY", key);
    }

    Ok(())
}

#[test]
fn test_global_config_defaults() {
    let global = GlobalConfig::default();

    assert_eq!(global.default_provider, "claude");
    assert_eq!(global.default_model, "claude-3-5-sonnet-20241022");
    assert_eq!(global.default_host, "anthropic");
    assert_eq!(global.max_context_tokens, 100_000);
    assert!(global.budget_limit.is_none());
    assert!(global.data_directory.to_string_lossy().contains(".aircher"));
}

#[test]
fn test_model_config_validation() {
    let model = ModelConfig {
        name: "test-model".to_string(),
        context_window: 4096,
        input_cost_per_1m: 1.0,
        output_cost_per_1m: 2.0,
        supports_streaming: true,
        supports_tools: false,
    };

    assert_eq!(model.name, "test-model");
    assert_eq!(model.context_window, 4096);
    assert_eq!(model.input_cost_per_1m, 1.0);
    assert_eq!(model.output_cost_per_1m, 2.0);
    assert!(model.supports_streaming);
    assert!(!model.supports_tools);
}

#[test]
fn test_provider_config_creation() {
    let provider = ProviderConfig {
        name: "test-provider".to_string(),
        api_key_env: "TEST_API_KEY".to_string(),
        base_url: "https://api.test.com".to_string(),
        fallback_urls: vec![],
        models: vec![],
        timeout_seconds: 30,
        max_retries: 3,
    };

    assert_eq!(provider.name, "test-provider");
    assert_eq!(provider.api_key_env, "TEST_API_KEY");
    assert_eq!(provider.base_url, "https://api.test.com");
    assert_eq!(provider.timeout_seconds, 30);
    assert_eq!(provider.max_retries, 3);
    assert!(provider.models.is_empty());
}
