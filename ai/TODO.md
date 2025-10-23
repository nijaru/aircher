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

### Week 3-4: ACP Protocol + Memory Port
**ACP Implementation (Rust)**:
- [ ] stdio transport (JSON-RPC over stdin/stdout)
- [ ] ACP Agent trait compliance
- [ ] Session management
- [ ] Streaming response support
- [ ] Test with Zed first (best ACP support)

**Memory System Port (3-4 weeks)**:
- [ ] Port knowledge graph builder to Rust (tree-sitter + petgraph)
- [ ] Port episodic memory to DuckDB
- [ ] Integrate with existing tool execution
- [ ] Repository auto-scanning on startup

### Week 5-6: Toad Integration + Intelligence Wiring
- [ ] Test Aircher agent with Toad (when Toad stabilizes)
- [ ] Wire intent classification to execution
- [ ] Activate dynamic context management
- [ ] Connect memory retrieval to tool calls

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
