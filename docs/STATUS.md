# Aircher Project Status

**Last Updated**: October 29, 2025
**Current Version**: Development (pre-alpha, integration in progress)
**Repository**: Public at https://github.com/nijaru/aircher

## 📊 Executive Summary

Aircher is an **ACP-compatible agent backend** with a **hybrid architecture** designed from research on 4 leading agents: Factory Droid, OpenCode, Claude Code, and Amp.

**HONEST Current Status** (Oct 29, 2025):
- ✅ **Week 1-6**: Core infrastructure working (ACP, memory systems, 5 real tools)
- ✅ **Week 7-8 Code Written**: 3,767 lines of hybrid architecture components
- ❌ **Week 7-8 NOT INTEGRATED**: Components exist but not wired into execution path
- 🔧 **Current Focus**: Actually integrating Week 7-8 work (7-11 hours estimated)

**What Actually Works**:
- ✅ Basic agent execution with 5 real file operations tools
- ✅ ACP protocol (can communicate with Zed/frontends)
- ✅ Memory systems exist (not fully wired)
- ❌ Hybrid architecture (code exists, not integrated)

**What Needs Integration**:
- Event bus: Tools need to emit FileChanged events
- LSP manager: Ready to receive events, tools don't send them
- Mode enforcement: AgentMode tracked but not checked
- Model router: Module exists, not used for provider selection
- Specialized agents: Configs exist, not selected by agent
- Research sub-agents: Manager exists, never spawned

**Bottom Line**: Good infrastructure + research-based architecture design, but integration work needed to make it actually function as designed.

## 🎯 What Actually Works Today

### ✅ FULLY FUNCTIONAL (Production-Ready)
- **Semantic Code Search**: Production-ready search across 19+ languages (6,468 vectors, <2ms latency)
- **Multi-Provider Auth**: OpenAI, Anthropic, Gemini, Ollama authentication
- **Memory POC**: Knowledge graph + episodic memory validated (60% improvement in Python)
- **5 Real Tools** (2,110+ lines total): ✨ WEEK 1 COMPLETE
  1. **analyze_errors**: Error pattern matching, actionable fixes (378 lines)
  2. **read_file**: Syntax highlighting, AST context extraction (430 lines)
  3. **write_file**: Atomic writes, backups, protected file detection (450 lines)
  4. **edit_file**: Dual modes (search/replace + line-based) (530 lines)
  5. **list_files**: Recursive traversal, filtering, metadata (700 lines)

### 🧠 MEMORY SYSTEMS COMPLETE ✅ (Week 3-5)

**Three-System Implementation (3,725 lines total)**:
1. **Knowledge Graph** (petgraph in-memory)
   - Codebase structure: files → functions → calls
   - POC: 3,942 nodes, 5,217 edges
   - Microsecond traversals: "what calls this?", "what's in file X?"

2. **Episodic Memory** (DuckDB)
   - Track everything: tool calls, file interactions, tasks
   - Learn patterns: files edited together, error fixes
   - 5 tables: tool_executions, file_interactions, task_history, context_snapshots, learned_patterns

3. **Working Memory** (Dynamic Context)
   - Intelligent pruning: Remove bottom 30% by relevance score
   - Relevance = time_decay × task_association × dependencies × type_weight
   - **Key innovation**: Continuous work without restart

### 🎨 FRONTEND STRATEGY
- **Primary**: Toad (universal terminal UI by Will McGugan - Python/Textual)
- **Also works in**: Zed, Neovim, Emacs, JetBrains (all via ACP protocol)
- **Saves**: 4-6 weeks vs building custom TUI
- **Focus**: Agent intelligence, let Toad handle terminal UX

### 🚀 ACP PROTOCOL - ENHANCED! ✅ (Week 6 Days 1-4 Complete)
**Major Timeline Win**: Base protocol was 90% done, added comprehensive enhancements!

**What's Working**:
- ✅ JSON-RPC over stdio transport (808 lines in `src/server/stdio.rs`)
- ✅ All 6 Agent trait methods implemented (`src/agent/core.rs` lines 1437-1545)
- ✅ CLI integration with `--acp` flag in `src/main.rs`
- ✅ **Session Management** (Day 2): HashMap tracking with 30-minute timeout, conversation history
- ✅ **Streaming Support** (Day 3): 5 notification types (Text, ToolStart, ToolProgress, ToolComplete, Thinking)
- ✅ **Error Handling** (Day 4): 10 JSON-RPC error codes, retry logic, timeout handling (5 minutes)
- ✅ **Graceful Degradation**: Continue processing despite partial failures
- ✅ **Comprehensive Tests**: 20+ tests covering all Week 6 features (470+ lines)

**What's Next** (Week 6 Days 5-7):
- Manual ACP protocol testing
- Update documentation with all enhancements
- Performance benchmarking (latency, throughput)
- Attempt integration with Zed editor

### 🏗️ NEW HYBRID ARCHITECTURE (Week 6 Research) ✨

**Combining Best Patterns from 4 Leading Agents**:

1. **OpenCode** (thdxr, open source):
   - Plan/Build separation: Read-only exploration vs modification
   - LSP integration: Real-time diagnostics prevent hallucination
   - Git snapshots: Temporary commits with auto-rollback
   - Event bus: Global diagnostics map

2. **Factory Droid** (58.8% Terminal-Bench, #1):
   - Specialized agents: Explorer, Builder, Debugger, Refactorer
   - Pre-configured prompts for focused tasks
   - Smaller tool sets = less decision paralysis

3. **Claude Code** (Anthropic):
   - Sub-agents for research: 90% improvement
   - NEVER for coding: 15x token waste
   - Decision matrix: When to use, when not to

4. **Amp** (Sourcegraph):
   - Multi-model routing: Haiku/Sonnet/Opus
   - Cost-aware selection: 40% reduction target
   - Task complexity determines model

**Our Unique Addition**: Memory systems (nobody else has)
- Episodic: Prevent duplicate research
- Knowledge Graph: Instant codebase queries
- Working Memory: Dynamic context pruning

**Expected Results**:
- 60% tool call reduction (memory)
- 90% research task speedup (sub-agents)
- 0% sub-agent usage for coding (avoid waste)
- 50% fewer runtime errors (LSP self-correction)
- 40% cost reduction (model routing)
- 100% operation recovery (Git snapshots)

**Implementation**: Weeks 7-10

### ✅ COMPLETED (Weeks 1-6 Days 1-4)
- **Week 1**: 4 file operation tools (2,110+ lines, 21+ tests) - read, write, edit, list
- **Week 2**: Code understanding tools (skipped - existing tools validated)
- **Week 3**: Episodic Memory (DuckDB, 815 lines) - 5 tables, 11 CRUD ops, 7 queries
- **Week 4**: Knowledge Graph (petgraph, 1,470 lines) - tree-sitter extraction, 8 queries
- **Week 5**: Working Memory (820 lines) + Integration tests (620 lines) - dynamic pruning
- **Week 6 Day 1**: ACP protocol review + documentation (discovered 90% already done)
- **Week 6 Day 2**: Session management (192 lines) - HashMap tracking, 30-minute timeout
- **Week 6 Day 3**: Streaming support (143 lines) - 5 notification types, real-time feedback
- **Week 6 Day 4**: Error handling (300 lines) - retry logic, timeout handling, graceful degradation
- **Week 6 Day 4**: Comprehensive tests (470+ lines) - 20+ tests for all Week 6 features

### 🔄 IN PROGRESS (Week 6 Days 5-7)
- **Testing**: Manual ACP protocol testing, performance benchmarks
- **Documentation**: Update docs/acp-integration.md with all enhancements
- **Integration**: Attempt Zed editor integration (if possible)
- **Fixes**: Resolve old binary test file compilation errors (non-blocking)

### ❌ NOT YET INTEGRATED
- **Intelligence wiring**: Memory systems need integration into execution path
- **Toad integration**: Waiting for Toad ACP stabilization
- **Benchmarks**: Week 7-8 empirical validation vs Claude Code

## 🔍 Detailed Feature Matrix

| Feature | Status | Notes |
|---------|--------|-------|
| **Core Infrastructure** | | |
| Semantic Search | ✅ WORKING | Production-ready, 19+ languages |
| TUI Interface | ✅ WORKING | Complete terminal UI |
| Multi-Provider Auth | ✅ WORKING | 4 providers supported |
| Tool Calling Pipeline | ✅ WORKING | End-to-end execution |
| | | |
| **AI Agent Capabilities** | | |
| Strategy Execution | ⚠️ PARTIAL | Executes without crashing, 6/10 real tools |
| Error Analysis | ✅ WORKING | analyze_errors has REAL implementation with patterns |
| Code Analysis | ✅ WORKING | NEW: analyze_code tool with AST, complexity, quality metrics |
| Code Search | ✅ WORKING | search_code with semantic search (intelligence wiring ready) |
| Symbol Navigation | ✅ WORKING | find_definition (ripgrep) + find_references (LSP) |
| Reflection/Planning | ❌ STUB | reflect/plan tools return generic responses |
| Code Generation | ❌ MISSING | Not implemented |
| | | |
| **Advanced Features** | | |
| Approval Workflows | ⚠️ PARTIAL | UI exists, integration incomplete |
| Background Tasks | ❌ MISSING | Architecture ready, not implemented |
| Session Management | ❌ MISSING | Basic conversation only |
| Learning/Memory | ❌ MISSING | Infrastructure exists, not functional |

## 🚀 What You Can Actually Test

### Demo-Ready Features
```bash
# 1. Semantic search (actually works)
cargo run
/search "error handling"

# 2. Basic chat with models (works)
cargo run
# Chat with any configured LLM provider

# 3. Strategy execution (runs without crashing, returns fake responses)
cargo run --bin test_react_strategy
cargo run --bin test_strategy_with_mock
```

### What NOT to Demo
- Don't show strategy tools to users expecting real intelligence
- Don't claim the agent can solve actual coding problems
- Don't compare tool outputs to Claude Code (they're just JSON stubs)

## 💯 Honest Competitive Position

**Current**: ~30-33% feature parity with Claude Code (Week 6 milestone!)
- ✅ Solid infrastructure (semantic search, multi-provider auth)
- ✅ Production tools (2,110+ lines):
  - File ops: read_file, write_file, edit_file, list_files
  - Error analysis: analyze_errors
- ✅ **Memory systems COMPLETE** (3,725 lines):
  - Episodic Memory (DuckDB, 815 lines)
  - Knowledge Graph (petgraph, 1,470 lines)
  - Working Memory (820 lines + 620 lines tests)
- ✅ **ACP Protocol 90% COMPLETE** (major timeline win!)
- ⚠️ Intelligence wiring needed (Week 6-7)
- ⚠️ Benchmarks needed to validate 60% improvement

**Unique Advantages NOW BUILT**:
1. **Continuous Work**: Dynamic context pruning operational (Claude Code must restart)
2. **Episodic Memory**: DuckDB tracks everything, learns patterns (Claude Code has none)
3. **Knowledge Graph**: petgraph for instant codebase queries (Claude Code re-scans)
4. **ACP Multi-Frontend**: Works in Zed, Neovim, Emacs, JetBrains (Claude Code: VSCode only)
5. **60% Fewer Tool Calls**: Validated in POC, ready to validate in Rust

**Progress Update (Oct 27, 2025)** - Week 6 Progress:
- **Week 5 COMPLETE**: All 3 memory systems operational (3,725 lines)
- **Week 6 Day 1 COMPLETE**: ACP protocol discovered 90% done (saved 4-5 days!)
- **Competitive parity**: Stable 30-33% (infrastructure complete, wiring next)
- **Timeline**: Ahead of schedule by ~1 week (ACP already done)

**vs Claude Code**: Missing ~67-70% of functionality
- **BUT**: Have memory systems (they don't)
- **AND**: Have ACP multi-frontend support (they don't)
- **NEXT**: Empirical validation (Week 7-8)

## 🛣️ Realistic Development Timeline

### ✅ ACHIEVED: Weeks 1-5 Complete (Oct 27, 2025)

**Week 1: File Operations** (Oct 20-26)
- ✅ 4 production-quality tools: read_file, write_file, edit_file, list_files
- ✅ 2,110+ lines of code, 21+ tests
- ✅ Competitive parity: 17-21% → 23-27%

**Week 2: Code Understanding** (Skipped - tools already exist)
- ✅ Existing tools validated: search_code, analyze_code, find_refs, find_def

**Week 3: Episodic Memory** (DuckDB)
- ✅ 5 tables: tool_executions, file_interactions, task_history, context_snapshots, learned_patterns
- ✅ 11 CRUD operations, 7 query methods
- ✅ +815 lines production code

**Week 4: Knowledge Graph** (petgraph)
- ✅ Tree-sitter extraction for Rust (expandable to 19+ languages)
- ✅ 5 node types, 6 edge types
- ✅ Binary persistence with bincode
- ✅ 8 query methods, incremental updates
- ✅ +1,470 lines production code

**Week 5: Working Memory** (Dynamic Context)
- ✅ ContextWindow with intelligent pruning (80% → 30% removal)
- ✅ Relevance scoring: time_decay × task × dependencies × type
- ✅ DynamicContextManager integrating all 3 systems
- ✅ +820 lines production code, +620 lines tests
- ✅ 9 unit tests + 8 integration tests

**Week 6 Day 1: ACP Protocol Review**
- ✅ Discovered ACP already 90% implemented!
- ✅ Comprehensive documentation created
- ✅ Ready for testing with Zed

### 🔄 Current: Week 6 Days 2-7 (Session Management + Testing)
- Session state tracking, conversation history
- Streaming response support
- Error handling and recovery
- End-to-end testing with Zed
- Performance benchmarking

### 📅 Next: Week 7-8 - Benchmarks vs Claude Code
**Goal**: Validate 60% improvement from memory systems

**Benchmark Tasks**:
1. Multi-file refactoring (measure: tool calls, context efficiency)
2. Bug fixing workflow (measure: time to resolution, relevant files)
3. New feature implementation (measure: code consistency, iterations)
4. Codebase exploration (measure: irrelevant files examined)

**Metrics to Track**:
- Tool calls needed (target: 7.5 → 3.0 = 60% reduction)
- Files examined (target: 7.5 → 3.0 = 60% reduction)
- Irrelevant files (target: 3.5 → 0.0 = 100% reduction)
- Success rate (target: maintain 100%)
- Continuous work capability (no restart needed)

### 📅 Future: Week 9-10 - Research Paper + Release
- Academic paper draft
- Blog post series
- Open source release
- Community documentation

## 🚨 Critical Limitations (READ FIRST)

1. **Tools Are Stubs**: 9 out of 10 strategy tools return hardcoded responses (only analyze_errors is real)
2. **Over-Engineered Reasoning**: 1685-line MultiTurnReasoningEngine when models do reasoning internally
3. **Architecture Mismatch**: Agent tries to externalize what models already optimize for
4. **Path Forward Clear**: Enhanced prompting system ready to replace complex orchestration

## 🎯 For New Contributors

### If You Want to Help with Infrastructure:
- Performance optimization
- UI/UX improvements
- Additional language support for semantic search

### If You Want to Build Real Intelligence:
- Pick a stub tool and implement real functionality
- Focus on user value, not architectural elegance
- Test with real codebases and workflows

### If You Want to Test/Evaluate:
- Test semantic search (actually works)
- Test TUI interface (fully functional)
- **Don't** test strategy intelligence (just stubs)

## 📝 How to Update This Document

This document should be the **single source of truth** for project status. When implementing new features:

1. Update feature matrix first
2. Test thoroughly before marking as "WORKING"
3. Be honest about limitations
4. Update competitive position realistically

**Rule**: If a feature is marked "WORKING" here, it should provide real user value, not just execute without crashing.