use anyhow::Result;
use tokio::time::Duration;

use aircher::testing::{MockProvider, MockIntelligenceTools, MockSessionManager, SessionManagerTrait};
use aircher::providers::{ChatRequest, LLMProvider};
use aircher::intelligence::tools::IntelligenceTools;
use aircher::sessions::Message;

/// Integration test demonstrating TUI functionality with mocked dependencies
#[tokio::test]
async fn test_tui_session_flow() -> Result<()> {
    // Setup mocked dependencies
    let mock_provider = MockProvider::new("test-provider".to_string());
    let mock_intelligence = MockIntelligenceTools::new();
    let mock_session_manager = MockSessionManager::new();
    
    // Configure mock responses
    mock_provider.add_response("Hello! I'm ready to help with your project.".to_string());
    
    // Test session creation
    let session = mock_session_manager.create_session(
        "Test TUI Session".to_string(),
        "test-provider".to_string(),
        "test-model".to_string(),
        Some("Integration test session".to_string()),
        vec!["test".to_string()],
    ).await?;
    
    assert_eq!(session.title, "Test TUI Session");
    assert_eq!(session.provider, "test-provider");
    
    // Test message flow
    let user_message = Message {
        id: "user-msg-1".to_string(),
        role: aircher::sessions::MessageRole::User,
        content: "Hello, can you help me with my project?".to_string(),
        timestamp: chrono::Utc::now(),
        tokens_used: Some(10),
        cost: Some(0.001),
    };
    
    mock_session_manager.add_message(&session.id, &user_message).await?;
    
    // Simulate AI response
    let request = ChatRequest::simple(
        user_message.content.clone(),
        "test-model".to_string(),
    );
    let ai_response = mock_provider.chat(&request).await?;
    
    let ai_message = Message {
        id: "ai-msg-1".to_string(),
        role: aircher::sessions::MessageRole::Assistant,
        content: ai_response.content.clone(),
        timestamp: chrono::Utc::now(),
        tokens_used: Some(ai_response.tokens_used),
        cost: ai_response.cost,
    };
    
    mock_session_manager.add_message(&session.id, &ai_message).await?;
    
    // Test intelligence context analysis
    let context = mock_intelligence.get_development_context("test query").await;
    assert_eq!(context.development_phase, "Mock phase");
    
    // Verify message persistence
    let messages = mock_session_manager.load_session_messages(&session.id).await?;
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].content, "Hello, can you help me with my project?");
    assert_eq!(messages[1].content, "Hello! I'm ready to help with your project.");
    
    // Verify provider was called
    assert_eq!(mock_provider.get_call_count(), 1);
    
    // Verify intelligence tools were called
    let intelligence_calls = mock_intelligence.get_calls();
    assert_eq!(intelligence_calls.len(), 1);
    assert!(intelligence_calls[0].contains("get_development_context"));
    
    // Verify session manager was called
    let session_calls = mock_session_manager.get_calls();
    assert!(session_calls.iter().any(|call| call.contains("create_session")));
    assert!(session_calls.iter().any(|call| call.contains("add_message")));
    assert!(session_calls.iter().any(|call| call.contains("load_session_messages")));
    
    Ok(())
}

/// Test project initialization and intelligence integration
#[tokio::test]
async fn test_project_intelligence_integration() -> Result<()> {
    let mock_intelligence = MockIntelligenceTools::new();
    
    // Test project momentum analysis
    let momentum = mock_intelligence.get_project_momentum().await;
    assert_eq!(momentum.recent_focus, "Mock focus");
    assert_eq!(momentum.velocity_indicators.len(), 1);
    
    // Test impact analysis
    let impact = mock_intelligence.analyze_change_impact(&["src/main.rs".to_string()]).await;
    assert_eq!(impact.direct_impacts.len(), 1);
    assert_eq!(impact.indirect_impacts.len(), 1);
    assert_eq!(impact.risk_areas.len(), 1);
    assert_eq!(impact.suggested_tests.len(), 1);
    
    // Test missing context suggestions
    let suggestions = mock_intelligence.suggest_missing_context(&["src/lib.rs".to_string()]).await;
    assert_eq!(suggestions.confidence, 0.8);
    assert_eq!(suggestions.missing_dependencies.len(), 1);
    
    // Verify all intelligence methods were called
    let calls = mock_intelligence.get_calls();
    assert_eq!(calls.len(), 3);
    assert!(calls.iter().any(|call| call.contains("get_project_momentum")));
    assert!(calls.iter().any(|call| call.contains("analyze_change_impact")));
    assert!(calls.iter().any(|call| call.contains("suggest_missing_context")));
    
    Ok(())
}

/// Test multiple provider interactions
#[tokio::test]
async fn test_multi_provider_scenario() -> Result<()> {
    let claude_provider = MockProvider::new("claude".to_string());
    let gemini_provider = MockProvider::new("gemini".to_string());
    
    // Configure different responses for each provider
    claude_provider.add_response("Claude's response about code structure".to_string());
    gemini_provider.add_response("Gemini's response about algorithms".to_string());
    
    // Test Claude interaction
    let claude_request = ChatRequest::simple(
        "Explain the code structure".to_string(),
        "claude-3-5-sonnet".to_string(),
    );
    let claude_response = claude_provider.chat(&claude_request).await?;
    assert!(claude_response.content.contains("Claude's response"));
    
    // Test Gemini interaction
    let gemini_request = ChatRequest::simple(
        "Explain the algorithm".to_string(),
        "gemini-pro".to_string(),
    );
    let gemini_response = gemini_provider.chat(&gemini_request).await?;
    assert!(gemini_response.content.contains("Gemini's response"));
    
    // Verify both providers were called
    assert_eq!(claude_provider.get_call_count(), 1);
    assert_eq!(gemini_provider.get_call_count(), 1);
    
    Ok(())
}

/// Test session persistence and recovery
#[tokio::test]
async fn test_session_persistence() -> Result<()> {
    let mock_session_manager = MockSessionManager::new();
    
    // Create a session
    let session = mock_session_manager.create_session(
        "Persistence Test".to_string(),
        "claude".to_string(),
        "claude-3-5-sonnet".to_string(),
        Some("Testing persistence".to_string()),
        vec!["persistence".to_string(), "test".to_string()],
    ).await?;
    
    // Add multiple messages
    for i in 1..=5 {
        let message = Message {
            id: format!("msg-{}", i),
            role: if i % 2 == 1 {
                aircher::sessions::MessageRole::User
            } else {
                aircher::sessions::MessageRole::Assistant
            },
            content: format!("Message {}", i),
            timestamp: chrono::Utc::now(),
            tokens_used: Some(10),
            cost: Some(0.001),
        };
        mock_session_manager.add_message(&session.id, &message).await?;
    }
    
    // Verify session can be loaded
    let loaded_session = mock_session_manager.load_session(&session.id).await?;
    assert!(loaded_session.is_some());
    assert_eq!(loaded_session.unwrap().title, "Persistence Test");
    
    // Verify messages are persisted
    let messages = mock_session_manager.load_session_messages(&session.id).await?;
    assert_eq!(messages.len(), 5);
    
    // Verify message order and content
    for (i, message) in messages.iter().enumerate() {
        assert_eq!(message.content, format!("Message {}", i + 1));
    }
    
    Ok(())
}

/// Test error handling in TUI components
#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let mock_provider = MockProvider::new("error-test".to_string());
    let mock_intelligence = MockIntelligenceTools::new();
    
    // Test that mocked components handle various scenarios gracefully
    let request = ChatRequest::simple(
        "Test error handling".to_string(),
        "error-model".to_string(),
    );
    
    // Mock provider should still return a response (no actual errors in mock)
    let response = mock_provider.chat(&request).await?;
    assert!(!response.content.is_empty());
    
    // Intelligence tools should handle empty file lists
    let context = mock_intelligence.get_development_context("").await;
    assert_eq!(context.development_phase, "Mock phase");
    
    Ok(())
}

/// Performance test for TUI operations
#[tokio::test]
async fn test_tui_performance() -> Result<()> {
    let mock_session_manager = MockSessionManager::new();
    
    let start_time = std::time::Instant::now();
    
    // Create multiple sessions rapidly
    for i in 1..=10 {
        let session = mock_session_manager.create_session(
            format!("Session {}", i),
            "claude".to_string(),
            "claude-3-5-sonnet".to_string(),
            None,
            vec![],
        ).await?;
        
        // Add messages to each session
        for j in 1..=5 {
            let message = Message {
                id: format!("msg-{}-{}", i, j),
                role: aircher::sessions::MessageRole::User,
                content: format!("Message {} in session {}", j, i),
                timestamp: chrono::Utc::now(),
                tokens_used: Some(10),
                cost: Some(0.001),
            };
            mock_session_manager.add_message(&session.id, &message).await?;
        }
    }
    
    let duration = start_time.elapsed();
    println!("Created 10 sessions with 5 messages each in {:?}", duration);
    
    // Performance should be reasonable (under 1 second for mock operations)
    assert!(duration < Duration::from_secs(1));
    
    Ok(())
}