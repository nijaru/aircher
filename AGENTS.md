# Aircher Agent Instructions

Entry point for AI agents working with Aircher - the multi-modal AI coding agent.

@external/agent-contexts/AI_AGENT_INDEX.md

## Key Files (Always Check/Update)

### 📊 Project Status & Reality
@PROJECT_STATUS.md                 # **READ FIRST**: Current capabilities & limitations
@internal/PROJECT_REALITY.md       # **HONEST ASSESSMENT**: Real vs claimed functionality
@internal/NOW.md                   # Current tasks & sprint status

### 🏗️ Architecture & Decisions
@docs/architecture/agent-first-architecture.md  # CRITICAL: New unified agent design
@docs/architecture/MODEL_VS_AGENT_ARCHITECTURE.md  # Key insight: Model vs agent responsibilities
@internal/DECISIONS.md             # Major decisions (append-only)
@internal/TECH_SPEC.md             # Technical specifications

### 🔬 Research & Intelligence
@internal/KNOWLEDGE.md             # Patterns & learnings
@internal/DISCOVERIES.md           # Competitive insights & breakthroughs

### 📚 Development Reference
@internal/STATUS.md                # Development phase tracking
@external/agent-contexts/standards/AI_CODE_PATTERNS.md  # Universal coding patterns

## Project Overview

**Dual-mode AI coding agent**: Terminal UI + Agent Client Protocol support

**⚠️ CRITICAL**: See @PROJECT_STATUS.md and @internal/PROJECT_REALITY.md for honest assessment
- Status: ~16-20% feature parity with Claude Code (stable infrastructure, limited functionality)
- Architecture: Single UnifiedAgent with multiple frontends (LocalClient for TUI, ACP for editors)

**What Works**: Semantic search (production-ready), TUI interface, multi-provider auth
**What Doesn't**: Most tools are stubs returning fake JSON

## Key Architecture Insight (Sep 19, 2025)

**Models are reasoning engines, agents are execution engines**
- Over-engineered: 1685-line MultiTurnReasoningEngine externalized what models do internally
- Research validated: 25-70% improvements from prompts, not orchestration
- Solution: Enhanced prompting system (300 lines) replaces complex orchestration
- Details: @docs/architecture/MODEL_VS_AGENT_ARCHITECTURE.md

**Note**: `src/agent/sub_agents.rs` exists but is DEPRECATED - we pivoted away from this approach.

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

## Development Philosophy

**Tool Approach**: Shell-first (power user of CLI tools vs reimplementation)
**Code Standards**: @external/agent-contexts/standards/AI_CODE_PATTERNS.md
**Development Flow**: Demo mode must work, performance critical, zero warnings tolerance

## TUI Features

**Notification System**: Operations line (above input), toast notifications (3s fade), status bar, inline system messages
**UX**: Fast Rust startup, smart defaults, stable input position
**Keyboard**: Enter (submit), Shift+Enter (newline), Tab (autocomplete/indent), Esc (close/interrupt), Ctrl+M (/model), Double Esc (history)

**Dynamic Features**: Streaming progress, predictive compaction (85% threshold), real-time model discovery from provider APIs
**Model Selection**: Rich metadata (context, pricing, capabilities), fuzzy autocomplete, smart caching

## Development Status

**Current Focus**: Intelligence-driven software development (pattern-aware comprehension/generation)
**Roadmap**: @docs/architecture/roadmap.md | @internal/TECH_SPEC.md | @docs/intelligence/INTELLIGENCE_ENHANCEMENT_PLAN.md

### Key Directories
- `src/ui/` - TUI (auth, model selection)
- `src/semantic_search.rs` - Production search
- `src/intelligence/` - Context engine
- `src/providers/` - Multi-provider APIs
- `src/agent/` - Tool system

## Quick Reference

**Build**: `cargo check` (zero warnings required), `cargo test`, `cargo run --release` (demo mode)
**Code Style**: @external/agent-contexts/standards/AI_CODE_PATTERNS.md

## Tool Format

**XML**: `<tool_use><tool>read_file</tool><params>{"path": "Cargo.toml"}</params></tool_use>`
**JSON**: `{"tool": "list_files", "params": {"path": "src/agent/tools"}}`

**Guidelines**: Read before editing, use `search_code` to locate targets, keep params minimal, chain tools over turns
**Status Display**: Running (🔧), Success (✓ with duration), Error (✗ with code), Batch (🔧 count)
