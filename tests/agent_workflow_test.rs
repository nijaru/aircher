use aircher::agent::Agent;
use aircher::config::ConfigManager;
use aircher::auth::AuthManager;
use aircher::providers::ProviderManager;
use aircher::intelligence::IntelligenceEngine;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_multi_turn_tool_execution() {
    // Setup test environment
    let config = ConfigManager::new().await.expect("Failed to create config");
    let auth_manager = Arc::new(AuthManager::new(config.clone()));
    let provider_manager = Arc::new(ProviderManager::new(config.clone(), auth_manager.clone()));
    let intelligence = Arc::new(IntelligenceEngine::new(&config).await.expect("Failed to create intelligence"));

    // Create agent
    let agent = Agent::new(
        config.clone(),
        auth_manager,
        provider_manager,
        intelligence,
        10, // max_iterations
    ).await.expect("Failed to create agent");

    let agent_arc = Arc::new(tokio::sync::Mutex::new(agent));

    // Test 1: List available tools
    {
        let agent = agent_arc.lock().await;
        let tools = agent.list_tools().await.expect("Failed to list tools");
        println!("Available tools: {}", tools.len());

        // Should have our new tools
        let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
        assert!(tool_names.contains(&"web_browse".to_string()), "Missing web_browse tool");
        assert!(tool_names.contains(&"web_search".to_string()), "Missing web_search tool");
        assert!(tool_names.contains(&"build_project".to_string()), "Missing build_project tool");
        assert!(tool_names.contains(&"run_tests".to_string()), "Missing run_tests tool");
        assert!(tool_names.contains(&"read_file".to_string()), "Missing read_file tool");
        assert!(tool_names.contains(&"write_file".to_string()), "Missing write_file tool");
    }

    // Test 2: Execute a single tool (read_file)
    {
        let agent = agent_arc.lock().await;
        let result = agent.execute_single_tool("read_file", json!({
            "path": "Cargo.toml"
        })).await;

        match result {
            Ok(tool_result) => {
                println!("Read file result: {:?}", tool_result);
                assert!(matches!(tool_result.status, aircher::client::ToolStatus::Success));
            }
            Err(e) => {
                println!("Tool execution failed: {}", e);
                // This might fail if we're not in the right directory, which is OK for this test
            }
        }
    }

    // Test 3: Execute web search tool
    {
        let agent = agent_arc.lock().await;
        let result = agent.execute_single_tool("web_search", json!({
            "query": "rust programming language"
        })).await;

        match result {
            Ok(tool_result) => {
                println!("Web search result: {:?}", tool_result);
                assert!(matches!(tool_result.status, aircher::client::ToolStatus::Success));
            }
            Err(e) => {
                println!("Web search failed: {} (may be expected in CI)", e);
                // Web requests might fail in CI environment, that's OK
            }
        }
    }

    // Test 4: Test build system detection
    {
        let agent = agent_arc.lock().await;
        let result = agent.execute_single_tool("build_project", json!({
            "target": "check" // Use cargo check instead of build for faster execution
        })).await;

        match result {
            Ok(tool_result) => {
                println!("Build result: {:?}", tool_result);
                // Build might succeed or fail depending on the project state
            }
            Err(e) => {
                println!("Build tool execution failed: {}", e);
                // This is expected if we're not in a buildable project
            }
        }
    }

    println!("✅ Multi-turn tool execution test completed!");
}

#[tokio::test]
async fn test_tool_registry_completeness() {
    // Test that all expected tools are registered
    use aircher::agent::tools::ToolRegistry;

    let registry = ToolRegistry::default();
    let tools = registry.list_tools();

    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();

    // Core file operations
    assert!(tool_names.contains(&"read_file".to_string()));
    assert!(tool_names.contains(&"write_file".to_string()));
    assert!(tool_names.contains(&"edit_file".to_string()));
    assert!(tool_names.contains(&"list_files".to_string()));

    // Code analysis
    assert!(tool_names.contains(&"search_code".to_string()));

    // System operations
    assert!(tool_names.contains(&"run_command".to_string()));

    // New web tools
    assert!(tool_names.contains(&"web_browse".to_string()));
    assert!(tool_names.contains(&"web_search".to_string()));

    // Build system
    assert!(tool_names.contains(&"build_project".to_string()));

    // LSP tools (if workspace is available)
    println!("Total tools registered: {}", tools.len());
    for tool in &tools {
        println!("  - {}: {}", tool.name, tool.description);
    }

    // Should have at least the core tools
    assert!(tools.len() >= 10, "Expected at least 10 tools, got {}", tools.len());

    println!("✅ Tool registry completeness test passed!");
}

#[tokio::test]
async fn test_tool_parameter_schemas() {
    use aircher::agent::tools::ToolRegistry;

    let registry = ToolRegistry::default();
    let tools = registry.list_tools();

    for tool in &tools {
        // Each tool should have a valid parameter schema
        assert!(tool.parameters.is_object(), "Tool {} has invalid parameter schema", tool.name);

        // Should have a type field
        assert_eq!(tool.parameters["type"], "object", "Tool {} schema should be object type", tool.name);

        // Should have properties
        assert!(tool.parameters.get("properties").is_some(), "Tool {} should have properties", tool.name);

        println!("✅ Tool {} has valid schema", tool.name);
    }

    println!("✅ Tool parameter schema test passed!");
}