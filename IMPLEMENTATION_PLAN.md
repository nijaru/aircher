# Aircher Implementation Plan: Agent-First Architecture

**Last Updated**: 2025-01-10  
**Goal**: Refactor Aircher to use a single ACP-native agent with multiple frontends

## Executive Summary

**Current State**: Duplicated agent logic (TUI has AgentController, ACP has separate implementation)  
**Target State**: Single UnifiedAgent implementing ACP trait, with TUI as a client  
**Timeline**: 2 weeks for core refactor, 1 week for testing/polish

## Architecture Vision

```
┌─────────────────────────────────────────────┐
│             User Interfaces                 │
├──────────┬──────────┬──────────┬───────────┤
│   TUI    │   Zed    │ VS Code  │    CLI    │
│(ratatui) │  (ACP)   │  (ACP)   │ (direct)  │
└────┬─────┴────┬─────┴────┬─────┴─────┬─────┘
     │          │          │           │
     ▼          ▼          ▼           ▼
┌─────────────────────────────────────────────┐
│           Client Abstraction Layer          │
├─────────────────────────────────────────────┤
│ LocalClient │      RemoteClient (ACP)       │
│  (direct)   │     (JSON-RPC/stdio)          │
└──────┬──────┴───────────┬───────────────────┘
       │                  │
       ▼                  ▼
┌─────────────────────────────────────────────┐
│         UnifiedAgent (ACP-native)           │
├─────────────────────────────────────────────┤
│ • Implements agent_client_protocol::Agent   │
│ • Tool execution & management               │
│ • Intelligence engine                       │
│ • Context & conversation management         │
│ • Provider abstraction                      │
└─────────────────────────────────────────────┘
```

## Implementation Phases

### Phase 0: Planning & Setup (Day 1) ✅
- [x] Document agent-first architecture
- [x] Create proof of concept
- [x] Write implementation plan
- [ ] Set up feature flags for gradual migration

### Phase 1: Core Agent Extraction (Days 2-3)

#### 1.1 Create UnifiedAgent Structure
```rust
// src/agent/unified.rs
pub struct UnifiedAgent {
    tools: ToolRegistry,
    intelligence: IntelligenceEngine,
    providers: ProviderManager,
    conversations: HashMap<SessionId, Conversation>,
}
```

**Tasks**:
- [ ] Create src/agent/unified.rs
- [ ] Move tool registry from controller
- [ ] Move intelligence engine integration
- [ ] Move provider management
- [ ] Implement session/conversation management

#### 1.2 Implement ACP Agent Trait
```rust
#[async_trait]
impl agent_client_protocol::Agent for UnifiedAgent {
    async fn initialize(&self, req: InitializeRequest) -> Result<InitializeResponse, Error>;
    async fn new_session(&self, req: NewSessionRequest) -> Result<NewSessionResponse, Error>;
    async fn prompt(&self, req: PromptRequest) -> Result<PromptResponse, Error>;
    // ... other required methods
}
```

**Tasks**:
- [ ] Implement initialize with capability reporting
- [ ] Implement session creation/management
- [ ] Implement prompt processing (core logic)
- [ ] Implement tool calling through ACP
- [ ] Add proper error handling

### Phase 2: Client Abstraction Layer (Days 4-5)

#### 2.1 Define Client Trait
```rust
// src/client/mod.rs
#[async_trait]
pub trait AgentClient: Send + Sync {
    async fn initialize(&self) -> Result<AgentInfo>;
    async fn create_session(&self) -> Result<SessionId>;
    async fn send_prompt(&self, session: SessionId, message: String) -> Result<Response>;
    async fn execute_tool(&self, tool_call: ToolCall) -> Result<ToolResult>;
}
```

#### 2.2 Implement LocalClient
```rust
// src/client/local.rs
pub struct LocalClient {
    agent: Arc<UnifiedAgent>,
}

impl LocalClient {
    // Direct function calls to agent, no serialization
    // Optimal for TUI performance
}
```

**Tasks**:
- [ ] Create client trait definition
- [ ] Implement LocalClient with direct calls
- [ ] Add streaming support for TUI
- [ ] Handle tool execution efficiently

#### 2.3 Implement RemoteClient (Future)
```rust
// src/client/remote.rs
pub struct RemoteClient {
    connection: AgentConnection,
}
```

**Tasks**:
- [ ] Design JSON-RPC transport
- [ ] Implement serialization/deserialization
- [ ] Add connection management
- [ ] Handle network errors gracefully

### Phase 3: TUI Refactor (Days 6-8)

#### 3.1 Update TuiManager
**Current State**:
```rust
// src/ui/mod.rs - Line 279
pub struct TuiManager {
    agent_controller: Option<AgentController>, // OLD
}
```

**Target State**:
```rust
pub struct TuiManager {
    agent_client: Arc<dyn AgentClient>,       // NEW
    session_id: SessionId,
}
```

**Tasks**:
- [ ] Replace AgentController with AgentClient
- [ ] Update message sending to use client
- [ ] Refactor tool display for client responses
- [ ] Maintain streaming functionality

#### 3.2 Update Message Flow
**Current**:
```
User Input → TuiManager → AgentController → Tools → Response
```

**New**:
```
User Input → TuiManager → LocalClient → UnifiedAgent → Tools → Response
```

**Tasks**:
- [ ] Update handle_message (line 3797)
- [ ] Update send_message_with_agent (line 4009)
- [ ] Preserve streaming updates
- [ ] Maintain tool status display

### Phase 4: ACP Server Implementation (Days 9-10)

#### 4.1 Create Server Wrapper
```rust
// src/server/mod.rs
pub struct AcpServer {
    agent: Arc<UnifiedAgent>,
}

impl AcpServer {
    pub async fn run_stdio(self) {
        // Handle JSON-RPC over stdin/stdout
    }
}
```

**Tasks**:
- [ ] Implement stdio transport
- [ ] Add JSON-RPC message handling
- [ ] Connect to UnifiedAgent
- [ ] Test with Zed editor

### Phase 5: Testing & Migration (Days 11-14)

#### 5.1 Test Suite
- [ ] Unit tests for UnifiedAgent
- [ ] Integration tests for LocalClient
- [ ] End-to-end tests for TUI flow
- [ ] ACP compliance tests
- [ ] Performance benchmarks

#### 5.2 Migration Strategy
- [ ] Feature flag for new architecture
- [ ] Gradual rollout (opt-in initially)
- [ ] A/B testing between old and new
- [ ] Performance comparison
- [ ] User feedback collection

### Phase 6: Cleanup & Polish (Days 15+)

#### 6.1 Remove Old Code
- [ ] Delete old AgentController
- [ ] Remove duplicate ACP implementation
- [ ] Clean up unused dependencies
- [ ] Update documentation

#### 6.2 Optimization
- [ ] Profile TUI performance
- [ ] Optimize LocalClient calls
- [ ] Add caching where beneficial
- [ ] Reduce memory usage

## File Structure (Target)

```
src/
├── agent/
│   ├── mod.rs              # Public API
│   ├── unified.rs          # UnifiedAgent implementation
│   ├── tools/              # Tool implementations
│   ├── intelligence/       # Intelligence engine
│   └── conversation.rs     # Conversation management
│
├── client/
│   ├── mod.rs              # Client trait
│   ├── local.rs            # LocalClient for TUI
│   └── remote.rs           # RemoteClient for network
│
├── server/
│   ├── mod.rs              # ACP server
│   └── stdio.rs            # Stdio transport
│
├── ui/                     # TUI (uses client)
│   ├── mod.rs              # Main TUI loop
│   └── ...                 # UI components
│
└── main.rs                 # Entry point routing
```

## Success Criteria

### Functional Requirements
- [x] Single agent implementation serves all frontends
- [ ] TUI works identically to current version
- [ ] ACP mode passes compliance tests
- [ ] Tool execution works in all modes
- [ ] Streaming works in TUI mode

### Performance Requirements
- [ ] TUI response time < 100ms (local client)
- [ ] No memory leaks in long sessions
- [ ] Tool execution < 500ms overhead
- [ ] Startup time < 1 second

### Code Quality
- [ ] Zero duplicate agent logic
- [ ] Clear separation of concerns
- [ ] Comprehensive test coverage (>80%)
- [ ] All compiler warnings resolved

## Risk Mitigation

### Risk: Breaking TUI During Refactor
**Mitigation**: Use feature flags, maintain old code until new is proven

### Risk: ACP Compliance Issues
**Mitigation**: Test early with Zed, use ACP test suite

### Risk: Performance Regression
**Mitigation**: Benchmark before/after, profile critical paths

### Risk: User Disruption
**Mitigation**: Gradual rollout, extensive testing, quick rollback plan

## Implementation Order

1. **Start with UnifiedAgent** - Core logic first
2. **Add LocalClient** - Maintain TUI functionality
3. **Refactor TUI gradually** - Behind feature flag
4. **Add ACP server last** - After core is stable

## Commit Strategy

- Commit after each subtask completion
- Use conventional commits (feat:, fix:, refactor:)
- Include tests with implementation
- Document breaking changes clearly

## Next Immediate Steps

1. Create src/agent/unified.rs with basic structure
2. Move core logic from AgentController
3. Implement minimal ACP Agent trait
4. Create LocalClient wrapper
5. Test with simple TUI integration

This plan provides a clear path from our current duplicated architecture to a clean, unified agent-first design that serves both TUI and ACP modes efficiently.