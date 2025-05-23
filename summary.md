# Aircher Project Summary

## **Project Overview**
**Aircher** is an API-agnostic AI coding assistant CLI tool designed to surpass Claude Code through intelligent automation and superior context management. Unlike Claude Code's single-provider approach, Aircher works with any LLM provider (OpenAI, Claude, Gemini, Ollama, custom endpoints).

## **Core Architecture Decisions**

### **Multi-Database Storage Architecture**
```
.aircher/
├── config.toml              # TOML configuration (chosen over YAML)
├── conversations.db         # Chat history with file references
├── knowledge.db            # Project understanding & decisions
├── file_index.db           # File relationships & metadata
├── todos.db               # Todo management system
└── cache/                 # Search results, snippets
```

**Key Decision**: Separate specialized databases instead of single database for efficiency and organization.

### **Intelligent Context Management**
- **Task-aware context**: Automatically detects current task (debugging, feature, refactor, etc.)
- **File relevance scoring**: 0.0-1.0 scores based on direct relevance, dependencies, context, historical usage
- **Smart compaction triggers**: Task completion, context shift, stagnation (not just token limits)
- **File snippets**: Store code snippets/references instead of full files to save tokens
- **Progressive decay**: File relevance decreases over time unless actively used

### **Token Efficiency Strategy**
- **File references with snippets** instead of full file content
- **Smart context limits**: 75% soft limit, 90% hard limit
- **Provider-aware token counting** (different tokenizers per provider)
- **Intelligent compaction** preserving key decisions, incomplete work, errors

## **Key Differentiators from Claude Code**

1. **Multi-Provider Support**: Works with any LLM, not just Claude
2. **Autonomous Web Search**: Automatically searches for current documentation/solutions
3. **Superior Context Management**: Task-aware, not just token-limit based
4. **Temporal Awareness**: Knows current time, searches for up-to-date information
5. **Project Knowledge Persistence**: Remembers architectural decisions, tech stack, patterns
6. **Smart Error Recovery**: Proactive solution search with current information

## **System Prompt Architecture**
- **Dynamic system prompt** includes current time, project info, available tools
- **Temporal awareness** for automatic searches on "latest", "current", version queries
- **Tool-aware** explanations of when to use search, file operations, etc.
- **Behavior guidance** for autonomous web search

## **Configuration: TOML Choice**
**Chosen TOML over YAML** for better:
- Comment support
- Clearer section nesting
- Less indentation-sensitive
- Better for complex configurations
- More readable for developers

## **Todo System (Claude Code inspired)**
- Auto-generated todos from AI analysis
- Manual todo creation with priorities
- Subtask support with dependencies
- Progress tracking with time estimates
- Terminal UI with status icons (⏳ 🔄 ✅)

## **File Tree Management**
**Compact + On-Demand approach**:
- Cache compact representation with relevance scores
- Rebuild when files change (fsnotify watcher)
- Filter by relevance, .gitignore, file size
- Show only relevant parts in context (max 3 levels deep)
- **More efficient than storing full tree**

## **Auto-Update System**
Using `rhysd/go-github-selfupdate`:
- SHA256 checksum validation
- ECDSA signature verification
- Rollback support on failed updates
- GitHub releases integration

## **MCP Integration**
Using `mark3labs/mcp-go` library:
- Built-in MCP servers: filesystem, git, web, shell, test, code analysis
- Type-safe tool definitions
- Both server and client support

## **Features to Adopt from Claude Code**
From changelog analysis:
- ✅ Todo system with auto-generation
- ✅ Memory system (# prefix messages)
- ✅ @-mention files for context
- ✅ Auto-compaction (but smarter)
- ✅ Thinking mode detection ("think", "think harder")
- ✅ Resume conversations
- ✅ Web search capability
- ✅ Settings management
- 🆕 Vim bindings option
- 🆕 Custom slash commands
- 🆕 Image support (paste images)
- 🆕 Word-level diffs
- 🆕 Cost tracking

## **Implementation Priorities**
1. **Phase 1**: System prompt + temporal awareness + basic chat + file ops + one LLM provider
2. **Phase 2**: Autonomous web search + MCP + conversation persistence + todo system
3. **Phase 3**: Multi-provider + auto-update + advanced error recovery + knowledge DB
4. **Phase 4**: Advanced git + test integration + performance optimization
5. **Phase 5**: Enterprise features + integrations + UI polish

## **Critical Technical Details**

### **Smart Compaction**
```go
// Compaction triggers (priority order):
1. Task completion detection
2. Context shift detection
3. Conversation stagnation
4. Token threshold (backup safety)

// Preservation strategy:
- Task summaries with key decisions
- Incomplete work and TODOs
- Recent context (last 10 messages)
- Error contexts and solutions
- Architectural decisions
```

### **File Relevance Scoring**
```go
- Direct relevance (0.8): Recently mentioned/modified
- Dependency relevance (0.6): Imports/includes
- Contextual relevance (0.3): Same module/similar patterns
- Historical relevance (0.4): Helped with similar tasks
- Time decay: Scores decrease over time
```

### **Project Knowledge Structure**
```go
type ProjectKnowledge struct {
    TechStack       TechStackInfo      // Languages, frameworks, versions
    Architecture    ArchitecturalDecisions
    FileStructure   *CompactFileTree
    CommonPatterns  []CodePattern
    Dependencies    []DependencyInfo
    BuildSystem     BuildSystemInfo
    TestingSetup    TestingInfo
}
```

## **Key Go Dependencies**
```go
// Core
"github.com/spf13/cobra"              // CLI framework
"github.com/BurntSushi/toml"          // TOML config
"github.com/mattn/go-sqlite3"         // Database

// Auto-update
"github.com/rhysd/go-github-selfupdate/selfupdate"
"github.com/blang/semver"

// MCP
"github.com/mark3labs/mcp-go/mcp"
"github.com/mark3labs/mcp-go/server"
"github.com/mark3labs/mcp-go/client"

// UI
"github.com/charmbracelet/bubbletea"
"github.com/charmbracelet/lipgloss"

// File watching
"github.com/fsnotify/fsnotify"
```

## **User Experience Flow**
1. **Initialize**: `aircher init` creates `.aircher/` directory with config
2. **Chat**: `aircher` starts conversation with auto-context
3. **Clear**: `aircher clear` intelligently rebuilds context
4. **Config**: `aircher config` opens TUI settings
5. **Update**: `aircher update` self-updates binary
6. **Resume**: `aircher --continue` resumes last conversation

This summary captures all major architectural decisions, technical choices, and implementation strategies discussed. The outline document contains the detailed technical specifications for each component.
