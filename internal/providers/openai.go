package providers

import (
	"context"
	"fmt"
	"os"
	"strings"
	"time"

	"github.com/aircher/aircher/internal/config"
	"github.com/rs/zerolog"
	"github.com/sashabaranov/go-openai"
)

// OpenAIProvider implements the LLMProvider interface for OpenAI
type OpenAIProvider struct {
	client      *openai.Client
	model       string
	apiKey      string
	baseURL     string
	costTable   map[string]CostInfo
	rateLimiter *ProviderRateLimiter
	logger      zerolog.Logger
	config      *config.OpenAIProviderConfig
}

// CostInfo represents pricing information for a model
type CostInfo struct {
	InputCostPer1K  float64
	OutputCostPer1K float64
}

// ProviderRateLimiter handles rate limiting for the provider
type ProviderRateLimiter struct {
	requestsPerMinute int
	tokensPerMinute   int
	lastReset         time.Time
	requestCount      int
	tokenCount        int
}

// NewOpenAIProvider creates a new OpenAI provider instance
func NewOpenAIProvider(cfg *config.OpenAIProviderConfig, logger zerolog.Logger) (*OpenAIProvider, error) {
	apiKey := os.Getenv(cfg.APIKeyEnv)
	if apiKey == "" {
		return nil, fmt.Errorf("OpenAI API key not found in environment variable %s", cfg.APIKeyEnv)
	}

	// Create OpenAI client config
	clientConfig := openai.DefaultConfig(apiKey)
	if cfg.BaseURL != "" {
		clientConfig.BaseURL = cfg.BaseURL
	}

	client := openai.NewClientWithConfig(clientConfig)

	provider := &OpenAIProvider{
		client:  client,
		model:   cfg.Model,
		apiKey:  apiKey,
		baseURL: cfg.BaseURL,
		logger:  logger.With().Str("provider", "openai").Logger(),
		config:  cfg,
		rateLimiter: &ProviderRateLimiter{
			requestsPerMinute: 3500, // Default OpenAI rate limits
			tokensPerMinute:   90000,
			lastReset:         time.Now(),
		},
	}

	// Initialize cost table
	provider.initializeCostTable()

	return provider, nil
}

// initializeCostTable sets up pricing information for OpenAI models
func (p *OpenAIProvider) initializeCostTable() {
	p.costTable = map[string]CostInfo{
		"gpt-4": {
			InputCostPer1K:  0.03,
			OutputCostPer1K: 0.06,
		},
		"gpt-4-turbo": {
			InputCostPer1K:  0.01,
			OutputCostPer1K: 0.03,
		},
		"gpt-4o": {
			InputCostPer1K:  0.005,
			OutputCostPer1K: 0.015,
		},
		"gpt-4o-mini": {
			InputCostPer1K:  0.00015,
			OutputCostPer1K: 0.0006,
		},
		"gpt-3.5-turbo": {
			InputCostPer1K:  0.001,
			OutputCostPer1K: 0.002,
		},
	}
}

// Chat sends a chat request and returns a complete response
func (p *OpenAIProvider) Chat(ctx context.Context, request *ChatRequest) (*ChatResponse, error) {
	start := time.Now()
	p.logger.Debug().Str("model", request.Model).Int("messages", len(request.Messages)).Msg("Sending chat request")

	// TODO: Implement actual OpenAI API call
	// For now, return a stub response
	response := &ChatResponse{
		Message: Message{
			Role:    RoleAssistant,
			Content: "OpenAI provider response not yet implemented",
		},
		TokensUsed: TokenUsage{
			PromptTokens:     100,
			CompletionTokens: 50,
			TotalTokens:      150,
		},
		Cost:     p.calculateCostForTokens(150, request.Model),
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
func (p *OpenAIProvider) ChatStream(ctx context.Context, request *ChatRequest) (<-chan *StreamChunk, error) {
	p.logger.Debug().Str("model", request.Model).Msg("Starting streaming chat request")

	// TODO: Implement actual OpenAI streaming API call
	// For now, return a stub stream
	stream := make(chan *StreamChunk, 1)
	
	go func() {
		defer close(stream)
		
		// Send a single chunk as stub
		stream <- &StreamChunk{
			Delta: MessageDelta{
				Role:    RoleAssistant,
				Content: "OpenAI streaming response not yet implemented",
			},
			TokensUsed: TokenUsage{
				TotalTokens: 150,
			},
			Cost:     p.calculateCostForTokens(150, request.Model),
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
func (p *OpenAIProvider) SupportsFunctions() bool {
	return strings.HasPrefix(p.model, "gpt-4") || strings.HasPrefix(p.model, "gpt-3.5-turbo")
}

// SupportsSystemMessages returns whether the provider supports system messages
func (p *OpenAIProvider) SupportsSystemMessages() bool {
	return true
}

// SupportsImages returns whether the provider supports image inputs
func (p *OpenAIProvider) SupportsImages() bool {
	return strings.Contains(p.model, "gpt-4") && !strings.Contains(p.model, "gpt-4-turbo")
}

// SupportsThinking returns whether the provider supports thinking mode
func (p *OpenAIProvider) SupportsThinking() bool {
	return false // OpenAI doesn't have explicit thinking mode like Claude
}

// GetTokenLimit returns the token limit for a specific model
func (p *OpenAIProvider) GetTokenLimit(model string) int {
	switch {
	case strings.HasPrefix(model, "gpt-4"):
		if strings.Contains(model, "turbo") {
			return 128000
		}
		return 8192
	case strings.HasPrefix(model, "gpt-3.5-turbo"):
		return 16385
	default:
		return 4096
	}
}

// CountTokens estimates token count for messages
func (p *OpenAIProvider) CountTokens(messages []Message) (int, error) {
	// TODO: Implement proper token counting using tiktoken or similar
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
func (p *OpenAIProvider) CalculateCost(tokens int, model string) (float64, error) {
	return p.calculateCostForTokens(tokens, model), nil
}

// calculateCostForTokens internal helper for cost calculation
func (p *OpenAIProvider) calculateCostForTokens(tokens int, model string) float64 {
	costInfo, exists := p.costTable[model]
	if !exists {
		// Default to GPT-4 pricing if model not found
		costInfo = p.costTable["gpt-4"]
	}
	
	// Assume 70% input tokens, 30% output tokens for estimation
	inputTokens := float64(tokens) * 0.7
	outputTokens := float64(tokens) * 0.3
	
	inputCost := (inputTokens / 1000) * costInfo.InputCostPer1K
	outputCost := (outputTokens / 1000) * costInfo.OutputCostPer1K
	
	return inputCost + outputCost
}

// Name returns the provider name
func (p *OpenAIProvider) Name() string {
	return ProviderOpenAI
}

// Models returns available models
func (p *OpenAIProvider) Models() []Model {
	return []Model{
		{
			ID:               "gpt-4",
			Name:             "GPT-4",
			Description:      "Most capable model, great for complex tasks",
			MaxTokens:        8192,
			SupportsFunctions: true,
			SupportsImages:   true,
			SupportsThinking: false,
			CostPer1KTokens:  0.045, // Average of input/output
			Tags:             []string{"reasoning", "function-calling"},
		},
		{
			ID:               "gpt-4-turbo",
			Name:             "GPT-4 Turbo",
			Description:      "Faster and cheaper than GPT-4",
			MaxTokens:        128000,
			SupportsFunctions: true,
			SupportsImages:   true,
			SupportsThinking: false,
			CostPer1KTokens:  0.02,
			Tags:             []string{"fast", "reasoning", "function-calling"},
		},
		{
			ID:               "gpt-4o",
			Name:             "GPT-4o",
			Description:      "Optimized for speed and cost",
			MaxTokens:        128000,
			SupportsFunctions: true,
			SupportsImages:   true,
			SupportsThinking: false,
			CostPer1KTokens:  0.01,
			Tags:             []string{"fast", "cost-effective"},
		},
		{
			ID:               "gpt-4o-mini",
			Name:             "GPT-4o Mini",
			Description:      "Fastest and most cost-effective",
			MaxTokens:        128000,
			SupportsFunctions: true,
			SupportsImages:   false,
			SupportsThinking: false,
			CostPer1KTokens:  0.0004,
			Tags:             []string{"fast", "cheap", "lightweight"},
		},
		{
			ID:               "gpt-3.5-turbo",
			Name:             "GPT-3.5 Turbo",
			Description:      "Good performance at low cost",
			MaxTokens:        16385,
			SupportsFunctions: true,
			SupportsImages:   false,
			SupportsThinking: false,
			CostPer1KTokens:  0.0015,
			Tags:             []string{"fast", "cheap"},
		},
	}
}

// IsAvailable checks if the provider is currently available
func (p *OpenAIProvider) IsAvailable(ctx context.Context) bool {
	// TODO: Implement actual availability check
	// For now, assume available if we have an API key
	return p.apiKey != ""
}

// GetRateLimit returns current rate limit information
func (p *OpenAIProvider) GetRateLimit() *RateLimit {
	return &RateLimit{
		RequestsPerMinute: p.rateLimiter.requestsPerMinute,
		TokensPerMinute:   p.rateLimiter.tokensPerMinute,
		RequestsPerDay:    p.rateLimiter.requestsPerMinute * 60 * 24,
		ResetTime:         p.rateLimiter.lastReset.Add(time.Minute),
	}
}