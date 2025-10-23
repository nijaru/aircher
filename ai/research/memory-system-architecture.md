# Memory System Architecture

**Purpose**: Enable continuous agent work without context restart via intelligent memory + context management

**Research Contribution**: 60% reduction in tool calls (POC validated)

## The Problem

**Current agents (Claude Code, etc.)**:
- Context fills up (100K-200K tokens)
- Must restart conversation or compact everything
- Lose track of what was done
- Repeat work, check same files multiple times

**Our solution**:
- Dynamic context pruning (remove least relevant)
- Episodic memory (track what we've done)
- Knowledge graph (understand codebase structure)
- Result: Continuous work, 60% fewer tool calls

## Three Memory Systems

### 1. Knowledge Graph (Semantic Memory)

**Purpose**: Understand codebase structure and relationships

**Storage**: In-memory petgraph (Rust)
- Fast graph traversals (microseconds)
- Serialize to binary file on session end
- Load on startup

**Nodes** (3,942 in Aircher POC):
```rust
pub enum NodeType {
    File { path: String, language: Language },
    Function { name: String, signature: String, line: usize },
    Class { name: String, line: usize },
    Import { module: String, items: Vec<String> },
    Variable { name: String, scope: Scope },
}
```

**Edges** (5,217 in Aircher POC):
```rust
pub enum EdgeType {
    Contains,      // file contains function
    Calls,         // function calls function
    Imports,       // file imports module
    Uses,          // function uses variable
    Inherits,      // class inherits class
    References,    // function references class
}
```

**Queries**:
- `graph.get_file_contents(path)` → all functions/classes in file
- `graph.get_callers(function)` → what calls this function
- `graph.get_dependencies(file)` → what files does this depend on
- `graph.find_symbol(name)` → where is this defined

**Build Process**:
1. Scan repository with tree-sitter
2. Extract nodes (files, functions, classes, imports)
3. Extract edges (calls, imports, uses)
4. Build petgraph structure
5. Query during task execution

**Update Strategy**:
- Incremental: File changed → re-parse only that file
- Repository scan: On startup or user request
- Cache: Keep parsed ASTs for fast updates

### 2. Episodic Memory (Experience Tracking)

**Purpose**: Remember what agent has done, learn patterns

**Storage**: DuckDB (embedded analytical database)
- Already have infrastructure (`src/intelligence/memory/duckdb_memory.rs`)
- Better than SQLite for complex queries
- JSON columns for flexibility

**Schema**:

```sql
-- Tool execution history
CREATE TABLE tool_executions (
    id INTEGER PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL,
    session_id VARCHAR NOT NULL,
    task_id VARCHAR,
    tool_name VARCHAR NOT NULL,
    parameters JSON NOT NULL,
    result JSON,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    duration_ms INTEGER,
    context_tokens INTEGER,

    -- Indexes for fast queries
    INDEX idx_session (session_id),
    INDEX idx_tool_timestamp (tool_name, timestamp),
    INDEX idx_task (task_id)
);

-- File interaction tracking
CREATE TABLE file_interactions (
    id INTEGER PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL,
    session_id VARCHAR NOT NULL,
    task_id VARCHAR,
    file_path VARCHAR NOT NULL,
    operation VARCHAR NOT NULL, -- read, write, edit, search, analyze
    line_range JSON, -- {start: 10, end: 50}
    success BOOLEAN NOT NULL,
    context TEXT, -- why this file (from LLM reasoning)
    changes_summary TEXT, -- what changed (for edits)

    INDEX idx_file (file_path),
    INDEX idx_session_file (session_id, file_path),
    INDEX idx_operation (operation, timestamp)
);

-- Task history (user-level tasks)
CREATE TABLE task_history (
    id INTEGER PRIMARY KEY,
    task_id VARCHAR UNIQUE NOT NULL,
    session_id VARCHAR NOT NULL,
    description TEXT NOT NULL,
    intent VARCHAR, -- CodeReading, CodeWriting, ProjectFixing, ProjectExploration
    status VARCHAR NOT NULL, -- active, completed, failed, paused
    started_at TIMESTAMP NOT NULL,
    completed_at TIMESTAMP,
    files_touched JSON, -- [file1, file2, ...]
    tools_used JSON, -- {read_file: 5, edit_file: 2, ...}
    outcome TEXT, -- summary of what happened

    INDEX idx_status (status),
    INDEX idx_session (session_id)
);

-- Context window snapshots (for debugging + recovery)
CREATE TABLE context_snapshots (
    id INTEGER PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL,
    session_id VARCHAR NOT NULL,
    task_id VARCHAR,
    context_items JSON NOT NULL, -- [{id, type, tokens, relevance}, ...]
    total_tokens INTEGER NOT NULL,
    pruned_items JSON, -- what was just removed
    reason VARCHAR NOT NULL, -- "pruning", "task_switch", "manual_snapshot"

    INDEX idx_session (session_id)
);

-- Learned patterns (co-editing, error fixes, refactoring patterns)
CREATE TABLE learned_patterns (
    id INTEGER PRIMARY KEY,
    pattern_type VARCHAR NOT NULL, -- "co-edit", "error-fix", "refactor"
    pattern_data JSON NOT NULL,
    confidence FLOAT NOT NULL, -- 0.0 to 1.0
    observed_count INTEGER NOT NULL,
    first_seen TIMESTAMP NOT NULL,
    last_seen TIMESTAMP NOT NULL,

    INDEX idx_pattern_type (pattern_type)
);

-- Example pattern_data for co-edit:
-- {
--   "files": ["src/auth.rs", "src/api.rs"],
--   "operations": ["edit", "edit"],
--   "within_seconds": 300
-- }

-- Cross-session learnings (persist across restarts)
CREATE TABLE session_learnings (
    id INTEGER PRIMARY KEY,
    session_id VARCHAR NOT NULL,
    learning_type VARCHAR NOT NULL, -- "file_purpose", "common_error", "workflow"
    data JSON NOT NULL,
    confidence FLOAT NOT NULL,

    INDEX idx_type (learning_type)
);
```

**Queries We Need**:

```sql
-- "Have I worked on this file before?"
SELECT * FROM file_interactions
WHERE file_path = ?
ORDER BY timestamp DESC
LIMIT 5;

-- "What files are often edited together?"
SELECT
    f1.file_path as file1,
    f2.file_path as file2,
    COUNT(*) as co_edit_count
FROM file_interactions f1
JOIN file_interactions f2
    ON f1.task_id = f2.task_id
    AND f1.file_path < f2.file_path
    AND f2.timestamp - f1.timestamp < INTERVAL '5 minutes'
WHERE f1.operation = 'edit' AND f2.operation = 'edit'
GROUP BY f1.file_path, f2.file_path
HAVING COUNT(*) >= 3
ORDER BY co_edit_count DESC;

-- "What did I last do with auth code?"
SELECT * FROM file_interactions
WHERE file_path LIKE '%auth%'
ORDER BY timestamp DESC
LIMIT 10;

-- "Success rate for this tool"
SELECT
    tool_name,
    COUNT(*) as total,
    SUM(CASE WHEN success THEN 1 ELSE 0 END) as successes,
    AVG(duration_ms) as avg_duration
FROM tool_executions
WHERE timestamp > NOW() - INTERVAL '7 days'
GROUP BY tool_name;

-- "What was I working on before?"
SELECT * FROM task_history
WHERE status = 'paused' OR status = 'active'
ORDER BY started_at DESC;
```

### 3. Working Memory (Dynamic Context Window)

**Purpose**: Maintain active LLM conversation without filling up

**Key Innovation**: Intelligent pruning removes low-value context while keeping essentials

**Storage**: In-memory, snapshot to DuckDB every N prunings

**Data Structure**:

```rust
pub struct ContextWindow {
    items: Vec<ContextItem>,
    token_count: usize,
    max_tokens: usize, // e.g., 180000 for Claude (leave 20K buffer)
    session_id: String,
    current_task_id: Option<String>,
    pruning_count: usize,
}

pub struct ContextItem {
    id: Uuid,
    content: String,
    item_type: ContextItemType,
    timestamp: DateTime<Utc>,
    relevance_score: f32, // 0.0 to 1.0
    token_cost: usize,
    task_id: Option<String>,
    dependencies: Vec<Uuid>, // other items this depends on
    sticky: bool, // never remove (e.g., system prompt)
}

pub enum ContextItemType {
    SystemPrompt,
    UserMessage { intent: Option<UserIntent> },
    AssistantResponse { tool_calls: Vec<String> },
    ToolResult { tool_name: String, file_path: Option<String> },
    CodeSnippet { file_path: String, line_range: (usize, usize) },
    TaskState { task_id: String, description: String },
    KnowledgeGraphQuery { query: String, result_summary: String },
}
```

**Dynamic Pruning Algorithm**:

```rust
impl DynamicContextManager {
    /// Check if pruning needed, execute if necessary
    pub async fn maybe_prune(&mut self) -> Result<()> {
        // Threshold: 80% of max tokens
        let threshold = (self.window.max_tokens as f32 * 0.8) as usize;

        if self.window.token_count > threshold {
            info!("Context at {}%, pruning...",
                  (self.window.token_count * 100) / self.window.max_tokens);
            self.prune_context().await?;
        }

        Ok(())
    }

    async fn prune_context(&mut self) -> Result<()> {
        let start_tokens = self.window.token_count;

        // 1. Calculate relevance score for each item
        let mut scored_items: Vec<_> = self.window.items
            .iter()
            .enumerate()
            .filter(|(_, item)| !item.sticky) // Never remove sticky items
            .map(|(idx, item)| {
                let score = self.calculate_relevance(item);
                (idx, item, score)
            })
            .collect();

        // 2. Sort by relevance (lowest first)
        scored_items.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

        // 3. Remove bottom 30% by token count (not item count)
        let target_removal = (self.window.token_count as f32 * 0.3) as usize;
        let mut removed_tokens = 0;
        let mut removed_items = vec![];

        for (idx, item, score) in scored_items {
            if removed_tokens >= target_removal {
                break;
            }

            removed_tokens += item.token_cost;
            removed_items.push((idx, item.clone()));
        }

        // 4. Before removing: summarize to episodic memory
        for (_, item) in &removed_items {
            self.summarize_to_episodic(item).await?;
        }

        // 5. Remove from context (reverse order to preserve indices)
        for (idx, _) in removed_items.iter().rev() {
            self.window.items.remove(*idx);
        }

        // 6. Update token count
        self.window.token_count -= removed_tokens;
        self.window.pruning_count += 1;

        // 7. Snapshot for debugging
        self.snapshot_context("pruning", &removed_items).await?;

        info!("Pruned {} items, {} → {} tokens",
              removed_items.len(), start_tokens, self.window.token_count);

        Ok(())
    }

    fn calculate_relevance(&self, item: &ContextItem) -> f32 {
        let mut score = 1.0;

        // Factor 1: Time decay (older = less relevant)
        let age_minutes = (Utc::now() - item.timestamp).num_minutes() as f32;
        let time_score = (-age_minutes / 60.0).exp(); // exponential decay, half-life ~1 hour
        score *= time_score;

        // Factor 2: Task association (current task = more relevant)
        if let Some(task_id) = &self.window.current_task_id {
            if item.task_id.as_ref() == Some(task_id) {
                score *= 2.0; // 2x boost for current task
            }
        }

        // Factor 3: Dependencies (other items reference this)
        let dependency_boost = 1.0 + (self.count_dependents(item.id) as f32 * 0.2);
        score *= dependency_boost;

        // Factor 4: Item type (some types are more important)
        let type_multiplier = match &item.item_type {
            ContextItemType::SystemPrompt => 100.0, // Never remove (but marked sticky anyway)
            ContextItemType::TaskState { .. } => 2.0, // Keep task state
            ContextItemType::UserMessage { .. } => 1.5, // Keep user intent
            ContextItemType::AssistantResponse { tool_calls } => {
                if !tool_calls.is_empty() { 1.2 } else { 0.9 }
            },
            ContextItemType::ToolResult { .. } => 0.8, // Tool results decay
            ContextItemType::CodeSnippet { .. } => 0.7, // Code snippets decay faster
            ContextItemType::KnowledgeGraphQuery { .. } => 0.6, // Can re-query graph
        };
        score *= type_multiplier;

        // Factor 5: Explicit relevance score (set by query)
        score *= item.relevance_score;

        score.max(0.0).min(100.0) // Clamp to reasonable range
    }

    fn count_dependents(&self, item_id: Uuid) -> usize {
        self.window.items
            .iter()
            .filter(|item| item.dependencies.contains(&item_id))
            .count()
    }

    async fn summarize_to_episodic(&self, item: &ContextItem) -> Result<()> {
        match &item.item_type {
            ContextItemType::ToolResult { tool_name, file_path } => {
                // Already tracked in tool_executions, just note it was pruned
                debug!("Pruned tool result: {} on {:?}", tool_name, file_path);
            }
            ContextItemType::CodeSnippet { file_path, line_range } => {
                // Record that we viewed this code
                self.episodic_memory.record_file_interaction(
                    self.window.session_id.clone(),
                    self.window.current_task_id.clone(),
                    file_path.clone(),
                    "viewed".to_string(),
                    Some(format!("Lines {}-{}", line_range.0, line_range.1)),
                    true,
                ).await?;
            }
            ContextItemType::AssistantResponse { tool_calls } => {
                // Tool calls already tracked, just note reasoning was pruned
                debug!("Pruned assistant response with {} tool calls", tool_calls.len());
            }
            _ => {
                // Generic: store summary
                let summary = item.content.chars().take(200).collect::<String>();
                debug!("Pruned {:?}: {}", item.item_type, summary);
            }
        }

        Ok(())
    }

    async fn snapshot_context(&self, reason: &str, removed: &[(usize, ContextItem)]) -> Result<()> {
        let snapshot = json!({
            "timestamp": Utc::now(),
            "session_id": self.window.session_id,
            "task_id": self.window.current_task_id,
            "total_tokens": self.window.token_count,
            "item_count": self.window.items.len(),
            "items": self.window.items.iter().map(|item| json!({
                "type": format!("{:?}", item.item_type),
                "tokens": item.token_cost,
                "relevance": item.relevance_score,
                "age_minutes": (Utc::now() - item.timestamp).num_minutes(),
            })).collect::<Vec<_>>(),
            "pruned_items": removed.iter().map(|(_, item)| json!({
                "type": format!("{:?}", item.item_type),
                "tokens": item.token_cost,
                "relevance": self.calculate_relevance(item),
            })).collect::<Vec<_>>(),
            "reason": reason,
        });

        self.episodic_memory.store_snapshot(snapshot).await?;
        Ok(())
    }
}
```

## Intelligent Context Preparation

**Before each LLM call**, fetch relevant context from memory:

```rust
impl DynamicContextManager {
    pub async fn prepare_context(&mut self, user_message: &str) -> Result<Vec<Message>> {
        // 1. Prune if needed
        self.maybe_prune().await?;

        // 2. Add user message to window
        let user_item = ContextItem {
            id: Uuid::new_v4(),
            content: user_message.to_string(),
            item_type: ContextItemType::UserMessage { intent: None },
            timestamp: Utc::now(),
            relevance_score: 1.0,
            token_cost: count_tokens(user_message),
            task_id: self.window.current_task_id.clone(),
            dependencies: vec![],
            sticky: false,
        };
        self.window.items.push(user_item);

        // 3. Fetch relevant code from knowledge graph
        let relevant_code = self.knowledge_graph
            .find_relevant_to_query(user_message)
            .await?;

        for code_snippet in relevant_code {
            let snippet_item = ContextItem {
                id: Uuid::new_v4(),
                content: code_snippet.content.clone(),
                item_type: ContextItemType::CodeSnippet {
                    file_path: code_snippet.file_path.clone(),
                    line_range: code_snippet.line_range,
                },
                timestamp: Utc::now(),
                relevance_score: code_snippet.relevance,
                token_cost: count_tokens(&code_snippet.content),
                task_id: self.window.current_task_id.clone(),
                dependencies: vec![],
                sticky: false,
            };
            self.window.items.push(snippet_item);
        }

        // 4. Fetch relevant history from episodic memory
        let similar_tasks = self.episodic_memory
            .find_similar_tasks(user_message, 3)
            .await?;

        if !similar_tasks.is_empty() {
            let history_summary = format!(
                "Relevant past work:\n{}",
                similar_tasks.iter()
                    .map(|t| format!("- {}: {}", t.description, t.outcome))
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            let history_item = ContextItem {
                id: Uuid::new_v4(),
                content: history_summary.clone(),
                item_type: ContextItemType::AssistantResponse { tool_calls: vec![] },
                timestamp: Utc::now(),
                relevance_score: 0.8,
                token_cost: count_tokens(&history_summary),
                task_id: self.window.current_task_id.clone(),
                dependencies: vec![],
                sticky: false,
            };
            self.window.items.push(history_item);
        }

        // 5. Build final message list for LLM
        let messages: Vec<Message> = self.window.items
            .iter()
            .map(|item| self.item_to_message(item))
            .collect();

        Ok(messages)
    }
}
```

## What to Track (Summary)

### Track Everything:
1. **Every tool execution** → tool_executions
   - Which tool, when, parameters, result, success/failure
   - Duration, token cost, context at time of call

2. **Every file interaction** → file_interactions
   - Read, write, edit, search, analyze operations
   - Which file, when, in what context
   - Line ranges modified

3. **Every task** → task_history
   - User-level goals: "fix authentication bug"
   - Status (active, completed, failed, paused)
   - Files touched, tools used, outcome

4. **Context window state** → context_snapshots
   - Periodic snapshots (every pruning, task switch)
   - What was in context, what was removed, why
   - For debugging and recovery

5. **Learned patterns** → learned_patterns
   - Files edited together (co-edit patterns)
   - Error → fix patterns
   - Refactoring patterns (rename → update imports)

### Don't Track:
- ❌ Raw LLM responses (too large, not useful)
- ❌ Full file contents (use knowledge graph)
- ❌ Intermediate reasoning (unless explicitly requested)

## Storage Decision: DuckDB

**Why DuckDB over SQLite**:

1. ✅ **Already have it**: `src/intelligence/memory/duckdb_memory.rs`
2. ✅ **Analytical queries**: "Show me all times I worked on auth code"
3. ✅ **JSON columns**: Flexible schema for patterns, metadata
4. ✅ **Vectorized execution**: Fast aggregations
5. ✅ **Embedded**: No server, single file
6. ✅ **Arrow integration**: Can export to DataFrame for analysis

**Size estimate**:
- 1000 tool executions: ~500KB
- 10,000 file interactions: ~5MB
- 100 tasks: ~100KB
- 1000 context snapshots: ~10MB
- **Total for typical session**: <20MB

## Implementation Timeline

### Week 3: DuckDB Episodic Memory (1 week)

**Day 1-2**: Schema + basic operations
- Create tables (tool_executions, file_interactions, task_history)
- Insert/query functions
- Connection pooling

**Day 3-4**: Recording layer
- Hook into tool execution
- Record every file interaction
- Track task lifecycle

**Day 5-7**: Query layer
- "Have I seen this before?"
- Co-edit pattern detection
- Similar task lookup

### Week 4: Knowledge Graph Port (1 week)

**Day 1-3**: Graph building
- Port tree-sitter extraction from Python
- Build petgraph structure
- Serialize/deserialize

**Day 4-5**: Query interface
- get_file_contents, get_callers, find_symbol
- Integration with existing semantic search

**Day 6-7**: Incremental updates
- File changed → re-parse only that file
- Edge case handling

### Week 5: Dynamic Context Management (1 week)

**Day 1-3**: Context window implementation
- ContextWindow struct
- Relevance scoring algorithm
- Pruning logic

**Day 4-5**: Integration
- Connect to episodic memory
- Connect to knowledge graph
- Test with real tasks

**Day 6-7**: Validation
- Run POC benchmark tasks
- Validate 60% improvement holds
- Fix issues

## Competitive Advantage

**Claude Code**:
- ❌ Restarts when context fills
- ❌ No cross-session memory
- ❌ No learned patterns
- ❌ Repeats work

**Aircher**:
- ✅ Continuous work (dynamic pruning)
- ✅ Episodic memory persists
- ✅ Knowledge graph persists
- ✅ Pattern learning improves over time
- ✅ 60% fewer tool calls (validated)

**This is the research contribution**: "Memory-Augmented Coding Agents: Empirical Validation"

## Next Steps

**Week 2** (current): Finish code understanding tools
**Week 3**: Port memory system to Rust (use this architecture)
**Week 7**: Benchmark vs Claude Code (prove continuous work advantage)

---

**See also**:
- ai/PLAN.md (10-week execution plan)
- poc-memory-agent/ (Python POC with validation)
- src/intelligence/memory/ (existing DuckDB infrastructure)
