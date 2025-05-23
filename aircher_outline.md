# Aircher - API-Agnostic AI Coding Assistant

## Project Overview

**Aircher** is a next-generation command-line AI coding assistant designed to surpass tools like Claude Code through intelligent automation and superior context management. Unlike Claude Code's single-provider approach, Aircher works with any LLM provider (OpenAI, Claude, Gemini, Ollama, and custom endpoints), making it truly API-agnostic.

### Core Differentiators
- **Universal LLM Support**: Works with any OpenAI-compatible API or major provider
- **Autonomous Web Search**: Automatically fetches current documentation without user intervention
- **Intelligent Context Management**: Task-aware context that automatically maintains relevance
- **Advanced Temporal Awareness**: Understands current time and searches for up-to-date information
- **Smart Error Recovery**: Proactive error resolution with current solutions
- **Self-Updating**: Seamless in-place updates like Claude Code
- **Enterprise-Ready**: Self-hosted, air-gapped support, audit logging

### Architecture Philosophy
Aircher is built around three core principles:
1. **Intelligence over Configuration**: The tool should understand context and intent automatically
2. **Provider Agnostic**: Never lock users into a single LLM provider or service
3. **Proactive Assistance**: Anticipate needs rather than wait for explicit commands

## 1. Intelligent Context Management System

### Task-Aware Context Engine
```go
type ContextManager struct {
    taskDetector    *TaskDetector
    relevanceEngine *FileRelevanceEngine
    compactor       *IntelligentCompactor
    storage         *ConversationStorage
    retriever       *ContextRetriever
}

type Task struct {
    ID            string
    Type          TaskType              // "debugging", "feature", "refactor", "documentation"
    StartTime     time.Time
    Status        TaskStatus            // "active", "completed", "paused", "abandoned"
    RelevantFiles []string
    Dependencies  []string              // Other files that might become relevant
    Description   string
    Keywords      []string
    Outcome       *TaskOutcome
}

type TaskDetector struct {
    patterns      map[TaskType][]Pattern
    fileWatcher   *FileSystemWatcher
    gitWatcher    *GitStatusWatcher
    userBehavior  *BehaviorAnalyzer
}

func (td *TaskDetector) IdentifyCurrentTask(conversation []Message, fileChanges []FileChange) (*Task, error) {
    // Analyze conversation for task indicators
    patterns := []TaskPattern{
        {Type: "debugging", Keywords: []string{"error", "bug", "fix", "broken", "issue"}},
        {Type: "feature", Keywords: []string{"add", "create", "implement", "new"}},
        {Type: "refactor", Keywords: []string{"refactor", "improve", "cleanup", "optimize"}},
        {Type: "documentation", Keywords: []string{"document", "comment", "readme", "docs"}},
        {Type: "testing", Keywords: []string{"test", "spec", "coverage", "mock"}},
    }
    
    // Combine conversation analysis with file change patterns
    // Recent git commits and file modifications
    // User's stated intentions
    
    return detectedTask, nil
}
```

### Dynamic File Relevance System
```go
type FileRelevanceEngine struct {
    dependencyGraph *DependencyGraph
    accessPatterns  *FileAccessTracker
    taskContext     *TaskContext
    relevanceScorer *RelevanceScorer
}

type FileRelevance struct {
    Path           string
    Score          float64              // 0-1 relevance to current task
    LastAccessed   time.Time
    AccessFreq     int
    Dependencies   []string
    RelevanceType  RelevanceType        // "direct", "dependency", "related", "historical"
    ExpiryTime     *time.Time           // When this relevance expires
}

type RelevanceType string

const (
    Direct      RelevanceType = "direct"      // Directly mentioned/modified
    Dependency  RelevanceType = "dependency"  // Import/include relationship
    Related     RelevanceType = "related"     // Similar functionality/module
    Historical  RelevanceType = "historical"  // Previously relevant to similar tasks
    Contextual  RelevanceType = "contextual"  // Provides helpful context
)

func (fre *FileRelevanceEngine) EvaluateFileRelevance(task *Task, file string) *FileRelevance {
    score := 0.0
    
    // Direct relevance (file mentioned in conversation or recently modified)
    if fre.isDirectlyRelevant(task, file) {
        score += 0.8
    }
    
    // Dependency relevance (imports, includes, references)
    if fre.isDependencyRelevant(task, file) {
        score += 0.6
    }
    
    // Contextual relevance (same module, similar patterns)
    if fre.isContextuallyRelevant(task, file) {
        score += 0.3
    }
    
    // Historical relevance (helped with similar tasks before)
    if fre.hasHistoricalRelevance(task, file) {
        score += 0.4
    }
    
    // Decay score based on time since last access
    score *= fre.calculateTimeDecay(file)
    
    return &FileRelevance{
        Path:  file,
        Score: math.Min(score, 1.0),
        // ... other fields
    }
}
```

### Intelligent Conversation Compactor
```go
type IntelligentCompactor struct {
    taskDetector     *TaskDetector
    summaryGenerator *SummaryGenerator
    importanceScorer *MessageImportanceScorer
    preservationRules *PreservationRules
}

type CompactionTrigger struct {
    TaskCompletion    bool              // Task was completed
    ContextShift      bool              // Major context shift detected
    TokenThreshold    bool              // Approaching token limit
    TimeThreshold     bool              // Long conversation duration
    UserRequest       bool              // Manual compaction request
}

type ConversationSummary struct {
    TaskSummary       string            // What was accomplished
    KeyDecisions      []Decision        // Important architectural/implementation decisions
    RemainingWork     []string          // Incomplete tasks or TODOs
    RelevantFiles     []string          // Files that remain relevant
    LessonsLearned    []string          // Patterns that worked/didn't work
    Context           map[string]string // Key context to preserve
    Timestamp         time.Time
    CompactionReason  string
}

func (ic *IntelligentCompactor) ShouldCompact(conversation *Conversation, task *Task) (*CompactionTrigger, bool) {
    trigger := &CompactionTrigger{}
    
    // Task completion detection
    if task.Status == "completed" || ic.detectTaskCompletion(conversation) {
        trigger.TaskCompletion = true
    }
    
    // Context shift detection (user switches to different task/project)
    if ic.detectContextShift(conversation) {
        trigger.ContextShift = true
    }
    
    // Token limit approaching (but not the primary trigger)
    if ic.approachingTokenLimit(conversation) {
        trigger.TokenThreshold = true
    }
    
    // Long conversation without progress
    if ic.detectStagnation(conversation) {
        trigger.TimeThreshold = true
    }
    
    shouldCompact := trigger.TaskCompletion || trigger.ContextShift || 
                     (trigger.TokenThreshold && trigger.TimeThreshold)
    
    return trigger, shouldCompact
}

func (ic *IntelligentCompactor) CompactConversation(conversation *Conversation, task *Task) (*ConversationSummary, []Message, error) {
    // Generate comprehensive summary preserving key information
    summary := ic.generateTaskSummary(conversation, task)
    
    // Identify messages to preserve (recent context, important decisions, errors)
    preservedMessages := ic.selectMessagesToPreserve(conversation.Messages)
    
    // Create new conversation starting with summary
    newConversation := []Message{
        {
            Role: "system",
            Content: ic.formatSummaryAsContext(summary),
            Metadata: map[string]interface{}{
                "type": "compaction_summary",
                "task_id": task.ID,
                "compaction_time": time.Now(),
            },
        },
    }
    
    // Append preserved messages
    newConversation = append(newConversation, preservedMessages...)
    
    return summary, newConversation, nil
}
```

### Multi-Database Storage Architecture
```go
type StorageManager struct {
    conversationDB *ConversationDB    // Conversation history
    knowledgeDB    *ProjectKnowledgeDB // Project understanding
    fileIndexDB    *FileIndexDB       // File relationships and metadata
    configManager  *ConfigManager     // TOML configuration
    basePath       string             // ".aircher" directory
}

// Conversation Database - stores actual conversations
type ConversationDB struct {
    db *sql.DB
}

type StoredConversation struct {
    ID            string
    Title         string               // Auto-generated descriptive title
    Description   string               // Brief description of what was accomplished
    StartTime     time.Time
    EndTime       *time.Time
    Status        ConversationStatus   // "active", "completed", "archived"
    TaskType      string
    Messages      []MessageRef         // References to messages with file snippets
    Summary       *ConversationSummary // Compacted summary if available
    RelevantFiles []FileReference      // File refs instead of full content
    Keywords      []string             // For search/retrieval
    TokenCount    int
    Provider      string
    Metadata      map[string]interface{}
}

// Message with efficient file references
type MessageRef struct {
    Role        string
    Content     string
    FileRefs    []FileSnippet        // Snippets instead of full files
    ToolCalls   []ToolCall
    Timestamp   time.Time
}

type FileSnippet struct {
    Path        string
    StartLine   int                   // 0 = whole file
    EndLine     int                   // 0 = whole file  
    Content     string                // Actual snippet content
    Context     string                // Brief context about why this snippet matters
    Hash        string                // File content hash for change detection
}

// Project Knowledge Database - persistent project understanding
type ProjectKnowledgeDB struct {
    db *sql.DB
}

type ProjectKnowledge struct {
    ProjectPath     string
    TechStack       TechStackInfo
    Architecture    ArchitectureInfo
    FileStructure   *CompactFileTree
    KeyDecisions    []ArchitecturalDecision
    CommonPatterns  []CodePattern
    Dependencies    []DependencyInfo
    BuildSystem     BuildSystemInfo
    TestingSetup    TestingInfo
    LastUpdated     time.Time
}

type TechStackInfo struct {
    Languages       []LanguageInfo       // Go 1.21, TypeScript 5.0, etc.
    Frameworks      []FrameworkInfo      // React 18.2, Echo v4, etc.
    Databases       []DatabaseInfo       // PostgreSQL 15, Redis 7, etc.
    Infrastructure  []InfraInfo          // Docker, K8s, AWS, etc.
    Tools           []ToolInfo           // Make, npm, cargo, etc.
}

type ArchitecturalDecision struct {
    ID          string
    Title       string                   // "Chose PostgreSQL over MongoDB"
    Decision    string                   // What was decided
    Rationale   string                   // Why it was decided
    Context     string                   // What problem it solved
    Files       []string                 // Files affected by this decision
    Date        time.Time
    Status      string                   // "active", "deprecated", "superseded"
}

// File Index Database - file relationships and metadata
type FileIndexDB struct {
    db *sql.DB
}

type FileIndex struct {
    Path            string
    Hash            string               // Content hash
    Size            int64
    ModTime         time.Time
    Language        string
    Dependencies    []string             // Files this depends on
    Dependents      []string             // Files that depend on this
    Exports         []Symbol             // Functions, classes, etc. exported
    Imports         []Import             // What this file imports
    LastAnalyzed    time.Time
    RelevanceScore  float64              // Current relevance to active tasks
}

// Compact file tree representation
type CompactFileTree struct {
    Root     *TreeNode
    Metadata TreeMetadata
}

type TreeNode struct {
    Name        string
    Type        string                   // "file", "dir"
    Children    []*TreeNode             // Only for directories
    Metadata    *FileMetadata           // Only for files
}

type FileMetadata struct {
    Language    string
    Size        int64
    ModTime     time.Time
    Purpose     string                   // "config", "test", "source", "docs"
    Importance  int                      // 1-5 relevance score
}
```

### Efficient Token Management
```go
type TokenOptimizer struct {
    fileSnippetizer *FileSnippetizer
    contextPrioritizer *ContextPrioritizer
    tokenCounter    map[string]TokenCounter  // Per-provider counters
}

type FileSnippetizer struct {
    languageAnalyzers map[string]LanguageAnalyzer
}

func (fs *FileSnippetizer) ExtractRelevantSnippets(filePath string, context string, maxTokens int) ([]FileSnippet, error) {
    // Parse file and extract relevant functions/classes/sections
    // Use AST analysis for structured languages
    // Use heuristics for unstructured files
    // Prioritize recently modified sections
    // Include minimal context for understanding
    
    analyzer := fs.languageAnalyzers[detectLanguage(filePath)]
    if analyzer == nil {
        return fs.extractSimpleSnippets(filePath, context, maxTokens)
    }
    
    return analyzer.ExtractSmartSnippets(filePath, context, maxTokens)
}

type ConversationCleaner struct {
    knowledgeDB     *ProjectKnowledgeDB
    fileIndexDB     *FileIndexDB
    contextBuilder  *SmartContextBuilder
}

func (cc *ConversationCleaner) ClearAndRebuild(reason string) (*FreshContext, error) {
    // 1. Save current conversation state to knowledge DB
    cc.updateProjectKnowledge()
    
    // 2. Detect current task/intent
    currentTask := cc.detectCurrentTask()
    
    // 3. Build fresh context from knowledge DB
    freshContext := &FreshContext{
        ProjectSummary:   cc.knowledgeDB.GetProjectSummary(),
        RelevantDecisions: cc.knowledgeDB.GetRelevantDecisions(currentTask),
        CurrentFileTree:  cc.buildCompactFileTree(),
        RelevantSnippets: cc.getRelevantCodeSnippets(currentTask),
        TodoList:        cc.getCurrentTodos(),
        TechStack:       cc.knowledgeDB.GetTechStack(),
    }
    
    return freshContext, nil
}
```

### Todo Management System (inspired by Claude Code)
```go
type TodoManager struct {
    db          *sql.DB
    currentTask *Task
    notifier    *TodoNotifier
}

type Todo struct {
    ID          string
    ParentID    *string              // For subtasks
    TaskID      string               // Associated task
    Title       string
    Description string
    Status      TodoStatus           // "pending", "in_progress", "completed", "cancelled"
    Priority    int                  // 1-5
    CreatedAt   time.Time
    UpdatedAt   time.Time
    CompletedAt *time.Time
    EstimatedEffort string           // "5m", "30m", "2h"
    ActualEffort    *time.Duration
    Files       []string             // Files related to this todo
    Dependencies []string            // Other todo IDs this depends on
    AutoGenerated bool               // Created by AI vs user
}

type TodoStatus string

const (
    TodoPending     TodoStatus = "pending"
    TodoInProgress  TodoStatus = "in_progress"
    TodoCompleted   TodoStatus = "completed"
    TodoCancelled   TodoStatus = "cancelled"
)

func (tm *TodoManager) CreateTodo(title, description string, priority int) (*Todo, error) {
    todo := &Todo{
        ID:          generateID(),
        TaskID:      tm.currentTask.ID,
        Title:       title,
        Description: description,
        Priority:    priority,
        Status:      TodoPending,
        CreatedAt:   time.Now(),
        UpdatedAt:   time.Now(),
        AutoGenerated: false,
    }
    
    return todo, tm.saveTodo(todo)
}

func (tm *TodoManager) GetActiveTodos() ([]*Todo, error) {
    // Get todos for current task that aren't completed
    // Sort by priority and dependencies
    // Show progress indicators
}

func (tm *TodoManager) AutoGenerateTodos(taskDescription string, analysis *CodeAnalysis) ([]*Todo, error) {
    // AI generates todos based on task and code analysis
    // Breaks down complex tasks into subtasks
    // Estimates effort for each todo
    // Identifies file dependencies
}

// Display todos in terminal UI
func (tm *TodoManager) DisplayTodos() string {
    todos, _ := tm.GetActiveTodos()
    
    var display strings.Builder
    display.WriteString("📋 Current Todos:\n")
    
    for _, todo := range todos {
        status := "⏳"
        if todo.Status == TodoCompleted {
            status = "✅"
        } else if todo.Status == TodoInProgress {
            status = "🔄"
        }
        
        display.WriteString(fmt.Sprintf("  %s %s", status, todo.Title))
        if todo.EstimatedEffort != "" {
            display.WriteString(fmt.Sprintf(" (%s)", todo.EstimatedEffort))
        }
        display.WriteString("\n")
        
        if todo.Description != "" {
            display.WriteString(fmt.Sprintf("     %s\n", todo.Description))
        }
    }
    
    return display.String()
}
```

### TOML Configuration (Better than YAML)
```go
// config.toml - More readable and supports comments better than YAML
[project]
name = "my-project"
path = "/path/to/project"

[context_management]
  [context_management.auto_compaction]
  enabled = true
  task_completion_trigger = true
  token_threshold = 8000
  time_threshold = "2h"
  preserve_messages = 10
  summary_detail = "medium"  # "brief", "medium", "detailed"
  
  [context_management.file_relevance]
  max_files = 20
  threshold = 0.2
  include_dependencies = true
  historical_weight = 0.3
  decay_rate = 0.1
  
  [context_management.retrieval]
  search_history = true
  similarity_threshold = 0.4
  max_historical_contexts = 3

[providers]
  [providers.openai]
  api_key_env = "OPENAI_API_KEY"
  model = "gpt-4-turbo"
  max_tokens = 4096
  
  [providers.claude]
  api_key_env = "ANTHROPIC_API_KEY"
  model = "claude-sonnet-4"
  max_tokens = 8192
  
  [providers.ollama]
  base_url = "http://localhost:11434"
  model = "llama2"

[search]
enabled = true
auto_search = true
providers = ["brave", "duckduckgo"]
max_results = 5

[ui]
vim_bindings = false
auto_accept = false
color_theme = "dark"  # "dark", "light", "ansi"
show_token_count = true
show_cost = true

[memory]
# Memory system like Claude Code - messages starting with '#' go to memory
enabled = true
max_entries = 100
auto_save_decisions = true

[mcp]
timeout = "30s"
debug = false

  [[mcp.servers]]
  name = "filesystem"
  command = ["aircher-fs-server"]
  transport = "stdio"
  
  [[mcp.servers]]
  name = "git"  
  command = ["aircher-git-server"]
  transport = "stdio"

type ConfigManager struct {
    configPath   string
    config       *Config
    watchers     []ConfigWatcher
}

func LoadConfig(projectPath string) (*Config, error) {
    configPath := filepath.Join(projectPath, ".aircher", "config.toml")
    
    // Load with defaults
    config := &Config{}
    if err := setDefaults(config); err != nil {
        return nil, err
    }
    
    // Override with file config if exists
    if _, err := os.Stat(configPath); err == nil {
        if err := toml.DecodeFile(configPath, config); err != nil {
            return nil, err
        }
    }
    
    return config, nil
}
```

### Efficient File Tree Management
```go
type FileTreeManager struct {
    indexDB      *FileIndexDB
    watcher      *fsnotify.Watcher
    lastSnapshot *FileTreeSnapshot
    gitIgnore    *GitIgnoreRules
}

type FileTreeSnapshot struct {
    Timestamp    time.Time
    Root         *TreeNode
    TotalFiles   int
    TotalSize    int64
    Hash         string               // Hash of entire tree structure
}

// Instead of storing full file tree, we store a compact representation
// and rebuild on demand with caching
func (ftm *FileTreeManager) GetCompactTree(maxDepth int, relevanceFilter float64) (*CompactFileTree, error) {
    // Check if cached version is still valid
    if ftm.isCacheValid() {
        return ftm.getCachedTree(maxDepth, relevanceFilter), nil
    }
    
    // Rebuild tree with smart filtering
    tree := &CompactFileTree{}
    
    // Walk file system but filter based on:
    // 1. .gitignore rules
    // 2. File relevance scores
    // 3. File size limits
    // 4. Known unimportant directories (node_modules, target, etc.)
    
    err := filepath.WalkDir(ftm.projectRoot, func(path string, d fs.DirEntry, err error) error {
        if err != nil {
            return err
        }
        
        // Skip ignored files/directories
        if ftm.gitIgnore.ShouldIgnore(path) {
            if d.IsDir() {
                return filepath.SkipDir
            }
            return nil
        }
        
        // Skip low relevance files unless specifically requested
        if relevance := ftm.getFileRelevance(path); relevance < relevanceFilter {
            return nil
        }
        
        // Add to tree with metadata
        ftm.addToTree(tree, path, d)
        return nil
    })
    
    if err != nil {
        return nil, err
    }
    
    // Cache the result
    ftm.cacheTree(tree)
    return tree, nil
}

// Smart file tree for context - only include relevant parts
func (ftm *FileTreeManager) GetContextTree(currentTask *Task, maxTokens int) (string, error) {
    tree, err := ftm.GetCompactTree(3, 0.1) // Max depth 3, min relevance 0.1
    if err != nil {
        return "", err
    }
    
    // Format for LLM context with token budget
    return ftm.formatTreeForContext(tree, currentTask, maxTokens), nil
}

func (ftm *FileTreeManager) formatTreeForContext(tree *CompactFileTree, task *Task, maxTokens int) string {
    var output strings.Builder
    
    output.WriteString("📁 Project Structure:\n")
    
    // Prioritize showing files relevant to current task
    relevantFiles := ftm.getTaskRelevantFiles(task)
    
    // Show directory structure with emphasis on relevant files
    ftm.formatNode(tree.Root, &output, 0, relevantFiles, maxTokens)
    
    return output.String()
}
```

### Context Limit Handling with Smart Compaction
```go
type ContextLimitManager struct {
    tokenCounters   map[string]TokenCounter
    compactor      *IntelligentCompactor
    limitsConfig   *ContextLimits
}

type ContextLimits struct {
    SoftLimit      int                  // Start warning/optimization at this point
    HardLimit      int                  // Must compact at this point
    PreservePct    float64              // % of context to preserve after compaction
    EmergencyMode  bool                 // Ultra-aggressive compaction
}

func (clm *ContextLimitManager) CheckContextLimits(conversation *Conversation, provider string) (*ContextAction, error) {
    tokenCount := clm.tokenCounters[provider].CountTokens(conversation.Messages)
    limit := clm.getProviderLimit(provider)
    
    softLimit := int(float64(limit) * 0.75)  // 75% soft limit
    hardLimit := int(float64(limit) * 0.90)  // 90% hard limit
    
    if tokenCount < softLimit {
        return &ContextAction{Type: "none"}, nil
    }
    
    if tokenCount < hardLimit {
        return &ContextAction{
            Type: "optimize",
            Reason: "Approaching token limit",
            Urgency: "medium",
            SuggestedAction: "smart_compact",
        }, nil
    }
    
    return &ContextAction{
        Type: "compact",
        Reason: "Token limit exceeded", 
        Urgency: "high",
        SuggestedAction: "emergency_compact",
    }, nil
}

func (clm *ContextLimitManager) SmartCompact(conversation *Conversation, urgency string) (*CompactedConversation, error) {
    if urgency == "high" {
        // Emergency compaction - very aggressive
        return clm.compactor.EmergencyCompact(conversation)
    }
    
    // Regular smart compaction based on task completion
    return clm.compactor.TaskAwareCompact(conversation)
}
```

### Claude Code-Inspired Features
```go
// Memory system - messages starting with '#' are saved to memory
type MemoryManager struct {
    db *sql.DB
}

type MemoryEntry struct {
    ID          string
    Content     string
    Keywords    []string
    ProjectPath string
    CreatedAt   time.Time
    Relevance   float64
}

func (mm *MemoryManager) SaveToMemory(content string) error {
    if !strings.HasPrefix(content, "#") {
        return nil // Not a memory message
    }
    
    // Extract content without '#' prefix
    memoryContent := strings.TrimSpace(content[1:])
    
    entry := &MemoryEntry{
        ID:          generateID(),
        Content:     memoryContent,
        Keywords:    extractKeywords(memoryContent),
        ProjectPath: mm.getCurrentProjectPath(),
        CreatedAt:   time.Now(),
        Relevance:   1.0,
    }
    
    return mm.saveMemoryEntry(entry)
}

// @-mention system for files
type MentionSystem struct {
    fileResolver *FileResolver
    contextBuilder *ContextBuilder
}

func (ms *MentionSystem) ProcessMentions(message string) (string, []FileReference, error) {
    // Find @mentions in message
    mentions := findMentions(message) // @src/main.go, @docs/, etc.
    
    fileRefs := []FileReference{}
    processedMessage := message
    
    for _, mention := range mentions {
        fileRef, err := ms.fileResolver.ResolveMention(mention)
        if err != nil {
            continue
        }
        
        fileRefs = append(fileRefs, fileRef)
        
        // Replace mention with context reference
        processedMessage = strings.Replace(processedMessage, mention, fmt.Sprintf("[File: %s]", fileRef.Path), 1)
    }
    
    return processedMessage, fileRefs, nil
}

// Thinking mode triggers
type ThinkingModeDetector struct {
    triggers []string
}

func (tmd *ThinkingModeDetector) ShouldEnableThinking(message string) bool {
    triggers := []string{
        "think", "think harder", "ultrathink", "reasoning", "analyze", "plan", "strategy"
    }
    
    messageLower := strings.ToLower(message)
    for _, trigger := range triggers {
        if strings.Contains(messageLower, trigger) {
            return true
        }
    }
    
    return false
}
```
```

### Smart Context Retrieval
```go
type ContextRetriever struct {
    storage          *ConversationStorage
    similarityEngine *ConversationSimilarity
    relevanceScorer  *HistoricalRelevanceScorer
}

type RetrievalContext struct {
    CurrentTask      *Task
    RecentFiles      []string
    ErrorContext     *ErrorContext
    Keywords         []string
    TimeWindow       time.Duration
}

func (cr *ContextRetriever) RetrieveRelevantContext(ctx *RetrievalContext) (*RetrievedContext, error) {
    // Search for historically relevant conversations
    searches := []SearchQuery{
        {Type: "similar_task", TaskType: ctx.CurrentTask.Type, Limit: 3},
        {Type: "file_overlap", Files: ctx.RecentFiles, Limit: 2},
        {Type: "error_pattern", Error: ctx.ErrorContext, Limit: 2},
        {Type: "keyword_match", Keywords: ctx.Keywords, Limit: 5},
    }
    
    relevantConversations := []RelevantConversation{}
    
    for _, search := range searches {
        conversations, err := cr.storage.SearchConversations(search)
        if err != nil {
            continue
        }
        
        for _, conv := range conversations {
            relevanceScore := cr.calculateRelevanceScore(conv, ctx)
            if relevanceScore > 0.3 { // Threshold for inclusion
                relevantConversations = append(relevantConversations, RelevantConversation{
                    Conversation: conv,
                    Relevance:   relevanceScore,
                    Context:     cr.extractRelevantContext(conv, ctx),
                })
            }
        }
    }
    
    return cr.synthesizeRetrievedContext(relevantConversations), nil
}
```

### Configuration & TUI Settings
```go
type ContextSettings struct {
    AutoCompaction      AutoCompactionSettings
    FileRelevance      FileRelevanceSettings
    Storage            StorageSettings
    Retrieval          RetrievalSettings
}

type AutoCompactionSettings struct {
    Enabled                bool          `yaml:"enabled" default:"true"`
    TaskCompletionTrigger  bool          `yaml:"task_completion" default:"true"`
    TokenThreshold         int           `yaml:"token_threshold" default:"8000"`
    TimeThreshold          time.Duration `yaml:"time_threshold" default:"2h"`
    PreserveMessageCount   int           `yaml:"preserve_messages" default:"10"`
    SummaryDetailLevel     string        `yaml:"summary_detail" default:"medium"` // "brief", "medium", "detailed"
}

type FileRelevanceSettings struct {
    MaxRelevantFiles       int           `yaml:"max_files" default:"20"`
    RelevanceThreshold     float64       `yaml:"threshold" default:"0.2"`
    IncludeDependencies    bool          `yaml:"include_deps" default:"true"`
    HistoricalWeight       float64       `yaml:"historical_weight" default:"0.3"`
    TimeDecayRate          float64       `yaml:"decay_rate" default:"0.1"`
}

type ConfigTUI struct {
    app         *tview.Application
    settings    *ContextSettings
    configPath  string
}

func (ct *ConfigTUI) ShowSettingsInterface() error {
    // Create interactive TUI similar to Claude Code's settings
    // Organized sections: Context Management, File Relevance, Storage, etc.
    // Real-time preview of how settings affect current conversation
    // Validation and helpful descriptions for each setting
}
```

### Context Flow Example
```go
// Typical workflow with intelligent context management
func (cm *ContextManager) HandleUserMessage(message string) (*Response, error) {
    // 1. Detect current task and context
    currentTask := cm.taskDetector.GetCurrentTask()
    
    // 2. Evaluate file relevance and update context
    relevantFiles := cm.relevanceEngine.GetRelevantFiles(currentTask)
    
    // 3. Check if compaction is needed
    if trigger, shouldCompact := cm.compactor.ShouldCompact(cm.currentConversation, currentTask); shouldCompact {
        summary, newConversation, err := cm.compactor.CompactConversation(cm.currentConversation, currentTask)
        if err == nil {
            // Store old conversation
            cm.storage.SaveConversation(cm.currentConversation)
            // Start fresh with summary
            cm.currentConversation = newConversation
        }
    }
    
    // 4. Retrieve relevant historical context if needed
    if cm.needsHistoricalContext(message) {
        historicalContext := cm.retriever.RetrieveRelevantContext(&RetrievalContext{
            CurrentTask: currentTask,
            RecentFiles: relevantFiles,
            Keywords:    extractKeywords(message),
        })
        // Inject relevant historical context
    }
    
    // 5. Build optimized context for LLM
    optimizedContext := cm.buildOptimizedContext(relevantFiles, message)
    
    // 6. Send to LLM and get response
    response := cm.llmProvider.Chat(optimizedContext)
    
    // 7. Update task status and file relevance based on response
    cm.updateTaskProgress(currentTask, response)
    
    return response, nil
}
```

## 2. System Prompt & Context Management

### Dynamic System Prompt
```go
type SystemPromptBuilder struct {
    basePrompt      string
    capabilities    []string
    workspaceInfo   *WorkspaceContext
    currentTime     time.Time
    userPreferences *UserConfig
}

func (spb *SystemPromptBuilder) BuildSystemPrompt(ctx context.Context) string {
    prompt := fmt.Sprintf(`You are an advanced AI coding assistant with access to:

CURRENT CONTEXT:
- Date/Time: %s
- Working Directory: %s
- Project Type: %s
- Git Status: %s
- Recent Changes: %s

AVAILABLE TOOLS:
%s

CAPABILITIES:
- File system operations (read, write, create, delete, search)
- Web search for current documentation and solutions
- Git operations (commit, push, pull, merge, branch management)
- Test execution and generation
- Code analysis and refactoring
- Package management and dependency updates

BEHAVIOR GUIDELINES:
- Always search the web for current documentation when dealing with:
  * Framework/library questions (check latest versions)
  * Error messages (find recent solutions)
  * API changes or deprecations
  * Best practices and patterns
- Proactively include relevant files in context
- Ask for confirmation before destructive operations
- Provide code examples and explanations
- Focus on practical, working solutions
- Keep security and best practices in mind

Your goal is to help the user build better software faster through intelligent automation and current information.`,
        spb.currentTime.Format("2006-01-02 15:04:05 MST"),
        spb.workspaceInfo.RootDir,
        spb.workspaceInfo.ProjectType,
        spb.workspaceInfo.GitStatus,
        spb.formatRecentChanges(),
        spb.formatAvailableTools(),
    )
    
    return prompt
}
```

### Workspace Detection & Context
```go
type WorkspaceContext struct {
    RootDir          string
    ProjectType      string              // go, node, python, rust, etc.
    ConfigFiles      []string            // go.mod, package.json, etc.
    GitStatus        string
    RecentChanges    []FileChange
    Dependencies     []Dependency
    BuildSystem      string              // make, go build, npm, cargo, etc.
    TestFramework    string
    LastActivity     time.Time
}

type WorkspaceDetector struct {
    detectors map[string]ProjectDetector
}

func (wd *WorkspaceDetector) DetectProject(rootDir string) (*WorkspaceContext, error) {
    // Detect project type by config files
    // Analyze git status and recent changes
    // Identify build system and test framework
    // Extract dependency information
}
```

## 2. Temporal Awareness & Current Information

### Time-Aware Search System
```go
type TemporalSearchEngine struct {
    currentTime     time.Time
    timezone        *time.Location
    searchProviders []SearchProvider
    cache          *TimeAwareCachedResults
}

type SearchTrigger struct {
    Keywords        []string            // "latest", "current", "2024", "new", "updated"
    TechStack       []string            // Framework/library names
    ErrorPatterns   []string            // Error message patterns
    VersionContext  *VersionContext     // Detected versions in conversation
}

func (tse *TemporalSearchEngine) ShouldSearchForCurrentInfo(message string, context *ConversationContext) (bool, []SearchQuery) {
    triggers := []SearchTrigger{
        // Version-specific queries
        {Keywords: []string{"latest", "current", "newest"}, Priority: "high"},
        // Error messages (likely need current solutions)
        {ErrorPatterns: []string{"error:", "failed:", "cannot"}, Priority: "medium"},
        // Framework/library mentions
        {TechStack: detectTechnologies(message), Priority: "medium"},
        // Temporal indicators
        {Keywords: []string{"2024", "2025", "recent", "updated"}, Priority: "high"},
    }
    
    // Generate targeted search queries
    queries := generateSearchQueries(message, triggers, tse.currentTime)
    return len(queries) > 0, queries
}

type VersionContext struct {
    DetectedVersions map[string]string  // "react": "18.2.0"
    LastChecked      map[string]time.Time
    KnownOutdated    []string
}
```

### Automatic Information Freshness
```go
type FreshnessChecker struct {
    knowledgeCutoff time.Time
    updatePatterns  map[string]time.Duration  // How often different types of info change
}

var updateFrequencies = map[string]time.Duration{
    "framework_docs":    24 * time.Hour,      // Daily for popular frameworks
    "api_changes":       6 * time.Hour,       // API docs change frequently  
    "security_updates":  2 * time.Hour,       // Security info is critical
    "package_versions":  12 * time.Hour,      // Package registries update frequently
    "error_solutions":   7 * 24 * time.Hour,  // Error solutions more stable
}
```

## 3. Core Chat Interface Architecture

### Message System
```go
type Message struct {
    Role      string                 `json:"role"`      // user, assistant, system, tool
    Content   string                 `json:"content"`
    ToolCalls []ToolCall            `json:"tool_calls,omitempty"`
    ToolCallID string               `json:"tool_call_id,omitempty"`
    Metadata  map[string]interface{} `json:"metadata,omitempty"`
}

type Conversation struct {
    ID       string    `json:"id"`
    Messages []Message `json:"messages"`
    Context  *Context  `json:"context"`
    Provider string    `json:"provider"`
    Model    string    `json:"model"`
}
```

### Context Management
- **Token counting** per provider (different tokenizers)
- **Sliding window** strategy when approaching limits
- **Smart summarization** of older messages
- **File context injection** - automatically include relevant files
- **Working directory awareness** - LLM knows current directory structure

### Streaming & Real-time Updates
```go
type StreamHandler interface {
    OnToken(token string)
    OnToolCall(call ToolCall)
    OnError(err error)
    OnComplete(response Message)
}
```

## 4. Advanced Conversation Management

### Conversation Persistence & Memory
```go
type ConversationStore struct {
    db          *sql.DB
    sessionID   string
    maxTokens   int
    compression *MessageCompressor
}

type ConversationMemory struct {
    ShortTerm   []Message                    // Recent messages (full context)
    LongTerm    map[string]*TopicSummary    // Compressed older conversations
    UserPref    *UserPreferences            // Learned preferences
    ProjectCtx  *ProjectMemory              // Project-specific context
}

type ProjectMemory struct {
    CommonPatterns    []CodePattern         // Frequently used patterns
    PreferredLibs     []string             // User's preferred libraries
    CodingStyle       *StylePreferences    // Formatting, naming conventions
    RecentDecisions   []ArchitecturalChoice // "Why did we choose X over Y"
    ProblemSolutions  map[string]Solution  // Previous solutions to problems
}

func (cs *ConversationStore) OptimizeContext(messages []Message, maxTokens int) ([]Message, error) {
    // Token counting per provider
    // Sliding window with smart summarization
    // Preserve important context (tool results, errors, decisions)
    // Compress repetitive information
}
```

### Intelligent Context Window Management
```go
type ContextOptimizer struct {
    tokenCounters map[string]TokenCounter    // Per-provider token counting
    summarizer   *ConversationSummarizer
    prioritizer  *MessagePrioritizer
}

type MessagePriority int

const (
    Critical MessagePriority = iota  // Errors, important decisions
    High                             // Tool results, code changes
    Medium                           // Regular conversation
    Low                              // Casual remarks, confirmations
)

func (co *ContextOptimizer) OptimizeForProvider(messages []Message, provider string, limit int) ([]Message, error) {
    // Count tokens using provider-specific tokenizer
    // Prioritize messages by importance
    // Summarize older conversations
    // Maintain context coherence
}
```

## 5. Enhanced Error Recovery & Resilience

### Comprehensive Error Handling
```go
type ErrorRecoveryService struct {
    searchService     *WebSearchService
    knowledgeBase    *LocalKnowledgeBase
    retryStrategies  map[string]RetryStrategy
    fallbackProviders []LLMProvider
}

type ErrorContext struct {
    ErrorType      string
    Command        string
    Environment    *SystemInfo
    RecentChanges  []FileChange
    StackTrace     string
    UserContext    string
}

func (ers *ErrorRecoveryService) HandleError(err error, context *ErrorContext) (*RecoveryPlan, error) {
    // Search for current solutions to the error
    // Check local knowledge base for similar issues
    // Generate step-by-step recovery plan
    // Suggest preventive measures
    
    recoveryPlan := &RecoveryPlan{
        ImmediateActions: []Action{},
        Explanations:     []string{},
        PreventiveMeasures: []string{},
        RelatedDocs:      []SearchResult{},
    }
    
    return recoveryPlan, nil
}
```

### Provider Failover & Rate Limiting
```go
type ProviderManager struct {
    providers    []LLMProvider
    rateLimiters map[string]*rate.Limiter
    healthCheck  *ProviderHealthChecker
    costTracker  *CostTracker
}

func (pm *ProviderManager) GetBestProvider(ctx context.Context, req *ChatRequest) (LLMProvider, error) {
    // Check rate limits
    // Verify provider health
    // Consider cost implications
    // Select optimal provider for request type
}
```

## 6. Performance & Resource Management

### Resource Monitoring
```go
type ResourceMonitor struct {
    memoryUsage    *MemoryTracker
    diskUsage      *DiskTracker
    networkUsage   *NetworkTracker
    tokenUsage     *TokenTracker
}

type PerformanceMetrics struct {
    ResponseTime    time.Duration
    TokensPerSecond float64
    MemoryUsage     int64
    CacheHitRate    float64
    ErrorRate       float64
}

func (rm *ResourceMonitor) OptimizePerformance() *OptimizationPlan {
    // Monitor resource usage
    // Identify bottlenecks
    // Suggest optimizations
    // Auto-tune cache sizes
}
```

### Intelligent Caching
```go
type CacheManager struct {
    responseCache   *LRUCache      // LLM responses
    searchCache     *TimeAwareCache // Web search results  
    fileCache       *FileCache     // File contents with change detection
    knowledgeCache  *KnowledgeCache // Processed documentation
}

type TimeAwareCache struct {
    cache map[string]*CacheEntry
    ttl   map[string]time.Duration  // Different TTL for different content types
}
```

## 7. File System Integration

### Directory Scanning & Context
```go
type FileSystemContext struct {
    WorkingDirectory string
    IgnorePatterns   []string  // .gitignore-style patterns
    MaxFileSize      int64     // Skip large files
    AllowedExtensions []string // Focus on text files
    FileTree         *FileNode
    RecentChanges    []FileChange
}

type FileNode struct {
    Path     string
    IsDir    bool
    Size     int64
    ModTime  time.Time
    Children []*FileNode
    Content  string // For text files under size limit
}
```

### File Operations as Tools
```go
type FileOperations struct {
    ReadFile      func(path string) (string, error)
    WriteFile     func(path, content string) error
    CreateFile    func(path, content string) error
    DeleteFile    func(path string) error
    MoveFile      func(oldPath, newPath string) error
    ListDirectory func(path string) ([]FileInfo, error)
    SearchFiles   func(pattern, content string) ([]SearchResult, error)
    WatchFiles    func(patterns []string) <-chan FileEvent
}
```

### Security & Sandboxing
- **Restrict to working directory** and subdirectories
- **Confirmation prompts** for destructive operations
- **File size limits** to prevent memory issues
- **Extension filtering** for security
- **Git-aware** - respect .gitignore, don't modify .git/

## 8. Model Context Protocol (MCP) Implementation

### MCP Server Integration
```go
type MCPServer struct {
    Name        string
    Endpoint    string
    Transport   MCPTransport // stdio, sse, websocket
    Tools       []MCPTool
    Resources   []MCPResource
    Prompts     []MCPPrompt
}

type MCPTool struct {
    Name        string                 `json:"name"`
    Description string                 `json:"description"`
    InputSchema map[string]interface{} `json:"inputSchema"`
}

type MCPClient struct {
    servers map[string]*MCPServer
    router  *ToolRouter
}
```

### MCP Implementation with mark3labs/mcp-go
```go
import (
    "github.com/mark3labs/mcp-go/mcp"
    "github.com/mark3labs/mcp-go/server"
    "github.com/mark3labs/mcp-go/client"
)

type MCPServerManager struct {
    servers map[string]*server.MCPServer
    client  *client.Client
}

func NewFileSystemMCPServer() *server.MCPServer {
    s := server.NewMCPServer(
        "filesystem-server",
        "1.0.0",
        server.WithToolCapabilities(true),
        server.WithResourceCapabilities(true, true),
    )
    
    // File operations tools
    readTool := mcp.NewTool("read_file",
        mcp.WithDescription("Read contents of a file"),
        mcp.WithString("path", mcp.Required(), mcp.Description("File path to read")),
    )
    s.AddTool(readTool, handleReadFile)
    
    writeTool := mcp.NewTool("write_file",
        mcp.WithDescription("Write content to a file"),
        mcp.WithString("path", mcp.Required(), mcp.Description("File path to write")),
        mcp.WithString("content", mcp.Required(), mcp.Description("Content to write")),
    )
    s.AddTool(writeTool, handleWriteFile)
    
    // File resources
    fileResource := mcp.NewResource(
        "file://{path}",
        "File Resource",
        mcp.WithResourceDescription("Access to file contents"),
        mcp.WithMIMEType("text/plain"),
    )
    s.AddResourceTemplate(fileResource, handleFileResource)
    
    return s
}

func handleReadFile(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
    path := mcp.ParseString(request, "path", "")
    if path == "" {
        return nil, errors.New("path is required")
    }
    
    content, err := os.ReadFile(path)
    if err != nil {
        return mcp.NewToolResultError(fmt.Sprintf("Failed to read file: %v", err)), nil
    }
    
    return mcp.NewToolResultText(string(content)), nil
}
```

### Built-in MCP Servers
- **File system server** - comprehensive file operations
- **Git server** - version control with advanced features  
- **Web server** - search, fetch, and analyze web content
- **Shell server** - safe command execution with sandboxing
- **Test server** - test framework integration and execution
- **Code analysis server** - linting, complexity analysis, refactoring

### Tool Routing & Execution
```go
type ToolRouter struct {
    tools map[string]Tool
    mcpServers map[string]*MCPServer
}

func (r *ToolRouter) ExecuteTool(ctx context.Context, name string, args map[string]interface{}) (*ToolResult, error) {
    // Route to appropriate MCP server or built-in tool
    // Handle errors and retries
    // Log tool usage for debugging
}
```

## 9. Autonomous Web Search System

### Search Decision Engine
```go
type SearchDecisionEngine struct {
    patterns []SearchPattern
    context  *ConversationContext
}

type SearchPattern struct {
    Triggers   []string // Keywords that suggest search needed
    Categories []string // "documentation", "current events", "errors"
    Confidence float64  // How likely this needs search
}

func (sde *SearchDecisionEngine) ShouldSearch(userMessage string, context *ConversationContext) (bool, SearchStrategy) {
    // Analyze message for:
    // - Current time references ("latest", "recent", "current")
    // - Technology/framework names that change frequently
    // - Error messages that might need current solutions
    // - Documentation requests
    // - Version-specific questions
}
```

### Multi-Provider Search
```go
type SearchProvider interface {
    Search(ctx context.Context, query string) (*SearchResults, error)
    Name() string
    RateLimit() time.Duration
}

type WebSearchService struct {
    providers []SearchProvider
    cache     *SearchCache
    fetcher   *URLFetcher
}

// Providers
type BraveSearchProvider struct{}
type DuckDuckGoProvider struct{}
type GoogleSearchProvider struct{}
type SerperProvider struct{}
```

### Content Extraction & Processing
```go
type URLFetcher struct {
    client      *http.Client
    extractors  map[string]ContentExtractor
    rateLimiter *rate.Limiter
}

type ContentExtractor interface {
    Extract(rawHTML string) (*ExtractedContent, error)
    CanHandle(url string) bool
}

type ExtractedContent struct {
    Title       string
    MainContent string
    CodeBlocks  []CodeBlock
    Links       []Link
    Metadata    map[string]string
}
```

### Search Integration Flow
1. **User sends message**
2. **Decision engine** determines if search needed
3. **Generate search queries** based on context
4. **Execute searches** across multiple providers
5. **Fetch and extract** top results
6. **Inject into context** before sending to LLM
7. **LLM responds** with current information

## 10. LLM Provider Abstraction

### Unified Interface
```go
type LLMProvider interface {
    Chat(ctx context.Context, req *ChatRequest) (*ChatResponse, error)
    ChatStream(ctx context.Context, req *ChatRequest) (<-chan StreamChunk, error)
    SupportsFunctions() bool
    SupportsSystemMessages() bool
    GetTokenLimit() int
    CountTokens(text string) int
    Name() string
}

type ChatRequest struct {
    Messages    []Message
    Tools       []Tool
    MaxTokens   int
    Temperature float64
    Stream      bool
}
```

### Provider Implementations
```go
type OpenAIProvider struct {
    client *openai.Client
    model  string
}

type ClaudeProvider struct {
    client *anthropic.Client
    model  string
}

type OllamaProvider struct {
    baseURL string
    model   string
}

type GeminiProvider struct {
    client *genai.Client
    model  string
}
```

### Function Calling Normalization
```go
type FunctionCallAdapter struct {
    provider LLMProvider
}

func (fca *FunctionCallAdapter) NormalizeToolCall(providerSpecific interface{}) *ToolCall {
    // Convert provider-specific function calls to standard format
    // Handle differences in parameter passing, result formatting
}
```

## 11. Auto-Update System

### Self-Update Implementation
```go
import (
    "github.com/blang/semver"
    "github.com/rhysd/go-github-selfupdate/selfupdate"
)

type UpdateManager struct {
    currentVersion semver.Version
    repoSlug       string
    checkFrequency time.Duration
}

func (um *UpdateManager) CheckForUpdates(ctx context.Context) (*selfupdate.Release, bool, error) {
    return selfupdate.DetectLatest(um.repoSlug)
}

func (um *UpdateManager) UpdateSelf(ctx context.Context) error {
    latest, err := selfupdate.UpdateSelf(um.currentVersion, um.repoSlug)
    if err != nil {
        return err
    }
    
    if !latest.Version.Equals(um.currentVersion) {
        log.Printf("Updated to version %s", latest.Version)
        log.Printf("Release notes: %s", latest.ReleaseNotes)
    }
    
    return nil
}
```

### Update Triggers
- **Automatic background checks** (configurable frequency)
- **Manual update command** (`llm-cli update`)
- **Startup version check** (non-blocking)
- **Rollback support** on failed updates

### Security Features
- SHA256 checksum validation
- ECDSA signature verification
- Secure GitHub releases integration
- Rollback on update failure

## 12. Enhanced Git Integration

### Advanced Git Operations
```go
type GitService struct {
    repo       *git.Repository
    workingDir string
}

type GitOperations struct {
    CommitChanges      func(message string, files []string) error
    CreatePullRequest  func(title, body, branch string) error
    ResolveMergeConflict func(file string, resolution string) error
    SearchHistory      func(query string) ([]git.Commit, error)
    AnalyzeDiff        func(commit1, commit2 string) (*DiffAnalysis, error)
    SuggestCommitMsg   func(changes []FileChange) (string, error)
}
```

### GitHub/GitLab Integration
- **PR/MR creation and management**
- **Issue linking in commits**
- **Branch management and workflows**
- **Code review integration**
- **CI/CD status checking**

## 13. Test Integration & Code Analysis

### Test Framework Integration
```go
type TestRunner struct {
    frameworks map[string]TestFramework
    coverage   *CoverageAnalyzer
}

type TestFramework interface {
    RunTests(ctx context.Context, patterns []string) (*TestResults, error)
    GenerateTests(ctx context.Context, targetFile string) ([]TestCase, error)
    AnalyzeFailures(results *TestResults) ([]FixSuggestion, error)
}

type TestOperations struct {
    RunAllTests       func() (*TestResults, error)
    RunSpecificTests  func(patterns []string) (*TestResults, error)
    GenerateTestCases func(sourceFile string) ([]TestCase, error)
    FixFailingTests   func(failures []TestFailure) ([]CodeFix, error)
    UpdateSnapshots   func() error
}
```

### Code Quality Analysis
```go
type CodeAnalyzer struct {
    linters      map[string]Linter
    complexity   *ComplexityAnalyzer
    dependencies *DependencyAnalyzer
}

type AnalysisOperations struct {
    RunLinters         func(files []string) ([]LintIssue, error)
    AnalyzeComplexity  func(function string) (*ComplexityReport, error)
    FindCodeSmells     func(directory string) ([]CodeSmell, error)
    SuggestRefactoring func(issues []CodeIssue) ([]RefactorSuggestion, error)
    AnalyzeDependencies func() (*DependencyGraph, error)
}
```

## 14. Configuration & Plugin System

### Configuration Management
```yaml
# ~/.llm-cli/config.yaml
providers:
  openai:
    api_key: "${OPENAI_API_KEY}"
    model: "gpt-4-turbo"
    max_tokens: 4096
  claude:
    api_key: "${ANTHROPIC_API_KEY}"
    model: "claude-sonnet-4"
  ollama:
    base_url: "http://localhost:11434"
    model: "llama2"

default_provider: "claude"

search:
  enabled: true
  auto_search: true
  providers: ["brave", "duckduckgo"]
  brave_api_key: "${BRAVE_API_KEY}"
  max_results: 5

file_system:
  auto_include_context: true
  max_file_size: "1MB"
  ignore_patterns: 
    - "node_modules"
    - ".git"
    - "*.log"
  watch_for_changes: true

mcp:
  servers:
    - name: "filesystem"
      command: ["llm-cli-fs-server"]
      transport: "stdio"
    - name: "git"
      command: ["llm-cli-git-server"]
      transport: "stdio"

security:
  require_confirmation:
    - "delete_file"
    - "execute_command"
    - "git_push"
  sandbox_mode: true
```

### Plugin Architecture
```go
type Plugin interface {
    Name() string
    Initialize(ctx context.Context, config map[string]interface{}) error
    GetTools() []Tool
    GetMCPServers() []MCPServer
    Shutdown() error
}

type PluginManager struct {
    plugins map[string]Plugin
    config  *Config
}
```

## 15. Advanced Features

### Smart Context Injection
```go
type ContextInjector struct {
    fileAnalyzer    *FileAnalyzer
    searchService   *WebSearchService
    conversationMgr *ConversationManager
}

func (ci *ContextInjector) BuildContext(userMessage string, workingDir string) (*EnhancedContext, error) {
    // Analyze user message for relevant files
    // Determine if web search needed
    // Include git status if relevant
    // Add recent file changes
    // Include relevant documentation
}
```

### Error Recovery & Debugging
```go
type ErrorRecoveryService struct {
    searchService *WebSearchService
    knowledgeBase *ErrorKnowledgeBase
}

func (ers *ErrorRecoveryService) HandleError(errorText string, context *ExecutionContext) (*RecoveryPlan, error) {
    // Search for error solutions
    // Check local knowledge base
    // Suggest debugging steps
    // Provide relevant documentation
}
```

### Performance Optimization
- **Response caching** for repeated queries
- **File content caching** with change detection
- **Search result caching** with TTL
- **Parallel tool execution** where safe
- **Background file indexing**

### Monitoring & Analytics
```go
type UsageTracker struct {
    tokenUsage    map[string]int64
    toolUsage     map[string]int64
    searchUsage   map[string]int64
    responseTime  []time.Duration
    errorRates    map[string]float64
}
```

## 16. Security Considerations

### File System Security
- Restrict operations to working directory tree
- Validate all file paths (prevent directory traversal)
- Confirm destructive operations
- Monitor file size and resource usage

### API Security
- Secure storage of API keys (OS keychain)
- Rate limiting to prevent quota exhaustion
- Input validation for all tool parameters
- Audit logging of all operations

### Network Security
- HTTPS only for all external requests
- Certificate validation
- Respect robots.txt for web scraping
- User-agent identification

## 17. Distribution & Updates

### Binary Distribution
```go
// Use goreleaser for multi-platform builds
// Single binary with embedded assets
// Auto-update mechanism with user consent
```

### Installation Methods
- **Homebrew** (macOS/Linux)
- **Scoop** (Windows)
- **Direct download** (GitHub releases)
- **Docker image** for containerized usage

## 18. Testing Strategy

### Unit Testing
- Mock LLM providers for deterministic testing
- Test tool execution in isolated environments
- Validate configuration parsing and validation

### Integration Testing
- Test with real LLM providers (rate limited)
- File system operations in temp directories
- MCP server communication

### End-to-End Testing
- Full conversation flows
- Multi-tool scenarios
- Error recovery paths

## Key Differentiators from Claude Code

### Multi-Provider Support
- **API Agnostic**: Works with OpenAI, Claude, Gemini, Ollama, and any OpenAI-compatible API
- **Provider-specific optimizations**: Token counting, function calling, context limits
- **Fallback providers**: Automatic failover when primary provider is unavailable
- **Cost optimization**: Route queries to most cost-effective provider for the task

### Enhanced Autonomy
- **Proactive web search**: Automatically searches for current documentation and solutions
- **Smart context injection**: Intelligently includes relevant files and information
- **Background indexing**: Maintains searchable index of codebase and documentation
- **Learned patterns**: Remembers successful patterns and workflows (locally)

### Enterprise-Ready Features
- **Self-hosted deployment**: No dependency on external services for core functionality
- **Air-gapped support**: Works in offline environments with local models
- **Audit logging**: Comprehensive logging for compliance requirements
- **Team collaboration**: Shared configurations and knowledge bases
- **Custom model endpoints**: Support for private model deployments

### Developer Experience
- **Faster startup**: Optimized for quick response times
- **Better error handling**: More helpful error messages and recovery suggestions  
- **Extensible architecture**: Plugin system for custom tools and workflows
- **Configuration management**: Fine-grained control over behavior and preferences

This architecture provides a robust foundation for building a Claude Code-like tool that's truly API-agnostic while providing seamless file system access, autonomous web search, and proper tool use capabilities.

## Additional Missing Components

### User Experience & Interface
```go
type InteractiveInterface struct {
    renderer     *TerminalRenderer
    prompter     *InteractivePrompter
    progressBars map[string]*ProgressBar
    notifications *NotificationManager
}

// Rich terminal UI with progress indicators
// Interactive confirmations for destructive operations
// Syntax highlighting for code output
// Clickable links and file paths
// Collapsible/expandable sections
```

### Database & Storage
```go
type StorageManager struct {
    conversations *ConversationDB
    knowledge     *KnowledgeDB
    cache         *CacheDB
    preferences   *UserPrefsDB
}

// SQLite for local data persistence
// Encrypted storage for sensitive information
// Search indexing for conversation history
// Knowledge base for learned solutions
```

### Integration Ecosystem
```go
type IntegrationManager struct {
    ide          []IDEIntegration      // VS Code, Vim, Emacs extensions
    ci           []CIIntegration       // GitHub Actions, GitLab CI
    deployment   []DeploymentPlatform  // Docker, K8s, Cloud platforms
    monitoring   []MonitoringTool      // Grafana, DataDog, New Relic
    databases    []DatabaseConnector   // MySQL, PostgreSQL, MongoDB
}
```

### Template & Macro System
```go
type AutomationEngine struct {
    templates    *TemplateManager      // Code templates, project scaffolding
    macros       *MacroRecorder        // Record and replay command sequences
    workflows    *WorkflowEngine       // Multi-step automated processes
    triggers     *EventTriggerSystem   // File change triggers, time-based
}
```