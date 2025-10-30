/// Integration tests for Week 7-8 hybrid architecture components
/// Tests event bus, mode enforcement, model routing, and agent selection
use aircher::agent::events::{AgentEvent, create_event_bus, EventListener, FileOperation, AgentMode};
use aircher::agent::model_router::{ModelRouter, AgentType, TaskComplexity};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_event_bus_file_changed_emission() {
    // Test that FileChanged events can be published and received
    let bus = create_event_bus();
    let mut listener = EventListener::new(&bus);

    // Publish a FileChanged event
    let test_path = std::path::PathBuf::from("/test/file.rs");
    bus.publish(AgentEvent::FileChanged {
        path: test_path.clone(),
        operation: FileOperation::Write,
        timestamp: std::time::SystemTime::now(),
    });

    // Wait for event with timeout
    let event = listener.wait_for(
        |event| matches!(event, AgentEvent::FileChanged { .. }),
        Duration::from_millis(100)
    ).await;

    assert!(event.is_some(), "Should receive FileChanged event");

    if let Some(AgentEvent::FileChanged { path, operation, .. }) = event {
        assert_eq!(path, test_path);
        assert!(matches!(operation, FileOperation::Write));
    } else {
        panic!("Expected FileChanged event");
    }
}

#[tokio::test]
async fn test_event_bus_multiple_subscribers() {
    // Test that multiple subscribers can receive the same event
    let bus = create_event_bus();
    let mut listener1 = EventListener::new(&bus);
    let mut listener2 = EventListener::new(&bus);

    // Publish event
    bus.publish(AgentEvent::FileChanged {
        path: std::path::PathBuf::from("/test.rs"),
        operation: FileOperation::Edit,
        timestamp: std::time::SystemTime::now(),
    });

    // Both listeners should receive it
    let result1 = timeout(Duration::from_millis(100), listener1.next()).await;
    let result2 = timeout(Duration::from_millis(100), listener2.next()).await;

    assert!(result1.is_ok() && result1.unwrap().is_some());
    assert!(result2.is_ok() && result2.unwrap().is_some());
}

#[test]
fn test_plan_mode_blocks_write_tools() {
    let mode = AgentMode::Plan;

    // Plan mode should allow read-only tools
    assert!(mode.is_tool_allowed("read_file"));
    assert!(mode.is_tool_allowed("search_code"));
    assert!(mode.is_tool_allowed("analyze_code"));

    // Plan mode should block modification tools
    assert!(!mode.is_tool_allowed("write_file"));
    assert!(!mode.is_tool_allowed("edit_file"));
    assert!(!mode.is_tool_allowed("run_command"));
}

#[test]
fn test_build_mode_allows_all_tools() {
    let mode = AgentMode::Build;

    // Build mode should allow read tools
    assert!(mode.is_tool_allowed("read_file"));
    assert!(mode.is_tool_allowed("search_code"));

    // Build mode should allow write tools
    assert!(mode.is_tool_allowed("write_file"));
    assert!(mode.is_tool_allowed("edit_file"));
    assert!(mode.is_tool_allowed("run_command"));
}

#[test]
fn test_mode_system_prompts() {
    let plan_prompt = AgentMode::Plan.system_prompt();
    let build_prompt = AgentMode::Build.system_prompt();

    // Plan mode prompt should mention read-only
    assert!(plan_prompt.contains("read-only") || plan_prompt.contains("PLAN mode"));

    // Build mode prompt should mention modification
    assert!(build_prompt.contains("BUILD mode") || build_prompt.contains("modification"));

    // Prompts should be different
    assert_ne!(plan_prompt, build_prompt);
}

#[test]
fn test_model_router_explorer_routing() {
    let router = ModelRouter::new();

    // Explorer with low complexity should use Haiku
    let config = router.select_model(AgentType::Explorer, TaskComplexity::Low, None);
    assert_eq!(config.model, "claude-haiku-4-5");
    assert_eq!(config.provider, "anthropic");

    // Explorer with medium complexity should use Sonnet
    let config = router.select_model(AgentType::Explorer, TaskComplexity::Medium, None);
    assert_eq!(config.model, "claude-sonnet-4-5");
    assert_eq!(config.provider, "anthropic");

    // Explorer with high complexity should use Sonnet (NOT Opus)
    let config = router.select_model(AgentType::Explorer, TaskComplexity::High, None);
    assert_eq!(config.model, "claude-sonnet-4-5");
}

#[test]
fn test_model_router_builder_always_sonnet() {
    let router = ModelRouter::new();

    // Builder should always use Sonnet regardless of complexity
    let low = router.select_model(AgentType::Builder, TaskComplexity::Low, None);
    assert_eq!(low.model, "claude-sonnet-4-5");

    let medium = router.select_model(AgentType::Builder, TaskComplexity::Medium, None);
    assert_eq!(medium.model, "claude-sonnet-4-5");

    let high = router.select_model(AgentType::Builder, TaskComplexity::High, None);
    assert_eq!(high.model, "claude-sonnet-4-5");
}

#[test]
fn test_model_router_subagents_use_haiku() {
    let router = ModelRouter::new();

    // All sub-agents should use Haiku (cheap parallelization)
    let file_searcher = router.select_model(AgentType::FileSearcher, TaskComplexity::Medium, None);
    assert_eq!(file_searcher.model, "claude-haiku-4-5");

    let pattern_finder = router.select_model(AgentType::PatternFinder, TaskComplexity::High, None);
    assert_eq!(pattern_finder.model, "claude-haiku-4-5");
}

#[test]
fn test_model_router_single_model_override() {
    use aircher::agent::model_router::ModelConfig;

    // Create router with single model override
    let sonnet = ModelConfig::claude_sonnet_4_5();
    let router = ModelRouter::with_single_model(sonnet);

    // All tasks should use the override model
    let explorer = router.select_model(AgentType::Explorer, TaskComplexity::Low, None);
    assert_eq!(explorer.model, "claude-sonnet-4-5");

    let builder = router.select_model(AgentType::Builder, TaskComplexity::High, None);
    assert_eq!(builder.model, "claude-sonnet-4-5");

    // Even sub-agents should use override
    let searcher = router.select_model(AgentType::FileSearcher, TaskComplexity::Low, None);
    assert_eq!(searcher.model, "claude-sonnet-4-5");
}

#[test]
fn test_model_router_cost_estimation() {
    use aircher::agent::model_router::ModelConfig;

    let haiku = ModelConfig::claude_haiku_4_5();
    let sonnet = ModelConfig::claude_sonnet_4_5();
    let opus = ModelConfig::claude_opus_4_1();

    // Verify cost ordering (haiku < sonnet < opus)
    let haiku_cost = haiku.estimate_cost(1000, 1000);
    let sonnet_cost = sonnet.estimate_cost(1000, 1000);
    let opus_cost = opus.estimate_cost(1000, 1000);

    assert!(haiku_cost < sonnet_cost);
    assert!(sonnet_cost < opus_cost);
}

#[tokio::test]
async fn test_model_router_usage_recording() {
    use aircher::agent::model_router::ModelConfig;

    let router = ModelRouter::new();
    let model = ModelConfig::claude_sonnet_4_5();

    // Record some usage
    router.record_usage(&model, 1000, 500).await;
    router.record_usage(&model, 2000, 1000).await;

    // Get stats report
    let report = router.generate_report().await;

    // Should contain model name and usage info
    assert!(report.contains("claude-sonnet-4-5"));
    assert!(report.contains("anthropic"));
}

#[test]
fn test_agent_mode_subagent_spawning_rules() {
    // Plan mode CAN spawn sub-agents (for research)
    assert!(AgentMode::Plan.can_spawn_subagents());

    // Build mode CANNOT spawn sub-agents (avoid 15x waste)
    assert!(!AgentMode::Build.can_spawn_subagents());
}

#[test]
fn test_file_operation_types() {
    use aircher::agent::events::FileOperation;

    // Verify all file operation types exist
    let _write = FileOperation::Write;
    let _edit = FileOperation::Edit;
    let _delete = FileOperation::Delete;
    let _create = FileOperation::Create;
}

#[tokio::test]
async fn test_event_bus_mode_changed_event() {
    let bus = create_event_bus();
    let mut listener = EventListener::new(&bus);

    // Publish ModeChanged event
    bus.publish(AgentEvent::ModeChanged {
        old_mode: AgentMode::Plan,
        new_mode: AgentMode::Build,
        reason: "User requested implementation".to_string(),
        timestamp: std::time::SystemTime::now(),
    });

    // Should receive event
    let result = timeout(Duration::from_millis(100), listener.next()).await;
    assert!(result.is_ok());
}

#[test]
fn test_model_config_context_windows() {
    use aircher::agent::model_router::ModelConfig;

    let haiku = ModelConfig::claude_haiku_4_5();
    let sonnet = ModelConfig::claude_sonnet_4_5();
    let opus = ModelConfig::claude_opus_4_1();

    // All should have 200K context window
    assert_eq!(haiku.max_context, 200_000);
    assert_eq!(sonnet.max_context, 200_000);
    assert_eq!(opus.max_context, 200_000);
}

#[test]
fn test_model_router_set_custom_route() {
    use aircher::agent::model_router::ModelConfig;

    let mut router = ModelRouter::new();

    // Set custom route: Explorer/High should use Opus
    let opus = ModelConfig::claude_opus_4_1();
    router.set_route(AgentType::Explorer, TaskComplexity::High, opus.clone());

    // Verify custom route is used
    let selected = router.select_model(AgentType::Explorer, TaskComplexity::High, None);
    assert_eq!(selected.model, "claude-opus-4-1");
}

#[test]
fn test_model_router_clear_single_model() {
    use aircher::agent::model_router::ModelConfig;

    let mut router = ModelRouter::with_single_model(ModelConfig::claude_sonnet_4_5());

    // Initially should use override
    let before = router.select_model(AgentType::Explorer, TaskComplexity::Low, None);
    assert_eq!(before.model, "claude-sonnet-4-5");

    // Clear override
    router.clear_single_model();

    // Now should use routing table (Haiku for Explorer/Low)
    let after = router.select_model(AgentType::Explorer, TaskComplexity::Low, None);
    assert_eq!(after.model, "claude-haiku-4-5");
}
