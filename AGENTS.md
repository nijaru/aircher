# Aircher Project - AI Development Guide

## Project Overview

AI-powered terminal assistant built with Rust 1.80+ and Ratatui. Integrates multiple LLM providers with Model Context Protocol (MCP) for extensible tool support.

**Key Features:**
- Terminal-native TUI with responsive design
- Multi-database architecture (SQLite: conversations, knowledge, file_index, sessions)
- Universal LLM provider interface (OpenAI, Claude, Gemini, Ollama)
- Intelligent context management with file relevance scoring
- MCP tool integration for extensible capabilities

## Essential Context Files

**Always reference:**
- `docs/core/MASTER_SPEC.md` - Architecture and technical specifications
- `docs/core/DEVELOPER_GUIDE.md` - Coding standards and patterns
- `docs/core/RUST_STRUCTURE.md` - Rust project structure and key traits
- `docs/tasks/tasks.json` - **SINGLE SOURCE** for all task management



## Task Management (JSON-Based)

**Critical**: `docs/tasks/tasks.json` is the ONLY location for task tracking. Never create duplicate task lists.

**Common queries:**
```bash
# Active tasks
jq '.tasks | to_entries | map(select(.value.status == "pending"))' docs/tasks/tasks.json

# Update status
jq '.tasks["TASK-ID"].status = "in_progress"' docs/tasks/tasks.json

# Sprint tasks
jq '.current_sprint.immediate_tasks[] as $id | .tasks[$id]' docs/tasks/tasks.json
```

## Technology Stack

- Rust 1.80+ with async/await, tokio runtime
- Ratatui for TUI framework
- SQLite + sqlx for database operations
- tracing for structured logging
- TOML configuration

**Architecture:**
- Clean Architecture with trait-based design
- Multi-database
 pattern
- Provider pattern for LLM integration
- MCP protocol implementation

## File Structure

```
aircher/
├── AGENTS.md              # This file - AI navigation hub
├── docs/
│   ├── core/              # Essential project context
│   │   ├── MASTER_SPEC.md     # Complete architecture
│   │   ├── DEVELOPER_GUIDE.md # Coding standards
│   │   └── RUST_MIGRATION_PLAN.md # Migration strategy
│   ├── tasks/             # JSON task management (SINGLE SOURCE)
│   │   ├── tasks.json         # Active tasks and progress
│   │   └── completed.json     # Completed task archive
│   ├── architecture/      # Technical specifications (tool-based access)
│   ├── development/       # Development resources (tool-based access)
│   └── reference/         # Quick access materials (tool-based access)
├── src/                   # Rust source code
└── examples/              # Usage examples
```

## Task-to-Documentation Lookup

| Task Type | Primary Source | Supporting |
|-----------|---------------|------------|
| **Architecture** | `docs/core/MASTER_SPEC.md` | `docs/core/DEVELOPER_GUIDE.md` |
| **CLI Commands** | `docs/architecture/commands/` | `docs/core/MASTER_SPEC.md` |
| **TUI Components** | `docs/architecture/output/` | `docs/core/DEVELOPER_GUIDE.md` |
| **LLM Integration** | `docs/architecture/plugins/` | `docs/core/MASTER_SPEC.md` |
| **Database** | `docs/architecture/` | `docs/core/MASTER_SPEC.md` |
| **Testing** | `docs/development/workflow/` | `docs/reference/validation/` |

## AI Workflow

1. **Check Tasks**: Review `docs/tasks/tasks.json` for priorities
2. **Minimal Context**: Include only task-relevant core documents
3. **Dynamic Access**: Use tools for architecture/development/reference docs
4. **Update Progress**: Mark status in `docs/tasks/tasks.json` (pending → in_progress → completed)
5. **Validate**: Check `docs/reference/validation/` requirements

## Documentation Maintenance

**Efficiency Principles:**
- **Token Conservation**: Be concise while preserving critical information
- **Implementation Focus**: Document only what exists, not future plans
- **Single Source**: Each concept has one authoritative location
- **Essential Only**: Capture requirements and constraints, not obvious details

**Key Guidelines:**
- Remove outdated technology references immediately
- Consolidate duplicate information across files
- Focus on decisions and constraints, not explanations
- Update task status in `tasks.json` every session
- Keep core files (`MASTER_SPEC.md`, `DEVELOPER_GUIDE.md`) current with architectural changes

## Current Status

**Phase**: Foundation & LLM Integration
**Priority**: Build robust Rust-based AI terminal assistant

**Immediate Tasks (Current Sprint):**
- OpenAI API Integration with streaming (`SPRINT-001`)
- Claude API Integration with streaming (`SPRINT-002`)
- CLI Authentication System (`SPRINT-003`)

**Build Commands:**
```bash
cargo build --release   # Build binary
cargo test              # Run tests
cargo clippy            # Run linting
```

## Key Rules

1. **Single Source**: `docs/tasks/tasks.json` ONLY for task tracking
2. **No Duplication**: Never create tasks in other files
3. **Always Update**: Mark progress in tasks.json every session
4. **Tool Access**: Use tools for non-core documentation
5. **Quality Focus**: Follow Rust best practices and maintain high code quality

---

**Start every session**: Check `docs/tasks/tasks.json` → Reference task lookup table → Update progress in tasks.json