# Aircher Developer Guide

Coding standards and implementation patterns for the Aircher project built with Rust.

## Technology Stack

- Rust 1.80+ with async/await and tokio runtime
- Ratatui for TUI framework
- SQLite + sqlx for database operations
- tracing for structured logging
- TOML configuration

## Development Setup

```bash
cargo build --release  # Build binary
cargo test             # Run tests
cargo clippy           # Run linting
cargo fmt              # Format code
cargo tarpaulin        # Generate coverage reports
```

## Coding Standards

### Package Organization
- Follow Go standard project layout
- Package names should be short, descriptive, and lowercase
- Use meaningful directory structure under `internal/` for application logic
- Separate concerns: UI, business logic, data access, external integrations

### Naming Conventions
- Use Go standard naming conventions
- Interfaces should describe behavior (e.g., `Provider`, `Storage`)
- Concrete types should be descriptive nouns
- Functions should use verb-noun pattern when appropriate
- Constants use CamelCase or ALL_CAPS for exported values

### Code Structure
- **Implement interfaces first, then concrete types** for better testability
- Use dependency injection for better testing and modularity
- Separate business logic from external dependencies (Clean Architecture)
- Keep functions focused and single-purpose

### Error Handling
- Error messages should be user-friendly and actionable
- Provider implementations should return consistent error types
- Use context.Context for all operations that might be cancelled
- Wrap errors with meaningful context using `fmt.Errorf` or error wrapping

### Context Usage
- Use `context.Context` for all operations that might be cancelled
- Pass context as the first parameter to functions
- Handle context cancellation appropriately
- Set reasonable timeouts for external API calls

## Architecture Patterns

### Clean Architecture
- Core business logic separated from external dependencies
- Domain layer should not depend on infrastructure
- Use interfaces to define contracts between layers
- Keep external concerns (UI, database, APIs) in outer layers

### Interface-Based Design
- All major components implement interfaces for testability
- Define interfaces at the consumer level, not the provider level
- Keep interfaces small and focused (Interface Segregation Principle)
- Use composition over inheritance where possible

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