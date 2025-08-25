# TODO - Current Tasks & Priorities

**Last Updated**: 2025-08-25

## 🚨 Immediate (This Week)

### Tool Calling Reliability
- [ ] Test end-to-end tool execution through TUI chat interface
- [ ] Verify tool result integration in conversation flow
- [ ] Test multi-turn conversations with tool results
- [ ] Handle tool execution errors gracefully
- [ ] Add retry logic for failed tool calls

### Provider Testing
- [ ] Test tool calling with OpenAI GPT-4
- [ ] Test tool calling with Anthropic Claude
- [ ] Test tool calling with Gemini
- [ ] Document provider-specific tool calling formats

## 📋 Next Sprint (Phase 2 - Tool Calling Loop)

### Tool Result Display
- [ ] Format tool results in conversation UI
- [ ] Add tool execution status indicators
- [ ] Show tool calling progress/spinner
- [ ] Implement collapsible tool result sections
- [ ] Add syntax highlighting for code in tool results

### Multi-turn Tool Execution
- [ ] Handle follow-up tool calls based on results
- [ ] Maintain context between tool calls
- [ ] Implement tool call chaining
- [ ] Add tool call history tracking

### Stream Tool Status
- [ ] Stream tool execution progress
- [ ] Show real-time file operation status
- [ ] Display command execution output as it arrives
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
- [ ] Document tool calling flow
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

## ✅ Recently Completed (2025-08-25)

- [x] Fix Ollama provider tool support (was hardcoded to false)
- [x] Verify agent system IS connected to TUI
- [x] Document actual vs assumed functionality
- [x] Test tool calling with gpt-oss model
- [x] Update documentation to reflect reality
- [x] Parse OpenAI-style JSON tool calls
- [x] Fix OllamaMessage struct fields

## 📝 Notes

- Agent system more functional than documentation claimed
- Main issues are provider-specific bugs and polish
- Tool calling works but needs UX improvements
- Focus on reliability before adding new features

See `STATUS.md` for current development phase details.