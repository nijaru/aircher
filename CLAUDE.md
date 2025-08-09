# Aircher Agent Instructions

This document contains important context and decisions for AI agents working on the Aircher codebase.

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

- **Tab** in input: Insert 4 spaces for code
- **Tab/←/→** in /model: Switch between provider and model selection
- **Ctrl+C**: Clear input if text exists, quit if empty (single press)
- **Double Escape** (within 400ms): Show conversation history modal
- **PgUp/PgDown**: Scroll chat history (planned)

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

**CRITICAL**: The agent system (`src/agent/`) exists but is NOT connected to the TUI. This must be fixed before any advanced features.

### Current Focus: Basic Agent Integration
1. Connect `AgentController` to `TuiManager` in `src/ui/mod.rs`
2. Parse LLM responses for tool calls (XML format like `<tool_use>`)
3. Execute tools through the existing `ToolRegistry`
4. Display tool results in the conversation

See `docs/architecture/roadmap.md` for the complete development plan.

### Future: Turbo Mode (Phase 6)
After basic tool calling works, turbo mode will add task orchestration with two-tier model configuration (high/low). Design available in `docs/architecture/turbo-mode.md` but this is NOT the current priority.