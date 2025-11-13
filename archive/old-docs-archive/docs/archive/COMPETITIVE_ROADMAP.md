# Aircher Competitive Development Roadmap

*Strategic plan to make Aircher the multi-modal model expert agent*

## Our Market Position (2025 Refined)

**The Multi-Modal Model Expert Agent** - serving developers who want:
- **Model Selection Transparency** (vs Amp's "always best models")
- **Multi-Modal Access** (Terminal TUI + Editor Integration via ACP)
- **Provider Flexibility** (Local Ollama + Cloud providers with cost transparency)
- **Performance Advantage** (Rust native vs Electron/Node.js alternatives)
- **Standards Compliance** (ACP compatibility for future-proofing)

## Competitive Analysis (2025 ACP Era)

### vs Amp
- **Amp's Strength**: Curated "always best" models, simplified experience
- **Our Advantage**: Model selection transparency + dual-mode access (TUI + ACP)
- **Strategic Edge**: Work in both terminal AND editor environments

### vs Claude Code
- **Their Strength**: Native Anthropic integration, ACP support in Zed
- **Our Advantage**: Multi-provider choice + ACP compatibility + terminal performance
- **Differentiation**: Model transparency while matching ACP standards

### vs OpenCode/Cursor
- **Their Strength**: Rich IDE integration
- **Our Advantage**: Standards-compliant ACP + terminal workflow + multi-provider
- **Future Position**: Work across editors (Zed, VS Code future, Neovim) vs vendor lock-in

### New Competitive Landscape: ACP-Native Agents
- **Industry Direction**: Agent Client Protocol becoming standard (Google + Zed + Anthropic)
- **Our Position**: First to combine model selection transparency WITH ACP compatibility
- **Unique Value**: Terminal performance + Editor integration + Provider choice

## Development Phases (Dual-Mode Architecture)

### Phase 1: Architecture Foundation (Week 1)
**Goal**: Refactor for dual-mode operation (TUI + ACP)

**✅ Completed Tasks**:
- ✅ Agent controller with 6 core tools
- ✅ TODO panel integrated into TUI
- ✅ Rich model selection with metadata
- ✅ Tool calling system functional

**Priority Tasks**:
1. **Shared Agent Core** (Architecture)
   - Extract agent logic from TUI coupling
   - Create mode-agnostic conversation management
   - Implement pluggable tool execution layer

2. **ACP Foundation** (Standards)
   - Add `agent-client-protocol` dependency
   - Implement basic `Agent` trait
   - Create JSON-RPC entry point

3. **Tool Layer Abstraction** (Flexibility)
   - Direct tools for TUI mode
   - Client-mediated tools for ACP mode
   - Unified tool interface

**Success Metrics**:
- ✅ Agent core works independently of UI
- ✅ Basic ACP agent responds to initialize/prompt
- ✅ Tool execution adapts to mode

### Phase 2: Dual-Mode Implementation (Week 2)
**Goal**: Working TUI and ACP modes

**Priority Tasks**:
1. **Entry Point Refactoring**
   - `--acp` flag for ACP mode
   - `--tui` (default) for terminal mode
   - Shared configuration system

2. **Session Management**
   - ACP-compatible session handling
   - State externalization
   - Multi-session support

3. **Tool System Integration**
   - File operations via ACP client requests
   - Command execution with permissions
   - Streaming tool output

4. **Zed Integration Testing**
   - Test with real Zed editor
   - Validate tool execution flow
   - Debug ACP protocol issues

**Success Metrics**:
- ✅ Works in Zed editor via ACP
- ✅ TUI mode unchanged experience
- ✅ Tools work in both modes
- ✅ Session persistence across modes

### Phase 3: Polish & Differentiation (Week 3-4)
**Goal**: Market-ready dual-mode agent

**Priority Tasks**:
1. **Enhanced ACP Features**
   - Rich tool metadata in ACP responses
   - Progress indication for long operations
   - Error handling and recovery

2. **Model Selection Integration**
   - Provider choice in ACP mode
   - Cost tracking across modes
   - Performance monitoring

3. **Advanced Capabilities**
   - Multi-file operations
   - Project context awareness
   - Git integration via ACP

4. **Documentation & Examples**
   - ACP integration guide
   - Zed setup instructions
   - Developer examples

**Success Metrics**:
- ✅ Competitive with Claude Code in Zed
- ✅ Unique model selection advantages clear
- ✅ Performance benefits demonstrated
- ✅ Community adoption starts

## Technical Architecture (Dual-Mode)

### Layered Architecture Design
```
┌─────────────────────────────────────────────────────────────┐
│                    User Interfaces                          │
│  ┌─────────────────┐              ┌─────────────────────────┐│
│  │   TUI Mode      │              │     ACP Mode            ││
│  │   (ratatui)     │              │  (JSON-RPC/stdio)      ││
│  │                 │              │                         ││
│  │ ┌─Conversation─┐│              │  ┌─Editor Integration─┐ ││
│  │ ├─TODO Panel──┤│              │  │  • File Operations  │ ││
│  │ ├─Input──────┤ │              │  │  • Tool Execution   │ ││
│  │ └─Status Bar─┘ │              │  │  • Session Mgmt     │ ││
│  └─────────────────┘              └─────────────────────────┘│
├─────────────────────────────────────────────────────────────┤
│              Aircher Agent Core                             │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  • Model Selection & Provider Management               ││
│  │  • Conversation Management                             ││
│  │  • Intelligence Engine                                 ││
│  │  • Session State Management                            ││
│  └─────────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────┤
│                Tool Execution Layer                         │
│  ┌─────────────────┐              ┌─────────────────────────┐│
│  │  Direct Tools   │              │  Client-Mediated Tools  ││
│  │  (TUI mode)     │              │     (ACP mode)          ││
│  │  • file_ops     │              │  • client.read_file()   ││
│  │  • run_command  │              │  • client.write_file()  ││
│  │  • search_code  │              │  • client.run_command() ││
│  └─────────────────┘              └─────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### Key Architectural Principles
1. **Shared Core Logic** - Model selection, conversation, intelligence same for both modes
2. **Pluggable Interfaces** - Tool execution adapts based on mode
3. **State Externalization** - Session management works locally and remotely
4. **Configuration-Driven** - Single codebase, runtime behavior selection

### Entry Points
```rust
// src/main.rs
match args.get(1) {
    Some("--acp") => acp_main().await,    // Agent Client Protocol mode
    _ => tui_main().await,                // Terminal User Interface mode
}
```

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
