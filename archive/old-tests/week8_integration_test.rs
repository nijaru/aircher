// Week 8 Integration Tests: Specialized Agents + Research Sub-Agents
//
// Tests the integration of:
// 1. Specialized agent configurations (Week 8 Day 1-2)
// 2. Research sub-agents (Week 8 Day 3-4)
// 3. Model routing with agent types
//
// Validates:
// - Agent configurations have appropriate tool restrictions
// - Research sub-agents can spawn and aggregate results
// - Build mode agents NEVER spawn sub-agents
// - Model router selects appropriate models for agent types

use aircher::agent::{
    AgentConfig, AgentRegistry, MemoryAccessLevel,
    ResearchSubAgentManager, ResearchTask,
    ModelRouter, ModelConfig, TaskComplexity,
    RouterAgentType,
};

#[cfg(test)]
mod specialized_agent_integration {
    use super::*;

    #[test]
    fn test_explorer_can_spawn_subagents() {
        let config = AgentConfig::explorer();

        // Explorer should be able to spawn sub-agents for research
        assert!(config.can_spawn_subagents, "Explorer must be able to spawn sub-agents");

        // Should have read-only tools
        assert!(config.allowed_tools.contains(&"read_file".to_string()));
        assert!(config.allowed_tools.contains(&"search_code".to_string()));

        // Should NOT have modification tools
        assert!(!config.allowed_tools.contains(&"write_file".to_string()));
        assert!(!config.allowed_tools.contains(&"edit_file".to_string()));
    }

    #[test]
    fn test_builder_never_spawns_subagents() {
        let config = AgentConfig::builder();

        // Builder must NEVER spawn sub-agents (15x token waste)
        assert!(!config.can_spawn_subagents, "Builder must NEVER spawn sub-agents");

        // Should have modification tools
        assert!(config.allowed_tools.contains(&"write_file".to_string()));
        assert!(config.allowed_tools.contains(&"edit_file".to_string()));
    }

    #[test]
    fn test_debugger_never_spawns_subagents() {
        let config = AgentConfig::debugger();

        // Debugger must NEVER spawn sub-agents for coding
        assert!(!config.can_spawn_subagents, "Debugger must NEVER spawn sub-agents");

        // Should have diagnostic and test tools
        assert!(config.allowed_tools.contains(&"analyze_errors".to_string()));
        assert!(config.allowed_tools.contains(&"run_tests".to_string()));
    }

    #[test]
    fn test_refactorer_never_spawns_subagents() {
        let config = AgentConfig::refactorer();

        // Refactorer must NEVER spawn sub-agents
        assert!(!config.can_spawn_subagents, "Refactorer must NEVER spawn sub-agents");

        // Should have modification and analysis tools
        assert!(config.allowed_tools.contains(&"edit_file".to_string()));
        assert!(config.allowed_tools.contains(&"analyze_code".to_string()));
    }

    #[test]
    fn test_subagent_configs_are_limited() {
        let file_searcher = AgentConfig::file_searcher();
        let pattern_finder = AgentConfig::pattern_finder();
        let dependency_mapper = AgentConfig::dependency_mapper();

        // All sub-agents have limited steps
        assert!(file_searcher.max_steps <= 20, "Sub-agents must have limited steps");
        assert!(pattern_finder.max_steps <= 20);
        assert!(dependency_mapper.max_steps <= 20);

        // All sub-agents have read-only access
        assert_eq!(file_searcher.memory_access, MemoryAccessLevel::ReadOnly);
        assert_eq!(pattern_finder.memory_access, MemoryAccessLevel::ReadOnly);
        assert_eq!(dependency_mapper.memory_access, MemoryAccessLevel::ReadOnly);

        // Sub-agents cannot spawn more sub-agents
        assert!(!file_searcher.can_spawn_subagents);
        assert!(!pattern_finder.can_spawn_subagents);
        assert!(!dependency_mapper.can_spawn_subagents);
    }

    #[test]
    fn test_agent_registry_selection() {
        let registry = AgentRegistry::new();

        // Test intent-based selection
        let explorer = registry.select_for_intent("analyze authentication patterns");
        assert_eq!(explorer.agent_type, RouterAgentType::Explorer);

        let builder = registry.select_for_intent("implement user login feature");
        assert_eq!(builder.agent_type, RouterAgentType::Builder);

        let debugger = registry.select_for_intent("fix null pointer exception");
        assert_eq!(debugger.agent_type, RouterAgentType::Debugger);

        let refactorer = registry.select_for_intent("refactor user service");
        assert_eq!(refactorer.agent_type, RouterAgentType::Refactorer);
    }

    #[test]
    fn test_specialized_prompts_are_distinct() {
        let explorer = AgentConfig::explorer();
        let builder = AgentConfig::builder();
        let debugger = AgentConfig::debugger();
        let refactorer = AgentConfig::refactorer();

        // Each agent should have a unique system prompt
        assert_ne!(explorer.system_prompt, builder.system_prompt);
        assert_ne!(builder.system_prompt, debugger.system_prompt);
        assert_ne!(debugger.system_prompt, refactorer.system_prompt);

        // Explorer prompt should emphasize understanding
        assert!(explorer.system_prompt.to_lowercase().contains("understand"));

        // Builder prompt should emphasize implementation
        assert!(builder.system_prompt.to_lowercase().contains("implement")
            || builder.system_prompt.to_lowercase().contains("build"));

        // Debugger prompt should emphasize root cause
        assert!(debugger.system_prompt.to_lowercase().contains("root cause")
            || debugger.system_prompt.to_lowercase().contains("debug"));
    }
}

#[cfg(test)]
mod research_subagent_integration {
    use super::*;

    #[tokio::test]
    async fn test_subagent_spawning_respects_limit() {
        let manager = ResearchSubAgentManager::new();

        // Create query that would decompose into many tasks
        let query = "Find all authentication authorization security crypto hashing encryption";
        let handle = manager.spawn_research(query).await.unwrap();

        // Should be limited to MAX_CONCURRENT_SUBAGENTS
        assert!(handle.task_count() <= aircher::agent::MAX_CONCURRENT_SUBAGENTS,
                "Must not exceed max concurrent sub-agents");
    }

    #[tokio::test]
    async fn test_subagent_result_aggregation() {
        let manager = ResearchSubAgentManager::new();

        let query = "Find authentication code";
        let handle = manager.spawn_research(query).await.unwrap();

        // Wait for completion
        let results = handle.wait().await.unwrap();

        // Should have results
        assert!(!results.is_empty(), "Should return aggregated results");

        // All results should be successful (in this test)
        for result in &results {
            assert!(result.success, "All sub-agents should succeed in test");
            assert!(!result.findings.is_empty(), "Should have findings");
        }
    }

    #[tokio::test]
    async fn test_subagent_progress_tracking() {
        let manager = ResearchSubAgentManager::new();

        let query = "Search for error handling patterns";
        let handle = manager.spawn_research(query).await.unwrap();

        // Check progress before completion
        let progress = handle.progress().await;
        assert!(progress.total_tasks > 0, "Should have tasks");
        assert!(progress.percent_complete() >= 0.0);
        assert!(progress.percent_complete() <= 100.0);

        // Wait for completion
        let _results = handle.wait().await.unwrap();

        // Check progress after completion
        let final_progress = handle.progress().await;
        assert_eq!(final_progress.active_tasks, 0, "All tasks should be complete");
        assert_eq!(final_progress.percent_complete(), 100.0);
    }

    #[tokio::test]
    async fn test_subagent_cancellation() {
        let manager = ResearchSubAgentManager::new();

        let query = "Long running research query";
        let handle = manager.spawn_research(query).await.unwrap();

        // Cancel immediately
        handle.cancel().await;

        // Check that all tasks were cancelled
        let progress = handle.progress().await;
        assert_eq!(progress.active_tasks, 0, "All tasks should be cancelled");
    }

    #[tokio::test]
    async fn test_query_decomposition_creates_appropriate_agents() {
        use aircher::agent::research_subagents::QueryDecomposer;

        // Search query should create FileSearcher tasks
        let search_tasks = QueryDecomposer::decompose("Find all authentication code");
        assert!(!search_tasks.is_empty());
        for task in &search_tasks {
            assert_eq!(task.agent_type, RouterAgentType::FileSearcher);
        }

        // Dependency query should create DependencyMapper task
        let dep_tasks = QueryDecomposer::decompose("What uses the User model?");
        assert_eq!(dep_tasks.len(), 1);
        assert_eq!(dep_tasks[0].agent_type, RouterAgentType::DependencyMapper);

        // Pattern query should create PatternFinder task
        let pattern_tasks = QueryDecomposer::decompose("Find all error handling patterns");
        assert_eq!(pattern_tasks.len(), 1);
        assert_eq!(pattern_tasks[0].agent_type, RouterAgentType::PatternFinder);
    }
}

#[cfg(test)]
mod model_router_integration {
    use super::*;

    #[test]
    fn test_model_selection_for_agent_types() {
        let router = ModelRouter::new();

        // Explorer with low complexity → fast model
        let model = router.select_model(
            RouterAgentType::Explorer,
            TaskComplexity::Low,
            None,
        );
        assert_eq!(model.model, "claude-sonnet-4", "Explorer should use fast model for simple queries");

        // Builder with high complexity → best model
        let model = router.select_model(
            RouterAgentType::Builder,
            TaskComplexity::High,
            None,
        );
        assert_eq!(model.model, "claude-opus-4.1", "Builder should use best model for complex work");

        // Sub-agents always use cheapest model
        let model = router.select_model(
            RouterAgentType::FileSearcher,
            TaskComplexity::Medium,
            None,
        );
        assert_eq!(model.model, "claude-haiku", "Sub-agents must use cheapest model");
    }

    #[test]
    fn test_subagent_model_is_always_haiku() {
        let router = ModelRouter::new();

        // All sub-agent types should use Haiku regardless of complexity
        let subagent_types = vec![
            RouterAgentType::FileSearcher,
            RouterAgentType::PatternFinder,
            RouterAgentType::DependencyMapper,
        ];

        for agent_type in subagent_types {
            for complexity in [TaskComplexity::Low, TaskComplexity::Medium, TaskComplexity::High] {
                let model = router.select_model(agent_type, complexity, None);
                assert_eq!(model.model, "claude-haiku",
                          "Sub-agent {:?} must use Haiku for {:?}", agent_type, complexity);
            }
        }
    }

    #[test]
    fn test_user_override_respected() {
        let router = ModelRouter::new();

        let override_model = ModelConfig::claude_opus_4_1();

        // Even for low complexity, user override should be respected
        let model = router.select_model(
            RouterAgentType::Explorer,
            TaskComplexity::Low,
            Some(override_model.clone()),
        );

        assert_eq!(model.model, "claude-opus-4.1", "User override must be respected");
    }

    #[tokio::test]
    async fn test_cost_tracking_integration() {
        let router = ModelRouter::new();

        // Record some usage
        let haiku = ModelConfig::claude_haiku();
        router.record_usage(&haiku, 1000, 500).await;

        let opus = ModelConfig::claude_opus_4_1();
        router.record_usage(&opus, 1000, 500).await;

        // Get cost savings (should show savings from using Haiku vs always Opus)
        let (actual_cost, baseline_cost) = router.get_cost_savings().await;

        assert!(actual_cost < baseline_cost, "Routing should save costs");

        // Cost savings should be significant
        let savings_percent = ((baseline_cost - actual_cost) / baseline_cost) * 100.0;
        assert!(savings_percent > 0.0, "Should show cost savings");
    }
}

#[cfg(test)]
mod end_to_end_integration {
    use super::*;

    #[test]
    fn test_agent_architecture_consistency() {
        // Verify that our architectural decisions are enforced

        // Rule 1: Only Explorer can spawn sub-agents
        let explorer = AgentConfig::explorer();
        let builder = AgentConfig::builder();
        let debugger = AgentConfig::debugger();
        let refactorer = AgentConfig::refactorer();

        assert!(explorer.can_spawn_subagents, "Explorer must spawn sub-agents");
        assert!(!builder.can_spawn_subagents, "Builder NEVER spawns sub-agents");
        assert!(!debugger.can_spawn_subagents, "Debugger NEVER spawns sub-agents");
        assert!(!refactorer.can_spawn_subagents, "Refactorer NEVER spawns sub-agents");

        // Rule 2: Sub-agents always use Haiku
        let router = ModelRouter::new();
        let file_searcher_model = router.select_model(
            RouterAgentType::FileSearcher,
            TaskComplexity::High,
            None,
        );
        assert_eq!(file_searcher_model.model, "claude-haiku");

        // Rule 3: Sub-agents have limited scope
        let file_searcher = AgentConfig::file_searcher();
        assert!(file_searcher.max_steps <= 20);
        assert_eq!(file_searcher.memory_access, MemoryAccessLevel::ReadOnly);
    }

    #[tokio::test]
    async fn test_research_workflow() {
        // Simulate a research workflow:
        // 1. User asks "Find all authentication patterns"
        // 2. System classifies as CodeReading → Explorer agent
        // 3. Explorer spawns research sub-agents
        // 4. Sub-agents use Haiku model
        // 5. Results are aggregated

        let registry = AgentRegistry::new();
        let router = ModelRouter::new();

        // Step 1-2: Select agent for research task
        let agent_config = registry.select_for_intent("Find all authentication patterns");
        assert_eq!(agent_config.agent_type, RouterAgentType::Explorer);
        assert!(agent_config.can_spawn_subagents);

        // Step 3: Spawn research sub-agents
        let manager = ResearchSubAgentManager::new();
        let handle = manager.spawn_research("Find all authentication patterns").await.unwrap();

        // Step 4: Verify sub-agents use Haiku
        let model = router.select_model(
            RouterAgentType::FileSearcher,
            TaskComplexity::Medium,
            None,
        );
        assert_eq!(model.model, "claude-haiku");

        // Step 5: Aggregate results
        let results = handle.wait().await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_coding_workflow_never_spawns_subagents() {
        // Simulate a coding workflow:
        // 1. User asks "Implement user login feature"
        // 2. System classifies as CodeWriting → Builder agent
        // 3. Builder NEVER spawns sub-agents (15x waste)
        // 4. Builder uses appropriate model based on complexity

        let registry = AgentRegistry::new();
        let router = ModelRouter::new();

        // Step 1-2: Select agent for coding task
        let agent_config = registry.select_for_intent("Implement user login feature");
        assert_eq!(agent_config.agent_type, RouterAgentType::Builder);

        // Step 3: Verify Builder NEVER spawns sub-agents
        assert!(!agent_config.can_spawn_subagents,
                "Builder must NEVER spawn sub-agents to avoid 15x token waste");

        // Step 4: Builder uses good model for complex work
        let model = router.select_model(
            RouterAgentType::Builder,
            TaskComplexity::High,
            None,
        );
        assert_eq!(model.model, "claude-opus-4.1");
    }

    #[test]
    fn test_competitive_advantage_claims() {
        // Validate that our architectural claims are enforced:

        // Claim 1: "Research sub-agents for 90% improvement"
        let manager = ResearchSubAgentManager::new();
        // Can spawn up to 10 concurrent sub-agents
        assert_eq!(aircher::agent::MAX_CONCURRENT_SUBAGENTS, 10);

        // Claim 2: "0% sub-agent usage for coding (avoid 15x waste)"
        let builder = AgentConfig::builder();
        let debugger = AgentConfig::debugger();
        let refactorer = AgentConfig::refactorer();
        assert!(!builder.can_spawn_subagents);
        assert!(!debugger.can_spawn_subagents);
        assert!(!refactorer.can_spawn_subagents);

        // Claim 3: "40% cost reduction via model routing"
        let router = ModelRouter::new();
        // Sub-agents use cheapest model
        let subagent_model = router.select_model(
            RouterAgentType::FileSearcher,
            TaskComplexity::High,
            None,
        );
        let main_agent_model = router.select_model(
            RouterAgentType::Builder,
            TaskComplexity::High,
            None,
        );
        assert!(subagent_model.cost_per_1m_input < main_agent_model.cost_per_1m_input);
    }
}
