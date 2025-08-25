# Authentication System

Aircher provides a comprehensive authentication system for managing API keys across multiple AI providers.

## Overview

The auth system consists of:
- **CLI commands** for managing API keys from the command line
- **TUI auth wizard** for interactive setup within the terminal interface
- **Secure storage** with key obfuscation
- **Provider status tracking** with real-time validation

## Storage Location

API keys are stored in `~/.aircher/auth.json` for simplicity and consistency with other developer tools.

## CLI Usage

### Available Commands

```bash
# Add or update an API key
aircher auth login <provider> [api-key]
# If api-key is not provided, you'll be prompted securely

# Remove an API key
aircher auth logout <provider>

# Check authentication status
aircher auth status [provider]
# Omit provider to see all statuses

# Test provider authentication
aircher auth test <provider>

# List all configured providers
aircher auth list

# Clear all stored keys
aircher auth clear
```

### Examples

```bash
# Login to Claude
aircher auth login claude
Enter API key for claude: [hidden input]
✅ API key stored for claude

# Check all provider statuses
aircher auth status
✓ claude: authenticated (sk-a...t123)
✓ openai: authenticated (sk-o...t456)
○ gemini: not configured
✓ ollama: authenticated

# Test a specific provider
aircher auth test openai
Testing authentication for openai...
✅ Authentication successful for openai
```

## TUI Auth Wizard

Within the TUI, use the `/auth` command to launch the interactive auth wizard:

1. Type `/auth` in the chat interface
2. Select a provider from the list
3. Enter your API key (input is masked)
4. The wizard will validate and store your key

### Visual Status Indicators

- ✅ **Authenticated**: Provider is ready to use
- ❌ **Needs setup**: API key required
- ⚡ **Local provider**: No authentication needed (e.g., Ollama)
- ○ **Not configured**: Provider available but not set up

## Security

### Key Storage
- Keys are obfuscated using a simple XOR cipher (not cryptographically secure)
- Stored in user's config directory with appropriate file permissions
- Future enhancement: OS keychain integration planned

### Best Practices
- Never commit API keys to version control
- Use environment variables for CI/CD environments
- Rotate keys regularly
- Use provider-specific restricted keys when available

## Environment Variables

You can also provide API keys via environment variables:

```bash
export ANTHROPIC_API_KEY=sk-ant-...
export OPENAI_API_KEY=sk-...
export GEMINI_API_KEY=AIza...
```

Environment variables take precedence over stored keys.

## Provider-Specific Notes

### Anthropic (Claude)
- Get your API key from https://console.anthropic.com/
- Key format: `sk-ant-...`

### OpenAI
- Get your API key from https://platform.openai.com/api-keys
- Key format: `sk-...`

### Google (Gemini)
- Get your API key from https://makersuite.google.com/app/apikey
- Key format: `AIza...`

### Ollama
- Local provider - no API key required
- Ensure Ollama is running locally on port 11434

### OpenRouter
- Get your API key from https://openrouter.ai/keys
- Key format: `sk-or-...`

## Troubleshooting

### "API key not found"
- Ensure you've run `aircher auth login <provider>`
- Check if the environment variable is set correctly
- Verify the key hasn't been cleared with `auth clear`

### "Invalid API key"
- Double-check your key is copied correctly
- Ensure the key hasn't been revoked
- Try generating a new key from your provider

### "Network error"
- Check your internet connection
- Verify provider endpoints aren't blocked
- Check if you're behind a corporate proxy