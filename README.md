# Aircher (pronounced "archer")

**AI-powered terminal assistant built with Rust** - Intelligent command-line interface with multi-LLM support, real-time interaction, and extensible tool ecosystem.

## What is Aircher?

Aircher is a **pure Rust terminal-based AI assistant** that brings the power of multiple LLM providers directly to your command line. Think of it as your personal AI coding assistant that lives in your terminal, with advanced features like real-time message steering, session resumption, and an extensible tool ecosystem.

## ✨ Key Features

### 🚀 **REPL-Style Terminal Interface**
- **Interactive Sessions** → Direct terminal-based AI assistant with natural language commands
- **Real-Time Steering** → Send messages while AI is responding to guide output
- **Session Resumption** → Seamless conversation continuation with `aircher --resume`
- **ESC Key Interruption** → Immediate response interruption capability
- **Context Usage Display** → Real-time token usage (e.g., "44k/200k tokens")
- **Pure Rust Performance** → Ratatui TUI with smooth streaming responses

### 🎯 **Advanced Interaction Features**
- **Slash Commands** → `/help`, `/clear`, `/resume`, `/switch-model`, `/web-search`, `/thinking`
- **@-Mention Integration** → Reference files and directories directly (`@README.md`, `@src/`)
- **Thinking Mode** → Optional AI reasoning visualization
- **Image Processing** → Upload and analyze images within conversations
- **Todo Management** → Built-in task tracking with `/todo` commands
- **Web Search** → Automatic and manual search capabilities

### 🤖 **Multi-Provider LLM Support**
- **Universal Interface** → OpenAI, Claude, Gemini, GitHub Copilot, Ollama
- **Provider Abstraction** → Trait-based design with multiple authentication methods
- **Smart Model Selection** → Task-specific optimization for cost efficiency
- **Streaming Support** → Real-time responses across all providers
- **Fallback System** → Automatic provider failover with graceful degradation
- **Cost Tracking** → Real-time usage monitoring and budget management

### 🧠 **Intelligent Context Management**
- **File Relevance Engine** → AI-driven scoring based on task context and dependencies
- **Task Detection** → Automatic identification of current work (debugging, features, refactoring)
- **Smart Compaction** → Preserve important context while optimizing token usage
- **Hierarchical Storage** → Global → Project → Worktree → Session organization

### 🔍 **Web Search & Information Retrieval**
- **Auto Search Triggers** → Detects when queries need fresh information
- **Multi-Provider Search** → Brave, Google, DuckDuckGo integration
- **Temporal Awareness** → Prioritizes current documentation and solutions
- **Error Solution Search** → Proactive search for encountered errors

### 🛠️ **Security & Tool Ecosystem**
- **Platform Sandboxing** → macOS Seatbelt, Linux Landlock, Windows Job Objects
- **Approval Policies** → Never/Ask/Auto system with command risk analysis
- **MCP Integration** → Model Context Protocol for extensible tool support
- **Built-in Tools** → Filesystem, Git, Web Search, Database, Image processing
- **Security by Design** → Comprehensive audit logging and permission scoping

### 📊 **Multi-Database Storage**
- **Specialized Databases** → Conversations, knowledge, file_index, sessions
- **Context Hierarchy** → Global, project, worktree, and session-specific storage
- **Session Management** → Resumable conversations with unique session IDs
- **Hybrid Storage** → SQLite for metadata, filesystem for large content

## 🚀 Installation

### **From Source (Currently Only Option)**
```bash
git clone https://github.com/aircher/aircher.git
cd aircher
cargo build --release
sudo mv target/release/aircher /usr/local/bin/
```

> **Note**: Cargo install and pre-built binaries will be available once the project reaches initial release.

## ⚡ Quick Start

### **Step 1: Authentication**
```bash
# Authentication system in development
aircher login         # (planned)
aircher login openai  # (planned)
aircher login claude  # (planned)
```

### **Step 2: Start Session**
```bash
# REPL interface in development
aircher                 # (in development)
aircher --resume        # (planned)
aircher --service openai # (planned)
```

### **Step 3: Start Chatting**
```bash
# In the REPL session:
/help                    # Show available commands
@README.md explain this  # Reference files with @-mentions
/web-search rust async   # Manual web search
/thinking                # Toggle AI reasoning display
/todo add "fix bug"      # Add todo item
```

## 💡 Usage Examples

### **REPL-Style Terminal Interface**
```bash
$ aircher
┌─ 🏹 Aircher ────────────────────────── Session: abc123 ─┐
│ Provider: claude-4-sonnet • Tokens: 44k/200k • 🟢 Ready        │
│                                                                │
│ 👤 You [14:32:20]                                             │
│ @src/auth.rs explain the authentication system                 │
│                                                                │
│ 🤖 Claude [14:32:21] 🧠 Thinking...                          │
│ Looking at your auth.rs file, I can see a JWT-based system... │
│                                                                │
│ ```rust                                                        │
│ // JWT middleware implementation                               │
│ pub fn auth_middleware() -> impl Filter<Extract = ...> {      │
│     warp::header::<String>("authorization")                    │
│         .and_then(validate_jwt)                                │
│ }                                                              │
│ ```                                                            │
│                                                                │
└────────────────────────────────────────────────────────────────┘
┌─ > /thinking on                                              ─┐
└─ ESC: Interrupt • /help: Commands • Ctrl+C: Exit ────────────────┘
```

### **Advanced Interaction Features**
```bash
# Session management
> aircher --resume              # Resume last conversation
> /resume abc123               # Resume specific session
> ESC                          # Interrupt AI mid-response

# File integration
> @README.md what does this project do?
> @src/ show me the main modules
> @git:HEAD~1 what changed?

# Real-time steering
> explain rust ownership [while AI responds: "focus on borrowing"]

# Slash commands
> /thinking                     # Toggle reasoning display
> /web-search rust async 2024  # Manual web search
> /todo add "implement auth"    # Task management
> /switch-model gpt-4          # Change model mid-conversation
```

### **Tool Integration & MCP**
```bash
# Built-in tools (no setup required)
> show me the database schema
> create a git branch for this feature
> search for "rust async patterns" on the web
> analyze this image @screenshot.png

# MCP server integration
> aircher mcp install github    # Install GitHub MCP server
> create a pull request for the current branch
> aircher mcp install postgres  # Database integration
> show me recent migrations
```

### **Interactive Commands & Shortcuts**

#### **Slash Commands**
```bash
/help                    # Toggle help panel
/clear                   # Clear conversation
/config                  # Settings management
/cost                    # Show usage statistics
/memory                  # Edit AGENTS.md memory
/search [query]          # Force web search
/think                   # Toggle thinking mode
/mcp                     # MCP server management
/tools                   # List available MCP tools
```

#### **Keyboard Shortcuts**
```bash
Ctrl+H                   # Toggle help panel
Ctrl+T                   # Toggle context sidebar
Ctrl+C / Esc            # Exit Aircher
Enter                    # Send message
```

#### **TUI Features**
- **Live Streaming**: See responses appear in real-time
- **Markdown Rendering**: Beautiful code highlighting and formatting
- **Context Panel**: View session info, costs, and available tools
- **Status Indicators**: Visual feedback for thinking/searching states
- **Responsive Layout**: Adapts to terminal size automatically

### **Automation & Scripting**
```bash
# Code review automation
git diff | aircher -p "review this code" --output-format json

# Batch processing
find . -name "*.go" | xargs -I {} aircher -p "add error handling to {}"

# Pipeline integration
aircher -p "check test coverage" --max-turns 1
```

## ⚙️ Configuration

Aircher uses **TOML configuration files** with intelligent defaults. Initialize with:

```bash
# Configuration system in development
aircher config  # (planned)
```

### **Configuration Locations**
- **User-global**: `~/.config/aircher/config.toml`
- **Project-specific**: `.agents/config.toml` (planned)
- **Credentials**: `~/.config/aircher/credentials.toml` (planned)

### **Key Configuration Options**

```toml
# ~/.config/aircher/config.toml
[providers]
default = "anthropic"         # Claude models as primary
fallback_enabled = true       # Automatic failover

[models]
auto_select = true            # Task-based model selection
anthropic_default = "claude-4-sonnet"
google_default = "gemini-2.5-pro"
openai_default = "gpt-4o"
openrouter_default = "deepseek-r1-0528"

# Task-specific optimization
[models.tasks]
summaries = "claude-3.5-haiku"        # Fast tasks: commits, docs, context compression
coding = "claude-4-sonnet"            # Main development: review, debug, implement, test
research = "claude-4-opus"            # Complex reasoning: architecture, exploration

[interface]
show_thinking = true          # Show AI reasoning
show_context_usage = true     # Display token usage
streaming = true             # Real-time responses

[context]
max_files = 20               # Intelligent management
auto_compaction = true       # Automatic optimization

[costs]
monthly_budget = 100.0       # Budget tracking
track_usage = true
prefer_cost_efficient = true # Auto cost optimization
```

## 🧠 Project Memory (AGENTS.md)

Create an **`AGENTS.md`** file in your project root for AI agent knowledge:

```markdown
# Project Name - AI Agent Memory

## Instructions
- This project uses Rust 1.80+ with Ratatui TUI framework
- Follow Rust best practices and clippy recommendations
- Use tokio for async operations

## Conventions
- Use snake_case for functions and variables
- Implement traits for testability
- Error handling with Result<T, E>

## Commands
- `cargo build --release` - Build optimized binary
- `cargo test` - Run all tests
- `cargo clippy` - Run linter

## Architecture
- Clean architecture with trait-based design
- Multi-database pattern (conversations, knowledge, sessions)
- Provider abstraction for LLM integration

## Glossary
- **Provider**: LLM service abstraction (OpenAI, Claude, etc.)
- **Session**: Resumable conversation state
- **Context**: File and conversation relevance system
```

## 🛠️ Development

### **Requirements**
- Rust 1.80+ (leverages latest async/await and trait features)
- SQLite (included)
- Node.js (for MCP servers)
- Modern terminal with color support
- Recommended: Terminal with Unicode support for best experience

### **Building from Source**

```bash
git clone https://github.com/aircher/aircher.git
cd aircher
cargo build --release  # Optimized build with all features
```

### **Rust Features Used**

Aircher leverages modern Rust features for performance and safety:

- **Async/Await**: Tokio runtime for concurrent operations
- **Trait System**: Provider abstraction and extensibility
- **Memory Safety**: Zero-cost abstractions without garbage collection
- **Error Handling**: Comprehensive Result<T, E> error management
- **Performance**: Native speed with Ratatui TUI framework

### **Running Tests**

```bash
cargo test
cargo clippy  # Linting
cargo fmt     # Code formatting
```

### **Health Check**

```bash
aircher doctor  # (planned)
```

## 🤝 Contributing

1. **Fork** the repository
2. **Create** a feature branch
3. **Make** your changes
4. **Add** tests
5. **Submit** a pull request

## 📄 License

**AGPL v3** - see [LICENSE](LICENSE) for details.

## 💬 Support

- **Issues** → [GitHub Issues](https://github.com/aircher/aircher/issues)
- **Documentation** → See `docs/` directory in this repository

## 📊 Project Status

🏗️ **Early Development** - Aircher is in active development with foundational architecture in place. Core REPL functionality is the current focus.

### **Implementation Status**
- ✅ **Multi-Database Architecture** → SQLite databases with migration system
- ✅ **TUI Framework** → Ratatui-based terminal interface foundation
- ✅ **Provider Abstractions** → Trait-based LLM provider system
- ✅ **Configuration System** → TOML-based hierarchical configuration
- ✅ **Project Analysis** → Automatic documentation generation
- 🚧 **REPL Interface** → Interactive session management and streaming
- 🚧 **LLM Integration** → OpenAI and Claude API implementations
- 🚧 **Advanced Features** → Real-time steering, @-mentions, session resumption

## 📚 Documentation

- [**MASTER_SPEC.md**](docs/core/MASTER_SPEC.md) → Technical specification and architecture
- [**DEVELOPER_GUIDE.md**](docs/core/DEVELOPER_GUIDE.md) → Development workflows and patterns
- [**tasks.json**](docs/tasks/tasks.json) → Current implementation tasks and progress
- [**AGENTS.md**](AGENTS.md) → Project memory for AI development

## 🎯 Why Aircher?

### **vs Commercial AI Assistants**
- ✅ **Multi-provider support** with intelligent routing
- ✅ **Pure Rust performance** and memory safety
- ✅ **Self-hosted** and air-gapped deployment options
- ✅ **Cost optimization** across multiple providers

### **vs Other Terminal Tools**
- ✅ **REPL-style interaction** with session resumption
- ✅ **Real-time message steering** and interruption
- ✅ **Advanced context management** with file relevance scoring
- ✅ **Extensible MCP tool ecosystem**
