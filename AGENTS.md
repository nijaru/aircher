# Aircher Agent Instructions

Entry point for AI agents working with Aircher - an intelligent ACP-compatible coding agent backend.

@external/agent-contexts/AI_AGENT_INDEX.md

## Project Overview

**Intelligent Agent Backend** via Agent Client Protocol (ACP)

**Core Value Proposition:**
- Novel agent intelligence with intent classification and dynamic context management
- Works in Zed, JetBrains IDEs (coming), Neovim, Emacs, or any ACP-compatible frontend
- Focus: Agent intelligence, not UI - let editors handle the interface

**⚠️ CRITICAL**: See @PROJECT_STATUS.md and @internal/PROJECT_REALITY.md for honest assessment
- Status: Strong architecture, infrastructure complete, tools need implementation
- Current: 16-20% feature parity (infrastructure vs actual capabilities)
- Focus: Agent intelligence research and implementation

## Key Files (Always Check/Update)

### 📊 Project Status & Reality
@PROJECT_STATUS.md                 # **READ FIRST**: Current capabilities & limitations
@internal/PROJECT_REALITY.md       # **HONEST ASSESSMENT**: Real vs claimed functionality
@internal/NOW.md                   # Current sprint & priorities

### 🏗️ Architecture & Decisions
@docs/architecture/agent-first-architecture.md  # ACP-compatible agent design
@docs/architecture/MODEL_VS_AGENT_ARCHITECTURE.md  # Model vs agent responsibilities
@internal/DECISIONS.md             # Major decisions (append-only)
@internal/TECH_SPEC.md             # Technical specifications

### 🔬 Research & Intelligence
@internal/KNOWLEDGE.md             # Competitive intelligence & patterns
@internal/DISCOVERIES.md           # Research insights & breakthroughs
@internal/AGENT_FIRST_ROADMAP.md   # Development plan (agent intelligence focus)

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

### 3. **Pattern-Aware Code Generation**
Learns project-specific conventions:
- Automatic style extraction
- Context-aware suggestions
- Architectural compliance checking

**Measurable Impact**: Code consistency matching existing codebase patterns

### 4. **Intelligent Debugging**
Root cause analysis with system awareness:
- Cross-file dependency tracking
- Multiple fix strategies with risk assessment
- Impact analysis before changes

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
├── Zed (native ACP support)
├── JetBrains IDEs (October 2025 collaboration)
├── Neovim (CodeCompanion, avante.nvim plugins)
├── Emacs (agent-shell)
└── VSCode (via ACP adapter)
    ↓ (Agent Client Protocol - JSON-RPC over stdio)
    ↓
Aircher Agent Backend
├── Intent Classification
├── Dynamic Context Management
├── Pattern Learning
├── Intelligent Tool Execution
└── Result Validation
```

**You work on**: Agent intelligence
**Frontend handles**: UI, keyboard shortcuts, themes, etc.

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
- **ACP Architecture**: Designed and ready for implementation
- **Intelligence Framework**: 210+ Rust files, substantial implementation
- **Multi-Provider**: OpenAI, Anthropic, Gemini, Ollama
- **Dynamic Context**: Architecture implemented

### What's In Progress 🔄
- **Tool Implementation**: Replacing stubs with real functionality
- **ACP Protocol**: stdio transport and session management
- **Intent Classification**: Making it operational
- **Benchmarking**: Validation vs Claude Code/competitors

### Current Priority (Week 1-2)
**Real Tool Implementation**: Replace 9 stub tools with production-quality implementations
- File operations (read, write, edit, list)
- Code understanding (search, analyze, references, definitions)
- **Target**: 8/10 tools real vs 1/10 currently

See @internal/NOW.md for current sprint details.

## Quick Reference

### For Development
- **Entry point**: This file (AGENTS.md)
- **Current tasks**: @internal/NOW.md
- **Roadmap**: @internal/AGENT_FIRST_ROADMAP.md (10-week agent intelligence plan)
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
- Document decisions in @internal/DECISIONS.md
- Honest assessment in @internal/PROJECT_REALITY.md

---

**Mission**: Build the smartest ACP-compatible agent backend through novel intelligence architecture and empirical validation.

**Not**: Another editor or UI - focus on agent intelligence, let frontends handle UX.
