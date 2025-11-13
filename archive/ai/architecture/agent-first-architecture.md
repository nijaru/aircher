# Agent-First Architecture

*Unified ACP-compatible agent with multiple frontends*

## Current Problem

We currently have duplicated agent logic:
- TUI directly uses tools and agent controller
- ACP mode reimplements agent behavior
- Two codepaths = inconsistency and maintenance burden

## Proposed Architecture

### Core Principle: One Agent, Multiple Frontends

```
┌─────────────────────────────────────────────────┐
│                  Users                           │
├────────────┬──────────────┬─────────────────────┤
│    TUI     │     Zed      │    VS Code          │
│ (ratatui)  │   (ACP)      │     (ACP)           │
└────────────┴──────────────┴─────────────────────┘
       │             │                │
       ▼             ▼                ▼
┌─────────────────────────────────────────────────┐
│            Frontend Layer                        │
├─────────────────────────────────────────────────┤
│  Local      │        JSON-RPC over stdio         │
│  Client     │         (ACP Protocol)             │
└─────────────┴────────────────────────────────────┘
       │                    │
       ▼                    ▼
┌─────────────────────────────────────────────────┐
│         Unified Agent Core (ACP-native)          │
├─────────────────────────────────────────────────┤
│  • Implements ACP Agent trait                    │
│  • Tool execution                                │
│  • Intelligence engine                           │
│  • Context management                            │
│  • Model provider abstraction                    │
└─────────────────────────────────────────────────┘
```

## Refactored Directory Structure

```
src/
├── agent/                    # Core ACP-compatible agent
│   ├── mod.rs               # Main agent implementation (ACP Agent trait)
│   ├── tools/               # Tool implementations
│   ├── intelligence/        # Intelligence engine
│   ├── context/            # Context and memory management
│   └── providers/          # LLM provider abstraction
│
├── client/                   # Client abstraction layer
│   ├── mod.rs              # Client trait definition
│   ├── local.rs            # Direct in-process client (for TUI)
│   └── remote.rs           # JSON-RPC client (for testing)
│
├── server/                   # ACP server implementation
│   ├── mod.rs              # ACP server setup
│   └── stdio.rs            # Stdio transport for editors
│
├── tui/                      # Terminal UI (acts as client)
│   ├── mod.rs              # TUI main loop
│   ├── ui/                 # UI components
│   └── app.rs              # App state (uses Client trait)
│
└── shared/                   # Shared utilities
    ├── config/             # Configuration management
    ├── auth/               # Authentication
    └── storage/            # Database and file storage
```

## Key Design Decisions

### 1. Agent is ACP-Native
The core agent directly implements the ACP `Agent` trait. This ensures:
- Full ACP compliance by design
- No translation layer needed for editor integration
- Single source of truth for agent behavior

### 2. TUI as Local Client
The TUI acts as an ACP client but calls the agent directly (no JSON-RPC):
```rust
// Instead of duplicating logic, TUI uses agent like any ACP client
impl TuiApp {
    async fn send_message(&mut self, message: String) {
        // Acts like an ACP client
        let request = PromptRequest {
            session_id: self.session_id,
            prompt: vec![ContentBlock::Text(message)],
        };

        // But calls agent directly (no JSON-RPC overhead)
        let response = self.agent.prompt(request).await?;
        self.display_response(response);
    }
}
```

### 3. Unified Tool System
Tools work identically regardless of frontend:
```rust
pub trait Tool: Send + Sync {
    async fn execute(&self, params: Value) -> Result<ToolOutput>;
}

// Same tool works in TUI and ACP modes
pub struct ReadFileTool;

impl Tool for ReadFileTool {
    async fn execute(&self, params: Value) -> Result<ToolOutput> {
        // Implementation once, used everywhere
    }
}
```

### 4. Client Abstraction
A simple trait allows different connection methods:
```rust
#[async_trait]
pub trait AgentClient: Send + Sync {
    async fn initialize(&self, request: InitializeRequest) -> Result<InitializeResponse>;
    async fn new_session(&self, request: NewSessionRequest) -> Result<NewSessionResponse>;
    async fn prompt(&self, request: PromptRequest) -> Result<PromptResponse>;
    // ... other ACP methods
}

// Direct in-process client for TUI
pub struct LocalClient {
    agent: Arc<Agent>,
}

// JSON-RPC client for remote connections (testing, distributed)
pub struct RemoteClient {
    connection: AgentConnection,
}
```

## Migration Path

### Phase 1: Extract Core Agent
1. Move agent logic to `src/agent/` as ACP-native implementation
2. Ensure it fully implements ACP Agent trait
3. Remove duplicate logic from TUI

### Phase 2: Create Client Abstraction
1. Define AgentClient trait
2. Implement LocalClient for TUI
3. Update TUI to use AgentClient instead of direct tool calls

### Phase 3: Integrate ACP Server
1. Create stdio server wrapper around agent
2. Test with Zed/other ACP-compatible editors
3. Ensure behavior is identical between TUI and ACP

### Phase 4: Optimize
1. Add caching layer for TUI (avoid re-parsing)
2. Implement streaming for better UX
3. Add telemetry to understand usage patterns

## Benefits

### Consistency
- Same agent behavior in all contexts
- No divergence between TUI and editor modes
- Single implementation to maintain

### Testability
- Test agent once, works everywhere
- Can test TUI against same test suite as ACP
- Mock client for testing agent in isolation

### Extensibility
- Easy to add new frontends (web UI, CLI)
- New tools automatically available everywhere
- Intelligence improvements benefit all users

### Performance
- TUI avoids JSON-RPC overhead (direct calls)
- Shared caching and state management
- Single process for TUI mode

## Example: Unified Message Flow

```rust
// TUI Mode
tui_input -> TuiApp -> LocalClient -> Agent -> Tool -> Response -> TUI display

// ACP Mode (Zed)
editor_input -> JSON-RPC -> Server -> Agent -> Tool -> Response -> JSON-RPC -> editor

// Same Agent, same Tools, same behavior!
```

## Implementation Priority

1. **Must Have** (Week 1)
   - Core agent with ACP trait
   - Basic LocalClient for TUI
   - Tool system working in both modes

2. **Should Have** (Week 2)
   - Full ACP compliance
   - Streaming support
   - Session management

3. **Nice to Have** (Week 3+)
   - Remote client for distributed mode
   - Web UI frontend
   - Performance optimizations

## Technical Considerations

### Threading Model
- Agent runs in its own async task
- TUI has dedicated render thread
- Tools may spawn background tasks

### State Management
- Agent owns conversation state
- TUI owns display state
- Clean separation of concerns

### Error Handling
- Agent returns ACP-compliant errors
- TUI translates to user-friendly messages
- Graceful degradation on tool failures

## Conclusion

By making the agent ACP-native and treating the TUI as just another client, we:
- Eliminate code duplication
- Ensure consistency across all interfaces
- Simplify maintenance and testing
- Position Aircher as a true multi-modal agent

The architecture naturally supports our competitive advantage: being the best at both terminal and editor integration, not just one or the other.
