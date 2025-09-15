use anyhow::Result;
use aircher::ui::intelligent_compaction::IntelligentCompactionAnalyzer;
use aircher::providers::{Message, MessageRole};

/// Test that intelligent compaction analyzer works without intelligence engine
#[tokio::test]
async fn test_basic_intelligent_compaction() -> Result<()> {
    let analyzer = IntelligentCompactionAnalyzer::new()?;

    let messages = vec![
        Message::user("I'm working on implementing authentication in src/auth.rs".to_string()),
        Message::assistant("I'll help you implement authentication. Let me read the current file.".to_string()),
        Message::user("The login function is failing with JWT parsing errors".to_string()),
        Message::assistant("Let me check the JWT parsing logic and fix the error.".to_string()),
        Message::user("Great, now let's add tests for this functionality".to_string()),
    ];

    let context = analyzer.analyze_conversation(&messages).await?;

    println!("🧪 Basic Intelligent Compaction Test Results:");
    println!("📊 Current task: {}", context.base_context.current_task);
    println!("📁 Recent files: {:?}", context.base_context.recent_files);
    println!("🔧 Active tools: {:?}", context.base_context.active_tools);
    println!("🏗️ Project type: {:?}", context.base_context.project_type);
    println!("💡 Intelligence confidence: {:.1}%", context.intelligence_confidence * 100.0);

    // Verify basic functionality
    assert!(!context.base_context.current_task.is_empty());
    assert!(context.base_context.recent_files.iter().any(|f| f.contains("auth.rs")));
    assert_eq!(context.base_context.project_type, Some("rust".to_string()));

    // Should not have intelligence insights without engine
    assert!(!context.has_intelligence_insights());
    assert_eq!(context.intelligence_confidence, 0.0);

    println!("✅ Basic intelligent compaction working correctly!");

    Ok(())
}

/// Test intelligent prompt generation
#[tokio::test]
async fn test_intelligent_prompt_generation() -> Result<()> {
    let analyzer = IntelligentCompactionAnalyzer::new()?;

    let messages = vec![
        Message::user("I need to implement user authentication with JWT tokens".to_string()),
        Message::assistant("I'll help you create a secure authentication system.".to_string()),
        Message::user("Let's start with the login endpoint in src/auth.rs".to_string()),
        Message::assistant("Here's the implementation for the login endpoint...".to_string()),
    ];

    let context = analyzer.analyze_conversation(&messages).await?;

    // Test fallback to base context prompt when no intelligence
    let prompt = if context.has_intelligence_insights() {
        context.generate_intelligent_prompt("test conversation")
    } else {
        context.base_context.generate_smart_prompt("test conversation")
    };

    println!("🤖 Generated Compaction Prompt:");
    println!("{}", prompt);

    // Verify prompt contains key elements
    assert!(prompt.contains("authentication"));
    assert!(prompt.contains("src/auth.rs"));
    assert!(prompt.contains("rust"));
    assert!(prompt.contains("Conversation to summarize"));

    println!("✅ Intelligent prompt generation working!");

    Ok(())
}

/// Test analysis summary generation
#[tokio::test]
async fn test_analysis_summary() -> Result<()> {
    let analyzer = IntelligentCompactionAnalyzer::new()?;

    let messages = vec![
        Message::user("Working on fixing database connection issues in src/db.rs".to_string()),
        Message::assistant("Let me analyze the database connection code.".to_string()),
        Message::user("The connection pool is failing with timeout errors".to_string()),
    ];

    let context = analyzer.analyze_conversation(&messages).await?;
    let summary = context.get_analysis_summary();

    println!("📋 Analysis Summary:");
    println!("{}", summary);

    // Verify summary contains relevant information
    assert!(summary.contains("analysis complete"));
    assert!(summary.contains("files"));
    assert!(summary.contains("tools"));

    println!("✅ Analysis summary generation working!");

    Ok(())
}

/// Test project type detection
#[tokio::test]
async fn test_project_type_detection() -> Result<()> {
    let analyzer = IntelligentCompactionAnalyzer::new()?;

    // Test Rust project detection
    let rust_messages = vec![
        Message::user("Let's update the Cargo.toml dependencies".to_string()),
        Message::assistant("I'll help you update the Rust dependencies.".to_string()),
    ];

    let rust_context = analyzer.analyze_conversation(&rust_messages).await?;
    assert_eq!(rust_context.base_context.project_type, Some("rust".to_string()));

    // Test Node.js project detection
    let node_messages = vec![
        Message::user("We need to update package.json scripts".to_string()),
        Message::assistant("I'll help you update the npm scripts.".to_string()),
    ];

    let node_context = analyzer.analyze_conversation(&node_messages).await?;
    assert_eq!(node_context.base_context.project_type, Some("node".to_string()));

    // Test Python project detection
    let python_messages = vec![
        Message::user("Let's install the dependencies from requirements.txt".to_string()),
        Message::assistant("I'll help you set up the Python environment.".to_string()),
    ];

    let python_context = analyzer.analyze_conversation(&python_messages).await?;
    assert_eq!(python_context.base_context.project_type, Some("python".to_string()));

    println!("✅ Project type detection working for rust, node, and python!");

    Ok(())
}

/// Test conversation context extraction
#[tokio::test]
async fn test_context_extraction() -> Result<()> {
    let analyzer = IntelligentCompactionAnalyzer::new()?;

    let messages = vec![
        Message::user("I decided to use tokio for async operations".to_string()),
        Message::assistant("Great choice! Tokio is excellent for async Rust.".to_string()),
        Message::user("Now I'm getting compilation errors in src/main.rs".to_string()),
        Message::assistant("Let me help you fix those compilation errors.".to_string()),
        Message::user("The error says 'cannot find function run_server'".to_string()),
    ];

    let context = analyzer.analyze_conversation(&messages).await?;

    // Verify context extraction
    assert!(context.base_context.current_task.to_lowercase().contains("compilation") ||
            context.base_context.current_task.to_lowercase().contains("error"));

    assert!(context.base_context.recent_files.iter().any(|f| f.contains("main.rs")));

    assert!(!context.base_context.key_decisions.is_empty());
    assert!(context.base_context.key_decisions.iter().any(|d| d.contains("tokio")));

    assert!(!context.base_context.unresolved_issues.is_empty());

    println!("✅ Context extraction working - found task, files, decisions, and issues!");

    Ok(())
}