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
- Status: Week 6 COMPLETE, 30-33% feature parity
- Memory Systems: ✅ ALL 3 COMPLETE (3,725 lines) - Episodic, Knowledge Graph, Working
- ACP Protocol: ✅ ENHANCED (+635 lines) - Session management, streaming, error handling
- **NEW**: Hybrid architecture combining SOTA patterns from 4 leading agents
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
@ai/SYSTEM_DESIGN_2025.md          # **NEW**: Hybrid SOTA architecture (Week 6 redesign)
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

### 4. **Hybrid Architecture Combining SOTA Patterns** (NEW: Week 6)
Combines best patterns from 4 leading agents:

**From OpenCode** (thdxr, production-validated):
- Plan/Build mode separation (safe exploration vs controlled modification)
- LSP integration (real-time diagnostics, self-correction)
- Git snapshots (100% recovery from failed operations)
- Event bus (global diagnostics map)

**From Factory Droid** (#1 on Terminal-Bench, 58.8%):
- Specialized agent configs (Explorer, Builder, Debugger, Refactorer)
- Pre-configured prompts for focused tasks
- Smaller tool sets = less decision paralysis

**From Claude Code** (Anthropic):
- Smart sub-agent usage:
  - ✅ Research: 90% improvement (parallel execution)
  - ❌ Coding: 15x waste avoided (never use for coding)
- Decision matrix based on task type

**From Amp** (Sourcegraph):
- Multi-model routing (Haiku/Sonnet/Opus)
- Cost-aware selection (40% reduction target)
- Task complexity determines model

**Our Unique Addition**: Memory systems (nobody else has)
- Episodic: Prevent duplicate research
- Knowledge Graph: Instant codebase queries
- Working Memory: Dynamic context pruning

**Expected Results**:
- 60% tool call reduction (memory systems)
- 90% research speedup (parallel sub-agents)
- 0% sub-agent usage for coding (avoid waste)
- 50% fewer runtime errors (LSP self-correction)
- 40% cost reduction (model routing)
- 100% operation recovery (Git snapshots)

**Implementation Timeline**: Weeks 7-10
**Details**: @ai/SYSTEM_DESIGN_2025.md

### 5. **Pattern-Aware Code Generation**
Learns project-specific conventions:
- Automatic style extraction
- Context-aware suggestions
- Architectural compliance checking

**Measurable Impact**: Code consistency matching existing codebase patterns

### 6. **Unified Intelligence Middleware**
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

### Hybrid Architecture: Smart Sub-Agent Usage (Oct 27, 2025)
- **Research finding**: Sub-agents have OPPOSITE effects for different tasks
  - ✅ **Research tasks**: 90% improvement (parallel execution)
  - ❌ **Coding tasks**: 15x token waste (context isolation fatal, 160k tokens for 3k work)
- **Our innovation**: Decision matrix based on UserIntent classification
  - **Plan mode**: Can spawn research sub-agents (max 10 concurrent)
  - **Build mode**: NEVER uses sub-agents (avoid 15x waste)
- **Memory integration**: Check episodic memory before spawning to prevent duplicate research
- **Details**: @ai/SYSTEM_DESIGN_2025.md and @ai/DECISIONS.md (2025-10-27)

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
- **Week 7-8**: Core architecture implementation
  - Event bus + LSP integration (Week 7 Days 1-2)
  - Plan/Build mode separation (Week 7 Days 3-4)
  - Git snapshots (Week 7 Day 5)
  - Model router (Week 7 Days 6-7)
  - Specialized agent configs (Week 8 Days 1-2)
  - Research sub-agents (Week 8 Days 3-4)
- **Week 9**: Benchmarks vs Claude Code (validate hybrid architecture improvements)
- **Week 10**: Research paper + open source release

### Current Priority (Week 7)
**Core Architecture Patterns**: Implementing hybrid design combining SOTA patterns
- **Days 1-2**: Event bus (tokio broadcast) + LSP integration
  - LspManager with global diagnostics map
  - Edit → LSP → diagnostics → agent self-corrects
  - Real-time feedback loop (50% fewer runtime errors target)
- **Days 3-4**: Plan/Build mode separation
  - AgentMode enum with tool restrictions
  - Plan: read-only, can spawn research sub-agents
  - Build: can modify, never uses sub-agents
- **Day 5**: Git snapshots for safe experimentation
- **Days 6-7**: Multi-model routing for cost optimization

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

**Week 6 Days 1-4: ACP Protocol Enhancements** ✅
- Day 1: Protocol review (discovered 90% already implemented)
- Day 2: Session management (192 lines) - HashMap tracking, 30-minute timeout
- Day 3: Streaming support (143 lines) - 5 notification types, real-time feedback
- Day 4: Error handling (300 lines) - retry logic, timeout handling, graceful degradation
- Day 4: Comprehensive tests (470+ lines) - 20+ tests for all Week 6 features
- Total: +635 lines production code, +470 lines test code

**Week 6 Days 5-6: SOTA Research + Architecture Redesign** ✅
- Researched Factory Droid, OpenCode, Claude Code, Amp
- Created ai/SYSTEM_DESIGN_2025.md (+500 lines)
- Updated all documentation with hybrid architecture
- Ready for Week 7 implementation

**Architecture Details**: See @ai/SYSTEM_DESIGN_2025.md and @ai/research/memory-system-architecture.md

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
