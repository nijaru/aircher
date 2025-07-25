# Aircher UI Modes

## Overview

Aircher implements a sophisticated mode system similar to Claude Code, providing different interaction patterns for various workflows. Modes are **session-based** and reset when you restart the application.

## Mode Types

### 🔄 Default Mode
- **Status**: No special indicator
- **Behavior**: Prompts for approval before making file changes
- **Use case**: Careful review of all changes, learning mode
- **Safety**: Highest - every change requires explicit approval

### ⏵⏵ Auto-Accept Edits Mode  
- **Status**: `⏵⏵ auto-accept edits on (shift+tab to cycle)` in purple color
- **Behavior**: Applies individual file changes automatically without prompting each edit
- **Scope**: Single-response automation - still requires user input for next steps
- **Use case**: Rapid development, trusted AI changes, removing edit approval friction
- **Safety**: Medium - changes applied immediately but logged, user controls flow

### 🚀 Turbo Mode
- **Status**: `▇ 🚀 turbo mode on (shift+tab to cycle)` in red color with animated spinner
- **Behavior**: Autonomous task completion with minimal human intervention
- **Scope**: Multi-step automation - continues working until task completion
- **Use case**: Complex refactoring, automated debugging, end-to-end feature implementation
- **Safety**: Lower - AI drives the entire workflow with minimal checkpoints
- **Toggle**: `/turbo` command toggles turbo mode on/off

### ⏸ Plan Mode
- **Status**: `⏸ plan mode on (shift+tab to cycle)` in cyan color
- **Behavior**: Creates detailed execution plans before making any changes
- **Use case**: Complex refactoring, architecture changes, learning
- **Safety**: Highest - see full plan before any execution

## Mode Control

### Shift+Tab Sequence
```
Default → Plan Mode → Auto-Accept → Turbo Mode → Default (cycles)
```

All modes are accessible through the single shift+tab cycling interface for consistency.

### Activation Messages
- **Auto-accept enabled**: "⏵⏵ Auto-accept edits enabled. File changes will be applied automatically."
- **Plan mode enabled**: "⏸ Plan mode enabled. Will create plans before making changes."  
- **Turbo mode enabled**: "🚀 Turbo mode enabled. AI will work autonomously until task completion."
- **Default mode**: "Default mode. Will prompt for approval before making changes."

## Status Bar Evolution

### Empty Chat
```
? for shortcuts                          claude-sonnet-4 (anthropic) 100%
```
Shows discovery help when starting.

### During Conversation - Default Mode
```
shift+tab to cycle modes                 claude-sonnet-4 (anthropic) 85%
```

### During Conversation - Auto-Accept Mode
```
⏵⏵ auto-accept edits on (shift+tab to cycle)          claude-sonnet-4 (anthropic) 85%
```

### During Conversation - Plan Mode
```
⏸ plan mode on (shift+tab to cycle)                   claude-sonnet-4 (anthropic) 85%
```

### During Conversation - Turbo Mode (Any Edit Mode)
```
🚀 turbo mode on • ⏵⏵ auto-accept edits (/turbo to toggle)      claude-sonnet-4 (anthropic) 85%
```

```
🚀 turbo mode on • ⏸ plan mode (/turbo to toggle)               claude-sonnet-4 (anthropic) 85%
```

## Mode Behaviors

### Default Mode
```bash
User: "Fix the bug in auth.rs"
AI: "I found the issue. Should I apply this fix to auth.rs?"
> [y/n] prompt appears
User: "y"
AI: "Applied fix to auth.rs"
```

### Auto-Accept Mode
```bash
User: "Fix the bug in auth.rs"  
AI: "I found the issue. Applying fix to auth.rs..."
> Changes applied automatically
AI: "Fixed authentication bug in auth.rs (line 45)"
```

### Plan Mode
```bash
User: "Refactor the authentication system"
AI: "✻ Thinking…
     I'm in plan mode, so I need to assess...and create a plan...without executing any changes."
AI: "## Authentication Refactor Plan
     1. Extract AuthService interface
     2. Implement JWT validation  
     3. Update middleware integration
     4. Add comprehensive tests
     
     Proceed with this plan? [y/n]"
User: "y"
AI: "Executing plan step 1..."
```

### Turbo Mode (Autonomous Operation)
```bash
User: "Fix all the TypeScript errors in the project"
AI: "🚀 Turbo mode: Starting autonomous error fixing..."
AI: "Found 15 TypeScript errors across 8 files. Beginning fixes..."
AI: "✓ Fixed import errors in auth.ts"
AI: "✓ Added type annotations in user.ts"  
AI: "✓ Resolved interface conflicts in api.ts"
AI: "◦ Running type check... ✓ 12 errors remaining"
AI: "✓ Fixed generic type constraints in utils.ts"
AI: "◦ Running type check... ✓ 3 errors remaining"
AI: "✓ Updated deprecated types in components.ts"
AI: "◦ Running final type check... ✓ All errors resolved!"
AI: "🚀 Task completed: Fixed all 15 TypeScript errors across 8 files"
```

## Technical Implementation

### Plan Mode System Message Injection
Plan mode works by injecting system messages that modify Claude's behavior:

1. **Mode Detection**: The TUI detects when plan mode is active
2. **System Message**: Adds context like "You are in plan mode" to the conversation
3. **Behavior Override**: Claude recognizes this and switches from "execute" to "plan" mode
4. **Thinking Transparency**: The thinking process shows the mode awareness: `"I'm in plan mode, so I need to assess...and create a plan...without executing any changes"`
5. **Structured Output**: Claude generates proper project plans with phases, time estimates, and approval modals

### Key Technical Insights
- **Seamless mode switching**: Claude immediately recognizes context changes mid-conversation
- **Behavioral adaptation**: Changes from execution to planning without losing context
- **Structured planning**: Generates comprehensive plans with realistic time estimates
- **User control**: Plan approval system gives users control over AI execution
- **Transparent reasoning**: Thinking section shows the mode transition process

### Plan Mode Output Format
```
✻ Thinking… [Shows mode awareness and planning process]

⏺ [Analysis of current state and improvement opportunities]

╭─────────────────────────────────────────────────────────╮
│ Ready to code?                                          │
│ Here is Claude's plan:                                  │
│ ╭─────────────────────────────────────────────────────╮ │
│ │ [Structured plan with phases and time estimates]   │ │
│ ╰─────────────────────────────────────────────────────╯ │
│ ❯ 1. Yes, and auto-accept edits                        │
│   2. Yes, and manually approve edits                   │
│   3. No, keep planning                                  │
╰─────────────────────────────────────────────────────────╯
```

This approach enables controllable AI interactions that feel natural while providing transparency and user control.

## Integration with Agent System

### File Operations
- **Default**: Each file edit prompts for approval
- **Auto-accept**: File edits applied directly via agent tools
- **Plan**: Shows file changes in plan before execution

### Error Handling
- **Default**: Stops on first error, prompts user
- **Auto-accept**: Logs errors, continues with remaining changes
- **Plan**: Validates plan feasibility before execution

## Best Practices

### When to Use Default Mode
- Learning how the AI makes changes
- Working with critical/production code
- Reviewing AI decision-making patterns
- First time using a complex command

### When to Use Auto-Accept
- Trusted AI operations (formatting, imports)
- Batch processing multiple files
- Rapid prototyping sessions
- Familiar, low-risk changes

### When to Use Plan Mode
- Complex refactoring across multiple files
- Architecture changes
- Understanding AI's approach before committing
- Teaching/learning scenarios

## Safety Features

### Session Reset
- All modes reset to **Default** on application restart
- Prevents accidental auto-accept in new sessions
- Forces conscious mode selection for each work session

### Mode Visibility
- Current mode always visible in status bar
- Clear activation messages when switching modes
- Mode explanation available in help (`/help`)

### Rollback Support
- All changes logged regardless of mode
- Git integration recommended for easy rollback
- File backups maintained during auto-accept operations

## Comparison with Claude Code

### Similarities
- **Shift+Tab cycling**: Same keyboard shortcut
- **Session-based**: Modes reset on restart
- **Status bar integration**: Dynamic mode display
- **Auto-accept concept**: Similar workflow acceleration

### Aircher Enhancements
- **Plan mode**: More detailed execution planning
- **Context-aware indicators**: Smarter status bar updates
- **Agent integration**: Seamless with file operation tools
- **Safety messaging**: Clear mode transition feedback

## Future Enhancements

### Planned Features
- **Custom mode shortcuts**: User-defined key bindings
- **Mode persistence**: Optional cross-session mode memory
- **Conditional auto-accept**: Rules for when to prompt
- **Plan templates**: Common refactoring pattern plans

### Integration Opportunities
- **Git hooks**: Mode-aware commit messages
- **IDE integration**: Sync modes with editor state
- **Team workflows**: Shared mode configurations
- **CI/CD integration**: Mode-based automation triggers

## Authentication Flow

### Overview
Aircher provides a seamless authentication flow directly within the TUI, prioritizing authenticated providers and enabling quick setup for new providers.

### Provider Selection
When using `/model` command:
- **Authenticated providers appear first** with ✓ icon
- **Local providers** (like Ollama) show ⚡ icon when available
- **Unauthenticated providers** show ✗ icon and appear after authenticated ones
- **Selecting an unauthenticated provider** automatically triggers the auth wizard

### Authentication Status Icons
- **✓** - Authenticated (API key configured and valid)
- **✗** - Not authenticated or service unavailable
- **⚡** - Local provider (no authentication needed)

### Ollama Integration
- Automatically checks if Ollama service is running
- Shows as unavailable (✗) if service not found
- Moves to unauthenticated list when not accessible
- No "(Local)" suffix for cleaner display

### Message Formatting

#### Claude Code Style
Aircher matches Claude Code's message formatting for improved readability:

- **User messages**: Comment-colored (gray) with "> " prefix
- **System messages**: Comment-colored (gray) for status updates
- **Assistant messages**: Off-white/beige color for better contrast
- **Thinking messages**: Comment-colored with "✻" indicator
- **Tool use**: Highlighted with special formatting

#### Message Spacing
- Empty lines between messages for visual separation
- Role-specific indentation (2 spaces for content)
- Consistent styling across all message types

## Message History Navigation

### Overview
Aircher maintains a history of your recent messages (up to 1000 entries) for easy recall and editing.

### Navigation Keys
- **Up Arrow**: Navigate to older messages in history
- **Down Arrow**: Navigate to newer messages in history
- **Any character**: Typing resets history navigation

### Behavior
```bash
User: "/search authentication bug"
User: "/model claude"
User: "fix the login issue"
# Press Up Arrow → recalls "fix the login issue"
# Press Up Arrow → recalls "/model claude"  
# Press Up Arrow → recalls "/search authentication bug"
# Press Down Arrow → returns to "/model claude"
# Type any character → resets to new input
```

### Features
- Duplicate consecutive messages are not saved
- History persists for the entire session
- Intelligent bounds checking (won't go past oldest/newest)
- Seamless integration with autocomplete (Up/Down moves autocomplete when visible)

## Command Shortcuts

### Help Access
- **?**: Quick help display (equivalent to `/help`)
- **/help** or **/h**: Full help with all commands and shortcuts

### Autocomplete
- **/** triggers command autocomplete immediately
- Shows all available commands with descriptions
- Tab/Enter accepts suggestion
- Escape dismisses autocomplete

## Command Enhancements

### Commands with Arguments
Several commands now support inline arguments:

```bash
/search <query>         # Semantic search
/model <name>          # Quick model selection
/compact <instructions> # Compaction with custom context
```

### Improved Command Parsing
- All commands are intercepted before sending to LLM
- Unknown commands show helpful error messages
- Partial command matching with aliases (`/h` → `/help`)

## Configuration

### Current Implementation
- Modes are purely session-based (no persistence)
- Cycling order is fixed: Default → Auto-Accept → Plan
- All modes available in all contexts
- Message history: 1000 entries max, session-based

### Future Configuration Options
```toml
[ui.modes]
default_mode = "default"  # default|auto-accept|plan
persistent_modes = false  # Remember across sessions
custom_cycle_order = ["default", "auto-accept"]  # Exclude plan mode
auto_accept_patterns = ["*.md", "format", "imports"]  # Smart auto-accept

[ui.history]
max_entries = 1000      # Message history size
persist_history = false # Save between sessions
search_history = true   # Enable Ctrl+R style search
```

This mode system significantly enhances Aircher's usability by adapting to different workflow needs while maintaining safety and transparency.