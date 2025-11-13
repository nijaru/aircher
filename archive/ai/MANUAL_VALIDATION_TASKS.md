# Manual Validation Tasks

**Purpose**: Pragmatic validation of agent capabilities without benchmark infrastructure

**Approach**: 5 realistic coding tasks that test core agent functionality

## Why Manual Tasks First

**Terminal-Bench Status**: npm package not yet published (404 error)
**SWE-bench**: Requires significant setup (Python environment, datasets, evaluation harness)
**Manual Tasks**: Can run immediately, validate core functionality

## 5 Core Validation Tasks

### Task 1: File Analysis and Reporting
**Goal**: Test read operations, analysis, and structured output

**Prompt**:
```
Analyze the src/agent/core.rs file and provide a report with:
1. Main structs and their purposes
2. Public API methods
3. Dependencies on other modules
4. Any TODO comments or potential issues
```

**Success Criteria**:
- ✅ Reads file without errors
- ✅ Identifies key structures
- ✅ Provides accurate analysis
- ✅ Structured output

**Tests**: read_file tool, code analysis, output formatting

---

### Task 2: Create a New Utility Function
**Goal**: Test write operations, code generation

**Prompt**:
```
Create a new file src/utils/context_stats.rs that contains:
1. A function to format ContextWindowStats as human-readable string
2. Include usage percentage, token counts
3. Add appropriate doc comments
4. Follow existing code style
```

**Success Criteria**:
- ✅ Creates valid Rust file
- ✅ Compiles successfully
- ✅ Follows project conventions
- ✅ Proper error handling

**Tests**: write_file tool, code generation, style awareness

---

### Task 3: Bug Fix with Testing
**Goal**: Test edit operations, understanding existing code

**Prompt**:
```
Fix the test_needs_pruning test failure in src/intelligence/working_memory.rs:

The test adds 8 items × 100 tokens = 800 tokens
Window max = 1000, threshold = 80% = 800
Test expects needs_pruning() to return true
But implementation checks token_count > threshold (800 > 800 = false)

Fix: Change implementation to use >= instead of >
```

**Success Criteria**:
- ✅ Identifies correct location
- ✅ Makes minimal change
- ✅ Test passes after fix
- ✅ No other tests break

**Tests**: edit_file tool, code understanding, test awareness

---

### Task 4: Git Workflow
**Goal**: Test git operations, commit creation

**Prompt**:
```
Stage and commit the changes from Task 2 and Task 3:
1. Check git status
2. Add the modified files
3. Create commit with descriptive message
4. Verify commit was created
```

**Success Criteria**:
- ✅ Uses git commands correctly
- ✅ Creates well-formatted commit message
- ✅ Verifies success
- ✅ No extra files committed

**Tests**: bash tool, git integration, verification

---

### Task 5: Multi-File Refactoring
**Goal**: Test cross-file understanding, coordinated changes

**Prompt**:
```
Extract the context window stats formatting into the new utility:
1. Move formatting logic from working_memory.rs to context_stats.rs
2. Import and use the new utility
3. Ensure all references are updated
4. Verify tests still pass
```

**Success Criteria**:
- ✅ Identifies all affected code
- ✅ Makes coordinated changes
- ✅ Updates imports correctly
- ✅ Tests pass after refactoring

**Tests**: Multi-file operations, refactoring, verification

## How to Run

### Manual Execution (Interactive)
```bash
# Start Aircher in interactive mode
cargo run --release

# Copy/paste each task prompt
# Observe agent behavior
# Document results
```

### Expected Results

**If 5/5 pass**: Agent has strong core functionality ✅
**If 3-4/5 pass**: Agent works, needs refinement ⚠️
**If 1-2/5 pass**: Core functionality issues ❌
**If 0/5 pass**: Major integration problems ❌

## What This Validates

### ✅ Core Functionality:
- File read/write/edit operations
- Code analysis and understanding
- Git integration
- Multi-step task execution
- Error handling and recovery

### ❌ Doesn't Validate:
- Performance at scale (need benchmarks)
- Competitive positioning (need comparisons)
- Memory system advantages (need A/B testing)
- Research parallelization (need complex tasks)

## Next Steps

**After Manual Validation**:
1. If tasks pass → Document capabilities, pursue real benchmarks when available
2. If tasks fail → Fix core issues before benchmarking
3. Either way → Honest assessment in ai/VALIDATION_RESULTS.md

## Timeline

- **Task 1-2**: 15-30 minutes
- **Task 3-4**: 15-30 minutes
- **Task 5**: 20-40 minutes
- **Total**: 1-2 hours for complete validation

**Advantage**: Immediate feedback, no infrastructure setup required
