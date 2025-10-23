# TODO

## Current Sprint: Week 2 - Code Understanding Tools + Memory Port

### Strategic Decision (2025-10-27)
**Frontend**: Toad (universal terminal UI) via ACP
**Backend**: Rust agent (stick with 86K lines investment)
**Memory**: Port validated POC design to Rust

**Why Toad + Rust**:
- Toad: Universal terminal UI (Python/Textual) - saves 4-6 weeks vs custom TUI
- ACP: Works in 5+ frontends (Toad, Zed, Neovim, Emacs, JetBrains)
- Rust: Performance critical for benchmarks, semantic search irreplaceable
- Memory POC: 60% improvement validated, port proven design

### ✅ POC COMPLETE - HYPOTHESIS VALIDATED! (Week 1-2)
- [x] Set up Python POC project structure ✅
- [x] Implement knowledge graph extraction ✅
  - [x] Tree-sitter Rust parsing ✅
  - [x] Extract: files, functions, classes, imports ✅
  - [x] Build NetworkX graph (3,942 nodes + 5,217 edges) ✅
  - [x] Query interface (what's in file X? what calls Y?) ✅
- [x] Implement episodic memory ✅
  - [x] SQLite database for action tracking ✅
  - [x] Record: tool calls, files touched, success/failure ✅
  - [x] Query: history, co-edit patterns ✅
- [x] Create benchmark harness ✅
  - [x] Test with/without memory ✅
  - [x] Metrics: tool calls, files examined, success rate ✅
- [x] Validate on real tasks ✅
  - [x] 4 realistic coding scenarios ✅
  - [x] **Result: 60% improvement** (exceeded 25-40% target!) ✅

### Week 2: Code Understanding Tools (Current)
- [ ] Real `search_code` - semantic search integration, query expansion
- [ ] Real `analyze_code` - AST-based analysis, complexity metrics
- [ ] Real `find_references` - cross-file symbol tracking
- [ ] Real `find_definition` - symbol lookup with context
- [ ] Target: 9/10 tools real (up from 5/10)

### Week 3: DuckDB Episodic Memory (1 week)
**Day 1-2: Schema + Basic Operations**:
- [ ] Create 5 tables (tool_executions, file_interactions, task_history, context_snapshots, learned_patterns)
- [ ] Insert/query functions for each table
- [ ] Connection pooling and error handling
- [ ] Unit tests for basic CRUD operations

**Day 3-4: Recording Layer**:
- [ ] Hook into tool execution pipeline
- [ ] Record every tool call (name, params, result, success, duration, tokens)
- [ ] Record every file interaction (read, write, edit, search, analyze)
- [ ] Track task lifecycle (start, complete, fail, pause)
- [ ] Integration tests for recording

**Day 5-7: Query Layer + Pattern Learning**:
- [ ] "Have I worked on this file before?" query
- [ ] Co-edit pattern detection (files edited together within 5 minutes)
- [ ] Similar task lookup (semantic similarity)
- [ ] Success rate by tool/file
- [ ] Performance tests (1000+ rows)

### Week 4: Knowledge Graph Port (1 week)
**Day 1-3: Graph Building**:
- [ ] Port tree-sitter extraction from Python POC
- [ ] Build petgraph structure (3,942 nodes, 5,217 edges target)
- [ ] Node types: File, Function, Class, Import, Variable
- [ ] Edge types: Contains, Calls, Imports, Uses, Inherits
- [ ] Serialize/deserialize to binary file
- [ ] Tests for graph construction

**Day 4-5: Query Interface**:
- [ ] get_file_contents(path) → all functions/classes in file
- [ ] get_callers(function) → what calls this function
- [ ] get_dependencies(file) → what files does this depend on
- [ ] find_symbol(name) → where is this defined
- [ ] Integration with existing semantic search

**Day 6-7: Incremental Updates**:
- [ ] File changed → re-parse only that file
- [ ] Update affected edges
- [ ] Repository scan on startup
- [ ] Cache parsed ASTs for performance
- [ ] Edge case handling (deleted files, renamed symbols)

### Week 3-4: ACP Protocol (can overlap with memory work)
**ACP Implementation (Rust)**:
- [ ] stdio transport (JSON-RPC over stdin/stdout)
- [ ] ACP Agent trait compliance
- [ ] Session management (create, resume, end)
- [ ] Streaming response support
- [ ] Tool execution via protocol
- [ ] Test with Zed first (best ACP support)

### Week 5: Dynamic Context Management (1 week)
**Day 1-3: Context Window Implementation**:
- [ ] ContextWindow struct (items, token_count, max_tokens)
- [ ] ContextItem struct (content, type, timestamp, relevance, dependencies)
- [ ] Relevance scoring algorithm:
  - Time decay (exponential, half-life ~1 hour)
  - Task association (2x boost for current task)
  - Dependency counting (items that reference this)
  - Item type weights (task state 2x, tool results 0.8x)
- [ ] Unit tests for relevance calculation

**Day 4-5: Pruning Logic**:
- [ ] maybe_prune() → check if 80% full, prune if needed
- [ ] prune_context() → remove bottom 30% by token count
- [ ] summarize_to_episodic() → save removed items to DuckDB
- [ ] snapshot_context() → record state for debugging
- [ ] Integration with episodic memory

**Day 6-7: Integration + Validation**:
- [ ] Connect to knowledge graph (fetch relevant code)
- [ ] Connect to episodic memory (fetch similar tasks)
- [ ] prepare_context() → build message list for LLM
- [ ] Test with POC benchmark tasks
- [ ] Validate 60% improvement holds in Rust
- [ ] Fix issues, tune relevance weights

### Week 6: Toad Integration + Intelligence Wiring
- [ ] Test Aircher agent with Toad (when Toad stabilizes)
- [ ] Wire intent classification to execution
- [ ] Activate dynamic context management in main loop
- [ ] Connect memory retrieval to tool calls
- [ ] End-to-end testing with real tasks

### Week 7-8: Benchmarks + Blog Posts
- [ ] Empirical comparison vs Claude Code
- [ ] Measure: tool calls, context efficiency, success rate
- [ ] Validate 60% memory improvement on real tasks
- [ ] Write blog post series (4-5 posts)
  - Post 1: Memory system validation
  - Post 2: Toad + ACP architecture
  - Post 3: Benchmark results
  - Post 4: Open source release

### Week 9-10: Research Paper + Release
- [ ] Academic paper draft (if pursuing publication)
- [ ] Open source memory system separately
- [ ] Contribute learnings to Aider/Continue.dev
- [ ] Final release + documentation

### Backlog
- [ ] Error guardrails (linting, auto-reject bad edits)
- [ ] Context management (last 5 interactions, collapse older)
- [ ] Intent classification operational
- [ ] Dynamic context management activation

## Daily Focus (2025-10-27)

**Completed Today**:
- ✅ Week 1 file tools complete (4/4)
- ✅ All internal docs updated
- ✅ Documentation reorganization complete (agent-contexts v0.1.1)
  - Created ai/ directory (TODO.md, STATUS.md, DECISIONS.md, RESEARCH.md)
  - Moved research findings to ai/research/
  - Eliminated internal/ directory (not needed for open-source)
  - Cleaned up docs/ structure (archived old planning directories)
  - Fixed external/agent-contexts submodule location
  - Removed deprecated pattern files (CODE_STANDARDS, etc.)
  - Updated all @internal/ references to @ai/ or @docs/

**Session Active (2025-10-27)**:
Strategy finalized: Toad frontend + Rust backend

**Current Sprint (Week 2)**:
1. Implement 4 code understanding tools (search, analyze, references, definitions)
2. Target: 9/10 tools real (up from 5/10)
3. Begin memory system port planning

**Immediate Next**:
- Start with `search_code` tool (leverage existing semantic search)
- Integrate tree-sitter for AST analysis in `analyze_code`
- Plan knowledge graph port to Rust (petgraph vs NetworkX)

## Notes
- Week 1 Success: 4 production tools (2,110+ lines, 21+ tests)
- Competitive parity: 17-21% → 23-27%
- Focus: Agent scaffolding (interfaces, guardrails, memory) not model reasoning
