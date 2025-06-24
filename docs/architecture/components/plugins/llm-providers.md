# LLM Provider System Technical Specification

## Overview

The Aircher LLM Provider System implements a universal interface for integrating multiple Language Learning Model providers. This system provides consistent API access across different providers while maintaining provider-specific optimizations and capabilities.

## Architecture Principles

### Universal Provider Interface
- **Consistent API**: All providers implement the same interface for seamless switching
- **Provider-Specific Optimizations**: Each provider can optimize for its unique capabilities
- **Graceful Degradation**: Fallback mechanisms when providers are unavailable
- **Cost Tracking**: Built-in cost calculation and budget management
- **Rate Limiting**: Automatic rate limiting to respect provider limits

### Supported Providers
- **OpenAI**: GPT-3.5, GPT-4, GPT-4 Turbo models
- **Anthropic Claude**: Claude-3 Haiku, Sonnet, Opus models
- **Google Gemini**: Gemini Pro, Gemini Pro Vision
- **Ollama**: Local model hosting with various open-source models

## Core Interface Definition

### LLMProvider Interface
```go
type LLMProvider interface {
    // Core chat functionality
    Chat(ctx context.Context, req *ChatRequest) (*ChatResponse, error)
    ChatStream(ctx context.Context, req *ChatRequest) (<-chan *ChatResponse, error)
    
    // Provider capabilities
    SupportsFunctions() bool
    SupportsSystemMessages() bool
    SupportsImages() bool
    SupportsThinking() bool
    
    // Model information
    GetTokenLimit(model string) int
    CountTokens(content string) (int, error)
    CalculateCost(tokens int, model string) (float64, error)
    
    // Provider metadata
    Name() string
    Models() []string
    
    // Health and status
    IsHealthy(ctx context.Context) bool
    GetStatus() ProviderStatus
}
```

### Request/Response Structures

#### ChatRequest
```go
type ChatRequest struct {
    Messages    []Message     `json:"messages"`
    Tools       []Tool        `json:"tools,omitempty"`
    MaxTokens   *int          `json:"max_tokens,omitempty"`
    Temperature *float64      `json:"temperature,omitempty"`
    Stream      bool          `json:"stream,omitempty"`
    Model       string        `json:"model"`
    Provider    string        `json:"provider"`
    
    // Advanced options
    TopP           *float64                `json:"top_p,omitempty"`
    FrequencyPenalty *float64              `json:"frequency_penalty,omitempty"`
    PresencePenalty  *float64              `json:"presence_penalty,omitempty"`
    StopSequences    []string              `json:"stop,omitempty"`
    
    // Aircher-specific options
    ThinkingMode    bool                   `json:"thinking_mode,omitempty"`
    ContextWindow   int                    `json:"context_window,omitempty"`
    CostLimit       *float64               `json:"cost_limit,omitempty"`
    TaskType        TaskType               `json:"task_type,omitempty"`
}

type Message struct {
    Role      string      `json:"role"` // user, assistant, system, tool
    Content   interface{} `json:"content"` // string or []ContentPart
    Name      string      `json:"name,omitempty"`
    ToolCalls []ToolCall  `json:"tool_calls,omitempty"`
    ToolCallId string     `json:"tool_call_id,omitempty"`
}

type ContentPart struct {
    Type     string                 `json:"type"` // text, image_url
    Text     string                 `json:"text,omitempty"`
    ImageURL *ImageURL              `json:"image_url,omitempty"`
}

type Tool struct {
    Type     string   `json:"type"` // function
    Function Function `json:"function"`
}

type Function struct {
    Name        string                 `json:"name"`
    Description string                 `json:"description"`
    Parameters  map[string]interface{} `json:"parameters"`
}
```

#### ChatResponse
```go
type ChatResponse struct {
    Message   *Message              `json:"message,omitempty"`
    Stream    *StreamResponse       `json:"stream,omitempty"`
    
    // Usage and cost information
    TokensUsed TokenUsage           `json:"usage"`
    Cost       float64              `json:"cost"`
    Duration   time.Duration        `json:"duration"`
    
    // Provider information
    Provider   string               `json:"provider"`
    Model      string               `json:"model"`
    
    // Metadata
    Metadata   map[string]interface{} `json:"metadata,omitempty"`
    
    // Error information
    Error      *ProviderError       `json:"error,omitempty"`
}

type TokenUsage struct {
    PromptTokens     int `json:"prompt_tokens"`
    CompletionTokens int `json:"completion_tokens"`
    TotalTokens      int `json:"total_tokens"`
}

type StreamResponse struct {
    Delta     *MessageDelta `json:"delta,omitempty"`
    Finished  bool          `json:"finished"`
    FinishReason string     `json:"finish_reason,omitempty"`
}

type MessageDelta struct {
    Role      string     `json:"role,omitempty"`
    Content   string     `json:"content,omitempty"`
    ToolCalls []ToolCall `json:"tool_calls,omitempty"`
}
```

## Provider Implementations

### OpenAI Provider
```go
type OpenAIProvider struct {
    client      *openai.Client
    model       string
    apiKey      string
    baseURL     string
    orgID       string
    
    // Cost and rate limiting
    costTable   map[string]CostInfo
    rateLimiter *rate.Limiter
    
    // Configuration
    config      OpenAIConfig
    logger      *zerolog.Logger
    
    // Metrics
    requestCount  int64
    totalCost     float64
    avgLatency    time.Duration
}

type OpenAIConfig struct {
    APIKey           string            `toml:"api_key_env"`
    Model            string            `toml:"model"`
    MaxTokens        int               `toml:"max_tokens"`
    Temperature      float64           `toml:"temperature"`
    BaseURL          string            `toml:"base_url"`
    OrgID            string            `toml:"org_id"`
    RequestsPerMin   int               `toml:"requests_per_min"`
    CustomModels     map[string]string `toml:"custom_models"`
}

type CostInfo struct {
    InputCostPer1K  float64 `json:"input_cost_per_1k"`
    OutputCostPer1K float64 `json:"output_cost_per_1k"`
    Currency        string  `json:"currency"`
}
```

#### OpenAI Implementation Details
- **Models Supported**: gpt-3.5-turbo, gpt-4, gpt-4-turbo, gpt-4o
- **Special Features**: Function calling, vision (GPT-4V), JSON mode
- **Rate Limiting**: Configurable requests per minute
- **Cost Tracking**: Accurate per-token pricing for all models
- **Streaming**: Full streaming support with delta updates

### Claude Provider
```go
type ClaudeProvider struct {
    client      *anthropic.Client
    apiKey      string
    model       string
    
    // Configuration
    config      ClaudeConfig
    rateLimiter *rate.Limiter
    logger      *zerolog.Logger
    
    // Cost tracking
    costTable   map[string]CostInfo
    totalCost   float64
}

type ClaudeConfig struct {
    APIKey           string  `toml:"api_key_env"`
    Model            string  `toml:"model"`
    MaxTokens        int     `toml:"max_tokens"`
    Temperature      float64 `toml:"temperature"`
    RequestsPerMin   int     `toml:"requests_per_min"`
}
```

#### Claude Implementation Details
- **Models Supported**: claude-3-haiku, claude-3-sonnet, claude-3-opus
- **Special Features**: Large context windows, thinking mode support
- **System Messages**: Full support for system prompts
- **Tool Use**: Native function calling support
- **Streaming**: Streaming with proper delta handling

### Gemini Provider
```go
type GeminiProvider struct {
    client    *genai.Client
    apiKey    string
    projectID string
    model     string
    
    // Configuration
    config      GeminiConfig
    rateLimiter *rate.Limiter
    logger      *zerolog.Logger
}

type GeminiConfig struct {
    APIKey           string `toml:"api_key_env"`
    Project          string `toml:"project"`
    Model            string `toml:"model"`
    MaxTokens        int    `toml:"max_tokens"`
    Temperature      float64 `toml:"temperature"`
    Location         string `toml:"location"`
}
```

#### Gemini Implementation Details
- **Models Supported**: gemini-pro, gemini-pro-vision
- **Special Features**: Multimodal capabilities, code generation
- **Authentication**: Service account or API key based
- **Safety Settings**: Configurable content filtering
- **Streaming**: Partial streaming support

### Ollama Provider
```go
type OllamaProvider struct {
    client    *http.Client
    baseURL   string
    model     string
    
    // Configuration
    config    OllamaConfig
    logger    *zerolog.Logger
    
    // Local model management
    availableModels []string
    modelInfo       map[string]ModelInfo
}

type OllamaConfig struct {
    BaseURL      string        `toml:"base_url"`
    Model        string        `toml:"model"`
    KeepAlive    time.Duration `toml:"keep_alive"`
    Temperature  float64       `toml:"temperature"`
    NumCtx       int           `toml:"num_ctx"`
    NumPredict   int           `toml:"num_predict"`
}

type ModelInfo struct {
    Name          string    `json:"name"`
    Size          int64     `json:"size"`
    Digest        string    `json:"digest"`
    ModifiedAt    time.Time `json:"modified_at"`
    Details       ModelDetails `json:"details"`
}
```

#### Ollama Implementation Details
- **Models Supported**: llama2, codellama, mistral, phi, etc.
- **Local Hosting**: No API costs, runs locally
- **Model Management**: Automatic model pulling and updates
- **Custom Models**: Support for custom trained models
- **Streaming**: Full streaming support

## Provider Management

### Provider Manager
```go
type ProviderManager struct {
    providers   map[string]LLMProvider
    defaultProvider string
    fallbackOrder []string
    
    // Configuration
    config      ProviderConfig
    logger      *zerolog.Logger
    
    // Task-specific model selection
    taskModelMap map[TaskType]string
    costOptimized bool
    
    // Metrics and monitoring
    healthChecker *HealthChecker
    metrics      *ProviderMetrics
    costTracker  *CostTracker
}

type ProviderConfig struct {
    Default         string                    `toml:"default"`
    FallbackOrder   []string                  `toml:"fallback_order"`
    HealthCheckInterval time.Duration         `toml:"health_check_interval"`
    Providers       map[string]interface{}    `toml:"providers"`
    TaskModels      map[string]string         `toml:"task_models"`
    CostOptimized   bool                      `toml:"prefer_cost_efficient"`
}

// Task types for intelligent model selection
type TaskType string

const (
    TaskCommitMessage   TaskType = "commit_messages"
    TaskSummary         TaskType = "summaries"
    TaskCodeReview      TaskType = "code_review"
    TaskDocumentation   TaskType = "documentation"
    TaskRefactoring     TaskType = "refactoring"
    TaskDebugging       TaskType = "debugging"
    TaskQuickQuestion   TaskType = "quick_questions"
    TaskCodeGeneration  TaskType = "code_generation"
    TaskDefault         TaskType = "default"
)

// GetModelForTask returns the optimal model for a specific task type
func (pm *ProviderManager) GetModelForTask(taskType TaskType) string {
    // Check for task-specific override
    if model, exists := pm.taskModelMap[taskType]; exists {
        return model
    }
    
    // Fall back to default model
    return pm.config.Default
}

// SelectProviderForTask chooses the best provider/model combination for a task
func (pm *ProviderManager) SelectProviderForTask(taskType TaskType) (string, string) {
    model := pm.GetModelForTask(taskType)
    
    // Find which provider supports this model
    for providerName, provider := range pm.providers {
        for _, supportedModel := range provider.Models() {
            if supportedModel == model {
                return providerName, model
            }
        }
    }
    
    // Fall back to default provider with its default model
    return pm.defaultProvider, pm.GetModelForTask(TaskDefault)
}

type ProviderMetrics struct {
    RequestCount    map[string]int64
    ErrorCount      map[string]int64
    AvgLatency      map[string]time.Duration
    TotalCost       map[string]float64
    LastHealthCheck map[string]time.Time
}
```

### Health Checking
```go
type HealthChecker struct {
    interval time.Duration
    timeout  time.Duration
    logger   *zerolog.Logger
}

type ProviderStatus struct {
    Name        string        `json:"name"`
    Healthy     bool          `json:"healthy"`
    LastCheck   time.Time     `json:"last_check"`
    Latency     time.Duration `json:"latency"`
    Error       string        `json:"error,omitempty"`
    Capabilities []string     `json:"capabilities"`
}

func (hc *HealthChecker) CheckProvider(ctx context.Context, provider LLMProvider) ProviderStatus {
    start := time.Now()
    
    healthCtx, cancel := context.WithTimeout(ctx, hc.timeout)
    defer cancel()
    
    healthy := provider.IsHealthy(healthCtx)
    latency := time.Since(start)
    
    status := ProviderStatus{
        Name:      provider.Name(),
        Healthy:   healthy,
        LastCheck: time.Now(),
        Latency:   latency,
    }
    
    if !healthy {
        status.Error = "Health check failed"
    }
    
    return status
}
```

### Task Detection and Auto-Model Selection

The provider system includes intelligent task detection and automatic model selection to optimize for both cost and performance based on the specific type of work being performed.

#### Task Detection Engine
```go
type TaskDetector struct {
    patterns        map[TaskType][]TaskPattern
    contextAnalyzer *ContextAnalyzer
    historyTracker  *HistoryTracker
    logger          *zerolog.Logger
}

type TaskPattern struct {
    Keywords        []string              `json:"keywords"`
    MessagePatterns []string              `json:"message_patterns"`
    FilePatterns    []string              `json:"file_patterns"`
    GitPatterns     []string              `json:"git_patterns"`
    Confidence      float64               `json:"confidence"`
}

type ContextAnalyzer struct {
    fileWatcher     *FileWatcher
    gitWatcher      *GitWatcher
    commandHistory  []string
    currentFiles    []string
}

// DetectTaskType analyzes the current context to determine the most likely task type
func (td *TaskDetector) DetectTaskType(messages []Message, context *Context) TaskType {
    scores := make(map[TaskType]float64)
    
    // Analyze message content
    for _, msg := range messages {
        content := strings.ToLower(msg.Content.(string))
        for taskType, patterns := range td.patterns {
            for _, pattern := range patterns {
                score := td.calculatePatternScore(content, pattern)
                scores[taskType] += score
            }
        }
    }
    
    // Analyze file context
    if context != nil {
        td.analyzeFileContext(context, scores)
        td.analyzeGitContext(context, scores)
    }
    
    // Return highest scoring task type
    return td.getBestMatch(scores)
}

// Example task detection patterns
var defaultTaskPatterns = map[TaskType][]TaskPattern{
    TaskCommitMessage: {
        {
            Keywords: []string{"commit", "git commit", "commit message", "changelog"},
            MessagePatterns: []string{"write.*commit", "generate.*commit", "create.*commit"},
            GitPatterns: []string{"staged_changes", "git_status"},
            Confidence: 0.9,
        },
    },
    TaskSummary: {
        {
            Keywords: []string{"summarize", "summary", "tldr", "overview", "brief"},
            MessagePatterns: []string{"summarize.*", "give.*summary", "what.*about"},
            FilePatterns: []string{"*.md", "*.txt", "*.log"},
            Confidence: 0.8,
        },
    },
    TaskCodeReview: {
        {
            Keywords: []string{"review", "code review", "feedback", "suggestions", "improvements"},
            MessagePatterns: []string{"review.*code", "look.*code", "feedback.*on"},
            FilePatterns: []string{"*.rs", "*.go", "*.py", "*.js", "*.ts"},
            GitPatterns: []string{"diff", "changes", "modified"},
            Confidence: 0.85,
        },
    },
    TaskDebugging: {
        {
            Keywords: []string{"bug", "error", "debug", "fix", "problem", "issue", "broken"},
            MessagePatterns: []string{"why.*not.*work", "error.*when", "debug.*", "fix.*"},
            FilePatterns: []string{"*.log", "*.err"},
            Confidence: 0.9,
        },
    },
}
```

#### Smart Model Selection
```go
type ModelSelector struct {
    taskModelMap    map[TaskType]ModelConfig
    costOptimizer   *CostOptimizer
    provider        *ProviderManager
    preferences     ModelPreferences
}

type ModelConfig struct {
    PreferredModel  string                 `json:"preferred_model"`
    FallbackModels  []string               `json:"fallback_models"`
    MaxCostPer1K    float64                `json:"max_cost_per_1k"`
    MinQualityScore float64                `json:"min_quality_score"`
    RequiredFeatures []string              `json:"required_features"`
}

type ModelPreferences struct {
    PrioritizeCost      bool    `json:"prioritize_cost"`
    PrioritizeSpeed     bool    `json:"prioritize_speed"`
    PrioritizeQuality   bool    `json:"prioritize_quality"`
    MaxCostPerRequest   float64 `json:"max_cost_per_request"`
    MaxResponseTime     time.Duration `json:"max_response_time"`
}

// SelectOptimalModel chooses the best model for a given task and context
func (ms *ModelSelector) SelectOptimalModel(taskType TaskType, context *RequestContext) (string, string, error) {
    config, exists := ms.taskModelMap[taskType]
    if !exists {
        return ms.getDefaultModel()
    }
    
    // Check if preferred model meets cost constraints
    if ms.preferences.PrioritizeCost {
        if cost := ms.costOptimizer.EstimateCost(config.PreferredModel, context); cost > ms.preferences.MaxCostPerRequest {
            return ms.selectCostEfficientModel(taskType, context)
        }
    }
    
    // Verify model availability and health
    provider, model := ms.findAvailableProvider(config.PreferredModel)
    if provider != "" {
        return provider, model, nil
    }
    
    // Try fallback models
    for _, fallbackModel := range config.FallbackModels {
        if provider, model := ms.findAvailableProvider(fallbackModel); provider != "" {
            return provider, model, nil
        }
    }
    
    return "", "", fmt.Errorf("no suitable model found for task type: %s", taskType)
}

// Default task-to-model mappings optimized for cost and performance
var defaultTaskModelConfig = map[TaskType]ModelConfig{
    TaskCommitMessage: {
        PreferredModel: "gpt-3.5-turbo",
        FallbackModels: []string{"claude-3-haiku", "gemini-pro"},
        MaxCostPer1K: 0.002,
        MinQualityScore: 0.7,
        RequiredFeatures: []string{"chat"},
    },
    TaskSummary: {
        PreferredModel: "claude-3-haiku",
        FallbackModels: []string{"gpt-3.5-turbo", "gemini-pro"},
        MaxCostPer1K: 0.001,
        MinQualityScore: 0.8,
        RequiredFeatures: []string{"chat", "long_context"},
    },
    TaskCodeReview: {
        PreferredModel: "gpt-4",
        FallbackModels: []string{"claude-3-5-sonnet", "gpt-4-turbo"},
        MaxCostPer1K: 0.03,
        MinQualityScore: 0.9,
        RequiredFeatures: []string{"chat", "code_analysis"},
    },
    TaskDebugging: {
        PreferredModel: "claude-3-5-sonnet",
        FallbackModels: []string{"gpt-4", "gpt-4-turbo"},
        MaxCostPer1K: 0.015,
        MinQualityScore: 0.85,
        RequiredFeatures: []string{"chat", "reasoning", "code_analysis"},
    },
    TaskCodeGeneration: {
        PreferredModel: "gpt-4",
        FallbackModels: []string{"claude-3-5-sonnet", "gpt-4-turbo"},
        MaxCostPer1K: 0.03,
        MinQualityScore: 0.9,
        RequiredFeatures: []string{"chat", "code_generation"},
    },
    TaskQuickQuestion: {
        PreferredModel: "gpt-3.5-turbo",
        FallbackModels: []string{"claude-3-haiku", "gemini-pro"},
        MaxCostPer1K: 0.002,
        MinQualityScore: 0.7,
        RequiredFeatures: []string{"chat"},
    },
}
```

#### Integration with Provider Manager
The task detection and model selection integrates seamlessly with the existing provider management system:

```go
// Enhanced ChatWithTaskDetection method
func (pm *ProviderManager) ChatWithTaskDetection(req *ChatRequest) (*ChatResponse, error) {
    // Auto-detect task type if not specified
    if req.TaskType == "" {
        context := pm.buildRequestContext(req)
        req.TaskType = pm.taskDetector.DetectTaskType(req.Messages, context)
        pm.logger.Info().Str("detected_task", string(req.TaskType)).Msg("Auto-detected task type")
    }
    
    // Select optimal provider and model for the task
    if req.Provider == "" || req.Model == "" {
        provider, model, err := pm.modelSelector.SelectOptimalModel(req.TaskType, nil)
        if err != nil {
            return nil, fmt.Errorf("model selection failed: %w", err)
        }
        req.Provider = provider
        req.Model = model
        pm.logger.Info().
            Str("provider", provider).
            Str("model", model).
            Str("task", string(req.TaskType)).
            Msg("Auto-selected optimal model")
    }
    
    // Proceed with regular chat processing
    return pm.Chat(req)
}
```

### Cost Tracking
```go
type CostTracker struct {
    dailyLimits   map[string]float64
    monthlyLimits map[string]float64
    currentCosts  map[string]DailyCost
    
    // Alerts
    alertThreshold float64
    alertCallback  func(provider string, cost float64, limit float64)
    
    mutex sync.RWMutex
}

type DailyCost struct {
    Date     time.Time `json:"date"`
    Cost     float64   `json:"cost"`
    Requests int       `json:"requests"`
    Tokens   int       `json:"tokens"`
}

func (ct *CostTracker) TrackCost(provider string, cost float64, tokens int) error {
    ct.mutex.Lock()
    defer ct.mutex.Unlock()
    
    today := time.Now().Format("2006-01-02")
    key := fmt.Sprintf("%s:%s", provider, today)
    
    if daily, exists := ct.currentCosts[key]; exists {
        daily.Cost += cost
        daily.Requests++
        daily.Tokens += tokens
        ct.currentCosts[key] = daily
    } else {
        ct.currentCosts[key] = DailyCost{
            Date:     time.Now(),
            Cost:     cost,
            Requests: 1,
            Tokens:   tokens,
        }
    }
    
    // Check limits
    if dailyLimit, exists := ct.dailyLimits[provider]; exists {
        if ct.currentCosts[key].Cost > dailyLimit {
            return fmt.Errorf("daily cost limit exceeded for provider %s", provider)
        }
    }
    
    return nil
}
```

## Configuration Integration

### TOML Configuration
```toml
[providers]
default = "openai"
fallback_order = ["openai", "claude", "gemini", "ollama"]
health_check_interval = "5m"

[providers.openai]
api_key_env = "OPENAI_API_KEY"
model = "gpt-4"
max_tokens = 4096
temperature = 0.7
requests_per_min = 60
base_url = "https://api.openai.com/v1"

[providers.claude]
api_key_env = "ANTHROPIC_API_KEY"
model = "claude-3-sonnet-20240229"
max_tokens = 4096
temperature = 0.7
requests_per_min = 50

[providers.gemini]
api_key_env = "GEMINI_API_KEY"
project = "your-project-id"
model = "gemini-pro"
max_tokens = 2048
temperature = 0.7
location = "us-central1"

[providers.ollama]
base_url = "http://localhost:11434"
model = "llama2"
keep_alive = "5m"
temperature = 0.7
num_ctx = 4096

# Task-specific model overrides for cost optimization
[models.tasks]
commit_messages = "gpt-3.5-turbo"        # Fast, cheap for git commits
summaries = "claude-3-haiku"             # Efficient for text summarization  
code_review = "gpt-4"                    # High-quality for code analysis
documentation = "claude-3-haiku"         # Good balance for docs
refactoring = "gpt-4"                    # Complex reasoning needed
debugging = "claude-3-5-sonnet"          # Strong analytical capabilities
quick_questions = "gpt-3.5-turbo"        # Fast responses for simple queries
code_generation = "gpt-4"                # High-quality code output

[costs]
monthly_budget = 100.0
daily_limit = 10.0
alert_threshold = 0.8
track_by_provider = true
prefer_cost_efficient = true             # Auto-select cheaper models when appropriate

[costs.limits]
openai_daily = 5.0
claude_daily = 3.0
gemini_daily = 2.0
```

## Error Handling and Resilience

### Error Types
```go
type ProviderError struct {
    Type        ErrorType `json:"type"`
    Message     string    `json:"message"`
    Code        string    `json:"code,omitempty"`
    StatusCode  int       `json:"status_code,omitempty"`
    Retryable   bool      `json:"retryable"`
    Provider    string    `json:"provider"`
}

type ErrorType string

const (
    ErrorTypeAuthentication ErrorType = "authentication"
    ErrorTypeRateLimit      ErrorType = "rate_limit"
    ErrorTypeQuotaExceeded  ErrorType = "quota_exceeded"
    ErrorTypeInvalidRequest ErrorType = "invalid_request"
    ErrorTypeServerError    ErrorType = "server_error"
    ErrorTypeNetworkError   ErrorType = "network_error"
    ErrorTypeTimeout        ErrorType = "timeout"
    ErrorTypeUnknown        ErrorType = "unknown"
)
```

### Retry Logic
```go
type RetryConfig struct {
    MaxRetries    int           `toml:"max_retries"`
    InitialDelay  time.Duration `toml:"initial_delay"`
    MaxDelay      time.Duration `toml:"max_delay"`
    BackoffFactor float64       `toml:"backoff_factor"`
    RetryableErrors []ErrorType `toml:"retryable_errors"`
}

func (pm *ProviderManager) ChatWithRetry(ctx context.Context, req *ChatRequest) (*ChatResponse, error) {
    var lastErr error
    
    for attempt := 0; attempt <= pm.retryConfig.MaxRetries; attempt++ {
        if attempt > 0 {
            delay := pm.calculateBackoffDelay(attempt)
            select {
            case <-ctx.Done():
                return nil, ctx.Err()
            case <-time.After(delay):
            }
        }
        
        resp, err := pm.chat(ctx, req)
        if err == nil {
            return resp, nil
        }
        
        lastErr = err
        if !pm.isRetryableError(err) {
            break
        }
    }
    
    return nil, lastErr
}
```

## Streaming Implementation

### Stream Handling
```go
type StreamManager struct {
    activeStreams map[string]*StreamContext
    mutex         sync.RWMutex
}

type StreamContext struct {
    ID       string
    Provider LLMProvider
    Request  *ChatRequest
    Channel  chan *ChatResponse
    Done     chan struct{}
    Error    chan error
    
    // Buffer management
    buffer   *StreamBuffer
    timeout  time.Duration
}

func (sm *StreamManager) StartStream(ctx context.Context, provider LLMProvider, req *ChatRequest) (<-chan *ChatResponse, error) {
    streamID := generateStreamID()
    
    streamCtx := &StreamContext{
        ID:       streamID,
        Provider: provider,
        Request:  req,
        Channel:  make(chan *ChatResponse, 100),
        Done:     make(chan struct{}),
        Error:    make(chan error, 1),
        buffer:   NewStreamBuffer(),
        timeout:  30 * time.Second,
    }
    
    sm.mutex.Lock()
    sm.activeStreams[streamID] = streamCtx
    sm.mutex.Unlock()
    
    go sm.handleStream(ctx, streamCtx)
    
    return streamCtx.Channel, nil
}
```

## Testing Framework

### Provider Testing
```go
type ProviderTestSuite struct {
    provider    LLMProvider
    testConfig  TestConfig
}

type TestConfig struct {
    TestAPIKey      string
    SkipCostTests   bool
    TimeoutDuration time.Duration
    TestModels      []string
}

func (pts *ProviderTestSuite) TestBasicChat(t *testing.T) {
    req := &ChatRequest{
        Messages: []Message{
            {Role: "user", Content: "Hello, world!"},
        },
        Model:       pts.testConfig.TestModels[0],
        MaxTokens:   ptrInt(100),
        Temperature: ptrFloat64(0.7),
    }
    
    ctx, cancel := context.WithTimeout(context.Background(), pts.testConfig.TimeoutDuration)
    defer cancel()
    
    resp, err := pts.provider.Chat(ctx, req)
    assert.NoError(t, err)
    assert.NotNil(t, resp)
    assert.NotEmpty(t, resp.Message.Content)
    assert.Greater(t, resp.TokensUsed.TotalTokens, 0)
}

func (pts *ProviderTestSuite) TestStreamingChat(t *testing.T) {
    req := &ChatRequest{
        Messages: []Message{
            {Role: "user", Content: "Count from 1 to 5"},
        },
        Model:   pts.testConfig.TestModels[0],
        Stream:  true,
    }
    
    ctx, cancel := context.WithTimeout(context.Background(), pts.testConfig.TimeoutDuration)
    defer cancel()
    
    stream, err := pts.provider.ChatStream(ctx, req)
    assert.NoError(t, err)
    
    var responses []*ChatResponse
    for resp := range stream {
        responses = append(responses, resp)
        if resp.Stream != nil && resp.Stream.Finished {
            break
        }
    }
    
    assert.Greater(t, len(responses), 0)
}
```

## Implementation Status

### ✅ Completed
- Universal provider interface design
- OpenAI provider implementation
- Claude provider implementation
- Basic configuration system
- Error handling framework

### 🚧 In Progress
- Gemini provider implementation
- Ollama provider implementation
- Advanced retry logic
- Comprehensive cost tracking
- Stream management optimization

### ❌ Pending
- Advanced health checking
- Provider failover mechanisms
- Comprehensive test suite
- Performance optimization
- Documentation and examples

## Performance Considerations

### Connection Pooling
- Reuse HTTP connections across requests
- Configure appropriate timeouts
- Implement connection health checks

### Request Batching
- Batch multiple requests when possible
- Implement request queuing for rate limiting
- Optimize token usage across requests

### Caching Strategies
- Cache model metadata and capabilities
- Implement response caching for identical requests
- Cache tokenization results

### Memory Management
- Efficient streaming buffer management
- Proper cleanup of completed streams
- Monitor memory usage in long-running processes

## Cost Optimization Strategies

### Intelligent Model Selection for Common Tasks

The multi-model configuration system enables significant cost savings by automatically selecting cheaper, faster models for routine tasks while preserving quality for complex operations.

#### Cost-Effective Task Mappings
```go
// Real-world cost optimization examples
var costOptimizedMappings = map[TaskType]CostStrategy{
    TaskCommitMessage: {
        Model:           "gpt-3.5-turbo",     // ~$0.001/1K tokens
        ExpectedSavings: "90%",               // vs GPT-4
        QualityLoss:     "minimal",
        UseCase:        "Simple git commit message generation",
    },
    TaskSummary: {
        Model:           "claude-3-haiku",    // ~$0.00025/1K tokens
        ExpectedSavings: "95%",               // vs Claude-3-Opus
        QualityLoss:     "low",
        UseCase:        "Code/document summarization",
    },
    TaskQuickQuestion: {
        Model:           "gpt-3.5-turbo",     // Fast + cheap
        ExpectedSavings: "85%",
        QualityLoss:     "minimal",
        UseCase:        "Simple Q&A, syntax help",
    },
}
```

### Dynamic Cost Controls
```go
type CostOptimizer struct {
    monthlyBudget   float64
    dailyBudget     float64
    currentSpend    float64
    alertThresholds []float64
    
    // Auto-downgrade expensive models when approaching limits
    autoDowngrade   bool
    downgradePaths  map[string][]string
}

// Example downgrade paths for cost control
var modelDowngradePaths = map[string][]string{
    "gpt-4":                {"gpt-4-turbo", "gpt-3.5-turbo"},
    "claude-3-opus":        {"claude-3-sonnet", "claude-3-haiku"},
    "claude-3-5-sonnet":    {"claude-3-haiku", "gpt-3.5-turbo"},
}
```

### Practical Cost Savings Examples

#### Example 1: Git Workflow Optimization
```toml
# Before: All tasks use GPT-4 (~$30/month)
# After: Task-specific models (~$8/month - 73% savings)

[models.tasks]
commit_messages = "gpt-3.5-turbo"        # $0.50/month
summaries = "claude-3-haiku"             # $0.25/month  
code_review = "gpt-4"                    # $5.00/month (keep quality)
quick_questions = "gpt-3.5-turbo"        # $2.00/month
documentation = "claude-3-haiku"         # $0.25/month
```

#### Example 2: Development Workflow
```go
// Typical development session cost breakdown
type SessionCostBreakdown struct {
    CodeReview:     "$0.15",  // 1-2 files, GPT-4
    CommitMessages: "$0.01",  // 5-10 commits, GPT-3.5-Turbo
    QuickQuestions: "$0.05",  // 10-15 questions, GPT-3.5-Turbo
    Documentation: "$0.02",   // README updates, Claude-3-Haiku
    TotalSession:   "$0.23",  // vs $1.20 with GPT-4 for everything
}
```

### Configuration Best Practices

#### Budget-Conscious Configuration
```toml
[costs]
monthly_budget = 25.0                    # Conservative budget
daily_limit = 1.5                       # Prevent surprise bills
alert_threshold = 0.75                  # 75% budget warning
prefer_cost_efficient = true            # Auto-select cheaper models
auto_downgrade_on_limit = true          # Fallback to cheaper models

[costs.task_budgets]
commit_messages = 2.0                   # Max $2/month for commits
summaries = 1.0                         # Max $1/month for summaries
quick_questions = 5.0                   # Max $5/month for Q&A
```

#### Performance vs Cost Balance
```toml
[models.tasks]
# High-quality tasks (worth the cost)
code_review = "gpt-4"                   # Complex reasoning needed
refactoring = "claude-3-5-sonnet"       # Architecture decisions
debugging = "gpt-4"                     # Problem-solving critical

# Cost-optimized tasks (quality sufficient)
commit_messages = "gpt-3.5-turbo"       # Simple text generation
summaries = "claude-3-haiku"            # Good comprehension, cheap
documentation = "claude-3-haiku"        # Writing assistant
quick_questions = "gpt-3.5-turbo"       # Fast responses
```

### Real-Time Cost Monitoring
```go
type CostMonitor struct {
    realTimeTracking bool
    costPerRequest   map[string]float64
    budgetAlerts     []AlertRule
    usageProjection  *UsageProjector
}

// Live cost feedback in TUI
type CostDisplay struct {
    CurrentSession:  "$0.23",
    DailySpend:     "$1.45 / $2.00",
    MonthlySpend:   "$18.50 / $25.00", 
    ProjectedMonth: "$22.30",
    SavingsToday:   "$0.85 (58%)",
}
```