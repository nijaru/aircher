use std::sync::Arc;
use aircher::agent::AgentController;
use aircher::auth::AuthManager;
use aircher::config::{ConfigManager, ProviderConfig};
use aircher::intelligence::IntelligenceEngine;
use aircher::storage::DatabaseManager;
use aircher::providers::ollama::OllamaProvider;
use aircher::providers::LLMProvider;
use aircher::agent::conversation::{ProjectContext, ProgrammingLanguage};

#[tokio::test]
async fn test_tool_execution_with_ollama() -> Result<(), Box<dyn std::error::Error>> {
    // Skip if Ollama not available
    if !is_ollama_running().await {
        println!("⚠️ Skipping - Ollama not running");
        return Ok(());
    }

    println!("\n🧪 TESTING ACTUAL TOOL EXECUTION");

    // Setup components
    let config = ConfigManager::load().await?;
    let db = DatabaseManager::new(&config).await?;
    let intelligence = IntelligenceEngine::new(&config, &db).await?;
    let auth = Arc::new(AuthManager::new()?);

    let project_context = ProjectContext {
        root_path: std::env::current_dir()?,
        language: ProgrammingLanguage::Rust,
        framework: Some("cargo".to_string()),
        recent_changes: Vec::new(),
    };

    let mut agent = AgentController::new(intelligence, auth.clone(), project_context)?;

    // Setup Ollama
    let provider_config = ProviderConfig {
        name: "ollama".to_string(),
        api_key_env: "".to_string(),
        base_url: "http://localhost:11434".to_string(),
        fallback_urls: vec![],
        models: vec![],
        timeout_seconds: 30,
        max_retries: 3,
    };

    let ollama = OllamaProvider::new(provider_config, auth).await?;

    // THE CRITICAL TEST
    let result = agent.process_message(
        "Read the Cargo.toml file and tell me the package name",
        &ollama,
        "gpt-oss"
    ).await;

    match result {
        Ok((response, tool_status)) => {
            println!("✅ Got response!");
            println!("Tool status count: {}", tool_status.len());

            // Print the actual response to see what we got
            println!("\n📝 Actual response:");
            println!("{}", response);
            println!("\n🔧 Tool status messages:");
            for msg in &tool_status {
                println!("  - {}", msg);
            }

            // Critical assertions
            assert!(!tool_status.is_empty(), "Should have tool status messages");
            assert!(
                response.contains("aircher") || response.contains("[package]"),
                "Response should contain actual file content, not hallucination"
            );

            println!("✅✅✅ TOOLS ARE ACTUALLY EXECUTING!");
        }
        Err(e) => {
            panic!("Tool execution failed: {}", e);
        }
    }

    Ok(())
}

async fn is_ollama_running() -> bool {
    tokio::process::Command::new("ollama")
        .arg("list")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}
