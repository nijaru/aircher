# Documentation Reference Map

**Quick navigation guide to find information in the codebase**

## Core Documentation

| Topic | Location | Description |
|-------|----------|-------------|
| **Project Overview** | `CLAUDE.md` | AI agent entry point and quick reference |
| **Public Status** | `docs/STATUS.md` | User-facing capabilities & limitations |
| **Current Tasks** | `ai/TODO.md` | Active tasks and sprint status |
| **Current State** | `ai/STATUS.md` | What works/doesn't, known issues |
| **Technical Spec** | `docs/TECH_SPEC.md` | Architecture and implementation details |
| **Decision History** | `ai/DECISIONS.md` | Why technical choices were made (append-only) |
| **Research Index** | `ai/RESEARCH.md` | External research findings and application |
| **Development Roadmap** | `docs/ROADMAP.md` | 10-week development plan |

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
| **Research Index** | `ai/RESEARCH.md` | External research findings |
| **Agent Scaffolding** | `ai/research/agent-scaffolding.md` | SWE-Agent patterns & LM-centric interfaces |
| **Discoveries** | `internal/DISCOVERIES.md` | Important findings and learnings |
| **Knowledge Base** | `internal/KNOWLEDGE.md` | Competitive intelligence & patterns |

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