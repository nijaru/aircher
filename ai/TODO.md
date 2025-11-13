# Aircher TODO

## Week 1: Memory Systems Foundation (✅ COMPLETE)

### DuckDB Episodic Memory
- [x] Create `src/aircher/memory/duckdb_memory.py` with schema from ai/research/memory-system-architecture.md
  - [x] Implement tool_executions table
  - [x] Implement file_interactions table
  - [x] Implement task_history table
  - [x] Implement context_snapshots table
  - [x] Implement learned_patterns table
- [x] Create high-level API in `src/aircher/memory/integration.py`
- [x] Hook into tool execution to auto-record operations (decorator pattern)
- [x] Implement queries: "Have I seen this?", "Co-edit patterns"
- [x] Write unit tests for all memory operations (21 tests, 100% coverage)

### ChromaDB Vector Search
- [x] Initialize sentence-transformers embedding model
- [x] Create ChromaDB collection for code snippets
- [x] Implement async codebase indexing (background task on startup)
- [x] Create query interface for semantic code search
- [x] Write comprehensive unit tests (20+ tests)
- [ ] Integrate with LangGraph agent workflow (Week 2)

### Knowledge Graph & Tree-sitter
- [x] Create `src/aircher/memory/tree_sitter_extractor.py` (Python, Rust support)
- [x] Create `src/aircher/memory/knowledge_graph.py` (NetworkX-based)
- [x] Implement build_from_file() for automatic extraction
- [x] Implement queries: get_file_contents(), get_callers(), find_symbol()
- [x] Test: Graph builds correctly, queries return accurate results (30+ tests, 100% coverage)

### Memory Integration
- [x] Create `src/aircher/memory/integration.py` unified interface
- [x] Implement @track_tool_execution decorator
- [x] Add factory function create_memory_system()
- [x] Write integration tests for multi-system queries (29 comprehensive tests)
- [ ] Validate 60% tool reduction mechanism operational (Week 2 benchmark)

**Week 1 Status**: ✅ COMPLETE
- Implementation: 5 files, ~1000 lines of production code
- Testing: 5 test files, 152 tests passing, 96% coverage on memory systems
- Test coverage: DuckDB 100%, Knowledge Graph 100%, Tree-sitter 97%, Vector Search 94%, Integration 88%
- **Detailed test results**:
  - DuckDB: 21 tests passing (100% coverage)
  - Vector Search: 30 tests passing (94% coverage)
  - Knowledge Graph: 34 tests passing (100% coverage)
  - Tree-sitter: 34 tests passing (97% coverage)
  - Integration: 33 tests passing (88% coverage)

## Week 2: LangGraph Integration & Memory Hookup (✅ COMPLETE)

### Knowledge Graph
- [x] Port tree-sitter extraction (completed in Week 1)
- [x] Build graph: files → classes → functions → variables
- [x] Create edges: contains, calls, imports, inherits, uses
- [x] Implement query interface: "What's in file X?", "What calls Y?"
- [x] Test: Graph builds on startup, queries return correct results (30+ tests)

### LangGraph Agent Workflow
- [x] Complete workflow graph definition in `src/aircher/agent/` (6-node workflow)
- [x] Integrate tools: file_ops, bash, search (5 tools loaded)
- [x] Add memory system queries (episodic, semantic, graph)
- [x] Implement intent classification node (memory-informed)
- [x] Add permission/approval nodes for WRITE mode (validate_permissions)
- [x] Implement real tool execution with memory tracking
- [x] Test: End-to-end user input → tool execution → response (30+ tests)

**Week 2 Status**: ✅ COMPLETE
- Agent workflow executes with 6 nodes
- Memory informs intent classification and tool selection
- 5 tools connected and operational
- File history and co-edit patterns working
- 30+ integration tests passing

## Week 3: Sub-Agent System & Context Management (⚠️ IN PROGRESS)

### LLM Integration (✅ COMPLETE)
- [x] Add ChatOpenAI initialization to agent (gpt-4o-mini, temperature=0.7)
- [x] Add graceful fallback if LLM initialization fails
- [x] Implement LLM-based tool planning in _generate_tool_plan()
  - [x] JSON schema-based planning prompts
  - [x] Markdown code block extraction
  - [x] Fallback to rule-based planning
- [x] Implement LLM-based response generation in _generate_response()
  - [x] Context-aware natural responses
  - [x] Memory context integration
  - [x] Fallback to template responses
- [ ] Add streaming response support (deferred to Week 5)
- [ ] Test: LLM generates appropriate tool plans and responses

### Conditional Workflow Edges (✅ COMPLETE)
- [x] Add error handling edges to workflow (dedicated handle_error node)
- [x] Add permission short-circuit (skip to response if denied)
- [x] Add conditional routing after tool selection (skip if no tools)
- [x] Add conditional routing after execution (success/error/partial)
- [x] Implement error collection and contextualization
- [ ] Add retry logic for failed tool executions (foundation in place)
- [ ] Implement early termination for simple queries
- [ ] Test: Workflow handles errors gracefully

### Sub-Agent Architecture (✅ COMPLETE)
- [x] Implement CodeReading agent (READ mode tools only)
- [x] Implement CodeWriting agent (WRITE mode tools)
- [x] Implement ProjectFixing agent (full toolset)
- [x] Create BaseSubAgent abstract class with 3-node workflow
- [x] Add tool restriction per agent type (tool filtering per specialization)
- [x] Implement session hierarchy (parent/child tracking via parent_session_id)
- [x] Add cost optimization (gpt-4o-mini default for sub-agents)
- [x] Implement spawn_sub_agent() factory method in main agent
- [x] Add memory tracking for sub-agent executions
- [ ] Implement ResearchAgent (external search/docs) - deferred to Week 4
- [ ] Test: Sub-agents spawn correctly, tool restrictions work

### Dynamic Context Management (✅ COMPLETE)
- [x] Implement relevance scoring algorithm:
  - [x] Time decay factor (exponential, half-life 60 minutes)
  - [x] Task association boost (2x for current task)
  - [x] Dependency boost (0.2 per dependent item)
  - [x] Type multiplier (system=100, task=2.0, user=1.5, assistant=1.2, tool=0.8, code=0.7, kg=0.6)
- [x] Implement intelligent pruning (remove bottom 30% at 80% capacity)
- [x] Save context snapshots to episodic memory before pruning
- [x] Integrate ContextWindow with agent workflow (system prompt, user/assistant messages)
- [ ] Implement prefetch from knowledge graph (deferred to Week 4)
- [ ] Test: Continuous multi-turn conversations without restarts (deferred to Week 4)

**Week 3 Success**: Sub-agents working, context pruning foundation complete

## Week 4: Model Routing & Cost Optimization (IN PROGRESS)

### Smart Model Router (✅ COMPLETE)
- [x] Implement model selection logic:
  - [x] Large/expensive: Opus-4, GPT-4 for complex reasoning
  - [x] Medium: Sonnet-4, GPT-4o for main agent
  - [x] Small/cheap: Haiku-4, GPT-4o-mini for sub-agents
  - [x] Local: Ollama for unlimited usage (zero cost)
- [x] Add cost tracking per session (SessionCostTracker)
- [x] Implement automatic fallback on rate limits (fallback chain by tier)
- [x] Add task-based model selection (select_model_for_task)
- [ ] Add user-configurable model preferences (deferred - use defaults)
- [ ] Test: Validate 40% cost savings vs always-large-model (needs real workload)

### LLM Provider Integration (✅ COMPLETE)
- [x] Complete OpenAI integration (GPT-4, GPT-4o, GPT-4o-mini)
- [x] Complete Anthropic integration (Opus-4, Sonnet-4, Haiku-4)
- [x] Add Ollama integration for local models
- [x] Add error handling and automatic fallback
- [x] Test: All providers working, graceful fallbacks (11 tests passing)
- [ ] Implement streaming response support (deferred to Week 5)

**Week 4 Progress**: Model routing operational (70% complete), cost tracking working

## Week 5: ACP Protocol & Testing

### ACP Protocol
- [ ] Implement stdio transport (JSON-RPC over stdin/stdout)
- [ ] Add session management (create, resume, end)
- [ ] Support tool execution via protocol
- [ ] Implement streaming responses
- [ ] Test: Can launch from Zed or other ACP clients

### Comprehensive Testing
- [ ] Unit tests for memory operations (target >80% coverage)
- [ ] Integration tests for agent workflow
- [ ] Tool execution tests with approval mocking
- [ ] Memory persistence tests
- [ ] Performance benchmarks (<100ms p95)
- [ ] Test: All tests passing, performance targets met

**Week 5 Success**: ACP compliant, test coverage >80%, performance validated

## Week 6: Benchmarking & Optimization

### Terminal-Bench Integration
- [ ] Set up Terminal-Bench evaluation harness
- [ ] Run baseline evaluation (expect 35-45%)
- [ ] Analyze failure patterns
- [ ] Optimize based on findings
- [ ] Re-run to validate improvements

### SWE-bench Sample
- [ ] Run SWE-bench Verified sample (50 tasks)
- [ ] Analyze performance vs SOTA (75%)
- [ ] Document findings

### Optimization & Polish
- [ ] Address benchmark failures
- [ ] Tune prompts and workflows
- [ ] Optimize memory queries for performance
- [ ] Validate 60% tool reduction claim
- [ ] Final testing and bug fixes

**Week 6 Success**: Terminal-Bench >43.2% (beat Claude Code), tool reduction validated

## Backlog (Post Week 6)

### Toad Integration
- [ ] Wait for Toad open source release
- [ ] Test Aircher with Toad frontend
- [ ] Optimize for Toad-specific features
- [ ] Update documentation

### Advanced Features
- [ ] Undo subsystem (git-based time travel)
- [ ] Smart context compaction (beyond pruning)
- [ ] Cross-session pattern learning improvements
- [ ] GUI automation capabilities (if needed)
- [ ] MCP server support (extensibility)

### Documentation & Release
- [ ] User documentation (docs/)
- [ ] API documentation (docstrings + mkdocs)
- [ ] Tutorial videos/examples
- [ ] Contribution guidelines
- [ ] Prepare for 0.1.0 release

## Notes

- Focus on Week 1-2 (memory systems) first - biggest competitive advantage
- Don't skip testing - comprehensive tests prevent regression
- Benchmark early (Week 6) to validate SOTA claims with data
- Keep ai/STATUS.md updated with progress

## References

- **Implementation Plan**: ai/PLAN.md (6-week roadmap)
- **Memory Architecture**: ai/research/memory-system-architecture.md
- **Sub-Agent Patterns**: ai/research/crush-subagent-architecture.md
- **Competitive Analysis**: ai/research/competitive-analysis-2025.md
