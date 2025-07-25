use anyhow::Result;
use std::env;
use std::sync::Arc;

use aircher::config::ConfigManager;
use aircher::providers::ProviderManager;
use aircher::auth::AuthManager;

#[tokio::test]
async fn test_provider_initialization() -> Result<()> {
    let config = ConfigManager::load().await?;
    let auth_manager = Arc::new(AuthManager::new()?);
    let providers = ProviderManager::new(&config, auth_manager).await?;
    
    // Should be able to create provider manager
    let provider_list = providers.list_providers();
    assert!(!provider_list.is_empty(), "Should have at least one provider");
    
    println!("Available providers: {:?}", provider_list);
    
    // Should be able to get each provider
    for provider_name in &provider_list {
        let provider = providers.get_provider(provider_name);
        assert!(provider.is_some(), "Provider '{}' should be available", provider_name);
        
        if let Some(p) = provider {
            let context_window = p.context_window();
            assert!(context_window > 0, "Context window should be positive");
            
            let cost = p.calculate_cost(1000, 1000);
            println!("{}: context={}, cost={:?}", provider_name, context_window, cost);
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_health_checks() -> Result<()> {
    let config = ConfigManager::load().await?;
    let auth_manager = Arc::new(AuthManager::new()?);
    let providers = ProviderManager::new(&config, auth_manager).await?;
    
    let health_results = providers.health_check_all().await;
    assert!(!health_results.is_empty(), "Should have health check results");
    
    for (provider, healthy) in &health_results {
        println!("{}: {}", provider, if *healthy { "✅" } else { "❌" });
    }
    
    Ok(())
}

#[tokio::test]
async fn test_invalid_api_keys() -> Result<()> {
    // Backup and set invalid keys
    let claude_backup = env::var("ANTHROPIC_API_KEY").ok();
    env::set_var("ANTHROPIC_API_KEY", "invalid_test_key");
    
    let config = ConfigManager::load().await?;
    let auth_manager = Arc::new(AuthManager::new()?);
    let result = ProviderManager::new(&config, auth_manager).await;
    
    // Should still be able to create provider manager
    assert!(result.is_ok(), "Provider manager should initialize even with invalid keys");
    
    // Restore original key
    if let Some(key) = claude_backup {
        env::set_var("ANTHROPIC_API_KEY", key);
    } else {
        env::remove_var("ANTHROPIC_API_KEY");
    }
    
    Ok(())
}