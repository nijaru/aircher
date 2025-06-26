package providers

import (
	"context"
	"fmt"
	"time"
)

// LLMProvider defines the interface that all LLM providers must implement
type LLMProvider interface {
	// Chat sends a chat request and returns a complete response
	Chat(ctx context.Context, request *ChatRequest) (*ChatResponse, error)
	
	// ChatStream sends a chat request and returns a streaming response
	ChatStream(ctx context.Context, request *ChatRequest) (<-chan *StreamChunk, error)
	
	// Capability checks
	SupportsFunctions() bool
	SupportsSystemMessages() bool
	SupportsImages() bool
	SupportsThinking() bool
	
	// Token and cost management
	GetTokenLimit(model string) int
	CountTokens(messages []Message) (int, error)
	CalculateCost(tokens int, model string) (float64, error)
	
	// Provider information
	Name() string
	Models() []Model
	
	// Health and status
	IsAvailable(ctx context.Context) bool
	GetRateLimit() *RateLimit
}

// ChatRequest represents a request to an LLM provider
type ChatRequest struct {
	Messages    []Message   `json:"messages"`
	Tools       []Tool      `json:"tools,omitempty"`
	MaxTokens   int         `json:"max_tokens,omitempty"`
	Temperature float64     `json:"temperature,omitempty"`
	Stream      bool        `json:"stream,omitempty"`
	Model       string      `json:"model"`
	Provider    string      `json:"provider"`
	SystemMsg   string      `json:"system_message,omitempty"`
	Thinking    bool        `json:"thinking,omitempty"`
	Metadata    interface{} `json:"metadata,omitempty"`
}

// ChatResponse represents a complete response from an LLM provider
type ChatResponse struct {
	Message     Message                `json:"message"`
	TokensUsed  TokenUsage             `json:"tokens_used"`
	Cost        float64                `json:"cost"`
	Duration    time.Duration          `json:"duration"`
	Provider    string                 `json:"provider"`
	Model       string                 `json:"model"`
	Metadata    map[string]interface{} `json:"metadata,omitempty"`
	Error       error                  `json:"error,omitempty"`
	FinishReason string                `json:"finish_reason,omitempty"`
}

// StreamChunk represents a chunk in a streaming response
type StreamChunk struct {
	Delta       MessageDelta           `json:"delta"`
	TokensUsed  TokenUsage             `json:"tokens_used,omitempty"`
	Cost        float64                `json:"cost,omitempty"`
	Provider    string                 `json:"provider"`
	Model       string                 `json:"model"`
	Done        bool                   `json:"done"`
	Error       error                  `json:"error,omitempty"`
	FinishReason string                `json:"finish_reason,omitempty"`
	Metadata    map[string]interface{} `json:"metadata,omitempty"`
}

// MessageDelta represents incremental content in a streaming response
type MessageDelta struct {
	Role         string      `json:"role,omitempty"`
	Content      string      `json:"content,omitempty"`
	ToolCalls    []ToolCall  `json:"tool_calls,omitempty"`
	ToolResults  []ToolResult `json:"tool_results,omitempty"`
}

// Message represents a chat message
type Message struct {
	Role        string       `json:"role"`
	Content     string       `json:"content"`
	ToolCalls   []ToolCall   `json:"tool_calls,omitempty"`
	ToolResults []ToolResult `json:"tool_results,omitempty"`
	Images      []Image      `json:"images,omitempty"`
	Metadata    interface{}  `json:"metadata,omitempty"`
}

// MessageRole constants
const (
	RoleSystem    = "system"
	RoleUser      = "user"
	RoleAssistant = "assistant"
	RoleTool      = "tool"
)

// Tool represents a function that can be called by the LLM
type Tool struct {
	Type        string      `json:"type"`
	Function    Function    `json:"function"`
	Description string      `json:"description,omitempty"`
}

// Function represents a function definition
type Function struct {
	Name        string      `json:"name"`
	Description string      `json:"description"`
	Parameters  interface{} `json:"parameters"`
}

// ToolCall represents a function call made by the LLM
type ToolCall struct {
	ID       string      `json:"id"`
	Type     string      `json:"type"`
	Function FunctionCall `json:"function"`
}

// FunctionCall represents the actual function call details
type FunctionCall struct {
	Name      string `json:"name"`
	Arguments string `json:"arguments"`
}

// ToolResult represents the result of a tool/function call
type ToolResult struct {
	ToolCallID string      `json:"tool_call_id"`
	Content    string      `json:"content"`
	Error      string      `json:"error,omitempty"`
	Metadata   interface{} `json:"metadata,omitempty"`
}

// Image represents an image input
type Image struct {
	URL    string `json:"url,omitempty"`
	Data   []byte `json:"data,omitempty"`
	Format string `json:"format,omitempty"`
	Alt    string `json:"alt,omitempty"`
}

// TokenUsage represents token consumption information
type TokenUsage struct {
	PromptTokens     int `json:"prompt_tokens"`
	CompletionTokens int `json:"completion_tokens"`
	TotalTokens      int `json:"total_tokens"`
}

// Model represents an available model
type Model struct {
	ID               string   `json:"id"`
	Name             string   `json:"name"`
	Description      string   `json:"description,omitempty"`
	MaxTokens        int      `json:"max_tokens"`
	SupportsFunctions bool    `json:"supports_functions"`
	SupportsImages   bool     `json:"supports_images"`
	SupportsThinking bool     `json:"supports_thinking"`
	CostPer1KTokens  float64  `json:"cost_per_1k_tokens"`
	Tags             []string `json:"tags,omitempty"`
}

// RateLimit represents rate limiting information
type RateLimit struct {
	RequestsPerMinute int           `json:"requests_per_minute"`
	TokensPerMinute   int           `json:"tokens_per_minute"`
	RequestsPerDay    int           `json:"requests_per_day"`
	ResetTime         time.Time     `json:"reset_time,omitempty"`
	RetryAfter        time.Duration `json:"retry_after,omitempty"`
}

// ProviderType constants
const (
	ProviderOpenAI  = "openai"
	ProviderClaude  = "claude"
	ProviderGemini  = "gemini"
	ProviderOllama  = "ollama"
	ProviderCustom  = "custom"
)

// ErrorType constants for provider errors
const (
	ErrorTypeRateLimit     = "rate_limit"
	ErrorTypeAuth         = "authentication"
	ErrorTypeInvalidRequest = "invalid_request"
	ErrorTypeServerError   = "server_error"
	ErrorTypeTimeout      = "timeout"
	ErrorTypeQuotaExceeded = "quota_exceeded"
	ErrorTypeModelNotFound = "model_not_found"
)

// ProviderError represents a provider-specific error
type ProviderError struct {
	Type         string                 `json:"type"`
	Message      string                 `json:"message"`
	Code         string                 `json:"code,omitempty"`
	StatusCode   int                    `json:"status_code,omitempty"`
	RetryAfter   time.Duration          `json:"retry_after,omitempty"`
	Details      map[string]interface{} `json:"details,omitempty"`
	Provider     string                 `json:"provider"`
}

func (e *ProviderError) Error() string {
	return fmt.Sprintf("%s provider error: %s", e.Provider, e.Message)
}

// ProviderConfig represents configuration for a specific provider
type ProviderConfig struct {
	Type        string                 `json:"type"`
	Name        string                 `json:"name"`
	APIKey      string                 `json:"api_key,omitempty"`
	BaseURL     string                 `json:"base_url,omitempty"`
	Model       string                 `json:"model"`
	MaxTokens   int                    `json:"max_tokens,omitempty"`
	Temperature float64                `json:"temperature,omitempty"`
	Timeout     time.Duration          `json:"timeout,omitempty"`
	Enabled     bool                   `json:"enabled"`
	Priority    int                    `json:"priority,omitempty"`
	Settings    map[string]interface{} `json:"settings,omitempty"`
}

// StreamReader provides an interface for reading streaming responses
type StreamReader interface {
	Read() (*StreamChunk, error)
	Close() error
}

// ProviderHealth represents the health status of a provider
type ProviderHealth struct {
	Provider    string        `json:"provider"`
	Available   bool          `json:"available"`
	LastChecked time.Time     `json:"last_checked"`
	Latency     time.Duration `json:"latency,omitempty"`
	Error       string        `json:"error,omitempty"`
	Models      []string      `json:"models,omitempty"`
}