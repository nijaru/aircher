# Aircher AI Agent Tool Use Demo

## Overview
The AI agent in Aircher can now execute tools on your behalf, including:
- Reading and writing files
- Running shell commands
- Searching code
- Finding definitions

## Permissions System
When the AI wants to execute a command, you'll see a permission modal with three options:
- **Yes**: Allow this specific command once
- **Yes & approve similar**: Allow this command and similar ones in the future
- **No**: Deny the command

## Demo Instructions

1. Start Aircher:
   ```bash
   cargo run
   ```

2. Try these example prompts:

### File Operations (No permission needed)
- "Read the README.md file"
- "Show me what's in the src directory"
- "Search for 'permission' in the codebase"

### Safe Commands (Pre-approved)
- "List the current directory contents"
- "Show me the current working directory"
- "Search for TODO comments in the code"

### Commands Requiring Permission
- "Run the tests with cargo test"
- "Build the project"
- "Check the git status"

## How It Works

1. When you send a message, the AI agent analyzes it
2. If it needs to use tools, it generates tool calls in a structured format
3. The tool execution is intercepted by the permissions system
4. For commands, a modal appears asking for your approval
5. Your choices are remembered for the session

## Security Features

- All commands require explicit approval (except pre-approved safe ones)
- File operations are restricted to the current project directory
- Approved commands are saved in `.aircher/permissions.toml`
- You can approve patterns (e.g., all `npm` commands) with "Yes & similar"

## Testing the Integration

To fully test the system:

1. Try a command that needs permission
2. Approve it with "Yes"
3. Try the same command again (should need permission again)
4. Try a similar command with "Yes & similar"
5. Try that pattern again (should be auto-approved)

The permissions modal will show:
- The exact command to be executed
- A description of what it does
- Options to approve once, approve similar, or deny