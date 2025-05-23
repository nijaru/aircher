package providers

import (
	"context"
	"fmt"
	"os"
	"strings"
	"time"

	"github.com/aircher/aircher/internal/config"
	"github.com/rs/zerolog"
)

// ClaudeProvider implements the LLMProvider interface for Anthropic Claude
type ClaudeProvider struct {
	client      *ClaudeClient
	model       string
	apiKey      string
	costTable   map[string]CostInfo
	rateLimiter *ProviderRateLimiter
	logger      zerolog.Logger
	config      *config.ClaudeProviderConfig
}

// ClaudeClient represents the Anthropic API client
type ClaudeClient struct {
	apiKey  string
	baseURL string
}

// NewClaudeProvider creates a new Claude provider instance
func NewClaudeProvider(cfg *config.ClaudeProviderConfig, logger zerolog.Logger) (*ClaudeProvider, error) {
	apiKey := os.Getenv(cfg.APIKeyEnv)
	if apiKey == "" {
		return nil, fmt.Errorf("Claude API key not found in environment variable %s", cfg.APIKeyEnv)
	}

	client := &ClaudeClient{
		apiKey:  apiKey,
		baseURL: "https://api.anthropic.com",
	}

	provider := &ClaudeProvider{
		client:  client,
		model:   cfg.Model,
		apiKey:  apiKey,
		logger:  logger.With().Str("provider", "claude").Logger(),
		config:  cfg,
		rateLimiter: &ProviderRateLimiter{
			requestsPerMinute: 1000, // Default Claude rate limits
			tokensPerMinute:   40000,
			lastReset:         time.Now(),
		},
	}

	// Initialize cost table
	provider.initializeCostTable()

	return provider, nil
}

// initializeCostTable sets up pricing information for Claude models
func (p *ClaudeProvider) initializeCostTable() {
	p.costTable = map[string]CostInfo{
		"claude-3-opus-20240229": {
			InputCostPer1K:  0.015,
			OutputCostPer1K: 0.075,
		},
		"claude-3-sonnet-20240229": {
			InputCostPer1K:  0.003,
			OutputCostPer1K: 0.015,
		},
		"claude-3-haiku-20240307": {
			InputCostPer1K:  0.00025,
			OutputCostPer1K: 0.00125,
		},
		"claude-3-5-sonnet-20241022": {
			InputCostPer1K:  0.003,
			OutputCostPer1K: 0.015,
		},
	}
}

// Chat sends a chat request and returns a complete response
func (p *ClaudeProvider) Chat(ctx context.Context, request *ChatRequest) (*ChatResponse, error) {
	start := time.Now()
	p.logger.Debug().Str("model", request.Model).Int("messages", len(request.Messages)).Msg("Sending chat request")

	// TODO: Implement actual Claude API call
	// For now, return a stub response
	response := &ChatResponse{
		Message: Message{
			Role:    RoleAssistant,
			Content: "Claude provider response not yet implemented",
		},
		TokensUsed: TokenUsage{
			PromptTokens:     120,
			CompletionTokens: 60,
			TotalTokens:      180,
		},
		Cost:     p.calculateCostForTokens(180, request.Model),
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
func (p *ClaudeProvider) ChatStream(ctx context.Context, request *ChatRequest) (<-chan *StreamChunk, error) {
	p.logger.Debug().Str("model", request.Model).Msg("Starting streaming chat request")

	// TODO: Implement actual Claude streaming API call
	// For now, return a stub stream
	stream := make(chan *StreamChunk, 1)
	
	go func() {
		defer close(stream)
		
		// Send a single chunk as stub
		stream <- &StreamChunk{
			Delta: MessageDelta{
				Role:    RoleAssistant,
				Content: "Claude streaming response not yet implemented",
			},
			TokensUsed: TokenUsage{
				TotalTokens: 180,
			},
			Cost:     p.calculateCostForTokens(180, request.Model),
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
func (p *ClaudeProvider) SupportsFunctions() bool {
	return strings.HasPrefix(p.model, "claude-3")
}

// SupportsSystemMessages returns whether the provider supports system messages
func (p *ClaudeProvider) SupportsSystemMessages() bool {
	return true
}

// SupportsImages returns whether the provider supports image inputs
func (p *ClaudeProvider) SupportsImages() bool {
	return strings.HasPrefix(p.model, "claude-3")
}

// SupportsThinking returns whether the provider supports thinking mode
func (p *ClaudeProvider) SupportsThinking() bool {
	return true // Claude supports thinking tags
}

// GetTokenLimit returns the token limit for a specific model
func (p *ClaudeProvider) GetTokenLimit(model string) int {
	switch {
	case strings.HasPrefix(model, "claude-3"):
		return 200000
	default:
		return 100000
	}
}

// CountTokens estimates token count for messages
func (p *ClaudeProvider) CountTokens(messages []Message) (int, error) {
	// TODO: Implement proper token counting for Claude
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
	
	// Rough approximation: 3.5 characters per token for Claude
	return int(float64(totalChars) / 3.5), nil
}

// CalculateCost calculates the cost for a given number of tokens
func (p *ClaudeProvider) CalculateCost(tokens int, model string) (float64, error) {
	return p.calculateCostForTokens(tokens, model), nil
}

// calculateCostForTokens internal helper for cost calculation
func (p *ClaudeProvider) calculateCostForTokens(tokens int, model string) float64 {
	costInfo, exists := p.costTable[model]
	if !exists {
		// Default to Sonnet pricing if model not found
		costInfo = p.costTable["claude-3-sonnet-20240229"]
	}
	
	// Assume 70% input tokens, 30% output tokens for estimation
	inputTokens := float64(tokens) * 0.7
	outputTokens := float64(tokens) * 0.3
	
	inputCost := (inputTokens / 1000) * costInfo.InputCostPer1K
	outputCost := (outputTokens / 1000) * costInfo.OutputCostPer1K
	
	return inputCost + outputCost
}

// Name returns the provider name
func (p *ClaudeProvider) Name() string {
	return ProviderClaude
}

// Models returns available models
func (p *ClaudeProvider) Models() []Model {
	return []Model{
		{
			ID:               "claude-3-opus-20240229",
			Name:             "Claude 3 Opus",
			Description:      "Most powerful model for complex tasks",
			MaxTokens:        200000,
			SupportsFunctions: true,
			SupportsImages:   true,
			SupportsThinking: true,
			CostPer1KTokens:  0.045,
			Tags:             []string{"reasoning", "thinking", "multimodal"},
		},
		{
			ID:               "claude-3-5-sonnet-20241022",
			Name:             "Claude 3.5 Sonnet",
			Description:      "Best balance of intelligence and speed",
			MaxTokens:        200000,
			SupportsFunctions: true,
			SupportsImages:   true,
			SupportsThinking: true,
			CostPer1KTokens:  0.009,
			Tags:             []string{"balanced", "thinking", "multimodal"},
		},
		{
			ID:               "claude-3-sonnet-20240229",
			Name:             "Claude 3 Sonnet",
			Description:      "Good balance of performance and cost",
			MaxTokens:        200000,
			SupportsFunctions: true,
			SupportsImages:   true,
			SupportsThinking: true,
			CostPer1KTokens:  0.009,
			Tags:             []string{"balanced", "thinking", "multimodal"},
		},
		{
			ID:               "claude-3-haiku-20240307",
			Name:             "Claude 3 Haiku",
			Description:      "Fastest and most cost-effective",
			MaxTokens:        200000,
			SupportsFunctions: true,
			SupportsImages:   true,
			SupportsThinking: true,
			CostPer1KTokens:  0.0007,
			Tags:             []string{"fast", "cheap", "multimodal"},
		},
	}
}

// IsAvailable checks if the provider is currently available
func (p *ClaudeProvider) IsAvailable(ctx context.Context) bool {
	// TODO: Implement actual availability check
	// For now, assume available if we have an API key
	return p.apiKey != ""
}

// GetRateLimit returns current rate limit information
func (p *ClaudeProvider) GetRateLimit() *RateLimit {
	return &RateLimit{
		RequestsPerMinute: p.rateLimiter.requestsPerMinute,
		TokensPerMinute:   p.rateLimiter.tokensPerMinute,
		RequestsPerDay:    p.rateLimiter.requestsPerMinute * 60 * 24,
		ResetTime:         p.rateLimiter.lastReset.Add(time.Minute),
	}
}