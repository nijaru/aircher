# Tool Failure Scenarios Test

## Current Tool Error Handling

The system has comprehensive tool error handling with multiple layers:

### ✅ Error Types Handled

1. **ToolError::InvalidParameters**: Malformed JSON parameters
2. **ToolError::NotFound**: Files/paths that don't exist
3. **ToolError::ExecutionFailed**: Runtime execution failures
4. **ToolError::PermissionDenied**: Blocked by permissions system
5. **Unknown tool errors**: When requested tool doesn't exist

### ✅ Tool-Specific Error Handling

#### File Operations (`ReadFileTool`, `WriteFileTool`, `EditFileTool`)
- **File not found**: `path.exists()` check with clear error message
- **Permission denied**: OS-level file permission errors
- **I/O failures**: Disk errors, corrupted files, etc.
- **Invalid path**: Malformed or dangerous paths

#### Command Execution (`RunCommandTool`)
- **Command not found**: Binary doesn't exist in PATH
- **Permission denied**: User permissions block execution
- **Timeout**: Commands exceeding configured timeout (default 30s)
- **Exit code failures**: Non-zero exit codes with stderr capture
- **Spawn failures**: Unable to start process

#### Code Analysis (`SearchCodeTool`, `FindDefinitionTool`)
- **Search backend unavailable**: Semantic search failures
- **Invalid query**: Malformed search patterns
- **Index not ready**: Search index not built yet

#### Git Operations (`GitStatusTool`)
- **Not a git repository**: Working directory isn't a git repo
- **Git command failures**: Git binary issues or repo corruption

### ✅ Error Display

Tool errors are formatted with clear icons and messages:
- **Success**: `✓ Tool executed successfully`
- **Failure**: `✗ tool_name: specific error message`

Examples:
- `✗ read_file: File not found: /nonexistent/file.txt`
- `✗ run_command: Command 'badcmd' was not approved`
- `✗ run_command: Command failed with exit code 1`

### Test Scenarios

#### Scenario 1: File Operation Failures

**Test 1.1**: Nonexistent file
```
Ask agent: "Please read the file /does/not/exist.txt"
Expected: ✗ read_file: File not found: /does/not/exist.txt
```

**Test 1.2**: Permission denied
```bash
# Create a file with restricted permissions
echo "secret" > /tmp/restricted.txt
chmod 000 /tmp/restricted.txt

# Ask agent: "Read /tmp/restricted.txt"
# Expected: ✗ read_file: Failed to read file: permission denied
```

**Test 1.3**: Invalid parameters
```
Ask agent to use tool with malformed JSON (this would be an LLM error)
Expected: ✗ read_file: Invalid parameters: missing field 'path'
```

#### Scenario 2: Command Execution Failures

**Test 2.1**: Command not found
```
Ask agent: "Run the command 'nonexistentcommand'"
Expected: ✗ run_command: Failed to spawn command: No such file or directory
```

**Test 2.2**: Command timeout
```
Ask agent: "Run 'sleep 60'" (with 30s timeout)
Expected: ✗ run_command: Command timed out after 30 seconds
```

**Test 2.3**: Command returns error
```
Ask agent: "Run 'ls /nonexistent'"
Expected: ✗ run_command: Command failed with exit code 2
```

**Test 2.4**: Permission denied by system
```
Ask agent: "Run the command 'rm -rf /'"
Expected: ✗ run_command: Command 'rm' was not approved
```

#### Scenario 3: Git Operation Failures

**Test 3.1**: Not a git repository
```bash
cd /tmp  # Non-git directory
# Ask agent: "Show me git status"
# Expected: ✗ git_status: Not a git repository
```

**Test 3.2**: Git command failure
```bash
# Corrupt git repository
rm -rf .git/refs
# Ask agent: "Show git status"
# Expected: ✗ git_status: git status failed: fatal: bad ref
```

### Validation Criteria

✅ **System should**:
- Show clear, actionable error messages with ✗ icons
- Continue agent conversation after tool failures
- Not crash or hang on tool errors
- Maintain streaming functionality during tool failures
- Provide recovery suggestions where possible

❌ **System should NOT**:
- Show raw error traces to users
- Stop processing other tools in batch operations
- Lose conversation state on tool failures
- Retry failed tools without user intervention

### Test Execution

```bash
# Run with debug logging to see tool execution details
RUST_LOG=debug cargo run

# Test various failure scenarios:
1. "Read the file /does/not/exist"
2. "Run the command 'fakecmd that does not exist'"
3. "Execute: sleep 100" (to test timeout)
4. "Show me git status" (in non-git directory)
5. "List files in /root" (permission denied)
```

### Expected Behavior Flow

```
User: "Read /nonexistent/file.txt"
Agent: "I'll read that file for you."
       🔧 Reading file: /nonexistent/file.txt
       ✗ read_file: File not found: /nonexistent/file.txt
       "I apologize, but the file /nonexistent/file.txt doesn't exist..."
```

## Conclusion

The tool failure handling system is **well-implemented** with:
- Comprehensive error categorization
- Clear user-friendly error messages
- Proper error recovery and continuation
- Stream-safe error handling
- Permission-based error protection

Testing should validate that all error paths work correctly and provide helpful guidance to users.