// Week 5 Day 6-7: End-to-end validation tests for all 3 memory systems
//
// Tests:
// 1. DynamicContextManager with all systems connected
// 2. Realistic workload filling context window
// 3. Pruning behavior validation
// 4. Episodic memory capturing removed items
// 5. Knowledge graph queries for relevant code
// 6. Continuous work without restart

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::Mutex;

use aircher::intelligence::{
    DynamicContextManager, DuckDBMemory, KnowledgeGraph, NodeType, EdgeType,
    FileInteraction, ToolExecution, ContextSnapshot,
};

#[tokio::test]
async fn test_memory_system_integration_basic() -> Result<()> {
    // Setup: Create all 3 memory systems
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.duckdb");
    let session_id = "test-session-basic".to_string();

    // 1. Create episodic memory (DuckDB)
    let episodic = Arc::new(Mutex::new(
        DuckDBMemory::new(&db_path).await?
    ));

    // 2. Create knowledge graph
    let repo_root = PathBuf::from("/test/repo");
    let knowledge_graph = Arc::new(Mutex::new(
        KnowledgeGraph::new(repo_root.clone())
    ));

    // Populate knowledge graph with some test data
    {
        let mut graph = knowledge_graph.lock().await;

        // Add a test file
        let file_path = repo_root.join("src/main.rs");
        graph.add_node(NodeType::File {
            path: file_path.clone(),
            language: "rust".to_string(),
            line_count: 50,
        });

        // Add a test function
        graph.add_node(NodeType::Function {
            name: "process_data".to_string(),
            signature: "fn process_data(input: &str) -> Result<String>".to_string(),
            line: 10,
            file_path: file_path.clone(),
        });

        // Add another function
        graph.add_node(NodeType::Function {
            name: "validate_input".to_string(),
            signature: "fn validate_input(data: &str) -> bool".to_string(),
            line: 25,
            file_path: file_path.clone(),
        });
    }

    // 3. Create dynamic context manager with both systems
    let mut context_manager = DynamicContextManager::new(session_id.clone(), 100000);
    context_manager.set_episodic_memory(episodic.clone());
    context_manager.set_knowledge_graph(knowledge_graph.clone());

    // Test basic operations
    context_manager.add_user_message("Fix the validation bug".to_string(), None);
    context_manager.add_assistant_response("I'll help with that".to_string(), vec![]);

    let stats = context_manager.stats();
    assert_eq!(stats.total_items, 2);
    assert!(stats.total_tokens > 0);

    // Test knowledge graph query
    context_manager.fetch_relevant_code("validate").await?;
    let stats_after = context_manager.stats();
    assert!(stats_after.total_items > stats.total_items, "Should have added code snippets");

    Ok(())
}

#[tokio::test]
async fn test_context_pruning_with_episodic_memory() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_pruning.duckdb");
    let session_id = "test-session-pruning".to_string();

    // Create episodic memory
    let episodic = Arc::new(Mutex::new(
        DuckDBMemory::new(&db_path).await?
    ));

    // Create context manager with small token limit (to trigger pruning)
    let mut context_manager = DynamicContextManager::new(session_id.clone(), 1000);
    context_manager.set_episodic_memory(episodic.clone());

    // Fill context to >80% capacity
    for i in 0..10 {
        context_manager.add_user_message(
            format!("This is message number {} with some content to fill tokens", i),
            None,
        );
        context_manager.add_assistant_response(
            format!("Response to message {} with additional content", i),
            vec![],
        );
    }

    let stats_before = context_manager.stats();
    println!("Before pruning: {} tokens, {} items", stats_before.total_tokens, stats_before.total_items);

    // Should need pruning now
    context_manager.maybe_prune().await?;

    let stats_after = context_manager.stats();
    println!("After pruning: {} tokens, {} items", stats_after.total_tokens, stats_after.total_items);

    // Verify pruning happened
    assert!(stats_after.total_tokens < stats_before.total_tokens, "Should have removed tokens");
    assert!(stats_after.total_items < stats_before.total_items, "Should have removed items");
    assert!(stats_after.utilization < 80.0, "Should be back under 80%");

    // Verify episodic memory received snapshot
    // (We don't have a direct query for snapshots yet, but it should not error)

    Ok(())
}

#[tokio::test]
async fn test_continuous_work_simulation() -> Result<()> {
    // Simulate a realistic workload: multiple tasks, file interactions, context filling
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_continuous.duckdb");
    let session_id = "test-session-continuous".to_string();

    // Setup all 3 memory systems
    let episodic = Arc::new(Mutex::new(
        DuckDBMemory::new(&db_path).await?
    ));

    let repo_root = PathBuf::from("/test/repo");
    let knowledge_graph = Arc::new(Mutex::new(
        KnowledgeGraph::new(repo_root.clone())
    ));

    // Populate knowledge graph
    {
        let mut graph = knowledge_graph.lock().await;

        // Add multiple files
        for file_name in &["auth.rs", "api.rs", "database.rs"] {
            let file_path = repo_root.join("src").join(file_name);
            graph.add_node(NodeType::File {
                path: file_path.clone(),
                language: "rust".to_string(),
                line_count: 100,
            });

            // Add functions for each file
            graph.add_node(NodeType::Function {
                name: format!("process_{}", file_name.replace(".rs", "")),
                signature: format!("fn process_{}()", file_name.replace(".rs", "")),
                line: 20,
                file_path: file_path.clone(),
            });
        }
    }

    let mut context_manager = DynamicContextManager::new(session_id.clone(), 5000);
    context_manager.set_episodic_memory(episodic.clone());
    context_manager.set_knowledge_graph(knowledge_graph.clone());

    // Simulate Task 1: Authentication work
    context_manager.set_current_task("task-1-auth".to_string());
    context_manager.add_user_message("Fix authentication bug".to_string(), None);
    context_manager.add_assistant_response("I'll check the auth module".to_string(), vec!["read_file".to_string()]);
    context_manager.add_tool_result("read_file".to_string(), Some("src/auth.rs".to_string()), "File contents...".to_string());

    // Record file interaction to episodic memory
    {
        let memory = episodic.lock().await;
        memory.record_file_interaction(FileInteraction {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            session_id: session_id.clone(),
            task_id: Some("task-1-auth".to_string()),
            file_path: "src/auth.rs".to_string(),
            operation: "read".to_string(),
            line_range: None,
            success: true,
            context: Some("Investigating authentication bug".to_string()),
            changes_summary: None,
        }).await?;
    }

    // Add more context to fill up
    for i in 0..15 {
        context_manager.add_user_message(format!("Follow-up question {}", i), None);
        context_manager.add_assistant_response(format!("Answer {}", i), vec![]);
    }

    let stats_mid = context_manager.stats();
    println!("Mid-task: {} tokens, {}% full", stats_mid.total_tokens, stats_mid.utilization);

    // Should trigger pruning
    context_manager.maybe_prune().await?;

    let stats_after_prune = context_manager.stats();
    println!("After pruning: {} tokens, {}% full", stats_after_prune.total_tokens, stats_after_prune.utilization);

    // Simulate Task 2: API work (should still have context)
    context_manager.set_current_task("task-2-api".to_string());
    context_manager.add_user_message("Now let's work on the API".to_string(), None);

    // Fetch relevant code from knowledge graph
    context_manager.fetch_relevant_code("api").await?;

    // Check file context from episodic memory
    let file_context = context_manager.get_file_context("src/auth.rs").await?;
    println!("File context: {:?}", file_context);

    // Verify we can continue working
    let final_stats = context_manager.stats();
    assert!(final_stats.total_items > 0, "Should still have context");
    assert!(final_stats.utilization < 90.0, "Should be under capacity");

    // Verify episodic memory has records
    {
        let memory = episodic.lock().await;
        let interactions = memory.get_file_interactions("src/auth.rs", 10).await?;
        assert!(!interactions.is_empty(), "Should have recorded file interactions");
    }

    Ok(())
}

#[tokio::test]
async fn test_knowledge_graph_integration() -> Result<()> {
    let session_id = "test-session-kg".to_string();
    let repo_root = PathBuf::from("/test/repo");

    // Create knowledge graph
    let knowledge_graph = Arc::new(Mutex::new(
        KnowledgeGraph::new(repo_root.clone())
    ));

    // Build a realistic graph structure
    {
        let mut graph = knowledge_graph.lock().await;

        let main_file = repo_root.join("src/main.rs");
        let main_file_idx = graph.add_node(NodeType::File {
            path: main_file.clone(),
            language: "rust".to_string(),
            line_count: 80,
        });

        let auth_file = repo_root.join("src/auth.rs");
        let auth_file_idx = graph.add_node(NodeType::File {
            path: auth_file.clone(),
            language: "rust".to_string(),
            line_count: 120,
        });

        // Add functions
        let main_fn = NodeType::Function {
            name: "main".to_string(),
            signature: "fn main()".to_string(),
            line: 5,
            file_path: main_file.clone(),
        };
        let main_fn_idx = graph.add_node(main_fn);

        let auth_fn = NodeType::Function {
            name: "authenticate_user".to_string(),
            signature: "fn authenticate_user(username: &str, password: &str) -> Result<Token>".to_string(),
            line: 15,
            file_path: auth_file.clone(),
        };
        let auth_fn_idx = graph.add_node(auth_fn);

        // Add call relationship (main function calls auth function)
        graph.add_edge(main_fn_idx, auth_fn_idx, EdgeType::Calls);

        let stats = graph.stats();
        println!("Knowledge graph: {} nodes, {} edges", stats.node_count, stats.edge_count);
    }

    // Create context manager with knowledge graph
    let mut context_manager = DynamicContextManager::new(session_id.clone(), 100000);
    context_manager.set_knowledge_graph(knowledge_graph.clone());

    // Query for authentication-related code
    context_manager.fetch_relevant_code("authenticate").await?;

    let stats = context_manager.stats();
    assert!(stats.total_items > 0, "Should have fetched code from knowledge graph");

    // Verify graph queries work
    {
        let graph = knowledge_graph.lock().await;

        // Query file contents
        let contents = graph.get_file_contents(&auth_file)?;
        assert!(!contents.is_empty(), "Should find symbols in file");

        // Find symbol
        let symbols = graph.find_symbol("authenticate")?;
        assert!(!symbols.is_empty(), "Should find authenticate symbol");
    }

    Ok(())
}

#[tokio::test]
async fn test_episodic_memory_pattern_learning() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_patterns.duckdb");
    let session_id = "test-session-patterns".to_string();

    let episodic = Arc::new(Mutex::new(
        DuckDBMemory::new(&db_path).await?
    ));

    // Record multiple tool executions
    {
        let memory = episodic.lock().await;

        for i in 0..5 {
            memory.record_tool_execution(ToolExecution {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                session_id: session_id.clone(),
                task_id: Some("task-refactor".to_string()),
                tool_name: "edit_file".to_string(),
                parameters: serde_json::json!({ "file": format!("src/module{}.rs", i) }),
                result: Some(serde_json::json!({ "success": true })),
                success: true,
                error_message: None,
                duration_ms: Some(150),
                context_tokens: Some(1200),
            }).await?;
        }

        // Query tool executions
        let executions = memory.get_tool_executions(&session_id, 10).await?;
        assert_eq!(executions.len(), 5, "Should have recorded 5 tool executions");

        // Check success rate
        let success_count = executions.iter().filter(|e| e.success).count();
        assert_eq!(success_count, 5, "All should be successful");
    }

    // Record co-edited files (pattern learning)
    {
        let memory = episodic.lock().await;
        let task_id = "task-feature".to_string();

        // Simulate editing files together (within 5 minutes)
        for file in &["src/api.rs", "src/models.rs", "tests/api_tests.rs"] {
            memory.record_file_interaction(FileInteraction {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                session_id: session_id.clone(),
                task_id: Some(task_id.clone()),
                file_path: file.to_string(),
                operation: "edit".to_string(),
                line_range: Some(serde_json::json!({"start": 10, "end": 50})),
                success: true,
                context: Some("Adding new API endpoint".to_string()),
                changes_summary: Some("Added new route".to_string()),
            }).await?;
        }

        // Query file interactions
        let interactions = memory.get_file_interactions("src/api.rs", 10).await?;
        assert!(!interactions.is_empty(), "Should have file interactions");
    }

    Ok(())
}

#[tokio::test]
async fn test_relevance_scoring_with_task_context() -> Result<()> {
    let session_id = "test-session-relevance".to_string();

    let mut context_manager = DynamicContextManager::new(session_id.clone(), 10000);

    // Set current task
    context_manager.set_current_task("task-bug-fix".to_string());

    // Add messages for current task
    context_manager.add_user_message("Fix the null pointer bug".to_string(), None);
    context_manager.add_assistant_response("Investigating...".to_string(), vec!["read_file".to_string()]);
    context_manager.add_tool_result("read_file".to_string(), Some("src/buggy.rs".to_string()), "Found the bug!".to_string());

    // Add older messages from different task
    for i in 0..5 {
        context_manager.add_user_message(format!("Old message {}", i), None);
        context_manager.add_assistant_response(format!("Old response {}", i), vec![]);
    }

    // Force high utilization
    for i in 0..20 {
        context_manager.add_user_message(format!("Filler message {}", i), None);
    }

    let stats_before = context_manager.stats();
    println!("Before pruning: {} items", stats_before.total_items);

    // Trigger pruning
    context_manager.maybe_prune().await?;

    let stats_after = context_manager.stats();
    println!("After pruning: {} items", stats_after.total_items);

    // Current task items should have higher relevance and be retained
    // Old unrelated items should be pruned first
    assert!(stats_after.total_items < stats_before.total_items, "Should have pruned items");

    Ok(())
}

#[tokio::test]
async fn test_full_memory_system_workflow() -> Result<()> {
    // Complete workflow: user works on multiple tasks, context fills and prunes,
    // all memory systems work together seamlessly

    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_full_workflow.duckdb");
    let session_id = "test-session-workflow".to_string();

    // Initialize all 3 systems
    let episodic = Arc::new(Mutex::new(DuckDBMemory::new(&db_path).await?));
    let repo_root = PathBuf::from("/test/repo");
    let knowledge_graph = Arc::new(Mutex::new(KnowledgeGraph::new(repo_root.clone())));

    // Populate knowledge graph with realistic structure
    {
        let mut graph = knowledge_graph.lock().await;

        for module in &["auth", "api", "database", "models", "utils"] {
            let file_path = repo_root.join("src").join(format!("{}.rs", module));
            graph.add_node(NodeType::File {
                path: file_path.clone(),
                language: "rust".to_string(),
                line_count: 200,
            });

            // Add 3 functions per module
            for i in 0..3 {
                graph.add_node(NodeType::Function {
                    name: format!("{}_{}", module, i),
                    signature: format!("fn {}_{}() -> Result<()>", module, i),
                    line: 20 + (i * 30),
                    file_path: file_path.clone(),
                });
            }
        }

        println!("Knowledge graph initialized: {:?}", graph.stats());
    }

    let mut context_manager = DynamicContextManager::new(session_id.clone(), 8000);
    context_manager.set_episodic_memory(episodic.clone());
    context_manager.set_knowledge_graph(knowledge_graph.clone());

    // === Phase 1: Authentication bug fix ===
    context_manager.set_current_task("task-1-auth-bug".to_string());
    context_manager.add_user_message("Fix authentication timeout".to_string(), None);
    context_manager.fetch_relevant_code("auth").await?;
    context_manager.add_assistant_response("Found the issue in auth.rs".to_string(), vec!["edit_file".to_string()]);
    context_manager.add_tool_result("edit_file".to_string(), Some("src/auth.rs".to_string()), "Fixed timeout".to_string());

    // Record to episodic memory
    {
        let memory = episodic.lock().await;
        memory.record_file_interaction(FileInteraction {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            session_id: session_id.clone(),
            task_id: Some("task-1-auth-bug".to_string()),
            file_path: "src/auth.rs".to_string(),
            operation: "edit".to_string(),
            line_range: Some(serde_json::json!({"start": 45, "end": 52})),
            success: true,
            context: Some("Fixed authentication timeout".to_string()),
            changes_summary: Some("Increased timeout to 30s".to_string()),
        }).await?;
    }

    println!("Phase 1 complete: {:?}", context_manager.stats());

    // === Phase 2: API endpoint work ===
    context_manager.set_current_task("task-2-api-endpoint".to_string());
    context_manager.add_user_message("Add new /health endpoint".to_string(), None);
    context_manager.fetch_relevant_code("api").await?;

    // Add lots of conversation to fill context
    for i in 0..15 {
        context_manager.add_user_message(format!("What about {}?", i), None);
        context_manager.add_assistant_response(format!("Considering {}...", i), vec![]);
    }

    context_manager.maybe_prune().await?;
    println!("Phase 2 after pruning: {:?}", context_manager.stats());

    // === Phase 3: Database migration ===
    context_manager.set_current_task("task-3-db-migration".to_string());
    context_manager.add_user_message("Run database migration".to_string(), None);

    // Check if we've worked on database before
    let db_context = context_manager.get_file_context("src/database.rs").await?;
    if let Some(ctx) = db_context {
        println!("Previous database work: {}", ctx);
    }

    context_manager.fetch_relevant_code("database").await?;
    context_manager.add_tool_result("run_migration".to_string(), None, "Migration successful".to_string());

    println!("Phase 3 complete: {:?}", context_manager.stats());

    // === Validation ===
    let final_stats = context_manager.stats();
    assert!(final_stats.total_items > 0, "Should have retained context");
    assert!(final_stats.utilization < 90.0, "Should be under capacity");
    assert!(final_stats.pruning_count > 0, "Should have pruned at least once");

    // Check episodic memory has full history
    {
        let memory = episodic.lock().await;
        let auth_history = memory.get_file_interactions("src/auth.rs", 10).await?;
        assert!(!auth_history.is_empty(), "Should have auth file history");
    }

    // Check knowledge graph still queryable
    {
        let graph = knowledge_graph.lock().await;
        let api_symbols = graph.find_symbol("api")?;
        assert!(!api_symbols.is_empty(), "Should find API symbols");
    }

    println!("✅ Full workflow validation complete!");
    Ok(())
}
