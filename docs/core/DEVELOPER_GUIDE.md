# Aircher Developer Guide

Coding standards and implementation patterns for the Aircher dual-architecture project.

## Technology Stack

### Aircher Terminal (Rust)
- Rust 1.80+ with async/await and tokio runtime
- Ratatui for TUI framework
- SQLite + sqlx for database operations
- tracing for structured logging
- TOML configuration

### Aircher Intelligence Engine (Python + Rust)
- Python 3.11+ with asyncio and MCP protocol
- Rust performance modules via PyO3 bindings
- numpy/scipy for vector operations
- sentence-transformers for embeddings
- tree-sitter for AST parsing (Rust-accelerated)
- uvx for deployment and dependency management

## Development Setup

### Rust Terminal Client
```bash
cargo build --release  # Build binary
cargo test             # Run tests
cargo clippy           # Run linting
cargo fmt              # Format code
cargo tarpaulin        # Generate coverage reports
```

### Python MCP Server
```bash
# Install with uvx for development
uvx --from . --editable aircher-intelligence

# Or traditional development
pip install -e .[dev]

# Run tests
pytest tests/
pytest --cov=aircher_intelligence tests/

# Linting and formatting
ruff check .
ruff format .
mypy aircher_intelligence/

# Build Rust extensions (if present)
maturin develop
```

## Coding Standards

### Rust Terminal Client

#### Package Organization
- Follow Rust standard project layout with `src/`, `tests/`, `examples/`
- Module names should be short, descriptive, and snake_case
- Use meaningful directory structure under `src/` for application logic
- Separate concerns: UI, business logic, data access, external integrations

#### Naming Conventions
- Use Rust standard naming conventions (snake_case for functions/variables, PascalCase for types)
- Traits should describe behavior (e.g., `LLMProvider`, `ContextStorage`)
- Structs should be descriptive nouns
- Functions should use verb_noun pattern when appropriate
- Constants use SCREAMING_SNAKE_CASE

### Python MCP Server

#### Package Organization
- Follow Python package structure with clear module separation
- Package names should be short, descriptive, and snake_case
- Use meaningful directory structure: `aircher_intelligence/backends/`, `aircher_intelligence/intelligence/`
- Separate concerns: MCP protocol, intelligence algorithms, storage, performance modules

#### Naming Conventions
- Use Python PEP 8 naming conventions
- Protocols should describe behavior (e.g., `FileSystemBackend`, `ASTBackend`)
- Classes should be descriptive nouns in PascalCase
- Functions should use verb_noun pattern in snake_case
- Constants use SCREAMING_SNAKE_CASE

### Code Structure

#### Rust Patterns
- **Implement traits first, then concrete types** for better testability
- Use dependency injection via trait objects for modularity
- Separate business logic from external dependencies (Clean Architecture)
- Keep functions focused and single-purpose
- Use Result<T, E> for all fallible operations

#### Python Patterns  
- **Implement protocols first, then concrete classes** for better testability
- Use dependency injection and protocol-based design for modularity
- Separate business logic from external dependencies (Clean Architecture)
- Keep functions focused and single-purpose
- Use modular backend architecture for performance optimization

### Error Handling

#### Rust Error Handling
- Use thiserror for custom error types
- Error messages should be user-friendly and actionable
- Provider implementations should return consistent Result types
- Use tokio's cancellation for async operations
- Chain errors with context using .with_context()

#### Python Error Handling
- Use custom exception hierarchies with clear inheritance
- Error messages should be user-friendly and actionable
- Backend implementations should return consistent result types
- Use asyncio cancellation for async operations
- Chain errors with meaningful context using exception chaining

### Context Usage

#### Rust Context Management
- Use tokio::task cancellation for async operations
- Pass context implicitly through async function chains
- Handle cancellation appropriately with timeout wrappers
- Set reasonable timeouts for external API calls

#### Python Context Management
- Use asyncio context and cancellation tokens
- Pass context as first parameter to async functions
- Handle cancellation appropriately with timeout wrappers
- Set reasonable timeouts for external API calls and MCP operations

## Architecture Patterns

### Clean Architecture

#### Rust Terminal Client
- Core business logic separated from external dependencies
- Domain layer should not depend on infrastructure
- Use traits to define contracts between layers
- Keep external concerns (TUI, database, APIs) in outer layers

#### Python MCP Server
- Core intelligence logic separated from MCP protocol and performance modules
- Domain layer should not depend on infrastructure
- Use protocols to define contracts between layers
- Keep external concerns (MCP protocol, Rust modules, databases) in outer layers

### Interface-Based Design

#### Rust Design
- All major components implement traits for testability
- Define traits at the consumer level, not the provider level
- Keep traits small and focused (Interface Segregation Principle)
- Use composition over inheritance where possible

#### Python Design
- All major components implement protocols for testability
- Define protocols at the consumer level, not the implementation level
- Keep protocols small and focused (Interface Segregation Principle)
- Use composition and modular backends for performance optimization

### Multi-Database Pattern
- Separate SQLite databases for different data types:
  - `conversations`: Chat history and sessions
  - `knowledge`: Project analysis and insights
  - `file_index`: File metadata and indexing
  - `sessions`: Persistent conversation state
- Database migrations handled automatically on startup
- Use sqlx for enhanced database functionality

### Provider Pattern
- Universal LLM provider interface supporting:
  - OpenAI
  - Claude (Anthropic)
  - Gemini (Google)
  - Ollama (Local)
- Consistent error handling across all providers
- Timeout handling for all external API calls
- Streaming support for real-time responses

## TUI Development Guidelines

### Ratatui Components
- TUI components should be responsive and handle terminal resizing gracefully
- Use Ratatui's widget ecosystem for consistency:
  - Built-in styling with ratatui::style
  - Standard widgets from ratatui::widgets
  - Custom widgets following Ratatui patterns
- Follow the Ratatui immediate-mode pattern

### Styling Standards
- All styling uses centralized Ratatui style definitions
- Define color constants for consistent theming
- Support both light and dark terminal themes
- Ensure accessibility with sufficient contrast ratios
- Handle terminal resizing gracefully

### User Experience
- Provide clear feedback for all user actions
- Show loading states for long-running operations
- Support keyboard shortcuts and vim-like navigation
- Implement proper error states and recovery

## Testing Requirements

### Test Coverage
- Aim for high test coverage across all components
- Focus on business logic and critical paths
- Use table-driven tests for multiple scenarios
- Test error conditions and edge cases

### Testing Patterns
- Use dependency injection to enable easy mocking
- Test interfaces, not implementations
- Use the `testify` package for assertions and mocks
- Separate unit tests from integration tests

### Test Organization
```
internal/
  component/
    component.go
    component_test.go
    testdata/
      fixtures.json
```

### Race Detection
- Always run tests with race detection: `go test -race`
- Use proper synchronization for concurrent operations
- Test concurrent scenarios explicitly

## Performance Guidelines

### Memory Management
- Use Swiss Tables map implementation for better performance
- Implement proper resource cleanup with `runtime.AddCleanup`
- Avoid memory leaks in long-running operations
- Profile memory usage for heavy operations

### Database Performance
- Use prepared statements for repeated queries
- Implement proper indexing strategies
- Use transactions for batch operations
- Monitor query performance and optimize as needed

### External API Optimization
- Implement proper timeout handling
- Use connection pooling for HTTP clients
- Cache responses when appropriate
- Implement retry logic with exponential backoff

## Configuration Management

### Configuration Format
- Use TOML format exclusively (avoid JSON/YAML)
- Support both project-specific and user-global configuration
- Provide sensible defaults for all configuration options
- Validate configuration on startup

### Configuration Layers
1. Default values in code
2. Global user configuration (`~/.config/aircher/config.toml`)
3. Project-specific configuration (`.aircher/config.toml`)
4. Environment variables for sensitive values
5. Command-line flags for overrides

## Security Considerations

### MCP Integration
- Use Rust's std::fs with proper permission checks for secure filesystem operations
- Implement proper permission system for tool execution
- Validate and sanitize all external inputs
- Use secure defaults for all configurations

### API Key Management
- Never hardcode API keys in source code
- Use environment variables or secure configuration files
- Implement key rotation strategies
- Log security events appropriately

## Documentation Requirements

### Code Documentation
- All exported functions and types must have godoc comments
- Include examples in documentation when helpful
- Document complex algorithms and business logic
- Keep documentation up-to-date with code changes

### Project Documentation Updates
When completing features, always update:
- `docs/tasks/tasks.json`: Mark completion status and outcomes using JSON structure
- `docs/core/PROJECT_ROADMAP.md`: Update roadmap and milestones
- `docs/core/MASTER_SPEC.md`: Update architectural overview
- `docs/architecture/`: Update relevant architecture documentation
  - `output/` for TUI components and response streaming
  - `storage-architecture.md` for database operations
  - `plugins/` for LLM provider implementations
  - `plugins/` for context management algorithms
  - `plugins/` for MCP tool integration
- `README.md`: Update usage examples and feature lists
- `docs/core/STATUS.md`: Update project metrics and status

### Documentation Maintenance
- Mark completed items with ✅
- Mark in-progress items with 🚧
- Mark not started items with ❌
- Add implementation notes for complex features
- Update code metrics (line counts, test coverage) regularly

## Glossary

### Technical Terms
- **TUI**: Terminal User Interface built with Ratatui
- **Provider**: LLM service integration (OpenAI, Claude, Gemini, Ollama)
- **MCP**: Model Context Protocol for extensible tool integration
- **Context Engine**: Intelligent file and conversation context management
- **Session**: Persistent conversation state with history
- **Slash Command**: Interactive commands starting with / (e.g., /help, /clear)
- **REPL**: Read-Eval-Print Loop - the interactive chat interface
- **Streaming**: Real-time response rendering as LLM generates text
- **Compaction**: Intelligent conversation summarization to manage context limits
- **Relevance Scoring**: Algorithm to determine file importance for context

## Quality Gates

### Pre-commit Checklist
- [ ] Code passes `make lint`
- [ ] Code is formatted with `make fmt`
- [ ] All tests pass with `make test`
- [ ] Documentation is updated
- [ ] Error handling is appropriate
- [ ] Performance impact is considered

### Code Review Standards
- Review for adherence to this guide
- Check error handling and edge cases
- Verify test coverage for new code
- Ensure documentation is clear and complete
- Validate security implications

## Common Patterns

### Error Wrapping
```go
if err != nil {
    return fmt.Errorf("failed to process user input: %w", err)
}
```

### Context Propagation
```go
func ProcessWithTimeout(ctx context.Context, data []byte) error {
    ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
    defer cancel()
    
    return processData(ctx, data)
}
```

### Interface Definition
```go
type Provider interface {
    GenerateResponse(ctx context.Context, prompt string) (*Response, error)
    StreamResponse(ctx context.Context, prompt string) (<-chan *StreamChunk, error)
}
```

### Dependency Injection
```go
type Service struct {
    provider Provider
    storage  Storage
    logger   zerolog.Logger
}

func NewService(provider Provider, storage Storage, logger zerolog.Logger) *Service {
    return &Service{
        provider: provider,
        storage:  storage,
        logger:   logger,
    }
}
```

---

This guide serves as the authoritative source for development practices in the Aircher project. When in doubt, refer to this document and update it as the project evolves.