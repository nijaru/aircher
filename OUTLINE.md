# Aircher Project Outline

## Project Vision

Aircher is a next-generation command-line AI coding assistant designed to work with any LLM provider while providing superior context management, autonomous web search, and intelligent automation. Unlike single-provider tools, Aircher offers true multi-provider support, cost optimization, and enterprise-ready features.

## Core Philosophy

- **API-Agnostic**: Works with OpenAI, Claude, Gemini, Ollama, and custom endpoints
- **Intelligence over Configuration**: Understands context and intent automatically  
- **Proactive Assistance**: Anticipates needs rather than waiting for explicit commands
- **Enterprise-Ready**: Self-hosted, air-gapped support, audit logging

## Key Features

### Multi-Provider LLM Support
- **Universal Interface**: Seamlessly switch between OpenAI, Claude, Gemini, Ollama
- **Intelligent Routing**: Automatically select optimal provider based on cost, features, availability
- **Provider-Specific Features**: Function calling, thinking mode, image support where available
- **Fallback Support**: Automatic failover when providers are unavailable
- **Cost Optimization**: Track usage and costs across all providers

### Intelligent Context Management
- **Task-Aware Context**: Automatically detects current task (debugging, feature development, refactoring)
- **Smart File Relevance**: Dynamic scoring based on task context, dependencies, and usage patterns
- **Quality-Based Compaction**: Preserves important context while managing token limits intelligently
- **Project Knowledge Persistence**: Long-term understanding of architecture and decisions
- **Context Retrieval**: Finds relevant historical conversations and solutions

### Autonomous Web Search
- **Temporal Awareness**: Automatically searches for current documentation and solutions
- **Smart Triggers**: Detects when queries need fresh information ("latest", "current", version mentions)
- **Error Recovery**: Proactive search for solutions to encountered errors
- **Multi-Provider Search**: Brave, DuckDuckGo, and custom search providers
- **Result Integration**: Seamlessly incorporates search results into responses

### Beautiful Terminal Interface (Charmbracelet TUI)
- **Modern TUI Framework**: Built with Charmbracelet's Bubble Tea for professional terminal UI
- **Real-time Streaming**: Live AI response rendering with smooth animations
- **Rich Markdown Rendering**: Beautiful code highlighting and formatting via Glamour
- **Interactive Panels**: Context sidebar, help system, and status indicators
- **Responsive Design**: Adapts seamlessly to any terminal size
- **Keyboard Shortcuts**: Efficient navigation (Ctrl+H help, Ctrl+T context, Ctrl+C exit)
- **Visual Feedback**: Live status indicators for thinking, searching, and streaming
- **Multiple Modes**: Interactive TUI, non-interactive CLI, and automation-friendly output
- **Session Management**: Visual session display with conversation history
- **Enhanced Commands**: Slash commands with visual feedback and help integration

### MCP Integration & Tools
- **Core Development Tools**: 
  - Filesystem operations with security controls
  - Git integration for version control
  - GitHub/GitLab/Gitea support for repository management
- **Web Tools**:
  - Web fetch for documentation retrieval
  - Search integration (Brave, Tavily) with cost transparency
- **Database Tools**: 
  - PostgreSQL, MySQL, SQLite for database operations
  - Redis for caching and key-value operations
- **Development Environment**:
  - Docker container management
  - Terminal/shell command execution
  - Build tool integration
- **Knowledge & Documentation**:
  - Memory systems for persistent context
  - RAG capabilities for code search
  - Markdown processing and generation
- **Extensibility**: 
  - Support for community MCP servers
  - Easy MCP server installation and management
  - Custom MCP server development support

### Project Memory System
- **AIRCHER.md Files**: Human-editable project memory for team-shared knowledge
- **Automatic Database**: File indexes, conversation history, and knowledge base handled automatically
- **Memory Types**: Instructions, conventions, commands, architecture notes, glossary
- **Instant Memory**: `#` prefix for quick memory additions from chat
- **Sync System**: Changes to AIRCHER.md automatically update internal databases

### Enterprise Features
- **Self-Hosted Deployment**: No dependency on external services for core functionality
- **Air-Gapped Support**: Works in offline environments with local models
- **Cost Tracking**: Monthly budgets, daily limits, provider-specific cost analysis
- **Health Diagnostics**: System health checks and troubleshooting
- **Audit Logging**: Comprehensive logging for compliance requirements
- **Git Integration**: Worktree support for parallel development sessions

## Architecture Overview

### Storage System
```
.aircher/
├── config.toml              # Configuration
├── conversations.db         # Chat history with file references  
├── knowledge.db            # Project understanding & decisions
├── file_index.db           # File relationships & metadata
├── sessions.db             # Session management
└── cache/                  # Search results, temporary data
```

### Core Components
- **Context Engine**: Task detection, file relevance, smart compaction
- **Provider Manager**: Multi-LLM support with intelligent routing
- **Search Engine**: Autonomous web search with temporal awareness  
- **Memory System**: Project knowledge with AIRCHER.md integration
- **Command Router**: Slash commands and custom command system
- **MCP Integration**: Model Context Protocol for extensible tools

## Key Differentiators

### vs Claude Code
- **Multi-Provider**: Works with any LLM, not just Claude
- **Beautiful TUI**: Modern terminal interface vs basic CLI
- **Real-time Experience**: Live streaming responses with visual feedback
- **Autonomous Search**: Proactive web search with temporal awareness
- **Superior Context**: Task-aware management vs token-limit based
- **Cost Optimization**: Multi-provider cost tracking and routing
- **Enterprise Features**: Self-hosted, air-gapped, audit logging

### vs Other AI Assistants
- **Professional TUI**: Terminal interface that rivals desktop applications
- **Live Streaming**: Real-time response rendering with smooth animations
- **Rich Formatting**: Beautiful markdown and code highlighting
- **Intelligent Automation**: Understands intent without explicit configuration
- **Project Persistence**: Long-term architectural understanding
- **Universal Compatibility**: Provider-agnostic with consistent interface
- **Advanced Context**: Quality-based compaction preserving key information
- **MCP Ecosystem**: Access to 200+ MCP servers for extended functionality
- **Tool Transparency**: Clear visibility and control over tool operations

## Usage Scenarios

### Beautiful Interactive Development
```bash
$ aircher
┌─ 🏹 Aircher ──────────────────────────── Provider: ollama • 🤖 Ready ─┐
│                                                                        │
│ ℹ️ System [14:32:15]                                                   │
│ Welcome to Aircher! 🏹 Detected Go project with 247 files.           │
│                                                                        │
│ 👤 You [14:32:20]                                                     │
│ explain the authentication system                                     │
│                                                                        │
│ 🤖 Aircher via ollama [14:32:21] 📡 Streaming...                     │
│ # Authentication System Overview                                      │
│                                                                        │
│ Based on your project structure, I can see you're using JWT-based     │
│ authentication with the following components...                       │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
┌─ > Type your message...                             Ctrl+H: Help ─────┐
└─ Ctrl+T: Context • Ctrl+C: Exit ─────────────────────────────────────────┘
```

### MCP Tool Usage
```bash
# Database operations
> show me the user table schema in our postgres database
[Uses PostgreSQL MCP to inspect database schema]

# Git operations
> create a new branch for the authentication fix
[Uses Git MCP to create branch and switch to it]

# Web documentation
> fetch the latest Next.js 14 routing documentation
[Uses web fetch MCP to retrieve and parse documentation]

# GitHub integration
> create a pull request for the current branch
[Uses GitHub MCP to create PR with generated description]
```

### Automation & Scripting
```bash
# Code review automation
git diff | aircher -p "review this code" --output-format json

# Batch processing
find . -name "*.go" | xargs -I {} aircher -p "add error handling to {}"

# Pipeline integration  
aircher -p "check test coverage" --max-turns 1
```

### Custom Commands
```bash
# .aircher/commands/optimize.md
# Optimize the $ARGUMENTS for performance and readability

$ aircher
> /project:optimize user authentication flow
[Executes custom optimization command with context]
```

## Command Interface

### Core Commands
```bash
# Interactive modes
aircher                              # Start interactive REPL
aircher "explain this project"       # REPL with initial prompt  
aircher -c                          # Continue last conversation
aircher -r "session-id"             # Resume specific session

# Non-interactive modes
aircher -p "query"                  # One-shot query, then exit
cat file.go | aircher -p "review"   # Process piped content
aircher -p "task" --output-format json  # Structured output

# Management commands
aircher config                      # Interactive configuration
aircher init                        # Initialize project with AIRCHER.md
aircher doctor                      # Health diagnostics
aircher update                      # Self-update with rollback
```

### Slash Commands
```bash
# Built-in commands
/clear                              # Clear conversation
/help                               # Show available commands
/config                            # Settings management
/cost                              # Usage and cost statistics
/memory                            # Edit AIRCHER.md memory
/search [query]                    # Force web search
/think                             # Enable thinking mode
/mcp                               # MCP server management
/tools                             # List available MCP tools

# MCP-specific commands
/mcp list                          # List installed MCP servers
/mcp install [server]              # Install an MCP server
/mcp enable [server]               # Enable an MCP server
/mcp disable [server]              # Disable an MCP server
/mcp status                        # Show MCP server status

# Custom commands
/project:optimize                  # Team-shared project command
/user:debug                        # Personal user command
```

## Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2) - COMPLETED ✅
**Goal**: Basic working system with core functionality
- ✅ Multi-provider LLM interface (OpenAI, Claude, Gemini, Ollama)
- ✅ Beautiful TUI REPL with Charmbracelet Bubble Tea
- ✅ Complete database system with SQLite
- ✅ TOML configuration system
- ✅ Enhanced slash commands with visual feedback
- ✅ MCP integration framework
- ✅ Professional terminal interface with real-time streaming

### Phase 2: Intelligence (Weeks 3-4) - FRAMEWORK COMPLETE 🚧
**Goal**: Smart context management and autonomous capabilities
- ✅ Task-aware context management framework (algorithms stubbed)
- ✅ Autonomous web search framework (API integration pending)
- ✅ Smart conversation compaction framework (algorithms stubbed)
- ✅ AIRCHER.md memory system framework (parsing pending)
- ✅ Custom slash commands with visual templating
- ✅ Enhanced TUI features with live status indicators

### Phase 3: Advanced Features (Weeks 5-6) - PARTIALLY COMPLETE 🚧
**Goal**: Rich interface and extended provider support
- ✅ Multi-scope MCP integration with visual management
- ✅ Enhanced TUI with interactive panels and shortcuts
- ✅ Output format system (JSON, markdown, streaming)
- ✅ Advanced terminal integration (vim mode pending)
- ✅ Complete provider support (OpenAI, Claude, Gemini, Ollama)
- 🚧 Multimodal input implementation
- 🚧 Live tool execution panels

### Phase 4: Enterprise (Weeks 7-8)
**Goal**: Production-ready features for enterprise use
- Git worktree integration
- Health diagnostics and monitoring
- Cost tracking and budget management
- Auto-update with rollback
- Advanced error recovery

### Phase 5: Production (Weeks 9-10)
**Goal**: Polish and distribution
- Performance optimization
- Comprehensive testing and documentation
- Package distribution (Homebrew, Scoop, Docker)
- Enterprise deployment guides
- Community features and plugin system

## AIRCHER.md Memory System

### Purpose
AIRCHER.md files serve as human-editable project memory for information that should be included in every conversation but isn't automatically handled by the database systems.

### What Goes in AIRCHER.md
- **Programming language and framework versions**
- **Coding style guides and conventions**
- **Team-specific instructions and preferences**
- **Architecture decisions and patterns**
- **Frequently used commands**
- **Project-specific terminology**

### What's Handled Automatically
- **File Index**: Relationships, dependencies, and metadata (file_index.db)
- **Conversation History**: Previous conversations and context (conversations.db)
- **Knowledge Base**: Learned patterns and solutions (knowledge.db)

### Example AIRCHER.md Structure
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

## Target Users

### Individual Developers
- **Solo Projects**: Enhanced productivity with intelligent assistance
- **Learning**: Guidance on best practices and current solutions
- **Debugging**: Automated error research and solution suggestions

### Development Teams
- **Shared Knowledge**: Team conventions in AIRCHER.md files
- **Onboarding**: New team members get instant project context
- **Consistency**: Uniform coding practices across team members

### Enterprise Organizations
- **Self-Hosted**: Deploy in secure, air-gapped environments
- **Cost Control**: Multi-provider cost optimization and budgets
- **Compliance**: Audit logging and security controls
- **Integration**: Custom tools via MCP protocol

## Success Metrics

### User Experience
- Faster task completion compared to existing tools
- Reduced context switching between documentation and code
- Higher user satisfaction with AI assistance quality

### Technical Performance  
- Efficient token usage through smart context management
- Fast response times with appropriate provider routing
- Reliable autonomous search with high relevance scores

### Adoption
- Growing user base across individual developers and teams
- Enterprise adoption for secure, self-hosted deployments
- Community contributions and custom command sharing

This outline provides the comprehensive vision for Aircher while maintaining focus on our core differentiators: multi-provider support, intelligent context management, and autonomous web search capabilities.
