# Claude Max Subscription OAuth Setup

**Critical**: API keys bill separately from Max subscriptions. To use your Max sub, you need OAuth tokens.

## Understanding the Difference

### ❌ API Key Authentication (What We Currently Have)
```toml
[providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"
```
**Problems**:
- Billed per token ($3 input / $15 output per 1M)
- **NOT covered by Max subscription**
- Separate charges on API usage page

### ✅ OAuth Authentication (What You Want)
```json
{
  "anthropic": {
    "type": "oauth",
    "refresh": "YOUR_REFRESH_TOKEN",
    "access": "YOUR_ACCESS_TOKEN"
  }
}
```
**Benefits**:
- Uses your Max subscription (unlimited usage)
- No per-token charges
- Same auth as Claude Code/OpenCode

## How to Get OAuth Tokens

### Option 1: Use Existing Claude Code Installation (Easiest) ⭐

If you have Claude Code installed:

1. **Login to Claude Code**:
   ```bash
   # This opens browser for OAuth
   claude
   # Select your Max subscription account
   ```

2. **Find your tokens**:
   ```bash
   # Claude Code stores tokens here:
   cat ~/.local/share/claude-code/auth.json
   # or on macOS:
   cat ~/Library/Application\ Support/claude-code/auth.json
   ```

3. **Copy tokens to Aircher**:
   ```bash
   # Create Aircher auth file
   mkdir -p ~/.local/share/aircher
   cat > ~/.local/share/aircher/auth.json <<EOF
   {
     "anthropic": {
       "type": "oauth",
       "refresh": "YOUR_REFRESH_TOKEN_FROM_CLAUDE_CODE",
       "access": "YOUR_ACCESS_TOKEN_FROM_CLAUDE_CODE"
     }
   }
   EOF
   ```

### Option 2: Manual OAuth Flow (If No Claude Code)

Use the `claude-code-login` tool:

```bash
# Install dependencies
git clone https://github.com/grll/claude-code-login
cd claude-code-login
bun install  # or npm install

# Generate login URL
bun run index.ts
# Opens URL like: https://auth.prod.claude.ai/authorize?client_id=...

# 1. Click the URL
# 2. Login with your Max subscription account
# 3. Authorize the app
# 4. You'll be redirected to a page with an authorization_code

# Exchange code for tokens
bun run index.ts <authorization_code>
# This outputs: CLAUDE_ACCESS_TOKEN, CLAUDE_REFRESH_TOKEN, CLAUDE_EXPIRES_AT
```

### Option 3: Extract from OpenCode (If Installed)

```bash
cat ~/.local/share/opencode/auth.json
```

Copy the `refresh` and `access` tokens to Aircher's auth file.

## Aircher OAuth Implementation Status

### Current State (Oct 30, 2025)
- ❌ **OAuth NOT implemented yet**
- ✅ API key authentication works
- ✅ Provider architecture supports OAuth (just needs implementation)

### What Needs to be Built

**File**: `src/providers/claude_api.rs`

```rust
// Current (API key only):
pub async fn new(config: ProviderConfig, auth_manager: Arc<AuthManager>) -> Result<Self> {
    let api_key = auth_manager.get_api_key("claude").await?;

    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))?
    );
    // ...
}

// Need to add (OAuth support):
pub async fn new(config: ProviderConfig, auth_manager: Arc<AuthManager>) -> Result<Self> {
    // Try OAuth first
    if let Ok(oauth_tokens) = auth_manager.get_oauth_tokens("anthropic").await {
        // Use OAuth tokens
        let access_token = oauth_tokens.access_token;

        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", access_token))?
        );

        // Implement refresh logic if access token expired
        if oauth_tokens.is_expired() {
            let new_tokens = self.refresh_oauth_tokens(&oauth_tokens.refresh_token).await?;
            auth_manager.store_oauth_tokens("anthropic", &new_tokens).await?;
        }
    } else {
        // Fall back to API key
        let api_key = auth_manager.get_api_key("claude").await?;
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", api_key))?);
    }
    // ...
}

// New method needed:
async fn refresh_oauth_tokens(&self, refresh_token: &str) -> Result<OAuthTokens> {
    // Call Anthropic's OAuth refresh endpoint
    let response = self.client.post("https://auth.prod.claude.ai/oauth/token")
        .json(&json!({
            "grant_type": "refresh_token",
            "refresh_token": refresh_token,
            "client_id": "9d1c250a-e61b-44d9-88ed-5944d1962f5e"  // OpenCode's client ID
        }))
        .send()
        .await?;

    let tokens: OAuthTokens = response.json().await?;
    Ok(tokens)
}
```

**New structs needed**:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,  // Unix timestamp
}

impl OAuthTokens {
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        now >= self.expires_at
    }
}
```

**AuthManager changes** (`src/auth/mod.rs`):
```rust
impl AuthManager {
    // Add these methods:
    pub async fn get_oauth_tokens(&self, provider: &str) -> Result<OAuthTokens> {
        let auth_file = self.get_auth_file_path();
        let content = tokio::fs::read_to_string(&auth_file).await?;
        let auth_data: serde_json::Value = serde_json::from_str(&content)?;

        let tokens = auth_data.get(provider)
            .and_then(|p| serde_json::from_value(p.clone()).ok())
            .ok_or_else(|| anyhow!("No OAuth tokens found for {}", provider))?;

        Ok(tokens)
    }

    pub async fn store_oauth_tokens(&self, provider: &str, tokens: &OAuthTokens) -> Result<()> {
        let auth_file = self.get_auth_file_path();
        let mut auth_data: serde_json::Value = if auth_file.exists() {
            let content = tokio::fs::read_to_string(&auth_file).await?;
            serde_json::from_str(&content)?
        } else {
            json!({})
        };

        auth_data[provider] = json!({
            "type": "oauth",
            "refresh": tokens.refresh_token,
            "access": tokens.access_token,
            "expires_at": tokens.expires_at
        });

        let content = serde_json::to_string_pretty(&auth_data)?;
        tokio::fs::write(&auth_file, content).await?;

        Ok(())
    }

    fn get_auth_file_path(&self) -> PathBuf {
        // ~/.local/share/aircher/auth.json (Linux)
        // ~/Library/Application Support/aircher/auth.json (macOS)
        let data_dir = dirs::data_local_dir()
            .expect("Failed to get data directory");
        data_dir.join("aircher").join("auth.json")
    }
}
```

## Implementation Timeline

### Immediate Workaround (1 hour)
**Manually use API key for now** - Accept per-token billing for initial testing

### Phase 1: OAuth Support (4-6 hours)
1. Add OAuthTokens struct
2. Implement refresh logic
3. Update ClaudeApiProvider to use OAuth
4. Test with manually extracted tokens from Claude Code

### Phase 2: OAuth Login Flow (8-12 hours)
1. Implement full OAuth PKCE flow
2. Add CLI command: `aircher auth claude`
3. Browser-based login
4. Token storage

## SWE-bench with OAuth

Once OAuth is implemented:

```bash
# 1. Login to get OAuth tokens
aircher auth claude
# Opens browser, you login with Max subscription

# 2. Tokens stored in ~/.local/share/aircher/auth.json

# 3. Run SWE-bench
python run.py \
  --model aircher \
  --dataset swebench_lite \
  --output_dir ./results

# Your Max subscription is used (no per-token charges)
```

## Recommendation

### Short-term (This Week) ⭐
**Option A**: Use API key, accept per-token billing for testing
- Fastest to get started
- ~$90-120 for SWE-bench Lite (300 tasks)
- Validates agent works

**Option B**: Extract tokens from Claude Code manually
- If you have Claude Code installed
- Copy tokens to `~/.local/share/aircher/auth.json`
- Implement minimal OAuth refresh in Aircher (2-3 hours)

### Long-term (Next Week)
Implement full OAuth support so future runs use Max subscription

## Cost Comparison

### With API Key (per-token billing)
- SWE-bench Lite (300 tasks): ~$90-120
- SWE-bench Verified (500 tasks): ~$150-200
- SWE-bench Full (2,294 tasks): ~$700-900

### With Max Subscription (OAuth)
- All of the above: $0 (covered by $200/month subscription)

## Decision Point

**Question**: Do you want to:

1. ✅ **Start quickly** - Use API key, pay ~$90-120 for initial validation
2. ✅ **Implement OAuth first** - 4-6 hours work, then free usage
3. ✅ **Extract Claude Code tokens** - 1 hour manual + 2-3 hours minimal OAuth (hybrid approach)

Which approach would you prefer?
