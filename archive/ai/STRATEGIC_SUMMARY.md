# Strategic Summary: Toad + Rust Decision

**Decision Date**: 2025-10-27
**Status**: Week 2 of 10-week plan

## TL;DR

✅ **Keep Rust backend** (86K lines investment)
✅ **Use Toad as primary frontend** (saves 4-6 weeks)
✅ **Memory POC validated** (60% improvement proven)
✅ **On track for 10-week delivery**

## The Question We Answered

> "How long to improve agent and build TUI? Is Rust best? What about Python for speed? What is OpenCode using?"

## The Answer

### Frontend: Toad (Don't Build, Integrate)

**What is Toad?**
- Universal terminal UI for agentic coding
- Built by Will McGugan (creator of Rich/Textual)
- Python/Textual framework
- ACP support announced July 2025
- URL: https://willmcgugan.github.io/announcing-toad/

**Why it saves us 4-6 weeks:**
- Someone else builds/maintains the TUI
- We implement ACP protocol (2-3 weeks)
- Works in 5+ frontends (Toad, Zed, Neovim, Emacs, JetBrains)
- vs building custom Ratatui TUI (4-6 weeks for just 1 frontend)

**Architecture:**
```
Toad (Python)  →  ACP (JSON-RPC stdio)  →  Aircher (Rust)
   ↑                      ↑                      ↑
  UI layer         Language-agnostic       Agent intelligence
```

### Backend: Rust (Keep 86K Lines)

**Why Rust wins:**

1. **Semantic search irreplaceable**
   - hnswlib-rs: 45x faster than Python alternatives
   - 6,468 vectors, <2ms search latency
   - Rebuilding in Python = slower benchmarks

2. **Performance critical for research**
   - Benchmarking vs Claude Code (Week 7-8)
   - True parallelism (no GIL) for multi-tool execution
   - Single binary deployment (easy to reproduce results)

3. **Research says: Rust > Python for agents**
   - Quote: "With 500 agents on 64-core machine, Rust scales while Python doesn't"
   - GIL is a real bottleneck for concurrent tool execution

4. **Already have production tools**
   - 5 tools: 2,110+ lines tested
   - Multi-provider auth working
   - Tree-sitter parsing for 19+ languages

**What about Python?**
- ✅ Perfect for POC (validated memory approach in 1-2 weeks)
- ❌ Wrong for production (slower, GIL limits, harder deployment)

### Memory System: Port POC to Rust

**POC Results (Python):**
- 60% reduction in tool calls (7.5 → 3.0)
- 100% elimination of irrelevant files
- Knowledge graph: 3,942 nodes, 5,217 edges
- Episodic memory: SQLite tracking

**Port Plan (Week 3-4):**
- Use petgraph (Rust) instead of NetworkX (Python)
- Keep DuckDB for episodic memory (already have infrastructure)
- Validate 60% improvement holds in Rust
- Budget: 1.5 weeks (can extend to 2 weeks)

## What OpenCode/Others Are Doing

### OpenCode
- **Stack**: TypeScript (51.7%) + Go (34.7%)
- **Strategy**: Split responsibilities (TS for UI, Go for performance)
- **Our parallel**: Toad (Python) for UI, Rust for performance

### Factory Droid
- **Performance**: #1 on Terminal-bench (58.8%)
- **Status**: Closed source, commercial
- **Our advantage**: Open source + memory system

### Goose (Square/Block)
- **Status**: Open source, ACP-compatible
- **Stack**: Python
- **Our advantage**: Rust performance + memory system

## Competitive Position After 10 Weeks

| Feature | Aircher | Claude Code | Factory Droid | Goose |
|---------|---------|-------------|---------------|-------|
| **Memory system** | ✅ 60% improvement | ❌ None | ❌ None | ❌ None |
| **Semantic search** | ✅ <2ms latency | ⚠️ Unknown | ⚠️ Unknown | ⚠️ Unknown |
| **ACP compatible** | ✅ 5+ frontends | ✅ Yes | ❌ No | ✅ Yes |
| **Open source** | ✅ Yes | ❌ No | ❌ No | ✅ Yes |
| **Performance** | ✅ Rust | ⚠️ Unknown | ✅ Top scores | ⚠️ Python |
| **Terminal focus** | ✅ Toad primary | ❌ Editor-focused | ⚠️ Unknown | ✅ Yes |

**Our unique value**: Only open-source ACP agent with validated memory system and Rust performance.

## Time Comparison

### Option 1: Toad + Rust (CHOSEN)
```
Week 2: Code tools (4 tools)              → 1 week
Week 3-4: ACP + memory port               → 2 weeks
Week 5-6: Toad integration + intelligence → 2 weeks
Week 7-8: Benchmarks + blog posts         → 2 weeks
Week 9-10: Paper + release                → 2 weeks
────────────────────────────────────────────────────
Total: 10 weeks, works in 5+ frontends
```

### Option 2: Python Rewrite + Custom TUI (REJECTED)
```
Week 1-4: Rebuild semantic search        → 4 weeks
Week 5-8: Port tools (2,110+ lines)      → 4 weeks
Week 9-10: Build Textual TUI             → 2 weeks
Week 11-12: ACP protocol                 → 2 weeks
Week 13-14: Intelligence                 → 2 weeks
Week 15-16: Benchmarks + paper           → 2 weeks
────────────────────────────────────────────────────
Total: 16 weeks, works in 1 frontend, slower benchmarks
```

**Verdict**: Toad + Rust saves 6 weeks and gives better results.

## Current Inventory (Week 2 Start)

### ✅ What We Have

**Infrastructure (86K lines Rust):**
- Semantic search: 6,468 vectors, 19+ languages, <2ms
- Multi-provider auth: OpenAI, Anthropic, Gemini, Ollama
- Tree-sitter parsing: 19+ languages
- Tool framework: End-to-end execution

**Production Tools (2,110+ lines, 21+ tests):**
1. analyze_errors (378 lines)
2. read_file (430 lines)
3. write_file (450 lines)
4. edit_file (530 lines)
5. list_files (700 lines)

**Memory POC (validated):**
- Knowledge graph: 3,942 nodes, 5,217 edges
- Episodic memory: SQLite tracking
- 60% improvement proven

### ❌ What We Need

**Week 2 (Code Understanding Tools):**
- search_code (integrate semantic search)
- analyze_code (AST with tree-sitter)
- find_references (symbol tracking)
- find_definition (lookup with context)

**Week 3-4 (ACP + Memory):**
- ACP protocol (stdio, JSON-RPC)
- Memory port to Rust (petgraph + DuckDB)

**Week 5-10 (Integration + Validation):**
- Toad integration (when stable)
- Intelligence wiring (intent, context)
- Benchmarks vs Claude Code
- Research paper + release

## Risk Analysis

### Low Risk ✅
- **Rust performance**: Proven advantage, research-backed
- **Semantic search**: Already production-ready
- **Memory POC**: Validated 60% improvement
- **Tool framework**: 5 tools working, pattern established

### Medium Risk ⚠️
- **Memory port complexity**: Python → Rust may be harder
  - Mitigation: Keep design simple, budget 1.5-2 weeks
- **ACP protocol stability**: Still evolving
  - Mitigation: Track upstream, implement core spec

### Managed Risk 🔄
- **Toad timeline**: May not stabilize by Week 5-6
  - Mitigation: Use Zed + Neovim as proven frontends
  - Toad is bonus, not blocker
- **Benchmark reproducibility**: Hard to match exact conditions
  - Mitigation: Use public datasets (SWE-bench, Terminal-bench)

## Next Steps (Immediate)

**Today:**
1. ✅ Update all docs (ai/, docs/, CLAUDE.md)
2. ✅ Create comprehensive plan (ai/PLAN.md)
3. Start Week 2: search_code tool

**This Week (Week 2):**
- Day 1-3: search_code (integrate semantic search)
- Day 3-5: analyze_code (tree-sitter AST)
- Day 5-6: find_references (symbol tracking)
- Day 6-7: find_definition (lookup)

**Success Criteria:**
- 9/10 tools real by end of week
- Competitive parity: 23-27% → 30-35%
- All tests passing

## Why This Works

1. ✅ **Leverage existing work** - 86K lines Rust stays
2. ✅ **Save time** - Toad handles UI (4-6 weeks saved)
3. ✅ **Proven design** - Memory POC validated (60% improvement)
4. ✅ **Clear milestones** - Week-by-week deliverables
5. ✅ **Research contribution** - Empirical validation + publication
6. ✅ **Multi-frontend** - Works in 5+ editors via ACP

## Bottom Line

**Question**: "Should we use Python for speed and build a TUI?"

**Answer**:
- ❌ No Python rewrite (lose 6 weeks + performance)
- ❌ No custom TUI (lose 4-6 weeks)
- ✅ Yes Toad frontend (universal terminal UI)
- ✅ Yes Rust backend (performance + existing work)

**Timeline**: 10 weeks to research paper + release (vs 16 weeks for Python rewrite)

**Outcome**: Open-source ACP agent with validated memory system, works in 5+ frontends, empirically proven improvements.

---

**Status**: Week 2 of 10 in progress
**See**: ai/PLAN.md for detailed execution plan
**See**: ai/TODO.md for current sprint tasks
