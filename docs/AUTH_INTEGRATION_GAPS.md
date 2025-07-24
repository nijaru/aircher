# Auth System Integration Gaps

## The Core Problem

We built a complete authentication system but **it's not connected to anything**. The providers still directly check environment variables, completely bypassing our auth system.

## Specific Code Gaps

### 1. ProviderManager Constructor
**File**: `src/providers/manager.rs`
```rust
// Current (line 24):
pub fn new(config: Arc<ConfigManager>) -> Result<Self>

// Needs to be:
pub fn new(config: Arc<ConfigManager>, auth_manager: Arc<AuthManager>) -> Result<Self>
```

### 2. Provider Initialization
**File**: `src/providers/anthropic.rs:47`
```rust
// Current:
let api_key = env::var("ANTHROPIC_API_KEY")
    .map_err(|_| anyhow!("ANTHROPIC_API_KEY not set"))?;

// Needs to be:
let api_key = auth_manager.get_api_key("anthropic")
    .await
    .or_else(|_| env::var("ANTHROPIC_API_KEY"))
    .map_err(|_| anyhow!("No API key found for Anthropic"))?;
```

### 3. AgentController Missing Auth
**File**: `src/agent/controller.rs`
- No AuthManager field
- No auth validation before LLM calls
- No handling of auth failures

### 4. Model Selection UI
**File**: `src/ui/model_selection.rs:156`
```rust
// Current:
let has_key = env::var(&provider_config.api_key_env).is_ok();

// Needs to be:
let has_key = self.auth_manager.get_provider_status(provider_name)
    .await
    .map(|s| s.is_authenticated())
    .unwrap_or(false);
```

## Data Flow Issues

### Current (Broken) Flow:
```
User → TUI → AgentController → ProviderManager → env::var() → ❌
         ↓
    AuthManager (sitting unused)
```

### Needed Flow:
```
User → TUI → AgentController → ProviderManager → AuthManager → API Key
                                                        ↓
                                                   env::var() (fallback)
```

## Missing Lifecycle Management

1. **AuthManager Creation**: Currently created fresh in multiple places
2. **Sharing**: No Arc<AuthManager> passed between components  
3. **Updates**: Auth status changes don't propagate to UI

## Provider Fallback Not Implemented

When a provider fails auth, nothing happens. We need:
```rust
async fn get_provider_with_fallback(&self, preferred: &str) -> Result<Box<dyn Provider>> {
    // Try preferred
    if let Ok(provider) = self.get_provider(preferred).await {
        return Ok(provider);
    }
    
    // Try fallbacks from config
    for fallback in &self.config.provider_fallbacks[preferred] {
        if let Ok(provider) = self.get_provider(fallback).await {
            return Ok(provider);
        }
    }
    
    Err(anyhow!("No authenticated providers available"))
}
```

## UI State Management

The TUI needs a shared AuthManager instance:
```rust
pub struct TuiState {
    // ... existing fields ...
    auth_manager: Arc<AuthManager>, // ADD THIS
}
```

Then pass it to:
- ModelSelectionOverlay
- AgentController  
- ProviderManager

## Testing Blockers

Can't test auth flow because:
1. Providers bypass auth system
2. No way to inject test keys
3. No mock providers for testing

## Priority Fix Order

1. **Add auth_manager parameter to ProviderManager** (unblocks everything)
2. **Update provider initialization** to use auth_manager
3. **Pass auth_manager through TUI components**
4. **Add auth validation in AgentController**
5. **Update model selection UI** to show real auth status
6. **Implement provider fallback logic**

Without fixing #1, the entire auth system remains decorative.