# Multi-Provider System: Next Steps

## 🚨 Critical Issue: Auth System Not Connected!

We built a complete auth system but it's not wired to actual LLM usage. Providers still only check environment variables directly.

## Immediate Technical Tasks

### 1. Wire Auth to Provider System
```rust
// Current (broken):
let api_key = env::var(&provider_config.api_key_env)?;

// Needed:
let api_key = auth_manager.get_api_key(&provider)
    .or_else(|_| env::var(&provider_config.api_key_env))?;
```

**Files to modify**:
- `src/providers/manager.rs`: Accept AuthManager in constructor
- `src/providers/anthropic.rs`: Use auth manager for keys
- `src/providers/openai.rs`: Same
- `src/agent/controller.rs`: Pass auth manager to providers

### 2. Fix Model Selection UI
- Connect to AuthManager for real-time status
- Show auth status indicators properly
- Handle "not authenticated" state gracefully

### 3. Implement Provider Fallback
- When auth fails, try next provider
- When rate limited, switch providers
- When usage limit hit, use fallback

### 4. Directory Structure Migration
- Revert from XDG to ~/.aircher for config
- Keep XDG for data files only
- Add migration command

## Task Priority Order

1. **Fix Provider Auth Integration** (blocks everything)
   - Modify ProviderManager to use AuthManager
   - Test with actual API calls

2. **Directory Structure Decision**
   - Implement ~/.aircher approach
   - Add migration from XDG

3. **Complete Model Selection UI**
   - Show auth status properly
   - Handle provider switching

4. **Test End-to-End Flow**
   - Auth → Provider → LLM call → Response
   - Multi-provider fallback scenarios

5. **Documentation Update**
   - Update based on new directory structure
   - Add auth setup guide

## Code Locations

### Auth Integration Points
- `src/providers/manager.rs:24` - ProviderManager::new() needs AuthManager
- `src/providers/anthropic.rs:47` - Direct env::var usage
- `src/agent/controller.rs:89` - No auth checking
- `src/ui/model_selection.rs:156` - Only checks env vars

### Missing Connections
```rust
// In ProviderManager
pub fn new(config: Arc<ConfigManager>, auth: Arc<AuthManager>) -> Result<Self> {
    // Use auth.get_api_key() instead of env::var()
}

// In AgentController  
async fn ensure_authenticated(&self, provider: &str) -> Result<()> {
    self.auth_manager.validate_provider(provider).await?;
}
```

## Testing Checklist

- [ ] Auth with stored key (no env var)
- [ ] Auth fallback (stored → env var)
- [ ] Provider switch on auth failure
- [ ] Usage limit fallback trigger
- [ ] Cost-based provider selection
- [ ] Auth status in UI updates correctly

## Architecture Decision

The auth system is well-designed but needs to be the **single source of truth** for API keys. Current issue: parallel systems (env vars vs auth storage) that don't talk to each other.

Solution: AuthManager becomes the gatekeeper for all API key access, with env vars as fallback only.