# TODO - Current Tasks & Priorities

**Last Updated**: 2025-09-10

## 🚨 Immediate (This Week) - DUAL-MODE ARCHITECTURE

### Phase 1: Architecture Foundation (Week 1)
- [x] Strategic analysis and planning (ACP compatibility assessed)  
- [x] Update competitive positioning for dual-mode approach
- [ ] Add agent-client-protocol dependency
- [ ] Extract agent core from TUI coupling
- [ ] Create shared agent core with pluggable tool layer
- [ ] Implement basic ACP Agent trait

### Completed Recent Work
- [x] TODO panel integrated into TUI layout
- [x] Rich model selection with metadata (context, pricing, capabilities)
- [x] Tool calling system validated (6 core tools working)
- [x] Agent controller properly connected to TUI

## 📋 Next Sprint (Phase 2 - Tool Calling Loop)

### Tool Result Display
- [ ] Implement collapsible tool result sections
- [ ] Add syntax highlighting for code in tool results

### Multi-turn Tool Execution
- [ ] Handle follow-up tool calls based on results
- [ ] Maintain context between tool calls
- [ ] Implement tool call chaining
- [ ] Add tool call history tracking

### Stream Tool Status
- [ ] Show real-time file operation status (progress-percentage where available)
- [ ] Display command execution output as it arrives for long commands
- [ ] Add cancellation support for long-running tools

## 🎯 Phase 3 - Core Tools Enhancement

### File Operations
- [ ] Implement multi-file edit support
- [ ] Add file creation with directory structure
- [ ] Implement safe file deletion with confirmation
- [ ] Add file move/rename operations
- [ ] Support batch file operations

### Git Integration
- [ ] Add git status display
- [ ] Implement git diff viewing
- [ ] Support commit creation
- [ ] Add branch operations
- [ ] Integrate git log viewing

### Workspace Awareness
- [ ] Detect project type (Rust/Python/JS/etc)
- [ ] Load project-specific configurations
- [ ] Understand build systems
- [ ] Parse dependency files
- [ ] Track open files context

### Test Execution
- [ ] Run tests with proper environment
- [ ] Parse test output formats
- [ ] Show test failure details
- [ ] Support different test frameworks
- [ ] Add test coverage reporting

## 🔧 Technical Debt

### Code Quality
- [ ] Fix remaining compiler warnings (20 warnings as of 2025-08-25)
- [ ] Add tests for tool calling integration
- [ ] Document agent system architecture
- [ ] Clean up unused imports and dead code
- [ ] Refactor large functions in TuiManager

### Documentation
- [ ] Update TECH_SPEC.md with agent architecture
- [ ] Document tool calling flow (streaming + operations line)
- [ ] Add provider integration guide
- [ ] Create troubleshooting guide
- [ ] Update API documentation

### Performance
- [ ] Profile tool execution bottlenecks
- [ ] Optimize message parsing
- [ ] Reduce memory usage in agent controller
- [ ] Cache provider responses where appropriate
- [ ] Batch tool operations when possible

## 💡 Future Ideas

### Turbo Mode v2 (Phase 6)
- [ ] Implement task orchestrator
- [ ] Add two-tier model configuration (high/low)
- [ ] Support parallel task execution
- [ ] Add task dependency resolution
- [ ] See `docs/architecture/turbo-mode.md`

### Intelligence Features
- [ ] Code understanding with AST analysis
- [ ] Smart refactoring suggestions
- [ ] Automatic bug detection
- [ ] Performance optimization hints
- [ ] Security vulnerability scanning

### Session Management
- [ ] Save/restore conversation sessions
- [ ] Branch conversation history
- [ ] Export sessions to markdown
- [ ] Share sessions between users
- [ ] Session templates for common tasks

## ✅ Recently Completed (2025-08-26)

- [x] Non-blocking sends (no UI freeze during streaming)
- [x] Operations line rendered above input (streaming status)
- [x] Predictive compaction before sending
- [x] TUI keybindings finalized (Enter/Shift/Ctrl+Enter, Tab, Ctrl+M)
- [x] Ollama defaults set (`ollama / gpt-oss`) and fallback to available model
- [x] Tool-line UX: compact status/results with durations, batch header
- [x] Ollama provider: streaming tool-calls surfaced at final chunk

## 📝 Notes

- Agent system more functional than documentation claimed
- Main issues are provider-specific bugs and polish
- Tool calling works but needs UX improvements
- Focus on reliability before adding new features

See `STATUS.md` for current development phase details.
