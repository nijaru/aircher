# Aircher Documentation

Welcome to the Aircher documentation system - a revolutionary AI-optimized documentation structure designed for efficient development and maintenance.

## Quick Navigation

### 🎯 **Start Here for AI Agents**
- **[`AGENTS.md`](../AGENTS.md)** - Primary AI navigation hub with task lookup table and workflow protocols

### 📋 **Essential Project Context**
- **[`docs/core/MASTER_SPEC.md`](core/MASTER_SPEC.md)** - Complete project architecture and technical specifications
- **[`docs/core/DEVELOPER_GUIDE.md`](core/DEVELOPER_GUIDE.md)** - Coding standards and implementation patterns
- **[`docs/tasks/tasks.json`](tasks/tasks.json)** - **SINGLE SOURCE** of truth for all task management

### 🔄 **Task Management System**
- **[`docs/tasks/`](tasks/)** - JSON-based task tracking (revolutionary approach)
  - [`tasks.json`](tasks/tasks.json) - Active tasks and progress tracking
  - [`completed.json`](tasks/completed.json) - Historical task archive
  - [`README.md`](tasks/README.md) - Task system usage guide

## Documentation Architecture

This project implements a **hierarchical, tool-optimized documentation pattern** with these key principles:

### AI-First Design
- **Minimal Initial Context**: Core documents provide essential knowledge
- **Tool-Based Access**: Detailed specifications accessed dynamically via AI tools
- **JSON Task Management**: Structured data enabling programmatic updates
- **Single Sources of Truth**: Eliminate duplication, maintain consistency

### Progressive Information Loading
```
AGENTS.md → Core Context → Architecture Details → Development Resources → Reference Materials
    ↓            ↓              ↓                    ↓                      ↓
Navigation    Essential      Component-Specific   Setup & Workflow     Quick Access
   Hub        Knowledge      Technical Specs      Documentation        Patterns
```

## Directory Structure

```
docs/
├── README.md                 # 📍 This file - Documentation navigation
├── core/                     # 📋 Essential Project Context (Always Load)
│   ├── MASTER_SPEC.md       # Complete architecture & technical specs
│   ├── DEVELOPER_GUIDE.md   # Coding standards & implementation patterns
│   ├── PROJECT_ROADMAP.md   # Feature roadmap & implementation phases
│   └── RUST_MIGRATION_PLAN.md # Migration from Go to Rust strategy
├── tasks/                    # 🔄 JSON Task Management (SINGLE SOURCE)
│   ├── tasks.json           # Active tasks, progress, metrics
│   ├── completed.json       # Historical task archive
│   └── README.md            # Task system guide
├── architecture/             # 🏗️ CLI-Specific Technical Architecture
│   ├── commands/            # CLI command specifications
│   ├── config/              # Configuration system architecture
│   ├── output/              # TUI rendering & response streaming
│   ├── plugins/             # MCP & LLM provider integration
│   └── storage-architecture.md # Database design & patterns
├── development/              # 🔧 Development Resources
│   ├── setup/               # Environment setup & dependencies
│   ├── workflow/            # Git workflow, testing, CI/CD
│   └── troubleshooting/     # Common issues & debugging
└── reference/                # 📚 Quick Access Materials
    ├── patterns/            # Code examples & best practices
    ├── commands/            # Build, test, debug command reference
    └── validation/          # Quality gates & requirement checklists
```

## Documentation Types and Usage

### 🎯 Entry Points (Start Here)
| File | Purpose | Audience |
|------|---------|----------|
| **[`AGENTS.md`](../AGENTS.md)** | AI agent navigation hub | AI Development Tools |
| **[`README.md`](../README.md)** | Project overview & installation | End Users |
| **[`docs/README.md`](README.md)** | Documentation navigation | Developers |

### 📋 Core Context (Always Reference)
| File | Purpose | When to Use |
|------|---------|-------------|
| **[`MASTER_SPEC.md`](core/MASTER_SPEC.md)** | Complete technical architecture | All development work |
| **[`DEVELOPER_GUIDE.md`](core/DEVELOPER_GUIDE.md)** | Coding standards & patterns | Implementation decisions |
| **[`tasks.json`](tasks/tasks.json)** | All task tracking & progress | Every development session |

### 🏗️ Architecture Documentation (Tool-Based Access)
| Directory | Contents | Access Pattern |
|-----------|----------|----------------|
| **[`architecture/commands/`](architecture/commands/)** | CLI command specs | When implementing commands |
| **[`architecture/config/`](architecture/config/)** | Configuration system | When working with config |
| **[`architecture/output/`](architecture/output/)** | TUI & streaming architecture | When building UI components |
| **[`architecture/plugins/`](architecture/plugins/)** | LLM & MCP integration | When adding providers/tools |

### 🔧 Development Resources (As Needed)
| Directory | Contents | When to Access |
|-----------|----------|----------------|
| **[`development/setup/`](development/setup/)** | Environment configuration | New developer onboarding |
| **[`development/workflow/`](development/workflow/)** | Git, testing, CI/CD | Process questions |
| **[`development/troubleshooting/`](development/troubleshooting/)** | Common issues & fixes | When problems occur |

### 📚 Reference Materials (Quick Access)
| Directory | Contents | Usage |
|-----------|----------|-------|
| **[`reference/commands/`](reference/commands/)** | Build & debug commands | Development workflow |
| **[`reference/patterns/`](reference/patterns/)** | Code examples | Implementation guidance |
| **[`reference/validation/`](reference/validation/)** | Quality checklists | Before committing code |

## JSON Task Management Revolution

### Why JSON Instead of Markdown?
- **Programmatic Updates**: AI agents can update status without merge conflicts
- **Structured Queries**: Filter by priority, phase, status using `jq`
- **Historical Tracking**: Complete audit trail in `completed.json`
- **Metrics Integration**: Automatic progress calculation and reporting
- **Multi-Agent Support**: Consistent structure across different AI tools

### Common Task Queries
```bash
# Get current high-priority tasks
jq '.tasks | to_entries | map(select(.value.priority == "critical" and .value.status != "completed"))' tasks/tasks.json

# Show current sprint tasks
jq '.current_sprint.immediate_tasks[] as $id | .tasks[$id]' tasks/tasks.json

# Update task status
jq '.tasks["TASK-ID"].status = "in_progress"' tasks/tasks.json

# Mark acceptance criterion complete
jq '.tasks["TASK-ID"].acceptance_criteria["criterion"].completed = true' tasks/tasks.json
```

## AI Agent Workflow Protocol

### 1. Session Initialization
1. **Start**: Read [`AGENTS.md`](../AGENTS.md) for navigation and current context
2. **Tasks**: Check [`tasks/tasks.json`](tasks/tasks.json) for priorities and progress
3. **Context**: Load only task-relevant core documents initially
4. **Architecture**: Fetch specific technical docs via tools as needed

### 2. Task Execution
1. **Select**: Choose task from `tasks.json` (ONLY task source)
2. **Research**: Use task→documentation lookup table in `AGENTS.md`
3. **Implement**: Follow patterns in `DEVELOPER_GUIDE.md`
4. **Update**: Mark progress in `tasks.json` using JSON structure

### 3. Session Completion
1. **Progress**: Update task status and acceptance criteria
2. **Metrics**: Refresh code statistics and progress percentages
3. **Archive**: Move completed tasks to `completed.json`
4. **Validate**: Check quality gates in `reference/validation/`

## Technology Stack Context

### Current (Rust Implementation)
- **Language**: Rust 1.80+ with async/await
- **TUI Framework**: Ratatui for terminal interface
- **Async Runtime**: Tokio for concurrent operations
- **Database**: SQLite with rusqlite/sqlx
- **Logging**: tracing crate for structured logging
- **Configuration**: TOML format (avoid JSON/YAML)

### Migration Status
Currently migrating from Go→Rust. See [`core/RUST_MIGRATION_PLAN.md`](core/RUST_MIGRATION_PLAN.md) for strategy and progress.

## Key Differentiators

This documentation system provides:
- **Context Minimization**: Avoid overwhelming AI context windows
- **Tool-First Approach**: Dynamic access to detailed specifications
- **Single Sources of Truth**: Eliminate information duplication
- **Programmatic Task Management**: JSON structure enabling automation
- **Multi-Agent Support**: Consistent interface for different AI tools

## Quality Standards

All documentation follows these principles:
- **Accuracy**: Information is current and verified
- **Accessibility**: Clear navigation and progressive disclosure
- **Actionability**: Specific guidance for implementation
- **Maintainability**: Single sources of truth, no duplication
- **Tool-Friendly**: Structured data and clear access patterns

## Contributing to Documentation

### Adding New Documentation
1. **Identify Category**: Core, Architecture, Development, or Reference
2. **Check Existing**: Ensure no duplication with existing docs
3. **Update Navigation**: Add entry to this README and `AGENTS.md`
4. **Cross-Reference**: Link from relevant task lookup table
5. **Validate Structure**: Ensure follows documentation principles

### Updating Task Management
1. **Use JSON Only**: Never create tasks in Markdown files
2. **Programmatic Updates**: Prefer `jq` over manual JSON editing
3. **Maintain History**: Archive completed tasks with completion details
4. **Update Metrics**: Keep project statistics current
5. **Validate Structure**: Ensure JSON remains valid after changes

---

**Remember**: This documentation system is optimized for AI-assisted development while maintaining human usability. Always start with `AGENTS.md` for navigation and use `tasks/tasks.json` as the single source of truth for all project management.