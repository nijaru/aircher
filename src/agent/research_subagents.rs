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
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use super::model_router::{AgentType as RouterAgentType, TaskComplexity};
use super::specialized_agents::AgentConfig;
use super::tools::{ToolRegistry, ToolOutput};

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

    /// Tool registry for executing actual research
    tool_registry: Arc<ToolRegistry>,

    /// Workspace root for tool execution
    workspace_root: PathBuf,
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
        let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            active: Arc::new(RwLock::new(Vec::new())),
            results: Arc::new(RwLock::new(Vec::new())),
            episodic_memory: None,
            tool_registry: Arc::new(ToolRegistry::default()),
            workspace_root,
        }
    }

    /// Create with custom tool registry and workspace
    pub fn with_tools(tool_registry: Arc<ToolRegistry>, workspace_root: PathBuf) -> Self {
        Self {
            active: Arc::new(RwLock::new(Vec::new())),
            results: Arc::new(RwLock::new(Vec::new())),
            episodic_memory: None,
            tool_registry,
            workspace_root,
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

        // Clone necessary data for the spawned task
        let tool_registry = Arc::clone(&self.tool_registry);
        let workspace_root = self.workspace_root.clone();

        // Spawn task in background
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();

            debug!("Executing research task: {}", task.description);

            // Execute actual research based on agent type
            let (findings, relevant_files, tokens_used) = match task.agent_type {
                RouterAgentType::FileSearcher => {
                    Self::execute_file_search(&task, &tool_registry, &workspace_root).await
                }
                RouterAgentType::PatternFinder => {
                    Self::execute_pattern_search(&task, &tool_registry, &workspace_root).await
                }
                RouterAgentType::DependencyMapper => {
                    Self::execute_dependency_mapping(&task, &tool_registry, &workspace_root).await
                }
                _ => {
                    // Fallback to basic file search
                    Self::execute_file_search(&task, &tool_registry, &workspace_root).await
                }
            };

            let (findings, relevant_files, tokens_used) = match (findings, relevant_files, tokens_used) {
                (Ok(f), Ok(r), Ok(t)) => (f, r, t),
                (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                    return Ok(ResearchResult {
                        task_id: task.id.clone(),
                        success: false,
                        findings: String::new(),
                        relevant_files: vec![],
                        tokens_used: 0,
                        duration_ms: start.elapsed().as_millis() as u64,
                        error: Some(format!("Research failed: {}", e)),
                    });
                }
            };

            Ok(ResearchResult {
                task_id: task.id.clone(),
                success: true,
                findings,
                relevant_files,
                tokens_used,
                duration_ms: start.elapsed().as_millis() as u64,
                error: None,
            })
        });

        Ok(handle)
    }

    /// Execute file search research
    async fn execute_file_search(
        task: &ResearchTask,
        tool_registry: &Arc<ToolRegistry>,
        workspace_root: &PathBuf,
    ) -> (Result<String>, Result<Vec<String>>, Result<usize>) {
        let mut findings = Vec::new();
        let mut relevant_files = Vec::new();
        let mut tokens_used = 0;

        // Extract search terms from task description
        let search_term = task.description
            .to_lowercase()
            .replace("find all", "")
            .replace("search for", "")
            .replace("in source files", "")
            .replace("in tests", "")
            .replace("in documentation", "")
            .trim()
            .to_string();

        // 1. List files in context directories
        if let Some(list_tool) = tool_registry.get("list_files") {
            for context_path in &task.context {
                let params = json!({
                    "path": workspace_root.join(context_path).to_string_lossy().to_string(),
                    "recursive": true,
                    "pattern": "*",
                });

                match list_tool.execute(params).await {
                    Ok(output) if output.success => {
                        if let Some(files) = output.result.get("files").and_then(|f| f.as_array()) {
                            findings.push(format!("Found {} files in {}", files.len(), context_path));
                            tokens_used += 100; // Estimate
                        }
                    }
                    Err(e) => {
                        findings.push(format!("Failed to list files in {}: {}", context_path, e));
                    }
                    _ => {}
                }
            }
        }

        // 2. Search code for the term (if search_code tool available)
        if !search_term.is_empty() {
            if let Some(search_tool) = tool_registry.get("search_code") {
                let params = json!({
                    "query": search_term,
                    "scope": if task.context.is_empty() { None } else { Some(task.context.clone()) },
                    "limit": 20,
                });

                match search_tool.execute(params).await {
                    Ok(output) if output.success => {
                        if let Some(results) = output.result.get("results").and_then(|r| r.as_array()) {
                            findings.push(format!("Search found {} matches for '{}'", results.len(), search_term));

                            for result in results.iter().take(10) {
                                if let Some(file) = result.get("file").and_then(|f| f.as_str()) {
                                    relevant_files.push(file.to_string());
                                }
                            }
                            tokens_used += 500; // Estimate
                        }
                    }
                    Err(e) => {
                        findings.push(format!("Search failed: {}", e));
                    }
                    _ => {}
                }
            }
        }

        // 3. Read a sample of relevant files
        if let Some(read_tool) = tool_registry.get("read_file") {
            for file_path in relevant_files.iter().take(3) {
                let params = json!({
                    "path": file_path,
                    "line_range": null,
                });

                match read_tool.execute(params).await {
                    Ok(output) if output.success => {
                        if let Some(content) = output.result.get("content").and_then(|c| c.as_str()) {
                            let preview = content.chars().take(200).collect::<String>();
                            findings.push(format!("Read {}: {}...", file_path, preview));
                            tokens_used += 200; // Estimate
                        }
                    }
                    Err(e) => {
                        findings.push(format!("Failed to read {}: {}", file_path, e));
                    }
                    _ => {}
                }
            }
        }

        let summary = if findings.is_empty() {
            format!("No results found for query: {}", task.description)
        } else {
            findings.join("\n")
        };

        (Ok(summary), Ok(relevant_files), Ok(tokens_used))
    }

    /// Execute pattern search research
    async fn execute_pattern_search(
        task: &ResearchTask,
        tool_registry: &Arc<ToolRegistry>,
        _workspace_root: &PathBuf,
    ) -> (Result<String>, Result<Vec<String>>, Result<usize>) {
        let mut findings = Vec::new();
        let mut relevant_files = Vec::new();
        let tokens_used = 500;

        // Use search_code to find patterns
        if let Some(search_tool) = tool_registry.get("search_code") {
            let params = json!({
                "query": task.description,
                "scope": task.context,
                "limit": 30,
            });

            match search_tool.execute(params).await {
                Ok(output) if output.success => {
                    if let Some(results) = output.result.get("results").and_then(|r| r.as_array()) {
                        findings.push(format!("Found {} pattern matches", results.len()));

                        for result in results {
                            if let Some(file) = result.get("file").and_then(|f| f.as_str()) {
                                relevant_files.push(file.to_string());
                            }
                        }
                    }
                }
                Err(e) => {
                    findings.push(format!("Pattern search failed: {}", e));
                }
                _ => {}
            }
        }

        let summary = if findings.is_empty() {
            format!("No patterns found for: {}", task.description)
        } else {
            findings.join("\n")
        };

        (Ok(summary), Ok(relevant_files), Ok(tokens_used))
    }

    /// Execute dependency mapping research
    async fn execute_dependency_mapping(
        task: &ResearchTask,
        tool_registry: &Arc<ToolRegistry>,
        _workspace_root: &PathBuf,
    ) -> (Result<String>, Result<Vec<String>>, Result<usize>) {
        let mut findings = Vec::new();
        let mut relevant_files = Vec::new();
        let tokens_used = 300;

        // Use find_references tool if available, otherwise search
        if let Some(refs_tool) = tool_registry.get("find_references") {
            // Extract symbol from query
            let symbol = task.description
                .replace("What uses", "")
                .replace("Where is", "")
                .replace("?", "")
                .trim()
                .to_string();

            let params = json!({
                "symbol": symbol,
                "scope": task.context,
            });

            match refs_tool.execute(params).await {
                Ok(output) if output.success => {
                    if let Some(refs) = output.result.get("references").and_then(|r| r.as_array()) {
                        findings.push(format!("Found {} references", refs.len()));

                        for ref_item in refs {
                            if let Some(file) = ref_item.get("file").and_then(|f| f.as_str()) {
                                relevant_files.push(file.to_string());
                            }
                        }
                    }
                }
                Err(e) => {
                    findings.push(format!("Reference search failed: {}", e));
                }
                _ => {}
            }
        }

        let summary = if findings.is_empty() {
            format!("No dependencies found for: {}", task.description)
        } else {
            findings.join("\n")
        };

        (Ok(summary), Ok(relevant_files), Ok(tokens_used))
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
            tool_registry: Arc::clone(&self.tool_registry),
            workspace_root: self.workspace_root.clone(),
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
