# Authentication System

## Overview

Aircher supports multiple authentication methods across different AI providers, with secure credential storage and OAuth2 support for premium services.

## Supported Providers

### OpenAI
**Method**: API Key
- Set `OPENAI_API_KEY` environment variable
- Or configure in settings: `providers.openai.api_key`
- Models: GPT-4, GPT-4o, GPT-4o-mini

### Anthropic  
**Methods**: API Key or OAuth2

**API Key** (Claude API):
- Set `ANTHROPIC_API_KEY` environment variable
- Or configure in settings: `providers.anthropic.api_key`
- Models: Claude Opus 4.1, Sonnet 4, Haiku

**OAuth2** (Claude Pro/Max):
- Premium features with higher rate limits
- Web browser authentication flow
- Automatic token refresh

### Google Gemini
**Method**: API Key
- Set `GOOGLE_API_KEY` environment variable  
- Or configure in settings: `providers.gemini.api_key`
- Models: Gemini 1.5 Pro, Flash

### Ollama
**Method**: Local connection
- No authentication required
- Connects to local Ollama instance
- Models: Any locally installed models

## Authentication Flow

### Initial Setup
1. Run `/auth` command in TUI
2. Select provider
3. Choose authentication method
4. Complete setup (API key or OAuth)
5. Test connection

### Auth Wizard Steps
```rust
enum WizardStep {
    ProviderSelection,  // Choose provider
    ApiKeyEntry,       // Enter API key
    OAuth,             // OAuth flow (Anthropic only)
    Testing,           // Validate credentials  
    Complete,          // Success
}
```

## Configuration Storage

### File Location
```bash
~/.config/aircher/config.toml  # Linux/macOS
%APPDATA%/aircher/config.toml  # Windows
```

### Config Format
```toml
[providers.openai]
api_key = "sk-..."

[providers.anthropic]
api_key = "sk-ant-..."
# OR for OAuth:
oauth_token = "..."
oauth_refresh_token = "..."

[providers.gemini]
api_key = "AI..."
```

## Security Features

### Credential Protection
- API keys encrypted at rest
- OAuth tokens automatically refreshed
- Secure keychain integration (planned)

### Validation
- Connection testing during setup
- Rate limit detection and handling
- Error reporting with actionable messages

## Error Handling

### Common Issues
1. **Invalid API Key**: Clear error with provider-specific instructions
2. **Rate Limited**: Automatic backoff and retry
3. **Network Issues**: Graceful degradation
4. **Token Expired**: Automatic refresh (OAuth)

### Error Messages
```
❌ OpenAI API key invalid
   Check your key at: https://platform.openai.com/api-keys
   
⚠️  Rate limit exceeded (Anthropic)
   Retrying in 30 seconds...
   
✅ Authentication successful (Claude Pro)
   Connected via OAuth2
```

## Provider-Specific Notes

### Anthropic OAuth Setup
1. Redirects to claude.ai for login
2. Handles consent and permissions
3. Stores access and refresh tokens
4. Automatic token renewal

### Ollama Setup
1. Detects local Ollama instance
2. Fetches available models dynamically
3. No credentials required
4. Shows connection status

## Troubleshooting

### Debug Mode
Enable debug logging:
```bash
RUST_LOG=aircher::auth=debug cargo run
```

### Common Fixes
1. **Connection Failed**: Check network/firewall
2. **Invalid Response**: Verify API key format
3. **OAuth Issues**: Clear tokens and re-authenticate
4. **Model Access**: Check API tier/permissions

## Implementation Details

### Auth Manager (`src/auth/mod.rs`)
```rust
pub struct AuthManager {
    config: Config,
    client: HttpClient,
}

impl AuthManager {
    pub async fn authenticate_provider(&mut self, provider: &str) -> Result<()>;
    pub async fn refresh_token(&mut self, provider: &str) -> Result<()>;
    pub fn is_authenticated(&self, provider: &str) -> bool;
}
```

### OAuth Flow (`src/providers/anthropic.rs`)
1. Generate OAuth URL with PKCE
2. Launch browser for user consent
3. Handle callback and token exchange
4. Store tokens securely
5. Set up automatic refresh

---

*For setup instructions see `docs/QUICK_START_DEV.md`*
*For provider configuration see `TECH_SPEC.md#provider-system`*