/// Trace exactly where time is spent during a request
use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;

use aircher::auth::AuthManager;
use aircher::config::ConfigManager;
use aircher::providers::{ProviderManager, ChatRequest, Message, MessageRole};
use aircher::intelligence::IntelligenceEngine;
use aircher::storage::DatabaseManager;
use aircher::agent::{
    core::Agent,
    conversation::{ProjectContext, ProgrammingLanguage},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔬 DETAILED REQUEST PIPELINE TRACE");
    println!("===================================\n");

    // Setup phase
    let setup_start = Instant::now();
    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let provider_manager = Arc::new(ProviderManager::new(&config, auth_manager.clone()).await?);
    println!("Setup: {:?}", setup_start.elapsed());

    // Test 1: Direct provider call
    println!("\n📊 Test 1: Direct Provider Call");
    if let Some(provider) = provider_manager.get_provider("ollama") {
        let start = Instant::now();

        let messages = vec![Message {
            id: "test".to_string(),
            role: MessageRole::User,
            content: "Say 'test'".to_string(),
            timestamp: chrono::Utc::now(),
            tokens_used: None,
            cost: None,
        }];

        let request = ChatRequest {
            messages,
            model: "gpt-oss".to_string(),
            temperature: Some(0.1),
            max_tokens: Some(10),
            stream: false,
            tools: None,
        };

        match provider.chat(&request).await {
            Ok(_) => println!("   Direct provider: {:?}", start.elapsed()),
            Err(e) => println!("   Error: {}", e),
        }
    }

    // Test 2: Agent without intelligence
    println!("\n📊 Test 2: Agent Creation & Message Processing");

    let db_start = Instant::now();
    let db_manager = DatabaseManager::new(&config).await?;
    println!("   DatabaseManager::new: {:?}", db_start.elapsed());

    let intel_start = Instant::now();
    let intelligence = IntelligenceEngine::new(&config, &db_manager).await?;
    println!("   IntelligenceEngine::new: {:?}", intel_start.elapsed());

    let project_context = ProjectContext {
        root_path: std::env::current_dir()?,
        language: ProgrammingLanguage::Rust,
        framework: None,
        recent_changes: Vec::new(),
    };

    let agent_start = Instant::now();
    let agent = Agent::new(intelligence, auth_manager.clone(), project_context).await?;
    println!("   Agent::new: {:?}", agent_start.elapsed());

    // Test actual message processing
    println!("\n📊 Test 3: Agent Message Processing");

    if let Some(provider) = provider_manager.get_provider("ollama") {
        let msg_start = Instant::now();

        // Call process_message directly to see internal timing
        let (response, _status) = agent.process_message("What is 1+1?", provider, "gpt-oss").await?;

        println!("   Total process_message: {:?}", msg_start.elapsed());
        println!("   Response length: {} chars", response.len());
    }

    // Test 4: Multiple quick requests to see if there's caching
    println!("\n📊 Test 4: Multiple Requests (caching test)");

    if let Some(provider) = provider_manager.get_provider("ollama") {
        for i in 1..=3 {
            let start = Instant::now();
            let (response, _) = agent.process_message(&format!("Say {}", i), provider, "gpt-oss").await?;
            println!("   Request {}: {:?} (response: {} chars)", i, start.elapsed(), response.len());
        }
    }

    println!("\n🎯 TRACE COMPLETE");

    Ok(())
}