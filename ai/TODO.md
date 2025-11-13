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
- Testing: 5 test files, 89 tests passing, >95% coverage on core systems
- Test coverage: DuckDB 100%, Knowledge Graph 100%, Tree-sitter 97%

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

### LLM Integration (STARTED)
- [x] Add ChatOpenAI initialization to agent (gpt-4o-mini, temperature=0.7)
- [x] Add graceful fallback if LLM initialization fails
- [ ] Implement LLM-based tool planning in _generate_tool_plan()
- [ ] Implement LLM-based response generation in _generate_response()
- [ ] Add streaming response support
- [ ] Test: LLM generates appropriate tool plans and responses

### Conditional Workflow Edges
- [ ] Add error handling edges to workflow
- [ ] Add permission short-circuit (skip to response if denied)
- [ ] Add retry logic for failed tool executions
- [ ] Implement early termination for simple queries
- [ ] Test: Workflow handles errors gracefully

### Sub-Agent Architecture
- [ ] Implement CodeReading agent (READ mode tools only)
- [ ] Implement CodeWriting agent (WRITE mode tools)
- [ ] Implement ProjectFixing agent (full toolset)
- [ ] Implement ResearchAgent (external search/docs)
- [ ] Add tool restriction per agent type (Crush patterns)
- [ ] Implement session hierarchy (parent/child tracking)
- [ ] Add cost optimization (small models for sub-agents)
- [ ] Test: Sub-agents spawn correctly, tool restrictions work

### Dynamic Context Management
- [ ] Implement relevance scoring algorithm:
  - [ ] Time decay factor
  - [ ] Task association boost
  - [ ] Dependency boost
  - [ ] Type multiplier
- [ ] Implement intelligent pruning (remove bottom 30% at 80% capacity)
- [ ] Save context snapshots to episodic memory before pruning
- [ ] Implement prefetch from knowledge graph
- [ ] Test: Continuous multi-turn conversations without restarts

**Week 3 Success**: Sub-agents working, context pruning prevents overflow

## Week 4: Model Routing & Cost Optimization

### Smart Model Router
- [ ] Implement model selection logic:
  - [ ] Large/expensive: Opus, GPT-4 for main agent
  - [ ] Small/cheap: Haiku, GPT-4o-mini for sub-agents
  - [ ] Local: Ollama for unlimited usage
- [ ] Add cost tracking per session
- [ ] Implement automatic fallback on rate limits
- [ ] Add user-configurable model preferences
- [ ] Test: Validate 40% cost savings vs always-large-model

### LLM Provider Integration
- [ ] Complete OpenAI integration (GPT-4, GPT-4o-mini)
- [ ] Complete Anthropic integration (Opus, Sonnet, Haiku)
- [ ] Add Ollama integration for local models
- [ ] Implement streaming response support
- [ ] Add error handling and retries
- [ ] Test: All providers working, graceful fallbacks

**Week 4 Success**: Model routing operational, cost savings validated

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
