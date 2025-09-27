# Documentation Reference Map

**Quick navigation guide to find information in the codebase**

## Core Documentation

| Topic | Location | Description |
|-------|----------|-------------|
| **Project Overview** | `CLAUDE.md` / `AGENTS.md` | AI agent context and quick reference |
| **Project Status** | `PROJECT_STATUS.md` | Current capabilities & limitations |
| **Reality Check** | `internal/PROJECT_REALITY.md` | Honest competitive assessment |
| **Current Tasks** | `internal/NOW.md` | Active priorities and sprint status |
| **Technical Spec** | `internal/TECH_SPEC.md` | Architecture and implementation details |
| **Decision History** | `internal/DECISIONS.md` | Why technical choices were made |
| **Development Status** | `internal/STATUS.md` | Development phase tracking |

## Architecture & Design

| Topic | Location | Description |
|-------|----------|-------------|
| **Agent Architecture** | `docs/architecture/agent-first-architecture.md` | Unified agent design |
| **Model vs Agent** | `docs/architecture/MODEL_VS_AGENT_ARCHITECTURE.md` | Critical architectural insight |
| **Turbo Mode Design** | `docs/architecture/turbo-mode.md` | Task orchestration architecture |
| **Intelligence Engine** | `docs/architecture/intelligence-engine.md` | AI capabilities design |

## Research & Intelligence

| Topic | Location | Description |
|-------|----------|-------------|
| **Knowledge Base** | `internal/KNOWLEDGE.md` | Patterns & learnings |
| **Discoveries** | `internal/DISCOVERIES.md` | Competitive insights & breakthroughs |

## Standards & Guidelines

| Topic | Location | Description |
|-------|----------|-------------|
| **Code Patterns** | `external/agent-contexts/standards/AI_CODE_PATTERNS.md` | Universal coding patterns |
| **Documentation Standards** | `docs/DOC_STANDARDS.md` | How to organize docs |
| **Auth System** | `docs/AUTH.md` | Authentication implementation |

## Archive

| Topic | Location | Description |
|-------|----------|-------------|
| **Old Assessments** | `docs/archive/` | Historical status files and completed plans |
| **Quick Start** | `docs/QUICK_START_DEV.md` | Developer setup guide |

## Key Code Locations

| Component | Location | Purpose |
|-----------|----------|---------|
| **TUI Manager** | `src/ui/mod.rs` | Main terminal interface |
| **Agent Controller** | `src/agent/controller.rs` | Tool execution orchestration |
| **Tool Registry** | `src/agent/tools/mod.rs` | Available tools definition |
| **Ollama Provider** | `src/providers/ollama.rs` | Local model integration |
| **Provider Manager** | `src/providers/mod.rs` | Multi-provider coordination |

## Recent Changes (2025-08-25)

1. **Tool Calling Fixed**: Ollama provider now supports tools
2. **Agent Connected**: Verified integration at `src/ui/mod.rs:3797`
3. **Documentation Updated**: Corrected inaccurate claims about disconnection

## Quick Links

- **Tests**: `tests/` - Integration and unit tests
- **Examples**: `examples/` - Usage demonstrations
- **Models**: `models/` - Embedding model files
- **Config**: `models.toml` - Model configuration

## Information Hierarchy

```
README.md         → Public introduction
AGENTS.md         → AI context (always include)
STATUS.md         → Current state
TECH_SPEC.md      → How it works
TODO.md           → What needs doing
DECISIONS.md      → Why things were done
docs/             → Detailed documentation
```

*Last Updated: 2025-08-25*