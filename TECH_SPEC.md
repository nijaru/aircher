# Aircher Technical Specification

## Overview

Aircher is a terminal-based AI assistant with advanced semantic code search capabilities. Built in Rust with a focus on performance, multi-provider support, and extensible tool integration.

## Core Architecture

### System Components

```rust
// High-level architecture
TUI (Ratatui) ──→ Provider Manager ──→ LLM APIs
     │                    │               │
     ↓                    ↓               ↓
Agent Controller ←─ Tool Registry    Streaming
     │                    │          
     ↓                    ↓          
Semantic Search    File Operations
```

### Key Technologies

- **Language**: Rust 2021 edition
- **TUI Framework**: Ratatui for terminal interface
- **Vector Search**: hnswlib-rs for high-performance HNSW
- **Language Parsing**: tree-sitter for 19+ languages
- **Async Runtime**: Tokio for concurrency
- **Serialization**: serde for configuration/data

## Provider System (`src/providers/`)

### Multi-Provider Architecture
Unified interface supporting:
- OpenAI (GPT-4, GPT-4o, GPT-4o-mini)
- Anthropic (Claude Opus 4.1, Sonnet 4, Haiku)
- Google (Gemini 1.5 Pro, Flash)
- Ollama (local models)

```rust
pub trait LLMProvider {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse>;
    async fn stream_chat(&self, request: &ChatRequest) -> Result<ChatStream>;
    fn get_models(&self) -> Vec<ModelInfo>;
}
```

### Authentication
- API key management through config
- OAuth2 support (Anthropic Pro/Max)
- Secure credential storage

## Semantic Search Engine (`src/intelligence/`)

### Vector Database
- **Backend**: hnswlib-rs for high performance
- **Embeddings**: Configurable model support
- **Languages**: 19+ with tree-sitter parsing
- **Performance**: 45x faster indexing, sub-millisecond search

### Language Support
```
Systems: Rust, C, C++, Go, Zig
Web: JavaScript, TypeScript, Python, Ruby, PHP
Enterprise: Java, C#, Kotlin, Swift
Others: Bash, SQL, TOML, JSON, YAML, Markdown
```

### Search Features
- Semantic similarity search
- File type filtering
- Scope filtering (functions, classes, etc)
- Similarity thresholds
- Query expansion and typo correction

## Agent System (`src/agent/`)

### Current State
Agent system is implemented but not connected to TUI. See `docs/CURRENT_STATE.md` for gap analysis.

### Tool Registry
Available tools (implemented but unused):
- File operations: read_file, write_file, edit_file
- Code analysis: search_code, find_definition
- System: run_command, git_status
- Permission handling: approval flow

### Future Architecture (Phase 1-6)
See `docs/architecture/roadmap.md` for detailed implementation plan.

## User Interface (`src/ui/`)

### TUI Components
- **Chat Interface**: Conversation view with streaming
- **Model Selection**: Dynamic provider/model switching
- **Configuration**: Settings and preferences
- **Help System**: Interactive documentation
- **Status Display**: Connection, cost, context usage

### Key Features
- Multi-line input with terminal shortcuts
- Real-time streaming responses
- Dynamic model fetching from providers
- Authentication wizard
- Auto-compaction warnings

### Keyboard Shortcuts
```
Ctrl+A/E    - Line start/end
Ctrl+W      - Delete word
Ctrl+K/U    - Delete to line end/start  
Alt+B/F     - Word navigation
Ctrl+L      - Jump to chat bottom
Ctrl+M      - Model selection
/model      - Provider selection
/search     - Semantic search
```

## Configuration System (`src/config/`)

### Hierarchical Configuration
```
Hardcoded defaults → Global config → Project config → Environment
```

### Key Settings
```toml
[global]
provider = "anthropic"
model = "claude-opus-4.1"

[providers.anthropic]
api_key = "sk-..."

[search]
embedding_model = "text-embedding-3-small"
default_limit = 10

[ui]
theme = "default"
auto_compaction = true
```

## Performance Characteristics

### Search Performance
- **First search**: 15-20s (builds index)
- **Subsequent searches**: 0.02s (cached)
- **Index size**: ~50MB for 3,000 chunks
- **Memory usage**: <200MB typical

### Indexing Performance
- **hnswlib-rs**: 45.4x faster than previous backend
- **Batch processing**: Handles large codebases efficiently
- **Background updates**: File change monitoring

## Build System

### Dependencies
Key crates:
- `ratatui` - Terminal UI
- `tokio` - Async runtime  
- `serde` - Serialization
- `anyhow` - Error handling
- `hnswlib-rs` - Vector search
- `tree-sitter` - Code parsing
- `reqwest` - HTTP client

### Build Configuration
```toml
# Cargo.toml features
default = ["search", "tui"]
mcp = ["mcp-client"] # MCP integration
```

## Future Architecture

### Phase 1: Agent Integration
- Connect AgentController to TUI
- Implement tool calling loop
- Add tool execution UI

### Phase 2: Enhanced Tools
- File operations integration
- Git workflow support
- Test execution

### Phase 3: Turbo Mode
See `docs/architecture/turbo-mode.md` for detailed design.

## Security Model

### Sandboxing
- Command execution requires approval
- File operations are scoped to workspace
- API keys stored securely

### Permission System
- Interactive approval for dangerous operations
- Configurable approval modes
- Audit trail for all tool usage

## Testing Strategy

### Test Structure
```
tests/
├── integration/    # End-to-end tests
├── performance/   # Benchmarks
└── unit/         # Component tests
```

### Benchmarking
- Performance tests with production data
- Memory usage profiling  
- Search accuracy metrics

## Deployment

### Installation
```bash
# From source
git clone https://github.com/user/aircher
cd aircher && cargo build --release

# From binary
curl -fsSL install.sh | sh
```

### System Requirements
- Rust toolchain 1.70+
- 4GB RAM minimum
- Terminal with true color support

---

*For development roadmap see `docs/architecture/roadmap.md`*
*For current implementation status see `STATUS.md`*