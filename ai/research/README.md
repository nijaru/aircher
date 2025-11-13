# Research Directory

**15 research files, 5,155+ lines of SOTA analysis**

## Navigation by Implementation Phase

### Week 1-2: Memory Systems (Priority 1)
Start here for biggest competitive advantage:

**Primary**:
- `memory-system-architecture.md` (668 lines) - **READ FIRST**
  - DuckDB schema (tool_executions, file_interactions, learned_patterns)
  - ChromaDB vector search setup
  - Knowledge graph design (tree-sitter extraction)
  - Dynamic context pruning algorithm
  - 60% tool reduction validated in POC

**Supporting**:
- `agent-scaffolding.md` (171 lines) - LM-centric tool interfaces
- `tools-strategy.md` (350 lines) - Tool bundling (ripgrep + ast-grep)

### Week 3-4: Sub-Agents & Agent Workflow
**Primary**:
- `crush-subagent-architecture.md` (425 lines) - **READ FIRST**
  - Production patterns from Charm Crush
  - Tool restriction per agent type (READ/WRITE modes)
  - Session hierarchy and cost optimization
  - Small models for sub-agents

**Supporting**:
- `agent-scaffolding.md` (171 lines) - SWE-Agent patterns
- `tool-calling-reality-check.md` (127 lines) - Tool execution fixes

### Week 5-6: Benchmarking & Validation
**Primary**:
- `benchmark-integration-plan.md` (483 lines) - **READ FIRST**
  - Terminal-Bench integration (target >43.2%, stretch >58.8%)
  - SWE-bench Verified approach
  - Performance validation

**Supporting**:
- `competitive-analysis-2025.md` (573 lines) - SOTA comparison
- `week8-integration-validation.md` (289 lines) - Testing strategy

---

## Quick Lookup by Topic

### Architecture
| File | Lines | Content |
|------|-------|---------|
| memory-system-architecture.md | 668 | 3-layer memory, 60% tool reduction |
| crush-subagent-architecture.md | 425 | Sub-agent patterns, tool restriction |
| agent-scaffolding.md | 171 | LM-centric interfaces, SWE-Agent |

### Competitive Intelligence
| File | Lines | Content |
|------|-------|---------|
| competitive-analysis-2025.md | 573 | SOTA agents, evidence levels |
| sota-research-analysis-2025.md | 136 | November 2025 trends |
| discoveries.md | 325 | Key findings, insights |

### Implementation
| File | Lines | Content |
|------|-------|---------|
| benchmark-integration-plan.md | 483 | Terminal-Bench + SWE-bench |
| tool-calling-reality-check.md | 127 | Tool execution patterns |
| week8-integration-validation.md | 289 | Integration testing |

### Tools & Integration
| File | Lines | Content |
|------|-------|---------|
| tools-strategy.md | 350 | Consolidated tool analysis |
| acp-integration-analysis.md | 214 | ACP protocol implementation |
| tui-agents-sota-2025.md | 449 | Toad frontend strategy |

### Strategic Decisions
| File | Lines | Content |
|------|-------|---------|
| rust-vs-python-decision.md | 281 | Language selection rationale |
| distribution-strategy.md | 163 | Packaging and deployment |

---

## Research Quality

**Evidence Levels**:
- ✅ VERIFIED: Open source code inspected (OpenCode, Zed, Sweep)
- 📄 DOCUMENTED: Official docs/blogs (Claude Code, Cursor, Windsurf)
- ⚠️ INFERRED: User reports, consistent patterns (HN, Reddit)
- ❓ UNKNOWN: Pure speculation (marked as such)

**Coverage**:
- SOTA agents (8 analyzed: OpenCode, Claude Code, Cursor, Factory Droid, etc.)
- Memory systems (episodic, semantic, knowledge graph)
- Sub-agent patterns (production code from Crush)
- Benchmarks (Terminal-Bench, SWE-bench)
- Tools (ripgrep, ast-grep, fd, jq, sd)

**Validation**:
- 60% tool call reduction: Validated in poc-memory-agent/
- Sub-agent patterns: Production code from Charm Crush
- Competitive gaps: Open source inspection + user feedback
- Benchmark targets: Terminal-Bench leaderboard (Factory Droid 58.8%)

---

## File Descriptions

### memory-system-architecture.md (668 lines)
**Purpose**: Complete 3-layer memory system design
**Key Content**:
- DuckDB schema (5 tables with indexes, queries)
- ChromaDB vector search setup
- Knowledge graph structure (nodes: files/functions/classes, edges: calls/imports)
- Dynamic context pruning algorithm (relevance scoring)
- POC validation (60% tool reduction: 7.5 → 3.0 calls)

**When to Read**: Week 1 - Before implementing memory systems

### crush-subagent-architecture.md (425 lines)
**Purpose**: Production sub-agent patterns from Charm Crush
**Key Content**:
- Sub-agent spawning (full agents with limited tools)
- Tool restriction (READ mode: safe, WRITE mode: full)
- Cost optimization (small models for sub-agents)
- Session hierarchy (parent/child tracking)

**When to Read**: Week 3 - Before implementing sub-agent system

### competitive-analysis-2025.md (573 lines)
**Purpose**: Evidence-based SOTA agent analysis
**Key Content**:
- 8 agents analyzed (OpenCode, Claude Code, Cursor, Factory Droid, etc.)
- Evidence levels (VERIFIED, DOCUMENTED, INFERRED, UNKNOWN)
- Competitive gaps (memory, local models, transparency)
- Terminal-Bench scores (Factory Droid 58.8%, Claude Code 43.2%)

**When to Read**: Week 6 - Before benchmarking, or when understanding competitive position

### benchmark-integration-plan.md (483 lines)
**Purpose**: Terminal-Bench + SWE-bench integration strategy
**Key Content**:
- Terminal-Bench setup (ACP-based integration)
- SWE-bench Verified approach (500 tasks, human-filtered)
- Performance targets (>43.2% beat Claude Code, >58.8% beat Factory Droid)
- Optimization strategy (failure analysis, prompt tuning)

**When to Read**: Week 6 - Before running benchmarks

### tools-strategy.md (350 lines)
**Purpose**: Consolidated tool selection, bundling, usage
**Key Content**:
- Bundle: ripgrep + ast-grep (~15MB)
- Assume with fallbacks: fd, jq, sd, git
- AI benefit analysis (10-100x faster search, semantic queries)
- Implementation: ToolManager class, platform detection

**When to Read**: Week 1 - Before implementing tool framework

### agent-scaffolding.md (171 lines)
**Purpose**: LM-centric tool interfaces from SWE-Agent
**Key Content**:
- Tool design principles (windowing, limits, validation)
- Error guardrails (linting, auto-reject)
- Result formatting (max 50 results, 100-line windows)

**When to Read**: Week 2 - Before completing agent workflow

### acp-integration-analysis.md (214 lines)
**Purpose**: ACP protocol implementation details
**Key Content**:
- stdio transport (JSON-RPC over stdin/stdout)
- Session management (create, resume, end)
- Tool execution via protocol
- Streaming responses

**When to Read**: Week 5 - Before implementing ACP protocol

### tui-agents-sota-2025.md (449 lines)
**Purpose**: Toad frontend analysis and strategy
**Key Content**:
- Toad features (no flicker, interactive scrollback)
- Will McGugan creator (Rich/Textual expert)
- ACP integration (universal frontend)
- Multi-frontend strategy (Toad + Zed + Neovim)

**When to Read**: Post Week 6 - When Toad releases open source

### rust-vs-python-decision.md (281 lines)
**Purpose**: Language selection rationale
**Key Content**:
- Development velocity (3-5x faster with Python)
- LangGraph ecosystem advantages
- Tradeoffs (performance vs speed)
- Mojo future path (performance-critical components)

**When to Read**: When understanding architecture decisions

### Other Files
- `sota-research-analysis-2025.md` - November 2025 trends
- `discoveries.md` - Key findings and insights
- `distribution-strategy.md` - Packaging and deployment
- `tool-calling-reality-check.md` - Tool execution patterns
- `week8-integration-validation.md` - Integration testing

---

## Usage Pattern

1. **Check STATUS.md** - Current phase, blockers
2. **Check TODO.md** - This week's tasks
3. **Use this README** - Find relevant research
4. **Read primary file** - Detailed implementation guide
5. **Read supporting files** - Additional context
6. **Implement** - Follow research systematically
7. **Update STATUS.md** - Track progress

---

**Research is comprehensive and SOTA-aware. Gap is implementation. Follow ai/PLAN.md systematically.**
