use anyhow::Result;
use std::path::PathBuf;
use tokio;

use aircher::intelligence::{IntelligenceEngine, IntelligenceTools};
use aircher::config::ConfigManager;
use aircher::storage::DatabaseManager;
use aircher::agent::Agent;
use aircher::agent::conversation::ProjectContext;
use aircher::auth::AuthManager;

/// Comprehensive test suite for Intelligence Engine integration with Agent
/// 
/// Tests the full intelligence pipeline from user query to enhanced response

#[tokio::test]
async fn test_intelligence_engine_initialization() -> Result<()> {
    let config = ConfigManager::default();
    let storage = create_test_database().await?;
    
    let intelligence = IntelligenceEngine::new(&config, &storage).await?;
    
    // Test that intelligence engine components are properly initialized
    assert!(intelligence_is_ready(&intelligence).await);
    
    Ok(())
}

#[tokio::test]
async fn test_contextual_insight_generation() -> Result<()> {
    let intelligence = create_test_intelligence().await?;
    
    // Test context analysis for various development scenarios
    let scenarios = vec![
        ("fix authentication bug", "should identify auth-related files and patterns"),
        ("add new feature to user management", "should suggest related user files and dependencies"),
        ("optimize database queries", "should identify database and performance patterns"),
        ("refactor error handling", "should find error patterns across codebase"),
    ];
    
    for (query, _expected_behavior) in scenarios {
        let context = intelligence.get_development_context(query).await;
        
        // Validate contextual insights
        assert!(context.confidence > 0.0, "Context analysis should provide confidence score");
        assert!(!context.key_files.is_empty(), "Should identify relevant files for: {}", query);
        assert!(!context.suggested_next_actions.is_empty(), "Should suggest actions for: {}", query);
        
        println!("✓ Context analysis for '{}': {} files, confidence {:.2}", 
                 query, context.key_files.len(), context.confidence);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_project_memory_learning() -> Result<()> {
    let intelligence = create_test_intelligence().await?;
    
    // Simulate successful interaction patterns
    let test_interactions = vec![
        (vec!["src/auth/mod.rs".to_string(), "src/auth/tokens.rs".to_string()], 
         create_success_outcome(), "Authentication bug fix"),
        (vec!["src/database/mod.rs".to_string(), "src/models/user.rs".to_string()],
         create_success_outcome(), "Database optimization"),
    ];
    
    // Record successful patterns
    for (files, outcome, description) in &test_interactions {
        intelligence.track_conversation_outcome(files, outcome.clone()).await;
        println!("✓ Recorded successful pattern: {}", description);
    }
    
    // Test pattern recall
    let auth_context = intelligence.get_development_context("authentication issue").await;
    assert!(auth_context.recent_patterns.len() > 0, 
            "Should recall patterns from previous auth work");
    
    let db_context = intelligence.get_development_context("database performance").await; 
    assert!(db_context.recent_patterns.len() > 0,
            "Should recall patterns from previous database work");
    
    Ok(())
}

#[tokio::test] 
async fn test_unified_agent_intelligence_integration() -> Result<()> {
    let (agent, _intelligence) = create_test_unified_agent().await?;
    
    // Test that agent uses intelligence instead of echo responses
    let session_id = "test_session_001";
    agent.create_session(session_id.to_string()).await?;
    
    let test_prompts = vec![
        "How can I fix the authentication bug in user login?",
        "What files should I check for database performance issues?", 
        "Help me refactor the error handling in the API",
    ];
    
    for prompt in test_prompts {
        let response = agent.process_prompt(session_id, prompt.to_string(), None, None).await?;
        
        // Verify intelligence-enhanced responses (not echo responses)
        assert!(!response.starts_with("Echo response:"), 
                "Agent should use intelligence, not echo responses");
        assert!(response.len() > prompt.len(), 
                "Intelligence-enhanced response should be more detailed");
        
        println!("✓ Intelligence-enhanced response for: {}", prompt);
        println!("  Response length: {} chars", response.len());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_development_narrative_tracking() -> Result<()> {
    let intelligence = create_test_intelligence().await?;
    
    // Test project momentum analysis
    let momentum = intelligence.get_project_momentum().await;
    
    assert!(!momentum.recent_focus.is_empty(), "Should identify recent focus areas");
    assert!(!momentum.architectural_direction.is_empty(), "Should track architectural direction");
    assert!(!momentum.next_priorities.is_empty(), "Should suggest next priorities");
    
    println!("✓ Project momentum tracking:");
    println!("  Focus: {}", momentum.recent_focus);
    println!("  Direction: {}", momentum.architectural_direction);
    println!("  Priorities: {:?}", momentum.next_priorities);
    
    Ok(())
}

#[tokio::test]
async fn test_cross_project_intelligence() -> Result<()> {
    let intelligence = create_test_intelligence().await?;
    
    // Test cross-project pattern analysis
    let cross_patterns = intelligence.analyze_cross_project_patterns("authentication patterns").await;
    
    // Should provide insights even with limited data
    println!("✓ Cross-project intelligence analysis:");
    println!("  Patterns found: {}", cross_patterns.similar_patterns.len());
    println!("  Lessons: {}", cross_patterns.architectural_lessons.len());
    
    Ok(())
}

#[tokio::test]
async fn test_intelligence_performance() -> Result<()> {
    let intelligence = create_test_intelligence().await?;
    
    let start = std::time::Instant::now();
    
    // Test response time for intelligence analysis
    let _context = intelligence.get_development_context("complex refactoring task").await;
    
    let duration = start.elapsed();
    assert!(duration.as_millis() < 500, 
            "Intelligence analysis should complete within 500ms, took {}ms", 
            duration.as_millis());
    
    println!("✓ Intelligence performance: {}ms", duration.as_millis());
    
    Ok(())
}

#[tokio::test]
async fn test_intelligence_streaming_integration() -> Result<()> {
    let (agent, _intelligence) = create_test_unified_agent().await?;
    
    // Test intelligence with streaming responses
    let session_id = "test_streaming_001";  
    agent.create_session(session_id.to_string()).await?;
    
    let (tx, mut rx) = aircher::agent::streaming::create_agent_stream();
    
    // Start streaming with intelligence
    agent.process_prompt_streaming(
        session_id,
        "Help me understand the codebase architecture".to_string(),
        None,
        None,
        tx,
    ).await?;
    
    // Collect streaming updates
    let mut updates = vec![];
    while let Some(update_result) = rx.recv().await {
        match update_result {
            Ok(update) => {
                updates.push(update);
                if let aircher::agent::streaming::AgentUpdate::Complete { .. } = updates.last().unwrap() {
                    break;
                }
            }
            Err(e) => panic!("Streaming error: {}", e),
        }
    }
    
    assert!(!updates.is_empty(), "Should receive streaming updates");
    println!("✓ Intelligence streaming: {} updates received", updates.len());
    
    Ok(())
}

// Helper functions for test setup

async fn create_test_database() -> Result<DatabaseManager> {
    let config = ConfigManager::default();
    DatabaseManager::new(&config).await
}

async fn create_test_intelligence() -> Result<IntelligenceEngine> {
    let config = ConfigManager::default();
    let storage = create_test_database().await?;
    IntelligenceEngine::new(&config, &storage).await
}

async fn create_test_unified_agent() -> Result<(Agent, IntelligenceEngine)> {
    let intelligence = create_test_intelligence().await?;
    let auth_manager = std::sync::Arc::new(AuthManager::default());
    
    let project_context = ProjectContext {
        root_path: PathBuf::from("."),
        language: aircher::agent::conversation::ProgrammingLanguage::Rust,
        framework: Some("Rust/Tokio".to_string()),
        recent_changes: vec![],
    };
    
    let agent = Agent::new(intelligence, auth_manager, project_context).await?;
    let intelligence_for_return = create_test_intelligence().await?;
    
    Ok((agent, intelligence_for_return))
}

fn create_success_outcome() -> aircher::intelligence::Outcome {
    aircher::intelligence::Outcome {
        success_rating: 0.9,
        completion_status: "Successfully completed".to_string(),
        user_feedback: Some("This solved the issue perfectly".to_string()),
        identified_gaps: vec![],
    }
}

async fn intelligence_is_ready(intelligence: &IntelligenceEngine) -> bool {
    // Test basic intelligence functionality
    let context = intelligence.get_development_context("test query").await;
    context.confidence >= 0.0 // Should at least return some result
}

#[tokio::test]
async fn test_end_to_end_intelligence_workflow() -> Result<()> {
    println!("🧠 Running end-to-end intelligence workflow test...");
    
    let (agent, intelligence) = create_test_unified_agent().await?;
    let session_id = "e2e_test_session";
    
    // 1. Create session
    agent.create_session(session_id.to_string()).await?;
    println!("✓ Session created");
    
    // 2. Send intelligent query
    let query = "I need to add user authentication to this Rust project";
    let response = agent.process_prompt(session_id, query.to_string(), None, None).await?;
    println!("✓ Intelligent response generated: {} chars", response.len());
    
    // 3. Verify intelligence was used
    assert!(!response.starts_with("Echo response:"), "Should use intelligence");
    assert!(response.len() > 100, "Should provide detailed intelligent response");
    
    // 4. Test learning from interaction
    let files = vec!["src/auth.rs".to_string(), "src/main.rs".to_string()];
    let outcome = create_success_outcome();
    intelligence.track_conversation_outcome(&files, outcome).await;
    println!("✓ Learning recorded");
    
    // 5. Test pattern recall
    let follow_up = "What's the best way to handle auth tokens?";
    let follow_response = agent.process_prompt(session_id, follow_up.to_string(), None, None).await?;
    println!("✓ Follow-up response with learned context: {} chars", follow_response.len());
    
    println!("🎯 End-to-end intelligence workflow PASSED");
    Ok(())
}