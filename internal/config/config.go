package config

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/BurntSushi/toml"
)

// Config represents the complete Aircher configuration
type Config struct {
	Project           ProjectConfig           `toml:"project"`
	Interface         InterfaceConfig         `toml:"interface"`
	Providers         ProvidersConfig         `toml:"providers"`
	ContextManagement ContextManagementConfig `toml:"context_management"`
	Search            SearchConfig            `toml:"search"`
	Memory            MemoryConfig            `toml:"memory"`
	Costs             CostsConfig             `toml:"costs"`
	MCP               MCPConfig               `toml:"mcp"`
	Security          SecurityConfig          `toml:"security"`

	// Internal fields
	configPath string
	projectDir string
}

// ProjectConfig contains project-specific settings
type ProjectConfig struct {
	Name string `toml:"name"`
	Path string `toml:"path"`
	Type string `toml:"type"`
}

// InterfaceConfig contains UI/UX settings
type InterfaceConfig struct {
	Mode           string `toml:"mode"`
	OutputFormat   string `toml:"output_format"`
	VimMode        bool   `toml:"vim_mode"`
	ColorTheme     string `toml:"color_theme"`
	ShowThinking   bool   `toml:"show_thinking"`
	ShowTokenCount bool   `toml:"show_token_count"`
	ShowCost       bool   `toml:"show_cost"`
}

// ProvidersConfig contains LLM provider configurations
type ProvidersConfig struct {
	Default string               `toml:"default"`
	OpenAI  OpenAIProviderConfig `toml:"openai"`
	Claude  ClaudeProviderConfig `toml:"claude"`
	Gemini  GeminiProviderConfig `toml:"gemini"`
	Ollama  OllamaProviderConfig `toml:"ollama"`
}

// OpenAIProviderConfig contains OpenAI-specific settings
type OpenAIProviderConfig struct {
	APIKeyEnv string `toml:"api_key_env"`
	Model     string `toml:"model"`
	MaxTokens int    `toml:"max_tokens"`
	BaseURL   string `toml:"base_url,omitempty"`
}

// ClaudeProviderConfig contains Claude-specific settings
type ClaudeProviderConfig struct {
	APIKeyEnv string `toml:"api_key_env"`
	Model     string `toml:"model"`
	MaxTokens int    `toml:"max_tokens"`
}

// GeminiProviderConfig contains Gemini-specific settings
type GeminiProviderConfig struct {
	APIKeyEnv string `toml:"api_key_env"`
	Model     string `toml:"model"`
	Project   string `toml:"project"`
	MaxTokens int    `toml:"max_tokens"`
}

// OllamaProviderConfig contains Ollama-specific settings
type OllamaProviderConfig struct {
	BaseURL   string `toml:"base_url"`
	Model     string `toml:"model"`
	KeepAlive string `toml:"keep_alive"`
}

// ContextManagementConfig contains context management settings
type ContextManagementConfig struct {
	AutoCompaction AutoCompactionConfig `toml:"auto_compaction"`
	FileRelevance  FileRelevanceConfig  `toml:"file_relevance"`
}

// AutoCompactionConfig contains auto-compaction settings
type AutoCompactionConfig struct {
	Enabled                     bool `toml:"enabled"`
	TaskCompletionTrigger       bool `toml:"task_completion_trigger"`
	ContextShiftTrigger         bool `toml:"context_shift_trigger"`
	QualityDegradationTrigger   bool `toml:"quality_degradation_trigger"`
	TokenThreshold              int  `toml:"token_threshold"`
	PreserveMessages            int  `toml:"preserve_messages"`
}

// FileRelevanceConfig contains file relevance settings
type FileRelevanceConfig struct {
	MaxFiles            int     `toml:"max_files"`
	Threshold           float64 `toml:"threshold"`
	IncludeDependencies bool    `toml:"include_dependencies"`
	HistoricalWeight    float64 `toml:"historical_weight"`
	DecayRate           float64 `toml:"decay_rate"`
}

// SearchConfig contains web search settings
type SearchConfig struct {
	Enabled         bool     `toml:"enabled"`
	AutoSearch      bool     `toml:"auto_search"`
	Providers       []string `toml:"providers"`
	BraveAPIKeyEnv  string   `toml:"brave_api_key_env"`
	MaxResults      int      `toml:"max_results"`
	CacheDuration   string   `toml:"cache_duration"`
}

// MemoryConfig contains memory system settings
type MemoryConfig struct {
	ProjectFile        string `toml:"project_file"`
	AutoSaveDecisions  bool   `toml:"auto_save_decisions"`
	SyncInterval       string `toml:"sync_interval"`
}

// CostsConfig contains cost tracking settings
type CostsConfig struct {
	MonthlyBudget    float64 `toml:"monthly_budget"`
	DailyLimit       float64 `toml:"daily_limit"`
	AlertThreshold   float64 `toml:"alert_threshold"`
	TrackByProvider  bool    `toml:"track_by_provider"`
}

// MCPConfig contains MCP-related settings
type MCPConfig struct {
	Timeout     string           `toml:"timeout"`
	Debug       bool             `toml:"debug"`
	AutoRestart bool             `toml:"auto_restart"`
	AutoInstall bool             `toml:"auto_install"`
	RegistryURL string           `toml:"registry_url"`
	Permissions MCPPermissions   `toml:"permissions"`
	Servers     []MCPServerConfig `toml:"servers"`
}

// MCPPermissions contains MCP permission settings
type MCPPermissions struct {
	FilesystemAllowedPaths  []string `toml:"filesystem_allowed_paths"`
	FilesystemReadonlyPaths []string `toml:"filesystem_readonly_paths"`
	RequireConfirmation     []string `toml:"require_confirmation"`
}

// MCPServerConfig contains individual MCP server configuration
type MCPServerConfig struct {
	Name      string            `toml:"name"`
	Command   string            `toml:"command"`
	Args      []string          `toml:"args,omitempty"`
	Transport string            `toml:"transport"`
	Scope     string            `toml:"scope"`
	Env       map[string]string `toml:"env,omitempty"`
	Enabled   bool              `toml:"enabled"`
}

// SecurityConfig contains security settings
type SecurityConfig struct {
	RequireConfirmation []string `toml:"require_confirmation"`
	SandboxMode         bool     `toml:"sandbox_mode"`
	MaxFileSize         string   `toml:"max_file_size"`
	AllowedExtensions   []string `toml:"allowed_extensions"`
}

// Load loads configuration from the appropriate location
func Load(projectDir string) (*Config, error) {
	config := &Config{
		projectDir: projectDir,
	}

	// Set up default configuration
	config.setDefaults()

	// Try to load from project-specific config first
	projectConfigPath := filepath.Join(projectDir, ".aircher", "config.toml")
	if _, err := os.Stat(projectConfigPath); err == nil {
		config.configPath = projectConfigPath
		if err := config.loadFromFile(projectConfigPath); err != nil {
			return nil, fmt.Errorf("failed to load project config: %w", err)
		}
		return config, nil
	}

	// Fall back to user config
	userConfigDir, err := getUserConfigDir()
	if err != nil {
		return nil, fmt.Errorf("failed to get user config directory: %w", err)
	}

	userConfigPath := filepath.Join(userConfigDir, "aircher", "config.toml")
	config.configPath = userConfigPath

	if _, err := os.Stat(userConfigPath); err == nil {
		if err := config.loadFromFile(userConfigPath); err != nil {
			return nil, fmt.Errorf("failed to load user config: %w", err)
		}
	}

	// Update project-specific fields
	config.Project.Path = projectDir
	if config.Project.Name == "" {
		config.Project.Name = filepath.Base(projectDir)
	}

	return config, nil
}

// setDefaults sets default configuration values
func (c *Config) setDefaults() {
	c.Interface = InterfaceConfig{
		Mode:           "interactive",
		OutputFormat:   "text",
		VimMode:        false,
		ColorTheme:     "auto",
		ShowThinking:   false,
		ShowTokenCount: true,
		ShowCost:       true,
	}

	c.Providers = ProvidersConfig{
		Default: "openai",
		OpenAI: OpenAIProviderConfig{
			APIKeyEnv: "OPENAI_API_KEY",
			Model:     "gpt-4",
			MaxTokens: 4096,
		},
		Claude: ClaudeProviderConfig{
			APIKeyEnv: "ANTHROPIC_API_KEY",
			Model:     "claude-3-sonnet-20240229",
			MaxTokens: 4096,
		},
		Gemini: GeminiProviderConfig{
			APIKeyEnv: "GOOGLE_API_KEY",
			Model:     "gemini-pro",
			MaxTokens: 4096,
		},
		Ollama: OllamaProviderConfig{
			BaseURL:   "http://localhost:11434",
			Model:     "llama2",
			KeepAlive: "5m",
		},
	}

	c.ContextManagement = ContextManagementConfig{
		AutoCompaction: AutoCompactionConfig{
			Enabled:                   true,
			TaskCompletionTrigger:     true,
			ContextShiftTrigger:       true,
			QualityDegradationTrigger: true,
			TokenThreshold:            8000,
			PreserveMessages:          5,
		},
		FileRelevance: FileRelevanceConfig{
			MaxFiles:            20,
			Threshold:           0.3,
			IncludeDependencies: true,
			HistoricalWeight:    0.7,
			DecayRate:           0.1,
		},
	}

	c.Search = SearchConfig{
		Enabled:        true,
		AutoSearch:     true,
		Providers:      []string{"brave"},
		BraveAPIKeyEnv: "BRAVE_API_KEY",
		MaxResults:     5,
		CacheDuration:  "1h",
	}

	c.Memory = MemoryConfig{
		ProjectFile:       "AIRCHER.md",
		AutoSaveDecisions: true,
		SyncInterval:      "5m",
	}

	c.Costs = CostsConfig{
		MonthlyBudget:   100.0,
		DailyLimit:      10.0,
		AlertThreshold:  0.8,
		TrackByProvider: true,
	}

	c.MCP = MCPConfig{
		Timeout:     "30s",
		Debug:       false,
		AutoRestart: true,
		AutoInstall: false,
		RegistryURL: "https://mcp-registry.aircher.ai",
		Permissions: MCPPermissions{
			FilesystemAllowedPaths:  []string{"."},
			FilesystemReadonlyPaths: []string{},
			RequireConfirmation:     []string{"file_write", "file_delete", "git_push", "process_execute"},
		},
		Servers: []MCPServerConfig{
			{
				Name:      "filesystem",
				Command:   "npx",
				Args:      []string{"-y", "@modelcontextprotocol/server-filesystem"},
				Transport: "stdio",
				Scope:     "local",
				Enabled:   true,
			},
		},
	}

	c.Security = SecurityConfig{
		RequireConfirmation: []string{"file_delete", "git_push", "process_execute"},
		SandboxMode:         false,
		MaxFileSize:         "10MB",
		AllowedExtensions:   []string{".txt", ".md", ".go", ".js", ".py", ".rs", ".java", ".c", ".cpp", ".h"},
	}
}

// loadFromFile loads configuration from a TOML file
func (c *Config) loadFromFile(path string) error {
	if _, err := toml.DecodeFile(path, c); err != nil {
		return fmt.Errorf("failed to decode TOML file: %w", err)
	}
	return nil
}

// Save saves the current configuration to file
func (c *Config) Save() error {
	// Ensure config directory exists
	configDir := filepath.Dir(c.configPath)
	if err := os.MkdirAll(configDir, 0755); err != nil {
		return fmt.Errorf("failed to create config directory: %w", err)
	}

	// Create temporary file
	tmpPath := c.configPath + ".tmp"
	file, err := os.Create(tmpPath)
	if err != nil {
		return fmt.Errorf("failed to create temporary config file: %w", err)
	}

	// Encode configuration to TOML
	encoder := toml.NewEncoder(file)
	if err := encoder.Encode(c); err != nil {
		file.Close()
		os.Remove(tmpPath)
		return fmt.Errorf("failed to encode configuration: %w", err)
	}

	file.Close()

	// Atomic replace
	if err := os.Rename(tmpPath, c.configPath); err != nil {
		os.Remove(tmpPath)
		return fmt.Errorf("failed to save configuration: %w", err)
	}

	return nil
}

// GetStorageDir returns the storage directory for databases and cache
func (c *Config) GetStorageDir() string {
	return filepath.Join(c.projectDir, ".aircher")
}

// GetProjectRoot returns the project root directory
func (c *Config) GetProjectRoot() string {
	return c.projectDir
}

// GetConfigPath returns the configuration file path
func (c *Config) GetConfigPath() string {
	return c.configPath
}

// ValidateProviderConfig validates provider configurations
func (c *Config) ValidateProviderConfig() error {
	// Check if default provider is configured
	switch c.Providers.Default {
	case "openai":
		if c.Providers.OpenAI.APIKeyEnv == "" {
			return fmt.Errorf("OpenAI API key environment variable not configured")
		}
	case "claude":
		if c.Providers.Claude.APIKeyEnv == "" {
			return fmt.Errorf("Claude API key environment variable not configured")
		}
	case "gemini":
		if c.Providers.Gemini.APIKeyEnv == "" {
			return fmt.Errorf("Gemini API key environment variable not configured")
		}
	case "ollama":
		if c.Providers.Ollama.BaseURL == "" {
			return fmt.Errorf("Ollama base URL not configured")
		}
	default:
		return fmt.Errorf("unknown default provider: %s", c.Providers.Default)
	}

	return nil
}

// getUserConfigDir returns the user configuration directory
func getUserConfigDir() (string, error) {
	configDir, err := os.UserConfigDir()
	if err != nil {
		return "", err
	}
	return configDir, nil
}

// InitializeConfig creates a default configuration file
func InitializeConfig(projectDir string) error {
	config := &Config{
		projectDir: projectDir,
	}
	config.setDefaults()

	// Set project-specific values
	config.Project.Path = projectDir
	config.Project.Name = filepath.Base(projectDir)

	// Determine config path
	configPath := filepath.Join(projectDir, ".aircher", "config.toml")
	config.configPath = configPath

	return config.Save()
}