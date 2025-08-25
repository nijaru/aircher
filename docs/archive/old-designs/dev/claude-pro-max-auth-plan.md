# Claude Pro/Max Authentication Implementation Plan

## Overview
Implement subscription-based Claude Pro/Max authentication using Claude Code's OAuth client, following OpenCode's approach.

## Technical Approach

### 1. OAuth 2.0 with PKCE Flow
```rust
// src/providers/claude_subscription.rs
pub struct ClaudeSubscriptionProvider {
    client_id: String,
    session_store: Arc<Mutex<SessionStore>>,
    http_client: reqwest::Client,
}

const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const AUTH_URL: &str = "https://claude.ai/oauth/authorize";
const TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";
const REDIRECT_URI: &str = "https://console.anthropic.com/oauth/code/callback";
const SCOPES: &str = "org:create_api_key user:profile user:inference";
```

### 2. Authentication Flow Implementation
```rust
impl ClaudeSubscriptionProvider {
    pub async fn start_auth_flow(&self) -> Result<AuthFlow> {
        let pkce = generate_pkce();
        let auth_url = format!(
            "{}?client_id={}&response_type=code&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256&state={}",
            AUTH_URL, CLIENT_ID, REDIRECT_URI, SCOPES, pkce.challenge, pkce.verifier
        );
        
        // Store PKCE verifier for later use
        self.session_store.lock().await.store_pkce_verifier(pkce.verifier.clone());
        
        Ok(AuthFlow {
            auth_url,
            pkce_verifier: pkce.verifier,
        })
    }
    
    pub async fn exchange_code(&self, code: &str, state: &str) -> Result<TokenResponse> {
        let pkce_verifier = self.session_store.lock().await.get_pkce_verifier()?;
        
        let token_request = TokenRequest {
            code: code.split('#').next().unwrap(),
            state: state.to_string(),
            grant_type: "authorization_code",
            client_id: CLIENT_ID,
            redirect_uri: REDIRECT_URI,
            code_verifier: pkce_verifier,
        };
        
        let response = self.http_client
            .post(TOKEN_URL)
            .json(&token_request)
            .send()
            .await?;
            
        let tokens: TokenResponse = response.json().await?;
        self.store_tokens(tokens.clone()).await?;
        
        Ok(tokens)
    }
}
```

### 3. Session Management
```rust
pub struct SessionStore {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_at: Option<DateTime<Utc>>,
    usage_info: Option<UsageInfo>,
}

impl SessionStore {
    pub async fn get_valid_token(&mut self) -> Result<String> {
        if let Some(token) = &self.access_token {
            if let Some(expires) = self.expires_at {
                if expires > Utc::now() {
                    return Ok(token.clone());
                }
            }
        }
        
        // Token expired, refresh it
        self.refresh_access_token().await
    }
    
    async fn refresh_access_token(&mut self) -> Result<String> {
        let refresh_token = self.refresh_token.as_ref()
            .ok_or_else(|| Error::NoRefreshToken)?;
            
        let refresh_request = RefreshRequest {
            grant_type: "refresh_token",
            refresh_token: refresh_token.clone(),
            client_id: CLIENT_ID,
        };
        
        let response = reqwest::Client::new()
            .post(TOKEN_URL)
            .json(&refresh_request)
            .send()
            .await?;
            
        let tokens: TokenResponse = response.json().await?;
        self.store_new_tokens(tokens.clone()).await?;
        
        Ok(tokens.access_token)
    }
}
```

### 4. Usage Tracking Implementation
```rust
#[derive(Debug, Clone)]
pub struct UsageInfo {
    pub current_usage: u64,
    pub limit: u64,
    pub reset_date: DateTime<Utc>,
    pub usage_percentage: f64,
    pub tier: SubscriptionTier,
    pub approaching_limit: bool,
}

impl ClaudeSubscriptionProvider {
    pub async fn get_usage_info(&self) -> Result<UsageInfo> {
        let token = self.session_store.lock().await.get_valid_token().await?;
        
        // Query usage from Claude API (endpoint TBD)
        let response = self.http_client
            .get("https://api.anthropic.com/v1/usage") // Placeholder URL
            .header("Authorization", format!("Bearer {}", token))
            .header("anthropic-beta", "oauth-2025-04-20")
            .send()
            .await?;
            
        let usage_data: UsageResponse = response.json().await?;
        
        Ok(UsageInfo {
            current_usage: usage_data.usage,
            limit: usage_data.limit,
            reset_date: usage_data.reset_date,
            usage_percentage: (usage_data.usage as f64 / usage_data.limit as f64) * 100.0,
            tier: usage_data.tier,
            approaching_limit: usage_data.usage as f64 / usage_data.limit as f64 > 0.8,
        })
    }
}
```

### 5. Provider Integration
```rust
#[async_trait]
impl LLMProvider for ClaudeSubscriptionProvider {
    fn name(&self) -> &str {
        "claude-subscription"
    }
    
    fn pricing_model(&self) -> PricingModel {
        PricingModel::Subscription {
            current_usage: self.get_current_usage(),
            limit: self.get_usage_limit(),
            reset_date: self.get_reset_date(),
            tier: self.get_subscription_tier(),
        }
    }
    
    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse> {
        // Check usage before making request
        let usage = self.get_usage_info().await?;
        if usage.approaching_limit {
            return Err(Error::ApproachingUsageLimit(usage));
        }
        
        let token = self.session_store.lock().await.get_valid_token().await?;
        
        let response = self.http_client
            .post("https://api.anthropic.com/v1/messages")
            .header("Authorization", format!("Bearer {}", token))
            .header("anthropic-beta", "oauth-2025-04-20")
            .json(req)
            .send()
            .await?;
            
        // Update usage tracking after successful request
        self.update_usage_tracking().await?;
        
        Ok(response.json().await?)
    }
    
    fn calculate_cost(&self, _tokens: u32) -> Option<f64> {
        // Subscription model - no per-token cost
        Some(0.0)
    }
    
    async fn get_usage_info(&self) -> Result<Option<UsageInfo>> {
        Ok(Some(self.get_usage_info().await?))
    }
}
```

## Implementation Steps

### Phase 1: Core OAuth Implementation
1. **Create `src/providers/claude_subscription.rs`**
2. **Implement PKCE OAuth flow** with proper state management
3. **Add session persistence** using SQLite storage
4. **Create token refresh mechanism** for long-lived sessions

### Phase 2: Usage Tracking
1. **Implement usage API calls** to get current subscription status
2. **Add usage limit warnings** at 80% threshold
3. **Create usage display** in TUI status bar (e.g., "44k/200k Claude Pro")
4. **Add tier detection** (Pro vs Max) with different limits

### Phase 3: TUI Integration
1. **Add authentication UI** for initial OAuth flow
2. **Integrate usage display** in status bar
3. **Add usage warnings** when approaching limits
4. **Create subscription management** interface

### Phase 4: Error Handling & Fallbacks
1. **Implement graceful degradation** when usage limits hit
2. **Add automatic provider fallback** to API-based providers
3. **Create proper error messages** for subscription issues
4. **Add reconnection handling** for expired sessions

## Configuration Updates

### Add to `config/providers.toml`:
```toml
[providers.claude_subscription]
name = "Claude Pro/Max"
description = "Subscription-based Claude access"
auth_type = "oauth"
pricing_model = "subscription"
features = ["subscription", "usage_limits", "free_usage"]

[providers.claude_subscription.limits]
pro_monthly_limit = 200000
max_monthly_limit = 500000
warning_threshold = 0.8
```

## Testing Strategy

### Unit Tests
- OAuth flow components
- Token refresh logic
- Usage calculation accuracy
- Error handling scenarios

### Integration Tests
- End-to-end authentication flow
- Usage tracking updates
- Provider fallback mechanisms
- Session persistence

### Manual Testing
- Browser OAuth flow
- Usage limit warnings
- Long-term session management
- Provider switching

## Security Considerations

1. **Token Storage**: Secure storage of refresh tokens
2. **PKCE Implementation**: Proper challenge/verifier generation
3. **State Validation**: Prevent CSRF attacks
4. **Token Rotation**: Automatic refresh token rotation
5. **Usage Validation**: Server-side usage verification

## Benefits Over API-Based Access

1. **Cost Savings**: No per-token charges for subscription users
2. **Higher Limits**: Subscription users get higher usage limits
3. **Better UX**: Familiar OAuth flow like Claude Code
4. **Usage Transparency**: Clear usage tracking like "44k/200k"
5. **Automatic Management**: Session-based auth with refresh

## Timeline
- **Week 1**: Core OAuth implementation and session management
- **Week 2**: Usage tracking and TUI integration
- **Week 3**: Error handling and fallback mechanisms
- **Week 4**: Testing and documentation

This approach mirrors OpenCode's successful implementation while providing a seamless experience for Aircher users with Claude Pro/Max subscriptions.