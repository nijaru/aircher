# Rust Terminal Client Structure Guide

**Note**: This document covers the Rust terminal client only. The Aircher Intelligence Engine MCP server is implemented in Python with Rust performance modules. See `docs/architecture/plugins/aircher-intelligence-mcp-server.md` for the Python server architecture.

## Workspace Layout

```toml
[workspace]
members = ["aircher-core", "aircher-tui", "aircher-cli"]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1.0", features = ["full"] }
ratatui = { version = "0.30", features = ["crossterm"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
serde = { version = "1.0", features = ["derive"] }
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls"] }
tracing = "0.1"
crossterm = "0.27"
thiserror = "1.0"
async-trait = "0.1"
```

## Terminal Client Module Architecture

The Rust terminal client focuses on UI, user interaction, and MCP client functionality to communicate with the Python Intelligence Engine.

```rust
src/
├── main.rs                 # CLI entry point for terminal client
├── lib.rs                  # Library root
├── core/
│   ├── mod.rs
│   ├── domain.rs           # Business logic for terminal operations
│   ├── providers.rs        # LLM provider traits
│   └── storage.rs          # Database layer
├── infrastructure/
│   ├── mod.rs
│   ├── tui/                # Terminal interface (Ratatui)
│   ├── config.rs           # Configuration management
│   └── mcp_client.rs       # MCP client to communicate with Python Intelligence Engine
├── application/
│   ├── mod.rs
│   ├── services.rs         # Application services
│   └── handlers.rs         # Command handlers
└── providers/
    ├── mod.rs
    ├── openai.rs
    ├── claude.rs
    ├── gemini.rs
    └── ollama.rs
```

## Core Traits

### LLM Provider Interface (Terminal Client)
```rust
#[async_trait]
pub trait LLMProvider: Send + Sync + Debug {
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, LLMError>;
    async fn chat_stream(&self, req: ChatRequest)
        -> Result<Pin<Box<dyn Stream<Item = Result<ChatToken, LLMError>> + Send>>, LLMError>;
    
    fn supports_functions(&self) -> bool;
    fn supports_system_messages(&self) -> bool;
    fn get_token_limit(&self, model: &str) -> u32;
    fn count_tokens(&self, content: &str) -> Result<u32, LLMError>;
    fn name(&self) -> &str;
    fn models(&self) -> &[String];
}
```

### MCP Client Interface (Terminal to Intelligence Engine)
```rust
#[async_trait]
pub trait MCPClient: Send + Sync + Debug {
    async fn call_tool(&self, tool_name: &str, params: serde_json::Value) -> Result<MCPResponse, MCPError>;
    async fn list_tools(&self) -> Result<Vec<MCPTool>, MCPError>;
    async fn list_resources(&self) -> Result<Vec<MCPResource>, MCPError>;
    
    fn is_connected(&self) -> bool;
    async fn connect(&mut self) -> Result<(), MCPError>;
    async fn disconnect(&mut self) -> Result<(), MCPError>;
}

// Integration with Python Intelligence Engine
pub struct AircherIntelligenceClient {
    // Connection to Python MCP server via stdio/http
    transport: MCPTransport,
    tool_registry: Vec<MCPTool>,
}

impl AircherIntelligenceClient {
    pub async fn project_analyze(&self, path: &str) -> Result<ProjectAnalysis, MCPError> {
        let params = serde_json::json!({ "path": path });
        let response = self.call_tool("project_analyze", params).await?;
        Ok(serde_json::from_value(response.result)?)
    }
    
    pub async fn context_score_files(&self, files: &[String], task_context: &str) -> Result<Vec<FileRelevanceScore>, MCPError> {
        let params = serde_json::json!({
            "files": files,
            "task_context": task_context
        });
        let response = self.call_tool("context_score_files", params).await?;
        Ok(serde_json::from_value(response.result)?)
    }
}
```

### Database Architecture
```rust
pub struct DatabaseManager {
    conversations: SqlitePool,  // conversations.db
    knowledge: SqlitePool,      // knowledge.db
    file_index: SqlitePool,     // file_index.db
    sessions: SqlitePool,       // sessions.db
}
```

### Configuration Structure
```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub providers: ProvidersConfig,
    pub context_management: ContextConfig,
    pub mcp_client: MCPClientConfig,
    pub storage: StorageConfig,
    pub tui: TUIConfig,
}

#[derive(Deserialize, Validate)]
pub struct ProvidersConfig {
    #[validate(url)]
    pub openai_base_url: Option<String>,
    #[validate(length(min = 1))]
    pub default_provider: String,
    pub api_keys: HashMap<String, String>,
}
```

## Error Handling
```rust
#[derive(Debug, thiserror::Error)]
pub enum AircherError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("LLM provider error: {0}")]
    Provider(#[from] LLMError),
    #[error("TUI error: {0}")]
    TUI(#[from] std::io::Error),
    #[error("MCP client error: {0}")]
    MCP(#[from] MCPError),
}

#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("API error: {message} (status: {status})")]
    API { message: String, status: u16 },
    #[error("Rate limit exceeded: retry after {seconds}s")]
    RateLimit { seconds: u64, provider: String },
    #[error("Token limit exceeded: {used}/{limit} tokens")]
    TokenLimit { used: u32, limit: u32, model: String },
    #[error("Streaming interrupted: {reason}")]
    StreamInterrupted { reason: String },
}
```

## Application State
```rust
pub struct App {
    state: AppState,
    config: Config,
    storage: Arc<DatabaseManager>,
    providers: ProviderRegistry,
    event_tx: mpsc::UnboundedSender<AppEvent>,
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    TokenReceived(ChatToken),
    StreamError(LLMError),
    UserInput(String),
    Quit,
}
```

## Key Dependencies

### Core Framework
- `tokio` - Async runtime
- `ratatui` + `crossterm` - TUI framework
- `sqlx` - Database operations
- `serde` + `toml` - Configuration

### LLM Integration
- `reqwest` - HTTP client for APIs
- `async-openai` - OpenAI SDK
- Custom implementations for Claude, Gemini, Ollama

### Development
- `tracing` + `tracing-subscriber` - Logging
- `thiserror` - Error handling
- `async-trait` - Async traits
- `clap` - CLI parsing

## Build Setup
```bash
# Initialize workspace
cargo init --lib aircher-core
cargo init --bin aircher-cli  
cargo init --lib aircher-tui

# Development commands
cargo build --release
cargo test
cargo clippy
cargo fmt

# Warp-inspired benchmark testing
cargo test --test swe_bench_integration
cargo test --test terminal_bench_integration
cargo bench --bench edit_performance
cargo bench --bench interactive_command_latency
```
