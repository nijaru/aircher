# Aircher Project - Development Memory

## Instructions

- This project uses Go 1.24+ with Charmbracelet's Bubble Tea TUI framework
- Uses Go 1.24 tool management for development tools (golangci-lint, gofumpt, staticcheck, air)
- Leverages Go 1.24's os.Root for secure MCP filesystem operations
- Takes advantage of Swiss Tables map implementation for performance
- Follow Go standard project layout and naming conventions
- Use zerolog for structured logging throughout the application
- All database operations use SQLite with sqlx for enhanced functionality
- Configuration uses TOML format, avoid JSON/YAML for config files
- Implement interfaces first, then concrete types for better testability
- TUI components should be responsive and handle terminal resizing gracefully
- Always update project documentation (TASKS.md, OUTLINE.md, SPEC.md) when completing features

## Conventions

- Package names should be short, descriptive, and lowercase
- Use context.Context for all operations that might be cancelled
- Error messages should be user-friendly and actionable
- Database migrations are handled automatically on startup
- Provider implementations should return consistent error types
- TUI styling uses Lipgloss with defined color constants
- All external API calls should have proper timeout handling
- Use dependency injection for better testing and modularity

## Commands

- `make build` - Build the aircher binary
- `make test` - Run all tests with race detection
- `make test-coverage` - Generate coverage report
- `make lint` - Run golangci-lint (Go 1.24 tool management)
- `make fmt` - Format code with gofumpt (better than gofmt)
- `make clean` - Clean build artifacts
- `make dev` - Build and run development version
- `make release` - Build for multiple platforms
- `make tools` - Install all development tools (Go 1.24 feature)
- `make tools-update` - Update all development tools
- `go mod tidy` - Clean up dependencies
- `go tool <toolname>` - Run development tools (Go 1.24 feature)
- `./aircher --help` - Show CLI help
- `./aircher` - Start interactive TUI mode

## Architecture

- **Clean Architecture**: Core business logic separated from external dependencies
- **Interface-Based Design**: All major components implement interfaces for testability
- **Multi-Database**: Separate SQLite databases for different data types (conversations, knowledge, file_index, sessions)
- **Provider Pattern**: Universal LLM provider interface supporting OpenAI, Claude, Gemini, Ollama
- **TUI-First**: Charmbracelet Bubble Tea provides modern terminal interface
- **MCP Integration**: Model Context Protocol for extensible tool support
- **Configuration Layers**: Project-specific and user-global TOML configuration

## Documentation Maintenance

- **TASKS.md**: Update completion status when finishing features or phases
- **OUTLINE.md**: Update roadmap and feature descriptions for major changes
- **SPEC.md**: Update technical specifications when architecture changes
- **README.md**: Keep usage examples and feature list current
- **STATUS.md**: Update project status and metrics after significant work
- Mark completed items with ✅, in-progress with 🚧, not started with ❌
- Add implementation notes for complex features
- Update code metrics (line counts, test coverage) regularly

## Glossary

- **TUI**: Terminal User Interface built with Charmbracelet Bubble Tea
- **Provider**: LLM service integration (OpenAI, Claude, Gemini, Ollama)
- **MCP**: Model Context Protocol for extensible tool integration
- **Context Engine**: Intelligent file and conversation context management
- **Session**: Persistent conversation state with history
- **Slash Command**: Interactive commands starting with / (e.g., /help, /clear)
- **REPL**: Read-Eval-Print Loop - the interactive chat interface
- **Streaming**: Real-time response rendering as LLM generates text
- **Compaction**: Intelligent conversation summarization to manage context limits
- **Relevance Scoring**: Algorithm to determine file importance for context

## Development Notes

- TUI components are built with Charmbracelet's ecosystem for consistency
- Provider API calls are currently stubbed - implement actual HTTP calls next
- Database schemas support all planned features but some tables are unused
- MCP server integration framework is complete but needs real tool execution
- Context management algorithms are framework-ready but need implementation
- Web search integration has decision engine but needs API integration
- All styling uses centralized Lipgloss definitions for maintainability
- Go 1.24 os.Root provides secure filesystem access for MCP operations
- Swiss Tables map implementation improves runtime performance by 2-3%
- Tool management in go.mod eliminates need for tools.go files

## Go 1.24 Features Used

- **Tool Management**: Development tools tracked in go.mod with `tool` directives
- **os.Root**: Secure directory-limited filesystem access for MCP security
- **Swiss Tables**: Improved map performance with automatic runtime optimization
- **Enhanced Build Info**: JSON output and VCS info embedded in binaries
- **Generic Type Aliases**: Available for future type system enhancements
- **Improved Finalizers**: runtime.AddCleanup for better resource management

## Current Priorities

1. Implement actual LLM API calls with TUI streaming integration
2. Build file relevance scoring algorithms for intelligent context
3. Integrate web search APIs (Brave Search, DuckDuckGo)
4. Complete AIRCHER.md parsing and memory synchronization
5. Add real MCP tool execution with permission system
6. Implement conversation compaction algorithms
7. Add comprehensive test coverage across all components