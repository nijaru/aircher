# Aircher Technical Specification

## System Architecture Overview

Aircher is a dual-architecture AI development system featuring a Rust terminal client and a Python MCP server with Rust performance modules. The system provides AI-powered development tools that work across any compatible environment with optimized performance for critical operations.

### Core Architecture Principles

- **Dual Architecture**: Rust terminal assistant + Python MCP server for maximum compatibility
- **Hybrid Performance**: Rust terminal client with Python MCP server featuring Rust performance modules
- **Clean Architecture**: Domain-driven design with clear separation of concerns
- **Modular Design**: Trait-based design (Rust) and protocol-based backends (Python) for performance optimization
- **Multi-Database Strategy**: Separate SQLite databases optimized for different data types and contexts
- **Universal Compatibility**: MCP server works with Claude Desktop, VS Code, and any MCP-compatible tool
- **Intelligent Context Management**: AI-driven file relevance scoring and cross-project learning
- **Cross-Project Intelligence**: Pattern recognition and success correlation across entire codebase
- **Provider-Agnostic Design**: Universal LLM provider interface supporting multiple authentication methods
- **Hierarchical Configuration**: Profile-based configuration system with environment overrides

## Core Components

### 1. Dual Architecture Overview

**Aircher Terminal** - Full-featured AI assistant with advanced terminal UI (Rust)
**Aircher Intelligence Engine** - Universal MCP server providing intelligent context management (Python + Rust)

### 2. Aircher Terminal (CLI)
**Detailed Specification**: `docs/tasks/tasks.json` (Future phase tasks)

**Service/Provider/Model Hierarchy** (inspired by Codex architecture):
- **Service**: API endpoint with authentication (OpenAI, Anthropic, OpenRouter, Ollama)
- **Provider**: Entity hosting model with specific capabilities (Anthropic, DeepSeek, Meta)
- **Model**: Specific model with cost/performance characteristics (claude-4-sonnet, claude-4-opus, o3, gpt-4o, gemini-2.5-pro, deepseek/deepseek-r1-0528)
- **Authentication**: Multiple methods (API keys, OAuth, local hosting) per provider

**Core Commands** (inspired by Claude Code patterns):
```bash
# Main Interface - Direct REPL-style interaction
aircher                          # Start interactive terminal session
aircher --resume                 # Resume previous conversation
aircher --service openai         # Start with specific service
aircher --model claude-4-sonnet  # Start with specific model
aircher --profile production     # Start with specific profile

# Interactive Session Commands (Claude Code-inspired slash commands)
/help                           # Show available commands and shortcuts
/exit                           # Exit session
/clear                          # Clear conversation history
/resume [session-id]            # Resume specific session
/switch-model gpt-4             # Switch model mid-conversation
/web-search [query]             # Trigger web search
/upload-image [path]            # Upload and analyze image
/thinking                       # Toggle thinking mode display

# Service Authentication & Management
aircher login                   # Interactive authentication setup
aircher login openai            # Configure specific service with guided flow
aircher login --check           # Validate all configured services
aircher logout openai           # Remove service authentication
aircher auth status             # Show configured services with health

# Context & File Integration (Claude Code @-mention pattern)
@README.md                      # Reference specific file in conversation
@src/                          # Reference entire directory
@git:HEAD~1                    # Reference git commit or diff
@search:function_name          # Search and reference code patterns

# Task & Todo Management (Claude Code pattern)
/todo add "Implement OAuth"     # Add todo item
/todo list                     # Show current todos
/todo complete 1               # Mark todo as complete
/todo clear                    # Clear completed todos

# Advanced Features
aircher --web-search           # Enable web search for session
aircher --image-support        # Enable image upload capability
aircher --thinking-mode        # Start with thinking mode enabled
```

**Key Features** (enhanced with Claude Code patterns):
- **REPL-Style Interaction**: Direct terminal-based AI assistant with natural language commands
- **Real-Time Steering**: Send messages while AI is working to guide responses
- **Contextual File Integration**: @-mention files, directories, and git references
- **Slash Command System**: In-session commands for model switching, web search, and task management
- **Conversation Resumption**: Seamless session continuation with `--resume`
- **Thinking Mode**: Optional display of AI reasoning process
- **Interrupt Capability**: ESC key interruption during AI processing
- **Context Usage Display**: Real-time token usage (44k/200k tokens)
- **Web Search Integration**: Automatic and manual web search capabilities
- **Image Processing**: Upload and analyze images within conversations

### 3. Modern Terminal Interface (TUI)
**Detailed Specification**: `docs/architecture/output/tui-improvements.md`

```rust
// Pure Rust TUI implementation with Ratatui (Claude Code-inspired)
use ratatui::prelude::*;
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone)]
pub struct AppState {
    pub input: String,
    pub messages: Vec<Message>,
    pub viewport_offset: usize,
    pub streaming: bool,
    pub interrupted: bool,          // ESC key interrupt capability
    pub show_help: bool,
    pub show_context: bool,
    pub show_thinking: bool,        // Thinking mode display
    pub current_model: String,
    pub token_usage: TokenUsage,
    pub web_search_enabled: bool,   // Web search capability
    pub image_support: bool,        // Image processing support
    pub todos: Vec<TodoItem>,       // Integrated todo management
    pub session_id: String,         // Session resumption support
}

// Enhanced interaction modes (Claude Code pattern)
#[derive(Debug, Clone)]
pub enum InteractionMode {
    Chat,                          // Standard conversation
    Thinking,                      // Show AI reasoning
    WebSearch,                     // Web search active
    FileReference,                 // @-mention file mode
    SlashCommand,                  // Command processing
}

pub struct App {
    state: AppState,
    llm_client: Arc<dyn LLMProvider>,
    config: Config,
    mode: InteractionMode,
    command_processor: CommandProcessor,  // Slash command handling
    file_referencer: FileReferencer,      // @-mention support
    web_searcher: WebSearcher,           // Web search integration
}

// Real-time steering capability (Claude Code innovation)
pub struct MessageSteering {
    pub active_stream: Option<StreamHandle>,
    pub steering_buffer: String,
    pub can_interrupt: bool,
}
```

**Key Features** (enhanced with Claude Code innovations):
- **REPL-Style Terminal Interface**: Interactive session with natural language commands
- **Real-Time Message Steering**: Send messages while AI is responding to guide output
- **ESC Key Interruption**: Immediate response interruption capability
- **Thinking Mode Display**: Optional AI reasoning visualization
- **@-Mention File Integration**: Direct file and directory referencing in conversation
- **Slash Command System**: In-session commands for model switching, search, and task management
- **Session Resumption**: Seamless conversation continuation with unique session IDs
- **Vim-Mode Navigation**: Advanced keyboard shortcuts and navigation patterns
- **Image Upload Support**: Direct image processing and analysis capabilities
- **Integrated Todo Management**: Built-in task tracking within the TUI

### 4. Aircher Intelligence Engine (MCP Server)
**Detailed Specification**: `docs/architecture/plugins/aircher-intelligence-mcp-server.md`
**Performance Architecture**: `docs/architecture/plugins/modular-performance-architecture.md`

**Core MCP Tools**:
- `project_analyze` - Automatic project structure analysis and component detection
- `context_score_files` - AI-driven file relevance scoring for current task
- `task_detect` - Identify current development task type (debugging, feature, etc.)
- `dependency_graph` - Build and query file relationship networks
- `success_patterns` - Learn and apply historical success patterns
- `cross_project_insights` - Apply learnings from similar contexts across projects
- `smart_context_assembly` - Optimize context for AI tools based on token limits

**Technology Stack**:
- **Python Core**: MCP protocol, async coordination, AI model integration
- **Rust Performance Modules**: File system operations, AST parsing, pattern matching (via PyO3)
- **uvx Deployment**: Modern Python CLI tool deployment with automatic dependency management
- **Modular Backends**: Swappable Python/Rust/Mojo implementations for performance optimization

**Universal Compatibility**:
- Works with Claude Desktop, VS Code extensions, and any MCP-compatible tool
- Provides intelligent context management as a service
- Cross-project learning and pattern recognition
- Automatic task detection and file relevance scoring
- 10-50x performance improvements for critical operations

### 5. Multi-Database Storage Architecture
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

**Context Hierarchy** (enhanced with Claude Code session management):
```
Global: ~/.config/aircher/global.db          # User preferences, auth tokens
Project: .aircher/db/core/                     # Project-wide knowledge and patterns
Worktree: .aircher/worktrees/{worktree-id}/    # Branch-specific context
Session: .aircher/sessions/{session-id}/       # Resumable conversation state
Temp: .aircher/temp/{timestamp}/               # Temporary uploads and processing
```

**Session Management** (Claude Code pattern):
```rust
#[derive(Debug, Clone)]
pub struct SessionState {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub conversation_history: Vec<Message>,
    pub current_model: String,
    pub context_files: Vec<PathBuf>,
    pub todos: Vec<TodoItem>,
    pub thinking_mode: bool,
    pub web_search_enabled: bool,
    pub resume_token: String,
}
```

### 6. Universal LLM Provider System
**Detailed Specification**: `docs/architecture/plugins/llm-providers.md`

```rust
// Enhanced provider interface based on Codex/Oli patterns
pub trait LLMProvider: Send + Sync {
    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse, ProviderError>;
    async fn stream_chat(&self, req: &ChatRequest) -> Result<impl Stream<Item = ChatResponse>, ProviderError>;
    
    // Capability detection
    fn supports_functions(&self) -> bool;
    fn supports_system_messages(&self) -> bool;
    fn supports_images(&self) -> bool;
    fn supports_thinking(&self) -> bool;
    
    // Token and cost management
    fn get_token_limit(&self, model: &str) -> Option<usize>;
    async fn count_tokens(&self, content: &str) -> Result<usize, ProviderError>;
    fn calculate_cost(&self, tokens: usize, model: &str) -> Result<f64, ProviderError>;
    
    // Provider metadata
    fn name(&self) -> &str;
    fn models(&self) -> Vec<ModelInfo>;
    fn authentication_methods(&self) -> Vec<AuthMethod>;
}

// Enhanced model information structure with Warp-inspired fallback chains
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_length: usize,
    pub cost_per_input_token: f64,
    pub cost_per_output_token: f64,
    pub capabilities: ModelCapabilities,
    pub reliability_tier: ReliabilityTier,
    pub fallback_priority: u8,
}

#[derive(Debug, Clone)]
pub enum ReliabilityTier {
    Primary,   // claude-4-sonnet
    Fallback,  // claude-3.7-sonnet, gemini-2.5-pro
    Emergency, // gpt-4.1, deepseek-r1
}

// Warp-inspired model fallback strategy
#[derive(Debug, Clone)]
pub struct ModelFallbackChain {
    pub primary: String,
    pub fallback_chain: Vec<String>,
    pub retry_failed_tool_calls_with_different_model: bool,
    pub max_retries_per_model: u32,
}

// Authentication method enumeration (enhanced from OpenCode)
#[derive(Debug, Clone)]
pub enum AuthMethod {
    ApiKey { 
        env_var: String,
        validation_endpoint: Option<String>,
    },
    OAuth { 
        scopes: Vec<String>,
        client_id: String,
        auth_url: String,
        token_url: String,
    },
    LocalModel { 
        path: String,
        model_type: LocalModelType,
    },
    GitHubCopilot {
        device_flow: bool,  // GitHub device flow for CLI
    },
}

// OAuth implementation for GitHub Copilot (inspired by OpenCode)
#[derive(Debug)]
pub struct GitHubOAuth {
    client_id: String,
    device_code: Option<String>,
    access_token: Option<String>,
}

impl GitHubOAuth {
    pub async fn start_device_flow(&mut self) -> Result<DeviceFlowResponse, OAuthError> {
        // Implement GitHub device flow for CLI authentication
    }
    
    pub async fn poll_for_token(&mut self, device_code: &str) -> Result<AccessToken, OAuthError> {
        // Poll GitHub for access token
    }
}
```

**Supported Providers** (enhanced from reference implementations):
- ✅ **OpenAI**: GPT-3.5, GPT-4, GPT-4 Turbo models with API key authentication
- ✅ **Anthropic Claude**: Claude-3 Haiku, Sonnet, Opus models with API key authentication
- 🚧 **Google Gemini**: Gemini Pro, Gemini Pro Vision with OAuth and API key support
- 🚧 **GitHub Copilot**: Integration via OAuth (inspired by OpenCode)
- 🚧 **Ollama**: Local model hosting with automatic discovery
- 🚧 **OpenRouter**: Multi-provider aggregation with cost optimization
- 🚧 **Custom Providers**: User-defined provider configurations

### 7. Intelligent Context Management
**Detailed Specification**: `docs/architecture/plugins/context-management.md`

**Task Detection System**:
```rust
use notify::Watcher;
use std::sync::Arc;

pub struct TaskDetector {
    patterns: Vec<TaskPattern>,
    file_watcher: Arc<FileWatcher>,
    git_watcher: Arc<GitWatcher>,
    behavior_analyzer: Arc<BehaviorAnalyzer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskType {
    Debugging,
    FeatureDevelopment,
    Refactoring,
    Documentation,
    Testing,
    Maintenance,
}

impl TaskType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskType::Debugging => "debugging",
            TaskType::FeatureDevelopment => "feature_development",
            TaskType::Refactoring => "refactoring",
            TaskType::Documentation => "documentation",
            TaskType::Testing => "testing",
            TaskType::Maintenance => "maintenance",
        }
    }
}
```

**File Relevance Engine**:
```rust
use std::collections::HashMap;
use std::path::PathBuf;

pub struct FileRelevanceEngine {
    dependency_graph: Arc<DependencyGraph>,
    access_patterns: Arc<AccessPatternAnalyzer>,
    task_context: Arc<TaskContext>,
    relevance_scorer: Arc<RelevanceScorer>,
}

impl FileRelevanceEngine {
    pub async fn score_files(&self, files: &[PathBuf]) -> Result<HashMap<PathBuf, f64>, RelevanceError> {
        // Implementation for intelligent file relevance scoring
        todo!()
    }
    
    pub fn update_access_pattern(&self, file: &PathBuf, access_type: AccessType) {
        // Track file access patterns for learning
    }
}
```

**Smart Conversation Compaction**:
- Automatic context optimization based on task completion
- Intelligent message importance scoring
- Configurable preservation rules for critical information

### 8. Advanced Tool System & Security
**Detailed Specification**: `docs/architecture/plugins/mcp-integration.md`

```rust
// Enhanced tool system inspired by Gemini CLI
pub trait Tool: Send + Sync {
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError>;
    fn validate_params(&self, params: &ToolParams) -> Result<(), ValidationError>;
    fn requires_confirmation(&self, params: &ToolParams) -> bool;
    fn security_level(&self) -> SecurityLevel;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

// MCP integration with enhanced security
pub struct MCPManager {
    local_servers: HashMap<String, MCPServer>,
    project_servers: HashMap<String, MCPServer>,
    user_servers: HashMap<String, MCPServer>,
    permission_system: PermissionSystem,
    sandbox_manager: SandboxManager,
}

// Security levels for tool execution
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel {
    Safe,        // Read-only operations
    Moderate,    // Limited write operations
    Dangerous,   // Destructive operations
    System,      // System-level changes
}
```

**Enhanced Tool Ecosystem** (inspired by reference implementations):

**Built-in Tools** (enhanced with Claude Code patterns):
- **filesystem**: File operations with @-mention integration and validation
- **web-search**: Automatic and manual search with multiple providers (Brave, Google)
- **git**: Repository operations with safety checks and diff analysis
- **github**: API integration with OAuth and issue management
- **database**: SQLite/PostgreSQL operations with query validation
- **shell**: Command execution with platform-specific sandboxing
- **memory**: Conversation and context management with session resumption
- **image**: Image upload, processing, and analysis capabilities
- **todo**: Integrated task management with /todo slash commands
- **thinking**: AI reasoning display and interleaved thinking mode

**Comprehensive Security Model** (inspired by Codex):

```rust
// Security policy configuration
#[derive(Debug, Clone, PartialEq)]
pub enum ApprovalPolicy {
    Never,      // Never execute without explicit approval
    Ask,        // Prompt user for each execution
    Auto,       // Auto-approve based on safety analysis
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel {
    Safe,       // Read-only operations (ls, cat, grep)
    Moderate,   // Limited write operations (mkdir, touch)
    Dangerous,  // Destructive operations (rm, mv, git reset)
    System,     // System-level changes (sudo, chmod +x)
}

// Platform-specific sandboxing
pub trait SandboxProvider {
    async fn execute_sandboxed(&self, command: &Command, policy: &SecurityPolicy) -> Result<Output, SandboxError>;
    fn supported_on_platform(&self) -> bool;
}

// macOS Seatbelt implementation
pub struct MacOSSandbox {
    seatbelt_profile: String,
    allowed_paths: Vec<PathBuf>,
}

// Linux Landlock implementation  
pub struct LinuxSandbox {
    landlock_ruleset: LandlockRuleset,
    allowed_paths: Vec<PathBuf>,
}
```

**Security Features**:
- **Approval Policies**: Never/Ask/Auto based on command risk analysis
- **Platform-Specific Sandboxing**: 
  - macOS: Seatbelt profiles for process isolation
  - Linux: Landlock filesystem restrictions
  - Windows: Job objects and restricted tokens
- **Command Analysis**: Pre-execution safety classification
- **Audit Logging**: Complete execution history with timestamps
- **Permission Scoping**: Global, project, and session-level permissions
- **Safe Defaults**: Conservative security settings out-of-the-box

### 9. Project Analysis System
**Status**: ✅ **Completed**

```rust
use tracing::info;
use sqlx::SqlitePool;

pub struct ProjectAnalyzer {
    project_root: PathBuf,
    storage_engine: Arc<StorageEngine>,
    db_pool: SqlitePool,
}

impl ProjectAnalyzer {
    pub async fn analyze_project(&self) -> Result<ProjectAnalysis, AnalysisError> {
        info!("Starting project analysis for {:?}", self.project_root);
        // Implementation
    }
}
```

**Capabilities**:
- Automatic project structure analysis
- Documentation generation in `.aircher/project_analysis.md`
- Component detection and classification
- Cross-reference mapping

## Key Data Structures

### Chat Communication
```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub stream: bool,
    pub model: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<StreamResponse>,
    pub tokens_used: TokenUsage,
    pub cost: f64,
    pub duration: Duration,
    pub provider: String,
    pub model: String,
}
```

### File Relevance
```rust
use chrono::{DateTime, Utc};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRelevance {
    pub path: PathBuf,
    pub score: f64,
    pub last_accessed: DateTime<Utc>,
    pub access_frequency: u32,
    pub dependencies: Vec<PathBuf>,
    pub relevance_type: RelevanceType,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelevanceType {
    Direct,        // Directly modified or accessed
    Dependency,    // Import/dependency relationship
    Similar,       // Similar file type or content
    Related,       // Same directory or project area
}
```

## Configuration System

**Architecture**: Hierarchical configuration system inspired by Codex with intelligent defaults and secure credential management

### Multi-Level Configuration Strategy (Codex Pattern)
**Precedence Order**: CLI args → profiles → project config → user config → defaults

- **User Configuration**: `~/.config/aircher/config.toml` - Global preferences and defaults
- **Credentials**: `~/.config/aircher/credentials.toml` - API keys and authentication (600 permissions)
- **Project Configuration**: `.aircher/config.toml` - Project-specific overrides
- **Profile System**: Named configuration sets for different workflows
- **Environment Variables**: Override any configuration value

### Profile System
```toml
# ~/.config/aircher/config.toml
[profiles.development]
model = "claude-3.5-haiku"  # Cost-efficient for dev work
auto_approval = false
sandbox_policy = "strict"

[profiles.production]
model = "claude-4-sonnet"   # Premier coding model
auto_approval = true
sandbox_policy = "relaxed"

[profiles.research]
model = "claude-4-opus"     # Top-tier reasoning model
web_search = true
```

### MVP Configuration Approach
```toml
# ~/.config/aircher/config.toml - Minimal user preferences
[providers]
default = "anthropic"         # Claude models as primary
fallback_enabled = true       # automatically fallback if primary fails

[models]
auto_select = true            # intelligent model selection based on task
anthropic_default = "claude-4-sonnet"    # Premier coding model
google_default = "gemini-2.5-pro"        # High-performance alternative
openai_default = "gpt-4o"                # General purpose
openrouter_default = "deepseek/deepseek-r1-0528"  # Latest reasoning model

# Task-specific model overrides
[models.tasks]
summaries = "claude-3.5-haiku"          # Fast tasks: commits, docs, context compression
coding = "claude-4-sonnet"              # Main development: review, debug, implement, test
research = "claude-4-opus"              # Complex reasoning: architecture, exploration

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
- **Model Selection**: Intelligent defaults (Claude-4-Sonnet, Claude-4-Opus, o3, GPT-4o, Gemini-2.5-Pro, DeepSeek-R1)
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
- Configuration system with smart defaults

### 🚧 Phase 2: Core Intelligence Engine
**Priority**: Build the universal MCP server for intelligent context management
- **MCP Server Framework**: Protocol implementation and tool architecture
- **Context Intelligence**: File relevance scoring and task detection algorithms
- **Cross-Project Learning**: Pattern recognition and success correlation
- **Universal Compatibility**: Works with Claude Desktop, VS Code, any MCP tool
- **LLM Provider Integration**: OpenAI and Claude API implementations

### 🚧 Phase 3: Terminal Assistant
**Dependencies**: Phase 2 intelligence foundation
- **REPL-Style Interface**: Interactive terminal session with natural language
- **Session Management**: Resumable conversations with unique session IDs
- **Advanced Interaction**: Real-time steering, @-mention files, slash commands
- **Streaming Integration**: Real-time response rendering in terminal UI
- **Intelligence Integration**: Use MCP server for context management

### 🚧 Phase 4: Advanced Features
**Dependencies**: Phase 2-3 core functionality
- **Security Framework**: Comprehensive permissions and audit logging
- **Web Search Integration**: Automatic and manual search capabilities
- **Cross-Project Insights**: Learnings and patterns across entire codebase
- **Team Collaboration**: Shared insights while maintaining privacy
- **Performance Optimization**: Caching, connection pooling, async processing

### ❌ Phase 5: Production & Distribution
**Dependencies**: All previous phases
- **Comprehensive Testing**: Security, performance, integration tests
- **Distribution**: Homebrew, cargo install, direct download
- **Auto-Update System**: Seamless version management
- **Enterprise Features**: Advanced monitoring and cost management
- **Documentation**: Complete user guides and API documentation

### Phase Rationale:
1. **Intelligence First**: The MCP server provides the core differentiator - intelligent context management
2. **Universal Compatibility**: Focus on working everywhere rather than just terminal
3. **Cross-Project Learning**: This is the key value proposition that competitors can't easily replicate
4. **Terminal as Consumer**: Terminal assistant uses the same intelligence engine as other tools
5. **Incremental Rollout**: Start with universal intelligence, then add terminal-specific features

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

### Core Framework (Pure Rust Implementation)
```toml
# Core application framework
ratatui = "0.24"                     # TUI framework
crossterm = "0.27"                   # Terminal control
tokio = { version = "1.34", features = ["full"] }  # Async runtime

# Database and storage
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls"] }

# LLM provider clients
reqwest = { version = "0.11", features = ["json", "stream"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Configuration and serialization  
toml = "0.8"                         # Configuration parsing
config = "0.13"                      # Hierarchical configuration

# Security and sandboxing (critical for Phase 2)
nix = "0.27"                         # Unix system calls
landlock = "0.3"                     # Linux sandboxing (requires kernel 5.13+)
libc = "0.2"                         # Low-level system interface

# Authentication and OAuth
oauth2 = "4.4"                       # OAuth2 client implementation
url = "2.4"                          # URL parsing
base64 = "0.21"                      # Base64 encoding for tokens

# Error handling and logging
thiserror = "1.0"                    # Error derive macros
tracing = "0.1"                      # Structured logging
tracing-subscriber = "0.3"           # Log output formatting

# Performance and caching
dashmap = "5.5"                      # Concurrent hashmap
lru = "0.12"                         # LRU cache implementation

# Terminal UI enhancements
syntect = "5.0"                      # Syntax highlighting
unicode-width = "0.1"                # Text width calculation
```


### Development Tools
```toml
# Development and testing
cargo-tarpaulin = "0.27"             # Code coverage
cargo-dist = "0.4"                   # Release distribution
cargo-deny = "0.14"                  # Security and license auditing
cargo-audit = "0.18"                 # Security vulnerability scanning
```

## Performance Targets

### Response Time Requirements
- **Startup Time**: < 100ms cold start, < 50ms warm start
- **First Token**: < 500ms for streaming responses  
- **Terminal Rendering**: 60fps for smooth scrolling
- **Context Loading**: < 200ms for typical projects (< 10MB)
- **Model Switch**: < 100ms provider/model switching

### Resource Usage Limits
- **Memory**: < 50MB baseline, < 200MB with large contexts
- **CPU**: < 5% idle, burst to 100% during LLM calls
- **Disk**: < 100MB for databases and cache
- **Network**: Efficient connection pooling and reuse

## Installation & Distribution

### Installation Methods
```bash
# Cargo (Rust users)
cargo install aircher

# Homebrew (macOS/Linux)
brew install aircher

# Direct download (all platforms)
curl -L https://github.com/user/aircher/releases/latest/download/aircher-$(uname -s)-$(uname -m) -o aircher
```

### Auto-Update System
```rust
pub struct UpdateManager {
    current_version: semver::Version,
    update_channel: UpdateChannel,  // Stable, Beta, Nightly
    auto_check: bool,
}

#[derive(Debug, Clone)]
pub enum UpdateChannel {
    Stable,   // Weekly releases
    Beta,     // Daily releases
    Nightly,  // Every commit
}
```

## Testing Strategy

### Test Coverage Requirements
- **Unit Tests**: > 80% line coverage for core logic
- **Integration Tests**: All LLM provider implementations
- **End-to-End Tests**: Key user workflows (chat, file operations)
- **Security Tests**: Sandbox bypass attempts, privilege escalation
- **Performance Tests**: Load testing with large contexts

### Test Structure
```rust
// Unit tests co-located with implementation
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_provider_fallback() {
        // Test provider failover logic
    }
}

// Integration tests in tests/ directory
// tests/integration/llm_providers.rs
// tests/integration/security_sandbox.rs
// tests/integration/config_management.rs
```

## Error Handling & Recovery

### Error Hierarchy
```rust
#[derive(thiserror::Error, Debug)]
pub enum AircherError {
    #[error("LLM provider error: {0}")]
    Provider(#[from] ProviderError),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Security violation: {0}")]
    Security(#[from] SecurityError),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Recovery Strategies
- **Provider Failures**: Automatic fallback to secondary providers
- **Network Issues**: Exponential backoff with circuit breaker
- **Configuration Errors**: Safe defaults with user notification
- **Security Violations**: Immediate termination with audit log

## Build Commands

```bash
# Development
cargo build --release    # Build optimized binary
cargo run               # Run development version
cargo test              # Run all tests
cargo clippy            # Lint with clippy
cargo fmt               # Format code
cargo tarpaulin         # Coverage reports

# Release
cargo dist build        # Build release artifacts
cargo dist upload       # Upload to GitHub releases
```

## Reference Architecture Analysis

Based on comprehensive analysis of leading AI coding assistants, our architecture incorporates proven patterns:

### Reference Repositories (see `external/`)
- **Codex** (`external/codex/`): Rust + TypeScript hybrid with sophisticated provider system
- **Oli** (`external/oli/`): Modern Rust backend with React/Ink frontend  
- **OpenCode** (`external/opencode/`): Go TUI with TypeScript server architecture
- **Gemini CLI** (`external/gemini-cli/`): TypeScript with comprehensive tool system
- **Claude Code**: Commercial AI terminal assistant (analyzed via changelog + docs)

### Key Architectural Insights Applied
1. **REPL-First Design (Claude Code)**: Interactive terminal session as primary interface
2. **Real-Time Interaction (Claude Code)**: Message steering while AI is responding
3. **Context Integration (Claude Code)**: @-mention files, slash commands, thinking mode
4. **Session Management (Claude Code)**: Resumable conversations with unique IDs
5. **Provider Abstraction (Codex/Oli)**: Multiple authentication methods and fallback systems
6. **Pure Rust Performance (Oli)**: Native terminal performance without hybrid complexity
7. **Security by Design (Codex)**: Platform-specific sandboxing and approval policies
8. **Tool Ecosystem (Gemini CLI)**: Extensible tool system with MCP integration

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
