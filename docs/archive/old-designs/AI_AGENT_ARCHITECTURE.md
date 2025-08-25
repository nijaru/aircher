# AI Agent Architecture for Aircher

## Overview

Transform Aircher from a semantic search tool into a full AI coding assistant similar to Claude Code, Cursor, and other AI development tools. The AI agent will use the existing search capabilities internally while providing a rich coding assistance interface.

## Core Architecture

### 1. Tool System

The agent needs tools to interact with the codebase:

```rust
// src/agent/tools/mod.rs
pub trait AgentTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, params: serde_json::Value) -> Result<ToolOutput>;
}

pub struct ToolOutput {
    pub success: bool,
    pub result: serde_json::Value,
    pub usage: Option<TokenUsage>,
}
```

#### Essential Tools

1. **File Operations**
   - `ReadFile` - Read file contents with line numbers
   - `WriteFile` - Create/overwrite files
   - `EditFile` - Search and replace within files
   - `ListFiles` - List directory contents with filters
   - `FindFiles` - Search for files by name/pattern

2. **Code Analysis** (using existing search)
   - `SearchCode` - Semantic search across codebase
   - `FindDefinition` - Find function/class definitions
   - `FindReferences` - Find usage of symbols
   - `GetContext` - Get relevant code context

3. **System Operations**
   - `RunCommand` - Execute shell commands
   - `GitStatus` - Get git status/diff
   - `RunTests` - Execute test suites

4. **Project Understanding**
   - `GetProjectStructure` - Understand project layout
   - `GetDependencies` - List project dependencies
   - `GetTodoList` - Track tasks and progress

### 2. Agent Controller

```rust
// src/agent/controller.rs
pub struct AgentController {
    tools: HashMap<String, Box<dyn AgentTool>>,
    intelligence: IntelligenceEngine,
    provider: Box<dyn Provider>,
    conversation: ConversationManager,
}

impl AgentController {
    pub async fn process_message(&mut self, message: &str) -> Result<AgentResponse> {
        // 1. Analyze message intent
        // 2. Gather relevant context
        // 3. Generate response with tool calls
        // 4. Execute tools and collect results
        // 5. Generate final response
    }
}
```

### 3. Conversation Management

Enhance existing conversation system for coding context:

```rust
// src/agent/conversation.rs
pub struct CodingConversation {
    pub messages: Vec<Message>,
    pub project_context: ProjectContext,
    pub active_files: Vec<PathBuf>,
    pub task_list: Vec<Task>,
}

pub struct ProjectContext {
    pub root_path: PathBuf,
    pub language: ProgrammingLanguage,
    pub framework: Option<String>,
    pub recent_changes: Vec<GitChange>,
}
```

### 4. Enhanced TUI

Transform the chat interface into a coding workspace:

```
┌─────────────────────────────────────────────────────────────┐
│ Aircher AI Assistant - Project: ~/myproject                 │
├─────────────────────────────────────────────────────────────┤
│ Active Files: src/main.rs, src/lib.rs                      │
│ Context: 15 files, 2.3k lines                              │
├─────────────────────────────────────────────────────────────┤
│ Assistant: I'll help you implement the new feature. Let me │
│ first understand your current code structure.               │
│                                                             │
│ 🔍 Searching for relevant files...                          │
│ 📖 Reading src/main.rs                                      │
│ 📖 Reading src/config.rs                                    │
│                                                             │
│ I found your configuration system. Here's my plan:          │
│ 1. Add new config field to Config struct                    │
│ 2. Update the parser to handle the new field               │
│ 3. Add validation for the new parameter                    │
│                                                             │
│ Shall I proceed with these changes?                         │
├─────────────────────────────────────────────────────────────┤
│ You: Yes, go ahead                                          │
├─────────────────────────────────────────────────────────────┤
│ [Tools: SearchCode ReadFile EditFile] [Model: llama3.3]    │
└─────────────────────────────────────────────────────────────┘
```

## Implementation Phases

### Phase 1: Tool Framework (Week 1)
- [ ] Create tool trait and base implementations
- [ ] Implement file operations (read, write, edit)
- [ ] Add command execution with safety checks
- [ ] Create tool registry and discovery

### Phase 2: Agent Controller (Week 2)
- [ ] Build message processing pipeline
- [ ] Implement tool calling parser
- [ ] Add context gathering logic
- [ ] Create response generation system

### Phase 3: Enhanced TUI (Week 3)
- [ ] Redesign chat interface for coding
- [ ] Add file browser component
- [ ] Show tool execution progress
- [ ] Add code preview/diff display

### Phase 4: Integration (Week 4)
- [ ] Connect to existing Intelligence Engine
- [ ] Integrate semantic search as internal tool
- [ ] Add project-wide understanding
- [ ] Implement task tracking

## Tool Calling Format

Use a simple XML-style format that works with all models:

```
User: Help me add error handling to the config parser