# Ollama Testing Guide for Aircher

## Current Status

### ✅ Working Features
- **AI Agent with Tool Use**: File operations, command execution
- **Permission System**: Modal blocks execution, saves approved commands
- **TUI Features**: Autocomplete, scrolling, multi-line input, session management
- **Real-time Streaming**: Agent responses stream incrementally with tool status updates
- **Error Recovery**: Graceful fallbacks and network error handling
- **Model Selection**: /model command to switch between models
- **Selected Model Used**: Agent now uses the model selected in TUI
- **Cost Tracking**: Shows cost for API providers (free for Ollama)
- **Clean Tool Display**: Tool executions shown as pretty status messages with icons
- **Token Tracking**: Real-time token counts during streaming

### ⚠️ Known Limitations
1. **Network Recovery**: Network interruption recovery needs testing
2. **Tool Failure Handling**: Edge cases in tool failures need validation

### 🔧 Pending Features
- Code syntax highlighting in messages
- /compact command for summarization  
- Discord Rich Presence
- F1/F2 keyboard shortcuts
- Terminal title bar updates

## Setup Instructions

1. **Configure Ollama Provider**
   ```bash
   # Edit ~/.aircher/config.toml or .aircher/config.toml
   ```
   
   Add your Ollama configuration:
   ```toml
   [[providers]]
   name = "ollama"
   endpoint = "http://your-fedora-pc.tailscale:11434"  # Replace with your Ollama URL
   model = "llama3.3"  # or your preferred model
   ```

2. **Start Aircher**
   ```bash
   cargo run
   ```

3. **Select Ollama Provider**
   - Type `/model` and select your Ollama model
   - Or start with: `cargo run -- --provider ollama`

## Testing AI Agent Features

### Basic File Operations
These work without permission prompts:

1. **Read a file**
   ```
   Please read the README.md file
   ```

2. **List directory contents**
   ```
   Show me all files in the src directory
   ```

3. **Search for code**
   ```
   Find all functions that handle permissions
   ```

### Command Execution
These will show the permission modal:

1. **Safe pre-approved commands**
   ```
   Run 'ls -la' to show all files
   ```

2. **Commands needing approval**
   ```
   Run 'cargo test' to verify the tests pass
   ```
   
   When the modal appears:
   - **Enter** → Execute once
   - **Tab** → Navigate options
   - **Y** → Yes (execute once)
   - **S** → Yes & approve similar 
   - **N/Esc** → No (deny)

3. **Test pattern approval**
   ```
   Run 'npm install'
   ```
   If you choose "Yes & similar", future npm commands will be auto-approved.

## Tool Call Formats

Aircher's agent supports multiple tool call formats. Test with different models:

### XML Style (Most reliable)
```
I'll read the file for you.
<tool_use>
<tool>read_file</tool><params>{"path": "package.json"}</params>
</tool_use>
```

### Function Style
```
Let me check: read_file({"path": "Cargo.toml"})
```

### JSON Style
```
{"tool": "list_files", "params": {"path": "src", "recursive": false}}
```

## Troubleshooting

### Model Not Responding with Tools
Some models may not understand the tool format. Try:
1. Use a more capable model (llama3.3, mixtral, etc.)
2. Test with explicit prompts: "Use the read_file tool to read README.md"

### Permission Modal Not Appearing
- Check `.aircher/permissions.toml` - command might be pre-approved
- Ensure the agent is enabled (not in fallback mode)

### Connection Issues
```bash
# Test Ollama connection
curl http://your-ollama-url:11434/api/tags
```

### Streaming Issues
If streaming appears broken:
1. Check provider connection - some providers may not support streaming
2. Look for error messages in the UI
3. Check logs with `RUST_LOG=debug` for streaming details

## Testing Streaming Features

The agent now supports real-time streaming responses. Test these scenarios:

1. **Simple streaming response**
   ```
   Write a detailed explanation of how Rust ownership works
   ```
   - Watch for incremental text appearing
   - Token count should update in real-time

2. **Streaming with tool use**
   ```
   Read the main.rs file and explain how it works
   ```
   - Tool status should appear: "🔧 Reading file: src/main.rs"
   - Then completion: "✓ Successfully read src/main.rs"
   - Followed by streaming explanation

3. **Multiple tool calls**
   ```
   List all files in src/, then read the mod.rs file and tell me about the project structure
   ```
   - Should see multiple tool status updates
   - Each tool execution shows progress indicators

## Example Testing Session

1. **Start conversation**
   ```
   Hi! Can you help me understand this codebase?
   ```

2. **Test file reading**
   ```
   Please read the Cargo.toml file and tell me about this project
   ```

3. **Test directory listing**
   ```
   Show me what's in the src directory
   ```

4. **Test command execution**
   ```
   Run 'echo Hello from Aircher' to test command execution
   ```

5. **Test complex task**
   ```
   Find all TODO comments in the codebase and list them
   ```

## Performance Notes

- First message may be slower as agent initializes
- Large file operations work but may take time
- Ollama models vary in tool use capability
- Local models are faster but may be less accurate with tools

## Debugging

Enable debug logging:
```bash
RUST_LOG=debug cargo run
```

Check agent tool parsing:
- Look for "Executing tool" messages in logs
- Tool executions show as clean status messages with icons (🔧 Running, ✓ Complete)
- Tool results are formatted as user-friendly messages

## Contributing

If you find issues:
1. Note the model you're using
2. Save the exact prompt that caused issues
3. Check `~/.aircher/aircher_errors.log`
4. Report at: https://github.com/nijaru/aircher/issues