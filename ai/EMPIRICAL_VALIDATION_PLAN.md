# Empirical Validation Plan (Week 9)

**Created**: 2025-10-29
**Status**: Ready to execute
**Goal**: Validate hybrid architecture improvements vs Claude Code with measurable metrics

---

## Executive Summary

**What's Actually Wired** (Code Review Findings):
- ✅ Event bus: Emits FileChanged events (core.rs:1639-1655)
- ✅ LSP manager: Listens and processes events (lsp_manager.rs)
- ✅ Mode enforcement: Checks allowed_tools() before execution (core.rs:1596-1611)
- ✅ Git snapshots: Creates before risky operations (core.rs:1614-1633)
- ✅ Model router: Selects model and records usage (core.rs:332-341, 548-552)
- ✅ Specialized agents: System prompts applied (core.rs:323-324, 449)
- ✅ Research sub-agents: Spawning logic implemented (core.rs:345-382)

**What Needs Validation**:
- ⚠️ Memory systems (episodic, knowledge graph, working) - exist but usage unclear
- ⚠️ LSP diagnostics - listening but actual diagnostics trigger unclear
- ⚠️ Research sub-agents - spawning implemented but results integration needs testing
- ⚠️ Cost tracking - recording happens but no reporting/validation
- ⚠️ Specialized agent effectiveness - prompts applied but quality untested

---

## Part 1: What to Test (Core Claims)

### Claim 1: 60% Tool Call Reduction (Memory Systems)
**Target Metric**: 7.5 → 3.0 tool calls (from POC)

**Test Scenarios**:
1. **Multi-file refactoring** - Rename function across 5 files
   - Baseline (no memory): Count tool calls needed
   - With memory: Count tool calls, check if memory prevents re-reading same files

2. **Bug fixing workflow** - Fix authentication error
   - Baseline: Count tool calls to locate issue
   - With memory: Check if episodic memory recalls similar fixes

3. **Code exploration** - "How does authentication work?"
   - Baseline: Count files examined
   - With memory: Check if knowledge graph provides instant answers

**What to Measure**:
- Total tool calls (read_file, search_code, list_files, etc.)
- Duplicate file reads (should be 0 with memory)
- Knowledge graph queries vs file system operations
- Episodic memory cache hits

**Current Status**:
- ⚠️ **BLOCKER**: Need to verify memory systems are actually queried during execution
- Check: DynamicContextManager usage in process_message()
- Check: IntelligenceEngine memory methods called

### Claim 2: 90% Research Speedup (Parallel Sub-Agents)
**Target Metric**: 10x faster for "find all X" queries

**Test Scenarios**:
1. **"Find all authentication patterns"** - Should spawn 5-10 sub-agents
2. **"List all database queries"** - Should parallelize directory scanning
3. **"Search for error handling"** - Should distribute across modules

**What to Measure**:
- Time to complete (baseline vs parallel)
- Number of sub-agents spawned
- Results aggregation quality
- Token usage (parallel should cost more but be faster)

**Current Status**:
- ✅ Spawning logic implemented (core.rs:345-382)
- ⚠️ Need to test with real Ollama models
- ⚠️ Results integration unclear (research_summary added to prompt but not validated)

### Claim 3: 0% Sub-Agent Usage for Coding (Avoid 15x Waste)
**Target Metric**: Coding tasks NEVER spawn sub-agents

**Test Scenarios**:
1. **"Implement user authentication"** - Should NOT spawn sub-agents
2. **"Fix bug in login.rs"** - Should NOT spawn sub-agents
3. **"Refactor database connection"** - Should NOT spawn sub-agents

**What to Measure**:
- Sub-agent spawn count (should be 0)
- Mode selection (should be Build mode, not Plan)
- Token usage (should be single-agent only)

**Current Status**:
- ✅ Logic implemented: `selected_agent.can_spawn_subagents` check
- ✅ Only Explorer agents with research queries spawn
- ⚠️ Need to validate intent classification accuracy

### Claim 4: 50% Fewer Runtime Errors (LSP Self-Correction)
**Target Metric**: Agent catches errors before execution

**Test Scenarios**:
1. **Edit file with syntax error** - LSP should report, agent should fix
2. **Type mismatch in Rust code** - rust-analyzer should catch
3. **Undefined variable in Python** - pyright should catch

**What to Measure**:
- LSP diagnostics received after edits
- Agent self-correction attempts (re-edits based on diagnostics)
- Runtime errors that slip through vs caught by LSP

**Current Status**:
- ✅ Event bus wired (core.rs:1649-1653)
- ✅ LSP manager listening (lsp_manager.rs:start_listening)
- ⚠️ **BLOCKER**: Need to verify diagnostics are actually returned to agent
- ⚠️ Need to implement agent reaction to diagnostics

### Claim 5: 40% Cost Reduction (Model Routing)
**Target Metric**: $100 → $60 for same tasks

**Test Scenarios**:
1. **Mix of simple and complex tasks** - Should use Haiku for simple, Sonnet for complex
2. **Research queries** - Should use Haiku sub-agents (cheap parallelization)
3. **Code generation** - Should use Sonnet (quality needed)

**What to Measure**:
- Model selection distribution (% Haiku vs Sonnet vs Opus)
- Token usage per model (input/output)
- Estimated cost per task
- Cost with routing vs all-Sonnet baseline

**Current Status**:
- ✅ Model selection implemented (core.rs:332-341)
- ✅ Usage recording implemented (core.rs:548-552)
- ⚠️ No cost reporting/visualization yet
- ⚠️ Need to generate cost reports

### Claim 6: 100% Operation Recovery (Git Snapshots)
**Target Metric**: All failed operations can be rolled back

**Test Scenarios**:
1. **Bad edit that breaks code** - Should rollback to snapshot
2. **Failed bash command** - Should rollback to snapshot
3. **Rejected file write** - Should rollback to snapshot

**What to Measure**:
- Snapshot creation success rate
- Rollback success rate
- Time to rollback
- Workspace state after rollback (should be clean)

**Current Status**:
- ✅ Snapshot creation implemented (core.rs:1614-1633)
- ⚠️ **BLOCKER**: Rollback logic not implemented yet
- ⚠️ Need error handling that triggers rollback
- ⚠️ Need validation that rollback actually works

---

## Part 2: What to Fix (Critical Issues)

### Issue 1: Memory Systems Not Queried ⚠️ CRITICAL
**Problem**: Memory systems exist but may not be used in execution path

**Code Locations to Check**:
- `src/agent/core.rs:466-480` - context_manager.update_context() called
- `src/agent/dynamic_context.rs` - Check if actually queries memory
- `src/intelligence/mod.rs` - Check if memory methods are called

**Fix Required**:
```rust
// In process_message(), before model call:

// 1. Check episodic memory for similar tasks
let past_tasks = self.intelligence.episodic_memory.find_similar_tasks(user_message, 3).await?;
if !past_tasks.is_empty() {
    info!("Found {} similar past tasks in episodic memory", past_tasks.len());
    // Add to system_prompt
}

// 2. Query knowledge graph for relevant code
let relevant_code = self.intelligence.knowledge_graph.find_relevant(user_message).await?;
if !relevant_code.is_empty() {
    info!("Knowledge graph found {} relevant code locations", relevant_code.len());
    // Add to system_prompt
}

// 3. Update working memory with this request
self.context_manager.add_to_context(user_message, &enhanced_context).await?;
```

**Validation**:
- Add debug logs showing memory queries
- Run benchmark task and check logs
- Measure tool call reduction with/without memory

### Issue 2: LSP Diagnostics Not Returning to Agent ⚠️ CRITICAL
**Problem**: LSP listens to events but diagnostics not fed back to agent

**Code Locations to Check**:
- `src/agent/lsp_manager.rs:start_listening()` - Event listener loop
- `src/agent/lsp_manager.rs` - Diagnostics storage in HashMap
- `src/agent/core.rs` - No code consuming diagnostics

**Fix Required**:
```rust
// After tool execution in process_message():

// If tool was edit_file or write_file, wait for LSP diagnostics
if tool_name == "edit_file" || tool_name == "write_file" {
    if let Some(path) = params.get("path") {
        // Wait up to 2 seconds for LSP diagnostics
        tokio::time::sleep(Duration::from_millis(2000)).await;

        let diagnostics = self.lsp_manager.get_diagnostics(path).await?;
        if !diagnostics.is_empty() {
            warn!("LSP found {} issues in {}", diagnostics.len(), path);

            // Add diagnostics to next LLM call
            let diagnostic_summary = format!(
                "⚠️ LSP Diagnostics:\n{}",
                diagnostics.iter()
                    .map(|d| format!("Line {}: {}", d.line, d.message))
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            // Give agent chance to self-correct
            // TODO: Implement self-correction loop
        }
    }
}
```

**Validation**:
- Edit file with intentional error
- Check logs for LSP diagnostics
- Verify agent receives diagnostics
- Test self-correction flow

### Issue 3: Research Sub-Agent Results Not Validated ⚠️ MEDIUM
**Problem**: Sub-agents spawn but results integration unclear

**Code Locations**:
- `src/agent/core.rs:354-381` - Spawning and waiting
- `src/agent/research_subagents.rs` - ResearchSubAgentManager

**Fix Required**:
```rust
// In process_message() after research sub-agents:

// Validate results are non-empty and useful
let mut research_summary = String::new();
for (i, result) in results.iter().enumerate() {
    if result.success && !result.findings.is_empty() {
        research_summary.push_str(&format!(
            "## Sub-Agent {} Findings:\n{}\n\n",
            i + 1,
            result.findings
        ));
    }
}

if research_summary.is_empty() {
    warn!("Research sub-agents returned no useful results");
} else {
    info!("Research summary: {} chars", research_summary.len());
    // Add to system_prompt (already done at line 369)
}
```

**Validation**:
- Run "find all X" query with Ollama
- Check logs for sub-agent spawning
- Verify results added to prompt
- Compare response quality with/without sub-agents

### Issue 4: Git Rollback Not Implemented ⚠️ MEDIUM
**Problem**: Snapshots created but rollback logic missing

**Code Locations**:
- `src/agent/core.rs:1616-1633` - Snapshot creation
- `src/agent/git_snapshots.rs` - SnapshotManager

**Fix Required**:
```rust
// In execute_single_tool() after tool execution:

match tool.execute(params.clone()).await {
    Ok(output) => {
        if !output.success && snapshot_id.is_some() {
            // Tool failed, rollback to snapshot
            warn!("Tool {} failed, rolling back to snapshot", tool_name);
            if let Some(snapshot_mgr) = &self.snapshot_manager {
                if let Some(id) = snapshot_id {
                    snapshot_mgr.rollback(id).await?;
                    info!("Rolled back to snapshot {}", id);
                }
            }
        }
        // ... rest of existing code
    }
    Err(e) => {
        // Also rollback on error
        if let (Some(snapshot_mgr), Some(id)) = (&self.snapshot_manager, snapshot_id) {
            warn!("Tool {} errored, rolling back", tool_name);
            snapshot_mgr.rollback(id).await?;
        }
        // ... rest of existing code
    }
}
```

**Validation**:
- Create intentional failing edit
- Check snapshot created
- Verify rollback executed
- Confirm workspace state restored

### Issue 5: Cost Reporting Missing ⚠️ LOW
**Problem**: Usage recorded but no reporting

**Code Locations**:
- `src/agent/model_router.rs:record_usage()` - Recording
- `src/agent/model_router.rs:generate_report()` - Exists but not called

**Fix Required**:
```rust
// Add method to Agent:
pub async fn get_cost_report(&self) -> String {
    self.model_router.generate_report().await
}

// Call at end of session or periodically
```

**Validation**:
- Run benchmark tasks
- Call get_cost_report()
- Verify shows model distribution and costs
- Compare with all-Sonnet baseline

---

## Part 3: What to Improve (Quality Enhancements)

### Improvement 1: Better Intent Classification
**Current**: Simple keyword matching (core.rs:345-351)
**Target**: ML-based or rule-based intent classifier

**Specific Changes**:
```rust
// Current detection:
if user_message.to_lowercase().contains("find all")
    || user_message.to_lowercase().contains("search for")

// Improved detection:
async fn is_research_task(&self, message: &str) -> bool {
    // Use intelligence engine for proper classification
    match self.intelligence.classify_intent(message).await {
        Ok(UserIntent::ProjectExploration { scope }) => true,
        Ok(UserIntent::CodeReading { complexity }) => {
            // Research if exploring multiple files
            matches!(complexity, ReadingComplexity::CrossFile | ReadingComplexity::Architectural)
        }
        _ => false
    }
}
```

### Improvement 2: Agent Self-Correction Loop
**Current**: Diagnostics received but no action
**Target**: Multi-turn self-correction until clean

**Specific Changes**:
```rust
async fn self_correct_with_diagnostics(
    &self,
    file_path: &str,
    diagnostics: Vec<Diagnostic>,
    provider: &dyn LLMProvider,
    model: &str,
) -> Result<bool> {
    const MAX_CORRECTIONS: usize = 3;

    for attempt in 1..=MAX_CORRECTIONS {
        info!("Self-correction attempt {}/{}", attempt, MAX_CORRECTIONS);

        // Ask LLM to fix diagnostics
        let fix_prompt = format!(
            "The code has these issues:\n{}\n\nFix them.",
            diagnostics.iter()
                .map(|d| format!("Line {}: {}", d.line, d.message))
                .collect::<Vec<_>>()
                .join("\n")
        );

        // Execute fix
        match self.process_message(&fix_prompt, provider, model).await {
            Ok(_) => {
                // Wait for new diagnostics
                tokio::time::sleep(Duration::from_secs(2)).await;
                let new_diagnostics = self.lsp_manager.get_diagnostics(file_path).await?;

                if new_diagnostics.is_empty() {
                    info!("Self-correction successful after {} attempts", attempt);
                    return Ok(true);
                }

                // Loop continues with new diagnostics
            }
            Err(e) => {
                warn!("Self-correction attempt {} failed: {}", attempt, e);
                break;
            }
        }
    }

    warn!("Self-correction failed after {} attempts", MAX_CORRECTIONS);
    Ok(false)
}
```

### Improvement 3: Memory System Wiring
**Current**: Memory exists but not queried
**Target**: Active memory usage in every request

**Specific Changes**:
See Issue 1 fix above, plus:
```rust
// Add to process_message() at start:
info!("Checking memory systems for relevant context");

// 1. Episodic memory - similar tasks
let similar_tasks = self.intelligence.find_similar_tasks(user_message, 3).await?;
debug!("Found {} similar tasks in episodic memory", similar_tasks.len());

// 2. Knowledge graph - relevant code
let relevant_nodes = self.intelligence.find_relevant_code(user_message).await?;
debug!("Found {} relevant code locations in knowledge graph", relevant_nodes.len());

// 3. Working memory - current context
let context_items = self.context_manager.get_relevant_context(10).await?;
debug!("Working memory has {} relevant items", context_items.len());

// Add all to system_prompt before LLM call
```

### Improvement 4: Research Sub-Agent Quality Control
**Current**: Spawns sub-agents without validation
**Target**: Filter low-quality results, deduplicate findings

**Specific Changes**:
```rust
// After sub-agents complete:
let mut deduplicated_results = Vec::new();
let mut seen_findings = HashSet::new();

for result in results {
    if !result.success || result.findings.is_empty() {
        continue; // Skip failed/empty
    }

    // Simple deduplication by first 100 chars
    let fingerprint = result.findings.chars().take(100).collect::<String>();
    if seen_findings.contains(&fingerprint) {
        debug!("Skipping duplicate finding");
        continue;
    }

    seen_findings.insert(fingerprint);
    deduplicated_results.push(result);
}

info!("Deduplicated {} → {} findings", results.len(), deduplicated_results.len());
```

### Improvement 5: Model Selection Heuristics
**Current**: Simple complexity assessment
**Target**: Better task complexity estimation

**Specific Changes**:
```rust
async fn assess_task_complexity(&self, message: &str, context: &EnhancedContext) -> TaskComplexity {
    // Current: Basic heuristics
    // Improved: Consider multiple factors

    let mut complexity_score = 0;

    // Factor 1: Length and detail
    if message.len() > 200 { complexity_score += 1; }
    if message.split_whitespace().count() > 50 { complexity_score += 1; }

    // Factor 2: Multi-step indicators
    if message.contains("then") || message.contains("and") { complexity_score += 1; }

    // Factor 3: Scope
    match context.detected_intent {
        UserIntent::ProjectExploration { scope: ExplorationScope::WholeProject } => {
            complexity_score += 2;
        }
        UserIntent::CodeWriting { multi_file: true, .. } => {
            complexity_score += 2;
        }
        _ => {}
    }

    // Factor 4: Technical depth
    if message.contains("architecture") || message.contains("pattern") {
        complexity_score += 1;
    }

    match complexity_score {
        0..=2 => TaskComplexity::Low,
        3..=5 => TaskComplexity::Medium,
        _ => TaskComplexity::High,
    }
}
```

---

## Part 4: Benchmark Test Scenarios

### Scenario 1: Multi-File Refactoring
**Task**: "Rename function `authenticate_user` to `verify_user_credentials` across the entire codebase"

**Expected Behavior**:
1. **With Memory**: Knowledge graph shows all references, episodic memory recalls similar renames
2. **Model Router**: Should use Sonnet (complex refactoring)
3. **Mode**: Should be Build mode (modifications needed)
4. **Git Snapshots**: Should create snapshot before edits
5. **LSP**: Should check each edit for correctness

**Success Criteria**:
- All references updated (100%)
- No broken imports
- LSP diagnostics clean
- Fewer tool calls than baseline

### Scenario 2: Bug Investigation
**Task**: "Debug why authentication fails for users with special characters in username"

**Expected Behavior**:
1. **Intent**: Should classify as ProjectFixing (Debugger agent)
2. **Research**: Should NOT spawn sub-agents (coding task)
3. **Model Router**: Should use Sonnet (debugging needs reasoning)
4. **Memory**: Episodic should recall similar auth bugs

**Success Criteria**:
- Identifies root cause
- Proposes fix
- No sub-agent spawn
- Uses debugging agent prompt

### Scenario 3: Code Exploration
**Task**: "Find all database query patterns in the codebase and document them"

**Expected Behavior**:
1. **Intent**: Should classify as CodeReading (Explorer agent)
2. **Research**: Should spawn 5-10 sub-agents (research task)
3. **Model Router**: Should use Haiku for sub-agents (cheap)
4. **Mode**: Should be Plan mode (read-only)
5. **Memory**: Knowledge graph should provide instant results

**Success Criteria**:
- Sub-agents spawned
- Results aggregated
- Documentation generated
- 90% faster than sequential

### Scenario 4: New Feature Implementation
**Task**: "Add rate limiting middleware to the API server"

**Expected Behavior**:
1. **Intent**: Should classify as CodeWriting (Builder agent)
2. **Research**: Should NOT spawn sub-agents (coding task)
3. **Model Router**: Should use Sonnet (complex implementation)
4. **Mode**: Should be Build mode
5. **Git Snapshots**: Should create before edits
6. **LSP**: Should validate syntax

**Success Criteria**:
- Clean implementation
- No sub-agents
- Git snapshots created
- LSP diagnostics clean

---

## Part 5: Execution Plan

### Phase 1: Fix Critical Issues (Days 1-2)
1. **Memory Integration** (Issue 1)
   - Wire memory queries into process_message()
   - Add debug logging
   - Validate memory systems are queried

2. **LSP Feedback Loop** (Issue 2)
   - Implement diagnostics consumption
   - Add to agent context
   - Test with real code edits

3. **Git Rollback** (Issue 4)
   - Implement rollback on failure
   - Test with intentional errors
   - Validate workspace restoration

### Phase 2: Test Core Claims (Days 3-4)
1. **Memory Systems Test**
   - Run Scenario 1 (multi-file refactoring)
   - Measure tool calls with/without memory
   - Validate 60% reduction

2. **Sub-Agents Test**
   - Run Scenario 3 (code exploration)
   - Measure time with/without sub-agents
   - Validate 90% speedup

3. **LSP Test**
   - Run Scenario 4 with intentional errors
   - Measure self-correction rate
   - Validate diagnostics prevent runtime errors

### Phase 3: Quality Improvements (Days 5-6)
1. **Intent Classification** (Improvement 1)
2. **Self-Correction Loop** (Improvement 2)
3. **Research Quality Control** (Improvement 4)

### Phase 4: Benchmarking & Analysis (Day 7)
1. **Run All 4 Scenarios**
   - With hybrid architecture
   - Without (baseline)
   - Document metrics

2. **Generate Reports**
   - Tool call reduction
   - Cost savings
   - Time improvements
   - Error rates

3. **Create Visualizations**
   - Graphs and tables
   - Cost breakdowns
   - Performance comparisons

---

## Part 6: Success Metrics (Summary)

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Tool call reduction | 60% (7.5 → 3.0) | Count tools in logs, compare with/without memory |
| Research speedup | 90% (10x faster) | Time multi-file searches, parallel vs sequential |
| Sub-agent waste avoidance | 0% for coding | Count sub-agents for CodeWriting tasks |
| LSP error prevention | 50% fewer runtime errors | Count errors with/without LSP feedback |
| Cost reduction | 40% ($100 → $60) | Model usage reports, cost estimation |
| Operation recovery | 100% rollback success | Test failed operations, verify rollback |

---

## Part 7: Expected Timeline

- **Day 1-2**: Fix Issues 1, 2, 4 (memory, LSP, rollback)
- **Day 3-4**: Run benchmark scenarios, collect metrics
- **Day 5-6**: Implement improvements, re-test
- **Day 7**: Final analysis, generate reports

**Total**: 1 week for complete empirical validation

---

## Next Steps

1. **Immediate**: Fix Issue 1 (memory integration) - most critical
2. **Then**: Fix Issue 2 (LSP feedback) - needed for Claim 4
3. **Then**: Test Scenarios 1-4 with current implementation
4. **Then**: Implement improvements based on results
5. **Finally**: Generate research paper metrics and graphs
