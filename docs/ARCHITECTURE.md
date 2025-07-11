# Aircher Technical Architecture

## Pure Rust Design Rationale

**Why Pure Rust**: The "intelligence" we need is smart algorithms and database analytics, not heavy ML. Pure Rust provides better performance, simpler deployment, and meets all requirements fully.

### System Architecture
```
┌─ Aircher Terminal (Pure Rust) ─┐
│  ├─ TUI (Ratatui)              │
│  ├─ LLM Providers              │
│  ├─ Session Management         │
│  └─ Intelligence Engine        │
│     ├─ Project Analyzer        │ (tree-sitter AST)
│     ├─ File Scorer             │ (git2 + heuristics)  
│     ├─ Context Optimizer       │ (smart algorithms)
│     ├─ Pattern Tracker         │ (SQLite analytics)
│     └─ Cost Optimizer          │ (statistical analysis)
└─────────────────────────────────┘
```

## Provider → Model → Host Hierarchy

```
Provider: Who created the model (OpenAI, Anthropic, Google, Meta)
Model: Specific model version (gpt-4o, claude-3.5-sonnet, llama-3.1-70b)  
Host: Where the model is accessed (Direct API, OpenRouter, Local, Enterprise)
```

## Core Components

### 1. Application Structure
```rust
pub struct ArcherApp {
    ui: TuiManager,              // Ratatui interface
    providers: ProviderManager,  // Multi-provider LLM system
    intelligence: Intelligence,  // Pure Rust intelligence engine
    sessions: SessionManager,    // Conversation persistence
    config: ConfigManager,       // TOML configuration
}
```

### 2. Intelligence Engine (Pure Rust)
```rust
pub struct IntelligenceEngine {
    // Core analysis
    project_analyzer: ProjectAnalyzer,      // Tree-sitter + git2
    file_scorer: FileRelevanceScorer,       // Heuristics + recent edits
    context_optimizer: ContextOptimizer,    // Smart truncation algorithms
    
    // Learning and patterns  
    pattern_tracker: PatternTracker,        // SQLite analytics
    session_analyzer: SessionAnalyzer,      // Usage pattern analysis
    cost_optimizer: CostOptimizer,         // Statistical cost optimization
    
    // External integrations
    embedding_client: Option<EmbeddingClient>, // API-based embeddings if needed
}

impl IntelligenceEngine {
    pub async fn analyze_project(&self, files: &[PathBuf]) -> Result<ProjectContext> {
        let git_status = self.project_analyzer.get_git_status().await?;
        let file_scores = self.file_scorer.score_files(files, &git_status).await?;
        let patterns = self.pattern_tracker.get_relevant_patterns(&git_status).await?;
        
        Ok(ProjectContext {
            files: file_scores,
            git_status,
            patterns,
            recommendations: self.generate_recommendations(&file_scores, &patterns).await?,
        })
    }
    
    pub async fn optimize_context(&self, conversation: &Conversation) -> Result<CompactContext> {
        let token_count = self.count_tokens(&conversation.messages).await?;
        let important_messages = self.identify_important_messages(&conversation.messages).await?;
        let truncated = self.context_optimizer.smart_truncate(&conversation.messages, important_messages).await?;
        
        Ok(CompactContext {
            messages: truncated,
            token_savings: token_count - self.count_tokens(&truncated).await?,
            relevance_score: self.calculate_relevance_score(&truncated).await?,
        })
    }
}
```

### 3. Technology Stack
- **Rust 1.80+**: Entire application
- **Ratatui**: Terminal UI framework with async support
- **SQLite + sqlx**: Multi-database storage with async queries
- **tree-sitter**: AST parsing for all programming languages
- **git2**: Git integration and change analysis
- **tiktoken-rs**: Token counting for cost optimization
- **tokio**: Async runtime
- **tracing**: Structured logging

## Intelligence Components Deep Dive

### Project Analyzer
```rust
pub struct ProjectAnalyzer {
    git_repo: git2::Repository,
    parsers: HashMap<String, tree_sitter::Parser>, // Language parsers
    dependency_graph: DependencyGraph,
}

impl ProjectAnalyzer {
    pub async fn analyze_structure(&self, files: &[PathBuf]) -> Result<ProjectStructure> {
        let mut structure = ProjectStructure::new();
        
        for file in files {
            if let Some(language) = self.detect_language(file) {
                let parser = self.parsers.get(&language).unwrap();
                let ast = parser.parse(&fs::read_to_string(file)?)?;
                structure.add_file_analysis(file, ast, language);
            }
        }
        
        structure.build_dependency_graph()?;
        Ok(structure)
    }
    
    pub async fn get_git_status(&self) -> Result<GitStatus> {
        let statuses = self.git_repo.statuses(None)?;
        let mut git_status = GitStatus::new();
        
        for entry in statuses.iter() {
            let path = entry.path().unwrap();
            let status = entry.status();
            
            git_status.files.insert(path.to_string(), FileStatus {
                modified: status.is_wt_modified(),
                staged: status.is_index_modified(),
                new: status.is_wt_new(),
            });
        }
        
        Ok(git_status)
    }
}
```

### File Relevance Scorer
```rust
pub struct FileRelevanceScorer {
    recent_edits: RecentEditsTracker,
    import_analyzer: ImportAnalyzer,
    usage_tracker: UsageTracker,
}

impl FileRelevanceScorer {
    pub async fn score_files(&self, files: &[PathBuf], git_status: &GitStatus) -> Result<Vec<FileScore>> {
        let mut scores = Vec::new();
        
        for file in files {
            let mut score = 0.0;
            
            // Git status influence
            if git_status.is_modified(file) { score += 0.4; }
            if git_status.is_staged(file) { score += 0.3; }
            
            // Recent editing activity  
            if self.recent_edits.was_edited_recently(file) { score += 0.3; }
            
            // Import relationships
            score += self.import_analyzer.calculate_import_score(file).await? * 0.2;
            
            // Usage patterns
            score += self.usage_tracker.get_usage_frequency(file).await? * 0.1;
            
            scores.push(FileScore {
                path: file.clone(),
                relevance: score.min(1.0), // Cap at 1.0
                reasons: self.generate_score_reasons(file, score).await?,
            });
        }
        
        scores.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        Ok(scores)
    }
}
```

### Context Optimizer
```rust
pub struct ContextOptimizer {
    token_counter: TokenCounter,
    message_analyzer: MessageAnalyzer,
}

impl ContextOptimizer {
    pub async fn smart_truncate(&self, messages: &[Message], important_indices: Vec<usize>) -> Result<Vec<Message>> {
        let total_tokens = self.token_counter.count_messages(messages).await?;
        let target_tokens = self.calculate_target_tokens(total_tokens);
        
        // Always keep system message and recent messages
        let mut result = vec![messages[0].clone()]; // System message
        let mut current_tokens = self.token_counter.count_message(&messages[0]).await?;
        
        // Add important messages
        for &idx in important_indices.iter().rev() {
            if idx > 0 && idx < messages.len() {
                let msg_tokens = self.token_counter.count_message(&messages[idx]).await?;
                if current_tokens + msg_tokens <= target_tokens {
                    result.push(messages[idx].clone());
                    current_tokens += msg_tokens;
                }
            }
        }
        
        // Fill remaining space with recent messages
        for message in messages.iter().rev().skip(1) {
            let msg_tokens = self.token_counter.count_message(message).await?;
            if current_tokens + msg_tokens <= target_tokens {
                if !result.iter().any(|m| m.id == message.id) {
                    result.push(message.clone());
                    current_tokens += msg_tokens;
                }
            }
        }
        
        result.sort_by_key(|m| m.timestamp);
        Ok(result)
    }
}
```

## LLM Provider System

### Universal Provider Interface
```rust
#[async_trait]
pub trait LLMProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse>;
    async fn stream(&self, req: &ChatRequest) -> Result<Box<dyn ResponseStream>>;
    
    // Capabilities
    fn supports_tools(&self) -> bool;
    fn supports_vision(&self) -> bool;
    fn context_window(&self) -> u32;
    
    // Pricing model detection
    fn pricing_model(&self) -> PricingModel;
    
    // Cost management (for API-based providers)
    fn calculate_cost(&self, tokens: u32) -> Option<f64>;
    fn get_pricing(&self) -> Option<PricingInfo>;
    
    // Usage tracking (for subscription-based providers)
    async fn get_usage_info(&self) -> Result<Option<UsageInfo>>;
    fn usage_warning_threshold(&self) -> Option<f64>;
}

#[derive(Debug, Clone)]
pub enum PricingModel {
    PerToken {
        input_cost_per_1m: f64,
        output_cost_per_1m: f64,
        currency: String,
    },
    Subscription {
        current_usage: u64,
        limit: u64,
        reset_date: chrono::DateTime<chrono::Utc>,
        tier: SubscriptionTier,
    },
    Free, // For local models like Ollama
}

#[derive(Debug, Clone)]
pub enum SubscriptionTier {
    Pro,
    Max,
    Team,
    Enterprise,
}

#[derive(Debug, Clone)]
pub struct UsageInfo {
    pub current_usage: u64,
    pub limit: u64,
    pub reset_date: chrono::DateTime<chrono::Utc>,
    pub usage_percentage: f64,
    pub tier: SubscriptionTier,
    pub approaching_limit: bool,
}
```

### Provider Access Patterns

#### API-Based Providers (Pay-per-token)
Most providers use standard API key authentication with per-token pricing:

```toml
[hosts.claude_api]
name = "Claude API"
description = "Direct Anthropic API access"
base_url = "https://api.anthropic.com/v1"
api_key_env = "ANTHROPIC_API_KEY"
pricing_model = "per_token"
pricing_multiplier = 1.0

[hosts.openrouter]
name = "OpenRouter"
description = "25% cheaper, higher rate limits"
base_url = "https://openrouter.ai/api/v1"
api_key_env = "OPENROUTER_API_KEY"
pricing_model = "per_token"
pricing_multiplier = 0.75
features = ["cheaper", "higher_limits"]

[hosts.direct_openai]  
name = "Direct OpenAI"
base_url = "https://api.openai.com/v1"
api_key_env = "OPENAI_API_KEY"
pricing_model = "per_token"
pricing_multiplier = 1.0
features = ["official", "reliable"]
```

#### Subscription-Based Providers (Usage limits)
Claude Pro/Max uses subscription-based access with monthly usage limits:

```toml
[hosts.claude_subscription]
name = "Claude Pro/Max"
description = "Subscription-based access with usage limits"
auth_type = "session"
pricing_model = "subscription"
usage_tracking = "limit_based"
features = ["subscription", "usage_limits", "limit_warnings"]

# Usage limit configurations
[hosts.claude_subscription.limits]
pro_monthly_limit = 200000    # Tokens or equivalent
max_monthly_limit = 500000    # Higher tier limit
warning_threshold = 0.8       # Warn at 80% usage
```

## Storage Architecture

### Multi-Database Design
```rust
pub struct DatabaseManager {
    conversations: ConversationDB,  // Chat history, sessions
    knowledge: KnowledgeDB,         // Project patterns, learnings
    file_index: FileIndexDB,        // File metadata, relevance scores
    sessions: SessionDB,            // Session state, preferences
}
```

### Database Schemas
```sql
-- conversations.db
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    title TEXT,
    provider TEXT,
    model TEXT,
    host TEXT,
    created_at DATETIME,
    updated_at DATETIME
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT REFERENCES conversations(id),
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    tokens_used INTEGER,
    cost REAL,
    timestamp DATETIME
);

-- knowledge.db  
CREATE TABLE patterns (
    id TEXT PRIMARY KEY,
    pattern_type TEXT, -- file_usage, command_sequence, cost_optimization
    context_hash TEXT,
    pattern_data JSON,
    success_rate REAL,
    usage_count INTEGER,
    last_used DATETIME
);

CREATE TABLE file_usage_patterns (
    file_path TEXT,
    context_type TEXT, -- git_status, recent_edit, import_relationship
    usage_frequency INTEGER,
    success_correlation REAL,
    last_accessed DATETIME,
    PRIMARY KEY (file_path, context_type)
);

-- file_index.db
CREATE TABLE files (
    path TEXT PRIMARY KEY,
    last_modified DATETIME,
    file_size INTEGER,
    language TEXT,
    relevance_score REAL,
    import_relationships JSON,
    ast_hash TEXT
);

CREATE TABLE file_relationships (
    source_file TEXT,
    target_file TEXT, 
    relationship_type TEXT, -- import, dependency, reference
    strength REAL,
    last_updated DATETIME,
    PRIMARY KEY (source_file, target_file, relationship_type)
);
```

## UI Architecture

### Ratatui Component System
```rust
pub struct UIManager {
    layout: LayoutManager,      // Dynamic layout calculation
    chat_view: ChatWidget,      // Conversation display with streaming
    input: InputWidget,         // Multi-line input with autocomplete
    status: StatusWidget,       // Provider, cost, token usage
    help: HelpWidget,          // Progressive disclosure help
    selection: SelectionModal, // Provider/model/host selection
}
```

### Model Selection Modal
```
┌──────────────── Select AI Configuration ─────────────────┐
│ Provider: [OpenAI] [Anthropic] [Google] [Meta]          │
│          ──────                                          │
│                                                          │
│ ● gpt-4o                    $3.75/$11.25 per 1M tokens  │
│ ○ gpt-4o-mini               $0.11/$0.45 per 1M tokens   │
│                                                          │
│ Host: ● OpenRouter (25% cheaper)  ○ Direct (standard)   │
│                                                          │
│ Session: 15.2K tokens ($0.17 saved)                     │
└──────────────────────────────────────────────────────────┘
```

## Performance Considerations

### Startup Optimization
- **Target**: <100ms cold start, <50ms warm start
- **Techniques**: Lazy loading, cached configurations, minimal dependencies
- **Database**: Connection pooling, prepared statements

### Memory Management  
- **Target**: <50MB baseline, <200MB with large contexts
- **Techniques**: Efficient string handling, smart caching
- **Intelligence**: Streaming file analysis, incremental updates

### Token Processing
- **Target**: <10ms for token counting, <50ms for context optimization
- **Techniques**: Parallel processing, cached token counts, smart algorithms

## Development Workflow

### Build System
```bash
# Development
cargo build
cargo test --all
cargo clippy --all-targets

# Release
cargo build --release
cargo test --release
```

### Code Organization
```
src/
├── main.rs                 # Entry point
├── app/                    # Main application logic
├── ui/                     # Ratatui components and widgets
├── providers/              # LLM provider implementations  
├── intelligence/           # Intelligence engine components
│   ├── analyzer.rs         # Project analysis
│   ├── scorer.rs           # File relevance scoring
│   ├── optimizer.rs        # Context optimization
│   ├── patterns.rs         # Pattern tracking
│   └── cost.rs            # Cost optimization
├── storage/                # Database management
├── config/                 # Configuration system
└── utils/                  # Shared utilities
```

## Benefits of Pure Rust Architecture

### Performance
- **Single binary**: No runtime dependencies
- **Memory efficiency**: Direct memory access, no serialization overhead
- **Startup speed**: No Python interpreter initialization  
- **Async throughout**: Non-blocking operations with tokio

### Development  
- **Single toolchain**: One language, consistent patterns
- **Type safety**: Compile-time guarantees across entire codebase
- **Better IDE support**: Full language server integration
- **Simpler debugging**: No cross-language boundary issues

### Deployment
- **Zero dependencies**: Just ship the binary
- **Cross-platform**: Single codebase for all platforms
- **Easy installation**: `cargo install aircher`
- **No version conflicts**: Self-contained application

### Future Extensibility
- **Plugin system**: Can add WASM plugins for extensibility
- **API integration**: Easy to add new model providers
- **MCP extraction**: Can still extract intelligence as MCP server later if needed
- **Performance modules**: Can optimize critical paths without language boundaries

This architecture provides all the intelligence features we need while maintaining simplicity, performance, and maintainability.