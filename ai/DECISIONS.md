# Aircher Decisions

## 2025-11-12: Python over Rust for Agent Backend

**Context**: Architecture migration decision after Rust prototype analysis
**Decision**: Python with LangGraph framework
**Rationale**:
- Development velocity: Python ecosystem 3-5x faster for agent development
- LangGraph: Production-validated agent framework with built-in state management
- Library ecosystem: Rich AI/ML tooling (sentence-transformers, ChromaDB, etc.)
- Team expertise: Strong Python skills vs learning Rust async patterns

**Tradeoffs**:
| Pro | Con |
|-----|-----|
| Faster development cycles | Lower raw performance |
| Rich AI ecosystem | GIL limitations for CPU work |
| LangGraph state management | Memory overhead |
| Easier testing/debugging | Deployment complexity |

**Evidence**: ai/research/python_vs_rust_analysis.md
**Commits**: Phase 2 complete setup

---

## 2025-11-12: Multi-Database Strategy

**Context**: Memory system architecture design
**Decision**: SQLite + DuckDB + ChromaDB
**Rationale**:
- **SQLite**: Proven, embedded, ACID compliance for session data
- **DuckDB**: Columnar analytics, complex queries for episodic memory
- **ChromaDB**: Specialized vector database with embedding support

**Tradeoffs**:
| Pro | Con |
|-----|-----|
| Right tool for each job | Complexity of 3 systems |
| Optimized performance | Data synchronization overhead |
| Proven technologies | Learning curve |

**Evidence**: ai/research/database_strategy_analysis.md

---

## 2025-11-12: Custom ACP Implementation

**Context**: agent-protocol Python package conflicts with pydantic v2
**Decision**: Implement custom ACP protocol
**Rationale**:
- Avoid dependency conflicts with modern pydantic
- Full control over protocol features and extensions
- Better integration with our architecture
- Learning opportunity for protocol implementation

**Tradeoffs**:
| Pro | Con |
|-----|-----|
| No dependency conflicts | More implementation work |
| Custom extensions possible | Maintenance responsibility |
| Full control | Need to ensure compatibility |

**Evidence**: Dependency resolution failures with agent-protocol package

---

## 2025-11-12: READ/WRITE + --admin Mode System

**Context**: Safety and usability requirements
**Decision**: READ/WRITE modes with optional --admin flag
**Rationale**:
- **READ**: Safe exploration, file reading only
- **WRITE**: File modifications with confirmation
- **--admin**: Full access without confirmations (overrides config)

**Tradeoffs**:
| Pro | Con |
|-----|-----|
| Intuitive terminology | Admin flag adds complexity |
| Clear permission model | Need --no-admin override |
| Progressive trust | Implementation overhead |

**Evidence**: User experience analysis, comparison with Claude Code/opencode patterns

---

## 2025-11-12: Modern Python Tooling

**Context**: Development environment setup
**Decision**: uv + ruff + ty + vulture + pytest
**Rationale**:
- **uv**: Fast package manager, resolves dependencies 10-100x faster
- **ruff**: Rust-based linting/formatting, 50-100x faster than traditional tools
- **ty**: Type checking from uv creators (replaces mypy)
- **vulture**: Dead code detection
- **pytest**: De facto testing standard with asyncio support

**Tradeoffs**:
| Pro | Con |
|-----|-----|
| Modern, fast tooling | Learning curve for team |
| Integrated workflows | Potential compatibility issues |
| Best practices | Dependency on newer tools |

**Evidence**: Modern Python development best practices research

---

## 2025-11-12: Phased TUI Approach

**Context**: Frontend architecture decision
**Decision**: CLI now, Toad integration later
**Rationale**:
- **Phase 3-4**: Focus on agent backend, simple CLI interface
- **Phase 5+**: Integrate Toad when released and stable
- **Priority**: Agent functionality over UI development
- **Resources**: Focus on core systems first

**Tradeoffs**:
| Pro | Con |
|-----|-----|
| Faster to market | Limited UI initially |
| Focus on core | Later integration work |
| Toad maturity | Wait for release |

**Evidence**: Development timeline analysis, Toad release status

---

## 2025-11-12: Python 3.13+ Minimum

**Context**: Python version selection
**Decision**: Target Python 3.13+ instead of 3.12+
**Rationale**:
- **Performance**: 3.13+ has significant improvements
- **Dependencies**: All major deps support 3.13+
- **Future-proof**: 3.14 available, most libs compatible
- **Modern syntax**: Latest language features

**Tradeoffs**:
| Pro | Con |
|-----|-----|
| Latest performance | Reduced compatibility |
| Modern syntax | Fewer supported systems |
| Future-proof | Newer runtime requirements |

**Evidence**: Dependency compatibility analysis, Python 3.13 performance benchmarks

---

## 2025-11-12: Bash/Code Tools over MCP

**Context**: Tool architecture philosophy
**Decision**: Simple bash/code tools instead of MCP servers
**Rationale**:
- **Context efficiency**: 225 tokens vs 13-18k tokens for MCP
- **Composability**: Tools can be chained, results saved to files
- **Flexibility**: Easy to modify/extend tools
- **Performance**: Direct execution vs protocol overhead

**Evidence**: pi-mono browser tools analysis, MCP token usage benchmarks

---

## 2025-11-12: Modern Tools Integration

**Context**: Tool selection for agent operations
**Decision**: Assume modern tools, fallback to standard tools
**Rationale**:
- **Performance**: ripgrep, fd, sd significantly faster
- **User experience**: Most developers have these tools
- **Fallback**: Graceful degradation to grep, find, sed
- **Structured data**: nushell for complex data processing

**Tool Strategy**:
- **Essential**: ripgrep, fd, sd, jq (assume, fallback available)
- **Optional**: ast-grep, nushell, bat, delta (detect, use if available)
- **Python-based**: tree-sitter, PyYAML, toml (always available)

**Evidence**: Modern tooling performance benchmarks, developer tooling surveys

---

## 2025-11-12: Python/Mojo Long-term Stack

**Context**: Language stack evolution planning
**Decision**: Python now, Mojo integration later
**Rationale**:
- **Current**: Python ecosystem unmatched for AI/ML
- **Performance**: Mojo for critical paths when 1.0 released
- **Interop**: Mojo-Python interop is excellent
- **Timeline**: Mojo 1.0 expected summer 2025

**Migration Strategy**:
- **Phase 3-4**: Pure Python development
- **Phase 5+**: Identify performance bottlenecks
- **Phase 6+**: Mojo for critical components
- **Package Manager**: Stick with uv, can integrate with pixi later

**Evidence**: Mojo development roadmap, Python-Mojo interop analysis

---

## 2025-11-13: LangGraph over Pydantic AI

**Context**: Framework evaluation after Week 1-2 memory system implementation
**Decision**: Continue with LangGraph, avoid hybrid approach
**Rationale**:
- **Multi-agent requirements**: Week 3-4 roadmap requires CodeReading, CodeWriting, ProjectFixing sub-agents
- **State management**: 3-layer memory system needs LangGraph's sophisticated checkpointing and persistent state
- **Already invested**: Memory systems fully integrated with LangGraph workflows
- **Production readiness**: Built-in fault tolerance, durable execution, human-in-the-loop
- **Complexity fits**: Graph-based architecture handles context pruning, model routing, dynamic decisions
- **Maintenance**: Single framework simpler than hybrid LangGraph + Pydantic AI approach

**Pydantic AI Strengths Considered**:
- Type safety and IDE support
- Performance (faster than LangGraph in benchmarks)
- Simpler API for single agents
- OpenTelemetry observability

**Why Not Pydantic AI**:
- Not designed for complex multi-agent orchestration
- Migration cost too high at this stage
- Less mature state management for complex workflows
- Would need custom orchestration layer

**Tradeoffs**:
| Pro | Con |
|-----|-----|
| Built for multi-agent systems | More complex API |
| Sophisticated state management | Slower performance |
| Already integrated | Larger learning curve |
| Production-grade features | More abstraction layers |

**Future Considerations**:
- Evaluate Pydantic models for LLM output validation
- Consider Pydantic Logfire for observability (Week 5)
- Use type hints following Pydantic patterns

**Evidence**: Web research, framework comparison analysis, roadmap requirements
**Impact**: Week 3-6 development continues with LangGraph
