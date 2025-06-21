# Aircher Project - AI Development Guide

## Project Overview

Aircher is an AI-powered terminal-based development assistant that provides intelligent conversation, context management, and project analysis capabilities. Built with Go 1.24+ and Charmbracelet's Bubble Tea TUI framework, it integrates multiple LLM providers and implements the Model Context Protocol (MCP) for extensible tool support.

**Key Differentiators:**
- Terminal-native interface with responsive design
- Multi-database architecture for different data types
- Universal LLM provider interface (OpenAI, Claude, Gemini, Ollama)
- Intelligent context management and file relevance scoring
- Project analysis system with auto-generated documentation

## Documentation Organization Principles

This project follows a hierarchical documentation pattern optimized for AI-assisted development:

### Context Minimization Strategy
- **Start Minimal**: Include only essential files (`docs/core/`) in initial AI context
- **Fetch Dynamically**: Use tools to access `docs/technical/` and `docs/external/` as needed
- **Single Source of Truth**: Avoid information duplication across documentation files
- **Tool-First Approach**: Leverage AI tools rather than overwhelming context windows

### Hierarchical Information Architecture
- **Navigation Hub**: This AGENTS.md file provides routing to all other documentation
- **Core Context**: `docs/core/` contains essential project knowledge
- **Technical Details**: `docs/technical/` contains component-specific implementations
- **External References**: `docs/external/` holds supporting materials

### Centralized Task Management
- **Single Task Source**: `docs/core/TASKS.md` is the ONLY location for task tracking
- **No Task Duplication**: Never create tasks in multiple files - always reference TASKS.md
- **Progress Updates**: All task completion status goes in TASKS.md only
- **Metrics Integration**: Project metrics and progress tracking centralized in TASKS.md

## Essential Context Files

**Always reference these core documents:**
- `docs/core/MASTER_SPEC.md` - Project architecture and technical specifications
- `docs/core/DEVELOPER_GUIDE.md` - Coding standards and implementation patterns
- `docs/core/TASKS.md` - **SINGLE SOURCE** for all task management, priorities, progress, and metrics

**Project status and roadmap:**
- `docs/core/PROJECT_ROADMAP.md` - Feature roadmap and implementation phases
- `docs/core/TASKS.md` - **CENTRALIZED** progress tracking and task completion status

## Tool-Based Access Files

**Access these via tools when needed (not in initial context):**
- `docs/technical/01-ui-improvements.md` - TUI enhancement specifications
- `docs/technical/07-configuration-architecture.md` - Configuration system technical specification
- `docs/config/mvp-config-spec.toml` - MVP configuration planning and specifications
- `docs/config/credentials-spec.toml` - API key management and `aircher login` command specs
- `README.md` - Usage examples and installation guide
- `go.mod` / `go.sum` - Dependencies and tool management
- `Makefile` - Build commands and development workflows
- `.aircher/project_analysis.md` - Auto-generated project analysis
- Source code files in `internal/`, `cmd/`, `examples/`
## Task-to-Documentation Lookup

| Development Task | Primary Source | Supporting References |
|------------------|----------------|----------------------|
| **Architecture & Design** | `docs/core/MASTER_SPEC.md` | `docs/core/DEVELOPER_GUIDE.md` |
| **TUI Components** | `docs/technical/01-ui-improvements.md` | `docs/core/DEVELOPER_GUIDE.md` |
| **LLM Provider Integration** | `docs/technical/04-llm-providers.md` | `docs/core/MASTER_SPEC.md` |
| **Database Operations** | `docs/technical/03-storage-architecture.md` | `docs/core/MASTER_SPEC.md` |
| **Context Management** | `docs/technical/05-context-management.md` | `docs/core/MASTER_SPEC.md` |
| **MCP Tool Integration** | `docs/technical/06-mcp-integration.md` | `docs/core/MASTER_SPEC.md` |
| **Testing & Quality** | `docs/core/DEVELOPER_GUIDE.md` | Test files throughout codebase |
| **Project Analysis** | `docs/core/MASTER_SPEC.md` | `internal/analyzer/` |
| **Configuration** | `docs/technical/07-configuration-architecture.md` | `docs/config/mvp-config-spec.toml`, `docs/config/credentials-spec.toml` |

## Development Guidelines

### Technology Stack
- **Go 1.24+** with tool management, os.Root, Swiss Tables
- **Charmbracelet Bubble Tea** for TUI framework
- **SQLite + sqlx** for database operations
- **zerolog** for structured logging
- **TOML** for configuration (avoid JSON/YAML)
- **Secure credential management** via `aircher login` command with proper file permissions

### Architecture Principles
- **Clean Architecture**: Core business logic separated from external dependencies
- **Interface-Based Design**: All major components implement interfaces for testability
- **Multi-Database**: Separate SQLite databases (conversations, knowledge, file_index, sessions)
- **Provider Pattern**: Universal LLM provider interface
- **MCP Integration**: Model Context Protocol for extensible tool support

### Coding Standards
- Follow Go standard project layout and naming conventions
- Implement interfaces first, then concrete types
- Use context.Context for all cancellable operations
- Error messages should be user-friendly and actionable
- TUI components must handle terminal resizing gracefully
- Use dependency injection for better testing and modularity

### Build Commands
```bash
make build           # Build the aircher binary
make dev            # Build and run development version
make test           # Run all tests with race detection
make test-coverage  # Generate coverage reports
make lint           # Run golangci-lint (Go 1.24 tool management)
make fmt            # Format code with gofumpt
make tools          # Install all development tools
make tools-update   # Update all development tools
```

## AI Workflow Patterns

### Session Initialization
1. **Start Here**: Always begin with this AGENTS.md file for navigation
2. **Check Tasks**: Review `docs/core/TASKS.md` for current priorities and progress
3. **Minimal Context**: Include only task-relevant core documents in initial context
4. **Dynamic Access**: Use tools to fetch specific technical specifications as needed

### Task Execution Flow
1. **Select Task**: Choose atomic, well-defined task from `docs/core/TASKS.md` (ONLY source for tasks)
2. **Review Specs**: Access relevant documentation via task lookup table above
3. **Follow Standards**: Implement according to `docs/core/DEVELOPER_GUIDE.md`
4. **Update Progress**: Mark completion status in `docs/core/TASKS.md` (NEVER create duplicate task lists)

### Error Recovery Strategy
1. **Diagnostic Tools**: Use `make test`, `make lint`, project-specific debugging
2. **Limited Attempts**: Maximum 2 fix attempts before documenting issues
3. **Documentation**: Record problems and partial solutions in `docs/core/TASKS.md` ONLY
4. **Graceful Handoff**: Provide clear context for next development session

### Session Completion
1. **Update Progress**: Mark completed work in `docs/core/TASKS.md` (SINGLE source for all progress)
2. **Document Decisions**: Record architectural choices in appropriate specs
3. **Update Metrics**: Refresh code counts and coverage in `docs/core/TASKS.md` ONLY
4. **Validate Quality**: Ensure code meets `docs/core/DEVELOPER_GUIDE.md` standards

**CRITICAL**: Always update `docs/core/TASKS.md` - it is the single source of truth for all task management, progress tracking, and project metrics. Never create duplicate task lists in other files.

## Common Issues & Solutions

### Build and Development
- **Tool Installation**: Use `make tools` for Go 1.24 tool management
- **Formatting Issues**: Run `make fmt` with gofumpt (better than gofmt)
- **Test Failures**: Check race conditions with `make test` (includes -race flag)
- **Linting Errors**: Fix with `make lint` using golangci-lint

### Architecture Decisions
- **Interface Design**: Define at consumer level, keep small and focused
- **Error Handling**: Use consistent error types across providers
- **Context Usage**: Always pass context.Context as first parameter
- **Database Design**: Use separate databases for different data types

### TUI Development
- **Responsiveness**: Handle terminal resizing in all components
- **Styling**: Use centralized Lipgloss definitions and color constants
- **User Experience**: Provide clear feedback and loading states
- **Navigation**: Support keyboard shortcuts and vim-like patterns

## Current Development Status

### Implementation Phases
- **Phase 1**: ✅ Project foundation and TUI framework
- **Phase 2**: 🚧 LLM provider integration and streaming
- **Phase 3**: ❌ Context management and file relevance scoring
- **Phase 4**: ❌ MCP tool execution and web search integration

### High Priority Tasks
1. Implement actual LLM API calls with TUI streaming integration
2. Build file relevance scoring algorithms for intelligent context
3. Complete MCP tool execution with security permissions
4. Add comprehensive test coverage across all components

### Completed Features
- ✅ Project Analysis System - automatic project structure analysis
- ✅ Auto-generated documentation in `.aircher/project_analysis.md`
- ✅ Multi-database architecture with migration system
- ✅ TUI framework with Charmbracelet Bubble Tea

## Quick Reference

### File Structure
```
aircher/
├── AGENTS.md              # 🎯 This file - AI Navigation Hub
├── docs/core/             # 📋 Essential Project Context
│   ├── MASTER_SPEC.md     # Architecture & technical specs
│   ├── DEVELOPER_GUIDE.md # Coding standards & patterns
│   ├── TASKS.md           # Current priorities, progress & metrics
│   └── PROJECT_ROADMAP.md # Feature roadmap & phases
├── docs/technical/        # 🔧 Implementation Specifications
│   ├── 01-ui-improvements.md
│   └── 02-ai-agents-config.md
├── internal/              # Core application logic
├── cmd/                   # CLI entry points
└── examples/              # Usage examples
```

### Documentation Maintenance
- **ALWAYS** update `docs/core/TASKS.md` every development session - it is the ONLY task tracking location
- Mark completed items with ✅, in-progress with 🚧, not started with ❌ in TASKS.md ONLY
- Update `docs/core/TASKS.md` with metrics and progress after significant work
- Keep `docs/core/MASTER_SPEC.md` current with architectural changes
- Maintain `docs/core/DEVELOPER_GUIDE.md` as single source of coding truth
- **NEVER** create duplicate task lists in README.md, issues, or other documentation files

### Task Management Rules
1. **Single Source**: `docs/core/TASKS.md` is the ONLY location for all task tracking
2. **No Duplication**: Never create tasks, TODOs, or progress tracking in other files
3. **Always Reference**: When mentioning tasks elsewhere, link to TASKS.md
4. **Consistent Updates**: Update TASKS.md in every development session without exception

## Multi-Agent Support

This project supports multiple AI development tools through a unified approach. The aircher tool can generate agent-specific configuration files that redirect to this `AGENTS.md` file as the single source of truth. See `docs/technical/02-ai-agents-config.md` for implementation details.

---

**Remember**: This file serves as your primary navigation hub. Start here for every development session, use the task lookup table to find relevant documentation, and always update progress in the core documentation files.
