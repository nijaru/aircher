# Aircher Project - AI Development Guide

## 🚨 CRITICAL RULES - ALWAYS FOLLOW 🚨

### ✅ ALWAYS
- **TASK_TRACKING**: Use `docs/tasks/tasks.json` ONLY - never create duplicate task lists
- **ARCHITECTURE_CHECK**: Verify changes against `docs/core/MASTER_SPEC.md` before implementation
- **VALIDATION**: Run `cargo clippy` and `cargo test` before any commit
- **PROGRESS_UPDATE**: Mark task status in `tasks.json` every session (pending → in_progress → completed)
- **CONTEXT_EFFICIENCY**: Load only task-relevant documentation to optimize tokens

### ❌ NEVER
- **DUPLICATE_TASKS**: Create tasks in files other than `docs/tasks/tasks.json`
- **SKIP_VALIDATION**: Commit without running tests and linting
- **INCOMPLETE_TASKS**: Leave tasks marked as `in_progress` when completed
- **TECH_DEBT**: Implement quick fixes without considering architecture patterns
- **UNSAFE_CODE**: Add unsafe Rust code without thorough documentation and review

## Project Context

### What is Aircher?
**"Dual-architecture AI development system with hybrid performance design"**
- **Aircher Terminal**: Advanced terminal-native TUI built in Rust with streaming LLM integration
- **Aircher Intelligence Engine**: Universal MCP server built in Python with Rust performance modules
- **Target Users**: Developers seeking AI-enhanced workflows across all tools
- **Key Differentiators**: Cross-project learning, universal compatibility, intelligent context assembly, modular performance optimization

### Current Status: Foundation & Intelligence Engine
**Focus**: Build dual-architecture system with terminal assistant and Python MCP server
- LLM Provider Integration (Rust Terminal) (`SPRINT-001`)
- Python MCP Intelligence Server with Rust performance modules (`SPRINT-002`) 
- Context Management Engine with modular backends (`SPRINT-003`)
- **Success Metrics**: Working terminal chat, universal MCP compatibility, intelligent context scoring, 10x performance improvements

## Task Management & Autonomous Work

### 🎯 Current Focus: "Foundation & LLM Integration"

**Next Task Priority Sequence**: SPRINT-001 → SPRINT-002 → SPRINT-003

**To work autonomously**: Check `docs/tasks/tasks.json` for:
1. **Current task**: Look for `"next_sequence"` array or tasks with `"status": "pending"`
2. **Task details**: `description`, `files`, `acceptance_criteria`, `dependencies`
3. **Required docs**: Use task-specific documentation mapping below
4. **Completion**: Update status, move completed tasks to `completed.json`

### Task Commands
```bash
# View active tasks
jq '.tasks | to_entries | map(select(.value.status == "pending"))' docs/tasks/tasks.json

# Update task status
jq '.tasks["TASK-ID"].status = "in_progress"' docs/tasks/tasks.json > tmp.json && mv tmp.json docs/tasks/tasks.json

# View current sprint
jq '.priorities.current_focus' docs/tasks/tasks.json
```

## Essential Sources

### Task-Specific Documentation
| Task Type | Primary Doc | Supporting | Notes |
|-----------|-------------|------------|-------|
| **Core Development** | `docs/core/MASTER_SPEC.md` | `docs/core/DEVELOPER_GUIDE.md` | System architecture |
| **CLI Commands** | `docs/architecture/commands/` | `docs/core/MASTER_SPEC.md` | Command specifications |
| **TUI Components** | `docs/architecture/output/` | `docs/core/DEVELOPER_GUIDE.md` | UI patterns |
| **LLM Integration** | `docs/architecture/plugins/llm-providers.md` | `docs/core/MASTER_SPEC.md` | Provider interfaces |
| **MCP Server** | `docs/architecture/plugins/aircher-intelligence-mcp-server.md` | `docs/architecture/plugins/mcp-integration.md` | Python intelligence engine |
| **Performance Architecture** | `docs/architecture/plugins/modular-performance-architecture.md` | `docs/core/MASTER_SPEC.md` | Modular Python/Rust backends |
| **Context Management** | `docs/architecture/plugins/context-management.md` | `docs/core/MASTER_SPEC.md` | Intelligence algorithms |
| **Database Operations** | `docs/architecture/storage-architecture.md` | `docs/core/MASTER_SPEC.md` | Multi-DB patterns |
| **Testing** | `docs/development/workflow/` | `docs/reference/validation/` | Test strategies |
| **Configuration** | `docs/architecture/config/` | `docs/core/DEVELOPER_GUIDE.md` | TOML specs |

### Core References (Always Available)
- `docs/core/MASTER_SPEC.md` - Complete technical architecture
- `docs/core/DEVELOPER_GUIDE.md` - Coding standards and patterns
- `docs/core/RUST_STRUCTURE.md` - Rust project structure and key traits
- `docs/core/GLOSSARY.md` - Domain terminology and concepts

## Development Essentials

### Quick Commands
```bash
cargo build --release    # Build optimized binary
cargo test               # Run all tests
cargo clippy            # Run linting
cargo fmt               # Format code
```

### Conventions
**Naming**: `snake_case` | **Modules**: `pub mod feature_name` | **Files**: `feature_name.rs`

### Core Patterns
- **Provider Pattern**: Trait-based LLM provider abstraction
- **Multi-Database**: Separate SQLite DBs for different concerns
- **Async-First**: Tokio runtime with streaming support
- **Clean Architecture**: Domain-driven design with clear boundaries

## Technology Stack

**Terminal Client (Rust):**
- Rust 1.80+ with async/await, tokio runtime
- Ratatui for terminal UI framework
- SQLite + sqlx for database operations
- tracing for structured logging
- TOML configuration system

**Intelligence Engine (Python + Rust):**
- Python 3.11+ with asyncio, MCP protocol
- Rust performance modules via PyO3 bindings
- numpy/scipy for vector operations
- sentence-transformers for embeddings
- tree-sitter for AST parsing (Rust-accelerated)

**Architecture Principles:**
- Clean Architecture with trait-based design (Rust) and protocol-based design (Python)
- Dual-architecture: Rust terminal assistant + Python MCP server with Rust performance cores
- Multi-database pattern (conversations, knowledge, file_index, sessions)
- Provider pattern for LLM integration
- Modular backend architecture for language-agnostic performance optimization
- Intelligent context management with cross-project learning

## Documentation Structure

```
aircher/
├── CLAUDE.md                    # AI agent hub (THIS FILE)
├── docs/
│   ├── core/                    # Fundamental project documents
│   │   ├── MASTER_SPEC.md       # Technical architecture overview
│   │   ├── DEVELOPER_GUIDE.md   # Development workflows and patterns
│   │   ├── RUST_STRUCTURE.md    # Rust project structure
│   │   └── GLOSSARY.md          # Domain terminology
│   ├── tasks/                   # Task management system
│   │   ├── tasks.json           # Active tasks with priorities
│   │   └── completed.json       # Archived completed tasks
│   ├── architecture/            # Technical design documents
│   │   ├── system/              # High-level system design
│   │   ├── components/          # Individual component specs
│   │   ├── integration/         # Inter-component relationships
│   │   ├── commands/            # CLI command specifications
│   │   ├── config/              # Configuration system
│   │   ├── output/              # TUI design and patterns
│   │   └── plugins/             # LLM providers and MCP
│   ├── business/                # Strategic and business context
│   ├── development/             # Developer resources
│   │   ├── setup/               # Environment and tooling
│   │   ├── troubleshooting/     # Common issues and solutions
│   │   └── workflow/            # Development processes
│   ├── reference/               # Quick lookup resources
│   │   ├── commands/            # Command reference
│   │   ├── patterns/            # Code patterns and examples
│   │   └── validation/          # Quality gates and checklists
│   └── research/                # Experimental and background
├── src/                         # Rust source code
├── tests/                       # Test suites
├── examples/                    # Usage demonstrations
└── scripts/                     # Automation and utilities
```

## Workflow

### AI Agent Workflow
1. **Check Tasks**: Review `docs/tasks/tasks.json` for current priorities
2. **Load Context**: Reference task-specific documentation from mapping table
3. **Implement**: Follow Rust best practices and architectural patterns
4. **Validate**: Run `cargo clippy` and `cargo test`
5. **Update Progress**: Mark status in `docs/tasks/tasks.json`
6. **Document**: Update relevant architecture docs if needed

### Error Recovery
1. **Compilation Issues**: Check `docs/development/troubleshooting/`
2. **Architecture Questions**: Reference `docs/core/MASTER_SPEC.md`
3. **Pattern Confusion**: Check `docs/reference/patterns/`
4. **Test Failures**: Review `docs/reference/validation/`

---

**Start every session**: Check `docs/tasks/tasks.json` → Reference task lookup table → Update progress in tasks.json