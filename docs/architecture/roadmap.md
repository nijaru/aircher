# Aircher Development Roadmap

## Current Architecture Status (August 2025)

### ✅ What's Working
- **TUI Chat Interface**: Full conversation UI with provider/model selection
- **Multi-Provider Support**: OpenAI, Anthropic, Gemini, Ollama integration
- **Semantic Search**: Advanced code search with 19+ language support
- **Configuration System**: Hierarchical config with auth management
- **Basic Streaming**: Real-time response streaming from LLMs

### ⚠️ What's Built but Not Connected
- **Agent Controller**: Exists but not wired to TUI (`src/agent/controller.rs`)
- **Tool Registry**: Complete tool system but not used (`src/agent/tools/`)
- **File Operations**: Tools implemented but not accessible
- **Permission System**: Command approval built but not integrated

### ❌ What's Missing (Gap Analysis)
- **No Tool Calling Loop**: LLMs can't execute tools during conversation
- **No Agent Integration**: Agent controller disconnected from chat
- **No Tool UI**: No way to show tool execution in TUI
- **No Structured Output**: Still using text parsing instead of JSON

## Development Phases

### Phase 0: Foundation Completion ✅
**Status**: COMPLETE
- ✅ Basic TUI with conversation flow
- ✅ Provider management and authentication
- ✅ Message streaming and display
- ✅ Configuration and settings

### Phase 1: Basic Agent Integration 🚨 **CURRENT PRIORITY**
**Goal**: Connect existing agent system to TUI
**Timeline**: 3-5 days
**Dependencies**: None

**Tasks**:
1. Wire AgentController to TuiManager
2. Parse LLM responses for tool calls
3. Execute tools through existing registry
4. Display tool results in conversation
5. Handle tool approval flow

**Success Criteria**:
- Can read/write files through conversation
- Can execute shell commands with approval
- Tool status visible in UI

### Phase 2: Tool Calling Loop
**Goal**: Implement proper multi-turn tool use
**Timeline**: 1 week
**Dependencies**: Phase 1

**Tasks**:
1. Implement structured tool calling format (XML or JSON)
2. Add iterative tool execution (max 10 iterations)
3. Stream tool status updates to UI
4. Handle tool errors gracefully
5. Add context management for tool results

**Success Criteria**:
- Multi-step tasks complete successfully
- No context overflow from tool results
- Clear feedback on tool execution

### Phase 3: Core Tools Enhancement
**Goal**: Reach parity with Claude Code/Cursor
**Timeline**: 1 week
**Dependencies**: Phase 2

**Tasks**:
1. Enhance file operations (create, edit, delete)
2. Improve code search integration
3. Add git operations
4. Implement workspace awareness
5. Add image/screenshot support

**Tools to Implement**:
- `create_file` - Create new files with directories
- `edit_file` - Precise line-based editing
- `search_code` - Semantic search integration
- `git_status`, `git_diff`, `git_commit`
- `run_tests` - Language-aware test execution

### Phase 4: Enhanced UI/UX
**Goal**: Professional tool execution experience
**Timeline**: 1 week
**Dependencies**: Phase 3

**Features**:
1. Tool execution progress indicators
2. Collapsible tool output sections
3. Tool approval shortcuts (Y/N/A)
4. Tool history and replay
5. Cost tracking per tool use

### Phase 5: Advanced Features
**Goal**: Differentiate from competitors
**Timeline**: 2-3 weeks
**Dependencies**: Phase 4

**Features**:
1. **Turbo Mode v1**: Basic task parallelization
   - Bypass approval for safe operations
   - Parallel tool execution
   - Batch file operations
   
2. **Intelligence Features**:
   - Auto-detect project type and tools
   - Smart command suggestions
   - Learning from corrections

3. **Session Management**:
   - Save/restore with full context
   - Branch conversations
   - Export/import sessions

### Phase 6: Turbo Mode v2 (Orchestration)
**Goal**: Advanced task orchestration
**Timeline**: 2-3 weeks
**Dependencies**: Phase 5

**Features**:
- Task decomposition into subtasks
- Parallel subtask execution
- Two-tier model configuration
- Progress tree visualization
- See `docs/architecture/turbo-mode.md`

## Technical Decisions

### Tool Calling Format
**Decision**: Use XML-style format (like Claude)
```xml
<tool_use>
<tool_name>read_file</tool_name>
<parameters>
  <path>src/main.rs</path>
</parameters>
</tool_use>
```

**Rationale**:
- More reliable parsing than JSON in responses
- Clear boundaries for streaming
- Compatible with all providers

### Architecture Pattern
**Decision**: Event-driven with channels
```rust
TuiManager <-> AgentController <-> ToolRegistry
    ↑              ↓                    ↓
    UI Events   Tool Events        Tool Execution
```

### Tool Approval Flow
**Decision**: Inline approval with timeout
```
🔧 Tool Request: run_command
Command: cargo test
[Y]es / [N]o / [A]lways / [S]kip all (10s) >
```

## Comparison with Competitors

| Feature | Claude Code | Cursor | Aircher (Current) | Aircher (Phase 3) | Aircher (Phase 6) |
|---------|------------|--------|-------------------|-------------------|-------------------|
| Basic chat | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tool calling | ✅ | ✅ | ❌ | ✅ | ✅ |
| File operations | ✅ | ✅ | ❌ | ✅ | ✅ |
| Shell commands | ✅ | ✅ | ❌ | ✅ | ✅ |
| Git integration | ✅ | ✅ | ❌ | ✅ | ✅ |
| Multi-provider | ❌ | Limited | ✅ | ✅ | ✅ |
| Semantic search | Limited | ❌ | ✅ | ✅ | ✅ |
| Task orchestration | Beta | ❌ | ❌ | ❌ | ✅ |
| Parallel execution | Limited | ❌ | ❌ | Limited | ✅ |
| Cost optimization | ❌ | ❌ | ❌ | ✅ | ✅ |

## Risk Mitigation

### Technical Risks
1. **Context overflow**: Implement smart summarization
2. **Tool loops**: Hard limit of 10 iterations
3. **Malicious commands**: Permission system + sandboxing
4. **Rate limits**: Backoff and retry logic

### Timeline Risks
1. **Blocked on dependencies**: Each phase is independently valuable
2. **Scope creep**: Clear success criteria per phase
3. **Technical debt**: Refactor windows built into timeline

## Success Metrics

### Phase 1-2 Success (Basic Tools)
- Can complete file manipulation tasks
- Can run and interpret command output
- Users can build simple projects with guidance

### Phase 3-4 Success (Parity)
- Feature parity with Claude Code
- Can handle complex multi-file edits
- Professional developer workflow support

### Phase 5-6 Success (Leadership)
- Unique orchestration capabilities
- Significant cost savings through optimization
- Power users choose Aircher over competitors

## Next Steps

1. **Immediate** (Today):
   - Begin Phase 1 implementation
   - Wire AgentController to TuiManager
   - Test basic tool execution

2. **This Week**:
   - Complete Phase 1
   - Begin Phase 2 tool calling loop
   - Update STATUS.md with progress

3. **Next Month**:
   - Achieve competitor parity (Phase 3)
   - Begin differentiation features (Phase 4-5)

## Implementation Notes

### File Structure
```
src/
├── agent/
│   ├── controller.rs    # Wire to TUI here
│   ├── tools/           # Already implemented
│   └── orchestrator.rs  # Future (Phase 6)
├── ui/
│   ├── mod.rs          # Add agent integration
│   └── tool_display.rs # New: tool execution UI
└── providers/
    └── tool_calling.rs # New: unified tool format
```

### Key Integration Points
1. `TuiManager::handle_message()` - Parse for tool calls
2. `AgentController::execute()` - Process tools
3. `StreamingState` - Add tool status variant
4. `Message` - Add tool result type

This roadmap provides a clear path from our current state to market leadership, with each phase building on the previous one and delivering immediate value.