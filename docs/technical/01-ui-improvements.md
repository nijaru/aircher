# Aircher UI/UX Improvements

## Overview

This document outlines the comprehensive UI/UX improvements made to Aircher's terminal interface, inspired by Claude Code's clean design principles while maintaining Aircher's unique multi-provider capabilities.

## Design Philosophy

### Core Principles
- **Clean Separation**: UI controls separate from conversation history
- **Progressive Disclosure**: Information revealed contextually as needed
- **Professional Aesthetics**: Modern CLI tool design with proper spacing and borders
- **Intuitive Interaction**: Natural keyboard shortcuts and command discovery
- **Contextual Help**: Dynamic assistance that adapts to user context

### Claude Code Inspiration
- Welcome screen with boxed layout and clear instructions
- Footer-based help system instead of cluttering chat history
- Clean prompt design with single ">" symbol
- Professional styling with consistent spacing
- Contextual hints that guide user behavior

## Major Improvements Implemented

### 1. Fixed Double Prompt Issue
**Problem**: Two ">" symbols appeared (purple and white)
**Solution**: 
- Removed extra prompt rendering in footer
- Set textarea prompt to empty string
- Single clean white ">" prompt handled by textarea component

### 2. Enhanced Welcome Experience
**Before**: Simple system message in chat
**After**: 
- Professional welcome screen with boxed layout
- Current working directory display
- Getting started tips
- Example usage suggestions
- Only appears when no chat history exists

### 3. Contextual Footer System
**Before**: Static keyboard shortcuts
**After**: Dynamic footer that shows:
- Default: "? for shortcuts"
- Command typing: Real-time autocomplete with descriptions
- Shortcuts mode: Full keyboard shortcut reference
- Contextual hints based on user input

### 4. Command System Redesign
**Removed Commands**:
- `/think` - Now automatic based on keywords
- `/search` - Now automatic via MCP tools

**Enhanced Commands**:
- `/help` - Toggles shortcuts in footer (not chat)
- `/status` - Shows current system setup
- `/init` - Creates AGENTS.md project memory
- Added provider/model switching commands
- Added session management commands

**New Command Categories**:
```
Core: /help, /clear, /cost, /status, /init
Config: /config, /memory
Providers: /provider, /model  
Sessions: /save, /load, /export, /session
Tools: /mcp, /tools
```

### 5. Automatic Intelligence Features

#### Thinking Mode Detection
- Automatic detection based on keywords:
  - "think", "analyze", "plan", "reason", "strategy"
  - "step by step", "break down", "pros and cons"
  - "implications", "trade-offs", "methodology"
- No manual commands needed
- Works only with compatible providers/models
- Transparent operation with status indicators

#### Search Integration
- Automatic web search via MCP tools
- Triggered by context and intent, not commands
- Multiple provider support (Brave, Tavily, custom)
- Intelligent caching to avoid redundant searches
- Cost transparency in status display

### 6. Improved Input Handling
**Before**: Single-line textinput with limited multiline support
**After**:
- Textarea component for proper multiline editing
- Shift+Enter for new lines
- Better text editing experience
- Configurable height (starts at 1 line, expands as needed)

### 7. Enhanced Help System
**Three-tier Help Approach**:
1. **Hints**: "? for shortcuts" 
2. **Contextual**: Command autocomplete while typing
3. **Full Help**: Complete keyboard shortcut reference

**Help Features**:
- Footer-based (doesn't clutter chat)
- Keyboard shortcut: ? or Ctrl+H
- Organized by categories with emojis
- Shows both commands and keyboard shortcuts

### 8. Context Panel Improvements
**Enhanced Content**:
- Session statistics (messages, cost, tokens)
- Current AI provider and model
- Streaming status indicators
- Available MCP tools listing
- Dynamic width handling

**Better Integration**:
- Proper width calculations when toggled
- Responsive design that adapts to terminal size
- Clear toggle instructions

### 8.1. Context Usage Display
**Real-time Token Tracking**:
- Status bar shows current context usage as fraction (e.g., "44k/200k")
- Updates in real-time as conversation progresses
- Model-aware limits based on current service (GPT-4: 128k, Claude-3: 200k, etc.)

**Visual Indicators**:
- Normal usage: Default color (e.g., "44k/200k")
- Warning level (80%): Yellow/amber color (e.g., "160k/200k")
- Critical level (95%): Red color (e.g., "190k/200k")

**Context Management Integration**:
- Quick access to context compaction when approaching limits
- Proactive warnings before hitting context limits
- Clear indication when context compaction is recommended

### 9. Visual Design Enhancements
**Styling Improvements**:
- Professional color scheme with consistent theming
- Proper border spacing from terminal edges
- Enhanced markdown rendering with syntax highlighting
- Better message formatting with timestamps
- Status bar with provider and cost information

**Layout Improvements**:
- Welcome screen with centered, boxed content
- Proper viewport height calculations
- Dynamic width adjustments for panels
- Responsive design for different terminal sizes

## Technical Implementation

### UI Architecture Changes
```go
// Enhanced Model structure
type Model struct {
    input         textarea.Model    // Multiline input support
    viewport      viewport.Model    // Chat display
    showShortcuts bool             // Footer shortcuts toggle
    // Removed: thinking, searching bools (now automatic)
}
```

### Command Routing
- Commands no longer add messages to chat history
- Results shown in footer/status area
- Better error messages with usage hints
- Contextual command suggestions

### Automatic Detection Systems
- Keyword-based thinking mode activation
- Intent-based search triggering via MCP
- Provider capability detection
- Graceful fallbacks for unsupported features

## User Experience Improvements

### Before vs After

**Command Discovery**:
- Before: Manual `/help` command cluttered chat
- After: "? for shortcuts" hint → contextual autocomplete → full help

**Input Experience**:
- Before: Single line with awkward multiline handling
- After: Proper textarea with Shift+Enter for newlines

**Chat Cleanliness**:
- Before: Commands and UI noise mixed with conversation
- After: Pure conversation history, UI controls in footer/status

**Help System**:
- Before: Static help panel obscuring content
- After: Progressive help that adapts to user needs

**Provider Features**:
- Before: Manual `/think` commands regardless of provider support
- After: Automatic thinking based on context and provider capabilities

## Keyboard Shortcuts

### Core Navigation
- `?` or `Ctrl+H`: Toggle shortcuts display
- `Ctrl+T`: Toggle context panel
- `Ctrl+C` or `Esc`: Exit application

### Input Handling
- `Enter`: Send message
- `Shift+Enter`: New line (multiline input)

### Command System
- `/` prefix: Slash commands with autocomplete
- Real-time suggestions as you type
- Descriptions shown for matching commands

## Future Considerations

### Potential Enhancements
1. **Theme Customization**: User-configurable color schemes
2. **Advanced Search**: More sophisticated search intent detection
3. **Session Visualization**: Timeline view of conversation sessions
4. **Tool Integration**: Visual feedback for MCP tool operations
5. **Customizable Shortcuts**: User-defined keyboard shortcuts

### Accessibility
- Ensure proper screen reader support
- High contrast mode options
- Keyboard-only navigation improvements

## Testing and Validation

### Manual Testing Checklist
- [ ] Single prompt symbol (no double ">")
- [ ] Welcome screen appears on fresh start
- [ ] "?" toggles shortcuts in footer
- [ ] Command autocomplete works while typing "/"
- [ ] Ctrl+T toggles context panel properly
- [ ] Multiline input with Shift+Enter
- [ ] Commands show results in footer, not chat
- [ ] Automatic thinking detection with keywords
- [ ] Clean chat history without UI noise

### Integration Testing
- [ ] Works with all supported providers
- [ ] MCP tool integration functions correctly
- [ ] Session management operates properly
- [ ] Cost tracking displays accurately

## Conclusion

These improvements transform Aircher from a basic CLI tool into a professional terminal application that rivals desktop interfaces while maintaining the efficiency and power of command-line interaction. The Claude Code-inspired design provides familiarity for users while the enhanced automation and multi-provider support gives Aircher unique advantages in the AI assistant space.

The focus on clean separation between UI and conversation, progressive help disclosure, and automatic intelligence features creates a more intuitive and powerful user experience that scales from simple queries to complex development workflows.