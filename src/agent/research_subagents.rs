// Research Sub-Agents System (Week 8 Day 3-4)
//
// Implements Claude Code's research sub-agent pattern BUT only for research tasks.
// NEVER spawn sub-agents for coding (15x token waste).
//
// References:
// - docs/architecture/SYSTEM_DESIGN_2025.md
// - ai/research/competitive-analysis-2025.md (Claude Code section)
// - ai/DECISIONS.md (2025-10-27 - Hybrid Sub-Agent Strategy)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use super::model_router::{AgentType as RouterAgentType, TaskComplexity};
use super::specialized_agents::AgentConfig;

/// Maximum concurrent sub-agents (from Claude Code research)
pub const MAX_CONCURRENT_SUBAGENTS: usize = 10;

/// Research sub-agent task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchTask {
    /// Unique task ID
    pub id: String,

    /// Task description (what to research)
    pub description: String,

    /// Sub-agent type to use
    pub agent_type: RouterAgentType,

    /// Task complexity (determines model selection)
    pub complexity: TaskComplexity,

    /// Maximum steps for this task
    pub max_steps: usize,

    /// Context for the task (relevant file paths, etc.)
    pub context: Vec<String>,
}

impl ResearchTask {
    /// Create a new research task
    pub fn new(
        description: String,
        agent_type: RouterAgentType,
        complexity: TaskComplexity,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            description,
            agent_type,
            complexity,
            max_steps: 20, // Sub-agents have limited scope
            context: Vec::new(),
        }
    }

    /// Add context to the task
    pub fn with_context(mut self, context: Vec<String>) -> Self {
        self.context = context;
        self
    }
}

/// Research result from a sub-agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    /// Task ID
    pub task_id: String,

    /// Success or failure
    pub success: bool,

    /// Findings from the research
    pub findings: String,

    /// Relevant file paths discovered
    pub relevant_files: Vec<String>,

    /// Token usage
    pub tokens_used: usize,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Error message if failed
    pub error: Option<String>,
}

/// Research query decomposition
pub struct QueryDecomposer;

impl QueryDecomposer {
    /// Decompose a research query into parallel sub-tasks
    pub fn decompose(query: &str) -> Vec<ResearchTask> {
        let query_lower = query.to_lowercase();

        // Simple heuristic-based decomposition
        // In production, this would use LLM-based decomposition

        if query_lower.contains("find all") || query_lower.contains("search for") {
            // File search task
            Self::decompose_search_query(query)
        } else if query_lower.contains("how many") || query_lower.contains("list all") {
            // Counting/listing task
            Self::decompose_listing_query(query)
        } else if query_lower.contains("what uses") || query_lower.contains("where is") {
            // Dependency tracking task
            Self::decompose_dependency_query(query)
        } else if query_lower.contains("pattern") || query_lower.contains("similar to") {
            // Pattern finding task
            Self::decompose_pattern_query(query)
        } else {
            // Default: single exploration task
            vec![ResearchTask::new(
                query.to_string(),
                RouterAgentType::FileSearcher,
                TaskComplexity::Medium,
            )]
        }
    }

    fn decompose_search_query(query: &str) -> Vec<ResearchTask> {
        // Example: "Find all authentication code"
        // Could decompose into:
        // 1. Search src/auth/ for authentication
        // 2. Search src/api/ for auth middleware
        // 3. Search tests/ for auth tests

        // Simple implementation: create 3 parallel search tasks
        vec![
            ResearchTask::new(
                format!("{} in source files", query),
                RouterAgentType::FileSearcher,
                TaskComplexity::Low,
            )
            .with_context(vec!["src/".to_string()]),
            ResearchTask::new(
                format!("{} in tests", query),
                RouterAgentType::FileSearcher,
                TaskComplexity::Low,
            )
            .with_context(vec!["tests/".to_string()]),
            ResearchTask::new(
                format!("{} in documentation", query),
                RouterAgentType::FileSearcher,
                TaskComplexity::Low,
            )
            .with_context(vec!["docs/".to_string(), "README.md".to_string()]),
        ]
    }

    fn decompose_listing_query(_query: &str) -> Vec<ResearchTask> {
        // Example: "List all API endpoints"
        // Single task with FileSearcher
        vec![ResearchTask::new(
            "List all instances".to_string(),
            RouterAgentType::FileSearcher,
            TaskComplexity::Medium,
        )]
    }

    fn decompose_dependency_query(query: &str) -> Vec<ResearchTask> {
        // Example: "What uses the User model?"
        // Use DependencyMapper
        vec![ResearchTask::new(
            query.to_string(),
            RouterAgentType::DependencyMapper,
            TaskComplexity::Medium,
        )]
    }

    fn decompose_pattern_query(query: &str) -> Vec<ResearchTask> {
        // Example: "Find all error handling patterns"
        // Use PatternFinder
        vec![ResearchTask::new(
            query.to_string(),
            RouterAgentType::PatternFinder,
            TaskComplexity::Medium,
        )]
    }
}

/// Research sub-agent manager
pub struct ResearchSubAgentManager {
    /// Active sub-agent handles
    active: Arc<RwLock<Vec<JoinHandle<Result<ResearchResult>>>>>,

    /// Completed results
    results: Arc<RwLock<Vec<ResearchResult>>>,

    /// Memory integration (to prevent duplicate research)
    #[allow(dead_code)]
    episodic_memory: Option<Arc<RwLock<EpisodicMemoryStub>>>,
}

/// Stub for episodic memory integration
/// In production, this would be the real EpisodicMemory implementation
struct EpisodicMemoryStub;

impl EpisodicMemoryStub {
    /// Check if similar research was done recently
    #[allow(dead_code)]
    async fn find_similar_research(
        &self,
        _query: &str,
        _similarity_threshold: f32,
    ) -> Result<Option<CachedResearch>> {
        // Stub implementation
        Ok(None)
    }

    /// Record research session
    #[allow(dead_code)]
    async fn record_research(&self, _query: &str, _results: &[ResearchResult]) -> Result<()> {
        // Stub implementation
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct CachedResearch {
    #[allow(dead_code)]
    query: String,
    #[allow(dead_code)]
    results: Vec<ResearchResult>,
    #[allow(dead_code)]
    timestamp: std::time::SystemTime,
}

impl ResearchSubAgentManager {
    /// Create a new research sub-agent manager
    pub fn new() -> Self {
        Self {
            active: Arc::new(RwLock::new(Vec::new())),
            results: Arc::new(RwLock::new(Vec::new())),
            episodic_memory: None,
        }
    }

    /// Spawn research sub-agents for a query
    pub async fn spawn_research(&self, query: &str) -> Result<ResearchHandle> {
        info!("Spawning research sub-agents for query: {}", query);

        // Check for cached research (if memory integration enabled)
        if let Some(_memory) = &self.episodic_memory {
            // Check if we've done similar research recently
            // If yes, return cached results instead of spawning
            debug!("Checking episodic memory for cached research");
            // let cached = memory.read().await.find_similar_research(query, 0.85).await?;
            // if let Some(cached) = cached {
            //     info!("Found cached research, skipping sub-agent spawn");
            //     return Ok(ResearchHandle::cached(cached.results));
            // }
        }

        // Decompose query into sub-tasks
        let tasks = QueryDecomposer::decompose(query);
        info!("Decomposed into {} sub-tasks", tasks.len());

        // Limit to MAX_CONCURRENT_SUBAGENTS
        let tasks = if tasks.len() > MAX_CONCURRENT_SUBAGENTS {
            warn!(
                "Query decomposed into {} tasks, limiting to {}",
                tasks.len(),
                MAX_CONCURRENT_SUBAGENTS
            );
            tasks.into_iter().take(MAX_CONCURRENT_SUBAGENTS).collect()
        } else {
            tasks
        };

        // Spawn sub-agents
        let mut handles = Vec::new();
        for task in tasks {
            let handle = self.spawn_task(task).await?;
            handles.push(handle);
        }

        // Store handles
        let task_count = handles.len();
        let mut active = self.active.write().await;
        active.extend(handles);

        Ok(ResearchHandle {
            tasks: task_count,
            manager: self.clone(),
        })
    }

    /// Spawn a single research task
    async fn spawn_task(
        &self,
        task: ResearchTask,
    ) -> Result<JoinHandle<Result<ResearchResult>>> {
        debug!("Spawning sub-agent for task: {}", task.id);

        // Get agent configuration
        let config = AgentConfig::explorer(); // Placeholder
        let _config = match task.agent_type {
            RouterAgentType::FileSearcher => AgentConfig::file_searcher(),
            RouterAgentType::PatternFinder => AgentConfig::pattern_finder(),
            RouterAgentType::DependencyMapper => AgentConfig::dependency_mapper(),
            _ => config,
        };

        // Spawn task in background
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();

            // Simulate research execution
            // In production, this would execute the actual agent logic
            debug!("Executing research task: {}", task.description);
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            // Placeholder result
            let result = ResearchResult {
                task_id: task.id.clone(),
                success: true,
                findings: format!("Research findings for: {}", task.description),
                relevant_files: vec![],
                tokens_used: 1000,
                duration_ms: start.elapsed().as_millis() as u64,
                error: None,
            };

            Ok(result)
        });

        Ok(handle)
    }

    /// Wait for all sub-agents to complete and aggregate results
    pub async fn aggregate_results(&self) -> Result<Vec<ResearchResult>> {
        let mut active = self.active.write().await;

        info!("Waiting for {} sub-agents to complete", active.len());

        // Await all handles
        let mut results = Vec::new();
        for handle in active.drain(..) {
            match handle.await {
                Ok(Ok(result)) => {
                    debug!("Sub-agent completed: {}", result.task_id);
                    results.push(result);
                }
                Ok(Err(e)) => {
                    warn!("Sub-agent failed: {}", e);
                }
                Err(e) => {
                    warn!("Sub-agent panicked: {}", e);
                }
            }
        }

        // Store results
        let mut stored_results = self.results.write().await;
        stored_results.extend(results.clone());

        info!("Research complete: {} results aggregated", results.len());

        Ok(results)
    }

    /// Get current research progress
    pub async fn get_progress(&self) -> ResearchProgress {
        let active = self.active.read().await;
        let completed = self.results.read().await;

        ResearchProgress {
            total_tasks: active.len() + completed.len(),
            completed_tasks: completed.len(),
            active_tasks: active.len(),
        }
    }

    /// Cancel all active research
    pub async fn cancel_all(&self) {
        let mut active = self.active.write().await;
        for handle in active.drain(..) {
            handle.abort();
        }
        info!("Cancelled all active research sub-agents");
    }
}

impl Clone for ResearchSubAgentManager {
    fn clone(&self) -> Self {
        Self {
            active: Arc::clone(&self.active),
            results: Arc::clone(&self.results),
            episodic_memory: self.episodic_memory.clone(),
        }
    }
}

impl Default for ResearchSubAgentManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle for managing research sub-agents
pub struct ResearchHandle {
    tasks: usize,
    manager: ResearchSubAgentManager,
}

impl ResearchHandle {
    /// Wait for all sub-agents to complete
    pub async fn wait(self) -> Result<Vec<ResearchResult>> {
        self.manager.aggregate_results().await
    }

    /// Get current progress
    pub async fn progress(&self) -> ResearchProgress {
        self.manager.get_progress().await
    }

    /// Cancel all sub-agents
    pub async fn cancel(&self) {
        self.manager.cancel_all().await;
    }

    /// Number of tasks spawned
    pub fn task_count(&self) -> usize {
        self.tasks
    }
}

/// Research progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchProgress {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub active_tasks: usize,
}

impl ResearchProgress {
    /// Get completion percentage
    pub fn percent_complete(&self) -> f32 {
        if self.total_tasks == 0 {
            return 100.0;
        }
        (self.completed_tasks as f32 / self.total_tasks as f32) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_task_creation() {
        let task = ResearchTask::new(
            "Find authentication code".to_string(),
            RouterAgentType::FileSearcher,
            TaskComplexity::Low,
        );

        assert!(!task.id.is_empty());
        assert_eq!(task.description, "Find authentication code");
        assert_eq!(task.agent_type, RouterAgentType::FileSearcher);
        assert_eq!(task.max_steps, 20);
    }

    #[test]
    fn test_query_decomposition_search() {
        let tasks = QueryDecomposer::decompose("Find all authentication code");
        assert!(tasks.len() >= 1);
        // Should create multiple parallel search tasks
        assert!(tasks.len() <= MAX_CONCURRENT_SUBAGENTS);
    }

    #[test]
    fn test_query_decomposition_dependency() {
        let tasks = QueryDecomposer::decompose("What uses the User model?");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].agent_type, RouterAgentType::DependencyMapper);
    }

    #[test]
    fn test_query_decomposition_pattern() {
        let tasks = QueryDecomposer::decompose("Find all error handling patterns");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].agent_type, RouterAgentType::PatternFinder);
    }

    #[tokio::test]
    async fn test_subagent_spawning() {
        let manager = ResearchSubAgentManager::new();
        let handle = manager.spawn_research("Find authentication code").await.unwrap();

        assert!(handle.task_count() > 0);
        assert!(handle.task_count() <= MAX_CONCURRENT_SUBAGENTS);
    }

    #[tokio::test]
    async fn test_result_aggregation() {
        let manager = ResearchSubAgentManager::new();
        let handle = manager.spawn_research("Test query").await.unwrap();

        let results = handle.wait().await.unwrap();
        assert!(!results.is_empty());

        // All results should be successful (in this test)
        for result in &results {
            assert!(result.success);
        }
    }

    #[tokio::test]
    async fn test_research_progress() {
        let manager = ResearchSubAgentManager::new();
        let handle = manager.spawn_research("Test query").await.unwrap();

        let progress = handle.progress().await;
        assert!(progress.total_tasks > 0);
        assert!(progress.percent_complete() >= 0.0);
        assert!(progress.percent_complete() <= 100.0);
    }

    #[tokio::test]
    async fn test_concurrent_limit() {
        let manager = ResearchSubAgentManager::new();

        // Create a query that would decompose into many tasks
        let query = "Find all authentication authorization security crypto hashing encryption decryption validation verification";
        let handle = manager.spawn_research(query).await.unwrap();

        // Should be limited to MAX_CONCURRENT_SUBAGENTS
        assert!(handle.task_count() <= MAX_CONCURRENT_SUBAGENTS);
    }

    #[tokio::test]
    async fn test_cancel_research() {
        let manager = ResearchSubAgentManager::new();
        let handle = manager.spawn_research("Long running query").await.unwrap();

        // Cancel immediately
        handle.cancel().await;

        let progress = handle.progress().await;
        assert_eq!(progress.active_tasks, 0);
    }

    #[test]
    fn test_research_result_serialization() {
        let result = ResearchResult {
            task_id: "test-123".to_string(),
            success: true,
            findings: "Found 5 authentication files".to_string(),
            relevant_files: vec!["auth.rs".to_string(), "login.rs".to_string()],
            tokens_used: 2500,
            duration_ms: 1500,
            error: None,
        };

        // Should serialize/deserialize
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ResearchResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.task_id, result.task_id);
        assert_eq!(deserialized.success, result.success);
        assert_eq!(deserialized.tokens_used, result.tokens_used);
    }
}
