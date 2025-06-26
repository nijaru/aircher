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

	// Convert our messages to OpenAI format
	openaiMessages := make([]openai.ChatCompletionMessage, 0, len(request.Messages))
	
	// Add system message if provided
	if request.SystemMsg != "" {
		openaiMessages = append(openaiMessages, openai.ChatCompletionMessage{
			Role:    openai.ChatMessageRoleSystem,
			Content: request.SystemMsg,
		})
	}
	
	for _, msg := range request.Messages {
		openaiMsg := openai.ChatCompletionMessage{
			Content: msg.Content,
		}
		
		switch msg.Role {
		case RoleUser:
			openaiMsg.Role = openai.ChatMessageRoleUser
		case RoleAssistant:
			openaiMsg.Role = openai.ChatMessageRoleAssistant
		case RoleSystem:
			openaiMsg.Role = openai.ChatMessageRoleSystem
		case RoleTool:
			openaiMsg.Role = openai.ChatMessageRoleTool
		default:
			openaiMsg.Role = openai.ChatMessageRoleUser
		}
		
		// Handle tool calls
		if len(msg.ToolCalls) > 0 {
			for _, toolCall := range msg.ToolCalls {
				openaiMsg.ToolCalls = append(openaiMsg.ToolCalls, openai.ToolCall{
					ID:   toolCall.ID,
					Type: openai.ToolTypeFunction,
					Function: openai.FunctionCall{
						Name:      toolCall.Function.Name,
						Arguments: toolCall.Function.Arguments,
					},
				})
			}
		}
		
		openaiMessages = append(openaiMessages, openaiMsg)
	}

	// Prepare OpenAI request
	model := request.Model
	if model == "" {
		model = p.model
	}
	
	openaiRequest := openai.ChatCompletionRequest{
		Model:       model,
		Messages:    openaiMessages,
		MaxTokens:   request.MaxTokens,
		Temperature: float32(request.Temperature),
		Stream:      false,
	}
	
	// Convert tools if provided
	if len(request.Tools) > 0 {
		openaiTools := make([]openai.Tool, 0, len(request.Tools))
		for _, tool := range request.Tools {
			openaiTools = append(openaiTools, openai.Tool{
				Type: openai.ToolTypeFunction,
				Function: openai.FunctionDefinition{
					Name:        tool.Function.Name,
					Description: tool.Function.Description,
					Parameters:  tool.Function.Parameters,
				},
			})
		}
		openaiRequest.Tools = openaiTools
	}

	// Make the API call
	openaiResponse, err := p.client.CreateChatCompletion(ctx, openaiRequest)
	if err != nil {
		p.logger.Error().Err(err).Msg("OpenAI API call failed")
		return nil, fmt.Errorf("OpenAI API call failed: %w", err)
	}

	// Convert response back to our format
	if len(openaiResponse.Choices) == 0 {
		return nil, fmt.Errorf("no response choices returned from OpenAI")
	}

	choice := openaiResponse.Choices[0]
	message := Message{
		Role:    RoleAssistant,
		Content: choice.Message.Content,
	}

	// Handle tool calls in response
	if len(choice.Message.ToolCalls) > 0 {
		for _, toolCall := range choice.Message.ToolCalls {
			message.ToolCalls = append(message.ToolCalls, ToolCall{
				ID:   toolCall.ID,
				Type: string(toolCall.Type),
				Function: FunctionCall{
					Name:      toolCall.Function.Name,
					Arguments: toolCall.Function.Arguments,
				},
			})
		}
	}

	tokenUsage := TokenUsage{
		PromptTokens:     openaiResponse.Usage.PromptTokens,
		CompletionTokens: openaiResponse.Usage.CompletionTokens,
		TotalTokens:      openaiResponse.Usage.TotalTokens,
	}

	response := &ChatResponse{
		Message:      message,
		TokensUsed:   tokenUsage,
		Cost:         p.calculateRealCost(tokenUsage, model),
		Duration:     time.Since(start),
		Provider:     p.Name(),
		Model:        model,
		FinishReason: string(choice.FinishReason),
		Metadata: map[string]interface{}{
			"openai_response_id": openaiResponse.ID,
			"created":           openaiResponse.Created,
		},
	}

	p.logger.Debug().
		Int("prompt_tokens", tokenUsage.PromptTokens).
		Int("completion_tokens", tokenUsage.CompletionTokens).
		Float64("cost", response.Cost).
		Dur("duration", response.Duration).
		Msg("OpenAI chat request completed")

	return response, nil
}

// ChatStream sends a chat request and returns a streaming response
func (p *OpenAIProvider) ChatStream(ctx context.Context, request *ChatRequest) (<-chan *StreamChunk, error) {
	p.logger.Debug().Str("model", request.Model).Msg("Starting streaming chat request")

	// Convert our messages to OpenAI format
	openaiMessages := make([]openai.ChatCompletionMessage, 0, len(request.Messages))
	
	// Add system message if provided
	if request.SystemMsg != "" {
		openaiMessages = append(openaiMessages, openai.ChatCompletionMessage{
			Role:    openai.ChatMessageRoleSystem,
			Content: request.SystemMsg,
		})
	}
	
	for _, msg := range request.Messages {
		openaiMsg := openai.ChatCompletionMessage{
			Content: msg.Content,
		}
		
		switch msg.Role {
		case RoleUser:
			openaiMsg.Role = openai.ChatMessageRoleUser
		case RoleAssistant:
			openaiMsg.Role = openai.ChatMessageRoleAssistant
		case RoleSystem:
			openaiMsg.Role = openai.ChatMessageRoleSystem
		case RoleTool:
			openaiMsg.Role = openai.ChatMessageRoleTool
		default:
			openaiMsg.Role = openai.ChatMessageRoleUser
		}
		
		// Handle tool calls
		if len(msg.ToolCalls) > 0 {
			for _, toolCall := range msg.ToolCalls {
				openaiMsg.ToolCalls = append(openaiMsg.ToolCalls, openai.ToolCall{
					ID:   toolCall.ID,
					Type: openai.ToolTypeFunction,
					Function: openai.FunctionCall{
						Name:      toolCall.Function.Name,
						Arguments: toolCall.Function.Arguments,
					},
				})
			}
		}
		
		openaiMessages = append(openaiMessages, openaiMsg)
	}

	// Prepare OpenAI streaming request
	model := request.Model
	if model == "" {
		model = p.model
	}
	
	openaiRequest := openai.ChatCompletionRequest{
		Model:       model,
		Messages:    openaiMessages,
		MaxTokens:   request.MaxTokens,
		Temperature: float32(request.Temperature),
		Stream:      true,
	}
	
	// Convert tools if provided
	if len(request.Tools) > 0 {
		openaiTools := make([]openai.Tool, 0, len(request.Tools))
		for _, tool := range request.Tools {
			openaiTools = append(openaiTools, openai.Tool{
				Type: openai.ToolTypeFunction,
				Function: openai.FunctionDefinition{
					Name:        tool.Function.Name,
					Description: tool.Function.Description,
					Parameters:  tool.Function.Parameters,
				},
			})
		}
		openaiRequest.Tools = openaiTools
	}

	// Create streaming response
	streamResponse, err := p.client.CreateChatCompletionStream(ctx, openaiRequest)
	if err != nil {
		p.logger.Error().Err(err).Msg("OpenAI streaming API call failed")
		return nil, fmt.Errorf("OpenAI streaming API call failed: %w", err)
	}

	// Create our stream channel
	stream := make(chan *StreamChunk, 10)
	
	go func() {
		defer close(stream)
		defer streamResponse.Close()

		var totalTokens TokenUsage
		var responseID string
		start := time.Now()
		
		for {
			response, err := streamResponse.Recv()
			if err != nil {
				if err.Error() == "EOF" {
					// Send final chunk with completion info
					stream <- &StreamChunk{
						Delta:       MessageDelta{},
						TokensUsed:  totalTokens,
						Cost:        p.calculateRealCost(totalTokens, model),
						Provider:    p.Name(),
						Model:       model,
						Done:        true,
						Metadata: map[string]interface{}{
							"openai_response_id": responseID,
							"duration":          time.Since(start),
						},
					}
					break
				}
				
				p.logger.Error().Err(err).Msg("Error receiving streaming response")
				stream <- &StreamChunk{
					Error:    err,
					Provider: p.Name(),
					Model:    model,
					Done:     true,
				}
				return
			}

			if responseID == "" {
				responseID = response.ID
			}

			// Note: Streaming responses don't include usage information per chunk
			// We'll calculate token usage at the end or estimate it

			// Process each choice
			for _, choice := range response.Choices {
				chunk := &StreamChunk{
					Provider: p.Name(),
					Model:    model,
					Done:     false,
				}

				if choice.Delta.Role != "" {
					chunk.Delta.Role = string(choice.Delta.Role)
				}

				if choice.Delta.Content != "" {
					chunk.Delta.Content = choice.Delta.Content
				}

				// Handle tool calls in delta
				if len(choice.Delta.ToolCalls) > 0 {
					for _, toolCall := range choice.Delta.ToolCalls {
						chunk.Delta.ToolCalls = append(chunk.Delta.ToolCalls, ToolCall{
							ID:   toolCall.ID,
							Type: string(toolCall.Type),
							Function: FunctionCall{
								Name:      toolCall.Function.Name,
								Arguments: toolCall.Function.Arguments,
							},
						})
					}
				}

				if choice.FinishReason != "" {
					chunk.FinishReason = string(choice.FinishReason)
					chunk.Done = true
				}

				stream <- chunk
			}
		}

		p.logger.Debug().
			Int("total_tokens", totalTokens.TotalTokens).
			Dur("duration", time.Since(start)).
			Msg("OpenAI streaming request completed")
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

// calculateCostForTokens internal helper for cost calculation (estimation)
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

// calculateRealCost calculates actual cost using real token usage
func (p *OpenAIProvider) calculateRealCost(tokens TokenUsage, model string) float64 {
	costInfo, exists := p.costTable[model]
	if !exists {
		// Default to GPT-4 pricing if model not found
		costInfo = p.costTable["gpt-4"]
	}
	
	inputCost := (float64(tokens.PromptTokens) / 1000) * costInfo.InputCostPer1K
	outputCost := (float64(tokens.CompletionTokens) / 1000) * costInfo.OutputCostPer1K
	
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