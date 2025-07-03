# Model Selection UI Specification

## Overview

Aircher's model selection UI implements a three-tier hierarchy that enables cost optimization and flexibility across multiple AI providers and hosting options. This design differentiates Aircher from single-provider tools by allowing users to select the optimal combination of model quality, cost, and performance for their specific needs.

## Selection Hierarchy: Provider → Model → Host

### Conceptual Framework

```
Provider: Who created the model (OpenAI, Anthropic, Google, Meta)
Model: Specific model version (gpt-4o, claude-3.5-sonnet, llama-3.1-70b)
Host: Where the model is accessed (Direct API, OpenRouter, Local, Enterprise)
```

### User Mental Model
- "I want Claude 3.5 Sonnet from Anthropic"
- "But I'll access it through OpenRouter hosting for 25% cost savings"
- "Or use Direct API for maximum reliability"

## UI Design

### Enhanced Modal Overlay Pattern

```
┌──────────────────── Select AI Configuration ─────────────────────┐
│ Current: OpenAI gpt-4o via OpenRouter                     [esc]  │
│                                                                   │
│ Provider: [OpenAI] [Anthropic] [Google] [Meta] [Ollama]          │
│          ──────                                                   │
│                                                                   │
│ ┌─ Available Models ─────────────────────────────────────────┐   │
│ │ ● gpt-4o                        $3.75/$11.25 per 1M tokens │   │
│ │ ○ gpt-4o-mini                   $0.11/$0.45 per 1M tokens  │   │
│ │ ○ o1-preview                    $11.25/$45 per 1M tokens   │   │
│ │ ○ o1-mini                       $2.25/$9 per 1M tokens     │   │
│ │                                                            │   │
│ │ Context: 128K tokens    Features: Vision, Tool Use        │   │
│ │ Speed: Fast            Reasoning: Excellent                │   │
│ └────────────────────────────────────────────────────────────┘   │
│                                                                   │
│ Host: ● OpenRouter (25% cheaper)   ○ Direct OpenAI (standard)    │
│       ○ Azure OpenAI (enterprise)  ○ Local (free)               │
│                                                                   │
│ Session Stats: 15.2K tokens used ($0.17 saved)   [Tab] Navigate  │
│ type to search • ↑↓ select • Enter confirm • Esc cancel          │
└───────────────────────────────────────────────────────────────────┘
```

### Key Features

#### 1. Three-Tier Selection Flow
- **Provider tabs**: Horizontal tab-style selection at top
- **Model list**: Vertical list with rich metadata for selected provider
- **Host options**: Horizontal selection showing cost/feature differences

#### 2. Rich Model Metadata
- **Pricing**: Per-token costs (input/output) for current host
- **Context window**: Maximum token capacity
- **Capabilities**: Vision, tool use, reasoning indicators
- **Performance**: Speed and quality ratings
- **Cost comparison**: Savings vs. direct API pricing

#### 3. Host Comparison
- **Cost optimization**: Visual indicators for cheaper options
- **Feature differences**: Enterprise, local, aggregator benefits
- **Performance hints**: Speed/reliability trade-offs
- **Availability status**: Real-time health checking

#### 4. Smart Recommendations
- **Task-aware suggestions**: Highlight optimal models for current context
- **Cost warnings**: Visual indicators for expensive models
- **Performance guidance**: Speed vs. quality recommendations
- **Context awareness**: Models that fit current conversation length

## Navigation and Interaction

### Keyboard Navigation
- **Tab**: Cycle between provider/model/host sections
- **Left/Right**: Navigate provider tabs (when focused)
- **Up/Down**: Navigate model list or host options
- **Type**: Real-time filtering of models by name
- **Enter**: Confirm selection and close modal
- **Esc**: Cancel without changes
- **1-9**: Quick access to favorite models
- **Ctrl+H**: Toggle host selection focus

### Search and Filtering
- **Typeahead**: Filter models as user types
- **Provider filtering**: Show only models from specific providers
- **Capability filtering**: Filter by features (vision, tools, etc.)
- **Cost filtering**: Show only models within budget range

## Data Structures

### Model Configuration
```rust
pub struct ModelConfiguration {
    provider: String,           // "openai"
    model: String,             // "gpt-4o"
    host: String,              // "openrouter"
    effective_pricing: Pricing, // Final cost per token
    capabilities: Vec<Capability>,
    context_window: u32,
    performance_rating: PerformanceRating,
}

pub struct ModelHost {
    name: String,              // "OpenRouter"
    description: String,       // "25% cheaper, higher rate limits"
    base_url: String,          // API endpoint
    pricing_multiplier: f64,   // 0.75 for 25% discount
    features: Vec<String>,     // ["cheaper", "higher_limits"]
    infrastructure: HostType,  // Aggregator, Direct, Local, Enterprise
    health_status: HealthStatus,
}

pub enum HostType {
    Direct,      // Official provider API
    Aggregator,  // OpenRouter, TogetherAI
    Local,       // Ollama, LMStudio
    Enterprise,  // Azure OpenAI, AWS Bedrock
}

pub struct Pricing {
    input_per_1m: f64,         // Input tokens per million
    output_per_1m: f64,        // Output tokens per million
    currency: String,          // "USD"
    host_discount: f64,        // Percentage savings vs direct
}
```

### Configuration Integration
```toml
[hosts.openrouter]
name = "OpenRouter"
description = "25% cheaper, higher rate limits"
base_url = "https://openrouter.ai/api/v1"
api_key_env = "OPENROUTER_API_KEY"
pricing_multiplier = 0.75
features = ["cheaper", "higher_limits", "unified_billing"]
infrastructure = "aggregator"

[hosts.direct_openai]
name = "Direct OpenAI"
description = "Official API, standard pricing"
base_url = "https://api.openai.com/v1"
api_key_env = "OPENAI_API_KEY"
pricing_multiplier = 1.0
features = ["official", "reliable"]
infrastructure = "direct"

[hosts.azure_openai]
name = "Azure OpenAI"
description = "Enterprise compliance, dedicated capacity"
base_url = "https://{deployment}.openai.azure.com"
api_key_env = "AZURE_OPENAI_KEY"
pricing_multiplier = 1.2
features = ["enterprise", "compliance", "dedicated"]
infrastructure = "enterprise"

[hosts.ollama_local]
name = "Ollama Local"
description = "Free, private, runs locally"
base_url = "http://localhost:11434"
pricing_multiplier = 0.0
features = ["free", "private", "offline"]
infrastructure = "local"
```

## Cost Optimization Features

### Real-Time Cost Calculation
- **Session tracking**: Current conversation cost with selected model/host
- **Projected costs**: Estimated cost for conversation completion
- **Savings display**: Amount saved vs. most expensive option
- **Budget warnings**: Visual alerts when approaching spending limits

### Smart Cost Controls
- **Auto-downgrade**: Suggest cheaper alternatives when approaching limits
- **Task-based optimization**: Recommend cost-effective models for specific tasks
- **Bulk operation pricing**: Show costs for large operations (file processing, etc.)

### Host Comparison Matrix
```
                  Direct    OpenRouter   Azure      Local
gpt-4o           $15/1M     $11.25/1M   $18/1M     N/A
claude-3.5       $15/1M     $11.25/1M   N/A        N/A
llama-3.1-70b    N/A        $0.88/1M    N/A        Free
```

## Implementation Considerations

### State Management
- **Selection persistence**: Remember user preferences across sessions
- **Context awareness**: Maintain selection context during model switching
- **Validation**: Verify model availability before switching
- **Error handling**: Graceful fallbacks for unavailable models/hosts

### Performance Optimization
- **Lazy loading**: Load model lists on demand
- **Caching**: Cache model metadata and pricing information
- **Background updates**: Refresh pricing and availability asynchronously
- **Quick access**: Keyboard shortcuts for frequently used combinations

### Integration Points
- **Provider manager**: Interface with existing LLM provider system
- **Cost tracker**: Integration with cost tracking and budget management
- **Session management**: Maintain conversation context during switches
- **Configuration system**: Integration with TOML configuration files

## Competitive Advantages

### vs. Claude Code
- **Multi-provider flexibility**: Not locked to single provider
- **Cost optimization**: Choose between direct API and cheaper alternatives
- **Host selection**: Enterprise, local, and aggregator options

### vs. Basic TUI Tools
- **Intelligent recommendations**: Context-aware model suggestions
- **Cost transparency**: Real-time pricing and savings calculations
- **Professional workflow**: Task-optimized model selection

### vs. Web Interfaces
- **Keyboard efficiency**: No mouse required for complex selections
- **Terminal integration**: Seamless with developer workflow
- **Offline capability**: Local model support without internet dependency

## Future Enhancements

### Advanced Features
- **Model benchmarking**: Performance comparisons for specific tasks
- **Custom host integration**: Support for private model hosting
- **Team configurations**: Shared model preferences and budgets
- **A/B testing**: Compare model outputs side-by-side

### UI Improvements
- **Theme customization**: User-configurable color schemes
- **Layout options**: Compact vs. detailed view modes
- **Quick switch**: Hotkey for recently used configurations
- **Favorite management**: Save and organize preferred model/host combinations

This specification provides the foundation for implementing Aircher's distinctive multi-provider, cost-optimized model selection interface that sets it apart from single-provider alternatives while maintaining the efficiency and elegance expected in professional developer tools.