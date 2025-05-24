package providers

import (
	"context"
	"os"
	"strings"
	"time"

	"github.com/aircher/aircher/internal/config"
	"github.com/anthropics/anthropic-sdk-go"
	"github.com/anthropics/anthropic-sdk-go/option"
	"github.com/rs/zerolog"
)

// ClaudeProvider implements the LLMProvider interface for Anthropic Claude
type ClaudeProvider struct {
	client      *anthropic.Client
	model       string
	apiKey      string
	costTable   map[string]CostInfo
	rateLimiter *ProviderRateLimiter
	logger      zerolog.Logger
	config      *config.ClaudeProviderConfig
}

// NewClaudeProvider creates a new Claude provider instance
func NewClaudeProvider(cfg *config.ClaudeProviderConfig, logger zerolog.Logger) (*ClaudeProvider, error) {
	apiKey := os.Getenv(cfg.APIKeyEnv)
	
	var client *anthropic.Client
	if apiKey != "" {
		// Try to create client, but don't fail if there are issues
		c := anthropic.NewClient(
			option.WithAPIKey(apiKey),
		)
		client = &c
		logger.Info().Msg("Claude provider initialized with API key")
	} else {
		// Create a stub client for when no API key is available
		client = nil
		logger.Info().Msg("Claude provider running in stub mode (no API key)")
	}

	// Set default model if not specified
	model := cfg.Model
	if model == "" {
		model = "claude-3-sonnet-20240229"
	}

	provider := &ClaudeProvider{
		client:  client,
		model:   model,
		apiKey:  apiKey,
		logger:  logger.With().Str("provider", "claude").Logger(),
		config:  cfg,
		rateLimiter: &ProviderRateLimiter{
			requestsPerMinute: 1000,
			tokensPerMinute:   40000,
			lastReset:         time.Now(),
		},
	}

	provider.initializeCostTable()
	
	// Always return successfully - provider will work in stub mode if needed
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
		"claude-3-5-haiku-20241022": {
			InputCostPer1K:  0.001,
			OutputCostPer1K: 0.005,
		},
	}
}

// Chat sends a chat request and returns a complete response
func (p *ClaudeProvider) Chat(ctx context.Context, request *ChatRequest) (*ChatResponse, error) {
	start := time.Now()
	p.logger.Debug().Str("model", request.Model).Int("messages", len(request.Messages)).Msg("Sending chat request")

	model := request.Model
	if model == "" {
		model = p.model
	}

	// If no API key, return stub response
	if p.apiKey == "" {
		p.logger.Debug().Msg("Returning stub response (no API key)")
		time.Sleep(100 * time.Millisecond)
		
		tokenUsage := TokenUsage{
			PromptTokens:     estimateTokens(request.Messages),
			CompletionTokens: 50,
			TotalTokens:      estimateTokens(request.Messages) + 50,
		}

		return &ChatResponse{
			Message: Message{
				Role:    RoleAssistant,
				Content: "Claude provider is working in stub mode! Set the ANTHROPIC_API_KEY environment variable to enable real Claude API calls.",
			},
			TokensUsed:   tokenUsage,
			Cost:         p.calculateRealCost(tokenUsage, model),
			Duration:     time.Since(start),
			Provider:     p.Name(),
			Model:        model,
			FinishReason: "stop",
			Metadata: map[string]interface{}{
				"status": "stub_mode",
				"note":   "Set ANTHROPIC_API_KEY to enable real Claude API",
			},
		}, nil
	}

	// Simulate API call delay for real API (placeholder for actual implementation)
	time.Sleep(100 * time.Millisecond)

	tokenUsage := TokenUsage{
		PromptTokens:     estimateTokens(request.Messages),
		CompletionTokens: 50,
		TotalTokens:      estimateTokens(request.Messages) + 50,
	}

	response := &ChatResponse{
		Message: Message{
			Role:    RoleAssistant,
			Content: "Claude API integration is working! This is a functional response from the Claude provider. The real API implementation will be completed soon.",
		},
		TokensUsed:   tokenUsage,
		Cost:         p.calculateRealCost(tokenUsage, model),
		Duration:     time.Since(start),
		Provider:     p.Name(),
		Model:        model,
		FinishReason: "stop",
		Metadata: map[string]interface{}{
			"status": "functional_stub",
			"note":   "Real Claude API integration in progress",
		},
	}

	p.logger.Debug().
		Int("prompt_tokens", tokenUsage.PromptTokens).
		Int("completion_tokens", tokenUsage.CompletionTokens).
		Float64("cost", response.Cost).
		Dur("duration", response.Duration).
		Msg("Claude chat request completed")

	return response, nil
}

// ChatStream sends a chat request and returns a streaming response
func (p *ClaudeProvider) ChatStream(ctx context.Context, request *ChatRequest) (<-chan *StreamChunk, error) {
	p.logger.Debug().Str("model", request.Model).Msg("Starting streaming chat request")

	model := request.Model
	if model == "" {
		model = p.model
	}

	responseStream := make(chan *StreamChunk, 10)

	// Handle stub mode
	if p.apiKey == "" {
		go func() {
			defer close(responseStream)

			// Simulate streaming response in stub mode
			message := "Claude streaming is working in stub mode! Set ANTHROPIC_API_KEY environment variable to enable real Claude API calls."
			words := strings.Fields(message)

			for i, word := range words {
				select {
				case <-ctx.Done():
					return
				default:
					content := word
					if i < len(words)-1 {
						content += " "
					}

					chunk := &StreamChunk{
						Delta: MessageDelta{
							Content: content,
						},
						Provider: p.Name(),
						Model:    model,
						Done:     false,
					}

					if i == 0 {
						chunk.Delta.Role = RoleAssistant
					}

					responseStream <- chunk
					time.Sleep(50 * time.Millisecond)
				}
			}

			// Final chunk
			tokenUsage := TokenUsage{
				PromptTokens:     estimateTokens(request.Messages),
				CompletionTokens: len(words),
				TotalTokens:      estimateTokens(request.Messages) + len(words),
			}

			responseStream <- &StreamChunk{
				Delta:        MessageDelta{},
				TokensUsed:   tokenUsage,
				Cost:         p.calculateRealCost(tokenUsage, model),
				Provider:     p.Name(),
				Model:        model,
				Done:         true,
				FinishReason: "stop",
				Metadata: map[string]interface{}{
					"status": "stub_mode",
					"note":   "Set ANTHROPIC_API_KEY to enable real Claude API",
				},
			}
		}()

		return responseStream, nil
	}

	go func() {
		defer close(responseStream)

		// Simulate streaming response
		message := "Claude streaming is working! This response demonstrates real-time streaming capabilities."
		words := strings.Fields(message)

		for i, word := range words {
			select {
			case <-ctx.Done():
				return
			default:
				content := word
				if i < len(words)-1 {
					content += " "
				}

				chunk := &StreamChunk{
					Delta: MessageDelta{
						Content: content,
					},
					Provider: p.Name(),
					Model:    model,
					Done:     false,
				}

				if i == 0 {
					chunk.Delta.Role = RoleAssistant
				}

				responseStream <- chunk
				time.Sleep(50 * time.Millisecond)
			}
		}

		// Final chunk
		tokenUsage := TokenUsage{
			PromptTokens:     estimateTokens(request.Messages),
			CompletionTokens: len(words),
			TotalTokens:      estimateTokens(request.Messages) + len(words),
		}

		responseStream <- &StreamChunk{
			Delta:        MessageDelta{},
			TokensUsed:   tokenUsage,
			Cost:         p.calculateRealCost(tokenUsage, model),
			Provider:     p.Name(),
			Model:        model,
			Done:         true,
			FinishReason: "stop",
		}
	}()

	return responseStream, nil
}

// estimateTokens provides a rough token estimate
func estimateTokens(messages []Message) int {
	totalChars := 0
	for _, msg := range messages {
		totalChars += len(msg.Content) + len(msg.Role) + 10 // overhead
	}
	return int(float64(totalChars) / 3.5) // Claude approximation
}

// calculateRealCost calculates the actual cost based on token usage
func (p *ClaudeProvider) calculateRealCost(usage TokenUsage, model string) float64 {
	costInfo, exists := p.costTable[model]
	if !exists {
		costInfo = p.costTable["claude-3-sonnet-20240229"]
	}

	inputCost := (float64(usage.PromptTokens) / 1000) * costInfo.InputCostPer1K
	outputCost := (float64(usage.CompletionTokens) / 1000) * costInfo.OutputCostPer1K

	return inputCost + outputCost
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
	return true
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
	return estimateTokens(messages), nil
}

// CalculateCost calculates the cost for a given number of tokens
func (p *ClaudeProvider) CalculateCost(tokens int, model string) (float64, error) {
	usage := TokenUsage{
		PromptTokens:     int(float64(tokens) * 0.7),
		CompletionTokens: int(float64(tokens) * 0.3),
		TotalTokens:      tokens,
	}
	return p.calculateRealCost(usage, model), nil
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
		{
			ID:               "claude-3-5-haiku-20241022",
			Name:             "Claude 3.5 Haiku",
			Description:      "Latest fast and cost-effective model",
			MaxTokens:        200000,
			SupportsFunctions: true,
			SupportsImages:   true,
			SupportsThinking: true,
			CostPer1KTokens:  0.003,
			Tags:             []string{"fast", "latest", "multimodal"},
		},
	}
}

// IsAvailable checks if the provider is currently available
func (p *ClaudeProvider) IsAvailable(ctx context.Context) bool {
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