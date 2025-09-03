# Aircher Agent Instructions

This document provides context and instructions for AI agents working with the Aircher project.

@docs/agent-contexts/AI_AGENT_INDEX.md

## Intelligence System Design

@docs/intelligence/autonomous-agent-design.md  # Core autonomous intelligence architecture
@docs/intelligence/refactoring-plan.md         # Implementation roadmap and phases
@docs/ui/tui-interface-design.md               # TUI enhancements for intelligence features

## Project Overview

Aircher is a **semantic code search engine** with TUI interface built in Rust. Currently production-ready for search functionality, with AI chat capabilities in development.

**What Works Today:**
- Advanced semantic code search (production-ready)
- TUI interface with graceful auth flow 
- Multi-provider authentication (Claude, OpenAI, Gemini, Ollama)
- Demo mode (full functionality without API keys)

**What's Missing (Current Development Priority):**
- Tool calling reliability & UX polish (agent connected; working with Ollama)
- Multi-turn tool execution refinement
- File operations through chat interface (basic tools exist)

## Current Status

- ✅ **Production-ready** semantic search system (99.9% faster subsequent searches)
- ✅ **TUI with demo mode** - full interface available without API keys
- ✅ **Clean codebase** - eliminated ~190 compiler warnings
- ✅ **19+ language support** with tree-sitter parsing
- ✅ **AI chat with tools** - agent connected to TUI; Phase 2 reliability in progress

## Key Features Completed

### 1. Semantic Code Search (Production Ready)
- Sub-second search with index persistence  
- Advanced filtering, query expansion, typo correction
- 19+ programming languages supported
- Real-time file monitoring

### 2. TUI Interface (Fully Functional)
- Demo mode works immediately without setup
- Interactive auth wizard for provider setup
- Model selection and provider switching
- Graceful fallback when API keys unavailable

### 3. Multi-Provider Support (Authentication Only)
- OpenAI, Anthropic, Gemini, Ollama integration
- OAuth2 support for Anthropic Pro/Max
- Dynamic model fetching from provider APIs

## Architectural Decisions

### Shell-First Approach for Language Tools

**Decision**: Use shell commands for language tooling instead of native integrations.

**Rationale**:
- **Simplicity**: No complex integrations to maintain
- **Transparency**: Users can see and reproduce exactly what the agent does
- **Flexibility**: Works with any tool immediately without integration work
- **Reliability**: Shell commands are stable interfaces

**Implementation Guidelines**:

1. **Prefer structured output when available**:
   ```bash
   # Good - use JSON output for parsing
   cargo test --format json
   pytest --json-report
   npm test --json
   
   # Parse with jq when needed
   cargo metadata --format-version 1 | jq '.packages'
   ```

2. **Use language servers over stdio**:
   ```bash
   # Instead of integrating LSP client libraries
   echo '{"method":"textDocument/definition"...}' | rust-analyzer
   ```

3. **Smart command detection**:
   ```bash
   # Check for features before using them
   if cargo test --help | grep -q "format json"; then
       cargo test --format json
   else
       cargo test  # fallback to text parsing
   fi
   ```

4. **Common patterns to remember**:
   - `rustc --error-format json` - structured compiler errors
   - `cargo clippy --message-format json` - linting with JSON
   - `rg --json` - ripgrep with structured output
   - `git log --format=json` - when available
   - Language servers (rust-analyzer, pyright, etc.) work over stdio

**What NOT to do**:
- Don't build native integrations for each tool
- Don't add language-specific dependencies
- Don't hide what commands are being run

## Tool Philosophy

The agent should be a power user of existing CLI tools rather than reimplementing functionality. This makes the agent's actions:
- Understandable to users
- Reproducible outside the agent
- Maintainable without language-specific knowledge
- Immediately compatible with new tools

## Code Standards

- Add newlines at end of files
- Follow existing code patterns
- Check for existing dependencies before adding new ones
- Prefer editing existing files over creating new ones

## Recent UI Improvements

### Notification System (Implemented)
A layered notification system provides appropriate feedback for different types of information:

1. **Operations Line** (above input) - Active work with progress:
   - `🔄 Loading Ollama models... [████░░░░] 67%`
   - `🔍 Searching for "auth" in src/...`
   - Only appears when operations are active

2. **Toast Notifications** (top-right) - Temporary alerts (3s fade):
   - `✓ Authentication successful`
   - `⚠️ Rate limit warning`
   - Auto-expire after 3 seconds

3. **Status Bar** (bottom) - Persistent context:
   - Model/provider info
   - Context usage percentage
   - Cost tracking

4. **Inline System Messages** - Important history:
   - Model changes
   - Authentication status updates

### Key UX Decisions

1. **Fast Startup**: Rust-based TUI starts instantly, differentiating from Electron-based alternatives
2. **Smart Defaults**: `/model` command starts with provider selection if not authenticated
3. **Stable Input Position**: Operations line appears above input so typing area never moves
4. **Clean Interface**: Notifications only appear when relevant

### Keyboard Shortcuts

- Enter: submit message
- Shift+Enter or Ctrl+Enter: newline
- Tab: accept autocomplete suggestion (if visible); otherwise insert 4 spaces (indent)
- Esc: close autocomplete or interrupt streaming
- Ctrl+M: open model selection overlay (/model also supported)
- Ctrl+C: clear input if text exists, quit if empty (single press)
- Double Escape (within 400ms): Show conversation history modal
- PgUp/PgDown: Scroll chat history (planned)

### Streaming & Operations Line (Implemented)
- Streaming progress renders inside the chat area in a one-line operations bar directly above the input.
- Shows rotating status like “Retrieving content… (Xs · tokens · esc to interrupt)”.

### Predictive Compaction (Implemented)
- Before sending long messages, the TUI checks projected context usage against the current model’s window.
- If auto-compaction is enabled, we compact proactively at ~85% headroom; otherwise, we show a warning.

### Dynamic Model Fetching (Fully Implemented)

Real-time model discovery from provider APIs for always-current model lists:

1. **Zero-latency selection** - Models prefetch as users navigate providers:
   - Hover/arrow through providers → models load in background
   - Fast internet users see models before hitting Enter
   - Smart caching prevents redundant API calls

2. **Progressive enhancement approach**:
   - Start with configured models for immediate response
   - Enhance with real API data when available
   - Fallback to config if API calls fail

3. **Provider-specific implementations**:
   - **OpenAI**: `/v1/models` API with intelligent filtering and sorting
   - **Anthropic**: Config-based (no public models API yet)
   - **Gemini**: Config-based with planned API integration
   - **Ollama**: Dynamic local model discovery via `/api/tags` (fixed parsing)
   - **OpenRouter**: Gateway models API for full catalog

4. **UX optimizations**:
   - Loading states with clear "Loading models..." feedback
   - No duplicate fetches per provider per session
   - Models prioritized by recency/capability

5. **Technical implementation**:
   - Async background tasks fetch models without blocking UI
   - Channel-based communication between async tasks and TUI event loop
   - Automatic model updates processed each render cycle

### Enhanced Model Selection UX (Implemented)

Rich metadata display for informed model selection:

1. **Smart formatting**:
   - Context window: `200k ctx`, `2M ctx` (auto-scales units)
   - Pricing: `$3.00⇄$15.00` (input⇄output), `Free` for local models
   - Capabilities: 🔧 (tools/functions), ⚡ (streaming support)

2. **Visual indicators**:
   - ⭐ Default/recommended model (first in list)
   - 🧠 Large context models (1M+ tokens)

3. **Fuzzy autocomplete**:
   - Typo tolerance: `/mdl` → `/model`
   - Case insensitive: `/M` → `/model`
   - Alias support: `/m` → `/model`

### Provider Manager and Model Selection Fixes (Latest)

Critical fixes for model selection reliability and user experience:

1. **Provider Manager Initialization**:
   - Fixed race condition where model selection opened before provider manager was available
   - Added graceful loading states (`🔄 Loading models...`) instead of error messages
   - Implemented automatic retry when provider manager becomes available
   - Enhanced debug logging for troubleshooting initialization issues

2. **Model Selection UX Improvements**:
   - Loading states replace harsh error messages during initialization
   - Models automatically refresh when provider manager is set
   - Multiple initialization paths now properly handle provider manager setup
   - Removed "Provider manager not initialized" errors in favor of smooth loading experience

3. **Robust Error Handling**:
   - `update_model_items()` gracefully handles missing provider manager
   - Smart detection of loading states for automatic refresh
   - Enhanced debug output for tracing initialization flow
   - Provider manager setup happens in multiple strategic locations

4. **Technical Implementation**:
   - `set_provider_manager()` now triggers model refresh if needed
   - `show_model_selection_with_auth_check()` includes double-check mechanism
   - Loading state detection prevents users from seeing internal errors
   - Provider manager availability checked before dynamic model fetching

### Planned Features

- Progress bars for operations that support percentage tracking
- Mouse scroll support for chat history
- Intelligent prompting for auth when no providers configured
- Session branching and message indexing for conversation management

## Development Priorities

**REALITY CHECK (Aug 2025)**: Agent system IS connected to TUI and partially functional. See `TOOL_CALLING_REALITY_CHECK.md` for empirical testing results.

### Current Focus: Tool Calling Reliability & UX
1. ✅ **COMPLETED**: AgentController connected to TuiManager (`src/ui/mod.rs:3797-3815`)
2. ✅ **COMPLETED**: Tool call parsing (XML + JSON formats via `ToolCallParser`)
3. ✅ **COMPLETED**: Tool execution through `ToolRegistry` (6+ functional tools)
4. ✅ **COMPLETED**: Ollama provider tool support (fixed hardcoded `false` issue)
5. 🔧 **IN PROGRESS**: End-to-end tool calling reliability testing
6. 📋 **NEXT**: Tool result display and conversation flow UX

See `docs/architecture/roadmap.md` for the complete development plan and `TECH_SPEC.md` for technical details.

### Future: Turbo Mode (Phase 6)
After basic tool calling works, turbo mode will add task orchestration with two-tier model configuration (high/low). Design available in `docs/architecture/turbo-mode.md` but this is NOT the current priority.

## Working With This Codebase

### Key Directories
- `src/ui/` - TUI implementation with auth flow and model selection
- `src/semantic_search.rs` - Production-ready search functionality  
- `src/intelligence/` - Context-aware assistance engine
- `src/providers/` - Multi-provider authentication and API integration
- `src/agent/` - Tool system (exists but not connected to TUI)
- `tests/` - Comprehensive test suite

### Development Flow
1. **Maintain user experience** - demo mode must work without API keys
2. **Performance is critical** - search must remain sub-second
3. **Zero tolerance for warnings** - fix immediately
4. **Test before committing** - run `cargo test` and manual TUI testing

### What Actually Works vs Documentation
- ✅ **Semantic search**: `/search` command works perfectly in TUI
- ✅ **Provider auth**: `/auth` and `/model` commands work
- ✅ **Demo mode**: Launch `aircher` without any setup
- ❌ **AI chat with tools**: Can chat with LLMs but they can't use tools yet

## Build & Test Commands

```bash
# Core development commands
cargo check          # Type checking and warnings (MUST be zero warnings)
cargo test           # Run all tests
cargo test -- test_name  # Run single test by name
cargo run --release  # Start TUI (demo mode works without API keys)

# Linting and formatting
cargo clippy --all-targets --all-features  # Linting
cargo fmt            # Code formatting
make check           # Run fmt + lint + test

# Manual testing scenarios
cargo run --release  # Test demo mode
# In TUI: /search error handling (semantic search)
ANTHROPIC_API_KEY=sk-ant-... cargo run --release  # Test with API keys
```

## Code Style Guidelines

- **Error Handling**: Use `anyhow::Result` for functions, `thiserror::Error` for custom error types
- **Imports**: Group by std, external crates, internal modules with empty lines between
- **Naming**: snake_case for functions/variables, PascalCase for types, SCREAMING_SNAKE for constants
- **Documentation**: Use `///` for public items, focus on behavior not implementation
- **Async**: Use `async_trait` for trait methods, prefer `tokio` ecosystem
- **Newlines**: Always end files with newlines, follow existing patterns exactly

## Critical Architecture Gaps

**The Big Issue**: AgentController exists in `src/agent/controller.rs` but is never instantiated or connected to the TUI. This means:

- Users can chat with LLMs through the TUI
- LLMs receive messages but cannot execute any tools
- Tools like `read_file`, `write_file`, `run_command` are implemented but unreachable
- No tool calling loop, no file operations, no command execution

**Phase 1 Fix Required**: Connect `AgentController` to `TuiManager` in `src/ui/mod.rs`

## Tool Use Guide (Claude/GPT)

Preferred tool call format (parser-friendly across providers):

```
<tool_use>
<tool>read_file</tool><params>{"path": "Cargo.toml", "line_start": 1, "line_end": 120}</params>
</tool_use>
```

Also accepted (OpenAI-style JSON):

```
{"tool": "list_files", "params": {"path": "src/agent/tools", "recursive": false}}
```

Guidelines:
- Read before editing; use `search_code` to locate targets.
- Keep params minimal and precise; avoid dumping large content.
- Chain tools over multiple turns where helpful.

TUI tool status lines (single-line summaries):
- Running: `🔧 read_file Cargo.toml — running…`
- Success: `✓ read_file Cargo.toml — 120 lines (48ms)`
- Error: `✗ run_command cargo test — exit 101`
- Batch: `🔧 Executing 3 tools…`

Notes:
- Tool lines include durations when available.
- `run_command` supports a `timeout_seconds` parameter (default 30s).

## TUI Keybindings

- Enter: submit message
- Shift+Enter or Ctrl+Enter: insert newline
- Tab: accept autocomplete suggestion (if visible)
- Esc: close autocomplete or interrupt streaming
- Ctrl+M: open model selection overlay
- /model: open model selection via slash command

Future option (planned): configurable Enter behavior.
- Proposal: `ui.submit_on_enter` boolean in config to switch default between submit vs newline, and a list of newline shortcuts (e.g., `["shift+enter", "ctrl+enter"]`).

## Predictive Compaction

Before sending long messages or tool-heavy turns, the TUI checks projected context usage against the model’s window and will compact proactively when auto-compaction is enabled. A brief system notice is shown when this happens.
