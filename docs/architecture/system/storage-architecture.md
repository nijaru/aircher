# Storage Architecture Technical Specification

## Overview

Aircher implements a sophisticated multi-database storage architecture designed to optimize different types of data storage and retrieval patterns. This hybrid approach combines SQLite databases for structured data with file system storage for large content and specialized indexes for semantic search.

## Architecture Principles

### Hybrid Storage Strategy
- **SQLite Databases**: Structured metadata, relationships, and transactional data
- **File System**: Large content, binary data, and version-controlled files
- **Specialized Indexes**: Vector embeddings and search optimization
- **Performance Optimization**: Each storage type optimized for its specific use case

### Multi-Database Design Philosophy
Rather than a single monolithic database, Aircher uses specialized databases:
- **conversations.db**: Chat history and interaction metadata
- **knowledge.db**: Project analysis, documentation, and learned patterns
- **file_index.db**: File metadata, relationships, and change tracking
- **sessions.db**: User sessions, preferences, and temporary state

### Hierarchical Context Architecture
Aircher supports multiple levels of context isolation for parallel development workflows:
- **Global Context**: User-wide settings and cross-project patterns
- **Project Context**: Repository-specific knowledge and configurations
- **Worktree Context**: Branch-specific conversations and file states
- **Session Context**: Task-specific temporary state and active work

### Context Inheritance Model
```
Global Knowledge (User patterns, preferences)
└── Project Knowledge (Architecture, decisions, shared patterns)
    └── Worktree Context (Branch-specific conversations, file states)
        └── Session Context (Active task state, temporary cache)
```

## Database Schemas

### Conversations Database (`conversations.db`)

#### Core Tables
```sql
-- Conversation tracking with context hierarchy
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    title TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    provider TEXT NOT NULL,
    model TEXT,
    total_tokens INTEGER DEFAULT 0,
    total_cost REAL DEFAULT 0.0,
    status TEXT DEFAULT 'active',
    -- Context hierarchy
    project_id TEXT,
    worktree_id TEXT DEFAULT 'main',
    session_id TEXT,
    context_type TEXT DEFAULT 'general' CHECK (context_type IN ('general', 'debugging', 'feature', 'review', 'refactor')),
    metadata JSON
);

-- Individual messages with context inheritance
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT REFERENCES conversations(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system', 'tool')),
    content TEXT,
    tokens INTEGER,
    cost REAL DEFAULT 0.0,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    tool_calls JSON,
    tool_results JSON,
    -- Context inheritance from conversation
    project_id TEXT,
    worktree_id TEXT,
    session_id TEXT,
    metadata JSON
);

-- Message attachments
CREATE TABLE message_attachments (
    id TEXT PRIMARY KEY,
    message_id TEXT REFERENCES messages(id) ON DELETE CASCADE,
    type TEXT NOT NULL CHECK (type IN ('file', 'image', 'code', 'diff')),
    path TEXT,
    content_hash TEXT,
    size_bytes INTEGER,
    metadata JSON
);

-- Conversation compaction tracking
CREATE TABLE compaction_history (
    id TEXT PRIMARY KEY,
    conversation_id TEXT REFERENCES conversations(id) ON DELETE CASCADE,
    trigger_type TEXT NOT NULL,
    original_message_count INTEGER,
    compacted_message_count INTEGER,
    tokens_saved INTEGER,
    performed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    summary TEXT,
    preserved_messages JSON
);
```

#### Indexes
```sql
CREATE INDEX idx_conversations_updated_at ON conversations(updated_at);
CREATE INDEX idx_conversations_provider ON conversations(provider);
CREATE INDEX idx_conversations_context ON conversations(project_id, worktree_id, context_type);
CREATE INDEX idx_conversations_worktree ON conversations(worktree_id);
CREATE INDEX idx_messages_conversation_timestamp ON messages(conversation_id, timestamp);
CREATE INDEX idx_messages_role ON messages(role);
CREATE INDEX idx_messages_context ON messages(project_id, worktree_id);
CREATE INDEX idx_attachments_message ON message_attachments(message_id);
CREATE INDEX idx_compaction_conversation ON compaction_history(conversation_id);
```

### Knowledge Database (`knowledge.db`)

#### Context Management Tables
```sql
-- Context hierarchy tracking
CREATE TABLE contexts (
    id TEXT PRIMARY KEY,
    context_type TEXT NOT NULL CHECK (context_type IN ('global', 'project', 'worktree', 'session')),
    parent_context_id TEXT REFERENCES contexts(id),
    name TEXT NOT NULL,
    path TEXT, -- file system path for project/worktree contexts
    git_branch TEXT, -- for worktree contexts
    git_commit TEXT, -- current commit for worktree contexts
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_active TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status TEXT DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'archived')),
    metadata JSON
);

-- Cross-context relationships and insights
CREATE TABLE context_relationships (
    id TEXT PRIMARY KEY,
    source_context_id TEXT REFERENCES contexts(id),
    target_context_id TEXT REFERENCES contexts(id),
    relationship_type TEXT NOT NULL CHECK (relationship_type IN (
        'parent_child', 'sibling', 'similar_task', 'shared_pattern', 'knowledge_transfer'
    )),
    strength REAL DEFAULT 1.0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata JSON,
    UNIQUE(source_context_id, target_context_id, relationship_type)
);

-- Project components and analysis
CREATE TABLE project_components (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    component TEXT NOT NULL,
    type TEXT NOT NULL,
    description TEXT,
    confidence REAL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    evidence JSON,
    metadata JSON,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Documentation files tracking
CREATE TABLE documentation_files (
    id TEXT PRIMARY KEY,
    file_path TEXT UNIQUE NOT NULL,
    doc_type TEXT,
    title TEXT,
    purpose TEXT,
    sections JSON,
    cross_refs JSON,
    last_analyzed TIMESTAMP,
    content_hash TEXT
);

-- Decision tracking and architectural choices
CREATE TABLE project_decisions (
    id TEXT PRIMARY KEY,
    decision_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    rationale TEXT,
    alternatives_considered JSON,
    impact_assessment TEXT,
    made_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status TEXT DEFAULT 'active' CHECK (status IN ('active', 'superseded', 'reverted')),
    tags JSON
);

-- Success pattern tracking with context awareness
CREATE TABLE success_patterns (
    id TEXT PRIMARY KEY,
    pattern_type TEXT NOT NULL,
    description TEXT,
    context JSON,
    success_indicators JSON,
    failure_indicators JSON,
    confidence_score REAL,
    usage_count INTEGER DEFAULT 0,
    last_successful TIMESTAMP,
    effectiveness_rating REAL,
    -- Context tracking
    discovered_in_context TEXT REFERENCES contexts(id),
    applicable_contexts JSON, -- list of context types/IDs where pattern works
    cross_context_effectiveness JSON -- effectiveness scores per context type
);

-- Cross-context insights and learnings
CREATE TABLE cross_context_insights (
    id TEXT PRIMARY KEY,
    insight_type TEXT NOT NULL CHECK (insight_type IN (
        'pattern_reuse', 'similar_solution', 'anti_pattern', 'best_practice'
    )),
    source_context_id TEXT REFERENCES contexts(id),
    target_context_id TEXT REFERENCES contexts(id),
    description TEXT,
    evidence JSON,
    confidence_score REAL,
    usage_count INTEGER DEFAULT 0,
    last_applied TIMESTAMP,
    effectiveness_rating REAL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

#### Learning and Context Tables
```sql
-- Conversation effectiveness tracking
CREATE TABLE conversation_effectiveness (
    id TEXT PRIMARY KEY,
    conversation_id TEXT,
    task_type TEXT,
    success_score REAL CHECK (success_score >= 0.0 AND success_score <= 1.0),
    completion_time_minutes INTEGER,
    tokens_used INTEGER,
    cost REAL,
    user_satisfaction INTEGER CHECK (user_satisfaction >= 1 AND user_satisfaction <= 5),
    key_factors JSON,
    lessons_learned TEXT,
    recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Context relevance learning
CREATE TABLE context_relevance_feedback (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL,
    conversation_id TEXT,
    relevance_score REAL CHECK (relevance_score >= 0.0 AND relevance_score <= 1.0),
    actual_usefulness REAL CHECK (actual_usefulness >= 0.0 AND actual_usefulness <= 1.0),
    context_type TEXT,
    feedback_source TEXT DEFAULT 'implicit',
    recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### File Index Database (`file_index.db`)

#### File Tracking and Relationships
```sql
-- Comprehensive file metadata with context awareness
CREATE TABLE files (
    id TEXT PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    extension TEXT,
    size_bytes INTEGER,
    permissions TEXT,
    created_at TIMESTAMP,
    modified_at TIMESTAMP,
    accessed_at TIMESTAMP,
    content_hash TEXT,
    git_status TEXT,
    is_binary BOOLEAN DEFAULT FALSE,
    language TEXT,
    encoding TEXT,
    line_count INTEGER,
    -- Context tracking
    project_id TEXT,
    discovered_in_worktree TEXT -- which worktree first discovered this file
);

-- File relationships and dependencies
CREATE TABLE file_relationships (
    id TEXT PRIMARY KEY,
    source_file_id TEXT REFERENCES files(id) ON DELETE CASCADE,
    target_file_id TEXT REFERENCES files(id) ON DELETE CASCADE,
    relationship_type TEXT NOT NULL CHECK (
        relationship_type IN ('imports', 'requires', 'includes', 'references', 'tests', 'documents')
    ),
    strength REAL DEFAULT 1.0,
    discovered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_verified TIMESTAMP,
    metadata JSON,
    UNIQUE(source_file_id, target_file_id, relationship_type)
);

-- File change tracking
CREATE TABLE file_changes (
    id TEXT PRIMARY KEY,
    file_id TEXT REFERENCES files(id) ON DELETE CASCADE,
    change_type TEXT NOT NULL CHECK (
        change_type IN ('created', 'modified', 'deleted', 'renamed', 'moved')
    ),
    old_path TEXT,
    new_path TEXT,
    old_hash TEXT,
    new_hash TEXT,
    changed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    change_size INTEGER,
    git_commit TEXT
);

-- Access pattern tracking
CREATE TABLE file_access_patterns (
    id TEXT PRIMARY KEY,
    file_id TEXT REFERENCES files(id) ON DELETE CASCADE,
    access_type TEXT NOT NULL CHECK (
        access_type IN ('read', 'write', 'execute', 'context_inclusion')
    ),
    accessed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    accessor TEXT, -- user, system, or tool identifier
    context_id TEXT, -- conversation or task context
    duration_ms INTEGER,
    success BOOLEAN DEFAULT TRUE
);
```

#### Relevance and Scoring Tables
```sql
-- File relevance scoring with context hierarchy
CREATE TABLE file_relevance_scores (
    id TEXT PRIMARY KEY,
    file_id TEXT REFERENCES files(id) ON DELETE CASCADE,
    context_id TEXT REFERENCES contexts(id),
    context_type TEXT NOT NULL,
    base_score REAL DEFAULT 0.0,
    frequency_score REAL DEFAULT 0.0,
    recency_score REAL DEFAULT 0.0,
    dependency_score REAL DEFAULT 0.0,
    success_correlation REAL DEFAULT 0.0,
    cross_context_boost REAL DEFAULT 0.0, -- boost from similar contexts
    final_score REAL DEFAULT 0.0,
    calculated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP,
    metadata JSON,
    UNIQUE(file_id, context_id, context_type)
);

-- File content snippets for quick access
CREATE TABLE file_snippets (
    id TEXT PRIMARY KEY,
    file_id TEXT REFERENCES files(id) ON DELETE CASCADE,
    snippet_type TEXT NOT NULL CHECK (
        snippet_type IN ('header', 'function', 'class', 'important', 'summary')
    ),
    content TEXT,
    line_start INTEGER,
    line_end INTEGER,
    relevance_score REAL DEFAULT 0.0,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Sessions Database (`sessions.db`)

#### Session Management
```sql
-- User sessions with context hierarchy
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_activity TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status TEXT DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'expired')),
    project_path TEXT,
    working_directory TEXT,
    environment_variables JSON,
    preferences JSON,
    -- Context hierarchy
    context_id TEXT REFERENCES contexts(id),
    project_id TEXT,
    worktree_id TEXT,
    task_type TEXT CHECK (task_type IN ('debugging', 'feature', 'review', 'refactor', 'maintenance')),
    task_description TEXT
);

-- Temporary state and caches
CREATE TABLE session_cache (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES sessions(id) ON DELETE CASCADE,
    cache_key TEXT NOT NULL,
    cache_value JSON,
    expires_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(session_id, cache_key)
);

-- User preferences and settings
CREATE TABLE user_preferences (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    preference_key TEXT NOT NULL,
    preference_value JSON,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, preference_key)
);
```

## Embedding and Search Integration

### Vector Embeddings Storage
```sql
-- Embedding metadata (stored in knowledge.db)
CREATE TABLE embedding_metadata (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL,
    chunk_index INTEGER,
    embedding_model TEXT NOT NULL,
    dimension INTEGER NOT NULL,
    content_hash TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    content_preview TEXT
);

-- Search indexes for approximate similarity
CREATE TABLE search_indexes (
    id TEXT PRIMARY KEY,
    index_type TEXT NOT NULL CHECK (index_type IN ('faiss', 'annoy', 'exact')),
    index_path TEXT,
    embedding_model TEXT,
    dimension INTEGER,
    item_count INTEGER DEFAULT 0,
    last_rebuilt TIMESTAMP,
    config JSON
);
```

### Hierarchical File System Storage Structure

```
# Global user-level storage
~/.config/aircher/
├── global.db               # Global user patterns and preferences
├── credentials.toml        # API keys and authentication
└── config.toml            # User-wide settings

# Project-level storage
.agents/
├── db/
│   ├── core/               # Core operational databases
│   │   ├── conversations.db    # All conversations (multi-worktree)
│   │   ├── knowledge.db        # Project knowledge and patterns
│   │   ├── file_index.db       # File metadata and relationships
│   │   └── sessions.db         # Session and context management
│   └── cache/              # Temporary/computed data
├── worktrees/              # Per-worktree isolated data
│   ├── main/               # Main branch context
│   │   ├── context.json        # Worktree-specific context
│   │   └── cache/             # Worktree-specific cache
│   ├── feature-auth/       # Feature branch context
│   │   ├── context.json
│   │   └── cache/
│   └── bugfix-login/       # Another branch context
│       ├── context.json
│       └── cache/
├── content/
│   ├── embeddings/         # Vector embeddings binary files
│   ├── attachments/        # Message attachments
│   ├── backups/           # Database backups
│   └── cache/             # Temporary cache files
├── indexes/
│   ├── faiss/             # FAISS indexes for similarity search
│   └── search/            # Full-text search indexes
├── sessions/              # Session-specific temporary data
│   ├── {session-id}/      # Individual session data
│   └── active/            # Currently active sessions
└── logs/
    ├── storage.log        # Storage operation logs
    ├── worktree.log       # Worktree management logs
    └── performance.log    # Performance metrics
```

## Performance Optimization

### SQLite Configuration
```sql
-- WAL mode for better concurrency
PRAGMA journal_mode = WAL;

-- Optimize for performance
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000;  -- 64MB cache
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 268435456; -- 256MB mmap

-- Auto-vacuum for maintenance
PRAGMA auto_vacuum = INCREMENTAL;
```

### Connection Pooling
- **Read Connections**: Pool of 5-10 connections for read operations
- **Write Connections**: Single connection per database for writes
- **Transaction Management**: Explicit transaction boundaries for batch operations

### Indexing Strategy
- **Primary Keys**: UUIDs for distributed compatibility
- **Foreign Keys**: Enforce referential integrity
- **Composite Indexes**: Multi-column indexes for common query patterns
- **Partial Indexes**: For filtered queries on large tables

## Data Lifecycle Management

### Retention Policies
```toml
[storage.lifecycle]
# File version history retention
file_version_retention_days = 90
file_version_max_count = 50

# Conversation archival
conversation_archive_days = 365
conversation_max_per_project = 1000

# Learning data retention
pattern_learning_min_samples = 10
success_correlation_decay_rate = 0.95
```

### Cleanup and Maintenance
- **Daily**: Remove expired cache entries and temporary files
- **Weekly**: Vacuum databases and update statistics
- **Monthly**: Rebuild search indexes and compress old data
- **Quarterly**: Archive old conversations and clean up unused embeddings

## Backup and Recovery

### Backup Strategy
```toml
[storage.backup]
enabled = true
backup_interval = "24h"
backup_retain_days = 30
compression_enabled = true
compression_threshold = "10MB"
```

### Recovery Procedures
1. **Database Corruption**: Automatic repair using SQLite recovery tools
2. **Data Loss**: Restore from most recent backup with minimal data loss
3. **Index Corruption**: Rebuild search indexes from source data
4. **File System Issues**: Validate and repair file references

## Monitoring and Health Checks

### Performance Metrics
```toml
[storage.monitoring]
health_check_interval = "5m"
disk_usage_alert_threshold = 0.85
query_performance_logging = true
slow_query_threshold = "1s"
```

### Health Indicators
- **Database Size**: Monitor growth patterns and disk usage
- **Query Performance**: Track slow queries and optimization opportunities
- **Cache Hit Rates**: Monitor effectiveness of caching strategies
- **Error Rates**: Track database errors and connection issues

## Configuration Integration

### TOML Configuration Example
```toml
[storage]
data_dir = ".agents"
max_db_size = "1GB"
backup_enabled = true
backup_interval = "24h"

[storage.sqlite]
journal_mode = "WAL"
synchronous = "NORMAL"
cache_size = -64000
temp_store = "MEMORY"
auto_vacuum = "INCREMENTAL"
optimize_interval = "7d"

[storage.embeddings]
enabled = true
embedding_model = "text-embedding-3-small"
dimension = 1536
index_type = "faiss"
similarity_threshold = 0.7
update_batch_size = 100
rebuild_interval = "30d"

[storage.performance]
concurrent_reads = 10
batch_insert_size = 1000
index_rebuild_threshold = 10000
query_timeout = "30s"
connection_pool_size = 5
```

## Implementation Status

### ✅ Completed
- Basic SQLite schema design
- Multi-database architecture foundation
- File system integration planning

### 🚧 In Progress
- Advanced indexing strategies
- Performance optimization implementation
- Backup and recovery system

### ❌ Pending
- Vector embedding integration
- Advanced search capabilities
- Comprehensive monitoring system
- Data lifecycle automation

## Worktree and Multi-Context Management

### Worktree Detection and Management
```sql
-- Worktree registry
CREATE TABLE worktrees (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    branch_name TEXT NOT NULL,
    worktree_path TEXT NOT NULL,
    git_commit TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_active TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status TEXT DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'stale')),
    metadata JSON,
    UNIQUE(project_id, worktree_path)
);

-- Context switching history
CREATE TABLE context_switches (
    id TEXT PRIMARY KEY,
    from_context_id TEXT REFERENCES contexts(id),
    to_context_id TEXT REFERENCES contexts(id),
    switch_reason TEXT,
    switched_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    user_id TEXT,
    metadata JSON
);
```

### Cross-Context Query Patterns
```sql
-- Find conversations across all worktrees for a project
SELECT c.*, w.branch_name, w.worktree_path
FROM conversations c
JOIN worktrees w ON c.worktree_id = w.id
WHERE c.project_id = ? AND c.context_type = 'debugging'
ORDER BY c.updated_at DESC;

-- Get cross-context insights for current task
SELECT cci.*, 
       source_ctx.name as source_context,
       target_ctx.name as target_context
FROM cross_context_insights cci
JOIN contexts source_ctx ON cci.source_context_id = source_ctx.id
JOIN contexts target_ctx ON cci.target_context_id = target_ctx.id
WHERE cci.insight_type = 'similar_solution'
AND cci.confidence_score > 0.7
ORDER BY cci.effectiveness_rating DESC;
```

### Context Management Commands
```bash
# Worktree management
aircher worktree list
aircher worktree switch feature-branch
aircher worktree compare main feature-branch

# Context insights
aircher context insights
aircher context patterns --across-worktrees
aircher context transfer feature-branch main
```

## Future Enhancements

### Advanced Multi-Context Features
- **Smart Context Switching**: Automatic context detection and switching
- **Cross-Worktree Insights**: Learning propagation between parallel work streams
- **Team Context Sharing**: Shared insights while maintaining conversation privacy
- **Temporal Context**: Time-based context relevance and historical debugging

### Advanced Features
- **Distributed Storage**: Support for multi-node deployments
- **Cloud Sync**: Synchronization with cloud storage providers
- **Advanced Analytics**: Machine learning on usage patterns
- **Real-time Collaboration**: Shared project knowledge bases

### Optimization Opportunities
- **Columnar Storage**: For analytical workloads
- **Compression**: Advanced compression for historical data
- **Partitioning**: Time-based partitioning for large datasets
- **Caching**: Multi-level caching with Redis integration
- **Context Prefetching**: Preload relevant context for faster switching