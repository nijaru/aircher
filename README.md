# Aircher

Next-generation AI coding assistant with multi-provider support, intelligent context management, and autonomous web search.

## Overview

Aircher is a command-line AI coding assistant designed to work with any LLM provider while providing superior context management, autonomous web search, and intelligent automation. Unlike single-provider tools, Aircher offers true multi-provider support, cost optimization, and enterprise-ready features.

## Key Features

### 🎨 Modern Terminal Interface
- **Beautiful TUI**: Powered by Charmbracelet's Bubble Tea framework
- **Real-time Streaming**: Watch AI responses appear live with smooth animations
- **Rich Formatting**: Markdown rendering with syntax highlighting
- **Interactive Panels**: Context sidebar, help system, and status indicators
- **Responsive Design**: Adapts seamlessly to any terminal size

### 🔄 Multi-Provider LLM Support
### 🚀 Go 1.24 Enhanced Performance
- **Swiss Tables Maps**: 2-3% CPU performance improvement with Go 1.24's new map implementation
- **Secure Filesystem**: Uses Go 1.24's os.Root for secure, sandboxed MCP operations
- **Enhanced Cryptography**: Ready for post-quantum cryptography with ML-KEM support
- **Tool Management**: Modern development workflow with Go 1.24 tool directives
- **Runtime Optimizations**: Better memory management and garbage collection performance

### 🔄 Multi-Provider LLM Support
- **Universal Interface**: Seamlessly switch between OpenAI, Claude, Gemini, Ollama
- **Intelligent Routing**: Automatically select optimal provider based on cost, features, availability
- **Provider-Specific Features**: Function calling, thinking mode, image support where available
- **Fallback Support**: Automatic failover when providers are unavailable
- **Cost Optimization**: Track usage and costs across all providers

### 🧠 Intelligent Context Management
- **Task-Aware Context**: Automatically detects current task (debugging, feature development, refactoring)
- **Smart File Relevance**: Dynamic scoring based on task context, dependencies, and usage patterns
- **Quality-Based Compaction**: Preserves important context while managing token limits intelligently
- **Project Knowledge Persistence**: Long-term understanding of architecture and decisions

### 🔍 Autonomous Web Search
- **Temporal Awareness**: Automatically searches for current documentation and solutions
- **Smart Triggers**: Detects when queries need fresh information ("latest", "current", version mentions)
- **Error Recovery**: Proactive search for solutions to encountered errors
- **Multi-Provider Search**: Brave, DuckDuckGo, and custom search providers

### 🎨 Beautiful Terminal Interface
- **Modern TUI**: Built with Charmbracelet's Bubble Tea framework
- **Real-time Streaming**: Live updates as AI responds
- **Rich Markdown Rendering**: Code highlighting and formatted responses
- **Interactive Panels**: Context sidebar, help system, status indicators
- **Keyboard Shortcuts**: Efficient navigation with vim-like controls

### 🛠️ MCP Integration & Tools
- **Core Development Tools**: Filesystem, Git, GitHub/GitLab integration
- **Database Tools**: PostgreSQL, MySQL, SQLite, Redis support
- **Web Tools**: Documentation retrieval, search integration
- **Development Environment**: Docker, terminal, build tool integration
- **Extensibility**: Support for 200+ community MCP servers

### 📝 Project Memory System
- **AIRCHER.md Files**: Human-editable project memory for team-shared knowledge
- **Automatic Database**: File indexes, conversation history, and knowledge base
- **Instant Memory**: `#` prefix for quick memory additions from chat
- **Sync System**: Changes to AIRCHER.md automatically update internal databases

## Installation

### From Source

```bash
git clone https://github.com/aircher/aircher.git
cd aircher
go build -o aircher ./cmd/aircher
sudo mv aircher /usr/local/bin/
```

### Using Go Install

```bash
go install github.com/aircher/aircher/cmd/aircher@latest
```

## Quick Start

### 1. Initialize a Project

```bash
cd your-project
aircher init
```

### 2. Configure Providers

Set up API keys for your preferred providers:

```bash
export OPENAI_API_KEY="your-openai-key"
export ANTHROPIC_API_KEY="your-claude-key"
export GOOGLE_API_KEY="your-gemini-key"
export BRAVE_API_KEY="your-brave-search-key"  # Optional
```

### 3. Start Interactive Mode

```bash
aircher
```

### 4. Or Use Non-Interactive Mode

```bash
aircher -p "explain this codebase"
cat main.go | aircher -p "review this code"
```

## Usage Examples

### Beautiful Interactive Interface
```bash
$ aircher
┌─ 🏹 Aircher ──────────────────────────── Provider: ollama • 🤖 Ready ─┐
│                                                                        │
│ ℹ️ System [14:32:15]                                                   │
│ Welcome to Aircher! 🏹                                                │
│                                                                        │
│ ℹ️ System [14:32:15]                                                   │
│ Type your question or use /help for commands.                         │
│                                                                        │
│ 👤 You [14:32:20]                                                     │
│ explain the authentication system                                     │
│                                                                        │
│ 🤖 Aircher via ollama [14:32:21]                                      │
│ # Authentication System Overview                                      │
│                                                                        │
│ Based on your project structure, I can see you're using JWT-based     │
│ authentication with the following components:                         │
│                                                                        │
│ ```go                                                                  │
│ // JWT middleware implementation                                       │
│ func AuthMiddleware(next http.Handler) http.Handler {                 │
│     return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
│         // Token validation logic...                                   │
│     })                                                                 │
│ }                                                                      │
│ ```                                                                    │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
┌─ > Type your message...                                              ─┐
└─ Ctrl+H: Help • Ctrl+T: Context • Ctrl+C: Exit ─────────────────────────┘
```

### Real-time Interaction
```bash
> /help                          # Toggle help panel
> /clear                         # Clear conversation  
> /think                         # Enable thinking mode
> Ctrl+T                         # Toggle context panel
> Ctrl+H                         # Toggle help
```

### MCP Tool Usage
```bash
# Database operations
> show me the user table schema in our postgres database

# Git operations  
> create a new branch for the authentication fix

# Web documentation
> fetch the latest Next.js 14 routing documentation

# GitHub integration
> create a pull request for the current branch
```

### Interactive Commands & Shortcuts

#### Slash Commands
```bash
/help                    # Toggle help panel
/clear                   # Clear conversation
/config                  # Settings management  
/cost                    # Show usage statistics
/memory                  # Edit AIRCHER.md memory
/search [query]          # Force web search
/think                   # Toggle thinking mode
/mcp                     # MCP server management
/tools                   # List available MCP tools
```

#### Keyboard Shortcuts
```bash
Ctrl+H                   # Toggle help panel
Ctrl+T                   # Toggle context sidebar
Ctrl+C / Esc            # Exit Aircher
Enter                    # Send message
```

#### TUI Features
- **Live Streaming**: See responses appear in real-time
- **Markdown Rendering**: Beautiful code highlighting and formatting
- **Context Panel**: View session info, costs, and available tools
- **Status Indicators**: Visual feedback for thinking/searching states
- **Responsive Layout**: Adapts to terminal size automatically

### Automation & Scripting
```bash
# Code review automation
git diff | aircher -p "review this code" --output-format json

# Batch processing
find . -name "*.go" | xargs -I {} aircher -p "add error handling to {}"

# Pipeline integration
aircher -p "check test coverage" --max-turns 1
```

## Configuration

Aircher uses TOML configuration files. Initialize with default settings:

```bash
aircher config
```

### Configuration Locations
- **Project-specific**: `.aircher/config.toml` 
- **User-global**: `~/.config/aircher/config.toml`

### Key Configuration Options

```toml
[providers]
default = "openai"

[providers.openai]
api_key_env = "OPENAI_API_KEY"
model = "gpt-4"
max_tokens = 4096

[providers.claude]
api_key_env = "ANTHROPIC_API_KEY"
model = "claude-3-sonnet-20240229"

[context_management]
auto_compaction.enabled = true
auto_compaction.token_threshold = 8000

[search]
enabled = true
auto_search = true
providers = ["brave"]

[costs]
monthly_budget = 100.0
daily_limit = 10.0
```

## Project Memory (AIRCHER.md)

Create an `AIRCHER.md` file in your project root for team-shared knowledge:

```markdown
# Project Name - Aircher Memory

## Instructions
- This project uses Go 1.21 with Echo framework
- Follow Google Go style guide
- Use testify for testing

## Conventions
- Use meaningful variable names
- Add comments for complex logic
- Error handling is mandatory

## Commands
- `make build` - Build the project
- `make test` - Run all tests
- `make lint` - Run linter

## Architecture
- Clean architecture with domain/service/repository layers
- Use dependency injection
- Database migrations in /migrations

## Glossary
- **Handler**: HTTP request handler functions
- **Service**: Business logic layer
- **Repository**: Data access layer
```

## Development

### Requirements
- Go 1.24+ (leverages Swiss Tables, os.Root security, and tool management)
- SQLite (included)
- Node.js (for MCP servers)
- Modern terminal with color support
- Recommended: Terminal with Unicode support for best experience

### Building from Source

```bash
git clone https://github.com/aircher/aircher.git
cd aircher
go mod download
make build  # Uses Go 1.24 optimizations and tool management
```

### Go 1.24 Features

Aircher leverages the latest Go 1.24 features for enhanced performance and security:

- **Swiss Tables**: Automatic map performance improvements (2-3% faster)
- **Secure Filesystem**: `os.Root` for sandboxed MCP server operations  
- **Tool Management**: Development tools managed in go.mod
- **Enhanced Crypto**: Post-quantum cryptography support
- **Runtime Improvements**: Better memory allocation and garbage collection

### Running Tests

```bash
go test ./...
```

### Health Check

```bash
aircher doctor
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/aircher/aircher/issues)
- **Discussions**: [GitHub Discussions](https://github.com/aircher/aircher/discussions)
- **Documentation**: [docs.aircher.ai](https://docs.aircher.ai)

**API-Agnostic AI Coding Assistant**

Aircher is a planned next-generation command-line AI coding assistant designed to work with any LLM provider while providing superior context management, autonomous web search, and intelligent automation.

## Project Status

🚧 **In Planning Phase** - This project is currently in active planning and design. No code has been implemented yet.

## Key Features (Planned)

- **Multi-Provider Support**: Works with OpenAI, Claude, Gemini, Ollama, and custom endpoints
- **Intelligent Context Management**: Task-aware context with quality-based conversation compaction
- **Autonomous Web Search**: Automatically searches for current documentation and solutions
- **Interactive REPL**: Primary conversational interface with rich terminal features
- **Project Memory**: AIRCHER.md files for team-shared knowledge and conventions
- **Custom Commands**: Project and user-scoped slash commands with templating

## Documentation

- [**OUTLINE.md**](OUTLINE.md) - Comprehensive project overview, features, and roadmap
- [**SPEC.md**](SPEC.md) - Detailed technical specification and architecture
- [**TASKS.md**](TASKS.md) - Implementation task list and progress tracking

## Installation

*Installation instructions will be available once the project is implemented.*

## Quick Start

*Usage instructions will be available once the project is implemented.*

## Key Differentiators

**vs Claude Code:**
- Multi-provider support (not just Claude)
- Autonomous web search with temporal awareness
- Task-aware context management vs token-limit based
- Cost optimization across providers

**vs Other AI Assistants:**
- Provider-agnostic with consistent interface
- Intelligent automation without extensive configuration
- Enterprise-ready features (self-hosted, air-gapped)

## Contributing

This project is in the planning phase. Contributions to the design and specification are welcome through GitHub issues and discussions.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

**Note**: This project is currently in planning and design phase. See [OUTLINE.md](OUTLINE.md) for comprehensive project details and [TASKS.md](TASKS.md) for implementation roadmap.
