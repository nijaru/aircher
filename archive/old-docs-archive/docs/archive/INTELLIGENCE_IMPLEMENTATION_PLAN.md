# Intelligence System Implementation Plan

## Current Status Assessment

### ✅ What's Working
- DuckDB pattern storage implemented
- Pattern learning data structures defined
- Similarity search logic created
- Predictive intelligence algorithms built
- File relationship tracking ready

### ❌ Critical Issues
1. **NOT CONNECTED** - Intelligence never called by agent
2. **Thread Safety Broken** - DuckDBConnection not Send (async fails)
3. **No Embeddings** - Text similarity too crude
4. **No Integration Hooks** - Agent unaware of intelligence

## Implementation Plan

### Phase 1: Fix Thread Safety (30 minutes)
**Problem**: DuckDBConnection cannot be shared between threads
**Solution**: Use blocking tasks for DB operations

#### Step 1.1: Update DuckDBMemory
```rust
// In src/intelligence/duckdb_memory.rs

use tokio::task;

impl DuckDBMemory {
    pub async fn record_pattern(&self, pattern: Pattern) -> Result<()> {
        let db = self.db.clone();
        let result = task::spawn_blocking(move || {
            let conn = db.blocking_lock();
            // All DB operations here
            Self::record_pattern_sync(&*conn, pattern)
        }).await??;
        Ok(result)
    }

    fn record_pattern_sync(conn: &DuckDBConnection, pattern: Pattern) -> Result<()> {
        // Existing DB logic moved here
    }
}
```

### Phase 2: Connect to AgentController (45 minutes)

#### Step 2.1: Make Intelligence Active
```rust
// In src/agent/controller.rs:18
pub struct AgentController {
    tools: ToolRegistry,
    intelligence: Arc<Mutex<IntelligenceEngine>>, // Remove underscore
    // ...
}
```

#### Step 2.2: Query Before Acting
```rust
// In src/agent/controller.rs:83
pub async fn process_message(&mut self, user_message: &str, ...) -> Result<...> {
    // NEW: Get intelligence context FIRST
    let intelligence_context = self.get_intelligence_context(user_message).await?;

    // ... existing code ...
}

async fn get_intelligence_context(&self, query: &str) -> Result<String> {
    let intel = self.intelligence.lock().await;

    // Get suggestions from past patterns
    let suggestions = intel.get_suggestions(query, None).await?;

    // Find related files if mentioned
    if let Some(file) = self.extract_file_mention(query) {
        let related = intel.predict_file_changes(&file).await?;
        return Ok(format!("Intelligence: {}\nRelated files: {:?}",
                         suggestions, related));
    }

    Ok(suggestions)
}
```

#### Step 2.3: Track During Execution
```rust
// In src/agent/controller.rs (execute_tools)
async fn execute_tools_tracked(&self, tool_calls: &[ToolCall])
    -> (Vec<(String, Result<Value>)>, Vec<AgentAction>)
{
    let mut results = Vec::new();
    let mut actions = Vec::new();

    for call in tool_calls {
        let start = Instant::now();
        let result = self.tools.execute(&call.name, &call.parameters).await;

        actions.push(AgentAction {
            tool: call.name.clone(),
            params: call.parameters.clone(),
            success: result.is_ok(),
            duration_ms: start.elapsed().as_millis() as u64,
            result_summary: format!("{:?}", result),
        });

        results.push((call.name.clone(), result));
    }

    (results, actions)
}
```

#### Step 2.4: Record After Completion
```rust
// At end of process_message, before returning
if !actions.is_empty() {
    self.record_interaction(user_message, actions, success).await?;
}

async fn record_interaction(&self, query: &str, actions: Vec<AgentAction>, success: bool) -> Result<()> {
    let intel = self.intelligence.lock().await;

    let pattern = Pattern {
        id: Uuid::new_v4().to_string(),
        description: query.to_string(),
        context: query.to_string(),
        actions,
        files_involved: self.extract_files_from_actions(&actions),
        success,
        timestamp: Utc::now(),
        session_id: "current".to_string(),
        embedding_text: query.to_string(),
    };

    intel.record_pattern(pattern).await?;
    Ok(())
}
```

### Phase 3: Add Embedding Support (30 minutes)

#### Step 3.1: Connect Semantic Search
```rust
// In src/intelligence/mod.rs
impl IntelligenceEngine {
    pub async fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        if let Some(semantic) = &self.semantic_search {
            let search = semantic.read().await;
            Ok(search.embed_text(text).await?)
        } else {
            Ok(vec![]) // Fallback
        }
    }
}
```

#### Step 3.2: Use Embeddings in Patterns
```rust
// When recording patterns
let embedding = self.get_embedding(query).await?;
let pattern = Pattern {
    // ... other fields ...
    embedding, // Add actual embedding
};

// When searching
let query_embedding = self.get_embedding(query).await?;
let similar = self.find_similar_by_embedding(&query_embedding).await?;
```

### Phase 4: Test End-to-End (30 minutes)

#### Test Script
```rust
#[tokio::test]
async fn test_intelligence_integration() {
    // 1. Create agent with intelligence
    let intel = IntelligenceEngine::new(...).await?;
    let agent = AgentController::new(intel, ...).await?;

    // 2. First query - should have no patterns
    let (response1, _) = agent.process_message("fix error in main.rs").await?;
    assert!(response1.contains("error"));

    // 3. Second similar query - should use learned pattern
    let (response2, _) = agent.process_message("fix error in lib.rs").await?;
    // Should be faster/more accurate

    // 4. Verify pattern was recorded
    let patterns = agent.intelligence.get_patterns().await?;
    assert!(!patterns.is_empty());
}
```

## Migration Steps

### Step 1: Fix Compilation (15 min)
```bash
# Fix thread safety issues
cargo check
# Fix any remaining compilation errors
```

### Step 2: Wire Intelligence (30 min)
```bash
# Connect intelligence to agent
# Remove underscore from _intelligence
# Add enhancement methods
```

### Step 3: Test Integration (15 min)
```bash
# Run tests
cargo test intelligence
# Manual test in TUI
cargo run
```

## Success Criteria

1. **Agent queries intelligence before acting**
   - Verify enhanced prompts contain past patterns

2. **Actions are tracked during execution**
   - Check that tool uses are recorded

3. **Patterns are saved after completion**
   - Verify database contains patterns

4. **Next query uses learned patterns**
   - Second similar query should be enhanced

## Timeline

- **Hour 1**: Fix thread safety + Wire intelligence
- **Hour 2**: Add embeddings + Test integration
- **Total**: 2 hours to full functionality

## Risk Mitigation

### If thread safety fix doesn't work:
- Alternative: Use a different DB (SQLite with rusqlite)
- Alternative: Use file-based storage temporarily

### If integration breaks agent:
- Feature flag to disable intelligence
- Graceful fallback to non-intelligent mode

### If embeddings are slow:
- Cache embeddings in memory
- Use smaller embedding model

## Final Checklist

- [ ] DuckDB operations use spawn_blocking
- [ ] Intelligence connected to AgentController
- [ ] Patterns queried before LLM calls
- [ ] Actions tracked during execution
- [ ] Patterns recorded after completion
- [ ] Embeddings replace text similarity
- [ ] Tests pass
- [ ] Manual testing successful

Once complete, the agent will:
1. Learn from every interaction
2. Get smarter over time
3. Predict user needs
4. Avoid past mistakes

**This transforms Aircher from a tool to an intelligent assistant.**
