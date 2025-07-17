# Aircher

**AI-powered terminal assistant built with Rust** - Chat with Claude, Gemini, OpenAI, and OpenRouter from your command line.

## ✅ What Works Now

**Full-featured AI development assistant with semantic code search:**

```bash
# Launch intelligent TUI (primary mode)
aircher
# Auto-detects project, creates .aircher/ directory, starts session

# Semantic code search (NEW!)
aircher search query "error handling patterns"    # Finds conceptually similar code
aircher search query "database connection logic"  # Works across languages
aircher search index                              # Index current directory
aircher model current                             # Show configured models

# One-shot conversations with project context
aircher "How do I refactor this function?"
aircher "Explain the architecture of this project"

# Interactive chat with session persistence
aircher
> hello world  
🤖 Hello! I can see you're working on the Aircher project...
> what files are most important?
🤖 Based on your project structure, here are the key files:
   - src/main.rs: Application entry point
   - src/ui/mod.rs: TUI implementation with session management
   - src/intelligence/: Intelligence engine for context-aware assistance
> /quit

# Different providers with same intelligent context
aircher --provider gemini "What's the current development focus?"
aircher --provider openai "Help me implement this feature"
aircher --provider openrouter "Find the best model for code review"
aircher --provider ollama "Local model for privacy-focused development"

# Configuration management 
aircher config show                     # Show current configuration
aircher config set ui.theme dark        # Update settings
aircher config set providers.claude.api_key sk-xxx
aircher config get ui.theme             # Get specific values
aircher config edit                     # Open config in $EDITOR

# Session management
aircher session list                    # List all sessions
aircher session new "Feature work"      # Create new session
aircher session export session_id --format markdown
```

**Working Features:**
- ✅ **Semantic Code Search** - Revolutionary AI-powered code understanding:
  - Find code by concept, not just text matching ("error handling patterns")
  - Cross-language semantic similarity detection
  - Intelligent code chunking and embedding generation
  - Works with Ollama (nomic-embed-text) for local, private search
- ✅ **Embedding Management** - Full embedding model lifecycle:
  - Auto-detection and setup of embedding models
  - Ollama integration for local models
  - Fallback to text search when embeddings unavailable
  - Smart model selection based on system capabilities
- ✅ **Project-aware TUI** - Automatically detects and initializes `.aircher/` projects
- ✅ **Intelligent TUI Integration** - TUI with full intelligence engine integration:
  - Project detection and automatic `.aircher/` directory management
  - Background file monitoring and analysis
  - Context injection system for rich system prompts
  - File tree walking and project scanning capabilities
- ✅ **Session persistence** - SQLite storage with conversation history and analytics
- ✅ **Intelligence Engine** - Context-aware development assistant with:
  - File purpose analysis and relevance scoring
  - Architectural decision tracking
  - Background file monitoring with change detection
  - Rich context injection into every conversation
  - TUI tools interface for seamless integration
- ✅ **Multi-provider support** - Claude, Gemini, OpenAI, OpenRouter, Ollama with cost optimization
- ✅ **Smart context injection** - Every conversation includes project context, file purposes, and recent changes
- ✅ **Background file monitoring** - Automatically detects and analyzes file changes
- ✅ **Export capabilities** - Sessions can be exported in JSON, Markdown, CSV, or plain text
- ✅ **One session per project** - Simple, predictable session management
- ✅ **Comprehensive testing framework** - Full TUI testing with dependency injection:
  - Mock implementations for all providers and intelligence tools
  - Integration tests for complete TUI workflows
  - Session persistence and error handling validation
  - Performance and multi-provider testing scenarios

## 🚧 Coming Next

- 🔥 **SweRankEmbed-Small Integration** - Embed best open source code model (137M params)
- **Advanced embedding features** - Model auto-selection, download-on-demand with resume
- **CodeXEmbed Integration** - SOTA code performance via ONNX Runtime (for power users)
- **Enhanced semantic search** - Cross-file relationship detection, architecture analysis
- **MCP server** - Universal Model Context Protocol server implementation

## 🚀 Quick Setup

### 1. Build from Source
```bash
git clone https://github.com/nijaru/aircher.git
cd aircher
cargo build --release
```

### 2. Configure API Keys
```bash
# Option 1: Environment variables (quick start)
export ANTHROPIC_API_KEY=your_key_here
export OPENAI_API_KEY=your_key_here        # Optional
export GOOGLE_API_KEY=your_key_here        # Optional

# Option 2: Configuration file (recommended)
aircher config set providers.claude.api_key sk-xxx
aircher config set providers.openai.api_key sk-xxx
aircher config set providers.gemini.api_key your_key

# For Ollama (local models) - no API key needed
# Just ensure Ollama is running: ollama serve
```

### 3. Start Using Aircher!
```bash
# Launch TUI (primary interface)
./target/release/aircher

# Or quick one-shot chat
./target/release/aircher "Hello, how are you?"

# Check configuration
./target/release/aircher config show
```

## 💡 Usage Examples

```bash
# Primary interface - Rich TUI
aircher                          # Interactive terminal UI

# Quick one-shot queries  
aircher "Explain Rust ownership"
aircher "Find error handling patterns in this codebase"

# Provider and model selection
aircher --provider gemini "Write a Python function"
aircher --provider ollama "Local model for privacy"
aircher --model gpt-4 "Help me debug this error"

# Semantic code search
aircher search index             # Index your codebase
aircher search query "database connection logic"

# Configuration management
aircher config show             # View current settings
aircher config set ui.theme light
aircher config set providers.claude.api_key sk-xxx

# Session management  
aircher session list
aircher session export 123 --format markdown

# Get help
aircher --help
aircher search --help
aircher config --help
```

## 🏗️ Architecture

**Pure Rust single binary** with:
- **TUI-first design** - Rich terminal interface as primary mode
- **Provider abstraction** - Unified interface for Claude, Gemini, OpenAI, OpenRouter, Ollama
- **Semantic code search** - AI-powered understanding beyond text matching
- **TOML configuration** - Cross-platform config files in standard locations
- **Project-aware intelligence** - Local `.aircher/` directory with:
  - `AGENT.md` - AI assistant configuration and project context
  - `sessions/` - SQLite database for conversation persistence
  - `intelligence/` - Cached project analysis and insights
  - Background file monitoring for automatic context updates

## 📊 Project Status

- **Phase 0: User Interface** - 100% complete (CLI-001 ✅, CLI-002 ✅, TUI-001 ✅, TUI-002 ✅)
- **Phase 1: Foundation** - 100% complete  
- **Phase 2: Providers** - 100% complete (Claude, Gemini, OpenAI, OpenRouter, Ollama)
- **Phase 3: Intelligence** - 100% complete (SPRINT-004 ✅, SPRINT-005 ✅, SPRINT-006 ✅)
- **Phase 4: Advanced Features** - 100% complete (Session management ✅, File monitoring ✅, TUI Integration ✅, Testing Framework ✅)
- **Phase 5: Semantic Search** - 95% complete (Ollama integration ✅, Semantic search ✅, Embedding management ✅, CLI integration ✅)

**Next**: SweRankEmbed-Small integration for embedded model support

## 🔥 Latest: Semantic Code Search

Revolutionary AI-powered code understanding! Search by **meaning**, not just text:

```bash
# Find conceptually similar code across your entire project
aircher search query "error handling patterns"
aircher search query "database connection logic"
aircher search query "authentication code"

# Quick setup
aircher model current          # Check configured models
aircher search index          # Index your codebase
# Now you have semantic superpowers!

# Works with Ollama for 100% local, private semantic search
# No API calls, no data leaving your machine
```

**Game-changer**: Goes beyond grep to understand code **meaning** and **context**.

## 🤝 Contributing

This project is in active development. Check `docs/tasks/tasks.json` for current priorities.

## 📄 License

MIT License - see LICENSE file for details.
