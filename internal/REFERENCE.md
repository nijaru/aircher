# Documentation Reference Map

**Quick navigation guide to find information in the codebase**

## Core Documentation

| Topic | Location | Description |
|-------|----------|-------------|
| **Project Overview** | `AGENTS.md` | AI agent context and quick reference |
| **Current Status** | `STATUS.md` | Development state, metrics, issues |
| **Technical Spec** | `TECH_SPEC.md` | Architecture and implementation details |
| **Active Tasks** | `TODO.md` | Current priorities and upcoming work |
| **Decision History** | `DECISIONS.md` | Why technical choices were made |
| **Tool Calling Status** | `TOOL_CALLING_REALITY_CHECK.md` | Empirical test results (Aug 2025) |

## Architecture & Design

| Topic | Location | Description |
|-------|----------|-------------|
| **Development Roadmap** | `docs/architecture/roadmap.md` | Phased development plan |
| **Turbo Mode Design** | `docs/architecture/turbo-mode.md` | Task orchestration architecture |
| **Intelligence Engine** | `docs/architecture/intelligence-engine.md` | AI capabilities design |

## Standards & Guidelines

| Topic | Location | Description |
|-------|----------|-------------|
| **Documentation Standards** | `docs/DOC_STANDARDS.md` | How to organize docs |
| **Code Standards** | `docs/CODE_STANDARDS.md` | Coding conventions |
| **Auth System** | `docs/AUTH.md` | Authentication implementation |
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