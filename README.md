# Aircher

**AI-powered terminal assistant built with Rust** - Chat with Claude, Gemini, and OpenRouter from your command line.

## ✅ What Works Now

**CLI-001 & CLI-002 & TUI-001 Complete!** You can now chat with AI:

```bash
# One-shot conversations
aircher "How do I write a Rust function?"
aircher "Explain async/await in simple terms"

# Interactive chat mode
aircher
> hello world
🤖 Hello! How can I help you today?
> /help
📖 Available Commands:
  /help     - Show this help message
  /quit     - Exit interactive mode
  /clear    - Clear conversation history
  /history  - Show conversation history
  /model    - Show current model information
> /quit

# TUI mode (rich terminal interface)
aircher --tui

# Choose your provider
aircher --provider gemini "What's the weather like?"
aircher --provider openrouter "Help me debug this code"

# Select specific models
aircher --model claude-3-5-sonnet-20241022 "Write a poem"

# Get help
aircher --help
```

**Working Features:**
- ✅ **One-shot chat** - Send a message, get a response
- ✅ **Interactive chat** - Back-and-forth conversations with history
- ✅ **TUI interface** - Rich terminal UI with real-time chat
- ✅ **Multi-provider support** - Claude, Gemini, OpenRouter
- ✅ **Clean error handling** - Helpful messages for missing API keys
- ✅ **Professional CLI** - Help, version, parameter support

## 🚧 Coming Next

- **TUI model selection** - Choose providers and models from the TUI
- **Session management** - Save and resume conversations
- **Intelligent context** - File analysis and smart context assembly
- **Integration testing** - Comprehensive provider testing

## 🚀 Quick Setup

### 1. Build from Source
```bash
git clone https://github.com/nijaru/aircher.git
cd aircher
cargo build --release
```

### 2. Set API Keys
```bash
# For Claude (required for default provider)
export ANTHROPIC_API_KEY=your_key_here

# For Gemini (optional)
export GOOGLE_API_KEY=your_key_here

# For OpenRouter (optional)
export OPENROUTER_API_KEY=your_key_here
```

### 3. Start Chatting!
```bash
./target/release/aircher "Hello, how are you?"
```

## 💡 Current Examples

```bash
# Basic chat
aircher "Explain Rust ownership"

# Interactive mode
aircher
> hello world
🤖 Hello! How can I help you today?
> explain rust ownership
🤖 Rust ownership is a memory management system...
> /quit

# TUI mode
aircher --tui
# Opens rich terminal interface with real-time chat

# Different providers
aircher --provider gemini "Write a Python function"
aircher --provider openrouter "Help me debug this error"

# Specific models
aircher --model claude-3-5-sonnet-20241022 "Write documentation"

# Get help
aircher --help
```

## 🏗️ Architecture

**Pure Rust single binary** with:
- **Provider abstraction** - Unified interface for Claude, Gemini, OpenRouter
- **Async architecture** - Tokio runtime with streaming support  
- **Clean error handling** - User-friendly messages, no panic traces
- **Lazy loading** - Providers initialized only when needed

## 📊 Project Status

- **Phase 0: User Interface** - 75% complete (CLI-001 ✅, CLI-002 ✅, TUI-001 ✅)
- **Phase 1: Foundation** - 100% complete  
- **Phase 2: Providers** - 70% complete (Claude, Gemini, OpenRouter)
- **Phase 3: Intelligence** - 0% complete
- **Phase 4: Advanced Features** - 0% complete

**Next**: TUI-002 (Model selection & settings)

## 🤝 Contributing

This project is in active development. Check `docs/tasks/tasks.json` for current priorities.

## 📄 License

MIT License - see LICENSE file for details.
