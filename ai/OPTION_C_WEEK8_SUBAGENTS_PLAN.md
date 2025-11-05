# Option C: Week 8 Sub-Agents Implementation

**Goal**: Implement research sub-agents for parallel information gathering (90% speedup target)

**Timeline**: 2-3 days (Week 8 Days 3-5)

**Expected Impact**: 90% improvement for research tasks, 0% usage for coding (avoid 15x waste)

## Problem Statement

**Current approach** (sequential research):
```
User: "Research authentication patterns in codebase"
  → Agent: Grep for "auth" (2s)
  → Agent: Read auth.rs (3s)
  → Agent: Grep for "login" (2s)
  → Agent: Read login.rs (3s)
  → Total: ~10s sequential
```

**Sub-agent approach** (parallel research):
```
User: "Research authentication patterns in codebase"
  → Main Agent: Spawns 5 sub-agents in parallel
    → Sub-agent 1: Search for "auth" patterns (2s)
    → Sub-agent 2: Search for "login" patterns (2s)
    → Sub-agent 3: Search for "session" patterns (2s)
    → Sub-agent 4: Search for "token" patterns (2s)
    → Sub-agent 5: Search for "oauth" patterns (2s)
  → Main Agent: Aggregates results (1s)
  → Total: ~3s parallel (90% speedup! 10s → 3s)
```

## Architecture Design (From Crush Analysis)

### 1. Sub-Agent Session Hierarchy

```rust
// Based on Crush pattern: ai/research/crush-subagent-architecture.md

#[derive(Debug, Clone)]
pub struct SubAgentSession {
    pub id: Uuid,
    pub parent_session_id: Uuid,  // ← Link to parent
    pub agent_type: AgentType,     // FileSearcher, PatternFinder, etc.
    pub task_description: String,  // What is this sub-agent doing?
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cost: f64,                 // Accumulated cost
    pub status: SubAgentStatus,
}

#[derive(Debug, Clone)]
pub enum SubAgentStatus {
    Active,
    Completed,
    Failed { error: String },
    Cancelled,
}

// Extension to existing SessionService
impl SessionService {
    pub fn create_sub_agent_session(
        &mut self,
        parent_id: Uuid,
        agent_type: AgentType,
        task_description: String,
    ) -> Result<SubAgentSession> {
        let session = SubAgentSession {
            id: Uuid::new_v4(),
            parent_session_id: parent_id,
            agent_type,
            task_description,
            created_at: Utc::now(),
            completed_at: None,
            cost: 0.0,
            status: SubAgentStatus::Active,
        };

        // Store in sessions table with parent_id foreign key
        self.db.insert_sub_agent_session(&session)?;

        Ok(session)
    }

    pub fn roll_up_cost(&mut self, sub_agent_id: Uuid) -> Result<()> {
        let sub_session = self.get_sub_agent_session(sub_agent_id)?;
        let parent = self.get_session(sub_session.parent_session_id)?;

        parent.cost += sub_session.cost;
        self.save_session(parent)?;

        Ok(())
    }
}
```

### 2. Sub-Agent Types (Specialized Researchers)

```rust
// From ai/SYSTEM_DESIGN_2025.md - extend existing AgentType

pub enum AgentType {
    // Primary agents (existing)
    Explorer,
    Builder,
    Debugger,
    Refactorer,

    // Sub-agents for parallel research (NEW)
    FileSearcher,      // Search files by pattern
    PatternFinder,     // Find code patterns
    DependencyMapper,  // Trace dependencies
    DocumentationReader, // Fetch and analyze docs
    WebSearcher,       // Web search (Brave/Exa)
}
```

### 3. Sub-Agent Manager

```rust
// New file: src/agent/sub_agent_manager.rs

pub struct SubAgentManager {
    parent_session_id: Uuid,
    sessions: Arc<SessionService>,
    max_concurrent: usize,  // Default: 10 (from Claude Code)
    active_subagents: Arc<RwLock<Vec<Uuid>>>,
}

pub struct SubAgentConfig {
    pub agent_type: AgentType,
    pub task_description: String,
    pub tools: Vec<String>,        // Limited tool set
    pub model: ModelConfig,         // Usually Haiku (cheap)
    pub auto_approve: bool,         // Usually true
    pub working_dir: PathBuf,       // Usually temp dir
}

pub struct SubAgentResult {
    pub session_id: Uuid,
    pub agent_type: AgentType,
    pub result: String,
    pub cost: f64,
    pub duration: Duration,
}

impl SubAgentManager {
    pub async fn spawn_research_subagents(
        &self,
        query: &str,
        max_subagents: usize,
    ) -> Result<Vec<SubAgentResult>> {
        // 1. Check memory first (prevent duplicate research)
        if let Some(cached) = self.check_episodic_memory(query).await? {
            info!("Cache hit: Found similar research from {} ago",
                  format_duration(cached.timestamp.elapsed()));
            return Ok(vec![cached.into()]);
        }

        // 2. Decompose query into parallel subtasks
        let subtasks = self.decompose_research_query(query).await?;
        info!("Decomposed into {} subtasks", subtasks.len());

        // 3. Limit concurrent sub-agents
        let subtasks: Vec<_> = subtasks.into_iter()
            .take(max_subagents.min(self.max_concurrent))
            .collect();

        // 4. Spawn sub-agents in parallel
        let handles: Vec<_> = subtasks.into_iter()
            .map(|task| {
                let config = SubAgentConfig {
                    agent_type: task.agent_type,
                    task_description: task.description.clone(),
                    tools: vec!["grep", "read", "glob", "find_definition"],
                    model: ModelConfig::claude_haiku(),  // Cheap!
                    auto_approve: true,
                    working_dir: self.create_temp_workspace()?,
                };

                let manager = self.clone();
                tokio::spawn(async move {
                    manager.run_subagent(task, config).await
                })
            })
            .collect();

        // 5. Wait for all sub-agents (join_all)
        let results = futures::future::join_all(handles).await;

        // 6. Aggregate results
        let mut aggregated = Vec::new();
        let mut total_cost = 0.0;

        for result in results {
            match result {
                Ok(Ok(sub_result)) => {
                    total_cost += sub_result.cost;
                    aggregated.push(sub_result);
                }
                Ok(Err(e)) => {
                    warn!("Sub-agent failed: {}", e);
                }
                Err(e) => {
                    warn!("Sub-agent panicked: {}", e);
                }
            }
        }

        // 7. Roll up costs to parent
        self.sessions.add_cost(self.parent_session_id, total_cost)?;

        // 8. Save to episodic memory (prevent duplicate research)
        self.save_to_episodic_memory(query, &aggregated).await?;

        Ok(aggregated)
    }

    async fn run_subagent(
        &self,
        task: ResearchSubtask,
        config: SubAgentConfig,
    ) -> Result<SubAgentResult> {
        let start = Instant::now();

        // Create sub-agent session
        let session = self.sessions.create_sub_agent_session(
            self.parent_session_id,
            config.agent_type,
            config.task_description.clone(),
        )?;

        info!("Sub-agent {} started: {}", session.id, config.task_description);

        // Create sub-agent with limited tools
        let agent = Agent::new(AgentConfig {
            agent_type: config.agent_type,
            session_id: session.id,
            tools: config.tools,
            model: config.model,
            system_prompt: self.build_subagent_prompt(&task),
            working_dir: config.working_dir,
            mode: AgentMode::Plan,  // Read-only!
        })?;

        // Auto-approve if configured (Crush pattern)
        if config.auto_approve {
            self.sessions.auto_approve(session.id)?;
        }

        // Execute research task
        let result = agent.execute(&task.prompt).await?;

        // Update session status
        let duration = start.elapsed();
        self.sessions.complete_sub_agent_session(
            session.id,
            &result,
            duration,
        )?;

        // Roll up cost to parent
        self.sessions.roll_up_cost(session.id)?;

        info!("Sub-agent {} completed in {:?}", session.id, duration);

        Ok(SubAgentResult {
            session_id: session.id,
            agent_type: config.agent_type,
            result,
            cost: session.cost,
            duration,
        })
    }

    async fn decompose_research_query(&self, query: &str) -> Result<Vec<ResearchSubtask>> {
        // Use main agent to break query into subtasks
        let prompt = format!(
            r#"Decompose this research query into 3-5 parallel subtasks:

Query: {query}

For each subtask:
- agent_type: FileSearcher | PatternFinder | DocumentationReader | WebSearcher
- description: What this sub-task will research
- prompt: Specific instructions for the sub-agent

Return JSON array of subtasks.
"#,
            query = query
        );

        let response = self.main_agent.execute(&prompt).await?;
        let subtasks: Vec<ResearchSubtask> = serde_json::from_str(&response)?;

        Ok(subtasks)
    }

    fn create_temp_workspace(&self) -> Result<PathBuf> {
        // Crush pattern: isolated temp directory per sub-agent
        let temp_dir = tempfile::tempdir_in(&self.config.data_dir)?;
        Ok(temp_dir.into_path())
    }
}
```

### 4. Web Search Integration

```rust
// New file: src/agent/tools/brave_search.rs

use reqwest::Client;

pub struct BraveSearchTool {
    api_key: String,
    client: Client,
}

impl BraveSearchTool {
    pub async fn search(&self, query: &str, count: usize) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://api.search.brave.com/res/v1/web/search?q={}&count={}",
            urlencoding::encode(query),
            count
        );

        let response = self.client
            .get(&url)
            .header("X-Subscription-Token", &self.api_key)
            .send()
            .await?;

        let results: BraveSearchResponse = response.json().await?;

        Ok(results.web.results)
    }
}

// New file: src/agent/tools/exa_search.rs

pub struct ExaSearchTool {
    api_key: String,
    client: Client,
}

impl ExaSearchTool {
    pub async fn search_code(&self, query: &str) -> Result<Vec<CodeSearchResult>> {
        let url = "https://api.exa.ai/search";

        let body = json!({
            "query": query,
            "type": "neural",
            "useAutoprompt": true,
            "numResults": 10,
            "contents": {
                "text": true
            }
        });

        let response = self.client
            .post(url)
            .header("x-api-key", &self.api_key)
            .json(&body)
            .send()
            .await?;

        let results: ExaSearchResponse = response.json().await?;

        Ok(results.results)
    }
}
```

## Implementation Steps

### Week 8 Day 3: Core Infrastructure (8 hours)

**Morning (4 hours): Session Hierarchy**
- [ ] Extend SessionService with sub-agent methods
- [ ] Create SubAgentSession struct + DB schema
- [ ] Implement create_sub_agent_session(), roll_up_cost()
- [ ] Add tests for session hierarchy

**Afternoon (4 hours): Sub-Agent Manager**
- [ ] Create `src/agent/sub_agent_manager.rs`
- [ ] Implement SubAgentManager struct
- [ ] Implement spawn_research_subagents() skeleton
- [ ] Implement run_subagent() method
- [ ] Add logging for debugging

### Week 8 Day 4: Web Search + Integration (8 hours)

**Morning (4 hours): Search APIs**
- [ ] Create `src/agent/tools/brave_search.rs`
- [ ] Create `src/agent/tools/exa_search.rs`
- [ ] Implement search methods
- [ ] Add API key configuration
- [ ] Test with real API calls

**Afternoon (4 hours): Query Decomposition**
- [ ] Implement decompose_research_query()
- [ ] Create ResearchSubtask struct
- [ ] Add subtask prompts and templates
- [ ] Test decomposition with sample queries
- [ ] Handle edge cases (0 subtasks, >10 subtasks)

### Week 8 Day 5: Testing & Validation (8 hours)

**Morning (4 hours): Integration Tests**
- [ ] Test: Spawn 5 sub-agents for parallel search
- [ ] Test: Cost rolls up to parent session
- [ ] Test: Max concurrent limit enforced (10)
- [ ] Test: Memory prevents duplicate research
- [ ] Test: Auto-approval works for sub-agents

**Afternoon (4 hours): Performance Validation**
- [ ] Benchmark: Sequential vs parallel research (target 90% speedup)
- [ ] Measure: Token usage (sub-agents use Haiku = 40% cost reduction)
- [ ] Measure: Time to completion (expect 3-5x speedup)
- [ ] Document results in evaluation file
- [ ] Create performance graphs (sequential vs parallel)

## Prompt Engineering for Sub-Agents

### Sub-Agent System Prompt

```rust
fn build_subagent_prompt(task: &ResearchSubtask) -> String {
    format!(
        r#"You are a research sub-agent focused on a specific subtask.

Your Task: {task_description}

IMPORTANT:
- You are ONE of MULTIPLE parallel sub-agents
- Focus ONLY on your specific subtask (don't try to solve everything)
- Be concise (main agent will aggregate all results)
- Use provided tools: {tools}
- You are in read-only mode (grep, read, glob ONLY)

Return your findings in this format:
- summary: One sentence summary
- details: Key findings (bullet points)
- files_examined: List of files you looked at
- confidence: How confident are you? (0.0 to 1.0)

Output JSON.
"#,
        task_description = task.description,
        tools = task.tools.join(", ")
    )
}
```

### Decomposition Prompt

```
Given this research query: "Find all authentication patterns in the codebase"

Decompose into parallel subtasks:
1. FileSearcher: Search for files containing "auth", "login", "session"
2. PatternFinder: Find authentication function patterns
3. DependencyMapper: Trace auth dependencies
4. DocumentationReader: Find auth-related documentation
5. WebSearcher: Search for similar auth patterns online (if relevant)

Each subtask runs independently and concurrently.
```

## Memory Integration (Prevent Duplicate Research)

```rust
impl SubAgentManager {
    async fn check_episodic_memory(&self, query: &str) -> Result<Option<CachedResearch>> {
        // Check if we've done similar research recently
        let similar = self.episodic_memory
            .find_similar_research(query, 0.85)  // 85% similarity
            .await?;

        if let Some(cached) = similar {
            // Check if still recent (within 1 hour)
            if cached.timestamp.elapsed() < Duration::from_secs(3600) {
                info!("Using cached research from {} ago",
                      format_duration(cached.timestamp.elapsed()));
                return Ok(Some(cached));
            }
        }

        Ok(None)
    }

    async fn save_to_episodic_memory(
        &self,
        query: &str,
        results: &[SubAgentResult],
    ) -> Result<()> {
        let aggregated = self.aggregate_results(results)?;

        self.episodic_memory.record_research_session(
            query,
            &aggregated,
            Utc::now(),
        ).await?;

        info!("Saved research to episodic memory: {}", query);

        Ok(())
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_hierarchy() {
        let mut sessions = SessionService::new();
        let parent_id = sessions.create_session("parent").unwrap();

        let sub_id = sessions.create_sub_agent_session(
            parent_id,
            AgentType::FileSearcher,
            "Find auth files".to_string(),
        ).unwrap();

        // Verify parent_id is correct
        let sub_session = sessions.get_sub_agent_session(sub_id).unwrap();
        assert_eq!(sub_session.parent_session_id, parent_id);
    }

    #[test]
    fn test_cost_rollup() {
        let mut sessions = SessionService::new();
        let parent_id = sessions.create_session("parent").unwrap();
        let sub_id = sessions.create_sub_agent_session(
            parent_id,
            AgentType::FileSearcher,
            "test".to_string(),
        ).unwrap();

        // Sub-agent accumulates $0.50 cost
        sessions.add_sub_agent_cost(sub_id, 0.50).unwrap();

        // Roll up to parent
        sessions.roll_up_cost(sub_id).unwrap();

        // Verify parent cost increased
        let parent = sessions.get_session(parent_id).unwrap();
        assert_eq!(parent.cost, 0.50);
    }

    #[test]
    fn test_max_concurrent_limit() {
        let manager = SubAgentManager::new(10);  // Max 10

        let subtasks = vec![/* 15 subtasks */];
        let limited = manager.limit_concurrent(subtasks);

        assert_eq!(limited.len(), 10);  // Only 10 spawned
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_parallel_subagents() {
    let manager = SubAgentManager::new(5);

    let query = "Find authentication patterns";

    let start = Instant::now();
    let results = manager.spawn_research_subagents(query, 5).await.unwrap();
    let duration = start.elapsed();

    // Assert: Got results from 5 sub-agents
    assert_eq!(results.len(), 5);

    // Assert: Completed in <5s (parallel execution)
    assert!(duration < Duration::from_secs(5));

    // Assert: Costs rolled up
    assert!(results.iter().map(|r| r.cost).sum::<f64>() > 0.0);
}
```

### Performance Benchmarks

```rust
#[tokio::test]
async fn benchmark_sequential_vs_parallel() {
    let query = "Find all error handling patterns";

    // Sequential (baseline)
    let start = Instant::now();
    let sequential_result = agent.execute_sequential_research(query).await.unwrap();
    let sequential_time = start.elapsed();

    // Parallel (sub-agents)
    let start = Instant::now();
    let parallel_result = manager.spawn_research_subagents(query, 5).await.unwrap();
    let parallel_time = start.elapsed();

    // Calculate speedup
    let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();

    println!("Sequential: {:?}", sequential_time);
    println!("Parallel: {:?}", parallel_time);
    println!("Speedup: {:.1}x", speedup);

    // Assert: At least 2x speedup (target: 3-5x)
    assert!(speedup > 2.0);
}
```

## Expected Outcomes

### Success Metrics

**Minimum (acceptable)**:
- [ ] 5 sub-agents spawn successfully
- [ ] Parallel execution works (not sequential)
- [ ] Costs roll up to parent session
- [ ] At least 2x speedup vs sequential

**Target (good)**:
- [ ] 3-5x speedup for research tasks
- [ ] 40% cost reduction (Haiku vs Sonnet)
- [ ] Memory prevents duplicate research (80%+ cache hit rate)
- [ ] Max 10 concurrent enforced

**Stretch (excellent)**:
- [ ] 5-10x speedup for highly parallel tasks
- [ ] Episodic memory cache hit rate >90%
- [ ] Web search integration working (Brave/Exa)
- [ ] Zero sub-agent usage for coding tasks (correct decision matrix)

## Integration with Hybrid Architecture

### Plan Mode: Can Spawn Sub-Agents ✅

```rust
impl Agent {
    pub async fn execute_in_plan_mode(&self, request: &str) -> Result<Response> {
        // Detect if request is research-heavy
        let intent = self.classify_intent(request).await?;

        match intent {
            UserIntent::CodeReading { complexity: Complexity::High } => {
                // Spawn sub-agents for parallel research
                let manager = SubAgentManager::new(self.session_id, 10);
                let results = manager.spawn_research_subagents(request, 5).await?;
                self.aggregate_and_respond(results).await
            }
            _ => {
                // Regular execution (no sub-agents)
                self.execute_single_agent(request).await
            }
        }
    }
}
```

### Build Mode: NEVER Spawn Sub-Agents ❌

```rust
impl Agent {
    pub async fn execute_in_build_mode(&self, request: &str) -> Result<Response> {
        // NEVER spawn sub-agents for coding
        // Reason: 15x token waste, context isolation fatal
        self.execute_single_agent(request).await
    }
}
```

## Files to Create/Modify

**New Files** (~800 lines total):
- `src/agent/sub_agent_manager.rs` (~400 lines)
- `src/agent/tools/brave_search.rs` (~150 lines)
- `src/agent/tools/exa_search.rs` (~150 lines)
- `tests/sub_agent_tests.rs` (~300 lines)

**Modified Files** (~200 lines added):
- `src/agent/session.rs` (add sub-agent session methods)
- `src/agent/mod.rs` (export sub_agent_manager)
- `src/config/mod.rs` (add Brave/Exa API keys)
- `Cargo.toml` (add dependencies: futures, tokio)

## Dependencies to Add

```toml
[dependencies]
futures = "0.3"           # For join_all
tokio = { version = "1", features = ["full"] }  # Already have
tempfile = "3"            # For temp workspaces
reqwest = { version = "0.11", features = ["json"] }  # Already have
urlencoding = "2"         # For URL encoding
```

## Risks & Mitigations

### Risk 1: Sub-agents don't speed up (network bound)
**Impact**: Parallel HTTP requests don't help
**Mitigation**: Benchmark early, pivot if no speedup

### Risk 2: Cost explodes (10 × Haiku still expensive)
**Impact**: 10 sub-agents × $0.25/1M = more than 1 × Sonnet
**Mitigation**: Limit concurrent, monitor costs, cache aggressively

### Risk 3: Results quality decreases (subtasks too narrow)
**Impact**: Sub-agents miss big picture
**Mitigation**: Main agent aggregates and synthesizes, can request clarification

## Success Criteria

**Must have**:
- [ ] Sub-agents execute in parallel (not sequential)
- [ ] Session hierarchy tracks parent-child relationships
- [ ] Costs roll up correctly
- [ ] At least 2x speedup demonstrated

**Should have**:
- [ ] 3-5x speedup for research tasks
- [ ] Memory integration prevents duplicates
- [ ] Web search APIs working (Brave or Exa)
- [ ] Max 10 concurrent enforced

**Nice to have**:
- [ ] 5-10x speedup for highly parallel tasks
- [ ] >90% cache hit rate from episodic memory
- [ ] Beautiful logs showing parallel execution
- [ ] Graph visualization of sub-agent hierarchy

## Next Steps After Completion

**If successful**:
- Use sub-agents in Week 9 validation
- Measure impact on SWE-bench (if research tasks exist)
- Document pattern in research paper

**If unsuccessful**:
- Analyze why parallel didn't help
- Consider sequential with better caching instead
- Keep architecture, tune parameters

**Either way**:
- This validates hybrid architecture's research component
- Completes Week 8 implementation
- Ready for Week 9 empirical validation
