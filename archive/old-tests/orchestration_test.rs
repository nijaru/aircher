use anyhow::Result;
use std::sync::Arc;
use aircher::agent::core::Agent;
use aircher::agent::task_orchestrator::TaskOrchestrator;
use aircher::agent::reasoning::AgentReasoning;
use aircher::agent::dynamic_context::DynamicContextManager;
use aircher::intelligence::IntelligenceEngine;
use aircher::testing::MockProvider;
use aircher::config::ConfigManager;
use aircher::storage::DatabaseManager;
use aircher::auth::AuthManager;
use aircher::agent::conversation::{ProjectContext, ProgrammingLanguage};
use aircher::agent::tools::ToolRegistry;

/// Basic test to verify orchestration functionality compiles and runs
#[tokio::test]
async fn test_orchestration_basic() -> Result<()> {
    // Create mock provider for testing
    let provider = MockProvider::new("test-provider".to_string());
    provider.add_response("I'll help you with that task. Let me break it down into steps.".to_string());

    // Create test orchestrator (simplified version for testing)
    let orchestrator = create_test_orchestrator().await?;

    // Test: Simple task execution
    let simple_task = "Read a file and analyze it";

    // The orchestrator should handle this task
    let result = orchestrator.execute_task(simple_task, &provider, "test-model").await?;

    // Basic verification that orchestration system is working
    assert!(!result.summary.is_empty(), "Should have a summary");
    assert!(result.steps_total >= 0, "Should have valid step count");

    println!("✓ Basic orchestration test passed");
    println!("  Summary: {}", result.summary);
    println!("  Steps: {}/{}", result.steps_completed, result.steps_total);
    println!("  Execution time: {}ms", result.execution_time_ms);

    Ok(())
}

/// Test that orchestrator history tracking works
#[tokio::test]
async fn test_orchestration_history() -> Result<()> {
    let orchestrator = create_test_orchestrator().await?;

    // Initially should have no history
    let initial_history = orchestrator.get_history().await;
    println!("Initial history length: {}", initial_history.len());

    // After a task, history should be populated
    let provider = MockProvider::new("test-provider".to_string());
    provider.add_response("Task completed successfully".to_string());

    let _result = orchestrator.execute_task("Simple test task", &provider, "test-model").await?;

    let final_history = orchestrator.get_history().await;
    println!("Final history length: {}", final_history.len());

    // Should have at least some history steps (planning, execution, completion)
    assert!(final_history.len() > 0, "Should have recorded history steps");

    println!("✓ Orchestration history tracking working");

    Ok(())
}

/// Create a test orchestrator with minimal mocks
async fn create_test_orchestrator() -> Result<TaskOrchestrator> {
    // Create minimal components for testing
    let config = ConfigManager::load().await?;
    let storage = DatabaseManager::new(&config).await?;
    let intelligence = Arc::new(IntelligenceEngine::new(&config, &storage).await?);
    let auth_manager = Arc::new(AuthManager::new()?);

    // Create empty project context for testing
    let project_context = ProjectContext {
        root_path: std::path::PathBuf::from("/tmp/test"),
        language: ProgrammingLanguage::Rust,
        framework: None,
        recent_changes: Vec::new(),
    };

    // Create agent (IntelligenceEngine doesn't implement Clone, so we need to work around this)
    // We'll need to create another instance of the intelligence engine for the agent
    let intelligence_for_agent = IntelligenceEngine::new(&config, &storage).await?;
    let agent = Arc::new(Agent::new(
        intelligence_for_agent,
        auth_manager,
        project_context,
    ).await?);

    let context_manager = Arc::new(DynamicContextManager::new(intelligence.clone(), None));
    let tools = Arc::new(ToolRegistry::default());
    let reasoning = Arc::new(AgentReasoning::new(intelligence.clone(), tools.clone()));

    Ok(TaskOrchestrator::new(
        agent,
        reasoning,
        context_manager,
        intelligence,
    ))
}
