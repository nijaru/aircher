# Aircher Core Project Guide

## What is Aircher?

**"Advanced AI terminal assistant with intelligent context management"**

- **Terminal Client**: Rust + Ratatui TUI with streaming LLM integration
- **Intelligence Engine**: Pure Rust with tree-sitter AST analysis and SQLite analytics  
- **Target Users**: Developers seeking AI-enhanced workflows with cost optimization
- **Key Differentiators**: Multi-provider support, intelligent context, cost transparency

## Architecture Overview

### Pure Rust Architecture (Single Binary)
```
┌─ Aircher Terminal (Pure Rust) ─┐
│  ├─ TUI (Ratatui)              │
│  ├─ LLM Providers              │
│  ├─ Session Management         │
│  └─ Intelligence Engine        │
│     ├─ Project Analyzer        │ (tree-sitter AST)
│     ├─ File Scorer             │ (git2 + heuristics)
│     ├─ Context Optimizer       │ (smart algorithms)
│     ├─ Pattern Tracker         │ (SQLite analytics)
│     └─ Cost Optimizer          │ (statistical analysis)
└─────────────────────────────────┘
```

### Technology Stack
- **Rust 1.80+**: Entire application (terminal, providers, intelligence)
- **Ratatui**: Terminal UI framework with async support
- **SQLite + sqlx**: Multi-database storage with async queries
- **tree-sitter**: AST parsing for all programming languages
- **git2**: Git integration and change analysis
- **tiktoken-rs**: Token counting for cost optimization

## Current Status

### The Reality Check
**We built sophisticated provider architecture but no working user interface**

- ✅ **Provider Architecture** (70%): Claude, Gemini, OpenRouter complete
- ❌ **User Interface** (0%): No working CLI or TUI - users can't interact with system
- ⏳ **Intelligence Engine** (0%): Context analysis, optimization
- ⏳ **Advanced Features** (0%): Session management, cost tracking

### New CLI-First Priority Sequence
**Phase 0: Working User Interface (CLI → TUI)**
1. 🚧 **CLI-001**: Basic CLI Functionality - `aircher 'hello world'`
2. ⏳ **CLI-002**: Interactive CLI Chat Mode - `aircher --interactive`
3. ⏳ **TUI-001**: Basic TUI Chat Interface - Rich terminal experience
4. ⏳ **TUI-002**: TUI Model Selection & Settings - Leverage provider architecture

**Phase 1: Integration & Polish**
5. ⏳ **INTEGRATION-001**: Provider Integration Testing - Fix what we built
6. ⏳ **SPRINT-004**: Intelligence Engine - Smart features
7. ⏳ **SPRINT-005**: Session Management - Persistence

## Quick Start for AI Agents

### Essential Files
- `CLAUDE.md` - AI agent instructions and workflow
- `docs/ARCHITECTURE.md` - Technical implementation details
- `docs/DEVELOPMENT.md` - Development patterns and setup
- `docs/tasks/tasks.json` - Current task priorities and status

### Development Commands
```bash
cargo build --release    # Build optimized binary
cargo test               # Run all tests  
cargo clippy            # Run linting
```

### Task Management
```bash
# View active tasks
jq '.tasks | to_entries | map(select(.value.status == "pending"))' docs/tasks/tasks.json

# Update task status  
jq '.tasks["TASK-ID"].status = "in_progress"' docs/tasks/tasks.json > tmp.json && mv tmp.json docs/tasks/tasks.json
```

## Goals & Success Metrics

### Primary Goals (Revised)
1. **Working user interface**: CLI then TUI that users can actually use
2. **Provider integration**: Use our sophisticated backend architecture
3. **Cost optimization**: 30-50% savings via smart provider/model selection
4. **Fast AI terminal**: <100ms startup, smooth 60fps rendering

### Success Metrics
- ✅ **CLI**: `aircher 'hello world'` returns AI response
- ✅ **Interactive Chat**: `aircher --interactive` for conversations
- ✅ **TUI**: Rich terminal interface for power users
- ✅ **Provider Selection**: Use our Claude/Gemini/OpenRouter architecture
- ⏳ **Cost Tracking**: Real-time usage and optimization
- ⏳ **Intelligence**: Context management and file suggestions

## Business Model & Positioning

### Market Position
- **vs Claude Code**: Multi-provider, cost optimization, advanced context management
- **vs Cursor**: Terminal-focused, cost-transparent, pattern learning across projects
- **vs Basic TUI tools**: Professional UI, intelligent suggestions, enterprise features

### Future Opportunities (Post-Integrated Success)
- Extract intelligence engine as MCP server for wider adoption
- Enterprise features: team configurations, compliance, analytics
- Community: plugins, themes, custom intelligence modules

## Key Architectural Decisions

### Why CLI-First (Not Advanced Features First)
- **User value**: People need to use the system before caring about advanced features
- **Validation**: Test our provider architecture with real usage
- **Feedback loop**: Users can provide input on what matters most
- **Marketing**: Working demo is better than sophisticated architecture docs

### Why Pure Rust (Not Rust + Python)
- **Simplicity**: Single language, toolchain, and runtime
- **Performance**: No serialization overhead, faster startup, lower memory usage
- **Deployment**: Single binary with zero dependencies
- **Intelligence needs**: Smart algorithms and database analytics, not heavy ML
- **Rust ecosystem**: tree-sitter, git2, and sqlx provide all needed capabilities

### Why Ratatui
- **Performance**: 60fps rendering, <100ms startup achievable
- **Ecosystem**: Proven by successful tools (Atuin, GitUI, Bottom)
- **Features**: Async support, rich widgets, professional styling
- **Keyboard-driven**: Optimal for developer workflows