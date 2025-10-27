# Aircher Agent Instructions

Entry point for AI agents working with Aircher - an intelligent ACP-compatible coding agent backend.

For organization patterns: @external/agent-contexts/PRACTICES.md

## Project Overview

**Intelligent Agent Backend** via Agent Client Protocol (ACP)

**Core Value Proposition:**
- Novel agent intelligence with intent classification and dynamic context management
- Works in Zed, JetBrains IDEs (coming), Neovim, Emacs, or any ACP-compatible frontend
- Focus: Agent intelligence, not UI - let editors handle the interface

**⚠️ CRITICAL**: See @docs/STATUS.md and @ai/STATUS.md for current state
- Status: Week 6 Day 1 complete, 30-33% feature parity
- Memory Systems: ✅ ALL 3 COMPLETE (3,725 lines) - Episodic, Knowledge Graph, Working
- ACP Protocol: ✅ 90% COMPLETE (major discovery!) - Ready for enhancements
- Frontend: Toad + Zed/Neovim/Emacs via ACP
- Backend: Rust (86K lines) - performance advantage for benchmarks
- Repository: Public at https://github.com/nijaru/aircher

## Key Files (Always Check/Update)

### 📊 AI Working Context (ai/)
@ai/TODO.md                        # **Active tasks** - What to work on now
@ai/STATUS.md                      # **Current state** - What works/doesn't
@ai/DECISIONS.md                   # **Architectural choices** (append-only)
@ai/RESEARCH.md                    # **Research index** - SOTA findings

### 📚 Project Documentation (docs/)
@docs/STATUS.md                    # **Public status** - User-facing capabilities
@docs/ROADMAP.md                   # **10-week plan** - Strategic timeline
@docs/TECH_SPEC.md                 # **Technical spec** - System architecture

### 🏗️ Architecture & Design
@docs/architecture/agent-first-architecture.md  # ACP-compatible design
@docs/architecture/MODEL_VS_AGENT_ARCHITECTURE.md  # Model vs agent split

## What Makes Aircher Unique

### 1. **Intent Classification System**
Automatic detection and routing based on user intent:
- `CodeReading` - Analysis and comprehension tasks
- `CodeWriting` - Implementation and generation
- `ProjectFixing` - Debugging and error resolution
- `ProjectExploration` - Codebase navigation and discovery

**Research Contribution**: Intent-driven execution strategies vs one-size-fits-all

### 2. **Dynamic Context Management**
Single agent with intelligent context vs sub-agents:
- Smart pruning and prefetching
- Relevance scoring and token optimization
- No coordination overhead or tunnel vision

**Empirical Evidence**: 19% performance advantage over sub-agent architectures

### 3. **Memory-Augmented Intelligence** (NEW: Week 3-5)
Three-layer memory system enabling continuous work without restart:

**Knowledge Graph** (petgraph in-memory):
- Codebase structure: 3,942 nodes, 5,217 edges (Aircher POC)
- Microsecond queries: "what calls this?", "what's in file X?"
- Incremental updates when files change

**Episodic Memory** (DuckDB):
- Track everything: tool calls, file interactions, tasks
- Learn patterns: co-edit detection, error-fix patterns
- 5 tables: tool_executions, file_interactions, task_history, context_snapshots, learned_patterns

**Working Memory** (Dynamic Context):
- Intelligent pruning: Remove bottom 30% by relevance score
- Relevance = time_decay × task_association × dependencies × type_weight
- **Key innovation**: Continuous work, no restart needed

**Measurable Impact**: 60% reduction in tool calls (POC validated)

### 4. **Pattern-Aware Code Generation**
Learns project-specific conventions:
- Automatic style extraction
- Context-aware suggestions
- Architectural compliance checking

**Measurable Impact**: Code consistency matching existing codebase patterns

### 5. **Unified Intelligence Middleware**
Transparent automatic enhancement:
```rust
EnhancedContext {
    detected_intent: UserIntent,
    intelligence_insights: Vec<IntelligenceInsight>,
    confidence: f32,
}
```

## Architecture: ACP-Compatible Agent Backend

```
Frontends (choose any)
├── Toad (PRIMARY - universal terminal UI, Python/Textual)
├── Zed (native ACP support)
├── JetBrains IDEs (October 2025 collaboration)
├── Neovim (CodeCompanion, avante.nvim plugins)
├── Emacs (agent-shell)
└── VSCode (via ACP adapter)
    ↓ (Agent Client Protocol - JSON-RPC over stdio)
    ↓
Aircher Agent Backend (Rust)
├── Semantic Search (hnswlib-rs, 45x faster)
├── Knowledge Graph + Episodic Memory (60% improvement)
├── Intent Classification
├── Dynamic Context Management
├── Pattern Learning
└── 9 Production Tools (Week 2 target)
```

**Primary Frontend**: Toad (universal terminal UI) - saves 4-6 weeks vs custom TUI
**Backend**: Rust agent - performance critical for benchmarks
**You work on**: Agent intelligence (tools, memory, intent, context)
**Toad handles**: Terminal UI, rendering, keyboard shortcuts

## Key Architecture Insights

### Models are Reasoning Engines, Agents are Execution Engines (Sep 19, 2025)
- **Discovery**: Over-engineered 1685-line MultiTurnReasoningEngine externalized what models do internally
- **Research validated**: 25-70% improvements from prompts, not orchestration
- **Solution**: Enhanced prompting system (300 lines) replaces complex orchestration
- **Details**: @docs/architecture/MODEL_VS_AGENT_ARCHITECTURE.md

### Dynamic Context > Sub-Agents (Sep 14, 2025)
- **Research finding**: Sub-agents cause 19% performance degradation
- **Problems**: Tunnel vision, context pollution, coordination overhead
- **Our innovation**: Single agent with intelligent context management
- **Competitive advantage**: Better than Claude Code's sub-agents without overhead

## Development Philosophy

**Focus: Research-Grade Agent Intelligence**
- Novel architectural contributions
- Empirical validation vs competitors
- Open source for community benefit
- Publication-worthy results

**Not Building:**
- ❌ Custom TUI or IDE
- ❌ UI themes and customization
- ❌ Enterprise features (SSO, audit, team collab)

**Building:**
- ✅ Intelligent agent backend
- ✅ ACP protocol implementation
- ✅ Real tool implementations
- ✅ Empirical benchmarks
- ✅ Research paper contributions

## Current Development Status

### What Works ✅
- **Semantic Search**: Production-ready, 19+ languages, 6,468 vectors indexed
- **ACP Protocol**: ✅ 90% COMPLETE - JSON-RPC over stdio, all 6 Agent trait methods
- **Memory Systems**: ✅ ALL 3 COMPLETE (3,725 lines)
  - Episodic Memory (DuckDB, 815 lines): 5 tables, 11 CRUD ops, 7 queries
  - Knowledge Graph (petgraph, 1,470 lines): tree-sitter extraction, 8 queries
  - Working Memory (820 lines + 620 test lines): dynamic pruning, relevance scoring
- **Intelligence Framework**: 210+ Rust files, substantial implementation
- **Multi-Provider**: OpenAI, Anthropic, Gemini, Ollama
- **Production Tools**: 5 file operations (2,110+ lines, 21+ tests)

### What's In Progress 🔄
- **Week 6 Days 2-7**: ACP protocol enhancements
  - Session state tracking (HashMap<SessionId, SessionState>)
  - Conversation history per session
  - Streaming response support (token-by-token)
  - Tool execution progress updates
  - Error handling and recovery
  - End-to-end testing with Zed
- **Week 7-8**: Benchmarks vs Claude Code (validate 60% memory improvement)
- **Week 9-10**: Research paper + open source release

### Current Priority (Week 6)
**ACP Protocol Enhancements**: Session management + streaming + testing
- Session state tracking: Maintain conversation history, timeouts
- Streaming: Token-by-token responses, progress updates
- Error recovery: Retry logic, graceful degradation
- Testing: End-to-end with Zed editor
- **Major Win**: ACP already 90% done (saved 4-5 days!)

### Completed (Weeks 1-5)

**Week 1: File Operations**
- 4 production tools: read_file, write_file, edit_file, list_files
- 2,110+ lines, 21+ tests

**Week 2: Code Understanding** (Skipped - tools already exist)
- Validated existing tools work

**Week 3: DuckDB Episodic Memory** ✅
- 5 tables, 11 CRUD operations, 7 query methods
- +815 lines production code

**Week 4: petgraph Knowledge Graph** ✅
- Tree-sitter extraction, 5 node types, 6 edge types
- Binary persistence, 8 query methods
- +1,470 lines production code

**Week 5: Dynamic Context Management** ✅
- ContextWindow with intelligent pruning
- Relevance scoring algorithm
- DynamicContextManager integrating all 3 systems
- +820 lines production code, +620 lines tests

**Week 6 Day 1: ACP Protocol Review** ✅
- Discovered ACP already 90% implemented
- Comprehensive documentation created (@docs/acp-integration.md)

**Architecture Details**: See @ai/research/memory-system-architecture.md (500+ lines)

See @ai/TODO.md for current sprint details.

## Quick Reference

### For Development
- **Entry point**: This file (AGENTS.md)
- **Current tasks**: @ai/TODO.md
- **Roadmap**: @docs/ROADMAP.md (10-week agent intelligence plan)
- **Architecture**: @docs/architecture/

### For Research
- **Novel contributions**: Intent classification, dynamic context, pattern learning
- **Benchmarks needed**: vs Claude Code, sub-agents, static context
- **Target publication**: Agent intelligence architecture paper

### For Integration
- **Protocol**: Agent Client Protocol (ACP)
- **Transport**: JSON-RPC over stdio
- **Frontends**: Zed (best), JetBrains (coming), Neovim, Emacs
- **Installation**: Via frontend's agent management (not standalone)

## Tool Format

**ACP Standard**: JSON-RPC over stdio
```json
{
  "jsonrpc": "2.0",
  "method": "agent/prompt",
  "params": {
    "session_id": "...",
    "content": [...]
  }
}
```

**Internal**: Rust async trait-based
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    async fn execute(&self, params: Value) -> Result<ToolOutput>;
}
```

## Code Standards

- Follow @external/agent-contexts/standards/AI_CODE_PATTERNS.md
- Zero warnings policy (competitive quality)
- Document decisions in @ai/DECISIONS.md
- Check current state in @ai/STATUS.md

---

**Mission**: Build the smartest ACP-compatible agent backend through novel intelligence architecture and empirical validation.

**Not**: Another editor or UI - focus on agent intelligence, let frontends handle UX.
