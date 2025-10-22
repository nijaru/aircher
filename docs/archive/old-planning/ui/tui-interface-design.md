# TUI Interface Design

*User interface specifications for Aircher's terminal-based coding agent*

## Design Philosophy

**Keyboard-First**: All functionality accessible via keyboard shortcuts.

**Clear Visual Hierarchy**: Distinct sections for different types of content.

**Progressive Enhancement**: Basic functionality always works, polish enhances experience.

## TUI Layout Design

### Visual Layout

**New Layout Philosophy:** Vertical stack optimized for terminal workflows:

1. **Conversation Area** (top, expandable) - Chat history with collapsible tool outputs
2. **TODO Panel** (middle, compact) - Active tasks and progress tracking  
3. **Input Area** (bottom, auto-expanding) - Message composition with smart features
4. **Status Bar** (bottom edge) - Model info, context usage, costs, todo count

This layout prioritizes:
- Maximum conversation visibility
- TODO tasks visible during message composition
- Stable input position (never moves)
- Rich status information always accessible

```
┌─ Aircher Agent ─────────────────────────────────────┐
│                                                     │
│ [conversation history with collapsible sections]    │
│                                                     │
│ ┌─ Tool Output (collapsible) ──────────────────────┐│
│ │ ✓ read_file Cargo.toml — 120 lines (48ms)        ││
│ │   [syntax highlighted content]                    ││
│ │   └─ Next actions: check dependencies, run tests ││
│ └───────────────────────────────────────────────────┘│
│                                                     │
├─ TODO Tasks ────────────────────────────────────────┤
│ ☐ Fix compilation error in main.rs                  │
│ ☐ Add tests for semantic search                     │
│ ✓ Update model selection UI                         │
├─────────────────────────────────────────────────────┤
│ > Type your message here...                         │
│                                                     │  
│   [dynamic expansion up to ~20 lines]               │
│                                                     │
└─────────────────────────────────────────────────────┘
  Claude 3.5 Sonnet • 15.2k/200k • $0.03 • 3 todos
```

### Input Component Specification

```rust
pub struct InputInterface {
    // Visual styling
    border: BorderStyle::Rounded,
    prompt_symbol: "> ",
    placeholder: "Ask me anything about your code...",
    
    // Behavior
    auto_expand: true,
    max_height: 20,  // lines
    min_height: 3,   // lines
    
    // Features
    syntax_highlighting: true,
    auto_completion: true,
    history_search: true,
}
```

**Key Design Elements**:
- **Rounded border** for clear input boundary
- **'>' prompt symbol** for clarity (inspired by Claude Code)
- **Dynamic expansion** based on content length
- **Placeholder text** to guide new users
- **Status line** showing model, token usage, cost

### Interaction Patterns

**Text Entry**:
- `Enter` - Submit message (single line) or newline (multi-line)
- `Shift+Enter` - Always insert newline
- `Tab` - Accept autocomplete suggestion or indent
- `Ctrl+L` - Clear input
- `Ctrl+U` - Clear current line

**Navigation**:
- `Ctrl+↑/↓` - Navigate conversation history
- `Ctrl+R` - Search conversation history
- `Esc` - Cancel current operation
- `Ctrl+M` - Open model selection

**Content Manipulation**:
- `Ctrl+V` - Paste (with smart formatting)
- `Ctrl+K` - Cut from cursor to end
- `Ctrl+A` - Select all text

## TODO Panel

### TODO Management

**Purpose**: Show active tasks and progress during agent conversations, inspired by Amp's TODO panel but integrated into the terminal workflow.

**Features**:
- Compact display above input area
- Real-time task updates during agent work
- Visual progress indicators
- Keyboard shortcuts for management
- Auto-sync with agent tool execution

**TODO Item States**:
```rust
pub enum TodoStatus {
    Todo,        // ☐ - Pending task
    InProgress,  // 🔄 - Currently being worked on  
    Completed,   // ✓ - Finished successfully
    Failed,      // ✗ - Task failed or blocked
}

pub struct TodoItem {
    id: String,
    content: String,
    status: TodoStatus,
    priority: TodoPriority, // High, Medium, Low
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

**Display Format**:
```
├─ TODO Tasks (3 active, 1 completed) ──────────────────┤
│ 🔄 Fix compilation error in main.rs            [High]  │
│ ☐ Add tests for semantic search               [Medium] │
│ ☐ Update documentation                         [Low]   │
│ ✓ Install dependencies                        [High]  │
└─────────────────────────────────────────────────────────┘
```

## Conversation Display

### Message Types

**User Messages**:
```
👤 User
How can I fix this compilation error in src/main.rs?
```

**Agent Messages**:
```
🤖 Claude 3.5 Sonnet
I'll help you fix the compilation error. Let me first read the file to understand the issue.

[tool execution with collapsible output]

Based on the error, you need to add the missing import...
```

**System Messages**:
```
ℹ️ System
Model changed to GPT-4 Turbo
Authentication required for OpenAI
```

### Tool Output Formatting

**Collapsible Tool Results**:
```rust
pub enum ToolResultDisplay {
    Collapsed {
        summary: String,        // "✓ read_file main.rs — 150 lines (32ms)"
        status: ToolStatus,     // Success/Error/Running
        duration: Option<Duration>,
    },
    Expanded {
        summary: String,
        content: HighlightedContent,
        suggestions: Vec<NextAction>,
        metadata: ToolMetadata,
    },
}
```

**Status Indicators**:
- `🔧` Running tool
- `✓` Tool completed successfully  
- `✗` Tool failed
- `⚠` Tool completed with warnings

**Syntax Highlighting**:
```rust
pub struct HighlightedContent {
    content: String,
    language: Option<String>,  // "rust", "json", "bash"
    highlights: Vec<Highlight>,
    line_numbers: bool,
}
```

## Keyboard Shortcuts

### Global Shortcuts

| Shortcut | Action | Context |
|----------|---------|---------|
| `Ctrl+C` | Interrupt/Quit | Always available |
| `Ctrl+M` | Model selection | Always available |
| `Ctrl+R` | Search history | Always available |
| `Ctrl+L` | Clear screen | Always available |
| `F1` | Help/shortcuts | Always available |

### Input-Specific Shortcuts

| Shortcut | Action | Context |
|----------|---------|---------|
| `Enter` | Submit/newline | Input focused |
| `Shift+Enter` | Force newline | Input focused |
| `Tab` | Autocomplete/indent | Input focused |
| `Ctrl+U` | Clear line | Input focused |
| `Ctrl+K` | Cut to end | Input focused |

### Conversation Navigation

| Shortcut | Action | Context |
|----------|---------|---------|
| `Ctrl+↑/↓` | Navigate messages | Conversation view |
| `Space` | Toggle tool result | Tool result focused |
| `j/k` | Scroll (vim-style) | Optional vim mode |
| `gg/G` | Go to top/bottom | Optional vim mode |

## Theme Integration

### Color Scheme Support

```rust
pub struct TUITheme {
    // Input interface
    input_border: Color,
    input_text: Color,
    input_placeholder: Color,
    prompt_symbol: Color,
    
    // Messages
    user_message: Color,
    agent_message: Color,
    system_message: Color,
    
    // Tool outputs
    tool_success: Color,
    tool_error: Color,
    tool_running: Color,
    
    // Syntax highlighting
    syntax_keyword: Color,
    syntax_string: Color,
    syntax_comment: Color,
    syntax_number: Color,
}
```

### Responsive Design

**Terminal Size Adaptation**:
- Minimum width: 80 columns
- Minimum height: 24 rows
- Graceful degradation for smaller terminals
- Horizontal scrolling for wide content

**Content Wrapping**:
- Smart word wrapping for code blocks
- Preserve indentation in wrapped lines
- Horizontal scroll for very long lines

## Performance Considerations

### Rendering Optimization

**Efficient Updates**:
- Only redraw changed sections
- Use incremental rendering for tool outputs
- Cache syntax highlighting results
- Lazy loading for conversation history

**Memory Management**:
- Limit conversation history in memory
- Compress older messages
- Stream large tool outputs
- Garbage collect unused highlights

### Responsive Interaction

**Non-Blocking Operations**:
- Show typing indicators during LLM responses
- Progressive rendering of tool outputs
- Cancel operations with Escape key
- Background processing for intelligence features

## Accessibility

### Screen Reader Support

**Semantic Structure**:
- Clear role definitions for UI elements
- Proper heading hierarchy
- Alt text for status indicators
- Keyboard navigation announcements

**Content Organization**:
- Logical tab order
- Skip navigation options
- Content landmarks
- Progress indicators for long operations

### Keyboard Navigation

**Full Keyboard Access**:
- No mouse-only functionality
- Consistent navigation patterns
- Visible focus indicators
- Customizable shortcuts

## Implementation Notes

### Component Architecture

```rust
pub struct TUIApp {
    conversation: ConversationView,
    input: InputInterface,
    status_bar: StatusBar,
    theme: TUITheme,
}

impl TUIApp {
    // Event handling
    pub fn handle_input(&mut self, key: KeyEvent) -> AppResult<()>;
    
    // Rendering
    pub fn render(&mut self, frame: &mut Frame) -> AppResult<()>;
    
    // State management
    pub fn update_conversation(&mut self, message: Message) -> AppResult<()>;
}
```

### Integration Points

**Agent Intelligence**:
- Show intelligence status in status bar
- Highlight relevant context in tool outputs
- Display learning indicators
- Provide intelligence shortcuts

**Tool System**:
- Consistent tool result formatting
- Progress indicators for long-running tools
- Error recovery suggestions
- Tool history navigation

### Testing Strategy

**Visual Testing**:
- Snapshot tests for UI layouts
- Cross-terminal compatibility tests
- Theme variation tests
- Responsive layout tests

**Interaction Testing**:
- Keyboard shortcut functionality
- Input handling edge cases
- Conversation navigation
- Tool result interaction

## Future Enhancements

### Advanced Features

**Split View** (Future):
- Code editor + chat interface
- Side-by-side conversation history
- Workspace file browser

**Session Management** (Future):
- Multiple conversation tabs
- Session save/restore
- Conversation branching

**Collaboration** (Future):
- Shared session indicators
- Multi-user input handling
- Conflict resolution UI