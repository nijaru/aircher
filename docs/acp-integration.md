# ACP Integration Guide

**Status**: Week 6 Days 1-4 Complete - Enhanced with Session Management, Streaming, and Error Handling

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

### ✅ WEEK 6 ENHANCEMENTS (Days 2-4)

**Session Management** (Day 2 - 192 lines):
- HashMap-based session tracking with UUID session IDs ✅
- 30-minute idle timeout with automatic cleanup ✅
- Conversation history per session (stores all messages) ✅
- Methods: `create_session()`, `get_session()`, `add_message_to_session()`, `cleanup_expired_sessions()` ✅

**Streaming Support** (Day 3 - 143 lines):
- 5 notification types: Text, ToolStart, ToolProgress, ToolComplete, Thinking ✅
- Real-time feedback via JSON-RPC notifications ✅
- Arc<Mutex<Stdout>> for thread-safe concurrent output ✅
- Methods: `send_notification()`, `stream_text()`, `stream_tool_start/progress/complete()` ✅

**Error Handling** (Day 4 - 300 lines):
- 10 JSON-RPC error codes (5 standard + 5 custom) ✅
- ErrorContext system with user-friendly messages, retryability flags, suggestions ✅
- Retry logic with exponential backoff (100ms → 200ms → 400ms, max 3 attempts) ✅
- 5-minute operation timeout using tokio::time::timeout ✅
- Graceful degradation (log errors, continue processing) ✅

**Total Week 6 Additions**: +635 lines production code, +470 lines test code

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

### Streaming Notifications (NEW - Week 6 Day 3)

During agent prompt processing, the server sends real-time notifications to provide progress updates. These notifications don't have an `id` field (they're not responses to requests).

#### session/update Notification

**Format**:
```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "update": {
      "type": "text|tool_start|tool_progress|tool_complete|thinking",
      ... (type-specific fields)
    }
  }
}
```

#### Notification Types

**1. Text** - Streaming text output:
```json
{
  "type": "text",
  "content": "Here is the analysis..."
}
```

**2. ToolStart** - Tool execution begins:
```json
{
  "type": "tool_start",
  "tool_name": "read_file",
  "parameters": {
    "path": "src/main.rs"
  }
}
```

**3. ToolProgress** - Tool execution progress:
```json
{
  "type": "tool_progress",
  "tool_name": "read_file",
  "message": "Reading file src/main.rs (1.2 MB)..."
}
```

**4. ToolComplete** - Tool execution finished:
```json
{
  "type": "tool_complete",
  "tool_name": "read_file",
  "result": {
    "content": "...",
    "lines": 450
  }
}
```

**5. Thinking** - Agent reasoning output:
```json
{
  "type": "thinking",
  "content": "I need to check the function definition first..."
}
```

### Error Responses (NEW - Week 6 Day 4)

All errors follow JSON-RPC 2.0 error response format with enhanced context:

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "Operation failed",
    "data": {
      "user_message": "The file you requested could not be found",
      "retryable": true,
      "suggestion": "Check that the file path is correct and try again"
    }
  },
  "id": 3
}
```

#### Error Codes

**Standard JSON-RPC Errors**:
- `-32700`: Parse error (invalid JSON)
- `-32600`: Invalid request (malformed structure)
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

**Custom Application Errors**:
- `-32000`: Server error (general)
- `-32001`: Session not found
- `-32002`: Session expired (idle > 30 minutes)
- `-32003`: Operation timeout (> 5 minutes)
- `-32004`: Rate limit exceeded

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

## Week 6 Progress

### Day 1 ✅: Implementation Review
- Reviewed existing ACP implementation
- Verified all core methods implemented
- Created comprehensive documentation
- **Discovery**: ACP already 90% complete (saved 4-5 days!)

### Day 2 ✅: Session Management Enhancement
- HashMap-based session tracking with UUID session IDs
- Conversation history per session (all messages with timestamps)
- 30-minute idle timeout with automatic cleanup
- **Code**: +192 lines in `src/server/stdio.rs`

### Day 3 ✅: Streaming Support
- 5 notification types (Text, ToolStart, ToolProgress, ToolComplete, Thinking)
- Real-time feedback via JSON-RPC notifications
- Thread-safe stdout handling with Arc<Mutex<Stdout>>
- **Code**: +143 lines

### Day 4 ✅: Error Handling & Recovery
- 10 JSON-RPC error codes (5 standard + 5 custom)
- ErrorContext system with user messages, retryability, suggestions
- Retry logic with exponential backoff (100ms, 200ms, 400ms)
- 5-minute operation timeout with graceful degradation
- **Code**: +300 lines
- **Tests**: +470 lines (`tests/acp_week6_features_test.rs`)

### Day 5-7 (Current): Testing & Documentation
- Manual ACP protocol testing
- Performance benchmarking (latency, throughput)
- Update documentation (this file)
- Attempt Zed editor integration

## Known Limitations

1. **No session persistence**: Sessions lost on restart (tracked in memory only)
2. ~~**No streaming**: Only final responses returned~~ ✅ **RESOLVED** (Week 6 Day 3)
3. ~~**Basic error handling**: Could be more robust~~ ✅ **RESOLVED** (Week 6 Day 4)
4. **Single model hardcoded**: Uses Ollama gpt-oss only
5. **No authentication**: Stub implementation only
6. **Old binary test files**: Some test binaries fail to compile (non-blocking)

## Future Enhancements

1. **Session Persistence** (Week 7+):
   - Save session state to DuckDB episodic memory
   - Resume conversations after restart
   - Export/import sessions

2. ~~**Streaming Responses**~~ ✅ **COMPLETE** (Week 6 Day 3):
   - ~~Stream token-by-token output~~
   - ~~Real-time tool execution updates~~
   - ~~Progress indicators~~

3. **Multi-Model Support** (Week 7+):
   - Let frontend specify model via initialize params
   - Per-session model selection
   - Automatic fallback on provider errors

4. **Advanced Features** (Post-Week 10):
   - Image/audio support (when ACP protocol adds support)
   - Integration with episodic memory for cross-session learning
   - Collaborative editing sessions (multi-user)

5. **Performance Optimizations** (Week 6 Day 5-7):
   - Benchmark latency and throughput
   - Optimize session cleanup (background task)
   - Profile memory usage under load

## References

- [Agent Client Protocol Specification](https://agentclientprotocol.com)
- [Zed ACP Integration](https://github.com/zed-industries/zed/tree/main/crates/agent_client_protocol)
- [Aircher Agent Architecture](./architecture/agent-first-architecture.md)

---

**Last Updated**: 2025-10-27 (Week 6 Days 1-4 Complete)

## Summary of Week 6 Achievements

- **Day 1**: Discovered ACP already 90% implemented (major timeline win!)
- **Day 2**: Added session management (+192 lines)
- **Day 3**: Implemented streaming notifications (+143 lines)
- **Day 4**: Enhanced error handling with retry logic (+300 lines)
- **Total**: +635 lines production code, +470 lines test code
- **Status**: ACP protocol ready for production testing with Zed editor
