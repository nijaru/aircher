# Comprehensive Plan: Toad + Rust Strategy

**Last Updated**: 2025-10-27
**Decision**: Use Toad as primary frontend, stick with Rust backend
**Timeline**: 10 weeks to research paper + release

## Strategic Architecture

```
┌─────────────────────────────────────────────────┐
│         Toad (Universal Terminal UI)             │
│         Python/Textual by Will McGugan           │
│         (Someone else builds/maintains)          │
└─────────────────────────────────────────────────┘
                     ↓
        ┌────────────────────────┐
        │  ACP Protocol (stdio)  │
        │    JSON-RPC messages   │
        │  Language-agnostic!    │
        └────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────┐
│         Aircher Agent (Rust)                     │
│         (What we build)                          │
├─────────────────────────────────────────────────┤
│ Semantic Search: 6,468 vectors, <2ms latency   │
│ 5 Production Tools: 2,110+ lines tested        │
│ Multi-Provider: OpenAI, Anthropic, Gemini      │
│ Memory System: 60% improvement (POC)           │
│ Knowledge Graph: 3,942 nodes, 5,217 edges      │
└─────────────────────────────────────────────────┘
```

## Current Inventory (What We Have)

### ✅ Production-Ready Infrastructure (86K lines Rust)

**Semantic Search** (irreplaceable advantage):
- hnswlib-rs backend (45x faster than alternatives)
- 6,468 vectors indexed across Aircher codebase
- 19+ language support via tree-sitter
- <2ms search latency
- Location: `src/intelligence/semantic_search/`

**Multi-Provider Auth** (working):
- OpenAI: GPT-4, GPT-4o, GPT-4o-mini
- Anthropic: Claude Opus 4.1, Sonnet 4, Haiku
- Google: Gemini 1.5 Pro, Flash
- Ollama: Local models
- Location: `src/providers/`

**Tree-sitter Parsing** (19+ languages):
- Rust, C, C++, Go, Zig
- JavaScript, TypeScript, Python, Ruby, PHP
- Java, C#, Kotlin, Swift
- Bash, SQL, TOML, JSON, YAML, Markdown
- Location: `src/intelligence/code_analysis/`

**Tool Framework** (proven):
- End-to-end execution without crashes
- Approval workflows (for dangerous operations)
- Streaming support
- Location: `src/agent/tools/`

### ✅ 5 Production Tools (2,110+ lines, 21+ tests)

1. **analyze_errors** (378 lines, 4 tests)
   - Error pattern matching
   - Actionable fix suggestions
   - Cross-file context
   - Location: `src/agent/tools/analyze_errors.rs`

2. **read_file** (430 lines, 4 tests)
   - Syntax highlighting with tree-sitter
   - AST context extraction
   - Smart truncation
   - Location: `src/agent/tools/read_file.rs`

3. **write_file** (450 lines, 6 tests)
   - Atomic writes (temp file + rename)
   - Automatic backups
   - Protected file detection
   - Location: `src/agent/tools/write_file.rs`

4. **edit_file** (530 lines, 7 tests)
   - Dual modes: search/replace + line-based
   - Change validation
   - Diff preview
   - Location: `src/agent/tools/edit_file.rs`

5. **list_files** (700 lines, 8 tests)
   - Recursive traversal
   - Gitignore respect
   - Metadata (size, modified)
   - Smart filtering
   - Location: `src/agent/tools/list_files.rs`

### ✅ Memory POC (60% improvement validated)

**Python POC** (proven design):
- Knowledge graph: 3,942 nodes, 5,217 edges
- Episodic memory: SQLite tracking
- 60% reduction in tool calls (7.5 → 3.0)
- 100% elimination of irrelevant files
- Location: `poc-memory-agent/`

**Components to port**:
- Graph extraction: tree-sitter → NetworkX
- Memory layer: SQLite episodic tracking
- Query interface: "What's in file X? What calls Y?"
- Pattern learning: Co-edit detection

### ❌ 5 Stub Tools (Week 2 target)

6. **search_code** (STUB)
   - Should: Leverage semantic search + query expansion
   - Currently: Returns hardcoded JSON
   - Week 2 priority #1

7. **analyze_code** (STUB)
   - Should: AST analysis, complexity metrics, pattern detection
   - Currently: Returns hardcoded JSON
   - Week 2 priority #2

8. **find_references** (STUB)
   - Should: Cross-file symbol tracking with tree-sitter
   - Currently: Returns hardcoded JSON
   - Week 2 priority #3

9. **find_definition** (STUB)
   - Should: Symbol lookup with context
   - Currently: Returns hardcoded JSON
   - Week 2 priority #4

10. **run_command** (STUB)
    - Should: Shell execution with safety checks
    - Currently: Returns hardcoded JSON
    - Week 3-4 (lower priority)

## 10-Week Execution Plan

### Week 2: Code Understanding Tools (Current)
**Goal**: 9/10 tools real (up from 5/10)

**Task 1: search_code** (2-3 days)
- Integrate existing semantic search
- Add query expansion (synonyms, related terms)
- Implement result ranking
- Max 50 results (research: LM-centric interfaces)
- Tests: 4+ covering edge cases

**Task 2: analyze_code** (2-3 days)
- AST extraction with tree-sitter
- Complexity metrics (cyclomatic, nesting depth)
- Pattern detection (antipatterns, best practices)
- Tests: 4+ covering different languages

**Task 3: find_references** (1-2 days)
- Symbol tracking with tree-sitter
- Cross-file search
- Import/export analysis
- Tests: 3+ covering scopes

**Task 4: find_definition** (1-2 days)
- Symbol lookup with tree-sitter
- Context extraction (surrounding code)
- Multi-definition handling (overloads)
- Tests: 3+ covering edge cases

**Success Criteria**:
- 9/10 tools real and tested
- All pass existing test suite
- Competitive parity: 23-27% → 30-35%

### Week 3-4: ACP Protocol + Memory Port
**Goal**: Agent works via ACP, memory system in Rust

**ACP Implementation** (1.5 weeks):
- stdio transport (JSON-RPC over stdin/stdout)
- ACP Agent trait full compliance
- Session management (create, resume, end)
- Streaming response support
- Tool execution via protocol
- Tests: 10+ covering protocol edge cases
- Reference: https://agentclientprotocol.com/

**Memory System Port** (1.5 weeks):
- Knowledge graph: tree-sitter + petgraph (vs NetworkX)
- Episodic memory: DuckDB (already have infrastructure)
- Repository auto-scanning on startup
- Query interface: Rust API
- Tests: 8+ covering graph operations
- Validate 60% improvement holds in Rust

**Success Criteria**:
- Aircher launches from Zed via ACP
- Memory system operational in Rust
- Benchmarks show 60% improvement

### Week 5-6: Toad Integration + Intelligence Wiring
**Goal**: Works in Toad, intelligence active

**Toad Integration** (1 week):
- Test with Toad (when ACP support stabilizes)
- Fix any protocol compatibility issues
- Performance tuning for terminal UX
- Documentation for Toad users

**Intelligence Wiring** (1 week):
- Connect intent classification to execution
- Activate dynamic context management
- Wire memory retrieval to tool calls
- Integrate knowledge graph queries

**Success Criteria**:
- Smooth experience in Toad + Zed
- Intent classification >80% accuracy
- Memory reduces tool calls by 60%

### Week 7-8: Benchmarks + Blog Posts
**Goal**: Empirical validation vs Claude Code

**Benchmark Tasks** (1 week):
1. Multi-file refactoring (measure tool calls)
2. Bug fixing (measure time to resolution)
3. Code generation (measure style consistency)
4. Codebase exploration (measure context efficiency)

**Metrics to validate**:
- 60% reduction in tool calls (memory system)
- 19% context efficiency gain (dynamic context)
- 15-30% improvement in specialized tasks (intent classification)

**Blog Post Series** (1 week):
- Post 1: Memory system validation (with graphs)
- Post 2: Toad + ACP architecture (why it works)
- Post 3: Benchmark results vs Claude Code
- Post 4: Open source release announcement

**Success Criteria**:
- Empirical proof of improvements
- Reproducible benchmarks
- Blog posts ready to publish

### Week 9-10: Research Paper + Release
**Goal**: Publication-ready contribution

**Research Paper** (1 week):
- Title: "Memory-Augmented Coding Agents: Empirical Validation"
- Sections: Intro, Related Work, Architecture, Evaluation, Results
- Focus: 60% improvement from knowledge graph + episodic memory
- Venue: arXiv + submit to conference (ICSE, FSE, or similar)

**Open Source Release** (1 week):
- Installation guide for Toad + Zed
- Contributor documentation
- Example usage patterns
- Demo video (Toad terminal workflow)
- Reddit/HN announcement

**Success Criteria**:
- Paper draft complete
- Installation < 5 minutes
- Community can reproduce benchmarks

## Competitive Position After 10 Weeks

### vs Claude Code
- ✅ Better: 60% fewer tool calls (memory system)
- ✅ Better: Works in more frontends (Toad, Zed, Neovim, Emacs, JetBrains)
- ✅ Better: Open source (community contributions)
- ⚠️ Similar: Intent classification (need to benchmark)
- ❌ Behind: Fewer total tools (but higher quality)

### vs Factory Droid
- ❌ Behind: Benchmark scores (58.8% vs our unknown)
- ✅ Better: Open source (Droid is closed/commercial)
- ✅ Better: Memory system (Droid doesn't have this)
- ✅ Better: Multi-frontend (Droid is proprietary)

### vs Goose
- ✅ Better: Memory system (Goose doesn't have)
- ✅ Better: Semantic search (hnswlib-rs advantage)
- ⚠️ Similar: ACP compatibility (both support)
- ❌ Behind: Market adoption (Goose has Block backing)

## Key Dependencies & Risks

### Dependencies
1. **Toad ACP support**: Announced July 2025, may not be stable yet
   - Mitigation: Use Zed (stable ACP) as primary test platform
   - Toad can come later (Week 5-6)

2. **ACP protocol stability**: Still evolving
   - Mitigation: Track github.com/zed-industries/agent-client-protocol
   - Implement core spec, extend as needed

3. **Rust ecosystem**: petgraph vs NetworkX for knowledge graph
   - Mitigation: petgraph is mature, 3.2M downloads
   - Fallback: Use serde_json for graph storage

### Risks
1. **Memory port complexity**: Python POC → Rust may be harder
   - Mitigation: Keep POC design simple, port incrementally
   - Budget: 1.5 weeks, can extend to 2 weeks

2. **Benchmark reproducibility**: Hard to match exact conditions
   - Mitigation: Use SWE-bench or Terminal-bench (public datasets)
   - Document exact setup (models, prompts, hardware)

3. **Toad timeline**: May not stabilize by Week 5-6
   - Mitigation: Focus on Zed + Neovim as proven frontends
   - Toad is bonus, not blocker

## Success Metrics (Week by Week)

| Week | Tools Real | Parity | Key Milestone |
|------|-----------|--------|---------------|
| 1    | 5/10      | 23-27% | File ops complete |
| 2    | 9/10      | 30-35% | Code tools complete |
| 3    | 9/10      | 35-40% | ACP working in Zed |
| 4    | 9/10      | 40-45% | Memory system ported |
| 5    | 9/10      | 45-50% | Toad integration |
| 6    | 9/10      | 50-55% | Intelligence wired |
| 7    | 9/10      | 55-60% | Benchmarks run |
| 8    | 9/10      | 60-65% | Blog posts written |
| 9    | 9/10      | 65-70% | Paper draft done |
| 10   | 9/10      | 70-75% | Released + documented |

## Why This Plan Works

1. **Leverage existing work**: 86K lines Rust stays (semantic search irreplaceable)
2. **Save time**: Toad handles UI (4-6 weeks saved)
3. **Proven design**: Memory POC validated (60% improvement)
4. **Clear milestones**: Week-by-week deliverables
5. **Research contribution**: Empirical validation + publication
6. **Multi-frontend**: Works in 5+ editors via ACP

## Next Actions (Week 2 Start)

**Immediate (Today)**:
1. Read `src/agent/tools/` to understand tool structure
2. Start `search_code` implementation (leverage `src/intelligence/semantic_search/`)
3. Create test plan for Week 2 tools

**This Week**:
- Day 1-3: search_code (integrate semantic search)
- Day 3-5: analyze_code (tree-sitter AST analysis)
- Day 5-6: find_references (symbol tracking)
- Day 6-7: find_definition (lookup with context)

**Success**: 9/10 tools real by end of Week 2.

---

**Mission**: Build the smartest ACP-compatible agent backend with validated memory system and empirical benchmarks.

**Focus**: Agent intelligence (tools, memory, intent), not UI (Toad handles that).

**Timeline**: 10 weeks to research paper + open source release.
