# Aircher TUI Demo Script

## 🎨 Beautiful Terminal Interface Demo

This demo showcases Aircher's modern terminal user interface built with Charmbracelet's Bubble Tea framework.

### Prerequisites
```bash
# Ensure you have a modern terminal with color support
# Recommended: iTerm2, Alacritty, or Windows Terminal
# Terminal size: at least 100x30 for best experience

# Build Aircher
make build
```

### Demo Flow

#### 1. Launch Aircher with Beautiful Welcome
```bash
./build/aircher
```

**What to show:**
- Beautiful header with Aircher logo 🏹
- Status bar showing provider information
- Welcome messages with emoji formatting
- Clean input field with prompt styling
- Professional borders and layout

#### 2. Demonstrate Rich Markdown Rendering
Type in the chat:
```
Can you show me a code example with syntax highlighting?
```

**What to show:**
- User message appears with timestamp and user icon 👤
- Assistant response streams in real-time
- Code blocks render with syntax highlighting
- Markdown formatting (headers, lists, emphasis)
- Smooth scrolling and text wrapping

#### 3. Interactive Help System
Press: `Ctrl+H`

**What to show:**
- Help panel slides in smoothly
- Comprehensive list of slash commands
- Keyboard shortcuts with visual formatting
- Easy-to-read command descriptions
- Toggle help on/off with same shortcut

#### 4. Context Panel Features  
Press: `Ctrl+T`

**What to show:**
- Context sidebar appears on the right
- Session information display
- Message count and statistics
- Available MCP tools list
- Real-time cost tracking
- Provider status indicators

#### 5. Slash Commands with Visual Feedback
Try these commands:
```
/cost
/think
/clear
/help
```

**What to show:**
- Commands process with visual feedback
- Status messages appear in chat
- Thinking mode toggle with indicators
- Clean conversation clearing
- Professional command responses

#### 6. Real-time Status Indicators
```
/think
```

**What to show:**
- 🤔 Thinking indicator in status bar
- Visual state changes in real-time
- Color-coded status messages
- Smooth animations and transitions

#### 7. Error Handling with Style
Type an invalid command:
```
/invalid-command
```

**What to show:**
- ❌ Error messages with proper styling
- Red error coloring for visibility
- Helpful error information
- Graceful error recovery

#### 8. Multi-line Input and Streaming
Type a longer question:
```
Explain how to implement a REST API in Go with proper error handling, middleware, and testing. Include code examples.
```

**What to show:**
- Input field handles longer text
- Response streams in character by character
- 📡 Streaming indicator in status bar
- Smooth text rendering
- Automatic scrolling to bottom

#### 9. Responsive Layout
Resize terminal window while using Aircher

**What to show:**
- Interface adapts to terminal size
- Text reflows properly
- Panels resize appropriately  
- No layout breaking or artifacts
- Consistent user experience

#### 10. Professional Exit
Press: `Ctrl+C`

**What to show:**
- Clean application exit
- No terminal artifacts left behind
- Proper cleanup and shutdown
- Returns to normal shell prompt

### Key TUI Features Demonstrated

#### Visual Design
- 🎨 **Modern styling** with Lipgloss
- 🌈 **Color-coded messages** (user/assistant/system/error)
- 📱 **Responsive layout** that adapts to terminal size
- 🎯 **Professional borders** and clean typography

#### Interactive Elements
- ⌨️ **Keyboard shortcuts** (Ctrl+H, Ctrl+T, Ctrl+C)
- 🔄 **Real-time streaming** responses
- 📋 **Interactive panels** (help, context, status)
- 💬 **Rich slash commands** with visual feedback

#### User Experience
- 🚀 **Smooth animations** and transitions
- 📊 **Live status indicators** (thinking, searching, streaming)
- 💰 **Real-time cost tracking** in status bar
- 🛠️ **Context awareness** with file and tool information

#### Technical Excellence
- ⚡ **High performance** rendering with Bubble Tea
- 🎭 **Markdown rendering** with Glamour
- 🔧 **Extensible architecture** for new features
- 🧪 **Error resilience** with graceful handling

### Advanced Demo Features

#### For Power Users
```bash
# Show configuration
./build/aircher config

# Non-interactive mode with TUI output
./build/aircher -p "Hello Aircher" --output-format json

# Resume specific session
./build/aircher -r "session-123"
```

#### Integration Demos
```bash
# Pipe input for processing
echo "Review this code" | ./build/aircher -p "review"

# Process files
cat example.go | ./build/aircher -p "explain this code"
```

### Comparison Points

#### vs Traditional CLI Tools
- **Rich formatting** instead of plain text
- **Real-time feedback** instead of static responses  
- **Interactive navigation** instead of linear output
- **Visual status** instead of text-only indicators

#### vs Web Interfaces
- **Terminal native** - no browser required
- **Keyboard efficient** - power user friendly
- **Lightweight** - minimal resource usage
- **Scriptable** - automation friendly

### Demo Tips

1. **Use a large terminal** (100x30 minimum) for best effect
2. **Enable color support** in your terminal
3. **Demonstrate responsiveness** by resizing during use
4. **Show keyboard shortcuts** - users love efficiency
5. **Highlight real-time features** - streaming is impressive
6. **Toggle panels** to show interface flexibility
7. **Use longer prompts** to show text handling capabilities

This demo showcases Aircher as a modern, professional AI coding assistant with a beautiful terminal interface that rivals desktop applications while maintaining the power and efficiency of command-line tools.