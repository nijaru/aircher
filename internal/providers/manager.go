package providers

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/aircher/aircher/internal/config"
	"github.com/rs/zerolog"
)

// Manager handles multiple LLM providers with intelligent routing
type Manager struct {
	providers       map[string]LLMProvider
	defaultProvider string
	routingEngine   *RoutingEngine
	healthChecker   *HealthChecker
	costTracker     *CostTracker
	config          *config.Config
	logger          zerolog.Logger
	mu              sync.RWMutex
}

// RoutingEngine handles provider selection logic
type RoutingEngine struct {
	rules        []RoutingRule
	fallbacks    map[string][]string
	costOptimizer *CostOptimizer
	healthChecker *HealthChecker
	logger       zerolog.Logger
}

// RoutingRule defines conditions for provider selection
type RoutingRule struct {
	Condition   RoutingCondition
	Provider    string
	Priority    int
	Explanation string
}

// RoutingCondition defines when a rule should apply
type RoutingCondition struct {
	RequiresFunctions bool
	RequiresImages    bool
	RequiresThinking  bool
	MaxCostPer1K      float64
	MinTokenLimit     int
	PreferredProviders []string
	TaskType          string
	TimeOfDay         *TimeRange
}

// TimeRange represents a time range for routing rules
type TimeRange struct {
	Start time.Time
	End   time.Time
}

// HealthChecker monitors provider availability and performance
type HealthChecker struct {
	healthStatus map[string]*ProviderHealth
	checkInterval time.Duration
	logger       zerolog.Logger
	mu           sync.RWMutex
}

// CostTracker tracks usage and costs across providers
type CostTracker struct {
	dailyUsage   map[string]*DailyUsage
	monthlyUsage map[string]*MonthlyUsage
	budgets      *BudgetConfig
	logger       zerolog.Logger
	mu           sync.RWMutex
}

// CostOptimizer helps select cost-effective providers
type CostOptimizer struct {
	costTracker *CostTracker
	providers   map[string]LLMProvider
	logger      zerolog.Logger
}

// DailyUsage tracks daily provider usage
type DailyUsage struct {
	Date         time.Time
	Provider     string
	Requests     int
	Tokens       int
	Cost         float64
	LastUpdated  time.Time
}

// MonthlyUsage tracks monthly provider usage
type MonthlyUsage struct {
	Month       time.Time
	Provider    string
	Requests    int
	Tokens      int
	Cost        float64
	LastUpdated time.Time
}

// BudgetConfig defines spending limits
type BudgetConfig struct {
	MonthlyBudget  float64
	DailyLimit     float64
	AlertThreshold float64
	TrackByProvider bool
}

// ProviderSelection represents the result of provider routing
type ProviderSelection struct {
	Provider    string
	Reasoning   string
	Confidence  float64
	Fallbacks   []string
	CostEstimate float64
}

// NewManager creates a new provider manager
func NewManager(cfg *config.Config, logger zerolog.Logger) (*Manager, error) {
	manager := &Manager{
		providers: make(map[string]LLMProvider),
		config:    cfg,
		logger:    logger.With().Str("component", "provider_manager").Logger(),
	}

	// Initialize health checker
	manager.healthChecker = &HealthChecker{
		healthStatus:  make(map[string]*ProviderHealth),
		checkInterval: 5 * time.Minute,
		logger:        logger.With().Str("component", "health_checker").Logger(),
	}

	// Initialize cost tracker
	budgets := &BudgetConfig{
		MonthlyBudget:   cfg.Costs.MonthlyBudget,
		DailyLimit:      cfg.Costs.DailyLimit,
		AlertThreshold:  cfg.Costs.AlertThreshold,
		TrackByProvider: cfg.Costs.TrackByProvider,
	}

	manager.costTracker = &CostTracker{
		dailyUsage:   make(map[string]*DailyUsage),
		monthlyUsage: make(map[string]*MonthlyUsage),
		budgets:      budgets,
		logger:       logger.With().Str("component", "cost_tracker").Logger(),
	}

	// Initialize cost optimizer
	costOptimizer := &CostOptimizer{
		costTracker: manager.costTracker,
		providers:   manager.providers,
		logger:      logger.With().Str("component", "cost_optimizer").Logger(),
	}

	// Initialize routing engine
	manager.routingEngine = &RoutingEngine{
		rules:         manager.createDefaultRoutingRules(),
		fallbacks:     manager.createDefaultFallbacks(),
		costOptimizer: costOptimizer,
		healthChecker: manager.healthChecker,
		logger:        logger.With().Str("component", "routing_engine").Logger(),
	}

	// Set default provider
	manager.defaultProvider = cfg.Providers.Default

	// Initialize configured providers
	if err := manager.initializeProviders(cfg); err != nil {
		return nil, fmt.Errorf("failed to initialize providers: %w", err)
	}

	// Start health checking
	go manager.healthChecker.start(context.Background(), manager.providers)

	return manager, nil
}

// initializeProviders sets up all configured providers
func (m *Manager) initializeProviders(cfg *config.Config) error {
	// Initialize OpenAI provider
	if cfg.Providers.OpenAI.APIKeyEnv != "" {
		provider, err := NewOpenAIProvider(&cfg.Providers.OpenAI, m.logger)
		if err != nil {
			m.logger.Warn().Err(err).Msg("Failed to initialize OpenAI provider")
		} else {
			m.providers[ProviderOpenAI] = provider
		}
	}

	// Initialize Claude provider
	if cfg.Providers.Claude.APIKeyEnv != "" {
		provider, err := NewClaudeProvider(&cfg.Providers.Claude, m.logger)
		if err != nil {
			m.logger.Warn().Err(err).Msg("Failed to initialize Claude provider")
		} else {
			m.providers[ProviderClaude] = provider
		}
	}

	// Initialize Gemini provider
	if cfg.Providers.Gemini.APIKeyEnv != "" {
		provider, err := NewGeminiProvider(&cfg.Providers.Gemini, m.logger)
		if err != nil {
			m.logger.Warn().Err(err).Msg("Failed to initialize Gemini provider")
		} else {
			m.providers[ProviderGemini] = provider
		}
	}

	// Initialize Ollama provider
	if cfg.Providers.Ollama.BaseURL != "" {
		provider, err := NewOllamaProvider(&cfg.Providers.Ollama, m.logger)
		if err != nil {
			m.logger.Warn().Err(err).Msg("Failed to initialize Ollama provider")
		} else {
			m.providers[ProviderOllama] = provider
		}
	}

	if len(m.providers) == 0 {
		return fmt.Errorf("no providers configured successfully")
	}

	m.logger.Info().Int("count", len(m.providers)).Msg("Initialized providers")
	return nil
}

// Chat sends a chat request using the best available provider
func (m *Manager) Chat(ctx context.Context, request *ChatRequest) (*ChatResponse, error) {
	// Select provider
	selection, err := m.routingEngine.SelectProvider(ctx, request)
	if err != nil {
		return nil, fmt.Errorf("failed to select provider: %w", err)
	}

	// Get provider
	provider, exists := m.providers[selection.Provider]
	if !exists {
		return nil, fmt.Errorf("selected provider %s not available", selection.Provider)
	}

	// Update request with selected provider
	request.Provider = selection.Provider

	// Check budget limits
	if err := m.costTracker.CheckBudget(selection.Provider, selection.CostEstimate); err != nil {
		return nil, fmt.Errorf("budget check failed: %w", err)
	}

	// Make the request
	response, err := provider.Chat(ctx, request)
	if err != nil {
		// Try fallback providers
		for _, fallbackProvider := range selection.Fallbacks {
			if fallback, exists := m.providers[fallbackProvider]; exists {
				m.logger.Warn().
					Str("original_provider", selection.Provider).
					Str("fallback_provider", fallbackProvider).
					Err(err).
					Msg("Trying fallback provider")

				request.Provider = fallbackProvider
				if response, err = fallback.Chat(ctx, request); err == nil {
					break
				}
			}
		}

		if err != nil {
			return nil, fmt.Errorf("all providers failed: %w", err)
		}
	}

	// Track usage
	m.costTracker.RecordUsage(response.Provider, response.TokensUsed, response.Cost)

	return response, nil
}

// ChatStream sends a streaming chat request using the best available provider
func (m *Manager) ChatStream(ctx context.Context, request *ChatRequest) (<-chan *StreamChunk, error) {
	// Select provider
	selection, err := m.routingEngine.SelectProvider(ctx, request)
	if err != nil {
		return nil, fmt.Errorf("failed to select provider: %w", err)
	}

	// Get provider
	provider, exists := m.providers[selection.Provider]
	if !exists {
		return nil, fmt.Errorf("selected provider %s not available", selection.Provider)
	}

	// Update request with selected provider
	request.Provider = selection.Provider
	request.Stream = true

	// Check budget limits
	if err := m.costTracker.CheckBudget(selection.Provider, selection.CostEstimate); err != nil {
		return nil, fmt.Errorf("budget check failed: %w", err)
	}

	// Make the streaming request
	stream, err := provider.ChatStream(ctx, request)
	if err != nil {
		return nil, fmt.Errorf("streaming request failed: %w", err)
	}

	// Wrap stream to track usage
	return m.wrapStreamForTracking(stream, selection.Provider), nil
}

// wrapStreamForTracking wraps a stream to track token usage and costs
func (m *Manager) wrapStreamForTracking(stream <-chan *StreamChunk, provider string) <-chan *StreamChunk {
	wrappedStream := make(chan *StreamChunk)

	go func() {
		defer close(wrappedStream)

		var totalTokens TokenUsage
		var totalCost float64

		for chunk := range stream {
			wrappedStream <- chunk

			// Accumulate usage
			totalTokens.PromptTokens += chunk.TokensUsed.PromptTokens
			totalTokens.CompletionTokens += chunk.TokensUsed.CompletionTokens
			totalTokens.TotalTokens += chunk.TokensUsed.TotalTokens
			totalCost += chunk.Cost

			// Record final usage when stream is done
			if chunk.Done {
				m.costTracker.RecordUsage(provider, totalTokens, totalCost)
			}
		}
	}()

	return wrappedStream
}

// GetProvider returns a specific provider by name
func (m *Manager) GetProvider(name string) (LLMProvider, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	provider, exists := m.providers[name]
	if !exists {
		return nil, fmt.Errorf("provider %s not found", name)
	}

	return provider, nil
}

// GetAvailableProviders returns a list of available provider names
func (m *Manager) GetAvailableProviders() []string {
	m.mu.RLock()
	defer m.mu.RUnlock()

	providers := make([]string, 0, len(m.providers))
	for name := range m.providers {
		providers = append(providers, name)
	}

	return providers
}

// GetProviderHealth returns health status for all providers
func (m *Manager) GetProviderHealth() map[string]*ProviderHealth {
	return m.healthChecker.GetStatus()
}

// GetUsageStats returns current usage statistics
func (m *Manager) GetUsageStats() map[string]interface{} {
	return m.costTracker.GetStats()
}

// createDefaultRoutingRules creates the default routing rules
func (m *Manager) createDefaultRoutingRules() []RoutingRule {
	return []RoutingRule{
		{
			Condition: RoutingCondition{
				RequiresFunctions: true,
			},
			Provider:    ProviderOpenAI,
			Priority:    100,
			Explanation: "OpenAI has excellent function calling support",
		},
		{
			Condition: RoutingCondition{
				RequiresThinking: true,
			},
			Provider:    ProviderClaude,
			Priority:    90,
			Explanation: "Claude supports thinking mode",
		},
		{
			Condition: RoutingCondition{
				RequiresImages: true,
			},
			Provider:    ProviderGemini,
			Priority:    85,
			Explanation: "Gemini has strong multimodal capabilities",
		},
		{
			Condition: RoutingCondition{
				MaxCostPer1K: 0.01,
			},
			Provider:    ProviderOllama,
			Priority:    80,
			Explanation: "Ollama is free for local inference",
		},
	}
}

// createDefaultFallbacks creates the default fallback chains
func (m *Manager) createDefaultFallbacks() map[string][]string {
	return map[string][]string{
		ProviderOpenAI: {ProviderClaude, ProviderGemini, ProviderOllama},
		ProviderClaude: {ProviderOpenAI, ProviderGemini, ProviderOllama},
		ProviderGemini: {ProviderOpenAI, ProviderClaude, ProviderOllama},
		ProviderOllama: {ProviderOpenAI, ProviderClaude, ProviderGemini},
	}
}

// SelectProvider selects the best provider for a request
func (re *RoutingEngine) SelectProvider(ctx context.Context, request *ChatRequest) (*ProviderSelection, error) {
	// If a specific provider is requested, use it
	if request.Provider != "" {
		return &ProviderSelection{
			Provider:     request.Provider,
			Reasoning:    "Explicitly requested",
			Confidence:   1.0,
			Fallbacks:    re.fallbacks[request.Provider],
			CostEstimate: 0.0, // Will be calculated by cost optimizer
		}, nil
	}

	// Apply routing rules
	candidates := make([]ProviderSelection, 0)

	for _, rule := range re.rules {
		if re.ruleMatches(rule.Condition, request) {
			candidates = append(candidates, ProviderSelection{
				Provider:    rule.Provider,
				Reasoning:   rule.Explanation,
				Confidence:  float64(rule.Priority) / 100.0,
				Fallbacks:   re.fallbacks[rule.Provider],
			})
		}
	}

	// If no rules match, use default provider logic
	if len(candidates) == 0 {
		return re.selectDefaultProvider()
	}

	// Select best candidate based on health, cost, and priority
	bestCandidate := re.selectBestCandidate(candidates, request)
	
	// Calculate cost estimate
	bestCandidate.CostEstimate = re.costOptimizer.EstimateCost(bestCandidate.Provider, request)

	return &bestCandidate, nil
}

// ruleMatches checks if a routing condition matches the request
func (re *RoutingEngine) ruleMatches(condition RoutingCondition, request *ChatRequest) bool {
	// Check function requirement
	if condition.RequiresFunctions && len(request.Tools) == 0 {
		return false
	}

	// Check image requirement
	if condition.RequiresImages {
		hasImages := false
		for _, msg := range request.Messages {
			if len(msg.Images) > 0 {
				hasImages = true
				break
			}
		}
		if !hasImages {
			return false
		}
	}

	// Check thinking requirement
	if condition.RequiresThinking && !request.Thinking {
		return false
	}

	// Add more condition checks as needed

	return true
}

// selectDefaultProvider selects a default provider when no rules match
func (re *RoutingEngine) selectDefaultProvider() (*ProviderSelection, error) {
	// Get healthy providers
	healthyProviders := re.healthChecker.GetHealthyProviders()
	if len(healthyProviders) == 0 {
		return nil, fmt.Errorf("no healthy providers available")
	}

	// Use the first healthy provider
	provider := healthyProviders[0]
	
	return &ProviderSelection{
		Provider:    provider,
		Reasoning:   "Default provider selection",
		Confidence:  0.5,
		Fallbacks:   re.fallbacks[provider],
	}, nil
}

// selectBestCandidate selects the best candidate from available options
func (re *RoutingEngine) selectBestCandidate(candidates []ProviderSelection, request *ChatRequest) ProviderSelection {
	if len(candidates) == 1 {
		return candidates[0]
	}

	// Filter by health
	healthyProviders := re.healthChecker.GetHealthyProviders()
	healthyMap := make(map[string]bool)
	for _, provider := range healthyProviders {
		healthyMap[provider] = true
	}

	// Find best healthy candidate with highest confidence
	var best ProviderSelection
	for _, candidate := range candidates {
		if healthyMap[candidate.Provider] && candidate.Confidence > best.Confidence {
			best = candidate
		}
	}

	return best
}

// Health checker methods

// start begins health checking routine
func (hc *HealthChecker) start(ctx context.Context, providers map[string]LLMProvider) {
	ticker := time.NewTicker(hc.checkInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			hc.checkAllProviders(ctx, providers)
		}
	}
}

// checkAllProviders checks health of all providers
func (hc *HealthChecker) checkAllProviders(ctx context.Context, providers map[string]LLMProvider) {
	for name, provider := range providers {
		go hc.checkProvider(ctx, name, provider)
	}
}

// checkProvider checks health of a single provider
func (hc *HealthChecker) checkProvider(ctx context.Context, name string, provider LLMProvider) {
	start := time.Now()
	available := provider.IsAvailable(ctx)
	latency := time.Since(start)

	hc.mu.Lock()
	defer hc.mu.Unlock()

	health := &ProviderHealth{
		Provider:    name,
		Available:   available,
		LastChecked: time.Now(),
		Latency:     latency,
	}

	if !available {
		health.Error = "Provider unavailable"
	}

	hc.healthStatus[name] = health
}

// GetStatus returns current health status
func (hc *HealthChecker) GetStatus() map[string]*ProviderHealth {
	hc.mu.RLock()
	defer hc.mu.RUnlock()

	status := make(map[string]*ProviderHealth)
	for name, health := range hc.healthStatus {
		status[name] = health
	}

	return status
}

// GetHealthyProviders returns list of healthy provider names
func (hc *HealthChecker) GetHealthyProviders() []string {
	hc.mu.RLock()
	defer hc.mu.RUnlock()

	healthy := make([]string, 0)
	for name, health := range hc.healthStatus {
		if health.Available {
			healthy = append(healthy, name)
		}
	}

	return healthy
}

// Cost tracker methods

// CheckBudget verifies if a request fits within budget limits
func (ct *CostTracker) CheckBudget(provider string, estimatedCost float64) error {
	ct.mu.RLock()
	defer ct.mu.RUnlock()

	// Check daily limit
	if usage, exists := ct.dailyUsage[provider]; exists {
		if usage.Cost+estimatedCost > ct.budgets.DailyLimit {
			return fmt.Errorf("daily limit exceeded for provider %s", provider)
		}
	}

	// Check monthly budget
	if usage, exists := ct.monthlyUsage[provider]; exists {
		if usage.Cost+estimatedCost > ct.budgets.MonthlyBudget {
			return fmt.Errorf("monthly budget exceeded for provider %s", provider)
		}
	}

	return nil
}

// RecordUsage records token usage and cost
func (ct *CostTracker) RecordUsage(provider string, tokens TokenUsage, cost float64) {
	ct.mu.Lock()
	defer ct.mu.Unlock()

	now := time.Now()
	today := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, time.UTC)
	month := time.Date(now.Year(), now.Month(), 1, 0, 0, 0, 0, time.UTC)

	// Update daily usage
	dailyKey := fmt.Sprintf("%s-%s", provider, today.Format("2006-01-02"))
	if usage, exists := ct.dailyUsage[dailyKey]; exists {
		usage.Requests++
		usage.Tokens += tokens.TotalTokens
		usage.Cost += cost
		usage.LastUpdated = now
	} else {
		ct.dailyUsage[dailyKey] = &DailyUsage{
			Date:        today,
			Provider:    provider,
			Requests:    1,
			Tokens:      tokens.TotalTokens,
			Cost:        cost,
			LastUpdated: now,
		}
	}

	// Update monthly usage
	monthlyKey := fmt.Sprintf("%s-%s", provider, month.Format("2006-01"))
	if usage, exists := ct.monthlyUsage[monthlyKey]; exists {
		usage.Requests++
		usage.Tokens += tokens.TotalTokens
		usage.Cost += cost
		usage.LastUpdated = now
	} else {
		ct.monthlyUsage[monthlyKey] = &MonthlyUsage{
			Month:       month,
			Provider:    provider,
			Requests:    1,
			Tokens:      tokens.TotalTokens,
			Cost:        cost,
			LastUpdated: now,
		}
	}
}

// GetStats returns current usage statistics
func (ct *CostTracker) GetStats() map[string]interface{} {
	ct.mu.RLock()
	defer ct.mu.RUnlock()

	stats := map[string]interface{}{
		"daily_usage":   ct.dailyUsage,
		"monthly_usage": ct.monthlyUsage,
		"budgets":       ct.budgets,
	}

	return stats
}

// Cost optimizer methods

// EstimateCost estimates the cost for a request with a specific provider
func (co *CostOptimizer) EstimateCost(providerName string, request *ChatRequest) float64 {
	provider, exists := co.providers[providerName]
	if !exists {
		return 0.0
	}

	// Count tokens in request
	tokens, err := provider.CountTokens(request.Messages)
	if err != nil {
		co.logger.Warn().Err(err).Msg("Failed to count tokens for cost estimation")
		return 0.0
	}

	// Estimate response tokens (rough approximation)
	responseTokens := request.MaxTokens
	if responseTokens == 0 {
		responseTokens = tokens / 2 // Rough estimate
	}

	totalTokens := tokens + responseTokens

	// Calculate cost
	cost, err := provider.CalculateCost(totalTokens, request.Model)
	if err != nil {
		co.logger.Warn().Err(err).Msg("Failed to calculate cost")
		return 0.0
	}

	return cost
}