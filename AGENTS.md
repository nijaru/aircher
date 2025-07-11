# Aircher - AI Development Terminal

## 🚨 CRITICAL WORKFLOW 🚨

### ✅ ALWAYS
- **TASK_TRACKING**: Use `docs/tasks/tasks.json` ONLY - update status every session
- **VALIDATION**: Run `cargo clippy && cargo test` before commits  
- **ARCHITECTURE**: Verify changes against `docs/ARCHITECTURE.md` before implementation
- **EFFICIENCY**: Load only essential docs to optimize tokens

### ❌ NEVER  
- **DUPLICATE_TASKS**: Create tasks outside `docs/tasks/tasks.json`
- **SKIP_TESTS**: Commit without running linting and tests
- **TECH_DEBT**: Quick fixes without considering architecture patterns

## Project Overview

**"Advanced AI terminal assistant with intelligent context management"**

- **Technology**: Pure Rust with Ratatui TUI and integrated intelligence engine
- **Architecture**: Single binary with built-in intelligence (not MCP server)
- **Target**: Developers seeking AI-enhanced workflows with cost optimization
- **Differentiators**: Multi-provider support, cost transparency, intelligent context

### Current Status: Working User Interface Phase
**Reality Check**: We built sophisticated provider architecture but no working user interface

- ✅ **Provider Architecture** (70%): Claude, Gemini, OpenRouter complete
- ❌ **User Interface** (0%): No CLI or TUI - users can't interact with system
- ⏳ **Intelligence Engine** (0%): Context analysis, optimization  
- ⏳ **Advanced Features** (0%): Session management, cost tracking

### Priority: CLI-001 → CLI-002 → TUI-001 → TUI-002 → Integration Testing

## Essential Resources

### Core Documentation
- `docs/CORE.md` - Project goals, status, business context
- `docs/ARCHITECTURE.md` - Technical implementation details
- `docs/DEVELOPMENT.md` - Development patterns, testing, workflows
- `docs/tasks/tasks.json` - Current task priorities and status
- `docs/tasks/README.md` - Complete task management system guide

### Current Focus: Phase 0 (Working User Interface)
**Next Tasks**: CLI-001 → CLI-002 → TUI-001 → TUI-002

### Quick Commands
```bash
# Development
make check  # runs fmt + lint + test

# Task management  
make current-tasks    # Show active work
make progress        # Show overall status
make validate-tasks  # Check JSON is valid

# Update task status
jq '.tasks["TASK-ID"].status = "in_progress"' docs/tasks/tasks.json > tmp.json && mv tmp.json docs/tasks/tasks.json

# Add notes to task
jq '.tasks["TASK-ID"].notes = "Progress update"' docs/tasks/tasks.json > tmp.json && mv tmp.json docs/tasks/tasks.json
```

## Architecture Summary

### Provider → Model → Host Hierarchy
```
Provider: Who created model (OpenAI, Anthropic, Google)
Model: Specific version (gpt-4o, claude-3.5-sonnet, llama-3.1-70b)
Host: Where accessed (Direct API, OpenRouter, Local, Enterprise)
```

### Technology Stack
- **Rust 1.80+**: Entire application (terminal, providers, intelligence)
- **Ratatui**: Terminal UI framework with async streaming
- **SQLite + sqlx**: Multi-database storage with async queries
- **tree-sitter**: AST parsing for all programming languages
- **git2**: Git integration and change analysis

### Key Components
```rust
pub struct ArcherApp {
    ui: TuiManager,              // Ratatui interface
    providers: ProviderManager,  // Multi-provider LLM system
    intelligence: Intelligence,  // Pure Rust intelligence engine  
    sessions: SessionManager,    // Conversation persistence
    config: ConfigManager,       // TOML configuration
}
```

## Development Workflow

### For AI Agents
1. **Check tasks**: Review `docs/tasks/tasks.json` for CLI-001, CLI-002, TUI-001, TUI-002
2. **Load context**: Reference task-specific documentation from above
3. **Implement**: Focus on user interface before advanced features
4. **Validate**: Run `cargo clippy && cargo test`
5. **Update progress**: Mark task status in `docs/tasks/tasks.json`

### Current Reality
**Working**: Sophisticated provider architecture (Claude, Gemini, OpenRouter)
**Missing**: Any way for users to interact with the system
**Next**: CLI-001 (Basic CLI) - `aircher 'hello world'` should work

### Code Standards
- **Rust**: snake_case functions, PascalCase types, async/await, Result types
- **Config**: TOML for all configuration, environment variable resolution
- **Tests**: Unit tests in modules, integration tests in tests/, comprehensive async testing
- **Intelligence**: Pure Rust algorithms using tree-sitter, git2, and SQLite analytics

### Error Recovery
```bash
# If compilation fails
grep -r "error\|warning" target/ | head -10

# If tests fail  
cargo test -- --nocapture

# If architecture questions
grep -r "pattern\|trait\|interface" docs/ARCHITECTURE.md
```

## Cost Optimization Features

### Multi-Host Support
- **OpenRouter**: 25% cheaper than direct APIs
- **Direct APIs**: Standard pricing, maximum reliability  
- **Local**: Free but requires setup (Ollama, etc.)
- **Enterprise**: Azure/AWS for compliance

### Smart Model Selection
- **Task-aware**: Cheap models for simple tasks (commit messages: gpt-3.5-turbo)
- **Quality-focused**: Premium models for complex work (code review: gpt-4)
- **Cost tracking**: Real-time session costs and budget management

## Key Differentiators

### vs Claude Code
- **Multi-provider**: Not locked to single provider (when UI works)
- **Cost optimization**: Choose optimal provider/model/host combination
- **Advanced context**: Intelligent file relevance and pattern recognition

### vs Cursor  
- **Terminal-focused**: Keyboard-driven workflow optimized for developers
- **Cost transparency**: Real-time pricing and savings calculations  
- **Pattern learning**: Cross-project intelligence and optimization

### Current State Honestly
**Strengths**: Excellent provider architecture, clean Rust design
**Weakness**: No working user interface - users can't use any of our features
**Fix**: CLI-001 → CLI-002 → TUI-001 → TUI-002

---

**Remember**: Check `docs/tasks/tasks.json` → Load relevant docs → Implement → Test → Update tasks → Commit

**Token Efficiency**: This file contains all essential information. Load additional docs only when implementing specific features.