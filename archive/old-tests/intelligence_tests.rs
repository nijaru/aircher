use anyhow::Result;
use tempfile::tempdir;
use std::env;

use aircher::config::ConfigManager;
use aircher::storage::DatabaseManager;
use aircher::intelligence::{IntelligenceEngine, IntelligenceTools};

#[tokio::test]
async fn test_intelligence_engine_initialization() -> Result<()> {
    // Create a temporary directory for test databases
    let temp_dir = tempdir()?;
    env::set_var("AIRCHER_DATA_DIR", temp_dir.path().to_str().unwrap());

    let config = ConfigManager::load().await?;
    let storage = DatabaseManager::new(&config).await?;

    // Should be able to create intelligence engine
    let intelligence = IntelligenceEngine::new(&config, &storage).await?;

    // Test basic functionality
    let context = intelligence.get_development_context("test query").await;
    assert!(!context.development_phase.is_empty());
    assert!(!context.active_story.is_empty());

    // Test impact analysis
    let impact = intelligence.analyze_change_impact(&["src/test.rs".to_string()]).await;

    // Test context suggestions
    let suggestions = intelligence.suggest_missing_context(&["src/main.rs".to_string()]).await;

    // Test project momentum
    let momentum = intelligence.get_project_momentum().await;
    assert!(!momentum.recent_focus.is_empty());

    storage.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_contextual_relevance_scoring() -> Result<()> {
    let temp_dir = tempdir()?;
    env::set_var("AIRCHER_DATA_DIR", temp_dir.path().to_str().unwrap());

    let config = ConfigManager::load().await?;
    let storage = DatabaseManager::new(&config).await?;
    let intelligence = IntelligenceEngine::new(&config, &storage).await?;

    // Test with a development-related query
    let context = intelligence.get_development_context("implement new provider").await;

    // Should have some confidence
    assert!(context.confidence >= 0.0);
    assert!(context.confidence <= 1.0);

    // Should have some suggested actions
    println!("Suggested actions: {:?}", context.suggested_next_actions);

    storage.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_conversation_outcome_tracking() -> Result<()> {
    let temp_dir = tempdir()?;
    env::set_var("AIRCHER_DATA_DIR", temp_dir.path().to_str().unwrap());

    let config = ConfigManager::load().await?;
    let storage = DatabaseManager::new(&config).await?;
    let intelligence = IntelligenceEngine::new(&config, &storage).await?;

    // Track a conversation outcome
    let files = vec!["src/main.rs".to_string(), "src/config.rs".to_string()];
    let outcome = aircher::intelligence::Outcome {
        success_rating: 0.8,
        completion_status: "completed".to_string(),
        user_feedback: Some("Great job!".to_string()),
        identified_gaps: vec!["Need more tests".to_string()],
    };

    // Should not panic
    intelligence.track_conversation_outcome(&files, outcome).await;

    storage.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_cross_project_features() -> Result<()> {
    let temp_dir = tempdir()?;
    env::set_var("AIRCHER_DATA_DIR", temp_dir.path().to_str().unwrap());

    let config = ConfigManager::load().await?;
    let storage = DatabaseManager::new(&config).await?;
    let intelligence = IntelligenceEngine::new(&config, &storage).await?;

    // Test cross-project analysis (should return default for now)
    let patterns = intelligence.analyze_cross_project_patterns("authentication").await;
    assert!(patterns.similar_patterns.is_empty()); // Expected since not implemented yet

    // Test AI configuration loading
    let ai_config = intelligence.load_ai_configuration().await;
    // Should not panic and return default structure

    storage.close().await?;
    Ok(())
}
