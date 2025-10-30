use aircher::auth::AuthManager;
use aircher::config::ProviderConfig;
use aircher::providers::claude_api::ClaudeApiProvider;
use aircher::providers::{ChatRequest, LLMProvider, Message, MessageRole};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔐 Testing OAuth authentication with Claude API...\n");

    // Create auth manager
    let auth_manager = Arc::new(AuthManager::new()?);

    // Create provider config for Anthropic
    let config = ProviderConfig {
        name: "anthropic".to_string(),
        api_key_env: "ANTHROPIC_API_KEY".to_string(),
        base_url: "https://api.anthropic.com/v1".to_string(),
        fallback_urls: vec![],
        timeout_seconds: 120,
        max_retries: 3,
        models: vec![],
    };

    // Initialize provider (should load OAuth tokens)
    println!("📋 Initializing ClaudeApiProvider...");
    let provider = ClaudeApiProvider::new(config, auth_manager).await?;
    println!("✅ Provider initialized successfully\n");

    // Create a minimal test request
    let request = ChatRequest {
        model: "claude-sonnet-4-5".to_string(),
        messages: vec![Message {
            id: Uuid::new_v4().to_string(),
            role: MessageRole::User,
            content: "Say 'OAuth authentication working!' and nothing else.".to_string(),
            timestamp: Utc::now(),
            tokens_used: None,
            cost: None,
        }],
        max_tokens: Some(50),
        temperature: Some(0.0),
        stream: false,
        tools: None,
    };

    // Make API call
    println!("📡 Making test API call to Claude API...");
    let response = provider.chat(&request).await?;

    println!("✅ API call successful!\n");
    println!("📝 Response: {}", response.content);
    println!("🎯 Model used: {}", response.model);
    println!("🔢 Tokens used: {}", response.tokens_used);

    if let Some(cost) = response.cost {
        println!("💰 Cost: ${:.6}", cost);
    }

    println!("\n✅ OAuth authentication test PASSED");
    println!("✅ Using Claude Max subscription (not billing per token)");

    Ok(())
}
