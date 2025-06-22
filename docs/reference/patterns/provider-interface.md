# LLM Provider Interface Patterns

## Core Provider Interface

```go
// Universal interface for all LLM providers
type LLMProvider interface {
    Chat(ctx context.Context, req *ChatRequest) (*ChatResponse, error)
    ChatStream(ctx context.Context, req *ChatRequest) (<-chan StreamChunk, error)
    
    // Provider capabilities
    Name() string
    Models() []string
    SupportsStreaming() bool
    
    // Token management
    CountTokens(messages []Message) (int, error)
    CalculateCost(tokens int, model string) float64
}

type ChatRequest struct {
    Messages    []Message `json:"messages"`
    Model       string    `json:"model"`
    Temperature *float64  `json:"temperature,omitempty"`
    MaxTokens   *int      `json:"max_tokens,omitempty"`
    Stream      bool      `json:"stream"`
}

type ChatResponse struct {
    Content      string  `json:"content"`
    TokensUsed   int     `json:"tokens_used"`
    Cost         float64 `json:"cost"`
    FinishReason string  `json:"finish_reason"`
}

type StreamChunk struct {
    Content string `json:"content"`
    Done    bool   `json:"done"`
    Error   error  `json:"error,omitempty"`
}
```

## Provider Implementation Pattern

```go
type OpenAIProvider struct {
    client      *http.Client
    apiKey      string
    baseURL     string
    rateLimiter *rate.Limiter
}

func (p *OpenAIProvider) Chat(ctx context.Context, req *ChatRequest) (*ChatResponse, error) {
    // Rate limiting
    if err := p.rateLimiter.Wait(ctx); err != nil {
        return nil, fmt.Errorf("rate limit: %w", err)
    }
    
    // Build request, make HTTP call, parse response
    // Return structured response
}

func (p *OpenAIProvider) ChatStream(ctx context.Context, req *ChatRequest) (<-chan StreamChunk, error) {
    resultChan := make(chan StreamChunk, 100)
    
    go func() {
        defer close(resultChan)
        // Handle SSE streaming, parse chunks
        // Send chunks to channel
    }()
    
    return resultChan, nil
}
```

## Provider Registry

```go
type ProviderRegistry struct {
    providers map[string]LLMProvider
    mutex     sync.RWMutex
}

func (r *ProviderRegistry) Register(name string, provider LLMProvider) {
    r.mutex.Lock()
    defer r.mutex.Unlock()
    r.providers[name] = provider
}

func (r *ProviderRegistry) Get(name string) (LLMProvider, error) {
    r.mutex.RLock()
    defer r.mutex.RUnlock()
    
    provider, exists := r.providers[name]
    if !exists {
        return nil, fmt.Errorf("provider %s not found", name)
    }
    return provider, nil
}
```

## Error Handling Pattern

```go
type LLMError struct {
    Type    string `json:"type"`
    Message string `json:"message"`
    Code    int    `json:"code,omitempty"`
    Retry   bool   `json:"retry"`
}

func (e *LLMError) Error() string {
    return fmt.Sprintf("%s: %s", e.Type, e.Message)
}

// Common error types
var (
    ErrRateLimit    = &LLMError{Type: "rate_limit", Retry: true}
    ErrTokenLimit   = &LLMError{Type: "token_limit", Retry: false}
    ErrAuth         = &LLMError{Type: "auth", Retry: false}
    ErrNetwork      = &LLMError{Type: "network", Retry: true}
)
```

## Configuration Pattern

```go
type ProviderConfig struct {
    Name         string            `toml:"name"`
    APIKey       string            `toml:"api_key"`
    BaseURL      string            `toml:"base_url,omitempty"`
    DefaultModel string            `toml:"default_model"`
    Settings     map[string]string `toml:"settings,omitempty"`
}

type ProvidersConfig struct {
    Default   string           `toml:"default"`
    Providers []ProviderConfig `toml:"providers"`
}
```

## Usage Pattern

```go
// Initialize providers
registry := NewProviderRegistry()
registry.Register("openai", NewOpenAIProvider(config.OpenAI))
registry.Register("claude", NewClaudeProvider(config.Claude))

// Use provider
provider, err := registry.Get("openai")
if err != nil {
    return err
}

response, err := provider.Chat(ctx, &ChatRequest{
    Messages: []Message{{Role: "user", Content: "Hello"}},
    Model:    "gpt-4",
    Stream:   false,
})
```

## Rust Migration Pattern

```rust
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LLMError>;
    async fn chat_stream(&self, request: ChatRequest) -> Result<impl Stream<Item = Result<StreamChunk, LLMError>>, LLMError>;
    
    fn name(&self) -> &'static str;
    fn models(&self) -> Vec<String>;
    fn supports_streaming(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: Option<f64>,
    pub max_tokens: Option<usize>,
    pub stream: bool,
}
```

## Key Principles

1. **Universal Interface**: All providers implement the same interface
2. **Error Consistency**: Standardized error types across providers
3. **Streaming Support**: Both blocking and streaming modes
4. **Rate Limiting**: Built into provider implementations
5. **Configuration**: TOML-based provider configuration
6. **Registry Pattern**: Dynamic provider registration and selection
7. **Context Support**: All operations support cancellation
8. **Migration Ready**: Patterns work for both Go and Rust implementations