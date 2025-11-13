# Research Index

**Purpose**: Map research findings to implementation tasks

**Usage**: "For X, read Y" - Start here to find relevant research

---

## Quick Reference by Task

| Implementation Task | Research File | Lines | Key Findings |
|---------------------|---------------|-------|--------------|
| **Memory systems** | memory-system-architecture.md | 668 | 3-layer design, 60% tool reduction validated |
| **Sub-agents** | crush-subagent-architecture.md | 425 | Tool restriction patterns, cost optimization |
| **Competitive positioning** | competitive-analysis-2025.md | 573 | Evidence-based SOTA analysis, clear gaps |
| **Benchmarking** | benchmark-integration-plan.md | 483 | Terminal-Bench + SWE-bench integration |
| **Tool strategy** | tools-strategy.md | 350 | Bundle ripgrep+ast-grep, assume fd/jq/sd |
| **ACP protocol** | acp-integration-analysis.md | 214 | stdio transport, session management |
| **Agent scaffolding** | agent-scaffolding.md | 171 | LM-centric interfaces, SWE-Agent patterns |
| **TUI insights** | tui-agents-sota-2025.md | 449 | Toad analysis, frontend strategy |
| **Distribution** | distribution-strategy.md | 163 | Python/Mojo roadmap, packaging |
| **Python decision** | rust-vs-python-decision.md | 281 | Development velocity rationale |

---

## By Research Area

### Architecture & Design
- **memory-system-architecture.md** - DuckDB + ChromaDB + Knowledge Graph (668 lines)
- **crush-subagent-architecture.md** - Sub-agent patterns from Charm Crush (425 lines)
- **agent-scaffolding.md** - LM-centric tool interfaces (171 lines)

### Competitive Intelligence
- **competitive-analysis-2025.md** - SOTA agents with evidence levels (573 lines)
- **sota-research-analysis-2025.md** - November 2025 research trends (136 lines)
- **discoveries.md** - Key findings and insights (325 lines)

### Implementation Strategy
- **benchmark-integration-plan.md** - Terminal-Bench + SWE-bench (483 lines)
- **tool-calling-reality-check.md** - Tool calling fixes (127 lines)
- **week8-integration-validation.md** - Integration testing (289 lines)

### Tools & Integration
- **tools-strategy.md** - Consolidated tool analysis (350 lines)
- **acp-integration-analysis.md** - ACP protocol implementation (214 lines)
- **tui-agents-sota-2025.md** - Toad frontend strategy (449 lines)

### Strategic Decisions
- **rust-vs-python-decision.md** - Language selection rationale (281 lines)
- **distribution-strategy.md** - Packaging and deployment (163 lines)

---

## Key Research Findings (Summary)

### Memory Systems (60% Tool Reduction)
**Source**: memory-system-architecture.md
**Validated**: POC shows 7.5 → 3.0 calls per task
**Components**:
1. DuckDB episodic memory (tool_executions, file_interactions, learned_patterns)
2. ChromaDB vector search (semantic code retrieval)
3. Knowledge graph (tree-sitter extraction, 3,942+ nodes)

### Sub-Agent Patterns
**Source**: crush-subagent-architecture.md
**Validated**: Production code from Charm Crush
**Patterns**:
- Full agents with limited tool sets
- Small models for cost optimization
- Session hierarchy for tracking
- Tool restriction: READ mode (safe), WRITE mode (full access)

### Competitive Gaps
**Source**: competitive-analysis-2025.md
**Evidence**: Open source inspection + user feedback
**Opportunities**:
- Memory persistence (Claude Code has none)
- Local models (unlimited usage vs rate limits)
- Transparent execution (vs "flying blind")
- Dynamic context (vs restarts when full)

### Benchmarking Strategy
**Source**: benchmark-integration-plan.md
**Targets**:
- Terminal-Bench: >43.2% (beat Claude Code), stretch >58.8% (beat Factory Droid)
- SWE-bench Verified: Baseline, compare vs 75% SOTA
**Approach**: ACP-based integration, tests production interface

---

## Research Quality Assessment

**15 Files, 5,155+ Total Lines**:
- ✅ Evidence-based (open source inspected, docs verified)
- ✅ SOTA-aware (November 2025 trends included)
- ✅ Actionable (detailed schemas, algorithms, patterns)
- ✅ Well-organized (clear headers, tables, examples)

---

## Implementation Path

**Priority 1: Memory Systems** (Weeks 1-2)
- Read: memory-system-architecture.md (668 lines)
- Implement: DuckDB schema, ChromaDB integration, knowledge graph
- Validate: 60% tool reduction claim

**Priority 2: Agent Workflow** (Weeks 2-3)
- Read: agent-scaffolding.md, crush-subagent-architecture.md
- Implement: LangGraph workflow, sub-agent routing
- Validate: Tool restrictions work, cost optimization

**Priority 3: Benchmarking** (Week 6)
- Read: benchmark-integration-plan.md, competitive-analysis-2025.md
- Implement: Terminal-Bench + SWE-bench integration
- Validate: >43.2% Terminal-Bench (beat Claude Code)

---

## When to Read What

**Starting fresh agent session?**
1. Read STATUS.md first (current state)
2. Read TODO.md (active tasks)
3. Use this index to find relevant research
4. Read PLAN.md for context (if needed)

**Implementing memory systems?**
→ memory-system-architecture.md + crush-subagent-architecture.md

**Implementing agent workflow?**
→ agent-scaffolding.md + crush-subagent-architecture.md + tools-strategy.md

**Implementing benchmarks?**
→ benchmark-integration-plan.md + competitive-analysis-2025.md

**Understanding decisions?**
→ DECISIONS.md (architecture), rust-vs-python-decision.md (language choice)

---

## Refactor History (2025-11-12)

**Consolidated**: 3 tool files → tools-strategy.md, 2 competitive files → competitive-analysis-2025.md
**Updated**: PLAN.md (Python-only), TODO.md (weekly tasks), STATUS.md (blockers)
**Created**: ai/design/ directory, ai/research/tools-strategy.md
**Removed**: 5 redundant files

**Result**: 15 research files, zero redundancy, clear navigation

---

**Research Principle**: Comprehensive SOTA-aware research. Gap is implementation. Follow ai/PLAN.md systematically - memory first (biggest advantage), then agent, then benchmarks.
