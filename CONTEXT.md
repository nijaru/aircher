# Aircher Development Context

**Quick Reference for Continuing Development**

## Current State (2025-11-13)

**Branch**: `claude/continue-previous-work-011CV5mymx5ezeBegfA7mrck`
**Phase**: Week 3 (LLM Integration) - STARTED
**Last Commit**: `8a143f5` - "wip: start Week 3 - add LLM initialization to agent"

### ✅ What's Complete

#### Week 1: 3-Layer Memory Systems (COMPLETE)
- **DuckDB Episodic Memory**: Tool executions, file interactions, task history, context snapshots, learned patterns
- **ChromaDB Vector Search**: Semantic code search with sentence-transformers embeddings
- **Knowledge Graph**: NetworkX-based graph with tree-sitter extraction (Python, Rust support)
- **Memory Integration**: Unified interface with `@track_tool_execution` decorator
- **Testing**: 89 tests passing, >95% coverage (DuckDB 100%, KG 100%, tree-sitter 97%)

#### Week 2: Agent-Memory Integration (COMPLETE)
- **LangGraph Workflow**: 6-node workflow (classify_intent → validate_permissions → select_tools → execute_task → generate_response → update_memory)
- **5 Tools Loaded**: ReadFile, WriteFile, ListDirectory, SearchFiles, Bash
- **Real Tool Execution**: Tools execute with memory tracking via decorator
- **Memory-Informed Decisions**:
  - Intent classification queries tool statistics (recent usage patterns)
  - Tool selection queries file history and co-edit patterns
  - Suggests related files based on co-edit patterns
- **Integration Tests**: 30+ tests covering all workflow nodes

#### Week 3: LLM Integration (STARTED)
- **ChatOpenAI Initialization**: Integrated with graceful fallback (temperature=0.7, gpt-4o-mini)

### ⚠️ What's Pending (Week 3+)

1. **LLM-based Tool Planning** - Replace rule-based `_generate_tool_plan()` (line 221)
2. **LLM-based Response Generation** - Replace template-based `_generate_response()` (line 255)
3. **Conditional Workflow Edges** - Error handling, permission short-circuits, retry logic
4. **Sub-Agent Architecture** - CodeReading, CodeWriting, ProjectFixing, Research agents
5. **Dynamic Context Pruning** - Relevance scoring + intelligent removal at 80% capacity
6. **Model Routing** - Small vs large model selection for cost optimization (40% savings target)
7. **End-to-End Integration Tests** - Full workflow validation

## Key Files

### Agent Core
- **`src/aircher/agent/__init__.py`** (523 lines) - Main LangGraph agent with 6-node workflow
  - Line 69-78: LLM initialization (RECENTLY ADDED)
  - Line 221-229: `_generate_tool_plan()` (NEEDS LLM INTEGRATION)
  - Line 255-268: `_generate_response()` (NEEDS LLM INTEGRATION)
  - Line 124-159: `_classify_intent()` - Memory-informed intent classification
  - Line 224-278: `_select_tools()` - Memory-informed tool selection
  - Line 314-386: `_execute_task()` - Real tool execution with tracking
  - Line 418-483: `_update_memory()` - Records to episodic memory

### Memory Systems
- **`src/aircher/memory/integration.py`** - Unified memory interface
- **`src/aircher/memory/duckdb_memory.py`** - Episodic memory (DuckDB)
- **`src/aircher/memory/vector_search.py`** - Semantic search (ChromaDB)
- **`src/aircher/memory/knowledge_graph.py`** - Code graph (NetworkX)
- **`src/aircher/memory/tree_sitter_extractor.py`** - Code parsing (tree-sitter)

### Tools
- **`src/aircher/tools/base.py`** - BaseTool, ToolInput, ToolOutput classes
- **`src/aircher/tools/file_ops.py`** - ReadFile, WriteFile, ListDirectory, SearchFiles
- **`src/aircher/tools/bash.py`** - Bash command execution

### Tests
- **`tests/unit/test_duckdb_memory.py`** (421 lines) - 21 tests, 100% coverage
- **`tests/unit/test_knowledge_graph.py`** - 30+ tests, 100% coverage
- **`tests/unit/test_tree_sitter_extractor.py`** - 25+ tests, 97% coverage
- **`tests/integration/test_agent_memory.py`** (352 lines) - 30+ integration tests

## Architecture Patterns

### Memory-Augmented Agent Flow
```
User Request → Classify Intent (query tool stats) → Validate Permissions
  → Select Tools (query file history + co-edit patterns) → Execute Task (with memory tracking)
  → Generate Response → Update Memory (record to DuckDB)
```

### Tool Execution with Memory Tracking
```python
# Memory decorator wraps tool execution
if self.memory:
    tool_func = self.memory.track_tool_execution(tool.execute)
else:
    tool_func = tool.execute

result = await tool_func(**parameters)
```

### File History & Co-Edit Patterns
```python
# Get file history from episodic memory
history = self.memory.query_file_history(file_path, limit=3)

# Find co-edit patterns (files frequently edited together)
co_edit_patterns = self.memory.find_co_edit_patterns(min_count=2)

# Suggest related files based on patterns
if pattern["file1"] in files_mentioned:
    related_files.add(pattern["file2"])
```

## Research Context

**Reference**: `ai/research/` directory (15 files, 5,155 lines)

### Key Research Files
- **`crush-subagent-architecture.md`** (425 lines) - Sub-agent patterns from Charm's Crush
  - Small models for sub-agents
  - Tool restriction per agent type
  - Parent/child session hierarchy
  - Cost optimization patterns

- **`memory-system-architecture.md`** - Context pruning strategies
  - Relevance scoring algorithm (time decay, task association, dependency boost)
  - Intelligent removal at 80% capacity
  - Context snapshot before pruning

- **`competitive-analysis-2025.md`** - SOTA agent comparison
  - Terminal-Bench: Claude Code 43.2%, Cursor 58.8%
  - Target: >43.2% (beat Claude Code), stretch >58.8%

## Known Issues & Limitations

1. **Network Limitation**: HuggingFace model downloads blocked (403 errors)
   - Impact: Vector search embedding model can't download
   - Mitigation: Tests run in isolation, core functionality unaffected

2. **Tool Planning is Rule-Based**: Currently uses simple keyword matching
   - Fix: Replace with LLM-based planning (Week 3 priority)

3. **Response Generation is Template-Based**: Simple string formatting
   - Fix: Replace with LLM-based generation (Week 3 priority)

## Quick Commands

```bash
# Setup
uv sync --dev

# Run tests
uv run pytest tests/ -v
uv run pytest tests/unit/test_duckdb_memory.py -v  # Single file
uv run pytest -k "test_agent_init" -v              # Single test

# Code quality
uv run ruff check . --fix
uv run mypy src/ --strict

# Git
git status
git log --oneline -5
git diff main...HEAD
```

## Next Steps (Week 3 Priority Order)

### Phase 1: LLM Integration (Immediate)
1. **Implement LLM-based tool planning** in `_generate_tool_plan()` (line 221)
   - Research SOTA prompt patterns from Cursor, Continue, Aider
   - Generate tool execution plans with reasoning
   - Include memory context in prompts

2. **Implement LLM-based response generation** in `_generate_response()` (line 255)
   - Use tool results to generate natural responses
   - Include context from memory
   - Format code blocks and explanations

### Phase 2: Robustness (Production-Ready)
3. **Add conditional workflow edges**
   - Error handling with retry logic
   - Permission short-circuits (skip execution if denied)
   - Early termination for simple queries

### Phase 3: Scalability (SOTA Features)
4. **Build sub-agent system** (CodeReading, CodeWriting agents)
   - Research patterns beyond crush-subagent-architecture.md
   - Implement agent routing and orchestration
   - Add tool restriction per agent type

5. **Implement dynamic context pruning**
   - Relevance scoring with time decay
   - Intelligent removal at 80% token capacity
   - Prefetch from knowledge graph

## Research Questions for Web Access

1. **LLM Tool Planning**: How do Cursor, Continue, Aider use LLMs for tool planning? What prompt patterns work best in 2025?

2. **Conditional LangGraph Edges**: What are production best practices for error handling and short-circuits in LangGraph workflows?

3. **Dynamic Context Management**: How do modern agents implement context pruning? What relevance scoring algorithms are used?

4. **Sub-Agent Orchestration**: Beyond Crush patterns, what are current best practices for:
   - Request routing to specialized agents
   - Session hierarchy (parent/child)
   - Tool restriction and cost optimization
   - Parallel execution coordination

5. **LangGraph 2025 Patterns**: What are the recommended patterns for production LangGraph agents today?

## Documentation Index

- **`ai/STATUS.md`** - Current state, phase tracking, blockers
- **`ai/TODO.md`** - Week-by-week checkboxes (Weeks 1-6)
- **`ai/PLAN.md`** - 6-week implementation roadmap
- **`ai/DECISIONS.md`** - Architectural rationale
- **`ai/RESEARCH.md`** - Research index ("for X, read Y")
- **`ai/research/`** - 15 SOTA research files
- **`ai/design/`** - Lightweight API specs
- **`CLAUDE.md`** - Project overview and workflow rules

## Commits

```
8a143f5 - wip: start Week 3 - add LLM initialization to agent
a92e36c - feat: integrate memory systems into agent workflow (Week 2)
2029e10 - test: add comprehensive unit tests for Week 1 memory systems
bd11151 - feat: implement Week 1 memory systems (DuckDB, ChromaDB, Knowledge Graph)
```

---

**For New Conversations**: Use this file as quick context. Read `ai/STATUS.md` for detailed state, `ai/TODO.md` for task checkboxes, and relevant `ai/research/` files for implementation guidance.
