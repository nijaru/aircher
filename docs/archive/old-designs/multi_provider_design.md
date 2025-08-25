# Multi-Provider Model Selection System Design

## Current System

### Current Architecture
- **Single Provider Selection**: `/model` command allows selecting one provider and one model
- **Global Defaults**: `default_provider` and `default_model` in config
- **Provider-Model Mapping**: Each provider has a list of models it supports
- **Simple Switching**: User manually switches between provider/model combinations

### Limitations
1. **One Size Fits All**: Same provider used for all tasks
2. **Manual Switching**: User must manually change providers for different use cases
3. **No Fallbacks**: If provider fails, no automatic alternative
4. **No Task Optimization**: Can't automatically select best model for specific tasks

## Enhanced Multi-Provider System

### Core Concepts

#### 1. Model-Provider Mapping
Multiple providers can offer the same model with different characteristics:

```toml
[model_preferences]
# Same model available from multiple providers
"claude-3-5-sonnet-20241022" = [
    { provider = "anthropic", priority = 1, cost_multiplier = 1.0 },
    { provider = "openrouter", priority = 2, cost_multiplier = 1.2 },
    { provider = "claude_pro", priority = 3, cost_multiplier = 0.0 }, # Free tier
]

"gpt-4o" = [
    { provider = "openai", priority = 1, cost_multiplier = 1.0 },
    { provider = "openrouter", priority = 2, cost_multiplier = 1.1 },
]

# Provider-specific models
"gemini-flash-2.0" = [
    { provider = "google", priority = 1, cost_multiplier = 1.0 },
]

"llama-3.3-70b" = [
    { provider = "ollama", priority = 1, cost_multiplier = 0.0 },
    { provider = "openrouter", priority = 2, cost_multiplier = 1.0 },
]
```

#### 2. Task-Based Model Selection
Different models for different types of tasks:

```toml
[task_preferences]
# AI Agent Tasks
agent_planning = { model = "claude-3-5-sonnet-20241022", temperature = 0.3 }
agent_coding = { model = "claude-3-5-sonnet-20241022", temperature = 0.1 }
agent_analysis = { model = "gpt-4o", temperature = 0.2 }

# Chat Tasks  
general_chat = { model = "claude-3-5-sonnet-20241022", temperature = 0.7 }
creative_writing = { model = "claude-3-5-sonnet-20241022", temperature = 0.9 }
quick_questions = { model = "gemini-flash-2.0", temperature = 0.5 }

# Specialized Tasks
code_review = { model = "claude-3-5-sonnet-20241022", temperature = 0.1 }
documentation = { model = "gpt-4o", temperature = 0.3 }
summarization = { model = "gemini-flash-2.0", temperature = 0.2 }
```

#### 3. Fallback Chains
Automatic provider fallback with configurable strategies:

```toml
[fallback_strategy]
# Global fallback behavior
strategy = "cost_aware" # Options: "fastest", "cheapest", "cost_aware", "quality"
max_cost_increase = 2.0 # Don't fallback to providers >2x cost
enable_free_fallback = true # Allow fallback to free tiers

# Provider-specific rules
[provider_fallbacks]
claude_pro = { fallback_to = ["anthropic", "openrouter"], max_usage_fallback = true }
anthropic = { fallback_to = ["openrouter"], rate_limit_fallback = true }
openai = { fallback_to = ["openrouter"], rate_limit_fallback = true }
ollama = { fallback_to = ["openrouter"], connection_fallback = true }
```

### Implementation Plan

#### Phase 1: Enhanced Config Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreference {
    pub provider: String,
    pub priority: u8,
    pub cost_multiplier: f64,
    pub conditions: Option<ProviderConditions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConditions {
    pub max_usage_percent: Option<f64>, // For subscription tiers
    pub required_features: Vec<String>, // e.g., ["streaming", "tools"]
    pub time_restrictions: Option<TimeRestrictions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPreference {
    pub model: String,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub system_prompt_override: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiProviderConfig {
    pub model_preferences: HashMap<String, Vec<ModelPreference>>,
    pub task_preferences: HashMap<String, TaskPreference>,
    pub fallback_strategy: FallbackStrategy,
    pub provider_fallbacks: HashMap<String, ProviderFallback>,
}
```

#### Phase 2: Intelligent Provider Selection

```rust
pub struct ModelSelector {
    config: MultiProviderConfig,
    provider_manager: Arc<ProviderManager>,
    cost_tracker: Arc<CostTracker>,
}

impl ModelSelector {
    pub async fn select_provider_for_model(
        &self,
        model: &str,
        task_type: Option<TaskType>,
        context: &SelectionContext,
    ) -> Result<ProviderSelection> {
        // 1. Get model preferences with provider priorities
        let preferences = self.config.model_preferences.get(model)?;
        
        // 2. Check provider availability and conditions
        let available_providers = self.filter_available_providers(preferences).await?;
        
        // 3. Apply fallback strategy if needed
        let selected = self.apply_selection_strategy(available_providers, context).await?;
        
        Ok(ProviderSelection {
            provider: selected.provider,
            model_name: model,
            cost_estimate: selected.cost_estimate,
            reasoning: selected.reasoning,
        })
    }
    
    pub async fn select_for_task(
        &self,
        task: TaskType,
        user_message: &str,
    ) -> Result<TaskSelection> {
        // 1. Get task preferences
        let task_pref = self.config.task_preferences.get(&task.to_string())?;
        
        // 2. Select provider for the preferred model
        let provider_selection = self.select_provider_for_model(
            &task_pref.model,
            Some(task),
            &SelectionContext::from_message(user_message),
        ).await?;
        
        // 3. Combine task preferences with provider selection
        Ok(TaskSelection {
            provider: provider_selection.provider,
            model: provider_selection.model_name,
            temperature: task_pref.temperature,
            max_tokens: task_pref.max_tokens,
            system_prompt_override: task_pref.system_prompt_override.clone(),
            reasoning: provider_selection.reasoning,
        })
    }
}
```

#### Phase 3: Enhanced UI Experience

##### Model Selection UI Improvements
1. **Grouped by Task**: Show models organized by task type
2. **Provider Options**: For each model, show available providers with status
3. **Cost Comparison**: Show real-time cost estimates for each provider
4. **Fallback Visualization**: Show fallback chains and current status

```
┌─ Model Selection ─────────────────────────────────────────┐
│ Task: Agent Coding                                        │
│                                                           │
│ Models:                                                   │
│ ● claude-3-5-sonnet-20241022                             │
│   ├─ anthropic       $0.003/1k  [⚡ fast]  ✓ available   │
│   ├─ openrouter      $0.004/1k  [⚡ fast]  ✓ available   │
│   └─ claude_pro      free       [⚡ fast]  ⚠ 80% used    │
│                                                           │
│ ○ gpt-4o                                                  │
│   ├─ openai          $0.005/1k  [⚡ fast]  ✓ available   │
│   └─ openrouter      $0.005/1k  [⚡ fast]  ✓ available   │
│                                                           │
│ [Tab] Switch Task  [Enter] Select  [Esc] Cancel          │
└───────────────────────────────────────────────────────────┘
```

##### New Commands
- `/model [task]`: Select model for specific task type
- `/providers`: Show all provider status and health
- `/fallback`: Configure fallback preferences

#### Phase 4: Automatic Fallback Handling

```rust
pub struct FallbackHandler {
    selector: ModelSelector,
    retry_policy: RetryPolicy,
}

impl FallbackHandler {
    pub async fn execute_with_fallback<T>(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<T> {
        let mut attempts = Vec::new();
        
        loop {
            // Select best available provider
            let selection = self.selector.select_provider_for_model(
                &request.model,
                context.task_type,
                &context.selection_context,
            ).await?;
            
            // Try the request
            match self.try_provider(&selection, &request).await {
                Ok(response) => return Ok(response),
                Err(e) if self.should_fallback(&e, &attempts) => {
                    attempts.push(FailedAttempt {
                        provider: selection.provider,
                        error: e,
                        timestamp: Utc::now(),
                    });
                    
                    // Update provider status and continue
                    self.mark_provider_degraded(&selection.provider).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }
}
```

### Benefits

#### For Users
1. **Automatic Optimization**: Best provider selected automatically
2. **Seamless Fallbacks**: No interruption when providers fail
3. **Cost Awareness**: Automatic cost optimization with user control
4. **Task Specialization**: Different models for different use cases

#### For Reliability
1. **High Availability**: Multiple providers for same models
2. **Graceful Degradation**: Fallback chains prevent service interruption
3. **Health Monitoring**: Track provider performance and availability
4. **Load Distribution**: Spread usage across providers

#### For Cost Management
1. **Provider Competition**: Use cheapest available option
2. **Usage Limits**: Respect subscription limits with fallbacks
3. **Cost Tracking**: Track costs across all providers
4. **Budget Controls**: Automatic cost management

### Migration Strategy

#### Phase 1: Backward Compatibility
- Keep existing `/model` command working
- Add new config sections as optional
- Default to current behavior if new config not present

#### Phase 2: Gradual Enhancement
- Add new `/model task` command for task-based selection
- Show provider options in existing model selection UI
- Add basic fallback for connection failures

#### Phase 3: Full Multi-Provider
- Enable all multi-provider features
- Add comprehensive UI for configuration
- Migrate existing configs to new format

This design provides sophisticated provider management while maintaining simplicity for basic use cases.