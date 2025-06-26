package providers

import (
	"context"
	"strings"
	"time"

	"github.com/aircher/aircher/internal/config"
	"github.com/rs/zerolog"
)

// OllamaProvider implements the LLMProvider interface for Ollama
type OllamaProvider struct {
	client      *OllamaClient
	baseURL     string
	model       string
	keepAlive   string
	costTable   map[string]CostInfo
	rateLimiter *ProviderRateLimiter
	logger      zerolog.Logger
	config      *config.OllamaProviderConfig
}

// OllamaClient represents the Ollama API client
type OllamaClient struct {
	baseURL   string
	keepAlive string
}

// NewOllamaProvider creates a new Ollama provider instance
func NewOllamaProvider(cfg *config.OllamaProviderConfig, logger zerolog.Logger) (*OllamaProvider, error) {
	client := &OllamaClient{
		baseURL:   cfg.BaseURL,
		keepAlive: cfg.KeepAlive,
	}

	provider := &OllamaProvider{
		client:    client,
		baseURL:   cfg.BaseURL,
		model:     cfg.Model,
		keepAlive: cfg.KeepAlive,
		logger:    logger.With().Str("provider", "ollama").Logger(),
		config:    cfg,
		rateLimiter: &ProviderRateLimiter{
			requestsPerMinute: 1000, // Local server typically has high limits
			tokensPerMinute:   100000,
			lastReset:         time.Now(),
		},
	}

	// Initialize cost table (mostly zeros for local models)
	provider.initializeCostTable()

	return provider, nil
}

// initializeCostTable sets up pricing information for Ollama models (typically free)
func (p *OllamaProvider) initializeCostTable() {
	p.costTable = map[string]CostInfo{
		"llama2": {
			InputCostPer1K:  0.0,
			OutputCostPer1K: 0.0,
		},
		"llama2:13b": {
			InputCostPer1K:  0.0,
			OutputCostPer1K: 0.0,
		},
		"llama2:70b": {
			InputCostPer1K:  0.0,
			OutputCostPer1K: 0.0,
		},
		"mistral": {
			InputCostPer1K:  0.0,
			OutputCostPer1K: 0.0,
		},
		"codellama": {
			InputCostPer1K:  0.0,
			OutputCostPer1K: 0.0,
		},
		"vicuna": {
			InputCostPer1K:  0.0,
			OutputCostPer1K: 0.0,
		},
		"orca-mini": {
			InputCostPer1K:  0.0,
			OutputCostPer1K: 0.0,
		},
	}
}

// Chat sends a chat request and returns a complete response
func (p *OllamaProvider) Chat(ctx context.Context, request *ChatRequest) (*ChatResponse, error) {
	start := time.Now()
	p.logger.Debug().Str("model", request.Model).Int("messages", len(request.Messages)).Msg("Sending chat request")

	// TODO: Implement actual Ollama API call
	// For now, return a stub response
	response := &ChatResponse{
		Message: Message{
			Role:    RoleAssistant,
			Content: "Ollama provider response not yet implemented",
		},
		TokensUsed: TokenUsage{
			PromptTokens:     90,
			CompletionTokens: 45,
			TotalTokens:      135,
		},
		Cost:     0.0, // Ollama is typically free
		Duration: time.Since(start),
		Provider: p.Name(),
		Model:    request.Model,
		Metadata: map[string]interface{}{
			"stub": true,
		},
	}

	return response, nil
}

// ChatStream sends a chat request and returns a streaming response
func (p *OllamaProvider) ChatStream(ctx context.Context, request *ChatRequest) (<-chan *StreamChunk, error) {
	p.logger.Debug().Str("model", request.Model).Msg("Starting streaming chat request")

	// TODO: Implement actual Ollama streaming API call
	// For now, return a stub stream
	stream := make(chan *StreamChunk, 1)
	
	go func() {
		defer close(stream)
		
		// Send a single chunk as stub
		stream <- &StreamChunk{
			Delta: MessageDelta{
				Role:    RoleAssistant,
				Content: "Ollama streaming response not yet implemented",
			},
			TokensUsed: TokenUsage{
				TotalTokens: 135,
			},
			Cost:     0.0, // Ollama is typically free
			Provider: p.Name(),
			Model:    request.Model,
			Done:     true,
			Metadata: map[string]interface{}{
				"stub": true,
			},
		}
	}()

	return stream, nil
}

// SupportsFunctions returns whether the provider supports function calling
func (p *OllamaProvider) SupportsFunctions() bool {
	// Most Ollama models don't support function calling yet
	return strings.Contains(p.model, "codellama") || strings.Contains(p.model, "mistral")
}

// SupportsSystemMessages returns whether the provider supports system messages
func (p *OllamaProvider) SupportsSystemMessages() bool {
	return true
}

// SupportsImages returns whether the provider supports image inputs
func (p *OllamaProvider) SupportsImages() bool {
	// Only some Ollama models support vision
	return strings.Contains(p.model, "llava") || strings.Contains(p.model, "vision")
}

// SupportsThinking returns whether the provider supports thinking mode
func (p *OllamaProvider) SupportsThinking() bool {
	return false // Most local models don't have explicit thinking mode
}

// GetTokenLimit returns the token limit for a specific model
func (p *OllamaProvider) GetTokenLimit(model string) int {
	switch {
	case strings.Contains(model, "70b"):
		return 4096 // Larger models typically have standard context
	case strings.Contains(model, "13b"):
		return 4096
	case strings.Contains(model, "7b") || model == "llama2":
		return 4096
	case strings.Contains(model, "codellama"):
		return 16384 // Code models often have larger context
	default:
		return 4096
	}
}

// CountTokens estimates token count for messages
func (p *OllamaProvider) CountTokens(messages []Message) (int, error) {
	// TODO: Implement proper token counting for Ollama models
	// For now, use a rough approximation
	totalChars := 0
	for _, msg := range messages {
		totalChars += len(msg.Content)
		totalChars += len(msg.Role)
		
		// Add overhead for tool calls
		for _, toolCall := range msg.ToolCalls {
			totalChars += len(toolCall.Function.Name)
			totalChars += len(toolCall.Function.Arguments)
		}
	}
	
	// Rough approximation: 4 characters per token
	return totalChars / 4, nil
}

// CalculateCost calculates the cost for a given number of tokens
func (p *OllamaProvider) CalculateCost(tokens int, model string) (float64, error) {
	return 0.0, nil // Ollama is typically free
}

// Name returns the provider name
func (p *OllamaProvider) Name() string {
	return ProviderOllama
}

// Models returns available models
func (p *OllamaProvider) Models() []Model {
	return []Model{
		{
			ID:               "llama2",
			Name:             "Llama 2 7B",
			Description:      "Meta's Llama 2 7B model",
			MaxTokens:        4096,
			SupportsFunctions: false,
			SupportsImages:   false,
			SupportsThinking: false,
			CostPer1KTokens:  0.0,
			Tags:             []string{"free", "local", "general"},
		},
		{
			ID:               "llama2:13b",
			Name:             "Llama 2 13B",
			Description:      "Meta's Llama 2 13B model",
			MaxTokens:        4096,
			SupportsFunctions: false,
			SupportsImages:   false,
			SupportsThinking: false,
			CostPer1KTokens:  0.0,
			Tags:             []string{"free", "local", "general"},
		},
		{
			ID:               "llama2:70b",
			Name:             "Llama 2 70B",
			Description:      "Meta's Llama 2 70B model",
			MaxTokens:        4096,
			SupportsFunctions: false,
			SupportsImages:   false,
			SupportsThinking: false,
			CostPer1KTokens:  0.0,
			Tags:             []string{"free", "local", "powerful"},
		},
		{
			ID:               "codellama",
			Name:             "Code Llama",
			Description:      "Code-specialized Llama model",
			MaxTokens:        16384,
			SupportsFunctions: true,
			SupportsImages:   false,
			SupportsThinking: false,
			CostPer1KTokens:  0.0,
			Tags:             []string{"free", "local", "code"},
		},
		{
			ID:               "mistral",
			Name:             "Mistral 7B",
			Description:      "Mistral's 7B model",
			MaxTokens:        8192,
			SupportsFunctions: true,
			SupportsImages:   false,
			SupportsThinking: false,
			CostPer1KTokens:  0.0,
			Tags:             []string{"free", "local", "efficient"},
		},
		{
			ID:               "vicuna",
			Name:             "Vicuna",
			Description:      "Vicuna chat model",
			MaxTokens:        4096,
			SupportsFunctions: false,
			SupportsImages:   false,
			SupportsThinking: false,
			CostPer1KTokens:  0.0,
			Tags:             []string{"free", "local", "chat"},
		},
		{
			ID:               "orca-mini",
			Name:             "Orca Mini",
			Description:      "Microsoft's Orca Mini model",
			MaxTokens:        4096,
			SupportsFunctions: false,
			SupportsImages:   false,
			SupportsThinking: false,
			CostPer1KTokens:  0.0,
			Tags:             []string{"free", "local", "compact"},
		},
	}
}

// IsAvailable checks if the provider is currently available
func (p *OllamaProvider) IsAvailable(ctx context.Context) bool {
	// TODO: Implement actual availability check by pinging Ollama server
	// For now, assume available if baseURL is configured
	return p.baseURL != ""
}

// GetRateLimit returns current rate limit information
func (p *OllamaProvider) GetRateLimit() *RateLimit {
	return &RateLimit{
		RequestsPerMinute: p.rateLimiter.requestsPerMinute,
		TokensPerMinute:   p.rateLimiter.tokensPerMinute,
		RequestsPerDay:    p.rateLimiter.requestsPerMinute * 60 * 24,
		ResetTime:         p.rateLimiter.lastReset.Add(time.Minute),
	}
}