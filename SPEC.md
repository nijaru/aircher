# Aircher Technical Specification

This document provides detailed technical specifications for Aircher's architecture, data models, APIs, and implementation strategies.

## System Architecture

### Core Foundation

```go
type AircherCore struct {
    repl           *repl.REPL              // Charmbracelet Bubble Tea TUI
    nonInteractive *NonInteractiveMode     // CLI mode
    sessionManager *SessionManager         // Session management
    commandRouter  *commands.Router        // Slash command routing
    contextEngine  *context.Engine         // Intelligent context management
    providerMgr    *providers.Manager      // Multi-LLM provider system
    storageEngine  *storage.Engine         // SQLite multi-DB storage
    searchEngine   *search.Engine          // Autonomous web search
    memoryManager  *memory.Manager         // AIRCHER.md integration
    mcpManager     *mcp.Manager           // MCP server management
}
```

### Modern Terminal Interface (TUI) Architecture

Built with Charmbracelet's excellent TUI ecosystem:

```go
// Bubble Tea Model for Interactive Interface (Claude Code-inspired design)
type Model struct {
    // Core components
    input    textarea.Model     // Multiline user input field
    viewport viewport.Model     // Message display area
    
    // Application state
    messages     []Message       // Conversation history
    width        int            // Terminal width
    height       int            // Terminal height
    ready        bool           // Initialization status
    
    // UI state
    streaming      bool         // Response streaming indicator
    showHelp       bool         // Help panel visibility (full help)
    showContext    bool         // Context panel visibility
    showShortcuts  bool         // Footer shortcuts visibility
    
    // Styling and rendering
    styles       Styles         // Lipgloss styling definitions
    renderer     *glamour.TermRenderer // Markdown rendering
}

// TUI Dependencies
// github.com/charmbracelet/bubbletea v1.3.5   - TUI framework
// github.com/charmbracelet/lipgloss v1.1.0    - Styling and layout
// github.com/charmbracelet/glamour v0.10.0    - Markdown rendering
// github.com/charmbracelet/bubbles v0.21.0    - Pre-built components
// github.com/charmbracelet/huh v0.7.0         - Interactive forms
```

#### TUI Features
- **Claude Code-Inspired Design**: Clean welcome screen with boxed layout and professional aesthetics
- **Contextual Footer System**: Dynamic help that adapts to user context - from simple hints to full shortcuts
- **Progressive Help Interface**: "? for shortcuts" → command autocomplete → full help panel
- **Clean Chat History**: Commands show results in footer/status area, keeping conversation pure
- **Real-time Streaming**: Live AI response rendering with smooth animations
- **Rich Formatting**: Markdown rendering with syntax highlighting and proper borders
- **Smart Automation**: Automatic thinking mode detection via keywords (think, analyze, plan, etc.)
- **Enhanced Command Autocomplete**: Real-time suggestions with descriptions as you type slash commands
- **Interactive Panels**: Context sidebar with session stats and available tools
- **Responsive Design**: Adapts seamlessly to terminal size with proper spacing
- **Intuitive Shortcuts**: Ctrl+H/? (help), Ctrl+T (context), Ctrl+C (exit), Shift+Enter (multiline)
- **Multiline Input Support**: Textarea component for complex queries and code snippets
- **Professional Styling**: Modern color schemes, typography, and spacing following CLI design best practices

### Storage Architecture

#### Directory Structure
```
.aircher/
├── config.toml              # TOML configuration
├── conversations.db         # Chat history with file references  
├── knowledge.db            # Project understanding & decisions
├── file_index.db           # File relationships & metadata
├── sessions.db             # Session management
├── mcp.json                # Local MCP server configuration
└── cache/
    ├── search/             # Web search results cache
    ├── snippets/           # File snippet cache
    └── providers/          # Provider response cache
```

#### Database Schemas

**conversations.db**
```sql
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    title TEXT,
    description TEXT,
    start_time DATETIME,
    end_time DATETIME,
    status TEXT CHECK(status IN ('active', 'completed', 'archived', 'compacted')),
    task_type TEXT,
    token_count INTEGER,
    provider TEXT,
    model TEXT,
    cost REAL,
    metadata JSON,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    role TEXT CHECK(role IN ('user', 'assistant', 'system', 'tool')),
    content TEXT,
    tool_calls JSON,
    tool_results JSON,
    timestamp DATETIME,
    metadata JSON,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

CREATE TABLE file_references (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL,
    file_path TEXT,
    start_line INTEGER DEFAULT 0,
    end_line INTEGER DEFAULT 0,
    content_snippet TEXT,
    context_description TEXT,
    content_hash TEXT,
    relevance_score REAL,
    FOREIGN KEY (message_id) REFERENCES messages(id)
);
```

**knowledge.db**
```sql
CREATE TABLE project_knowledge (
    id TEXT PRIMARY KEY,
    project_path TEXT UNIQUE NOT NULL,
    tech_stack JSON,
    architecture JSON,
    file_structure JSON,
    last_updated DATETIME,
    version TEXT
);

CREATE TABLE architectural_decisions (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    title TEXT,
    decision TEXT,
    rationale TEXT,
    context TEXT,
    affected_files JSON,
    date DATETIME,
    status TEXT CHECK(status IN ('active', 'deprecated', 'superseded')),
    FOREIGN KEY (project_id) REFERENCES project_knowledge(id)
);

CREATE TABLE code_patterns (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    pattern_name TEXT,
    description TEXT,
    example_code TEXT,
    usage_count INTEGER DEFAULT 1,
    last_seen DATETIME,
    FOREIGN KEY (project_id) REFERENCES project_knowledge(id)
);

CREATE TABLE memory_entries (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    content TEXT,
    memory_type TEXT CHECK(memory_type IN ('instructions', 'conventions', 'commands', 'architecture', 'glossary')),
    keywords JSON,
    created_at DATETIME,
    relevance_score REAL DEFAULT 1.0,
    source TEXT CHECK(source IN ('aircher_md', 'auto_generated', 'user_input')),
    FOREIGN KEY (project_id) REFERENCES project_knowledge(id)
);
```

**file_index.db**
```sql
CREATE TABLE file_index (
    id TEXT PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    content_hash TEXT,
    size INTEGER,
    mod_time DATETIME,
    language TEXT,
    last_analyzed DATETIME,
    current_relevance REAL DEFAULT 0.0
);

CREATE TABLE file_dependencies (
    id TEXT PRIMARY KEY,
    file_id TEXT NOT NULL,
    depends_on_id TEXT NOT NULL,
    dependency_type TEXT CHECK(dependency_type IN ('import', 'include', 'reference', 'call')),
    confidence REAL DEFAULT 1.0,
    FOREIGN KEY (file_id) REFERENCES file_index(id),
    FOREIGN KEY (depends_on_id) REFERENCES file_index(id)
);

CREATE TABLE symbols (
    id TEXT PRIMARY KEY,
    file_id TEXT NOT NULL,
    name TEXT,
    type TEXT CHECK(type IN ('function', 'class', 'variable', 'constant', 'type')),
    line_start INTEGER,
    line_end INTEGER,
    exported BOOLEAN DEFAULT FALSE,
    signature TEXT,
    FOREIGN KEY (file_id) REFERENCES file_index(id)
);
```

**sessions.db**
```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    title TEXT,
    description TEXT,
    start_time DATETIME,
    last_activity DATETIME,
    message_count INTEGER DEFAULT 0,
    tokens_used INTEGER DEFAULT 0,
    cost REAL DEFAULT 0.0,
    provider TEXT,
    model TEXT,
    working_dir TEXT,
    project_type TEXT,
    status TEXT CHECK(status IN ('active', 'paused', 'completed', 'archived')),
    tags JSON,
    bookmarks JSON
);
```

## Intelligent Context Management

### Task Detection System

```go
type TaskDetector struct {
    patterns      map[TaskType][]Pattern
    fileWatcher   *FileSystemWatcher
    gitWatcher    *GitStatusWatcher
    behaviorAnalyzer *UserBehaviorAnalyzer
}

type TaskType string

const (
    TaskDebugging      TaskType = "debugging"
    TaskFeature        TaskType = "feature"
    TaskRefactor       TaskType = "refactor"
    TaskDocumentation  TaskType = "documentation"
    TaskTesting        TaskType = "testing"
    TaskMaintenance    TaskType = "maintenance"
)

type Task struct {
    ID            string
    Type          TaskType
    StartTime     time.Time
    Status        TaskStatus
    RelevantFiles []string
    Dependencies  []string
    Description   string
    Keywords      []string
    CompletionCriteria []string
    Outcome       *TaskOutcome
}

func (td *TaskDetector) IdentifyCurrentTask(conversation []Message, fileChanges []FileChange, gitStatus *GitStatus) (*Task, error) {
    taskIndicators := map[TaskType][]string{
        TaskDebugging:     {"error", "bug", "fix", "broken", "issue", "crash", "exception"},
        TaskFeature:       {"add", "create", "implement", "new", "feature", "build"},
        TaskRefactor:      {"refactor", "improve", "cleanup", "optimize", "restructure"},
        TaskDocumentation: {"document", "comment", "readme", "docs", "explain"},
        TaskTesting:       {"test", "spec", "coverage", "mock", "unit", "integration"},
        TaskMaintenance:   {"update", "upgrade", "dependency", "security", "performance"},
    }
    
    // Analyze recent conversation for task keywords and patterns
    scores := make(map[TaskType]float64)
    for _, message := range conversation {
        content := strings.ToLower(message.Content)
        for taskType, keywords := range taskIndicators {
            for _, keyword := range keywords {
                if strings.Contains(content, keyword) {
                    scores[taskType] += 1.0
                }
            }
        }
    }
    
    // Factor in file changes and git activity
    if len(fileChanges) > 0 {
        for _, change := range fileChanges {
            if strings.Contains(change.Path, "test") {
                scores[TaskTesting] += 0.5
            }
            if strings.Contains(change.Path, "README") || strings.Contains(change.Path, "doc") {
                scores[TaskDocumentation] += 0.5
            }
        }
    }
    
    // Determine primary task
    var primaryTask TaskType
    var maxScore float64
    for taskType, score := range scores {
        if score > maxScore {
            maxScore = score
            primaryTask = taskType
        }
    }
    
    if maxScore == 0 {
        primaryTask = TaskFeature // Default assumption
    }
    
    return &Task{
        ID:          generateID(),
        Type:        primaryTask,
        StartTime:   time.Now(),
        Status:      TaskActive,
        Keywords:    taskIndicators[primaryTask],
        Description: fmt.Sprintf("Detected %s task based on conversation analysis", primaryTask),
    }, nil
}
```

### File Relevance Scoring

```go
type FileRelevanceEngine struct {
    dependencyGraph *DependencyGraph
    accessPatterns  *FileAccessTracker
    taskContext     *TaskContextAnalyzer
    relevanceScorer *RelevanceScorer
}

type FileRelevance struct {
    Path           string
    Score          float64              // 0.0-1.0 relevance to current task
    LastAccessed   time.Time
    AccessFrequency int
    Dependencies   []string
    RelevanceType  RelevanceType
    ExpiryTime     *time.Time
    ConfidenceScore float64
}

func (fre *FileRelevanceEngine) CalculateRelevance(task *Task, filePath string) *FileRelevance {
    score := 0.0
    
    // Direct relevance (0.8) - recently mentioned or modified
    if fre.isDirectlyRelevant(task, filePath) {
        score += 0.8
    }
    
    // Dependency relevance (0.6) - import/include relationships
    if deps := fre.getDependencyRelevance(task, filePath); deps > 0 {
        score += deps * 0.6
    }
    
    // Contextual relevance (0.4) - same module, similar patterns
    if contextual := fre.getContextualRelevance(task, filePath); contextual > 0 {
        score += contextual * 0.4
    }
    
    // Historical relevance (0.3) - helped with similar tasks before
    if historical := fre.getHistoricalRelevance(task, filePath); historical > 0 {
        score += historical * 0.3
    }
    
    // Apply time decay
    score *= fre.calculateTimeDecay(filePath)
    
    return &FileRelevance{
        Path:            filePath,
        Score:           math.Min(score, 1.0),
        LastAccessed:    time.Now(),
        RelevanceType:   fre.determineRelevanceType(score),
        ConfidenceScore: fre.calculateConfidence(score),
    }
}
```

### Smart Conversation Compaction

```go
type SmartCompactor struct {
    taskDetector     *TaskDetector
    summaryGenerator *ConversationSummarizer
    importanceScorer *MessageImportanceScorer
    preservationRules *PreservationRuleEngine
}

type CompactionTrigger struct {
    TaskCompletion    bool              // Task completed successfully
    ContextShift      bool              // Major context/topic change detected
    QualityDegradation bool             // Context quality declining
    TokenThreshold    bool              // Approaching provider token limit
    TimeThreshold     bool              // Long conversation without progress
    UserRequest       bool              // Manual /compact command
}

func (sc *SmartCompactor) ShouldCompact(conversation *Conversation, task *Task) (*CompactionTrigger, bool) {
    trigger := &CompactionTrigger{}
    
    // Primary triggers (quality-based)
    if task != nil && (task.Status == "completed" || sc.detectTaskCompletion(conversation)) {
        trigger.TaskCompletion = true
    }
    
    if sc.detectMajorContextShift(conversation) {
        trigger.ContextShift = true
    }
    
    if sc.detectQualityDegradation(conversation) {
        trigger.QualityDegradation = true
    }
    
    // Backup triggers
    if sc.approachingTokenLimit(conversation) {
        trigger.TokenThreshold = true
    }
    
    if sc.detectConversationStagnation(conversation) {
        trigger.TimeThreshold = true
    }
    
    shouldCompact := trigger.TaskCompletion || trigger.ContextShift || trigger.QualityDegradation ||
                     (trigger.TokenThreshold && trigger.TimeThreshold)
    
    return trigger, shouldCompact
}
```

## Intelligent Automation Features

### Automatic Thinking Detection

Aircher automatically detects when thinking mode should be enabled based on keywords in user messages, similar to Claude Code but more comprehensive:

```go
// detectThinkingKeywords checks if the input contains keywords that suggest thinking mode should be enabled
func (m Model) detectThinkingKeywords(input string) bool {
    thinkingKeywords := []string{
        "think", "thinking", "reason", "reasoning", "analyze", "analysis",
        "consider", "evaluate", "examine", "plan", "planning", "strategy",
        "approach", "methodology", "step by step", "walk through", "break down",
        "pros and cons", "trade-offs", "implications", "consequences",
    }
    
    lowerInput := strings.ToLower(input)
    for _, keyword := range thinkingKeywords {
        if strings.Contains(lowerInput, keyword) {
            return true
        }
    }
    return false
}

// Provider request includes thinking mode when detected
request := &providers.ChatRequest{
    Messages: messages,
    Model:    model,
    Stream:   true,
    Thinking: enableThinking, // Automatically set based on keywords
}
```

#### Thinking Mode Behavior
- **Automatic Detection**: No manual `/think` commands needed
- **Provider Compatibility**: Only enabled for providers/models that support it
- **Transparent Operation**: Users see thinking indicators in status bar
- **Keyword Triggers**: Natural language keywords automatically enable reasoning mode
- **Fallback Graceful**: Works seamlessly with non-thinking models

### Automatic Web Search Integration

Web search is handled automatically through MCP tools rather than manual commands:

```go
// Automatic search integration via MCP
type SearchCapability struct {
    MCPManager     *mcp.Manager
    SearchProviders []SearchProvider
    AutoSearch     bool
    Cache          *SearchCache
}

// Search is triggered automatically when:
// 1. Questions about current events, documentation, or APIs
// 2. Error messages that need recent solutions
// 3. Technology questions requiring up-to-date information
// 4. Explicit search intent detected in natural language
```

#### Search Integration Features
- **MCP-Based**: Uses Model Context Protocol for extensible search capabilities
- **Automatic Triggering**: No `/search` commands - triggered by context and intent
- **Multiple Providers**: Brave Search, Tavily, or custom providers via MCP
- **Intelligent Caching**: Avoids redundant searches with time-based cache
- **Cost Transparency**: Search costs tracked and displayed in status
- **Fallback Handling**: Graceful degradation when search unavailable

## Multi-Provider LLM System
</edits>

### Universal Provider Interface

```go
type LLMProvider interface {
    Chat(ctx context.Context, req *ChatRequest) (*ChatResponse, error)
    ChatStream(ctx context.Context, req *ChatRequest) (<-chan StreamChunk, error)
    SupportsFunctions() bool
    SupportsSystemMessages() bool
    SupportsImages() bool
    SupportsThinking() bool
    GetTokenLimit() int
    CountTokens(text string) int
    CalculateCost(tokens int) float64
    Name() string
    Models() []string
}

type ChatRequest struct {
    Messages    []Message
    Tools       []Tool
    MaxTokens   int
    Temperature float64
    Stream      bool
    Model       string
    Provider    string
}

type ChatResponse struct {
    Message     Message
    Stream      <-chan StreamChunk
    TokensUsed  int
    Cost        float64
    Duration    time.Duration
    Provider    string
    Model       string
    Metadata    map[string]interface{}
}
```

### Provider Implementations

#### Implementation Status: ✅ OpenAI & Claude Complete, 🚧 Gemini & Ollama Pending

```go
type OpenAIProvider struct {
    client      *openai.Client          // ✅ Client initialized
    model       string                  // ✅ Model configuration
    apiKey      string                  // ✅ API key management
    baseURL     string                  // ✅ Custom endpoint support
    costTable   map[string]CostInfo     // ✅ Complete cost tables
    rateLimiter *ProviderRateLimiter   // ✅ Rate limiting
    logger      zerolog.Logger         // ✅ Structured logging
    config      *config.OpenAIProviderConfig // ✅ Configuration
}

// Provider Status:
// ✅ OpenAI Provider - Complete with real API integration and streaming
// ✅ Claude Provider - Complete with Anthropic SDK, context caching, and streaming
// 🚧 Gemini Provider - Structure complete, API integration pending
// 🚧 Ollama Provider - Structure complete, API integration pending
```

func (p *OpenAIProvider) Chat(ctx context.Context, req *ChatRequest) (*ChatResponse, error) {
    oaiReq := openai.ChatCompletionRequest{
        Model:       req.Model,
        Messages:    convertMessagesToOpenAI(req.Messages),
        Tools:       convertToolsToOpenAI(req.Tools),
        MaxTokens:   req.MaxTokens,
        Temperature: float32(req.Temperature),
        Stream:      req.Stream,
    }
    
    start := time.Now()
    resp, err := p.client.CreateChatCompletion(ctx, oaiReq)
    if err != nil {
        return nil, err
    }
    
    tokens := resp.Usage.TotalTokens
    cost := p.CalculateCost(tokens)
    
    return &ChatResponse{
        Message:    convertFromOpenAI(resp.Choices[0].Message),
        TokensUsed: tokens,
        Cost:       cost,
        Duration:   time.Since(start),
        Provider:   "openai",
        Model:      req.Model,
    }, nil
}

// Claude Provider
type ClaudeProvider struct {
    client      *anthropic.Client
    model       string
    apiKey      string
    costTable   map[string]CostStructure
    rateLimiter *rate.Limiter
}

// Gemini Provider
type GeminiProvider struct {
    client    *genai.Client
    model     string
    apiKey    string
    project   string
    costTable map[string]CostStructure
}

// Ollama Provider
type OllamaProvider struct {
    baseURL     string
    model       string
    client      *http.Client
    keepAlive   time.Duration
}
```

### Provider Routing Engine

```go
type ProviderRoutingEngine struct {
    rules         []RoutingRule
    fallbacks     []string
    costOptimizer *CostOptimizer
    healthChecker *ProviderHealthChecker
}

type RoutingRule struct {
    Condition     RoutingCondition
    Provider      string
    Priority      int
    Explanation   string
}

func (pre *ProviderRoutingEngine) SelectProvider(req *ChatRequest, prefs *UserPreferences) (string, error) {
    candidates := []ProviderCandidate{}
    
    for providerName, provider := range pre.availableProviders {
        if !pre.meetsRequirements(provider, req) {
            continue
        }
        
        score := pre.calculateProviderScore(provider, req, prefs)
        candidates = append(candidates, ProviderCandidate{
            Name:     providerName,
            Provider: provider,
            Score:    score,
        })
    }
    
    if len(candidates) == 0 {
        return "", errors.New("no suitable providers available")
    }
    
    sort.Slice(candidates, func(i, j int) bool {
        return candidates[i].Score > candidates[j].Score
    })
    
    return candidates[0].Name, nil
}
```

## Autonomous Web Search System

### Temporal Search Engine

```go
type TemporalSearchEngine struct {
    currentTime     time.Time
    timezone        *time.Location
    searchProviders []SearchProvider
    decisionEngine  *SearchDecisionEngine
    cache          *TimeAwareCache
    fetcher        *ContentFetcher
}

type SearchDecisionEngine struct {
    temporalTriggers []TemporalPattern
    techTriggers     []TechnologyPattern
    errorTriggers    []ErrorPattern
    contextAnalyzer  *ConversationContextAnalyzer
}

func (tse *TemporalSearchEngine) ShouldSearch(message string, context *ConversationContext) (bool, []SearchQuery) {
    queries := []SearchQuery{}
    
    // Temporal indicators
    temporalKeywords := []string{"latest", "current", "recent", "new", "updated", "2024", "2025"}
    for _, keyword := range temporalKeywords {
        if strings.Contains(strings.ToLower(message), keyword) {
            queries = append(queries, tse.generateTemporalQuery(message, keyword))
        }
    }
    
    // Technology/framework mentions
    if techs := tse.detectTechnologies(message); len(techs) > 0 {
        for _, tech := range techs {
            queries = append(queries, tse.generateTechQuery(tech, message))
        }
    }
    
    // Error patterns
    if errors := tse.detectErrors(message); len(errors) > 0 {
        for _, errorPattern := range errors {
            queries = append(queries, tse.generateErrorQuery(errorPattern, context))
        }
    }
    
    return len(queries) > 0, queries
}

// Search Providers
type SearchProvider interface {
    Search(ctx context.Context, query string) (*SearchResults, error)
    Name() string
    RateLimit() time.Duration
    SupportsType(searchType SearchType) bool
}

type BraveSearchProvider struct {
    apiKey      string
    client      *http.Client
    rateLimiter *rate.Limiter
}
```

## Memory System (AIRCHER.md Integration)

### Implementation Status: ✅ OpenAI & Claude Complete, 🚧 Others Pending

### Architecture Overview

The memory system combines human-editable AIRCHER.md files with automatic database storage:

- **AIRCHER.md**: Human-editable project memory for user-specified knowledge
- **Database Storage**: Automatic handling of file indexes, conversation history, and learned patterns
- **Sync System**: Automatic synchronization between AIRCHER.md and knowledge.db

### What Goes Where

**AIRCHER.md (User-Specified)**:
- Programming language and framework versions
- Coding style guides and team conventions  
- Project-specific instructions and preferences
- Architecture decisions and patterns
- Frequently used commands
- Project-specific terminology and glossary

**Database (Automatic)**:
- File relationships and dependencies (file_index.db)
- Previous conversations and solutions (conversations.db)
- Learned code patterns and solutions (knowledge.db)
- File access patterns and relevance scores
- Task completion patterns and outcomes

**Auto-Generated Documentation**:
- Project structure analysis (`.aircher/project_analysis.md`)
- Documentation file discovery and cross-references
- Technology stack detection (languages, frameworks, build systems)
- Architecture pattern recognition (MVC, Clean Architecture, microservices)
- Project metadata extraction (names, versions, dependencies)

### Implementation

```go
type MemoryManager struct {
    projectMemory *ProjectMemoryFile      // AIRCHER.md in project root
    userMemory    *UserMemoryFiles        // ~/.aircher/memory/
    knowledgeDB   *ProjectKnowledgeDB     // Database backend
    fileWatcher   *fsnotify.Watcher       // Watch for AIRCHER.md changes
}

type ProjectMemoryFile struct {
    FilePath        string              // ./AIRCHER.md
    Instructions    []Instruction       // Team-shared instructions
    Conventions     []Convention        // Code style, patterns, naming
    Commands        []FrequentCommand   // Build, test, deploy, lint commands
    Architecture    []ArchitecturalNote // System design, patterns
    Glossary        []GlossaryEntry     // Project-specific terminology
    Dependencies    []DependencyNote    // Important package info
    LastModified    time.Time
    SyncedToDB      bool                // Whether changes are synced to DB
}

func (mm *MemoryManager) HandleMemoryInput(input string) error {
    if !strings.HasPrefix(input, "#") {
        return nil
    }
    
    content := strings.TrimSpace(input[1:])
    
    // Interactive memory type selection
    memoryTypes := []string{
        "instructions",    // How to work with this project
        "conventions",     // Code style and patterns
        "commands",        // Frequently used commands
        "architecture",    // System design notes
        "glossary",        // Project terminology
        "dependencies",    // Important package notes
    }
    
    selectedType := mm.promptForMemoryType(memoryTypes)
    
    // Add to AIRCHER.md and sync to database
    if err := mm.addToMemoryFile(content, selectedType); err != nil {
        return err
    }
    
    return mm.syncToDatabase()
}
```

### Project Analysis System

The automatic project analysis system discovers project structure, documentation, and technology stack without user intervention:

```go
type ProjectAnalyzer struct {
    projectRoot   string
    storageEngine *storage.Engine
    logger        zerolog.Logger
}

type DocumentationFile struct {
    FilePath     string    `json:"file_path"`
    DocType      string    `json:"doc_type"`      // 'readme', 'spec', 'api', 'guide'
    Title        string    `json:"title"`
    Purpose      string    `json:"purpose"`
    Sections     []string  `json:"sections"`      // Extracted headings
    CrossRefs    []string  `json:"cross_refs"`    // References to other docs
    LastAnalyzed time.Time `json:"last_analyzed"`
    ContentHash  string    `json:"content_hash"`  // For change detection
}

type ProjectComponent struct {
    Component   string                 `json:"component"`
    Type        string                 `json:"type"`        // 'language', 'framework', 'build', 'architecture'
    Description string                 `json:"description"`
    Confidence  float64                `json:"confidence"`  // 0.0 to 1.0
    Evidence    []string               `json:"evidence"`    // Supporting files/patterns
    Metadata    map[string]interface{} `json:"metadata"`
    LastUpdated time.Time              `json:"last_updated"`
}

type DocumentationGenerator struct {
    projectRoot string
    logger      zerolog.Logger
}

func (pa *ProjectAnalyzer) AnalyzeProject() (*AnalysisResult, error) {
    // Analyze documentation files
    docs := pa.analyzeDocumentation()
    
    // Detect languages and frameworks
    components := pa.analyzeProjectStructure()
    
    // Extract project metadata
    metadata := pa.extractProjectMetadata()
    
    // Store in enhanced knowledge database
    return &AnalysisResult{
        Documentation: docs,
        Components:    components,
        Metadata:      metadata,
        AnalyzedAt:    time.Now(),
    }, nil
}
```

#### Enhanced Knowledge Database Schema

```sql
-- Auto-discovered documentation files
CREATE TABLE documentation_analysis (
    file_path TEXT PRIMARY KEY,
    doc_type TEXT NOT NULL,
    title TEXT,
    purpose TEXT,
    sections TEXT,        -- JSON array of headings
    cross_refs TEXT,      -- JSON array of references
    last_analyzed INTEGER NOT NULL,
    content_hash TEXT     -- For change detection
);

-- Project components (languages, frameworks, architecture)
CREATE TABLE project_structure_analysis (
    component TEXT PRIMARY KEY,
    type TEXT NOT NULL,   -- 'language', 'framework', 'build', 'architecture'
    description TEXT NOT NULL,
    confidence REAL DEFAULT 0.0,
    evidence TEXT,        -- JSON array of supporting files
    metadata TEXT,        -- JSON object with additional info
    last_updated INTEGER NOT NULL
);

-- Key-value project metadata
CREATE TABLE project_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    confidence REAL DEFAULT 0.0,
    source TEXT,          -- Where this info came from
    category TEXT,        -- 'language', 'framework', 'build', 'structure'
    last_updated INTEGER NOT NULL
);
```

#### Auto-Generated Documentation

The system generates `.aircher/project_analysis.md` containing:

- **Project Overview**: Detected languages, frameworks, component summary
- **Project Structure**: Top-level directory layout
- **Documentation Files**: Discovered docs grouped by type with purposes and cross-references
- **Technology Stack**: Programming languages and frameworks with confidence levels
- **Architecture Patterns**: Detected patterns (MVC, Clean Architecture, microservices)
- **Build Systems**: Make, CMake, Gradle, Docker, etc.
- **Project Metadata**: Names, versions, dependencies extracted from config files

#### Integration with Context Engine

```go
func (ce *ContextEngine) GatherContext(task *Task) (*Context, error) {
    context := &Context{}
    
    // Load manual team knowledge from AIRCHER.md
    if teamMemory := ce.memoryManager.GetProjectMemory(); teamMemory != nil {
        context.Instructions = teamMemory.Instructions
        context.Conventions = teamMemory.Conventions
        context.Commands = teamMemory.Commands
    }
    
    // Load auto-discovered project knowledge
    if analysisPath := ce.memoryManager.GetProjectAnalysisPath(); fileExists(analysisPath) {
        context.ProjectAnalysis = readFile(analysisPath)
    }
    
    // Combine with file relevance and conversation history
    context.RelevantFiles = ce.fileEngine.GetRelevantFiles(task)
    context.ConversationHistory = ce.getRecentConversation()
    
    return context, nil
}
```

## Command System Architecture

### Slash Command Router

```go
type SlashCommandRouter struct {
    builtInCommands map[string]*BuiltInCommand
    projectCommands map[string]*CustomCommand    // .aircher/commands/
    userCommands    map[string]*CustomCommand    // ~/.aircher/commands/
    middleware      []CommandMiddleware
}

// Core slash commands
var coreCommands = map[string]*BuiltInCommand{
    "clear": {
        Name:        "clear",
        Description: "Clear conversation history",
        Handler:     handleClear,
    },
    "help": {
        Name:        "help", 
        Description: "Show available commands",
        Handler:     handleHelp,
    },
    "config": {
        Name:        "config",
        Description: "Configuration management",
        Handler:     handleConfig,
        Args:        []CommandArg{{Name: "key", Optional: true}, {Name: "value", Optional: true}},
    },
    "cost": {
        Name:        "cost",
        Description: "Show usage and cost statistics",
        Handler:     handleCost,
    },
    "memory": {
        Name:        "memory",
        Description: "Edit AIRCHER.md memory file",
        Handler:     handleMemory,
        Args:        []CommandArg{{Name: "action", Optional: true}},
    },
    "search": {
        Name:        "search",
        Description: "Force web search",
        Handler:     handleWebSearch,
        Args:        []CommandArg{{Name: "query", Required: true}},
    },
}

type CustomCommand struct {
    Name         string
    Scope        CommandScope             // project, user
    FilePath     string
    Content      string                   // Markdown template
    Arguments    []ArgumentPlaceholder    // Parsed $ARGUMENTS usage
    Metadata     CommandMetadata
}
```

## MCP (Model Context Protocol) Integration

### MCP Architecture

```go
type MCPManager struct {
    localServers   map[string]*MCPServer    // .aircher/mcp.json
    projectServers map[string]*MCPServer    // .mcp.json (version controlled)
    userServers    map[string]*MCPServer    // ~/.aircher/mcp.json
    client         *client.Client
    serverProcesses map[string]*exec.Cmd
    registry       *MCPRegistry             // Available MCP servers
    installer      *MCPInstaller            // Server installation manager
}

type MCPScope string

const (
    LocalScope   MCPScope = "local"      // Project-specific, user private
    ProjectScope MCPScope = "project"    // Shared via .mcp.json
    UserScope    MCPScope = "user"       // User global across all projects
)

type MCPServer struct {
    Name        string                   `json:"name"`
    Command     []string                 `json:"command"`
    Args        []string                 `json:"args,omitempty"`
    Env         map[string]string        `json:"env,omitempty"`
    Transport   MCPTransport             `json:"transport"`
    Scope       MCPScope                 `json:"scope"`
    Enabled     bool                     `json:"enabled"`
    Tools       []MCPTool                `json:"tools,omitempty"`
    Resources   []MCPResource            `json:"resources,omitempty"`
    Prompts     []MCPPrompt              `json:"prompts,omitempty"`
    LastSeen    time.Time                `json:"last_seen"`
    Category    MCPCategory              `json:"category"`
}

type MCPCategory string

const (
    CoreDevelopment MCPCategory = "core_dev"     // Git, filesystem, etc.
    WebTools        MCPCategory = "web"          // Search, fetch, scraping
    Database        MCPCategory = "database"     // PostgreSQL, MySQL, etc.
    DevEnvironment  MCPCategory = "dev_env"      // Docker, terminal, etc.
    Knowledge       MCPCategory = "knowledge"    // Memory, RAG, search
    Communication   MCPCategory = "comm"         // Slack, GitHub, etc.
)
```

### Priority MCP Servers

```go
// Core MCP servers that ship with Aircher
var CoreMCPServers = []MCPServerConfig{
    {
        Name:     "filesystem",
        Package:  "@modelcontextprotocol/server-filesystem",
        Category: CoreDevelopment,
        Description: "Secure file operations with configurable access controls",
        Required: true,
    },
    {
        Name:     "git",
        Package:  "mcp-server-git", 
        Category: CoreDevelopment,
        Description: "Git repository operations and history",
        Required: true,
    },
    {
        Name:     "github",
        Package:  "@modelcontextprotocol/server-github",
        Category: CoreDevelopment,
        Description: "GitHub API integration for PRs, issues, etc.",
        EnvVars:  []string{"GITHUB_PERSONAL_ACCESS_TOKEN"},
    },
    {
        Name:     "fetch",
        Package:  "@modelcontextprotocol/server-fetch",
        Category: WebTools,
        Description: "Web content fetching and markdown conversion",
        Required: true,
    },
    {
        Name:     "brave-search",
        Package:  "@modelcontextprotocol/server-brave-search",
        Category: WebTools,
        Description: "Web search with cost transparency",
        EnvVars:  []string{"BRAVE_API_KEY"},
    },
}

// Recommended MCP servers for development
var RecommendedMCPServers = []MCPServerConfig{
    // Databases
    {Name: "postgresql", Package: "@modelcontextprotocol/server-postgres", Category: Database},
    {Name: "sqlite", Package: "@modelcontextprotocol/server-sqlite", Category: Database},
    {Name: "mysql", Package: "mcp-server-mysql", Category: Database},
    {Name: "redis", Package: "@modelcontextprotocol/server-redis", Category: Database},
    
    // Development Environment
    {Name: "docker", Package: "mcp-server-docker", Category: DevEnvironment},
    {Name: "terminal", Package: "mcp-server-terminal-control", Category: DevEnvironment},
    {Name: "puppeteer", Package: "@modelcontextprotocol/server-puppeteer", Category: DevEnvironment},
    
    // Knowledge & Documentation
    {Name: "memory", Package: "@modelcontextprotocol/server-memory", Category: Knowledge},
    {Name: "sequential-thinking", Package: "@modelcontextprotocol/server-sequential-thinking", Category: Knowledge},
    
    // Communication & Project Management
    {Name: "slack", Package: "@modelcontextprotocol/server-slack", Category: Communication},
    {Name: "linear", Package: "mcp-server-linear", Category: Communication},
    {Name: "github-actions", Package: "mcp-server-github-actions", Category: Communication},
    
    // Additional Development Tools
    {Name: "gitlab", Package: "@modelcontextprotocol/server-gitlab", Category: CoreDevelopment},
    {Name: "sentry", Package: "@modelcontextprotocol/server-sentry", Category: DevEnvironment},
    {Name: "tavily", Package: "mcp-server-tavily", Category: WebTools},
}
```

### MCP Installation & Management

```go
type MCPInstaller struct {
    npmPath      string
    uvxPath      string
    pipPath      string
    cacheDir     string
    registry     *MCPRegistry
}

func (mi *MCPInstaller) InstallServer(server MCPServerConfig) error {
    switch server.Language {
    case "typescript":
        return mi.installNPMServer(server)
    case "python":
        return mi.installPythonServer(server)
    default:
        return fmt.Errorf("unsupported server language: %s", server.Language)
    }
}

func (mi *MCPInstaller) ListAvailable(category MCPCategory) []MCPServerInfo {
    // List available servers from registry filtered by category
    return mi.registry.GetByCategory(category)
}
```

### MCP Tool Security & Permissions

```go
type MCPPermissionSystem struct {
    rules           map[string]PermissionRule
    confirmations   map[string]bool
    allowedPaths    []string
    readOnlyPaths   []string
    deniedPaths     []string
}

type ToolPermission string

const (
    FileRead        ToolPermission = "file_read"
    FileWrite       ToolPermission = "file_write"
    FileDelete      ToolPermission = "file_delete"
    GitRead         ToolPermission = "git_read"
    GitWrite        ToolPermission = "git_write"
    GitPush         ToolPermission = "git_push"
    DatabaseRead    ToolPermission = "database_read"
    DatabaseWrite   ToolPermission = "database_write"
    NetworkRequest  ToolPermission = "network_request"
    ProcessExecute  ToolPermission = "process_execute"
)

func (mps *MCPPermissionSystem) CheckPermission(tool string, action ToolPermission, path string) (bool, error) {
    // Check if action requires user confirmation
    if mps.requiresConfirmation(tool, action) {
        if !mps.getUserConfirmation(tool, action, path) {
            return false, fmt.Errorf("user denied permission")
        }
    }
    
    // Check path-based permissions
    if action == FileWrite || action == FileDelete {
        if !mps.isPathAllowed(path) {
            return false, fmt.Errorf("path not allowed: %s", path)
        }
    }
    
    return true, nil
}

// Interactive permission prompt
func (mps *MCPPermissionSystem) getUserConfirmation(tool, action, target string) bool {
    prompt := fmt.Sprintf("\n⚠️  Tool '%s' wants to perform '%s' on '%s'\nAllow? (y/N): ", 
        tool, action, target)
    // Interactive prompt implementation
    return promptUser(prompt)
}
```

### MCP Tool Result Processing

```go
type MCPToolResultProcessor struct {
    formatters map[string]ResultFormatter
    cache      *ToolResultCache
}

func (trp *MCPToolResultProcessor) ProcessWebFetch(url string, content string) *ProcessedResult {
    // Convert HTML to clean markdown
    markdown := trp.htmlToMarkdown(content)
    
    // Extract code blocks
    codeBlocks := trp.extractCodeBlocks(markdown)
    
    // Create structured result
    return &ProcessedResult{
        Type:       "web_content",
        Source:     url,
        Content:    markdown,
        CodeBlocks: codeBlocks,
        Metadata: map[string]interface{}{
            "fetched_at": time.Now(),
            "size_bytes": len(content),
        },
    }
}

func (trp *MCPToolResultProcessor) ProcessSearchResults(results []SearchResult) *ProcessedResult {
    // Format search results for display
    formatted := trp.formatSearchResults(results)
    
    // Add cost information
    cost := trp.calculateSearchCost(len(results))
    
    return &ProcessedResult{
        Type:    "search_results",
        Content: formatted,
        Metadata: map[string]interface{}{
            "result_count": len(results),
            "estimated_cost": cost,
        },
    }
}
```

## Configuration System

### TOML Configuration Schema

```toml
[project]
name = "my-project"
path = "/path/to/project"
type = "go"

[interface]
mode = "auto"                        # "repl", "print", "auto"
output_format = "text"               # "text", "json", "stream-json"
vim_mode = false
color_theme = "dark"
show_thinking = true
show_token_count = true
show_cost = true

[providers]
default = "claude"

  [providers.openai]
  api_key_env = "OPENAI_API_KEY"
  model = "gpt-4-turbo"
  max_tokens = 4096
  
  [providers.claude]
  api_key_env = "ANTHROPIC_API_KEY"
  model = "claude-sonnet-4"
  max_tokens = 8192
  
  [providers.gemini]
  api_key_env = "GEMINI_API_KEY"
  model = "gemini-pro"
  project = "my-gcp-project"
  
  [providers.ollama]
  base_url = "http://localhost:11434"
  model = "llama2"
  keep_alive = "5m"

[context_management]
  [context_management.auto_compaction]
  enabled = true
  task_completion_trigger = true
  context_shift_trigger = true
  quality_degradation_trigger = true
  token_threshold = 8000
  preserve_messages = 10
  
  [context_management.file_relevance]
  max_files = 20
  threshold = 0.2
  include_dependencies = true
  historical_weight = 0.3
  decay_rate = 0.1

[search]
enabled = true
auto_search = true
providers = ["brave", "duckduckgo"]
brave_api_key_env = "BRAVE_API_KEY"
max_results = 5
cache_duration = "1h"

[memory]
project_file = "AIRCHER.md"
auto_save_decisions = true
sync_interval = "5m"

[costs]
monthly_budget = 50.0
daily_limit = 5.0
alert_threshold = 0.8
track_by_provider = true

[mcp]
timeout = "30s"
debug = false
auto_restart = true
auto_install = true
registry_url = "https://mcp-registry.aircher.dev"

  [mcp.permissions]
  filesystem_allowed_paths = ["/Users/nick/projects", "/tmp"]
  filesystem_readonly_paths = ["/etc", "/usr"]
  require_confirmation = ["file_delete", "git_push", "database_write"]
  
  [[mcp.servers]]
  name = "filesystem"
  command = ["npx", "-y", "@modelcontextprotocol/server-filesystem"]
  args = ["/Users/nick/projects"]
  transport = "stdio"
  scope = "local"
  enabled = true
  
  [[mcp.servers]]
  name = "git"
  command = ["uvx", "mcp-server-git"]
  args = ["--repository", "."]
  transport = "stdio"
  scope = "local"
  enabled = true
  
  [[mcp.servers]]
  name = "github"
  command = ["npx", "-y", "@modelcontextprotocol/server-github"]
  transport = "stdio"
  scope = "user"
  env = {"GITHUB_PERSONAL_ACCESS_TOKEN" = "${GITHUB_TOKEN}"}
  enabled = false

[security]
require_confirmation = ["delete_file", "execute_command", "git_push"]
sandbox_mode = true
max_file_size = "10MB"
allowed_extensions = [".go", ".py", ".js", ".ts", ".md", ".json", ".yaml", ".toml"]
```

## Current Implementation Status

### ✅ Completed Components (Production Ready)
- **Multi-Provider LLM System**: OpenAI and Claude fully integrated with real API calls
- **Claude Provider**: Complete Anthropic SDK integration with context caching and streaming
- **Provider Selection**: CLI flags for explicit provider choice (--provider claude/openai)
- **TUI Framework**: Complete Charmbracelet Bubble Tea implementation
- **CLI Interface**: Full Cobra-based command system with help
- **Configuration System**: TOML configuration with user/project scopes
- **Database Layer**: Complete SQLite multi-database architecture
- **Provider Framework**: Universal interface with OpenAI, Claude, Gemini, Ollama
- **MCP Integration**: Complete server management and configuration
- **Command System**: Enhanced slash commands with visual feedback
- **Session Management**: Conversation tracking and resumption
- **Styling System**: Professional Lipgloss theming throughout

### 🚧 Framework Complete (API Integration Pending)
- **Gemini Provider**: Google AI SDK integration needed
- **Ollama Provider**: Local model API integration needed
- **LLM Provider APIs**: Structures complete, actual API calls stubbed
- **Context Management**: Task detection and file relevance frameworks ready
- **Web Search**: Temporal search engine framework, provider APIs stubbed
- **Memory System**: AIRCHER.md integration framework, parsing pending
- **Smart Compaction**: Conversation analysis framework, algorithms stubbed

### ❌ Not Yet Implemented
- **Enhanced Context Caching**: Full utilization of Claude's context caching in long conversations
- **Actual LLM Streaming**: Real API calls with TUI streaming integration
- **File Analysis**: Intelligent file relevance scoring algorithms
- **Web Search APIs**: Brave/DuckDuckGo integration
- **MCP Tool Execution**: Real tool calling and result processing
- **Auto-Update System**: Self-update with rollback capability
- **Health Diagnostics**: Comprehensive system health checks

### 📊 Project Metrics
- **Total Code**: 5,634 lines of Go
- **Build Status**: ✅ Compiles and runs successfully
- **Test Coverage**: Framework in place, tests pending
- **Dependencies**: Modern, well-maintained packages
- **Architecture**: Clean, maintainable, extensible design

## Key Go Dependencies

### Core Framework
```go
"github.com/spf13/cobra"              // CLI framework
"github.com/BurntSushi/toml"          // TOML parsing (replaced Viper)
"github.com/mattn/go-sqlite3"         // SQLite database
"github.com/jmoiron/sqlx"             // Enhanced SQL operations
"github.com/rs/zerolog"               // Structured logging
```

### Charmbracelet TUI Stack
```go
"github.com/charmbracelet/bubbletea"  // TUI framework
"github.com/charmbracelet/lipgloss"   // Styling and layout
"github.com/charmbracelet/glamour"    // Markdown rendering
"github.com/charmbracelet/bubbles"    // UI components
"github.com/charmbracelet/huh"        // Interactive forms
```

### LLM Providers
```go
"github.com/sashabaranov/go-openai"   // OpenAI API
// Note: Claude, Gemini providers implemented without external SDKs
// Ollama integration via HTTP API
```

### MCP Integration
```go
// Custom MCP implementation - no external dependencies needed
"github.com/mark3labs/mcp-go/server"  // MCP server
"github.com/mark3labs/mcp-go/client"  // MCP client
```

### Terminal and UI
```go
"github.com/charmbracelet/bubbletea"  // TUI framework
"github.com/charmbracelet/lipgloss"   // Terminal styling
"github.com/charmbracelet/glamour"    // Markdown rendering
"github.com/chzyer/readline"          // Line editing
"golang.org/x/term"                   // Terminal control
```

### Web Search and HTTP
```go
"github.com/gocolly/colly/v2"         // Web scraping
"golang.org/x/net/html"               // HTML parsing
"github.com/PuerkitoBio/goquery"      // HTML querying
```

### File System and Git
```go
"github.com/fsnotify/fsnotify"        // File system notifications
"github.com/go-git/go-git/v5"         // Git operations
"github.com/bmatcuk/doublestar/v4"    // Glob patterns
```

### Utilities
```go
"golang.org/x/time/rate"              // Rate limiting
"github.com/google/uuid"              // UUID generation
"github.com/pkg/errors"               // Error handling
"golang.org/x/sync/errgroup"          // Error groups
"golang.org/x/crypto/blake2b"         // Hashing
```

### Auto-Update
```go
"github.com/rhymd/go-github-selfupdate/selfupdate"
"github.com/blang/semver"             // Semantic versioning
```

### Testing
```go
"github.com/stretchr/testify"         // Testing framework
"github.com/golang/mock"              // Mocking
"github.com/golangci/golangci-lint"   // Linting
```

This technical specification provides the detailed implementation guidance needed to build Aircher according to our architectural vision, with clear separation between user-editable knowledge (AIRCHER.md) and automatically managed data (databases).
