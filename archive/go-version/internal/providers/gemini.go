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

// GeminiProvider implements the LLMProvider interface for Google Gemini
type GeminiProvider struct {
	client      *GeminiClient
	model       string
	apiKey      string
	project     string
	costTable   map[string]CostInfo
	rateLimiter *ProviderRateLimiter
	logger      zerolog.Logger
	config      *config.GeminiProviderConfig
}

// GeminiClient represents the Google Gemini API client
type GeminiClient struct {
	apiKey  string
	project string
	baseURL string
}

// NewGeminiProvider creates a new Gemini provider instance
func NewGeminiProvider(cfg *config.GeminiProviderConfig, logger zerolog.Logger) (*GeminiProvider, error) {
	apiKey := os.Getenv(cfg.APIKeyEnv)
	if apiKey == "" {
		return nil, fmt.Errorf("Gemini API key not found in environment variable %s", cfg.APIKeyEnv)
	}

	client := &GeminiClient{
		apiKey:  apiKey,
		project: cfg.Project,
		baseURL: "https://generativelanguage.googleapis.com",
	}

	provider := &GeminiProvider{
		client:  client,
		model:   cfg.Model,
		apiKey:  apiKey,
		project: cfg.Project,
		logger:  logger.With().Str("provider", "gemini").Logger(),
		config:  cfg,
		rateLimiter: &ProviderRateLimiter{
			requestsPerMinute: 60, // Default Gemini rate limits
			tokensPerMinute:   32000,
			lastReset:         time.Now(),
		},
	}

	// Initialize cost table
	provider.initializeCostTable()

	return provider, nil
}

// initializeCostTable sets up pricing information for Gemini models
func (p *GeminiProvider) initializeCostTable() {
	p.costTable = map[string]CostInfo{
		"gemini-pro": {
			InputCostPer1K:  0.0005,
			OutputCostPer1K: 0.0015,
		},
		"gemini-pro-vision": {
			InputCostPer1K:  0.0005,
			OutputCostPer1K: 0.0015,
		},
		"gemini-1.5-pro": {
			InputCostPer1K:  0.001,
			OutputCostPer1K: 0.003,
		},
		"gemini-1.5-flash": {
			InputCostPer1K:  0.00015,
			OutputCostPer1K: 0.0006,
		},
	}
}

// Chat sends a chat request and returns a complete response
func (p *GeminiProvider) Chat(ctx context.Context, request *ChatRequest) (*ChatResponse, error) {
	start := time.Now()
	p.logger.Debug().Str("model", request.Model).Int("messages", len(request.Messages)).Msg("Sending chat request")

	// TODO: Implement actual Gemini API call
	// For now, return a stub response
	response := &ChatResponse{
		Message: Message{
			Role:    RoleAssistant,
			Content: "Gemini provider response not yet implemented",
		},
		TokensUsed: TokenUsage{
			PromptTokens:     110,
			CompletionTokens: 55,
			TotalTokens:      165,
		},
		Cost:     p.calculateCostForTokens(165, request.Model),
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
func (p *GeminiProvider) ChatStream(ctx context.Context, request *ChatRequest) (<-chan *StreamChunk, error) {
	p.logger.Debug().Str("model", request.Model).Msg("Starting streaming chat request")

	// TODO: Implement actual Gemini streaming API call
	// For now, return a stub stream
	stream := make(chan *StreamChunk, 1)
	
	go func() {
		defer close(stream)
		
		// Send a single chunk as stub
		stream <- &StreamChunk{
			Delta: MessageDelta{
				Role:    RoleAssistant,
				Content: "Gemini streaming response not yet implemented",
			},
			TokensUsed: TokenUsage{
				TotalTokens: 165,
			},
			Cost:     p.calculateCostForTokens(165, request.Model),
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
func (p *GeminiProvider) SupportsFunctions() bool {
	return strings.Contains(p.model, "1.5") || strings.Contains(p.model, "pro")
}

// SupportsSystemMessages returns whether the provider supports system messages
func (p *GeminiProvider) SupportsSystemMessages() bool {
	return true
}

// SupportsImages returns whether the provider supports image inputs
func (p *GeminiProvider) SupportsImages() bool {
	return strings.Contains(p.model, "vision") || strings.Contains(p.model, "1.5")
}

// SupportsThinking returns whether the provider supports thinking mode
func (p *GeminiProvider) SupportsThinking() bool {
	return false // Gemini doesn't have explicit thinking mode
}

// GetTokenLimit returns the token limit for a specific model
func (p *GeminiProvider) GetTokenLimit(model string) int {
	switch {
	case strings.Contains(model, "1.5"):
		return 1000000 // Gemini 1.5 has very large context window
	case strings.Contains(model, "pro"):
		return 30720
	default:
		return 30720
	}
}

// CountTokens estimates token count for messages
func (p *GeminiProvider) CountTokens(messages []Message) (int, error) {
	// TODO: Implement proper token counting for Gemini
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
	
	// Rough approximation: 4 characters per token for Gemini
	return totalChars / 4, nil
}

// CalculateCost calculates the cost for a given number of tokens
func (p *GeminiProvider) CalculateCost(tokens int, model string) (float64, error) {
	return p.calculateCostForTokens(tokens, model), nil
}

// calculateCostForTokens internal helper for cost calculation
func (p *GeminiProvider) calculateCostForTokens(tokens int, model string) float64 {
	costInfo, exists := p.costTable[model]
	if !exists {
		// Default to Gemini Pro pricing if model not found
		costInfo = p.costTable["gemini-pro"]
	}
	
	// Assume 70% input tokens, 30% output tokens for estimation
	inputTokens := float64(tokens) * 0.7
	outputTokens := float64(tokens) * 0.3
	
	inputCost := (inputTokens / 1000) * costInfo.InputCostPer1K
	outputCost := (outputTokens / 1000) * costInfo.OutputCostPer1K
	
	return inputCost + outputCost
}

// Name returns the provider name
func (p *GeminiProvider) Name() string {
	return ProviderGemini
}

// Models returns available models
func (p *GeminiProvider) Models() []Model {
	return []Model{
		{
			ID:               "gemini-1.5-pro",
			Name:             "Gemini 1.5 Pro",
			Description:      "Most capable model with large context window",
			MaxTokens:        1000000,
			SupportsFunctions: true,
			SupportsImages:   true,
			SupportsThinking: false,
			CostPer1KTokens:  0.002,
			Tags:             []string{"reasoning", "multimodal", "large-context"},
		},
		{
			ID:               "gemini-1.5-flash",
			Name:             "Gemini 1.5 Flash",
			Description:      "Fast and efficient model",
			MaxTokens:        1000000,
			SupportsFunctions: true,
			SupportsImages:   true,
			SupportsThinking: false,
			CostPer1KTokens:  0.0004,
			Tags:             []string{"fast", "cheap", "multimodal", "large-context"},
		},
		{
			ID:               "gemini-pro",
			Name:             "Gemini Pro",
			Description:      "Good balance of performance and cost",
			MaxTokens:        30720,
			SupportsFunctions: true,
			SupportsImages:   false,
			SupportsThinking: false,
			CostPer1KTokens:  0.001,
			Tags:             []string{"balanced", "function-calling"},
		},
		{
			ID:               "gemini-pro-vision",
			Name:             "Gemini Pro Vision",
			Description:      "Multimodal model with vision capabilities",
			MaxTokens:        30720,
			SupportsFunctions: true,
			SupportsImages:   true,
			SupportsThinking: false,
			CostPer1KTokens:  0.001,
			Tags:             []string{"multimodal", "vision", "function-calling"},
		},
	}
}

// IsAvailable checks if the provider is currently available
func (p *GeminiProvider) IsAvailable(ctx context.Context) bool {
	// TODO: Implement actual availability check
	// For now, assume available if we have an API key
	return p.apiKey != ""
}

// GetRateLimit returns current rate limit information
func (p *GeminiProvider) GetRateLimit() *RateLimit {
	return &RateLimit{
		RequestsPerMinute: p.rateLimiter.requestsPerMinute,
		TokensPerMinute:   p.rateLimiter.tokensPerMinute,
		RequestsPerDay:    p.rateLimiter.requestsPerMinute * 60 * 24,
		ResetTime:         p.rateLimiter.lastReset.Add(time.Minute),
	}
}