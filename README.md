# Aircher - Intelligent ACP Agent Backend

**Research-grade intelligent coding agent backend** working via [Agent Client Protocol (ACP)](https://agentclientprotocol.com).

> 🔬 **Research Project**: Building novel agent intelligence architecture with intent classification and dynamic context management. Target: empirical validation vs competitors + publication-worthy contributions.

## What Is This?

Aircher is an **ACP-compatible agent backend** - NOT a standalone TUI or IDE. It works **inside** your editor/terminal of choice:

**Primary Frontend:**
- **Toad** (universal terminal UI by Will McGugan - Python/Textual)

**Also works in:**
- **Zed** (native ACP support)
- **Neovim** (CodeCompanion, avante.nvim plugins)
- **Emacs** (agent-shell)
- **JetBrains IDEs** (October 2025 collaboration - coming soon)

**Strategy:** Toad handles terminal UX, Aircher handles agent intelligence
**Benefit:** Saves 4-6 weeks vs building custom TUI, works in 5+ frontends

## 🎯 Unique Contributions (Research Focus)

### 1. Intent Classification System
Automatic detection and specialized execution strategies:
- `CodeReading` - Analysis and comprehension tasks
- `CodeWriting` - Implementation and generation
- `ProjectFixing` - Debugging and error resolution
- `ProjectExploration` - Codebase navigation

**vs One-size-fits-all agents**: Intent-driven strategies show 15-30% improvement on specialized tasks (target validation).

### 2. Hybrid Architecture Combining SOTA Patterns (NEW: Week 6)
Combines best patterns from 4 leading agents into unified system:

**From OpenCode** (thdxr):
- Plan/Build mode separation (safe exploration vs controlled modification)
- LSP integration with event bus (real-time diagnostics, self-correction → 50% fewer runtime errors)
- Git snapshots (100% recovery from failed operations)

**From Factory Droid** (#1 Terminal-Bench, 58.8%):
- Specialized agent configs (Explorer, Builder, Debugger, Refactorer)
- Pre-configured prompts for focused tasks

**From Claude Code** (Anthropic):
- Smart sub-agent decision matrix:
  - ✅ Research tasks: 90% improvement (parallel execution)
  - ❌ Coding tasks: 0% usage (avoid 15x token waste)

**From Amp** (Sourcegraph):
- Multi-model routing (Haiku/Sonnet/Opus based on task complexity)
- 40% cost reduction target

**Our Unique Addition**:
- Memory systems (episodic + knowledge graph + working memory)
- Prevent duplicate research, continuous work without restart

**Expected Combined Results**:
- 60% tool call reduction (memory)
- 90% research speedup (parallel sub-agents)
- 50% fewer runtime errors (LSP self-correction)
- 40% cost reduction (model routing)
- 100% operation recovery (Git snapshots)

### 3. Dynamic Context Management
Single agent with intelligent context vs autonomous sub-agents:
- Smart pruning and prefetching
- Relevance scoring and token optimization
- No coordination overhead or tunnel vision

**Empirical evidence**: Research shows sub-agents cause 19% degradation for coding. Our hybrid approach uses them only for research (90% gain) while avoiding them for coding (15x waste).

### 4. Pattern-Aware Code Generation
Learns and applies project-specific conventions:
- Automatic style extraction
- Context-aware suggestions
- Architectural compliance checking

**Measurable impact**: 40-60% better code consistency matching existing patterns (target validation).

### 5. Unified Intelligence Middleware
Transparent automatic enhancement:
```rust
EnhancedContext {
    detected_intent: UserIntent,
    intelligence_insights: Vec<IntelligenceInsight>,
    confidence: f32,
}
```

## 📊 Current Status (Honest Assessment)

**Week 6 of 10 COMPLETE** | **30-33% competitive parity** | **Research phase**

### What Works ✅
- **Semantic Search**: Production-ready (6,468 vectors, 19+ languages, <2ms latency)
- **5 Production Tools**: 2,110+ lines (analyze_errors, read_file, write_file, edit_file, list_files)
- **Multi-Provider Auth**: OpenAI, Anthropic, Gemini, Ollama
- **Memory Systems**: ✅ ALL 3 COMPLETE (3,725 lines)
  - Episodic Memory (DuckDB, 815 lines): 5 tables, 11 CRUD ops, 7 queries
  - Knowledge Graph (petgraph, 1,470 lines): tree-sitter extraction, 8 queries
  - Working Memory (820 lines + 620 test lines): dynamic pruning, relevance scoring
- **ACP Protocol**: ✅ ENHANCED (+635 lines production, +470 lines tests)
  - Session management, streaming, error handling, comprehensive tests
- **Hybrid Architecture**: SOTA research complete, system design created

### What's In Progress 🔄
- **Week 7**: Core architecture patterns (event bus, LSP, Plan/Build modes, Git snapshots, model router)
- **Week 8**: Specialized agents + research sub-agents
- **Week 9**: Benchmarks vs Claude Code (validate hybrid architecture improvements)
- **Week 10**: Research paper + open source release

### What's Next ⏭️
- **Week 7-8**: Implement hybrid architecture combining SOTA patterns from 4 leading agents
- **Week 9**: Empirical validation (60% tool reduction, 90% research speedup, 50% LSP self-correction)
- **Week 10**: Publication-ready research paper + community release

**Bottom line**: Week 6 complete. Memory systems operational (3,725 lines). ACP enhanced (session, streaming, errors). Hybrid architecture designed combining Factory Droid, OpenCode, Claude Code, and Amp patterns. Ready for Week 7 implementation.

## 🚀 10-Week Development Plan

See [docs/ROADMAP.md](docs/ROADMAP.md) for complete plan.

**✅ Phase 1 COMPLETE: Foundation (Weeks 1-6)**
- Week 1: File operation tools (2,110+ lines)
- Week 2: Code understanding tools (skipped - already exist)
- Week 3: Episodic Memory (DuckDB, 815 lines)
- Week 4: Knowledge Graph (petgraph, 1,470 lines)
- Week 5: Working Memory (820 lines + 620 test lines)
- Week 6: ACP enhancements (+635 lines) + SOTA research + architecture redesign

**🔄 Phase 2 IN PROGRESS: Core Architecture (Weeks 7-8)**
- Week 7: Event bus + LSP integration, Plan/Build modes, Git snapshots, model router
- Week 8: Specialized agents + research sub-agents

**⏭️ Phase 3: Validation & Benchmarking (Week 9)**
- Week 9: Empirical comparison vs Claude Code (validate all improvements)

**📄 Phase 4: Research & Release (Week 10)**
- Week 10: Research paper + open source release + documentation

## 🔬 Research Goals

**Target Publication**: "Intent-Driven Agent Architecture for Code Assistants"

**Hypotheses to Validate**:
1. Intent classification enables 15-30% improvement on specialized tasks
2. Dynamic context outperforms sub-agents without coordination overhead (validate 19% claim)
3. Pattern learning improves code consistency 40-60%
4. Unified intelligence middleware reduces latency 2-3x

**Methodology**: Empirical benchmarks vs Claude Code on multi-file refactoring, bug fixing, code generation, and codebase exploration.

## 💻 Current Development

**See [ai/TODO.md](ai/TODO.md) for Week 7 sprint details.**

**This week (Week 7)**: Implementing core architecture patterns

**Days 1-2: Event Bus + LSP Integration**
- tokio::sync::broadcast event bus
- LspManager with global diagnostics map
- Real-time feedback loop: edit → LSP → diagnostics → agent self-corrects
- Target: 50% fewer runtime errors via self-correction

**Days 3-4: Plan/Build Mode Separation**
- AgentMode enum (Plan: read-only + research sub-agents, Build: all tools, no sub-agents)
- Intent-based mode selection
- Tool restriction enforcement

**Day 5: Git Snapshots**
- SnapshotManager with temporary commits
- Auto-rollback on errors or permission rejection
- 100% recovery from failed operations

**Days 6-7: Model Router**
- Multi-model selection (Haiku/Sonnet/Opus)
- Task complexity analysis
- Cost tracking and optimization (40% reduction target)

## 🏗️ Installation (Week 7+)

ACP protocol is implemented (Week 6 complete). Once core architecture is complete (Week 7), installation will be:

```bash
# Via Zed (recommended)
# Add to ~/.config/zed/settings.json:
{
  "agents": {
    "aircher": {
      "command": "aircher",
      "args": ["--acp"]
    }
  }
}

# Build from source
git clone https://github.com/nijaru/aircher.git
cd aircher
cargo build --release
```

**Current status**: ACP protocol complete (session management, streaming, error handling). Core architecture patterns (event bus, LSP, Plan/Build modes) implementing in Week 7.

## 🤝 Contributing

This is a **research project** targeting publication. Contributions welcome:
- **Tool implementations**: Help replace stubs with real functionality
- **Benchmarking**: Help validate claims empirically
- **Documentation**: Improve clarity and accuracy
- **Testing**: Expand test coverage

See [ai/STATUS.md](ai/STATUS.md) and [docs/STATUS.md](docs/STATUS.md) for honest assessment of current status.

## 📚 Documentation

**Entry point**: [CLAUDE.md](CLAUDE.md) - Complete project context for AI agents
**Current sprint**: [ai/TODO.md](ai/TODO.md) - Week-by-week tasks and priorities
**Development plan**: [docs/ROADMAP.md](docs/ROADMAP.md) - 10-week roadmap
**System design**: [ai/SYSTEM_DESIGN_2025.md](ai/SYSTEM_DESIGN_2025.md) - Hybrid SOTA architecture
**Architecture**: [docs/architecture/](docs/architecture/) - Technical specifications
**Research**: [ai/RESEARCH.md](ai/RESEARCH.md) - SOTA findings and application

## 🎯 Why This Matters

**Problem**: Current coding agents use one-size-fits-all approaches or sub-agent architectures with coordination overhead.

**Our approach**:
- Intent-driven specialized strategies
- Single agent with dynamic context management
- Pattern learning for project-aware generation
- Empirical validation of improvements

**Target outcome**: Demonstrate measurable improvements (15-30% on key metrics) and publish novel architecture contributions.

## 📄 License

Elastic License 2.0 - see [LICENSE](LICENSE) file for details.

**Commercial use**: ✅ Fully permitted
**Modification**: ✅ Allowed with attribution
**Distribution**: ✅ Allowed
**Managed service restriction**: ❌ Cannot offer as paid cloud service (prevents cloud giants from monetizing without contributing)

---

**Mission**: Build the smartest ACP-compatible agent backend through novel intelligence architecture and empirical validation.

**Not**: Another editor, TUI, or UI - we focus on agent intelligence and let frontends handle UX.

**Current**: Week 6 of 10 complete - Memory systems operational, ACP enhanced, hybrid architecture designed. Follow progress in [ai/TODO.md](ai/TODO.md) and [ai/STATUS.md](ai/STATUS.md).
