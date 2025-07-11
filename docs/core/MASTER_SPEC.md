# Aircher Master Technical Specification

## Executive Summary

**Current Reality**: We built sophisticated provider architecture but no working user interface. Users cannot interact with our system.

**New Direction**: CLI-first development approach - get users a working interface before adding advanced features.

## Architecture Overview

### Pure Rust Single Binary
```
┌─ Aircher Terminal (Pure Rust) ─┐
│  ├─ CLI Interface (Phase 0)    │  ← NEW PRIORITY
│  │  ├─ One-shot mode           │
│  │  └─ Interactive mode        │
│  ├─ TUI Interface (Phase 0)    │  ← NEW PRIORITY
│  │  ├─ Chat view               │
│  │  ├─ Model selection         │
│  │  └─ Settings panel          │
│  ├─ Provider Architecture      │  ← ALREADY BUILT
│  │  ├─ Claude API              │  ✅ Complete
│  │  ├─ Gemini API              │  ✅ Complete
│  │  ├─ OpenRouter              │  ✅ Complete
│  │  └─ Host abstraction        │  ✅ Complete
│  ├─ Intelligence Engine        │  ← FUTURE
│  │  ├─ Context analysis        │
│  │  ├─ File scoring            │
│  │  └─ Cost optimization       │
│  └─ Session Management         │  ← FUTURE
│     ├─ Persistence             │
│     └─ Analytics               │
└─────────────────────────────────┘
```

## Development Phases

### Phase 0: Working User Interface (CURRENT PRIORITY)
**Goal**: Get users a working interface to interact with our provider architecture

| Task | Status | Description |
|------|--------|-------------|
| CLI-001 | Pending | Basic CLI: `aircher 'hello world'` |
| CLI-002 | Pending | Interactive CLI: `aircher --interactive` |
| TUI-001 | Pending | Basic TUI chat interface |
| TUI-002 | Pending | TUI model selection & settings |

### Phase 1: Integration & Polish
**Goal**: Fix and test our existing provider architecture

| Task | Status | Description |
|------|--------|-------------|
| INTEGRATION-001 | Pending | Provider integration testing |
| SPRINT-004 | Pending | Intelligence engine core |
| SPRINT-005 | Pending | Session management |

### Phase 2: Advanced Features
**Goal**: Add the sophisticated features we originally designed

| Task | Status | Description |
|------|--------|-------------|
| SPRINT-006 | Pending | OpenAI provider |
| SPRINT-007 | Pending | Ollama local provider |
| SPRINT-008 | Pending | Cost optimization |

## Provider Architecture (Already Built)

### Hierarchy: Provider → Model → Host
```
Provider: Who created the model (OpenAI, Anthropic, Google)
Model: Specific version (gpt-4o, claude-3.5-sonnet, gemini-2.0-flash)
Host: Where accessed (Direct API, OpenRouter, Local)
```

### Implemented Providers
- ✅ **Claude API**: Complete with streaming support
- ✅ **Gemini API**: Complete with streaming support  
- ✅ **OpenRouter**: Complete with dynamic model discovery
- ⏳ **OpenAI**: Planned
- ⏳ **Ollama**: Planned

## User Interface Design

### CLI Interface (Priority 1)
```bash
# One-shot mode
aircher 'hello world'
aircher 'explain this code' < main.rs

# Interactive mode
aircher --interactive
> hello world
AI: Hello! How can I help you today?
> /help
AI: Available commands: /help, /quit, /clear, /model
> /model claude-3.5-sonnet
AI: Switched to Claude 3.5 Sonnet
> /quit
```

### TUI Interface (Priority 2)
```
┌─ Aircher v0.1.0 ─ Claude 3.5 Sonnet ─ $0.17 session ─┐
│                                                        │
│ Chat History:                                          │
│ ┌────────────────────────────────────────────────────┐ │
│ │ > hello world                                      │ │
│ │                                                    │ │
│ │ AI: Hello! How can I help you today?              │ │
│ │                                                    │ │
│ │ > explain this rust code                           │ │
│ │                                                    │ │
│ │ AI: This code defines a struct...                 │ │
│ └────────────────────────────────────────────────────┘ │
│                                                        │
│ Input: ┌─────────────────────────────────────────────┐ │
│        │ Type your message here...                   │ │
│        └─────────────────────────────────────────────┘ │
│                                                        │
│ Tab: Models | Ctrl+S: Settings | Ctrl+C: Quit         │
└────────────────────────────────────────────────────────┘
```

### Model Selection Modal
```
┌──────────────── Select AI Configuration ─────────────────┐
│ Provider: [Claude] [Gemini] [OpenRouter]                │
│          ──────                                          │
│                                                          │
│ ● claude-3.5-sonnet         $3.00/$15.00 per 1M tokens  │
│ ○ claude-3-haiku            $0.25/$1.25 per 1M tokens   │
│                                                          │
│ Host: ● Direct API  ○ OpenRouter (25% cheaper)          │
│                                                          │
│ Session: 15.2K tokens ($0.17 total)                     │
└──────────────────────────────────────────────────────────┘
```

## Technology Stack

### Core Technologies
- **Rust 1.80+**: Entire application
- **Ratatui**: Terminal UI framework
- **SQLite + sqlx**: Multi-database storage
- **tree-sitter**: AST parsing (future)
- **git2**: Git integration (future)
- **tokio**: Async runtime
- **tracing**: Structured logging

### Dependencies
```toml
[dependencies]
clap = { version = "4.0", features = ["derive"] }
ratatui = "0.28"
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json", "stream"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## File Structure

```
src/
├── main.rs                 # Entry point - CLI arg parsing
├── app/
│   ├── mod.rs             # Main application struct
│   └── state.rs           # Application state management
├── cli/
│   ├── mod.rs             # CLI command handling
│   ├── oneshot.rs         # One-shot mode: aircher 'hello'
│   └── interactive.rs     # Interactive mode: aircher --interactive
├── ui/
│   ├── mod.rs             # TUI manager
│   ├── chat.rs            # Chat interface
│   ├── selection.rs       # Model selection modal
│   └── components/        # Reusable UI components
├── providers/             # ✅ Already implemented
│   ├── mod.rs             # Provider trait and manager
│   ├── claude_api.rs      # ✅ Claude API implementation
│   ├── gemini.rs          # ✅ Gemini API implementation
│   ├── openrouter.rs      # ✅ OpenRouter host implementation
│   └── hosts.rs           # ✅ Host abstraction
├── config/                # ✅ Already implemented
│   ├── mod.rs             # Configuration management
│   └── providers.toml     # Provider configurations
├── storage/               # ✅ Basic implementation
│   ├── mod.rs             # Database manager
│   └── conversations.rs   # Conversation persistence
└── utils/                 # ✅ Already implemented
    ├── mod.rs             # Utility functions
    └── errors.rs          # Error types
```

## Implementation Strategy

### Phase 0: Working User Interface

#### CLI-001: Basic CLI Functionality
```rust
// src/cli/oneshot.rs
pub async fn handle_oneshot(message: &str) -> Result<()> {
    let config = Config::load()?;
    let provider = ClaudeProvider::new(&config.claude_api_key)?;
    
    let request = ChatRequest {
        messages: vec![Message::user(message)],
        model: "claude-3.5-sonnet".to_string(),
        ..Default::default()
    };
    
    let response = provider.chat(&request).await?;
    println!("{}", response.content);
    Ok(())
}
```

#### CLI-002: Interactive CLI Chat Mode
```rust
// src/cli/interactive.rs
pub async fn handle_interactive() -> Result<()> {
    let mut conversation = Conversation::new();
    let provider = get_provider()?;
    
    loop {
        print!("> ");
        let input = read_line()?;
        
        match input.as_str() {
            "/quit" => break,
            "/help" => show_help(),
            "/clear" => conversation.clear(),
            message => {
                conversation.add_user_message(message);
                let response = provider.chat(&conversation.to_request()).await?;
                println!("AI: {}", response.content);
                conversation.add_assistant_message(&response.content);
            }
        }
    }
    Ok(())
}
```

#### TUI-001: Basic TUI Chat Interface
```rust
// src/ui/chat.rs
pub struct ChatWidget {
    messages: Vec<Message>,
    input: String,
    scroll_offset: usize,
}

impl ChatWidget {
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // Render chat history
        let chat_area = Rect::new(area.x, area.y, area.width, area.height - 3);
        self.render_messages(chat_area, buf);
        
        // Render input box
        let input_area = Rect::new(area.x, area.y + area.height - 3, area.width, 3);
        self.render_input(input_area, buf);
    }
}
```

## Configuration System

### Provider Configuration
```toml
# config/providers.toml
[providers.claude]
api_key_env = "ANTHROPIC_API_KEY"
base_url = "https://api.anthropic.com/v1"
models = ["claude-3.5-sonnet", "claude-3-haiku", "claude-3-opus"]

[providers.gemini]
api_key_env = "GOOGLE_API_KEY"
base_url = "https://generativelanguage.googleapis.com/v1beta"
models = ["gemini-2.0-flash-exp", "gemini-1.5-pro"]

[hosts.openrouter]
api_key_env = "OPENROUTER_API_KEY"
base_url = "https://openrouter.ai/api/v1"
pricing_multiplier = 0.75
```

### User Configuration
```toml
# ~/.config/aircher/config.toml
[default]
provider = "claude"
model = "claude-3.5-sonnet"
host = "direct"

[ui]
theme = "dark"
auto_save = true
show_tokens = true
show_cost = true
```

## Success Metrics

### Phase 0 Success Criteria
- [ ] CLI-001: `aircher 'hello world'` returns AI response
- [ ] CLI-002: `aircher --interactive` works for conversations
- [ ] TUI-001: Rich terminal interface launches and works
- [ ] TUI-002: Model selection modal works with our providers

### Quality Targets
- **Startup time**: < 100ms (CLI), < 200ms (TUI)
- **Memory usage**: < 50MB baseline, < 200MB with conversation
- **Error handling**: Graceful failures with helpful messages
- **User experience**: Intuitive keyboard shortcuts and navigation

## Testing Strategy

### Unit Tests
- Provider implementations (already working)
- CLI command parsing
- TUI component rendering
- Configuration loading

### Integration Tests
- End-to-end CLI workflows
- Provider API integration
- Configuration file loading
- Database operations

### Manual Testing
- Real API key testing with each provider
- UI responsiveness across terminal sizes
- Error scenarios (no API key, network issues)
- Cost calculation accuracy

## Deployment

### Single Binary Distribution
```bash
# Install from source
cargo install --git https://github.com/username/aircher

# Run one-shot
aircher 'hello world'

# Run interactive
aircher --interactive

# Run TUI
aircher  # default mode
```

### Configuration Setup
```bash
# First run - setup wizard
aircher --setup

# Manual configuration
export ANTHROPIC_API_KEY="your-key-here"
export GOOGLE_API_KEY="your-key-here"
aircher --configure
```

## Future Extensibility

### Intelligence Engine (Phase 1)
Once we have working UI, add:
- File relevance scoring with git2
- Context optimization with tree-sitter
- Cost optimization with usage analytics
- Pattern learning with SQLite

### Additional Providers (Phase 2)
- OpenAI (GPT-4, GPT-4o)
- Ollama (local models)
- Azure OpenAI (enterprise)
- Custom API endpoints

### Advanced Features (Phase 2)
- Session persistence and search
- Export conversations
- Plugin system
- Team configurations

## Key Principles

### User-First Development
1. **Working over perfect**: Get users a functional interface first
2. **Feedback-driven**: Let user experience guide feature priorities
3. **Incremental value**: Each phase should provide standalone value

### Technical Excellence
1. **Pure Rust**: Single language, single toolchain, single binary
2. **Async-first**: Non-blocking operations throughout
3. **Error handling**: Graceful failures with actionable messages
4. **Performance**: Fast startup, smooth interactions

### Maintainability
1. **Clean architecture**: Clear separation of concerns
2. **Comprehensive tests**: Unit, integration, and manual testing
3. **Documentation**: Keep specs updated with reality
4. **Monitoring**: Telemetry for usage patterns and errors

---

**Remember**: We have excellent provider architecture. Now we need excellent user interface to make it useful.