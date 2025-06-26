# Aircher

**AI-powered terminal assistant built with Rust** - Intelligent command-line interface with multi-LLM support and universal context management.

## What is Aircher?

Aircher is a **dual-architecture AI development system** built in pure Rust:

1. **Aircher Terminal** - Full-featured AI assistant with advanced terminal UI
2. **Aircher Intelligence Engine** - Universal MCP server providing intelligent context management to any AI tool

Think of it as your intelligent coding companion that works everywhere - from your terminal to Claude Desktop to VS Code extensions.

## ✨ Core Features

### 🏹 **Aircher Terminal**
- **REPL-Style Interface** → Interactive terminal sessions with natural language commands
- **Multi-Provider Support** → OpenAI, Claude, Gemini, GitHub Copilot, Ollama with intelligent routing
- **Real-Time Streaming** → See responses appear as they're generated
- **Session Management** → Resume conversations with `aircher --resume`
- **Advanced Interaction** → @-mention files, slash commands, thinking mode display

### 🧠 **Aircher Intelligence Engine (MCP Server)**
- **Universal Compatibility** → Works with Claude Desktop, VS Code, and any MCP-compatible tool
- **Intelligent Context** → AI-driven file relevance scoring based on current task
- **Task Detection** → Automatic identification of development work (debugging, features, refactoring)
- **Cross-Project Learning** → Pattern recognition and success correlation across projects
- **Smart Assembly** → Optimize context for AI tools based on token limits
- **Dependency Analysis** → Build and query file relationship networks

### 🔒 **Security & Performance**
- **Pure Rust** → Memory safety and native performance
- **Multi-Database Architecture** → Specialized SQLite databases for different data types
- **Hierarchical Storage** → Global → Project → Worktree → Session organization
- **Elastic License 2.0** → Enterprise-friendly with protection against exploitation

## 🚀 Installation

### From Source (Current)
```bash
git clone https://github.com/aircher/aircher.git
cd aircher
cargo build --release
sudo mv target/release/aircher /usr/local/bin/
```

*Note: Pre-built binaries and package managers will be available at initial release.*

## ⚡ Quick Start

### Terminal Assistant
```bash
# Start interactive session (in development)
aircher

# With specific provider/model (planned)
aircher --service openai --model gpt-4

# Resume previous conversation (planned)
aircher --resume
```

### MCP Intelligence Server
```bash
# Start MCP server (planned)
aircher-mcp --port 3000

# Use with Claude Desktop, VS Code, etc.
# Server provides intelligent context tools to any MCP-compatible AI tool
```

## 💡 Usage Examples

### Terminal Interface
```bash
# Reference files directly
> @README.md explain this project

# Use slash commands
> /thinking on               # Show AI reasoning
> /web-search rust async     # Manual web search
> /switch-model claude-4     # Change model

# Real-time steering
> explain ownership [while AI responds: "focus on borrowing"]
```

### MCP Tools (Any Compatible AI Tool)
- `project_analyze` → Automatic project structure analysis
- `context_score_files` → AI-driven file relevance for current task
- `task_detect` → Identify current development task type
- `dependency_graph` → Build and query file relationships
- `success_patterns` → Learn from historical patterns
- `smart_context_assembly` → Optimize context for token limits

## ⚙️ Configuration

```toml
# ~/.config/aircher/config.toml
[providers]
default = "anthropic"
fallback_enabled = true

[models]
anthropic_default = "claude-4-sonnet"
openai_default = "gpt-4o"
auto_select = true  # Task-based model selection

[interface]
show_thinking = true
show_context_usage = true
streaming = true

[context]
max_files = 20
auto_compaction = true

[costs]
monthly_budget = 100.0
track_usage = true
```

## 🛠️ Development

### Requirements
- Rust 1.80+
- SQLite (included)
- Modern terminal with color support

### Building
```bash
cargo build --release
cargo test
cargo clippy
```

### Project Structure
```
aircher/
├── src/                    # Rust source code
├── docs/                   # Architecture and specifications
│   ├── core/              # Technical specifications
│   ├── tasks/             # Task management
│   └── architecture/      # Component designs
├── tests/                 # Test suites
└── examples/              # Usage demonstrations
```

## 📊 Project Status

**Early Development** - Core architecture established, implementing LLM integration.

### ✅ Completed
- Multi-database architecture with SQLite
- Terminal UI framework with Ratatui
- Project analysis and documentation generation
- Configuration system with smart defaults
- Development infrastructure and tooling

### 🚧 In Progress
- LLM provider implementations (OpenAI, Claude)
- MCP server architecture and protocol
- Interactive terminal interface
- Intelligent context management algorithms

### 📋 Planned
- Cross-project learning and pattern recognition
- Advanced security framework
- Web search integration
- Team collaboration features

## 🎯 Why Aircher?

### Universal Intelligence
- **Works Everywhere** → Terminal, Claude Desktop, VS Code, any MCP tool
- **Cross-Project Learning** → Insights and patterns across your entire codebase
- **Task-Aware Context** → Understands what you're working on and provides relevant context

### Developer-First Design
- **Pure Rust Performance** → Native speed with memory safety
- **Multi-Provider Support** → Best model for each task with automatic fallback
- **Self-Hosted Options** → Full control over your data and AI interactions

### Enterprise Ready
- **Elastic License 2.0** → Open source with commercial use protection
- **Security by Design** → Comprehensive permissions and audit logging
- **Cost Optimization** → Intelligent model selection and budget management

## 📄 License

**Elastic License 2.0** - see [LICENSE](LICENSE) for details.

Allows broad commercial use while protecting against unauthorized SaaS offerings.

## 💬 Support

- **Issues** → [GitHub Issues](https://github.com/aircher/aircher/issues)
- **Documentation** → `docs/` directory in this repository

---

**Vision**: Aircher aims to be the intelligent context layer for AI-powered development, providing universal compatibility and cross-project learning to accelerate developer workflows everywhere.
