use aircher::agent::tools::ToolRegistry;
use aircher::config::ConfigManager;
use aircher::intelligence::IntelligenceEngine;
use aircher::storage::DatabaseManager;
use aircher::agent::reasoning::AgentReasoning;
use anyhow::Result;
use std::sync::Arc;
use tempfile::tempdir;
use serde_json::json;

/// Test that the intelligence system is created with LocalClient
#[tokio::test]
async fn test_intelligence_creation() -> Result<()> {
    // Setup
    let temp_dir = tempdir()?;
    std::env::set_var("AIRCHER_CONFIG_DIR", temp_dir.path());

    let config = ConfigManager::default();
    let db_manager = DatabaseManager::new(&config).await?;
    let intelligence = IntelligenceEngine::new(&config, &db_manager).await?;

    // Verify intelligence engine was created
    assert!(intelligence.get_suggestions("test query", None).await.is_ok());

    Ok(())
}

/// Test that web browsing tool works
#[tokio::test]
async fn test_web_browsing_tool() -> Result<()> {
    let registry = ToolRegistry::default();
    let web_tool = registry.get("web_browse").expect("web_browse tool should exist");

    // Test fetching a known endpoint
    let params = json!({
        "url": "https://httpbin.org/json",
        "timeout_seconds": 10
    });

    let result = web_tool.execute(params).await?;
    assert!(result.success);

    // Verify we got JSON content
    let content = result.result["content"].as_str().unwrap_or("");
    assert!(content.contains("slideshow"));

    Ok(())
}

/// Test that web search tool works
#[tokio::test]
async fn test_web_search_tool() -> Result<()> {
    let registry = ToolRegistry::default();
    let search_tool = registry.get("web_search").expect("web_search tool should exist");

    // Test searching
    let params = json!({
        "query": "Rust programming language",
        "max_results": 5
    });

    let result = search_tool.execute(params).await?;
    assert!(result.success);

    // Verify we got results
    let results = result.result["results"].as_array();
    assert!(results.is_some());

    Ok(())
}

/// Test that all expected tools are registered
#[tokio::test]
async fn test_tool_registration() -> Result<()> {
    let registry = ToolRegistry::default();
    let tools = registry.list_tools();

    // We should have exactly 20 tools as verified
    assert_eq!(tools.len(), 20, "Expected 20 tools, got {}", tools.len());

    // Verify specific tools exist
    let expected_tools = vec![
        "read_file", "write_file", "edit_file", "list_files",
        "search_code", "run_command",
        "web_browse", "web_search", "build_project",
        "smart_commit", "create_pr", "branch_management", "run_tests",
        "code_completion", "hover_info", "go_to_definition",
        "find_references", "rename_symbol", "get_diagnostics", "format_code"
    ];

    for tool_name in expected_tools {
        assert!(
            tools.iter().any(|t| t.name == tool_name),
            "Tool '{}' not found in registry",
            tool_name
        );
    }

    Ok(())
}

/// Test file operations tools
#[tokio::test]
async fn test_file_operations() -> Result<()> {
    let temp_dir = tempdir()?;
    let test_file = temp_dir.path().join("test.txt");

    let registry = ToolRegistry::default();

    // Test write_file
    let write_tool = registry.get("write_file").expect("write_file should exist");
    let params = json!({
        "path": test_file.to_str().unwrap(),
        "content": "Hello, Aircher!"
    });
    let result = write_tool.execute(params).await?;
    assert!(result.success);

    // Test read_file
    let read_tool = registry.get("read_file").expect("read_file should exist");
    let params = json!({
        "path": test_file.to_str().unwrap()
    });
    let result = read_tool.execute(params).await?;
    assert!(result.success);
    assert!(result.result["content"].as_str().unwrap().contains("Hello, Aircher!"));

    // Test edit_file
    let edit_tool = registry.get("edit_file").expect("edit_file should exist");
    let params = json!({
        "path": test_file.to_str().unwrap(),
        "search": "Hello",
        "replace": "Goodbye"
    });
    let result = edit_tool.execute(params).await?;
    assert!(result.success);

    // Verify edit worked
    let params = json!({
        "path": test_file.to_str().unwrap()
    });
    let result = read_tool.execute(params).await?;
    assert!(result.result["content"].as_str().unwrap().contains("Goodbye, Aircher!"));

    Ok(())
}


/// Test build system detection
#[tokio::test]
async fn test_build_system_detection() -> Result<()> {
    let registry = ToolRegistry::default();
    let build_tool = registry.get("build_project").expect("build_project should exist");

    // The current directory should be detected as a Rust project
    let params = json!({
        "command": "check",  // Use check instead of build to be faster
        "args": []
    });

    let result = build_tool.execute(params).await?;
    // Even if build fails, the tool should successfully detect Cargo
    assert!(result.result.to_string().contains("cargo") || result.success);

    Ok(())
}


/// Test reasoning engine creation
#[tokio::test]
async fn test_reasoning_creation() -> Result<()> {
    let temp_dir = tempdir()?;
    std::env::set_var("AIRCHER_CONFIG_DIR", temp_dir.path());

    let config = ConfigManager::default();
    let db_manager = DatabaseManager::new(&config).await?;
    let intelligence = Arc::new(IntelligenceEngine::new(&config, &db_manager).await?);
    let tools = Arc::new(ToolRegistry::default());

    // Just verify we can create the reasoning engine
    let _reasoning = AgentReasoning::new(intelligence, tools);

    Ok(())
}

/// Test multi-turn tool execution: write -> edit -> read
#[tokio::test]
async fn test_multi_turn_tool_execution() -> Result<()> {
    let temp_dir = tempdir()?;
    let test_file = temp_dir.path().join("multi_turn_test.txt");

    let registry = ToolRegistry::default();

    // Turn 1: Write a file
    let write_tool = registry.get("write_file").expect("write_file should exist");
    let params = json!({
        "path": test_file.to_str().unwrap(),
        "content": "Initial content for multi-turn test"
    });
    let result = write_tool.execute(params).await?;
    assert!(result.success, "Write should succeed");

    // Turn 2: Edit the file
    let edit_tool = registry.get("edit_file").expect("edit_file should exist");
    let params = json!({
        "path": test_file.to_str().unwrap(),
        "search": "Initial",
        "replace": "Modified"
    });
    let result = edit_tool.execute(params).await?;
    assert!(result.success, "Edit should succeed");

    // Turn 3: Read the file back
    let read_tool = registry.get("read_file").expect("read_file should exist");
    let params = json!({
        "path": test_file.to_str().unwrap()
    });
    let result = read_tool.execute(params).await?;
    assert!(result.success, "Read should succeed");

    let content = result.result["content"].as_str().expect("Should have content");
    assert!(content.contains("Modified content for multi-turn test"));

    Ok(())
}
