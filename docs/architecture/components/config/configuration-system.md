# Configuration Architecture - Technical Specification

## Overview

Aircher implements a two-file configuration strategy that separates user preferences from sensitive credentials, emphasizing smart defaults to minimize configuration burden while maintaining security and flexibility.

## Core Principles

### 1. Minimal Configuration Philosophy
- **Smart Defaults**: Most settings auto-detected from models.dev API and provider capabilities
- **Zero Configuration**: Works out-of-the-box with `aircher login` only
- **Progressive Enhancement**: Advanced users can override defaults as needed
- **Security First**: Credentials separated from configuration with proper file permissions

### 2. Two-File Architecture
```
~/.config/aircher/
├── config.toml          # User preferences (644 permissions)
├── credentials.toml     # API keys (600 permissions, user read/write only)
└── cache/
    └── models.json      # Cached models.dev data (24-hour TTL)
```

## File Specifications

### Configuration File (`config.toml`)

**Location**: `~/.config/aircher/config.toml` or `project/.agents/config.toml`
**Permissions**: 644 (standard user file)
**Purpose**: Non-sensitive user preferences and overrides

```toml
[providers]
default = "auto"              # auto-detect best available provider
fallback_enabled = true       # automatically fallback if primary fails
fallback_order = ["openai", "anthropic", "google", "groq"]

[models]
# Smart defaults (override only if needed)
auto_select = true                          # intelligent model selection based on task
openai_default = "gpt-4"                    # latest GPT-4 variant
anthropic_default = "claude-3-5-sonnet"     # latest Sonnet
google_default = "gemini-2.5-pro"           # latest Gemini Pro

[interface]
show_thinking = true          # show AI thinking process
show_context_usage = true     # show token usage (e.g., "44k/200k")
show_cost = true             # show cost estimates
markdown_rendering = true     # rich markdown formatting
streaming = true             # real-time response streaming
syntax_highlighting = true    # code syntax highlighting

[context]
max_files = 20               # maximum files to include in context
relevance_threshold = 0.3    # minimum file relevance score (0.0-1.0)
auto_compaction = true       # automatically manage context limits
preserve_recent = 5          # always keep last N messages

[costs]
monthly_budget = 100.0       # monthly spending limit (USD)
warn_threshold = 0.8         # warn when reaching 80% of budget
track_usage = true           # track and display costs
```

### Credentials File (`credentials.toml`)

**Location**: `~/.config/aircher/credentials.toml`
**Permissions**: 600 (user read/write only)
**Purpose**: API keys and authentication tokens
**Management**: Exclusively via `aircher login` command

```toml
# Aircher API Credentials
# This file is automatically managed by `aircher login`
# File permissions: 600 (user read/write only)

[openai]
api_key = "sk-..."
organization = ""  # optional
project = ""       # optional

[anthropic]
api_key = "sk-ant-..."

[google]
api_key = "AI..."
project_id = ""    # optional for Vertex AI

[groq]
api_key = "gsk_..."

[openrouter]
api_key = "sk-or-..."

[ollama]
base_url = "http://localhost:11434"
# No API key needed for local Ollama
```

## Smart Defaults System

### Models.dev API Integration

```go
type ModelRegistry struct {
    cache      map[string]*ModelInfo
    lastUpdate time.Time
    apiURL     string
}

type ModelInfo struct {
    ID           string             `json:"id"`
    Name         string             `json:"name"`
    Provider     string             `json:"provider"`
    Limits       ModelLimits        `json:"limit"`
    Cost         ModelCost          `json:"cost"`
    Capabilities ModelCapabilities  `json:"capabilities"`
}

type ModelLimits struct {
    Context int `json:"context"`
    Output  int `json:"output"`
}

type ModelCost struct {
    Input     float64 `json:"input"`      // per 1M tokens
    Output    float64 `json:"output"`     // per 1M tokens
    CacheRead float64 `json:"cache_read"` // per 1M tokens
}
```

### Automatic Model Selection Logic

```go
func SelectBestModel(providers []string, task TaskContext) (*ModelInfo, error) {
    for _, provider := range providers {
        models := getAvailableModels(provider)
        
        switch provider {
        case "openai":
            return selectOpenAIModel(models, task)
        case "anthropic":
            return selectClaudeModel(models, task)
        case "google":
            return selectGeminiModel(models, task)
        }
    }
    
    return nil, errors.New("no suitable model found")
}

func selectOpenAIModel(models []ModelInfo, task TaskContext) *ModelInfo {
    // Prefer latest GPT-4 variant
    if model := findModel(models, "gpt-4.1"); model != nil {
        return model
    }
    if model := findModel(models, "gpt-4"); model != nil {
        return model
    }
    return findModel(models, "gpt-4o-mini") // fallback
}
```

### Provider Detection

```go
func DetectAvailableProviders() []string {
    var available []string
    
    // Check credentials file
    if hasCredential("openai") {
        available = append(available, "openai")
    }
    if hasCredential("anthropic") {
        available = append(available, "anthropic")
    }
    
    // Check environment variables (fallback)
    if os.Getenv("OPENAI_API_KEY") != "" {
        available = append(available, "openai")
    }
    
    // Check local services
    if isOllamaRunning() {
        available = append(available, "ollama")
    }
    
    return available
}
```

## Security Implementation

### File Permissions

```go
func SaveCredentials(creds *Credentials) error {
    credPath := getCredentialsPath()
    
    // Create directory with proper permissions
    if err := os.MkdirAll(filepath.Dir(credPath), 0755); err != nil {
        return fmt.Errorf("failed to create config directory: %w", err)
    }
    
    // Marshal credentials
    data, err := toml.Marshal(creds)
    if err != nil {
        return fmt.Errorf("failed to marshal credentials: %w", err)
    }
    
    // Write with restricted permissions
    if err := os.WriteFile(credPath, data, 0600); err != nil {
        return fmt.Errorf("failed to write credentials: %w", err)
    }
    
    return nil
}
```

### API Key Security Practices

```go
type SecureString struct {
    data []byte
}

func NewSecureString(s string) *SecureString {
    data := make([]byte, len(s))
    copy(data, s)
    return &SecureString{data: data}
}

func (s *SecureString) String() string {
    return string(s.data)
}

func (s *SecureString) Clear() {
    for i := range s.data {
        s.data[i] = 0
    }
}

// Never log API keys
func (s *SecureString) GoString() string {
    return "[REDACTED]"
}
```

## CLI Command Implementation

### `aircher login` Interactive Flow

```go
func RunLoginCommand() error {
    providers := []string{"openai", "anthropic", "google", "groq", "openrouter", "ollama"}
    
    selected, err := promptProviderSelection(providers)
    if err != nil {
        return err
    }
    
    switch selected {
    case "ollama":
        return configureOllama()
    default:
        return configureAPIProvider(selected)
    }
}

func configureAPIProvider(provider string) error {
    fmt.Printf("Configuring %s...\n", provider)
    fmt.Printf("Get your API key from: %s\n", getAPIKeyURL(provider))
    
    apiKey, err := promptSecureInput("Paste your API key: ")
    if err != nil {
        return err
    }
    
    // Validate key
    if err := validateAPIKey(provider, apiKey); err != nil {
        return fmt.Errorf("invalid API key: %w", err)
    }
    
    // Save credentials
    creds := loadCredentials()
    creds.SetAPIKey(provider, apiKey)
    
    if err := saveCredentials(creds); err != nil {
        return fmt.Errorf("failed to save credentials: %w", err)
    }
    
    fmt.Printf("✓ %s configured successfully\n", provider)
    
    // Optionally set as default
    if promptYesNo("Set as default provider?") {
        config := loadConfig()
        config.Providers.Default = provider
        saveConfig(config)
    }
    
    return nil
}
```

### Credential Validation

```go
func validateAPIKey(provider string, apiKey string) error {
    client := createProviderClient(provider, apiKey)
    
    ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
    defer cancel()
    
    switch provider {
    case "openai":
        return validateOpenAIKey(ctx, client)
    case "anthropic":
        return validateAnthropicKey(ctx, client)
    case "google":
        return validateGoogleKey(ctx, client)
    default:
        return fmt.Errorf("validation not implemented for %s", provider)
    }
}

func validateOpenAIKey(ctx context.Context, client *openai.Client) error {
    _, err := client.ListModels(ctx)
    if err != nil {
        return fmt.Errorf("OpenAI API validation failed: %w", err)
    }
    return nil
}
```

## Configuration Loading Strategy

### Hierarchical Loading

```go
type ConfigLoader struct {
    userConfigPath    string
    projectConfigPath string
    credentialsPath   string
}

func (c *ConfigLoader) LoadConfig() (*Config, error) {
    config := DefaultConfig()
    
    // 1. Load user global config
    if userConfig, err := c.loadUserConfig(); err == nil {
        config = mergeConfigs(config, userConfig)
    }
    
    // 2. Load project-specific config (overrides user config)
    if projectConfig, err := c.loadProjectConfig(); err == nil {
        config = mergeConfigs(config, projectConfig)
    }
    
    // 3. Apply environment variable overrides
    config = applyEnvOverrides(config)
    
    return config, nil
}

func (c *ConfigLoader) LoadCredentials() (*Credentials, error) {
    creds := &Credentials{}
    
    // 1. Load from credentials file
    if fileExists(c.credentialsPath) {
        fileCreds, err := loadCredentialsFromFile(c.credentialsPath)
        if err != nil {
            return nil, fmt.Errorf("failed to load credentials file: %w", err)
        }
        creds = fileCreds
    }
    
    // 2. Fallback to environment variables
    creds = enrichFromEnvironment(creds)
    
    return creds, nil
}
```

## Error Handling

### User-Friendly Error Messages

```go
type ConfigError struct {
    Type    ErrorType
    Message string
    Fix     string
}

func (e ConfigError) Error() string {
    return fmt.Sprintf("%s\n\nTo fix this:\n%s", e.Message, e.Fix)
}

var (
    ErrNoAPIKey = ConfigError{
        Type:    ErrorTypeAuth,
        Message: "No API key configured for OpenAI",
        Fix: `1. Run: aircher login openai
2. Get an API key from https://platform.openai.com/api-keys  
3. Paste it when prompted`,
    }
    
    ErrInvalidAPIKey = ConfigError{
        Type:    ErrorTypeAuth,
        Message: "OpenAI API key is invalid",
        Fix: `1. Get a new API key from https://platform.openai.com/api-keys
2. Run: aircher login openai
3. Paste your new key when prompted`,
    }
)
```

## Testing Strategy

### Configuration Testing

```go
func TestConfigLoading(t *testing.T) {
    tempDir := t.TempDir()
    
    // Create test config
    configPath := filepath.Join(tempDir, "config.toml")
    configContent := `
[providers]
default = "openai"

[interface]
show_thinking = false
`
    
    err := os.WriteFile(configPath, []byte(configContent), 0644)
    require.NoError(t, err)
    
    // Load and validate
    loader := &ConfigLoader{userConfigPath: configPath}
    config, err := loader.LoadConfig()
    require.NoError(t, err)
    
    assert.Equal(t, "openai", config.Providers.Default)
    assert.False(t, config.Interface.ShowThinking)
}

func TestCredentialSecurity(t *testing.T) {
    tempDir := t.TempDir()
    credPath := filepath.Join(tempDir, "credentials.toml")
    
    creds := &Credentials{
        OpenAI: ProviderCredentials{
            APIKey: "sk-test123",
        },
    }
    
    err := saveCredentialsToPath(creds, credPath)
    require.NoError(t, err)
    
    // Check file permissions
    info, err := os.Stat(credPath)
    require.NoError(t, err)
    assert.Equal(t, os.FileMode(0600), info.Mode().Perm())
}
```

## Integration Points

### TUI Integration

The configuration system integrates with the TUI to provide real-time feedback:

```go
type ConfigModel struct {
    config      *Config
    credentials *Credentials
    status      ConfigStatus
}

func (m ConfigModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
    switch msg := msg.(type) {
    case ConfigChangedMsg:
        m.config = msg.Config
        return m, nil
    case CredentialsValidatedMsg:
        m.status = StatusValid
        return m, nil
    }
    return m, nil
}

func (m ConfigModel) View() string {
    var status strings.Builder
    
    status.WriteString("Configuration Status:\n")
    
    for provider, cred := range m.credentials.GetAll() {
        icon := "✗"
        if cred.IsValid() {
            icon = "✓"
        }
        status.WriteString(fmt.Sprintf("  %s %s\n", icon, provider))
    }
    
    return status.String()
}
```

## Performance Considerations

### Caching Strategy

- **Config Loading**: Cache parsed configuration in memory
- **Model Registry**: Cache models.dev API response for 24 hours
- **Credential Validation**: Cache validation results for session duration
- **File Watching**: Monitor config files for changes and reload automatically

### Lazy Loading

```go
type LazyConfig struct {
    once   sync.Once
    config *Config
    err    error
}

func (l *LazyConfig) Get() (*Config, error) {
    l.once.Do(func() {
        l.config, l.err = loadConfiguration()
    })
    return l.config, l.err
}
```

## Migration Strategy

### Version Compatibility

The configuration system includes version detection and automatic migration:

```go
type ConfigVersion struct {
    Version string `toml:"version"`
}

func migrateConfig(path string) error {
    version, err := detectConfigVersion(path)
    if err != nil {
        return err
    }
    
    switch version {
    case "1.0":
        return migrateFromV1ToV2(path)
    case "2.0":
        return nil // current version
    default:
        return fmt.Errorf("unsupported config version: %s", version)
    }
}
```

This configuration architecture provides a robust, secure, and user-friendly foundation for managing Aircher's settings while maintaining the principle of smart defaults and minimal user configuration burden.