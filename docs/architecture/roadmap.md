# Aircher Development Roadmap

## Quick Summary (2025-08-26)

**Current Phase**: Phase 2 - Tool Calling Loop (Polish & Reliability)  
**Previous**: Phase 1 ✅ Agent Integration COMPLETED  
**Next**: Phase 3 - Core Tools Enhancement

**Key Discovery**: Agent system was already connected, just needed provider fixes!

## Current Architecture Status (August 2025)

**Reality Check (2025-08-26)**: Agent system IS connected; streaming and tool status integrated. See `TOOL_CALLING_REALITY_CHECK.md`.

### ✅ What's Working
- **TUI Chat Interface**: Full conversation UI with provider/model selection
- **Multi-Provider Support**: OpenAI, Anthropic, Gemini, Ollama integration  
- **Semantic Search**: Advanced code search with 19+ language support
- **Configuration System**: Hierarchical config with auth management
- **Basic Streaming**: Real-time response streaming from LLMs
- **Agent Integration**: AgentController connected to TUI (`src/ui/mod.rs:3797`)
- **Tool Registry**: 6+ working tools (file ops, code search, commands)
- **Tool Parsing**: XML and JSON format support
- **Ollama Tools**: Fixed provider to support gpt-oss tool calling

### ⚠️ What Needs Polish
- **Tool Result Display**: Add collapsible sections + code highlighting
- **Multi-turn Execution**: Reliability testing and loop termination guards
- **Error Recovery**: Concise provider/tool error surfaces
- **Permission UI**: Approval modal shortcuts and smoother flow

### ❌ What's Missing (Gap Analysis)  
- **Tool History**: Review of past tool executions
- **Advanced Tools**: Git, test execution, workspace awareness
- **Cost Tracking**: Per-tool usage metrics

## Development Phases

### Phase 0: Foundation Completion ✅
**Status**: COMPLETE
- ✅ Basic TUI with conversation flow
- ✅ Provider management and authentication
- ✅ Message streaming and display
- ✅ Configuration and settings

### Phase 1: Basic Agent Integration ✅ **COMPLETED**
**Goal**: Connect existing agent system to TUI
**Completed**: 2025-08-25
**Dependencies**: None

**Tasks Completed**:
1. ✅ Wire AgentController to TuiManager
2. ✅ Parse LLM responses for tool calls (XML + JSON)
3. ✅ Execute tools through existing registry  
4. ✅ Display tool results in conversation
5. ✅ Fix Ollama provider tool support

**Achievements**:
- Tool calling works with Ollama gpt-oss
- 6+ tools functional in registry
- Agent processes messages with tool execution
- Provider-specific bugs fixed

### Phase 2: Tool Calling Loop 🚨 **CURRENT PRIORITY**
**Goal**: Polish tool execution UX and reliability
**Timeline**: 1 week  
**Dependencies**: Phase 1 ✅

**Recent Progress**
- Structured tool calls (XML/JSON) parsed; Ollama streaming tool-calls surfaced
- Non-blocking send + operations line above input
- Predictive compaction pre-send (~85% context)
- Compact, one-line tool status/result lines with durations + batch header

**Tasks**
1. Expand multi-turn execution reliability; guard against infinite loops (max 10)
2. Improve error surfaces for provider and tool failures (concise, actionable)
3. Collapsible tool outputs + code highlighting for results
4. Optional: Tool history and replay view

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

### Phase 5: Intelligence-Driven Development
**Goal**: Revolutionary software development intelligence
**Timeline**: 4-6 weeks
**Dependencies**: Phase 4

**Features**:
1. **Enhanced Code Comprehension**:
   - Purpose analysis and business logic understanding
   - Architecture pattern detection and validation
   - Code quality analysis and improvement suggestions
   - Dependency mapping and relationship analysis

2. **Pattern-Aware Code Generation**:
   - Project-specific style learning and consistency
   - Architectural compliance and integration
   - Context-aware code that fits seamlessly
   - Intelligent error handling and logging patterns

3. **Intelligent Debugging Engine**:
   - Root cause analysis with system-wide impact assessment
   - Multiple fix strategy generation with risk analysis
   - Automated fix validation and regression prevention
   - Learning from successful fixes for future improvement

4. **Advanced Intelligence Features**:
   - Cross-file architectural understanding
   - Business domain concept extraction
   - Performance and security analysis
   - Automated code review and suggestions

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

| Feature | Claude Code | Cursor | Aircher (Current) | Aircher (Phase 3) | Aircher (Phase 5) |
|---------|------------|--------|-------------------|-------------------|-------------------|
| Basic chat | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tool calling | ✅ | ✅ | ✅ | ✅ | ✅ |
| File operations | ✅ | ✅ | ✅ | ✅ | ✅ |
| Shell commands | ✅ | ✅ | ✅ | ✅ | ✅ |
| Git integration | ✅ | ✅ | ✅ | ✅ | ✅ |
| Multi-provider | ❌ | Limited | ✅ | ✅ | ✅ |
| Semantic search | Limited | ❌ | ✅ | ✅ | ✅ |
| **Code comprehension** | **Limited** | **Limited** | **Basic** | **Enhanced** | **🚀 Revolutionary** |
| **Pattern learning** | **❌** | **Basic** | **Basic** | **Advanced** | **🚀 Project-aware** |
| **Intelligent debugging** | **❌** | **❌** | **❌** | **Basic** | **🚀 Root cause analysis** |
| **Architecture understanding** | **❌** | **❌** | **❌** | **❌** | **🚀 Deep architectural intelligence** |
| Task orchestration | Beta | ❌ | ❌ | ❌ | Advanced |
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

1. **Immediate** (This Week):
   - Phase 2 reliability testing for multi-turn tools (gpt-oss)
   - Improve provider error messaging + first-run prompts
   - Document current behavior in STATUS.md/AGENTS.md (done)

2. **Next**:
   - Phase 3 core tools (git/test/workspace) and result UX polish (collapsible + highlighting)

3. **Following**:
   - Phase 4 approvals UX and cost tracking; Phase 5 Turbo v1; Phase 6 orchestration

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
