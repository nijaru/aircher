# Context Awareness Improvement

**Created**: 2025-10-30
**Issue**: Agent tracks context but doesn't expose it to the model for decision-making
**User Insight**: "Claude mentions we have 97K tokens left... we should provide this info to the model"

## Current State

**What We Have** ✅:
```rust
// src/intelligence/working_memory.rs
pub struct ContextWindow {
    token_count: usize,      // Current usage
    max_tokens: usize,       // Limit (e.g., 200K)
    ...
}

pub struct ContextWindowStats {
    total_tokens: usize,
    max_tokens: usize,
    utilization: f32,        // Percentage
    ...
}
```

**What We Don't Do** ❌:
- Model doesn't see context stats
- Model can't make decisions based on remaining capacity
- Model can't request context pruning/editing

## The Problem

**Scenario**: Agent with 150K/200K tokens used
- **Current**: Agent doesn't know it's running out of space
- **Better**: Agent sees "75% full" and decides whether to:
  - Continue with current approach
  - Summarize old context
  - Remove low-value items
  - Switch strategies

## Solution: Context-Aware System Prompts

### Approach 1: Include in Every Turn (Simple)

**Implementation**: Add context stats to system message

```rust
// src/agent/core.rs
pub async fn execute_turn(&mut self, user_message: &str) -> Result<Response> {
    // Get current context stats
    let stats = self.context_window.get_stats();

    // Build context-aware system prompt
    let system_prompt = format!(
        "{}\n\n\
        **Context Status**:\n\
        - Tokens: {}/{} used ({}%)\n\
        - Items: {} context items ({} pruning operations so far)\n\
        - Capacity: {} tokens remaining\n\
        \n\
        Use this information to decide:\n\
        - Whether to continue current approach\n\
        - Whether to request context pruning\n\
        - Whether to summarize old discussions\n\
        - Whether to switch strategies\n\
        \n\
        If approaching limit (>80%), consider:\n\
        1. Removing low-value context items\n\
        2. Summarizing completed tasks\n\
        3. Focusing on essential information",
        self.base_system_prompt,
        stats.total_tokens,
        stats.max_tokens,
        (stats.utilization * 100.0) as u32,
        stats.total_items,
        stats.pruning_count,
        stats.max_tokens - stats.total_tokens
    );

    // Continue with chat request...
}
```

**Example Output to Model**:
```
**Context Status**:
- Tokens: 97,234 / 200,000 used (48%)
- Items: 47 context items (0 pruning operations so far)
- Capacity: 102,766 tokens remaining

Use this information to decide whether to continue current approach...
```

### Approach 2: Agent-Directed Pruning (Better)

**Add new tool**: `edit_context`

```rust
pub struct EditContextTool;

#[async_trait]
impl Tool for EditContextTool {
    fn name(&self) -> &str { "edit_context" }

    fn description(&self) -> &str {
        "Edit conversation context to manage token usage. \
         Can remove old messages, summarize discussions, or mark items as removable."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["remove_range", "summarize_range", "list_items"],
                    "description": "What to do with context"
                },
                "range": {
                    "type": "object",
                    "properties": {
                        "start": {"type": "integer"},
                        "end": {"type": "integer"}
                    }
                },
                "summary": {
                    "type": "string",
                    "description": "Summary to replace removed items (for summarize_range)"
                }
            }
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput> {
        let action = params["action"].as_str().unwrap_or("list_items");

        match action {
            "list_items" => {
                // Return list of context items with token costs
                let items = self.context_window.list_items_with_costs();
                Ok(ToolOutput::success(items))
            }
            "remove_range" => {
                // Remove items in range
                let start = params["range"]["start"].as_u64().unwrap() as usize;
                let end = params["range"]["end"].as_u64().unwrap() as usize;
                self.context_window.remove_range(start, end)?;
                Ok(ToolOutput::success("Removed items"))
            }
            "summarize_range" => {
                // Replace range with summary
                let start = params["range"]["start"].as_u64().unwrap() as usize;
                let end = params["range"]["end"].as_u64().unwrap() as usize;
                let summary = params["summary"].as_str().unwrap();
                self.context_window.summarize_range(start, end, summary)?;
                Ok(ToolOutput::success("Summarized items"))
            }
            _ => Err(anyhow!("Unknown action"))
        }
    }
}
```

**Agent Usage Example**:
```
Agent: I see I'm at 78% context usage. Let me review what's in context.

<tool_use name="edit_context">
{
  "action": "list_items"
}
</tool_use>

<tool_result>
Items 0-10: Initial task discussion (12,300 tokens)
Items 11-25: File analysis results (8,400 tokens)
Items 26-40: Recent work (15,200 tokens)
</tool_result>

Agent: I can summarize the initial task discussion to save space.

<tool_use name="edit_context">
{
  "action": "summarize_range",
  "range": {"start": 0, "end": 10},
  "summary": "Task: Fix test_needs_pruning. Approach: Change > to >= in working_memory.rs line 162."
}
</tool_use>
```

### Approach 3: Automatic with Agent Override (Best)

**Combine both approaches**:
1. Automatic pruning at 80% (current behavior)
2. Context stats visible to agent (Approach 1)
3. Agent can manually edit if needed (Approach 2)

**Benefits**:
- Agent aware of context state
- Can make informed decisions
- Can override automatic pruning
- Can be proactive about context management

## Implementation Priority

### Phase 1: Simple Stats (1 hour)
- Add context stats to system prompt
- Model can see but not edit
- Immediate improvement

### Phase 2: Read-Only Tool (2 hours)
- Add `list_context` tool
- Agent can inspect context items
- Can make suggestions about pruning

### Phase 3: Edit Tool (4 hours)
- Add full `edit_context` tool
- Agent can remove/summarize items
- Full context control

### Phase 4: Smart Heuristics (future)
- Agent learns what to keep/remove
- Patterns of effective context management
- Proactive suggestions

## Expected Benefits

**Better Decision Making**:
- Agent knows when to be concise vs verbose
- Can adapt strategy based on remaining space
- Won't run out of context unexpectedly

**Improved Efficiency**:
- Agent removes low-value items proactively
- Keeps essential information
- Summarizes completed work

**User Experience**:
- No surprise "context full" errors
- Agent explains context decisions
- Transparent token management

## Example Scenarios

### Scenario 1: Approaching Limit
```
Agent: I notice I'm at 85% context usage (170K/200K tokens).
       I can summarize our earlier discussion about the Skills system
       since that task is complete and paused. This will free up ~20K tokens.

       Proceed with summarization? (yes/no)
```

### Scenario 2: Complex Task
```
Agent: This task will require significant file reading (~50K tokens).
       Currently at 60% usage (120K/200K tokens).
       I have sufficient capacity. Proceeding with file analysis.
```

### Scenario 3: Proactive Management
```
Agent: I'm at 70% context usage. Before continuing, I'll remove:
       - Old test output logs (Items 15-22, ~8K tokens)
       - Completed task discussions (Items 5-10, ~6K tokens)

       This will free 14K tokens and keep essential information.
```

## Implementation Checklist

### Phase 1 (Immediate):
- [ ] Add `get_stats()` method to ContextWindow (already exists)
- [ ] Inject stats into system prompt on each turn
- [ ] Test with long conversations
- [ ] Document in user-facing docs

### Phase 2 (Soon):
- [ ] Create `list_context` tool
- [ ] Return items with token costs
- [ ] Add to tool registry
- [ ] Test agent usage

### Phase 3 (Later):
- [ ] Create full `edit_context` tool
- [ ] Implement remove/summarize operations
- [ ] Add safety checks (can't remove system prompts)
- [ ] Test with agent-directed pruning

### Phase 4 (Future):
- [ ] Track which context items were most useful
- [ ] Learn patterns of effective pruning
- [ ] Proactive suggestions
- [ ] A/B test effectiveness

## References

- **Working Memory**: `src/intelligence/working_memory.rs` (ContextWindow struct)
- **Dynamic Context**: `src/agent/dynamic_context.rs` (context management)
- **Context Engine**: `src/agent/context_engine.rs` (context building)

## User's Insight

> "claude has mentioned before that we have say 97k tokens left in max context for the conversation and shouldnt need to worry about compacting before completing the task. we should provide this info to the model and use that to make decisions on tasks."

**This is correct** - exposing context stats makes the agent significantly more intelligent about resource management. Should implement Phase 1 immediately.
