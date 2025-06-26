# Worktree and Multi-Context Management

## Overview

Aircher's worktree support enables parallel development workflows by providing context isolation per git worktree while maintaining cross-context insights and knowledge sharing. This addresses the real-world need for developers to work on multiple features, bug fixes, and experiments simultaneously.

## Core Concepts

### Context Hierarchy

```
Global Context (User-wide patterns)
└── Project Context (Repository knowledge)
    └── Worktree Context (Branch-specific conversations)
        └── Session Context (Active task state)
```

### Context Types

1. **Global**: User preferences, cross-project patterns, credential management
2. **Project**: Architecture decisions, team conventions, shared knowledge base
3. **Worktree**: Branch-specific conversations, file states, task context
4. **Session**: Active work session, temporary cache, current task focus

## Worktree Detection and Management

### Automatic Worktree Discovery

```rust
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub id: String,
    pub path: PathBuf,
    pub branch: String,
    pub commit: String,
    pub is_bare: bool,
    pub is_main: bool,
    pub last_active: DateTime<Utc>,
}

pub struct WorktreeManager {
    project_root: PathBuf,
    current_worktree: Option<WorktreeInfo>,
    available_worktrees: HashMap<String, WorktreeInfo>,
    context_cache: Arc<RwLock<HashMap<String, WorktreeContext>>>,
}

impl WorktreeManager {
    pub async fn discover_worktrees(&mut self) -> Result<Vec<WorktreeInfo>> {
        // Use `git worktree list --porcelain` to discover all worktrees
        // Parse output to build WorktreeInfo structures
        // Update database with discovered worktrees
    }
    
    pub async fn switch_context(&mut self, worktree_id: &str) -> Result<()> {
        // Load worktree-specific context
        // Update current working context
        // Preserve cross-context insights
    }
}
```

### Context Switching

When switching between worktrees, Aircher:

1. **Preserves Session State**: Current conversations and context remain available
2. **Loads Worktree Context**: Branch-specific conversations and file relevance
3. **Maintains Cross-Context Insights**: Successful patterns from other branches remain accessible
4. **Updates File Relevance**: File scoring updates based on worktree-specific state

## Database Schema Integration

### Context Management Tables

The storage architecture includes several new tables to support multi-context workflows:

- `contexts`: Hierarchical context tracking
- `context_relationships`: Cross-context insights and relationships  
- `worktrees`: Git worktree registry and metadata
- `context_switches`: History of context changes
- `cross_context_insights`: Learnings shared between contexts

### Context-Aware Queries

```sql
-- Get all conversations for current project across worktrees
SELECT c.*, w.branch_name, w.worktree_path
FROM conversations c
JOIN worktrees w ON c.worktree_id = w.id  
WHERE c.project_id = :project_id
ORDER BY c.updated_at DESC;

-- Find similar solutions from other worktrees
SELECT cci.description, cci.confidence_score,
       source_ctx.name as source_branch,
       cci.evidence
FROM cross_context_insights cci
JOIN contexts source_ctx ON cci.source_context_id = source_ctx.id
WHERE cci.insight_type = 'similar_solution'
AND cci.confidence_score > 0.7
AND source_ctx.context_type = 'worktree'
ORDER BY cci.effectiveness_rating DESC;
```

## CLI Commands

### Worktree Management

```bash
# List all worktrees with status
aircher worktree list
# Output:
# main        /path/to/project       (active)   [2h ago]
# feature-auth /path/to/feature-auth  (inactive) [1d ago]  
# bugfix-login /path/to/bugfix-login  (stale)    [1w ago]

# Switch to different worktree context
aircher worktree switch feature-auth
# Updates context, loads branch-specific conversations

# Compare insights between worktrees
aircher worktree compare main feature-auth
# Shows shared patterns, conflicts, and unique insights

# Transfer learnings between worktrees
aircher worktree transfer feature-auth main
# Applies successful patterns from feature-auth to main
```

### Context Management

```bash
# Show current context hierarchy
aircher context status
# Output:
# Global: /Users/dev/.config/aircher
# Project: /path/to/project/.aircher  
# Worktree: feature-auth (branch: feature/authentication)
# Session: debugging-session-123 (active 2h)

# List available contexts
aircher context list
# Show all projects, worktrees, and sessions

# Switch to specific context
aircher context switch project:other-repo
aircher context switch worktree:main
aircher context switch session:feature-dev-456
```

### Cross-Context Insights

```bash
# Show insights from other contexts
aircher insights
# Output:
# 🔍 Similar pattern found in worktree 'main':
#   Authentication middleware pattern (confidence: 0.85)
#   Last used: 2h ago, success rate: 92%

# Search for patterns across all contexts
aircher insights search "authentication middleware"

# Get recommendations for current task
aircher insights recommend --task debugging
```

## File System Organization

### Hierarchical Storage

```
# Global user configuration
~/.config/aircher/
├── global.db              # Cross-project patterns
├── credentials.toml       # API keys
└── config.toml           # User preferences

# Project-level storage
.aircher/
├── db/
│   ├── conversations.db   # All conversations (multi-worktree)
│   ├── knowledge.db      # Project knowledge
│   ├── file_index.db     # File metadata
│   └── sessions.db       # Session management
├── worktrees/            # Per-worktree context
│   ├── main/
│   │   ├── context.json  # Branch-specific context
│   │   └── cache/        # Worktree cache
│   ├── feature-auth/
│   │   ├── context.json
│   │   └── cache/
│   └── bugfix-login/
│       ├── context.json
│       └── cache/
└── sessions/             # Session-specific data
    ├── debugging-123/
    └── feature-dev-456/
```

### Context Configuration

Each worktree maintains a `context.json` file:

```json
{
  "worktree_id": "feature-auth",
  "branch": "feature/authentication", 
  "last_active": "2024-01-15T10:30:00Z",
  "task_type": "feature_development",
  "active_files": [
    "src/auth/middleware.rs",
    "src/auth/jwt.rs",
    "tests/auth_test.rs"
  ],
  "conversation_ids": [
    "conv-auth-123",
    "conv-auth-456"
  ],
  "insights": {
    "successful_patterns": [
      "jwt-validation-pattern",
      "error-handling-middleware"
    ],
    "failed_approaches": [
      "session-based-auth"
    ]
  },
  "cross_context_references": [
    {
      "source_worktree": "main",
      "pattern": "middleware-composition",
      "confidence": 0.9
    }
  ]
}
```

## Implementation Phases

### Phase 1: Foundation (MVP-Ready)
1. **Context-Aware Schema**: Update database schemas with context fields
2. **Worktree Detection**: Basic git worktree discovery and tracking
3. **Context Switching**: Simple context switching with state preservation
4. **Basic CLI Commands**: `aircher worktree list/switch`

### Phase 2: Intelligent Context Management
1. **Cross-Context Insights**: Pattern recognition across worktrees
2. **Smart Context Switching**: Automatic context detection
3. **File Relevance**: Context-aware file scoring
4. **Advanced CLI**: Full worktree management commands

### Phase 3: Advanced Features
1. **Context Transfer**: Move insights between worktrees
2. **Team Collaboration**: Shared insights with privacy
3. **Temporal Context**: Time-based context relevance
4. **Performance Optimization**: Lazy loading and caching

## Benefits for Developers

### Parallel Development
- Work on multiple features simultaneously without losing context
- Each worktree maintains its own conversation history and file relevance
- Switch between tasks without losing AI context

### Knowledge Sharing
- Successful patterns from one branch automatically available in others
- Cross-context insights help avoid repeating solved problems
- Team knowledge accumulates across all development streams

### Improved Productivity  
- No need to restart conversations when switching branches
- AI maintains awareness of what works across different contexts
- Reduced context switching overhead

### Real-World Workflow Support
- Matches how developers actually work with git worktrees
- Supports complex branching strategies and parallel development
- Integrates naturally with existing git workflows

## Future Enhancements

### Advanced Context Features
- **Smart Merging**: Automatically merge successful patterns when branches merge
- **Context Recommendations**: Suggest relevant contexts based on current task
- **Cross-Project Insights**: Share patterns between different repositories
- **Team Context Sharing**: Collaborative insights while maintaining privacy

### Performance Optimizations
- **Lazy Context Loading**: Load context data on-demand
- **Context Prefetching**: Preload likely-to-be-used contexts
- **Efficient Context Storage**: Optimize storage for frequently accessed contexts
- **Context Cleanup**: Automatic cleanup of stale contexts

### Integration Features
- **IDE Integration**: Context switching from IDE
- **Git Hook Integration**: Automatic context switching on branch changes
- **CI/CD Integration**: Context-aware build and deployment insights
- **Issue Tracking**: Link contexts to issues and pull requests

This worktree support positions Aircher as the only AI coding assistant built for real-world parallel development workflows, providing a significant competitive advantage in the developer tools market.