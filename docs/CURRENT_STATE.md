# Aircher Current State Analysis

## What Actually Works Today

### ✅ Fully Functional (2025-08-26)
- **TUI Chat Interface** (`src/ui/mod.rs`)
  - Non-blocking send/streaming; operations line above input shows live status
  - Provider/model selection (`/model`, Ctrl+M)
  - Authentication wizard (`/auth`)
  - Settings modal with “Submit on Enter” toggle

- **Multi-Provider Support** (`src/providers/`)
  - OpenAI, Anthropic, Gemini, Ollama
  - API key management
  - Model selection and validation

- **Semantic Search** (`src/search/`, `src/intelligence/`)
  - `/search` command in TUI
  - 19+ language support
  - hnswlib-rs vector backend with index persistence

- **Agent System** (`src/agent/`)
  - Connected to TUI; executes tools in-stream
  - Tool calls parsed (XML + JSON); Ollama streaming tool-calls surfaced
  - Tool status/result lines rendered in chat
  - Predictive compaction before sending to avoid context limits

### ⚠️ Needs Polish / Reliability
- Multi-turn tool execution: extend reliability tests and guard loops
- Tool result UX: collapsible sections + code highlighting for long outputs
- Provider error surfacing: concise, actionable one-liners
- Permission modal UX: shortcut keys and smoother flow

## Code Locations

### Key Files
1. **src/ui/mod.rs** (TuiManager)
   - Non-blocking send + agent stream draining
   - Operations line rendering above input
   - Predictive compaction preflight
   - Provider/model preflight and selection overlay

2. **src/agent/controller.rs**
   - Structured parsing of tool calls (XML/JSON)
   - Streaming updates (text chunks + tool status/result lines)
   - Tool execution loop with iteration cap

3. **src/providers/**
   - Provider adapters; Ollama streaming tool-calls exposed at final chunk

## Why This Matters

**Current user experience**:
```
User: "Read the file src/main.rs"
Assistant: "I'll read that file for you. [Describes what it would do but can't actually do it]"
```

**What competitors have**:
```
User: "Read the file src/main.rs"
Assistant: <tool_use>read_file</tool_use>
[Actually reads and shows file contents]
```

## Quick Test Commands

To verify current state:

```bash
# This works (chat):
cargo run --release
> /model  # select provider
> Hello   # get response

# This works (search):
> /search async functions

# This DOESN'T work (tools):
> Read the file Cargo.toml
# Bot will say it'll help but can't actually read the file
```

## The Gap (2025-08-26)

The core loop is connected and functional. The remaining work is reliability and UX polish for tool-heavy, multi-turn tasks and improving provider/model first-run flows.

## Next Steps

1. Reliability: multi-turn tool execution tests with gpt-oss
2. UX: collapsible tool result sections; code highlighting
3. Provider UX: improved first-run prompts when models/providers are unavailable
4. Optional: configurable Enter behavior (implemented) and discoverability in docs
