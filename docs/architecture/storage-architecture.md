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

## Database Schemas

### Conversations Database (`conversations.db`)

#### Core Tables
```sql
-- Conversation tracking
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
    metadata JSON
);

-- Individual messages
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
CREATE INDEX idx_messages_conversation_timestamp ON messages(conversation_id, timestamp);
CREATE INDEX idx_messages_role ON messages(role);
CREATE INDEX idx_attachments_message ON message_attachments(message_id);
CREATE INDEX idx_compaction_conversation ON compaction_history(conversation_id);
```

### Knowledge Database (`knowledge.db`)

#### Project Analysis Tables
```sql
-- Project components and analysis
CREATE TABLE project_components (
    id TEXT PRIMARY KEY,
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

-- Success pattern tracking
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
    effectiveness_rating REAL
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
-- Comprehensive file metadata
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
    line_count INTEGER
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
-- File relevance scoring
CREATE TABLE file_relevance_scores (
    id TEXT PRIMARY KEY,
    file_id TEXT REFERENCES files(id) ON DELETE CASCADE,
    context_type TEXT NOT NULL,
    base_score REAL DEFAULT 0.0,
    frequency_score REAL DEFAULT 0.0,
    recency_score REAL DEFAULT 0.0,
    dependency_score REAL DEFAULT 0.0,
    success_correlation REAL DEFAULT 0.0,
    final_score REAL DEFAULT 0.0,
    calculated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP,
    metadata JSON
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
-- User sessions
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_activity TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status TEXT DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'expired')),
    project_path TEXT,
    working_directory TEXT,
    environment_variables JSON,
    preferences JSON
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

### File System Storage Structure

```
.agents/
├── db/
│   ├── core/                # Core operational databases
│   │   ├── conversations.db
│   │   └── sessions.db
│   ├── knowledge/           # Knowledge and analysis databases
│   │   ├── knowledge.db
│   │   └── file_index.db
│   └── cache/               # Temporary/computed data
├── content/
│   ├── embeddings/          # Vector embeddings binary files
│   ├── attachments/         # Message attachments
│   ├── backups/            # Database backups
│   └── cache/              # Temporary cache files
├── indexes/
│   ├── faiss/              # FAISS indexes for similarity search
│   └── search/             # Full-text search indexes
└── logs/
    ├── storage.log         # Storage operation logs
    └── performance.log     # Performance metrics
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

## Future Enhancements

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