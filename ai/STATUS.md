# Aircher Status

## Current State
| Metric | Value | Updated |
|--------|-------|---------|
| Phase | Phase 5 Complete → Phase 6 Next (Terminal-Bench) | 2025-11-14 |
| Tests | 180 unit tests (100% pass) | 2025-11-13 |
| Coverage | Memory: 96%, Model Router: 100%, ACP: 100% | 2025-11-13 |
| Implementation | Phases 1-5 complete, ready for benchmarking | 2025-11-13 |

## Recent Achievements (Last Session)

**Git Repository Cleanup**:
- Purged large files from history (code_graph.json, screenshots, old Rust UI)
- Removed archive/ directory (7.8MB, all Rust code/docs)
- Git repo reduced from 43.31 MiB to 6.08 MiB packed (-85%)
- All Rust architectural concepts documented in ai/research/

**Phase 5 - ACP Protocol** (2025-11-13 COMPLETE):
- ✅ Stdio transport (JSON-RPC 2.0 over stdin/stdout)
- ✅ ACP server with 7 method handlers
- ✅ Session management (create, get, end)
- ✅ Agent prompt handling with mode support
- ✅ Tool execution via protocol
- ✅ Cost tracking integration
- ✅ CLI: `aircher serve --model gpt-4o`
- **Tests**: 14 ACP protocol tests (100% pass)

## Implementation Summary

**Phase 1-2 (Complete)**: Memory Systems
- DuckDB episodic memory, ChromaDB vector search, Knowledge Graph
- 152 tests, 96% coverage
- 60% tool call reduction validated in POC

**Phase 2-3 (Complete)**: LangGraph Agent & Sub-Agents
- 6-node workflow with conditional edges
- CodeReading, CodeWriting, ProjectFixing agents
- LLM-based tool planning and response generation

**Phase 3 (Complete)**: Dynamic Context Management
- ContextWindow with 5-factor relevance scoring
- Intelligent pruning at 80% capacity (120k/150k tokens)
- Episodic memory summarization

**Phase 4 (Complete)**: Model Routing & Cost Tracking
- Smart Model Router (OpenAI, Anthropic, Ollama)
- Task-based routing (main → medium, sub → small)
- SessionCostTracker with per-model usage
- 11 model router tests (100% pass)

**Phase 5 (Complete)**: ACP Protocol
- Custom JSON-RPC stdio transport
- Session management, tool execution, cost tracking
- 14 protocol tests (100% pass)

## Active Blockers

**None** - All phases complete, ready for Terminal-Bench evaluation

## Deferred Items (Non-Critical)

- Streaming responses (complex SSE implementation - optimize if benchmarks show need)
- Integration tests with real ACP client (manual testing works, can formalize later)
- Performance benchmarks (<100ms p95 - measure during Terminal-Bench)

## Next Phase (Phase 6: Terminal-Bench)

**Goal**: Validate >43.2% (beat Claude Code), target >58.8% (beat Factory Droid)

**Tasks**:
1. Set up Terminal-Bench harness
2. Run baseline evaluation
3. Analyze failure patterns
4. Optimize based on findings
5. Validate improvements

**Dependencies**: ✅ All satisfied (Phases 1-5 complete)

## Technical Debt

- SQLite cleanup: Unused tool_calls table in sessions storage (low priority)
- Consider Pydantic models for LLM output validation (deferred)
- Consider Pydantic Logfire for observability (nice-to-have)

## Key Learnings (Recent)

**Git History Management**:
- git-filter-repo effective for purging large files
- 85% size reduction while preserving all commits
- Archive concepts fully documented in ai/research/ before deletion

**LangGraph Integration**:
- Conditional edges critical for error handling
- Memory integration works well with decorators
- Sub-agent pattern scales effectively

**Model Routing**:
- Task-based routing (main vs sub-agent) validates 40% cost savings
- Fallback chains (large → medium → small → local) increase reliability
- Ollama integration enables unlimited local testing

**ACP Protocol**:
- Custom implementation avoids dependency conflicts (Pydantic v2)
- Stdio transport simpler than HTTP for local use case
- Cost tracking in responses valuable for user transparency

## Architecture Decisions

See ai/DECISIONS.md for full rationale:
- Python over Rust (development velocity, LangGraph ecosystem)
- Multi-database strategy (SQLite + DuckDB + ChromaDB)
- Custom ACP implementation (avoid pydantic v1 conflicts)
- LangGraph over Pydantic AI (multi-agent requirements)

## What Worked

- **Modern Python stack**: uv, ruff, ty, vulture, pytest (fast, reliable)
- **LangGraph**: Clean state-based workflow, good for complex agents
- **Memory systems**: 3-layer architecture validates 60% tool reduction
- **Research-driven development**: 15 files, 5,155 lines guides implementation
- **Comprehensive testing**: 180 tests catch issues early
- **git-filter-repo**: Effective for repository cleanup

## What Didn't Work

- agent-protocol package (Pydantic v1 conflicts) → Custom ACP implementation
- Streaming responses complexity → Deferred to post-benchmarking
- Real ACP client integration testing → Manual testing sufficient for now

## References

- **PLAN.md**: Phase dependencies, architecture, next actions
- **TODO.md**: Active tasks with phase breakdown
- **DECISIONS.md**: Architectural rationale with tradeoffs
- **RESEARCH.md**: Index to 15 research files (5,155 lines)
- **ai/research/**: Detailed SOTA analysis and implementation guidance
