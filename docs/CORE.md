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

### Phase Progress
- ✅ **Foundation** (100%): Project setup, Rust architecture
- 🚧 **LLM Integration** (60%): Claude API complete, subscription pending
- ⏳ **Intelligence Engine** (0%): Context analysis, optimization
- ⏳ **Advanced Features** (0%): TUI, session management

### Priority Sequence
1. ✅ **SPRINT-001**: Claude API Provider Implementation (Complete)
2. 🚧 **SPRINT-001B**: Claude Pro/Max Subscription Integration
3. ⏳ **SPRINT-002**: Gemini Provider Implementation
4. ⏳ **SPRINT-003**: OpenRouter Host Integration

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

### Primary Goals
1. **Fast AI terminal**: <100ms startup, smooth 60fps rendering
2. **Cost optimization**: 30-50% savings via smart provider/model selection
3. **Intelligent context**: AI-driven file relevance and pattern recognition
4. **Multi-provider support**: OpenAI, Anthropic, OpenRouter, local models

### Success Metrics
- Working terminal with streaming LLM responses
- Provider → Model → Host selection UI
- Cost tracking and optimization
- Session persistence and management
- Intelligent file context suggestions

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

### Why Integrated (Not MCP First)
- **User acquisition**: Easier to explain and adopt complete solution
- **Development speed**: Single build/test/deploy cycle, faster iteration
- **Performance**: Direct function calls vs JSON-RPC overhead
- **User experience**: Optimized for Aircher's specific needs

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