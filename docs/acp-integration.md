# ACP Integration Guide

**Status**: Week 6 Day 1 - Implementation Review Complete

## Overview

Aircher implements the Agent Client Protocol (ACP) for seamless integration with multiple editors and frontends. The ACP server is already fully implemented and functional.

## Current Implementation Status

### ✅ COMPLETE

**Core ACP Server** (`src/server/`):
- `stdio.rs`: JSON-RPC over stdin/stdout transport ✅
- `mod.rs`: Server module structure ✅

**ACP Agent Implementation** (`src/agent/core.rs` lines 1437-1545):
- `initialize()`: Agent capabilities negotiation ✅
- `new_session()`: Session creation with UUID ✅
- `prompt()`: User message processing ✅
- `authenticate()`: Authentication stub ✅
- `load_session()`: Returns not implemented ✅
- `cancel()`: Cancel notification handler ✅

**CLI Integration** (`src/main.rs`):
- `--acp` flag triggers ACP mode ✅
- Proper logging to stderr (doesn't interfere with JSON-RPC) ✅
- Mode detection and routing ✅

**Dependencies** (`Cargo.toml`):
- `agent-client-protocol = "0.1.1"` ✅
- Feature flag: `acp = ["agent-client-protocol"]` ✅

### 🔄 NEEDS ENHANCEMENT

**Session Management**:
- Currently creates new session ID but doesn't track sessions
- No session state persistence
- No conversation history per session

**Streaming Support**:
- Protocol supports streaming but not implemented
- Currently returns only final response

**Tool Execution Visibility**:
- Tools execute but intermediate results not streamed
- No progress updates during long operations

**Error Recovery**:
- Basic error handling present
- Could add retry logic and graceful degradation

## How to Run ACP Server

### Start the Server

```bash
# Build with ACP support
cargo build --features acp --release

# Run in ACP mode
./target/release/aircher --acp
```

### Test with Manual JSON-RPC

```bash
# In one terminal, start the server
./target/release/aircher --acp

# In another terminal, send JSON-RPC messages
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocol_version":"1.0","auth_methods":[]},"id":1}' | ./target/release/aircher --acp
```

### Integration with Zed Editor

1. **Install Zed** (if not already installed)

2. **Configure Zed to use Aircher**:

   Add to Zed settings (`.config/zed/settings.json`):

   ```json
   {
     "agents": {
       "aircher": {
         "command": "/path/to/aircher/target/release/aircher",
         "args": ["--acp"]
       }
     }
   }
   ```

3. **Use in Zed**:
   - Open command palette (`Cmd+Shift+P` on Mac, `Ctrl+Shift+P` on Linux)
   - Type "Agent" and select agent commands
   - Aircher will process requests via ACP

## Protocol Details

### Message Format

All messages follow JSON-RPC 2.0 format over stdio:

```json
{
  "jsonrpc": "2.0",
  "method": "method_name",
  "params": { ... },
  "id": 1
}
```

### Supported Methods

#### 1. initialize

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "protocol_version": "1.0",
    "auth_methods": []
  },
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "protocol_version": "1.0",
    "agent_capabilities": {
      "load_session": false,
      "prompt_capabilities": {
        "image": false,
        "audio": false,
        "embedded_context": true
      }
    },
    "auth_methods": []
  },
  "id": 1
}
```

#### 2. session/new

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "session/new",
  "params": {},
  "id": 2
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "session_id": "550e8400-e29b-41d4-a716-446655440000"
  },
  "id": 2
}
```

#### 3. session/prompt

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "session/prompt",
  "params": {
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "prompt": [
      {
        "type": "text",
        "text": "Explain this code"
      }
    ]
  },
  "id": 3
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "stop_reason": "end_turn"
  },
  "id": 3
}
```

#### 4. authenticate (optional)

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "authenticate",
  "params": {
    "method": "api_key",
    "credentials": { ... }
  },
  "id": 4
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {},
  "id": 4
}
```

#### 5. cancel (notification)

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "cancel",
  "params": {
    "session_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

No response (notification).

## Architecture

```
┌─────────────────────────────────────────────────┐
│              Editor/Frontend                     │
│        (Zed, Neovim, Emacs, etc.)               │
└─────────────────────────────────────────────────┘
                    │
                    │ JSON-RPC over stdio
                    ▼
┌─────────────────────────────────────────────────┐
│            ACP Server (Rust)                     │
│         src/server/stdio.rs                      │
├─────────────────────────────────────────────────┤
│  • Reads JSON-RPC from stdin                    │
│  • Writes responses to stdout                   │
│  • Logs to stderr (doesn't interfere)           │
└─────────────────────────────────────────────────┘
                    │
                    │ Agent trait calls
                    ▼
┌─────────────────────────────────────────────────┐
│         Agent (implements AcpAgent)              │
│         src/agent/core.rs                        │
├─────────────────────────────────────────────────┤
│  • Intelligence Engine                           │
│  • Memory Systems (episodic, knowledge, working)│
│  • Tool Registry                                 │
│  • LLM Providers                                 │
└─────────────────────────────────────────────────┘
```

## Testing

### Unit Tests

```bash
cargo test --features acp acp_integration_test
```

### Manual Testing

1. **Start server**:
   ```bash
   RUST_LOG=aircher=debug cargo run --features acp -- --acp
   ```

2. **Send test message** (in another terminal):
   ```bash
   echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocol_version":"1.0","auth_methods":[]},"id":1}' | nc localhost 8080
   ```

3. **Expected output**:
   - Server logs to stderr
   - JSON response on stdout

### Integration Testing with Zed

1. Configure Zed to use local Aircher build (debug mode):
   ```json
   {
     "agents": {
       "aircher-dev": {
         "command": "/path/to/aircher/target/debug/aircher",
         "args": ["--acp"],
         "env": {
           "RUST_LOG": "aircher=debug"
         }
       }
     }
   }
   ```

2. Use agent in Zed and check logs:
   ```bash
   tail -f ~/.local/state/zed/logs/Zed.log
   ```

## Week 6 Plan

### Day 1 ✅: Implementation Review
- Reviewed existing ACP implementation
- Verified all core methods implemented
- Identified enhancement opportunities

### Day 2: Session Management Enhancement
- Add session state tracking
- Implement conversation history per session
- Session cleanup and timeout

### Day 3: Streaming Support
- Implement streaming responses
- Progress updates during tool execution
- Real-time feedback to editor

### Day 4: Error Handling & Recovery
- Retry logic for failed operations
- Graceful degradation
- Better error messages to user

### Day 5-7: Testing & Documentation
- End-to-end testing with Zed
- Performance benchmarking
- User documentation
- Video demo

## Known Limitations

1. **No session persistence**: Sessions lost on restart
2. **No streaming**: Only final responses returned
3. **Basic error handling**: Could be more robust
4. **Single model hardcoded**: Uses Ollama gpt-oss only
5. **No authentication**: Stub implementation only

## Future Enhancements

1. **Session Persistence**:
   - Save session state to DuckDB
   - Resume conversations after restart
   - Export/import sessions

2. **Streaming Responses**:
   - Stream token-by-token output
   - Real-time tool execution updates
   - Progress indicators

3. **Multi-Model Support**:
   - Let frontend specify model
   - Per-session model selection
   - Automatic fallback on errors

4. **Advanced Features**:
   - Image/audio support (when protocol supports)
   - Multi-turn conversations with context
   - Collaborative editing sessions

## References

- [Agent Client Protocol Specification](https://agentclientprotocol.com)
- [Zed ACP Integration](https://github.com/zed-industries/zed/tree/main/crates/agent_client_protocol)
- [Aircher Agent Architecture](./architecture/agent-first-architecture.md)

---

**Last Updated**: 2025-10-27 (Week 6 Day 1)
