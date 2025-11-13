use anyhow::Result;
use tempfile::tempdir;
use tokio_test;
use std::sync::Mutex;

use aircher::config::{ConfigManager, DatabaseConfig};
use aircher::sessions::{SessionManager, Session, SessionFilter, ExportFormat};
use aircher::providers::{Message, MessageRole};
use aircher::storage::DatabaseManager;

// Serialize tests that create databases
static DB_MUTEX: Mutex<()> = Mutex::new(());

async fn create_test_session_manager() -> Result<SessionManager> {
    use std::path::PathBuf;

    let mut config = ConfigManager::default();
    config.database = DatabaseConfig {
        conversations_db: PathBuf::from(":memory:"),
        knowledge_db: PathBuf::from(":memory:"),
        file_index_db: PathBuf::from(":memory:"),
        sessions_db: PathBuf::from(":memory:"),
    };

    let database_manager = DatabaseManager::new(&config).await?;
    SessionManager::new(&database_manager).await
}

#[tokio::test]
async fn test_session_creation() -> Result<()> {
    let _guard = DB_MUTEX.lock().unwrap();

    let session_manager = create_test_session_manager().await?;

    let session = session_manager.create_session(
        "Test Chat".to_string(),
        "claude".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        Some("A test conversation".to_string()),
        vec!["test".to_string(), "demo".to_string()],
    ).await?;

    assert_eq!(session.title, "Test Chat");
    assert_eq!(session.provider, "claude");
    assert_eq!(session.model, "claude-3-5-sonnet-20241022");
    assert_eq!(session.description, Some("A test conversation".to_string()));
    assert_eq!(session.tags, vec!["test", "demo"]);
    assert_eq!(session.total_cost, 0.0);
    assert_eq!(session.total_tokens, 0);
    assert_eq!(session.message_count, 0);
    assert!(!session.is_archived);

    Ok(())
}

#[tokio::test]
async fn test_session_persistence() -> Result<()> {
    let _guard = DB_MUTEX.lock().unwrap();

    let session_manager = create_test_session_manager().await?;

    // Create a session
    let session = session_manager.create_session(
        "Persistent Chat".to_string(),
        "openai".to_string(),
        "gpt-4o".to_string(),
        None,
        vec!["persistence".to_string()],
    ).await?;

    let session_id = session.id.clone();

    // Load the session back
    let loaded_session = session_manager.load_session(&session_id).await?;
    assert!(loaded_session.is_some());

    let loaded_session = loaded_session.unwrap();
    assert_eq!(loaded_session.id, session.id);
    assert_eq!(loaded_session.title, session.title);
    assert_eq!(loaded_session.provider, session.provider);
    assert_eq!(loaded_session.model, session.model);
    assert_eq!(loaded_session.tags, session.tags);

    Ok(())
}

#[tokio::test]
async fn test_message_handling() -> Result<()> {
    let _guard = DB_MUTEX.lock().unwrap();

    let session_manager = create_test_session_manager().await?;

    // Create a session
    let session = session_manager.create_session(
        "Message Test".to_string(),
        "gemini".to_string(),
        "gemini-2.0-flash-exp".to_string(),
        None,
        vec![],
    ).await?;

    // Add messages
    let user_message = Message::user("Hello, how are you?".to_string());
    session_manager.add_message(&session.id, &user_message).await?;

    let assistant_message = Message::new(
        MessageRole::Assistant,
        "I'm doing well, thank you! How can I help you today?".to_string(),
    );
    session_manager.add_message(&session.id, &assistant_message).await?;

    // Load messages
    let messages = session_manager.load_session_messages(&session.id).await?;
    assert_eq!(messages.len(), 2);

    assert_eq!(messages[0].role, MessageRole::User);
    assert_eq!(messages[0].content, "Hello, how are you?");
    assert_eq!(messages[0].sequence_number, 1);

    assert_eq!(messages[1].role, MessageRole::Assistant);
    assert_eq!(messages[1].content, "I'm doing well, thank you! How can I help you today?");
    assert_eq!(messages[1].sequence_number, 2);

    // Check session statistics were updated
    let updated_session = session_manager.load_session(&session.id).await?.unwrap();
    assert_eq!(updated_session.message_count, 2);

    Ok(())
}

#[tokio::test]
async fn test_session_search() -> Result<()> {
    let _guard = DB_MUTEX.lock().unwrap();

    let session_manager = create_test_session_manager().await?;

    // Create multiple sessions
    let _session1 = session_manager.create_session(
        "Claude Chat".to_string(),
        "claude".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        Some("A chat with Claude".to_string()),
        vec!["claude".to_string()],
    ).await?;

    let _session2 = session_manager.create_session(
        "OpenAI Test".to_string(),
        "openai".to_string(),
        "gpt-4o".to_string(),
        Some("Testing OpenAI".to_string()),
        vec!["openai".to_string(), "test".to_string()],
    ).await?;

    let _session3 = session_manager.create_session(
        "Gemini Experiment".to_string(),
        "gemini".to_string(),
        "gemini-1.5-pro".to_string(),
        None,
        vec!["gemini".to_string()],
    ).await?;

    // Search all sessions
    let all_sessions = session_manager.search_sessions(&SessionFilter::default(), None).await?;
    assert_eq!(all_sessions.len(), 3);

    // Search with provider filter
    let filter = SessionFilter {
        provider: Some("claude".to_string()),
        ..Default::default()
    };
    let claude_sessions = session_manager.search_sessions(&filter, None).await?;
    assert_eq!(claude_sessions.len(), 1);
    assert_eq!(claude_sessions[0].provider, "claude");

    Ok(())
}

#[tokio::test]
async fn test_session_archiving() -> Result<()> {
    let _guard = DB_MUTEX.lock().unwrap();

    let session_manager = create_test_session_manager().await?;

    let session = session_manager.create_session(
        "Archive Test".to_string(),
        "claude".to_string(),
        "claude-3-5-haiku-20241022".to_string(),
        None,
        vec![],
    ).await?;

    assert!(!session.is_archived);

    // Archive the session
    session_manager.archive_session(&session.id, true).await?;

    // Check it's archived
    let archived_session = session_manager.load_session(&session.id).await?.unwrap();
    assert!(archived_session.is_archived);

    // Unarchive it
    session_manager.archive_session(&session.id, false).await?;

    let unarchived_session = session_manager.load_session(&session.id).await?.unwrap();
    assert!(!unarchived_session.is_archived);

    Ok(())
}

#[tokio::test]
async fn test_session_deletion() -> Result<()> {
    let _guard = DB_MUTEX.lock().unwrap();

    let session_manager = create_test_session_manager().await?;

    let session = session_manager.create_session(
        "Delete Test".to_string(),
        "openai".to_string(),
        "gpt-4o-mini".to_string(),
        None,
        vec![],
    ).await?;

    // Add a message
    let message = Message::user("This will be deleted".to_string());
    session_manager.add_message(&session.id, &message).await?;

    // Verify session and message exist
    assert!(session_manager.load_session(&session.id).await?.is_some());
    let messages = session_manager.load_session_messages(&session.id).await?;
    assert_eq!(messages.len(), 1);

    // Delete the session
    session_manager.delete_session(&session.id).await?;

    // Verify session is gone
    assert!(session_manager.load_session(&session.id).await?.is_none());

    // Verify messages are also gone (cascade delete)
    let messages_after_delete = session_manager.load_session_messages(&session.id).await?;
    assert_eq!(messages_after_delete.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_session_analytics() -> Result<()> {
    let _guard = DB_MUTEX.lock().unwrap();

    let session_manager = create_test_session_manager().await?;

    // Create sessions with some cost and token data
    let session1 = session_manager.create_session(
        "Analytics Test 1".to_string(),
        "claude".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        None,
        vec![],
    ).await?;

    let session2 = session_manager.create_session(
        "Analytics Test 2".to_string(),
        "openai".to_string(),
        "gpt-4o".to_string(),
        None,
        vec![],
    ).await?;

    // Add messages with cost data
    let mut message1 = Message::user("Test message 1".to_string());
    message1.cost = Some(0.01);
    message1.tokens_used = Some(100);
    session_manager.add_message(&session1.id, &message1).await?;

    let mut message2 = Message::assistant("Response 1".to_string());
    message2.cost = Some(0.02);
    message2.tokens_used = Some(50);
    session_manager.add_message(&session1.id, &message2).await?;

    let mut message3 = Message::user("Test message 2".to_string());
    message3.cost = Some(0.015);
    message3.tokens_used = Some(75);
    session_manager.add_message(&session2.id, &message3).await?;

    // Get analytics
    let analytics = session_manager.get_analytics().await?;

    assert_eq!(analytics.total_sessions, 2);
    assert_eq!(analytics.total_messages, 3);
    assert!((analytics.total_cost - 0.045).abs() < 0.0001); // 0.01 + 0.02 + 0.015
    assert_eq!(analytics.total_tokens, 225); // 100 + 50 + 75
    assert!((analytics.avg_session_cost - 0.0225).abs() < 0.0001); // 0.045 / 2
    assert!((analytics.avg_session_length - 1.5).abs() < 0.0001); // 3 messages / 2 sessions

    Ok(())
}

#[tokio::test]
async fn test_export_formats() -> Result<()> {
    let _guard = DB_MUTEX.lock().unwrap();

    let session_manager = create_test_session_manager().await?;

    let session = session_manager.create_session(
        "Export Test".to_string(),
        "claude".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        Some("Testing export functionality".to_string()),
        vec!["export".to_string()],
    ).await?;

    // Add some messages
    let user_message = Message::user("Hello!".to_string());
    session_manager.add_message(&session.id, &user_message).await?;

    let assistant_message = Message::assistant("Hi there!".to_string());
    session_manager.add_message(&session.id, &assistant_message).await?;

    // Test JSON export
    let json_export = session_manager.export_session(&session.id, ExportFormat::Json).await?;
    assert!(json_export.contains("Export Test"));
    assert!(json_export.contains("Hello!"));
    assert!(json_export.contains("Hi there!"));
    assert!(json_export.contains("\"provider\": \"claude\""));

    // Test Markdown export
    let markdown_export = session_manager.export_session(&session.id, ExportFormat::Markdown).await?;
    assert!(markdown_export.contains("# Export Test"));
    assert!(markdown_export.contains("**Provider:** claude"));
    assert!(markdown_export.contains("### User"));
    assert!(markdown_export.contains("### Assistant"));
    assert!(markdown_export.contains("Hello!"));
    assert!(markdown_export.contains("Hi there!"));

    // Test Plain export
    let plain_export = session_manager.export_session(&session.id, ExportFormat::Plain).await?;
    assert!(plain_export.contains("Export Test"));
    assert!(plain_export.contains("Provider: claude"));
    assert!(plain_export.contains("[USER] Hello!"));
    assert!(plain_export.contains("[ASSISTANT] Hi there!"));

    // Test CSV export
    let csv_export = session_manager.export_session(&session.id, ExportFormat::Csv).await?;
    assert!(csv_export.contains("timestamp,role,content,tokens,cost"));
    assert!(csv_export.contains("user,\"Hello!\""));
    assert!(csv_export.contains("assistant,\"Hi there!\""));

    Ok(())
}

#[tokio::test]
async fn test_message_conversion() -> Result<()> {
    let _guard = DB_MUTEX.lock().unwrap();

    // Test Message to SessionMessage conversion
    let original_message = Message::new(MessageRole::User, "Test content".to_string());
    let session_message = original_message.to_session_message("session123", "claude", "claude-3-5-sonnet-20241022");

    assert_eq!(session_message.id, original_message.id);
    assert_eq!(session_message.session_id, "session123");
    assert_eq!(session_message.role, original_message.role);
    assert_eq!(session_message.content, original_message.content);
    assert_eq!(session_message.timestamp, original_message.timestamp);
    assert_eq!(session_message.tokens_used, original_message.tokens_used);
    assert_eq!(session_message.cost, original_message.cost);
    assert_eq!(session_message.provider, "claude");
    assert_eq!(session_message.model, "claude-3-5-sonnet-20241022");

    // Test SessionMessage to Message conversion
    let converted_back = session_message.to_message();
    assert_eq!(converted_back.id, original_message.id);
    assert_eq!(converted_back.role, original_message.role);
    assert_eq!(converted_back.content, original_message.content);
    assert_eq!(converted_back.timestamp, original_message.timestamp);
    assert_eq!(converted_back.tokens_used, original_message.tokens_used);
    assert_eq!(converted_back.cost, original_message.cost);

    Ok(())
}

#[tokio::test]
async fn test_session_filter_edge_cases() -> Result<()> {
    let _guard = DB_MUTEX.lock().unwrap();

    let session_manager = create_test_session_manager().await?;

    // Test empty filter (should return all sessions)
    let empty_filter = SessionFilter::default();
    let all_sessions = session_manager.search_sessions(&empty_filter, None).await?;
    // Should not fail, might be empty if no sessions exist

    // Test filter with non-existent provider
    let nonexistent_filter = SessionFilter {
        provider: Some("nonexistent-provider".to_string()),
        ..Default::default()
    };
    let no_sessions = session_manager.search_sessions(&nonexistent_filter, None).await?;
    // Should not fail, should return empty list

    Ok(())
}

#[tokio::test]
async fn test_session_cost_tracking() -> Result<()> {
    let _guard = DB_MUTEX.lock().unwrap();

    let session_manager = create_test_session_manager().await?;

    let session = session_manager.create_session(
        "Cost Tracking Test".to_string(),
        "openai".to_string(),
        "gpt-4o".to_string(),
        None,
        vec![],
    ).await?;

    // Add messages with different costs
    let mut expensive_message = Message::user("This is expensive".to_string());
    expensive_message.cost = Some(0.05);
    expensive_message.tokens_used = Some(1000);
    session_manager.add_message(&session.id, &expensive_message).await?;

    let mut cheap_message = Message::assistant("This is cheap".to_string());
    cheap_message.cost = Some(0.01);
    cheap_message.tokens_used = Some(200);
    session_manager.add_message(&session.id, &cheap_message).await?;

    // Load session and verify cost tracking
    let updated_session = session_manager.load_session(&session.id).await?.unwrap();
    assert!((updated_session.total_cost - 0.06).abs() < 0.0001);
    assert_eq!(updated_session.total_tokens, 1200);
    assert_eq!(updated_session.message_count, 2);

    Ok(())
}
