# Aircher Technical Specification

## System Architecture Overview

Aircher is an AI-powered terminal-based development assistant built with Rust 1.80+ and Ratatui TUI framework. The system integrates multiple LLM providers and implements the Model Context Protocol (MCP) for extensible tool support.

### Core Architecture Principles

- **Clean Architecture**: Core business logic separated from external dependencies
- **Trait-Based Design**: All major components implement traits for testability
- **Multi-Database**: Separate SQLite databases optimized for different data types
- **Provider Pattern**: Universal LLM provider interface supporting multiple backends
- **Intelligent Context Management**: AI-driven file relevance scoring and task detection
- **MCP Integration**: Extensible tool ecosystem via Model Context Protocol

## Core Components

### 1. Command Line Interface (CLI)
**Detailed Specification**: `docs/tasks/tasks.json` (Future phase tasks)

**Service/Provider/Model Hierarchy**:
- **Service**: API endpoint (OpenAI, Anthropic, OpenRouter, Ollama)
- **Provider**: Entity hosting model (Anthropic, DeepSeek, Meta, etc.)
- **Model**: Specific model (gpt-4, claude-3-sonnet, llama-3.1-8b)

**Core Commands**:
```bash
# Main Interface (Claude Code-inspired)
aircher                          # Start unified TUI
aircher --service openai         # Start with specific service
aircher --model claude-3-sonnet  # Start with specific model
aircher --worktree feature-auth  # Start with specific worktree context

# Service Authentication
aircher auth                     # Interactive service setup
aircher auth openai              # Configure specific service
aircher auth status              # Show configured services
aircher auth set                 # Set default service
aircher auth remove openai       # Remove service

# Model Management
aircher model                    # Interactive model selection
aircher model gpt-4              # Set specific model
aircher model list               # List available models
aircher model list --provider deepseek  # Filter by provider (OpenRouter)

# Worktree Management
aircher worktree list            # List all worktrees with status
aircher worktree switch main     # Switch to main worktree context
aircher worktree compare main feature-auth  # Compare insights between worktrees

# Context Management
aircher context status           # Show current context hierarchy
aircher context insights         # Show cross-context insights
aircher context transfer feature-auth main  # Transfer learnings between contexts
```

**Key Features**:
- Clean service → provider → model hierarchy
- Secure API key management with validation
- OpenRouter provider filtering support
- Context usage display in TUI (44k/200k tokens)
- Unified chat interface following Claude Code patterns

### 2. Modern Terminal Interface (TUI)
**Detailed Specification**: `docs/architecture/output/tui-improvements.md`

```go
type Model struct {
    input         textinput.Model
    viewport      viewport.Model
    messages      []Message
    width, height int
    ready         bool
    streaming     bool
    showHelp      bool
    showContext   bool
    styles        Styles
    renderer      *lipgloss.Renderer
}
```

**Key Features**:
- Responsive terminal interface with Ratatui framework
- Real-time streaming response display
- Context-aware help and shortcuts
- Vim-mode support and keyboard navigation

### 3. Multi-Database Storage Architecture
**Detailed Specification**: `docs/architecture/storage-architecture.md`

**Database Design**:
- `conversations.db` - Chat history with worktree context and interaction metadata
- `knowledge.db` - Project analysis, cross-context insights, learned patterns
- `file_index.db` - File metadata, relationships, context-aware change tracking
- `sessions.db` - User sessions, context hierarchy, temporary state

**Hybrid Storage Strategy**:
- **SQLite**: Structured metadata and relationships
- **File System**: Large content and binary data
- **Specialized Indexes**: Vector embeddings for semantic search
- **Hierarchical Context Storage**: Global → Project → Worktree → Session

**Context Hierarchy**:
```
Global: ~/.config/aircher/global.db
Project: .agents/db/core/
Worktree: .agents/worktrees/{worktree-id}/
Session: .agents/sessions/{session-id}/
```

### 4. Universal LLM Provider System
**Detailed Specification**: `docs/architecture/plugins/llm-providers.md`

```go
type LLMProvider interface {
    Chat(ctx context.Context, req *ChatRequest) (*ChatResponse, error)
    ChatStream(ctx context.Context, req *ChatRequest) (<-chan *ChatResponse, error)
    SupportsFunctions() bool
    SupportsSystemMessages() bool
    SupportsImages() bool
    SupportsThinking() bool
    GetTokenLimit(model string) int
    CountTokens(content string) (int, error)
    CalculateCost(tokens int, model string) (float64, error)
    Name() string
    Models() []string
}
```

**Supported Providers**:
- ✅ **OpenAI**: GPT-3.5, GPT-4, GPT-4 Turbo models
- ✅ **Anthropic Claude**: Claude-3 Haiku, Sonnet, Opus models
- 🚧 **Google Gemini**: Gemini Pro, Gemini Pro Vision
- 🚧 **Ollama**: Local model hosting with various open-source models

### 5. Intelligent Context Management
**Detailed Specification**: `docs/architecture/plugins/context-management.md`

**Task Detection System**:
```go
type TaskDetector struct {
    patterns        []TaskPattern
    fileWatcher     *FileWatcher
    gitWatcher      *GitWatcher
    behaviorAnalyzer *BehaviorAnalyzer
}

type TaskType string
const (
    TaskDebugging      TaskType = "debugging"
    TaskFeatureDev     TaskType = "feature_development"
    TaskRefactoring    TaskType = "refactoring"
    TaskDocumentation  TaskType = "documentation"
    TaskTesting        TaskType = "testing"
    TaskMaintenance    TaskType = "maintenance"
)
```

**File Relevance Engine**:
```go
type FileRelevanceEngine struct {
    dependencyGraph    *DependencyGraph
    accessPatterns     *AccessPatternAnalyzer
    taskContext        *TaskContext
    relevanceScorer    *RelevanceScorer
}
```

**Smart Conversation Compaction**:
- Automatic context optimization based on task completion
- Intelligent message importance scoring
- Configurable preservation rules for critical information

### 6. MCP (Model Context Protocol) Integration
**Detailed Specification**: `docs/architecture/plugins/mcp-integration.md`

```go
type MCPManager struct {
    localServers    map[string]*MCPServer
    projectServers  map[string]*MCPServer
    userServers     map[string]*MCPServer
    client          *MCPClient
    registry        *MCPRegistry
    installer       *MCPInstaller
    permissionSystem *MCPPermissionSystem
}
```

**Core MCP Servers**:
- **filesystem**: File operations and management
- **brave-search**: Web search capabilities
- **git**: Git repository operations
- **github**: GitHub API integration
- **postgres/sqlite**: Database operations

**Security Features**:
- Comprehensive permission system with user confirmation
- Scoped access control (local, project, user)
- Audit logging and security monitoring

### 7. Project Analysis System
**Status**: ✅ **Completed**

```go
type ProjectAnalyzer struct {
    projectRoot    string
    storageEngine  *StorageEngine
    logger         *zerolog.Logger
}
```

**Capabilities**:
- Automatic project structure analysis
- Documentation generation in `.agents/project_analysis.md`
- Component detection and classification
- Cross-reference mapping

## Key Data Structures

### Chat Communication
```go
type ChatRequest struct {
    Messages    []Message     `json:"messages"`
    Tools       []Tool        `json:"tools,omitempty"`
    MaxTokens   *int          `json:"max_tokens,omitempty"`
    Temperature *float64      `json:"temperature,omitempty"`
    Stream      bool          `json:"stream,omitempty"`
    Model       string        `json:"model"`
    Provider    string        `json:"provider"`
}

type ChatResponse struct {
    Message   *Message              `json:"message,omitempty"`
    Stream    *StreamResponse       `json:"stream,omitempty"`
    TokensUsed TokenUsage           `json:"usage"`
    Cost       float64              `json:"cost"`
    Duration   time.Duration        `json:"duration"`
    Provider   string               `json:"provider"`
    Model      string               `json:"model"`
}
```

### File Relevance
```go
type FileRelevance struct {
    Path              string             `json:"path"`
    Score             float64            `json:"score"`
    LastAccessed      time.Time          `json:"last_accessed"`
    AccessFrequency   int                `json:"access_frequency"`
    Dependencies      []string           `json:"dependencies"`
    RelevanceType     RelevanceType      `json:"relevance_type"`
    ConfidenceScore   float64            `json:"confidence_score"`
}
```

## Configuration System

**Architecture**: Minimal configuration with intelligent defaults and secure credential management

### Two-File Strategy
- **Configuration**: `~/.config/aircher/config.toml` - User preferences, no secrets
- **Credentials**: `~/.config/aircher/credentials.toml` - API keys, restricted permissions (600)

### MVP Configuration Approach
```toml
# ~/.config/aircher/config.toml - Minimal user preferences
[providers]
default = "auto"              # auto-detect best available provider
fallback_enabled = true       # automatically fallback if primary fails

[models]
auto_select = true            # intelligent model selection based on task
openai_default = "gpt-4"      # smart defaults per provider
anthropic_default = "claude-3-5-sonnet"

# Task-specific model overrides for cost optimization
[models.tasks]
commit_messages = "gpt-3.5-turbo"        # Fast, cheap for git commits
summaries = "claude-3-haiku"             # Efficient for text summarization
code_review = "gpt-4"                    # High-quality for code analysis
documentation = "claude-3-haiku"         # Good balance for docs
refactoring = "gpt-4"                    # Complex reasoning needed
debugging = "claude-3-5-sonnet"          # Strong analytical capabilities

[interface]
show_thinking = true          # show AI thinking process
show_context_usage = true     # show token usage (e.g., "44k/200k")
streaming = true             # real-time response streaming

[context]
max_files = 20               # intelligent context management
auto_compaction = true       # automatic optimization

[costs]
monthly_budget = 100.0       # budget tracking and warnings
track_usage = true
prefer_cost_efficient = true # auto-select cheaper models when appropriate
```

### Credential Management via `aircher login`
```toml
# ~/.config/aircher/credentials.toml - API keys (file permissions: 600)
[openai]
api_key = "sk-..."

[anthropic]
api_key = "sk-ant-..."

[google]
api_key = "AI..."
```

### Smart Defaults Strategy
- **Model Parameters**: Auto-detected from models.dev API (max_tokens, temperature, context limits)
- **Provider Selection**: Auto-detect from available credentials
- **Model Selection**: Intelligent defaults (GPT-4, Claude-3.5-Sonnet, Gemini-2.5-Pro)
- **Context Management**: File relevance scoring with automatic optimization
- **Cost Tracking**: Real-time pricing from models.dev API

### Planning Documentation
- **MVP Configuration**: `docs/config/mvp-config-spec.toml`
- **Credential Management**: `docs/config/credentials-spec.toml`

## Implementation Phases

### ✅ Phase 1: Foundation (Completed)
- Project setup and development environment
- TUI framework with Ratatui implementation
- Multi-database architecture with migration system
- Project Analysis System with auto-generated documentation
- Basic configuration system

### 🚧 Phase 2: Intelligence (Framework Complete)
- LLM provider interfaces and OpenAI/Claude implementations
- Context management system architecture
- Task detection framework
- File relevance scoring foundation

### 🚧 Phase 3: Advanced Features (Partially Complete)
- MCP integration framework
- Smart conversation compaction
- Web search integration
- Enhanced security and permissions

### ❌ Phase 4: Enterprise Features
- Advanced monitoring and health checks
- Cost management and budgeting
- Git workflow integration
- Performance optimization

### ❌ Phase 5: Production Ready
- Comprehensive test coverage
- Documentation completion
- Auto-update system
- Distribution and packaging

## Current Implementation Status

### ✅ Production Ready
- **Project Analysis System**: Automatic analysis and documentation generation
- **Multi-Database Architecture**: SQLite databases with migration system
- **TUI Framework**: Responsive terminal interface with Ratatui
- **Configuration System**: TOML-based configuration management
- **Basic Provider Interface**: OpenAI and Claude provider foundations

### 🚧 Implementation Pending
- **Actual LLM API Integration**: Complete provider implementations with streaming
- **Task-Specific Model Selection**: Automatic cost-optimized model selection for different task types
- **File Relevance Algorithms**: Intelligent context selection algorithms
- **MCP Tool Execution**: Security-controlled tool execution system
- **Smart Compaction**: Conversation optimization and summarization

### ❌ Not Yet Implemented
- **Advanced Context Management**: Task-aware context assembly
- **Web Search Integration**: Automatic search trigger system
- **Comprehensive Testing**: Unit and integration test coverage
- **Performance Optimization**: Caching, connection pooling, async processing

## Technical Dependencies

### Core Framework
```go
# Core application framework
ratatui = "0.24"                     # TUI framework
crossterm = "0.27"                   # Terminal control
tokio = { version = "1.34", features = ["full"] }  # Async runtime

# Database and storage
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls"] }
```

### LLM Providers
```go
// Provider implementations
github.com/sashabaranov/go-openai    // OpenAI client
github.com/anthropics/anthropic-sdk-go // Claude client
cloud.google.com/go/aiplatform       // Gemini client
```

### Development Tools
```go
# Development and testing
tracing = "0.1"                      # Structured logging
serde = { version = "1.0", features = ["derive"] }  # Serialization
toml = "0.8"                         # Configuration parsing
```

## Build Commands

```bash
cargo build --release  # Build the aircher binary
cargo run             # Build and run development version
cargo test            # Run all tests
cargo tarpaulin       # Generate coverage reports
cargo clippy          # Run clippy linter
cargo fmt             # Format code with rustfmt
```

## Related Documentation

- **Developer Guide**: `docs/core/DEVELOPER_GUIDE.md` - Coding standards and patterns
- **Tasks and Progress**: `docs/tasks/tasks.json` - Current priorities and implementation status (JSON-based task management)
- **Project Roadmap**: `docs/core/PROJECT_ROADMAP.md` - Feature timeline and milestones

### Architecture Documentation
- **CLI Commands**: `docs/architecture/commands/` - Command specifications and implementations
- **Configuration System**: `docs/architecture/config/` - TOML configuration architecture
- **TUI & Output**: `docs/architecture/output/` - Terminal interface and response streaming
- **LLM Providers & MCP**: `docs/architecture/plugins/` - Provider integration and MCP tools
- **Storage Architecture**: `docs/architecture/storage-architecture.md` - Database design patterns
- **Development Workflow**: `docs/development/workflow/` - Git, testing, and AI agent configuration

---

**Note**: This specification serves as the architectural overview. For detailed implementation specifics, refer to the component-specific documentation in `docs/architecture/`. All task tracking and progress updates are maintained in `docs/tasks/tasks.json` using the revolutionary JSON-based task management system.
