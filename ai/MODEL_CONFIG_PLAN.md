# Model Configuration System Plan

**Status**: Planning for model routing improvements (Week 9+)

**Philosophy**: Smart task-based defaults with simple overrides

## Core Principle

**Smart swapping = Use the right model for the task**
- NOT about usage limits or cost budgets
- About task complexity and agent type
- Should work great with zero configuration

## Configuration Levels

### Level 0: Zero Config (Not Implemented Yet)
```toml
# User doesn't configure anything
# We detect available provider from environment/credentials
# Default to Anthropic if available, else OpenAI, else error
```

### Level 1: Smart Default (Recommended)
```toml
[model]
provider = "anthropic"  # or "openrouter" or "openai" or "google" or "ollama"

# We automatically use task-based routing:
# - Explorer low complexity → haiku-4.5
# - Most tasks → sonnet-4.5
# - Sub-agents → haiku-4.5
```

**This should be the default experience** - great results, no tuning needed.

**OpenRouter as default?** Consider recommending OpenRouter as default provider:
- One API key for all models
- Automatic host selection (best latency/reliability)
- Fallback if primary host fails
- Access to Claude, OpenAI, Gemini, DeepSeek through one interface
- Optional exacto premium versions

### Level 2: Single Model Override
```toml
[model]
provider = "anthropic"
model = "claude-sonnet-4.5"  # Override: use this for EVERYTHING

# Disables smart routing
# User explicitly wants one model
```

### Level 3: Ollama Simple
```toml
[model]
provider = "ollama"
model = "qwen2.5-coder:latest"  # One model for everything
```

## Smart Routing Tables by Provider

### Anthropic (Level 1 Default)

| Task Type | Model | Reasoning |
|-----------|-------|-----------|
| Explorer Low | haiku-4.5 | Quick queries, fast |
| Explorer Medium | sonnet-4.5 | Most analysis |
| Explorer High | sonnet-4.5 | Deep analysis |
| Builder All | sonnet-4.5 | Code generation needs quality |
| Debugger All | sonnet-4.5 | Debugging needs reasoning |
| Refactorer All | sonnet-4.5 | Refactoring needs understanding |
| Sub-agents All | haiku-4.5 | Cheap parallelization |

**Result**: ~85% sonnet-4.5, ~15% haiku-4.5, ~0% opus-4.1

### OpenAI (Level 1 Default)

| Task Type | Model | Reasoning |
|-----------|-------|-----------|
| Explorer Low | gpt-4o-mini | Quick queries |
| Explorer Medium | gpt-5-codex | Most analysis |
| Explorer High | gpt-5-codex | Deep analysis |
| Builder All | gpt-5-codex | Code generation |
| Debugger All | gpt-5-codex | Debugging |
| Refactorer All | gpt-5-codex | Refactoring |
| Sub-agents All | gpt-4o-mini | Cheap parallelization |

**Result**: ~85% gpt-5-codex, ~15% gpt-4o-mini

### Google (Level 1 Default)

| Task Type | Model | Reasoning |
|-----------|-------|-----------|
| Explorer Low | gemini-2.5-flash-lite | Quick queries |
| Explorer Medium | gemini-2.5-pro | Most analysis |
| Explorer High | gemini-2.5-pro | Deep analysis |
| Builder All | gemini-2.5-pro | Code generation |
| Debugger All | gemini-2.5-pro | Debugging |
| Refactorer All | gemini-2.5-pro | Refactoring |
| Sub-agents All | gemini-2.5-flash-lite | Cheap parallelization |

**Result**: ~85% gemini-2.5-pro, ~15% gemini-2.5-flash-lite

### OpenRouter (Gateway to All Providers)

OpenRouter provides unified access to many models with:
- Multiple hosts per model (auto-failover)
- Latency and success rate tracking
- "Exacto" premium versions (better reliability)
- Single API key for all providers

```toml
[model]
provider = "openrouter"

# Smart routing with OpenRouter models
# We can use any model through OpenRouter
```

**Example routing**:
- Explorer Low: `anthropic/claude-haiku-4.5` (via OpenRouter)
- Most tasks: `anthropic/claude-sonnet-4.5` (via OpenRouter)
- Or use cheaper: `deepseek/deepseek-chat` (via OpenRouter)

**Benefits**:
- One API key for all providers
- Automatic host selection (best latency/reliability)
- Fallback if primary host down
- Optional "exacto" premium endpoints

**Config**:
```toml
[model]
provider = "openrouter"
use_exacto = true  # Use premium endpoints

# Can still do smart routing or single model
model = "anthropic/claude-sonnet-4.5"  # Single model through OpenRouter
```

### Ollama (No Smart Routing)

User specifies one model, we use it for everything:
```toml
[model]
provider = "ollama"
model = "qwen2.5-coder:latest"
```

No smart routing - all tasks use the same model.

**Why**: Ollama models vary wildly based on what user has installed. Let them decide.

## Implementation Plan

### Phase 1: Fix Current Code (Immediate)

**File**: `src/agent/model_router.rs`

**Changes**:
1. Fix model names to actual API strings (need to research exact names)
2. Reduce opus-4.1 usage (mostly sonnet-4.5)
3. Add provider-specific routing tables
4. Add single-model override support

**Code changes**:
```rust
pub struct ModelRouter {
    routing_table: HashMap<(AgentType, TaskComplexity), ModelConfig>,
    single_model_override: Option<ModelConfig>,  // NEW
    provider: String,  // NEW: "anthropic" | "openai" | "google" | "ollama"
}

impl ModelRouter {
    pub fn new_for_provider(provider: &str) -> Self {
        let routing_table = match provider {
            "anthropic" => Self::anthropic_routing_table(),
            "openai" => Self::openai_routing_table(),
            "google" => Self::google_routing_table(),
            "ollama" => HashMap::new(),  // No routing for ollama
            _ => Self::anthropic_routing_table(),  // Default
        };

        Self {
            routing_table,
            single_model_override: None,
            provider: provider.to_string(),
            stats: Arc::new(RwLock::new(ModelUsageStats::default())),
            baseline_model: ModelConfig::claude_sonnet_4_5(),  // Updated baseline
        }
    }

    pub fn with_single_model(provider: &str, model: &str) -> Self {
        let model_config = ModelConfig::from_string(provider, model);
        Self {
            routing_table: HashMap::new(),
            single_model_override: Some(model_config),
            provider: provider.to_string(),
            stats: Arc::new(RwLock::new(ModelUsageStats::default())),
            baseline_model: model_config.clone(),
        }
    }

    fn anthropic_routing_table() -> HashMap<(AgentType, TaskComplexity), ModelConfig> {
        let mut table = HashMap::new();

        // Explorer
        table.insert((AgentType::Explorer, TaskComplexity::Low),
                     ModelConfig::claude_haiku_4_5());
        table.insert((AgentType::Explorer, TaskComplexity::Medium),
                     ModelConfig::claude_sonnet_4_5());
        table.insert((AgentType::Explorer, TaskComplexity::High),
                     ModelConfig::claude_sonnet_4_5());

        // Builder - always sonnet
        table.insert((AgentType::Builder, TaskComplexity::Low),
                     ModelConfig::claude_sonnet_4_5());
        table.insert((AgentType::Builder, TaskComplexity::Medium),
                     ModelConfig::claude_sonnet_4_5());
        table.insert((AgentType::Builder, TaskComplexity::High),
                     ModelConfig::claude_sonnet_4_5());

        // Debugger - always sonnet
        table.insert((AgentType::Debugger, TaskComplexity::Low),
                     ModelConfig::claude_sonnet_4_5());
        table.insert((AgentType::Debugger, TaskComplexity::Medium),
                     ModelConfig::claude_sonnet_4_5());
        table.insert((AgentType::Debugger, TaskComplexity::High),
                     ModelConfig::claude_sonnet_4_5());

        // Refactorer - always sonnet
        table.insert((AgentType::Refactorer, TaskComplexity::Low),
                     ModelConfig::claude_sonnet_4_5());
        table.insert((AgentType::Refactorer, TaskComplexity::Medium),
                     ModelConfig::claude_sonnet_4_5());
        table.insert((AgentType::Refactorer, TaskComplexity::High),
                     ModelConfig::claude_sonnet_4_5());

        // Sub-agents - always haiku (cheap)
        table.insert((AgentType::FileSearcher, TaskComplexity::Low),
                     ModelConfig::claude_haiku_4_5());
        table.insert((AgentType::FileSearcher, TaskComplexity::Medium),
                     ModelConfig::claude_haiku_4_5());
        table.insert((AgentType::FileSearcher, TaskComplexity::High),
                     ModelConfig::claude_haiku_4_5());
        // ... etc for other sub-agents

        table
    }

    pub fn select_model(
        &self,
        agent_type: AgentType,
        complexity: TaskComplexity,
    ) -> ModelConfig {
        // Single model override takes precedence
        if let Some(config) = &self.single_model_override {
            return config.clone();
        }

        // Otherwise use routing table
        let key = (agent_type, complexity);
        self.routing_table
            .get(&key)
            .cloned()
            .unwrap_or_else(|| {
                // Fallback to best model for provider
                match self.provider.as_str() {
                    "anthropic" => ModelConfig::claude_sonnet_4_5(),
                    "openai" => ModelConfig::gpt_5_codex(),
                    "google" => ModelConfig::gemini_2_5_pro(),
                    _ => ModelConfig::claude_sonnet_4_5(),
                }
            })
    }
}
```

### Phase 2: Add Model Configs (Immediate)

**File**: `src/agent/model_router.rs`

Add these model configs (need exact API strings):

```rust
impl ModelConfig {
    // Anthropic (current models)
    pub fn claude_sonnet_4_5() -> Self {
        Self {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4.5".to_string(),  // TODO: Verify exact name
            cost_per_1m_input: 3.0,  // TODO: Verify pricing
            cost_per_1m_output: 15.0,
            max_context: 200_000,
            tokens_per_second: 100,
        }
    }

    pub fn claude_haiku_4_5() -> Self {
        Self {
            provider: "anthropic".to_string(),
            model: "claude-haiku-4.5".to_string(),  // TODO: Verify exact name
            cost_per_1m_input: 0.25,  // TODO: Verify pricing
            cost_per_1m_output: 1.25,
            max_context: 200_000,
            tokens_per_second: 150,
        }
    }

    pub fn claude_opus_4_1() -> Self {
        Self {
            provider: "anthropic".to_string(),
            model: "claude-opus-4.1".to_string(),  // TODO: Verify exact name
            cost_per_1m_input: 15.0,
            cost_per_1m_output: 75.0,
            max_context: 200_000,
            tokens_per_second: 50,
        }
    }

    // OpenAI (current models)
    pub fn gpt_5_codex() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "gpt-5-codex".to_string(),  // TODO: Verify exact name
            cost_per_1m_input: 2.5,  // TODO: Verify pricing
            cost_per_1m_output: 10.0,
            max_context: 128_000,
            tokens_per_second: 80,
        }
    }

    pub fn gpt_4o_mini() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "gpt-4o-mini".to_string(),
            cost_per_1m_input: 0.15,
            cost_per_1m_output: 0.60,
            max_context: 128_000,
            tokens_per_second: 120,
        }
    }

    // Google (current models)
    pub fn gemini_2_5_pro() -> Self {
        Self {
            provider: "google".to_string(),
            model: "gemini-2.5-pro".to_string(),  // TODO: Verify exact name
            cost_per_1m_input: 1.25,  // TODO: Verify pricing
            cost_per_1m_output: 5.0,
            max_context: 1_000_000,  // Gemini has huge context
            tokens_per_second: 60,
        }
    }

    pub fn gemini_2_5_flash() -> Self {
        Self {
            provider: "google".to_string(),
            model: "gemini-2.5-flash".to_string(),
            cost_per_1m_input: 0.075,
            cost_per_1m_output: 0.30,
            max_context: 1_000_000,
            tokens_per_second: 100,
        }
    }

    pub fn gemini_2_5_flash_lite() -> Self {
        Self {
            provider: "google".to_string(),
            model: "gemini-2.5-flash-lite".to_string(),
            cost_per_1m_input: 0.04,
            cost_per_1m_output: 0.16,
            max_context: 1_000_000,
            tokens_per_second: 150,
        }
    }

    // Generic constructor for Ollama/custom
    pub fn from_string(provider: &str, model: &str) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
            cost_per_1m_input: 0.0,  // Ollama is free
            cost_per_1m_output: 0.0,
            max_context: 128_000,  // Assume reasonable default
            tokens_per_second: 50,  // Varies by hardware
        }
    }
}
```

### Phase 3: Update Agent Initialization (Immediate)

**File**: `src/agent/core.rs`

Update where ModelRouter is created:

```rust
// Current (line ~162):
let model_router = Arc::new(ModelRouter::new());

// Updated:
let provider = config.model.provider.as_deref().unwrap_or("anthropic");
let model_router = if let Some(model) = &config.model.model {
    // User specified single model - use it for everything
    Arc::new(ModelRouter::with_single_model(provider, model))
} else {
    // Use smart task-based routing
    Arc::new(ModelRouter::new_for_provider(provider))
};
```

### Phase 4: Update Config Struct (Immediate)

**File**: `src/config/mod.rs` (or wherever config is defined)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Provider: "anthropic" | "openai" | "google" | "ollama"
    pub provider: Option<String>,

    /// Optional: Single model to use for everything
    /// If set, disables smart routing
    pub model: Option<String>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: Some("anthropic".to_string()),
            model: None,  // Use smart routing
        }
    }
}
```

**Example config.toml**:
```toml
# Level 1: Smart routing (default)
[model]
provider = "anthropic"

# Level 2: Single model
[model]
provider = "anthropic"
model = "claude-sonnet-4.5"

# Level 3: Ollama
[model]
provider = "ollama"
model = "qwen2.5-coder:latest"
```

## Edge Cases & Error Handling

### 1. Provider Not Available
```rust
// User sets provider = "anthropic" but no API key
// Fallback to ollama if available, else error

if !provider_available(&config.model.provider) {
    if ollama_available() {
        warn!("Provider {} not available, falling back to Ollama",
              config.model.provider);
        // Use ollama
    } else {
        error!("Provider {} not available and no fallback",
               config.model.provider);
        return Err(...);
    }
}
```

### 2. Model Not Available
```rust
// User specifies model that doesn't exist for provider
// Example: provider = "anthropic", model = "gpt-5-codex"

if !provider.supports_model(&config.model.model) {
    error!("Model {} not supported by provider {}",
           config.model.model, config.model.provider);
    // Fallback to default model for provider
    config.model.model = provider.default_model();
}
```

### 3. Ollama Model Not Installed
```rust
// User sets provider = "ollama", model = "qwen2.5-coder:latest"
// But model not pulled locally

if !ollama.has_model(&config.model.model) {
    warn!("Ollama model {} not found locally", config.model.model);

    // Option 1: Auto-pull
    if config.ollama.auto_pull {
        info!("Pulling model {}...", config.model.model);
        ollama.pull(&config.model.model)?;
    }

    // Option 2: Suggest alternatives
    let available = ollama.list_models()?;
    error!("Model not found. Available: {:?}", available);
    return Err(...);
}
```

### 4. Rate Limits Hit
```rust
// Future feature - for now just log
// When we implement usage-based swapping (later), handle here

if response.error == "rate_limit_exceeded" {
    warn!("Rate limit hit for {}", current_model);

    // Future: Could downgrade to cheaper model
    // For now: Just propagate error
    return Err(response.error);
}
```

### 5. Invalid Configuration
```rust
// User sets nonsensical config
// Example: provider = "anthropic", model = "definitely-not-a-real-model"

pub fn validate_config(config: &ModelConfig) -> Result<()> {
    // Check provider is supported
    let supported_providers = ["anthropic", "openai", "google", "ollama"];
    if let Some(provider) = &config.provider {
        if !supported_providers.contains(&provider.as_str()) {
            return Err(anyhow!(
                "Unsupported provider: {}. Supported: {:?}",
                provider, supported_providers
            ));
        }
    }

    // For known providers, validate model format
    if let Some(model) = &config.model {
        match config.provider.as_deref() {
            Some("anthropic") => {
                if !model.starts_with("claude-") {
                    warn!("Anthropic model should start with 'claude-': {}", model);
                }
            }
            Some("openai") => {
                if !model.starts_with("gpt-") {
                    warn!("OpenAI model should start with 'gpt-': {}", model);
                }
            }
            Some("google") => {
                if !model.starts_with("gemini-") {
                    warn!("Google model should start with 'gemini-': {}", model);
                }
            }
            Some("ollama") => {
                // Ollama models can be anything - user's choice
            }
            _ => {}
        }
    }

    Ok(())
}
```

## Testing Plan

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_routing_anthropic() {
        let router = ModelRouter::new_for_provider("anthropic");

        // Check explorer low uses haiku
        let model = router.select_model(AgentType::Explorer, TaskComplexity::Low);
        assert_eq!(model.model, "claude-haiku-4.5");

        // Check builder uses sonnet
        let model = router.select_model(AgentType::Builder, TaskComplexity::Medium);
        assert_eq!(model.model, "claude-sonnet-4.5");
    }

    #[test]
    fn test_single_model_override() {
        let router = ModelRouter::with_single_model("anthropic", "claude-opus-4.1");

        // All tasks should use opus
        let model = router.select_model(AgentType::Explorer, TaskComplexity::Low);
        assert_eq!(model.model, "claude-opus-4.1");

        let model = router.select_model(AgentType::Builder, TaskComplexity::High);
        assert_eq!(model.model, "claude-opus-4.1");
    }

    #[test]
    fn test_ollama_no_routing() {
        let router = ModelRouter::with_single_model("ollama", "qwen2.5-coder:latest");

        // All tasks use same model
        let model = router.select_model(AgentType::Explorer, TaskComplexity::Low);
        assert_eq!(model.model, "qwen2.5-coder:latest");
    }
}
```

### Integration Tests
- Test with actual Ollama (if available)
- Test with mock Anthropic API
- Test fallback logic
- Test invalid configs

## Documentation Updates Needed

### Files to Update
1. `ai/STATUS.md` - Note model routing improvements
2. `ai/TODO.md` - Add model config implementation tasks
3. `docs/STATUS.md` - Update feature status
4. `CLAUDE.md` - Document new model configuration
5. `README.md` - Update quick start with model config
6. `docs/configuration.md` - Detailed config guide (create if doesn't exist)

### Key Points to Document
1. Smart routing is default (zero config)
2. Single model override (simple config)
3. Provider selection (anthropic, openai, google, ollama)
4. Ollama setup and model selection
5. OAuth subscriptions (when implemented)

## Future Enhancements (NOT for Phase 1)

### Usage-Based Swapping (Later)
- Track usage against rate limits (5 hour, 1 week windows)
- Auto-downgrade when approaching limits
- Reset when window refreshes
- **Skip for now** - too complex

### Per-Agent Overrides (Later)
```toml
[model.overrides]
builder = "claude-opus-4.1"  # Use opus just for building
explorer = "claude-haiku-4.5"  # Use haiku for all exploration
```
**Skip for now** - users can just use single model override

### Cost Budgets (Later)
```toml
[model.budget]
daily = 10.00  # $10/day
monthly = 200.00  # $200/month
```
**Skip for now** - just track costs, don't enforce limits

### A/B Testing Models (Later)
```toml
[model.experiments]
test_gpt5_vs_sonnet = true
split_percent = 50  # 50% GPT-5, 50% Sonnet
```
**Skip for now** - research feature

## Summary

**Phase 1 (Immediate)**:
1. Fix model names (need to research exact API strings)
2. Implement provider-specific routing tables
3. Add single model override support
4. Update Agent initialization
5. Update config structs
6. Add model configs for anthropic/openai/google
7. Test with Ollama

**Phase 2 (Week 9)**:
1. Update documentation
2. Test with real providers
3. Handle edge cases
4. User feedback

**Phase 3 (Later)**:
1. Usage-based swapping
2. OAuth subscriptions (Claude Max, ChatGPT Plus?)
3. Cost budgets
4. Advanced features

**Key Principle**: Smart task-based routing by default, simple single-model override for users who want it.
