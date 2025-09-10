# Aircher Competitive Development Roadmap

*Strategic plan to make Aircher competitive with Amp, Claude Code, and other coding agents*

## Our Market Position

**Target Users**: Developers who want:
- **Choice** in models and providers (vs Amp's "always best models")
- **Transparency** in costs and usage
- **Local options** (Ollama) alongside cloud providers
- **Fast terminal interface** (vs Electron-based competitors)

## Competitive Analysis

### vs Amp
- **Amp's Strength**: Curated "always best" models, no choice complexity
- **Our Advantage**: Model selection transparency, multi-provider flexibility, local model support
- **Must Match**: TODO panel, tool calling reliability, conversation UX

### vs Claude Code  
- **Their Weakness**: Electron-based, performance issues with long conversations
- **Our Advantage**: Native Rust TUI, fast startup, efficient rendering
- **Must Match**: Polish, conversation threading

### vs OpenCode/Cursor
- **Their Strength**: IDE integration
- **Our Advantage**: Terminal-first workflow, faster for CLI-heavy developers
- **Must Match**: Code editing capabilities

## Development Phases

### Phase 1: Core Functionality (Week 1)
**Goal**: Fix critical blockers, establish foundation

**Priority Tasks**:
1. **Fix Tool Calling System** (Critical)
   - Debug tool execution pipeline
   - Fix agent-to-TUI tool integration
   - Test end-to-end tool workflows

2. **Enhanced Model Selection UX** (Differentiation)
   - Polish model picker with rich metadata
   - Add cost estimation and comparison
   - Smart recommendations based on task type

3. **Clean Codebase** (Quality)
   - Fix all build warnings (currently 8)
   - Optimize imports and dead code
   - Improve error handling

4. **Basic TODO System** (Parity)
   - Implement TODO panel in TUI
   - Basic CRUD operations for tasks
   - Integration with agent tool execution

**Success Metrics**:
- ✅ Agent can successfully use tools (read_file, write_file, run_command)
- ✅ Model selection feels polished and informative
- ✅ Zero compiler warnings
- ✅ Basic TODO tracking functional

### Phase 2: UX Polish (Week 2-3)
**Goal**: Match Amp's conversation experience

**Priority Tasks**:
1. **Collapsible Tool Outputs**
   - Clean conversation view with expandable sections
   - Smart summarization of tool results
   - Performance optimization for long conversations

2. **Thread Persistence** 
   - Save and resume conversations
   - Thread management interface
   - Search/browse previous conversations

3. **File Change Tracking**
   - Track agent file modifications
   - Revert/undo functionality
   - Diff visualization

4. **Enhanced Conversation UI**
   - Better message formatting
   - Syntax highlighting in tool outputs
   - Progress indicators and streaming

**Success Metrics**:
- ✅ Long conversations remain performant
- ✅ Tool outputs are cleanly presented
- ✅ Users can easily revert unwanted changes
- ✅ Conversation history is persistent and searchable

### Phase 3: Differentiation (Week 4+)
**Goal**: Establish unique competitive advantages

**Priority Tasks**:
1. **Superior Model Selection**
   - Real-time model availability
   - Cost tracking and budgeting
   - Performance comparisons
   - Context window optimization

2. **Multi-Provider Excellence**
   - Seamless provider switching
   - Fallback mechanisms
   - Cost optimization across providers
   - Usage analytics

3. **Local Model Advantage**
   - Enhanced Ollama integration
   - Model management (install/update)
   - Hybrid local/cloud workflows
   - Privacy-focused features

4. **Developer Workflow Integration**
   - Git integration improvements
   - Project-specific agent memory
   - Custom tool development
   - Workflow automation

**Success Metrics**:
- ✅ Users prefer our model selection over competitors
- ✅ Multi-provider setup is seamless
- ✅ Local model integration feels native
- ✅ Developers adopt Aircher as primary coding agent

## Technical Architecture

### TUI Layout (Revised)
```
┌─ Aircher Agent ─────────────────────────────────────┐
│                                                     │
│ [Conversation Area - Expandable]                    │
│                                                     │
├─ TODO Tasks ────────────────────────────────────────┤
│ ☐ Task 1  ✓ Task 2  🔄 Task 3                      │
├─────────────────────────────────────────────────────┤
│ > Input Area - Auto-expanding                       │
└─────────────────────────────────────────────────────┘
  Status Bar: Model • Context • Cost • Tasks
```

### Key Components to Build
1. **TodoPanel** - Task tracking widget
2. **ConversationView** - Collapsible message renderer  
3. **ModelSelector** - Enhanced model picker with metadata
4. **ThreadManager** - Conversation persistence
5. **FileTracker** - Change tracking and revert system

## Success Metrics

### Technical KPIs
- Tool calling success rate: >95%
- TUI startup time: <100ms
- Memory usage: <100MB at steady state
- Zero compiler warnings maintained

### User Experience KPIs  
- Model selection completion time: <30 seconds
- TODO task management: <5 seconds per operation
- Conversation loading: <1 second for 100+ messages
- File change revert: <3 seconds

### Competitive KPIs
- Feature parity with Amp: 90% within 4 weeks
- Performance advantage: 2x faster than Electron alternatives
- Model selection satisfaction: >8/10 user rating
- Local model integration: Best-in-class Ollama support

## Risk Mitigation

### Technical Risks
- **Tool calling complexity**: Start with basic read/write/run, expand gradually
- **TUI performance**: Profile early, optimize rendering pipeline
- **Multi-provider reliability**: Implement robust fallback mechanisms

### Market Risks
- **Feature scope creep**: Focus on core differentiation first
- **User adoption**: Leverage existing semantic search user base
- **Competition speed**: Ship frequently, iterate based on feedback

## Timeline Summary

- **Week 1**: Tool calling + Model selection + TODO basics
- **Week 2**: Conversation UX + Thread persistence
- **Week 3**: File tracking + Performance optimization  
- **Week 4+**: Differentiation features + Market expansion

**Milestone**: Competitive with Amp core features by Week 3
**Goal**: Superior model selection experience becomes our calling card
