package core

import (
	"context"
	"fmt"
	"io"
	"os"
	"path/filepath"

	"github.com/aircher/aircher/internal/commands"
	"github.com/aircher/aircher/internal/config"
	contextengine "github.com/aircher/aircher/internal/context"
	"github.com/aircher/aircher/internal/memory"
	"github.com/aircher/aircher/internal/mcp"
	"github.com/aircher/aircher/internal/providers"
	"github.com/aircher/aircher/internal/repl"
	"github.com/aircher/aircher/internal/search"
	"github.com/aircher/aircher/internal/storage"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

// AircherCore represents the main application structure
type AircherCore struct {
	config         *config.Config
	repl           *repl.REPL
	sessionManager *SessionManager
	commandRouter  *commands.Router
	contextEngine  *contextengine.Engine
	providerMgr    *providers.Manager
	storageEngine  *storage.Engine
	searchEngine   *search.Engine
	memoryManager  *memory.Manager
	mcpManager     *mcp.Manager
	logger         zerolog.Logger
	workingDir     string
	projectRoot    string
}

// SessionManager handles conversation sessions
type SessionManager struct {
	storage       *storage.Engine
	currentSession *Session
	logger        zerolog.Logger
}

// Session represents an active conversation session
type Session struct {
	ID          string
	StartTime   int64
	LastUpdate  int64
	MessageCount int
	Provider    string
	Model       string
	Context     *context.Context
}

// NewAircher initializes a new Aircher instance
func NewAircher() (*AircherCore, error) {
	// Setup logging
	logger := log.With().Str("component", "core").Logger()

	// Get working directory
	workingDir, err := os.Getwd()
	if err != nil {
		return nil, fmt.Errorf("failed to get working directory: %w", err)
	}

	// Find project root (look for .git, go.mod, etc.)
	projectRoot := findProjectRoot(workingDir)

	// Load configuration
	cfg, err := config.Load(projectRoot)
	if err != nil {
		return nil, fmt.Errorf("failed to load configuration: %w", err)
	}

	// Initialize storage engine
	storageEngine, err := storage.NewEngine(cfg.GetStorageDir())
	if err != nil {
		return nil, fmt.Errorf("failed to initialize storage: %w", err)
	}

	// Initialize session manager
	sessionManager := &SessionManager{
		storage: storageEngine,
		logger:  logger.With().Str("component", "session").Logger(),
	}

	// Initialize provider manager
	providerMgr, err := providers.NewManager(cfg, logger.With().Str("component", "providers").Logger())
	if err != nil {
		return nil, fmt.Errorf("failed to initialize provider manager: %w", err)
	}

	// Initialize context engine
	contextEngine, err := contextengine.NewEngine(cfg, storageEngine, logger.With().Str("component", "context").Logger())
	if err != nil {
		return nil, fmt.Errorf("failed to initialize context engine: %w", err)
	}

	// Initialize search engine
	searchEngine, err := search.NewEngine(cfg, logger.With().Str("component", "search").Logger())
	if err != nil {
		return nil, fmt.Errorf("failed to initialize search engine: %w", err)
	}

	// Initialize memory manager
	memoryManager, err := memory.NewManager(cfg, projectRoot, storageEngine, logger.With().Str("component", "memory").Logger())
	if err != nil {
		return nil, fmt.Errorf("failed to initialize memory manager: %w", err)
	}

	// Initialize MCP manager
	mcpManager, err := mcp.NewManager(cfg, logger.With().Str("component", "mcp").Logger())
	if err != nil {
		return nil, fmt.Errorf("failed to initialize MCP manager: %w", err)
	}

	// Initialize command router
	commandRouter, err := commands.NewRouter(cfg, projectRoot, logger.With().Str("component", "commands").Logger())
	if err != nil {
		return nil, fmt.Errorf("failed to initialize command router: %w", err)
	}

	// Initialize REPL
	replInstance, err := repl.New(cfg, logger.With().Str("component", "repl").Logger())
	if err != nil {
		return nil, fmt.Errorf("failed to initialize REPL: %w", err)
	}

	aircher := &AircherCore{
		config:         cfg,
		repl:           replInstance,
		sessionManager: sessionManager,
		commandRouter:  commandRouter,
		contextEngine:  contextEngine,
		providerMgr:    providerMgr,
		storageEngine:  storageEngine,
		searchEngine:   searchEngine,
		memoryManager:  memoryManager,
		mcpManager:     mcpManager,
		logger:         logger,
		workingDir:     workingDir,
		projectRoot:    projectRoot,
	}

	// Wire up dependencies
	replInstance.SetCore(aircher)
	commandRouter.SetCore(aircher)

	return aircher, nil
}

// RunInteractive starts the interactive REPL mode
func (a *AircherCore) RunInteractive(initialPrompt string) error {
	a.logger.Info().Str("mode", "interactive").Msg("Starting Aircher")

	// Load project memory
	if err := a.memoryManager.LoadProjectMemory(); err != nil {
		a.logger.Warn().Err(err).Msg("Failed to load project memory")
	}

	// Start MCP servers
	if err := a.mcpManager.StartServers(context.Background()); err != nil {
		a.logger.Warn().Err(err).Msg("Failed to start MCP servers")
	}

	// Create or resume session
	session, err := a.sessionManager.CreateOrResumeSession()
	if err != nil {
		return fmt.Errorf("failed to create session: %w", err)
	}

	// Show welcome message
	a.showWelcomeMessage()

	// Start REPL with initial prompt
	replSession := &repl.Session{
		ID:          session.ID,
		StartTime:   session.StartTime,
		LastUpdate:  session.LastUpdate,
		MessageCount: session.MessageCount,
		Provider:    session.Provider,
		Model:       session.Model,
	}
	return a.repl.Run(replSession, initialPrompt)
}

// RunNonInteractive processes a single prompt and exits
func (a *AircherCore) RunNonInteractive(prompt, outputFormat string) error {
	a.logger.Info().Str("mode", "non-interactive").Str("format", outputFormat).Msg("Processing prompt")

	// Check for piped input
	var input string = prompt
	if !isTerminal(os.Stdin) {
		pipedInput, err := io.ReadAll(os.Stdin)
		if err != nil {
			return fmt.Errorf("failed to read piped input: %w", err)
		}
		if len(pipedInput) > 0 {
			input = fmt.Sprintf("%s\n\n%s", prompt, string(pipedInput))
		}
	}

	// Load project memory
	if err := a.memoryManager.LoadProjectMemory(); err != nil {
		a.logger.Warn().Err(err).Msg("Failed to load project memory")
	}

	// Start MCP servers
	if err := a.mcpManager.StartServers(context.Background()); err != nil {
		a.logger.Warn().Err(err).Msg("Failed to start MCP servers")
	}

	// Create temporary session
	session, err := a.sessionManager.CreateTemporarySession()
	if err != nil {
		return fmt.Errorf("failed to create session: %w", err)
	}

	// Process the prompt
	response, err := a.processPrompt(session, input)
	if err != nil {
		return fmt.Errorf("failed to process prompt: %w", err)
	}

	// Output response in requested format
	return a.outputResponse(response, outputFormat)
}

// ContinueLastConversation resumes the most recent conversation
func (a *AircherCore) ContinueLastConversation() error {
	_, err := a.sessionManager.GetLastSession()
	if err != nil {
		return fmt.Errorf("failed to get last session: %w", err)
	}

	a.logger.Info().Msg("Continuing last conversation")
	
	return a.RunInteractive("")
}

// ResumeSession resumes a specific conversation session
func (a *AircherCore) ResumeSession(sessionID string) error {
	_, err := a.sessionManager.GetSession(sessionID)
	if err != nil {
		return fmt.Errorf("failed to get session: %w", err)
	}

	a.logger.Info().Str("session_id", sessionID).Msg("Resuming session")
	
	return a.RunInteractive("")
}

// RunConfigManager starts the interactive configuration interface
func (a *AircherCore) RunConfigManager() error {
	a.logger.Info().Msg("Starting configuration manager")
	// TODO: Implement interactive config management
	return fmt.Errorf("configuration manager not yet implemented")
}

// InitializeProject creates AIRCHER.md and sets up project structure
func (a *AircherCore) InitializeProject() error {
	a.logger.Info().Msg("Initializing project")
	return a.memoryManager.InitializeProject()
}

// RunHealthCheck performs system diagnostics
func (a *AircherCore) RunHealthCheck() error {
	a.logger.Info().Msg("Running health check")
	// TODO: Implement health diagnostics
	return fmt.Errorf("health check not yet implemented")
}

// RunSelfUpdate performs application self-update
func (a *AircherCore) RunSelfUpdate() error {
	a.logger.Info().Msg("Running self-update")
	// TODO: Implement self-update functionality
	return fmt.Errorf("self-update not yet implemented")
}

// Close gracefully shuts down Aircher
func (a *AircherCore) Close() error {
	a.logger.Info().Msg("Shutting down Aircher")

	var errors []error

	// Stop MCP servers
	if err := a.mcpManager.StopServers(); err != nil {
		errors = append(errors, fmt.Errorf("failed to stop MCP servers: %w", err))
	}

	// Close storage
	if err := a.storageEngine.Close(); err != nil {
		errors = append(errors, fmt.Errorf("failed to close storage: %w", err))
	}

	if len(errors) > 0 {
		return fmt.Errorf("shutdown errors: %v", errors)
	}

	return nil
}

// GetConfig returns the current configuration
func (a *AircherCore) GetConfig() interface{} {
	return a.config
}

// GetProviderManager returns the provider manager
func (a *AircherCore) GetProviderManager() interface{} {
	return a.providerMgr
}

// GetContextEngine returns the context engine
func (a *AircherCore) GetContextEngine() interface{} {
	return a.contextEngine
}

// GetSearchEngine returns the search engine
func (a *AircherCore) GetSearchEngine() interface{} {
	return a.searchEngine
}

// GetMemoryManager returns the memory manager
func (a *AircherCore) GetMemoryManager() interface{} {
	return a.memoryManager
}

// GetMCPManager returns the MCP manager
func (a *AircherCore) GetMCPManager() interface{} {
	return a.mcpManager
}

// GetCommandRouter returns the command router
func (a *AircherCore) GetCommandRouter() interface{} {
	return a.commandRouter
}

// Helper methods

func (a *AircherCore) processPrompt(session *Session, prompt string) (string, error) {
	// TODO: Implement prompt processing with context, providers, etc.
	return "Response processing not yet implemented", nil
}

func (a *AircherCore) outputResponse(response, format string) error {
	switch format {
	case "json":
		// TODO: Implement JSON output
		fmt.Printf(`{"response": %q}`, response)
	case "markdown":
		// TODO: Implement Markdown output
		fmt.Printf("## Response\n\n%s\n", response)
	default:
		fmt.Print(response)
	}
	return nil
}

func (a *AircherCore) showWelcomeMessage() {
	fmt.Printf("Welcome to Aircher! ")
	
	// Detect project type and file count
	if projectInfo := a.contextEngine.GetProjectInfo(); projectInfo != nil {
		fmt.Printf("Detected %s project with %d files.\n", 
			projectInfo.Type, projectInfo.FileCount)
	} else {
		fmt.Printf("Ready to assist.\n")
	}
	fmt.Println()
}

func findProjectRoot(startDir string) string {
	dir := startDir
	for {
		// Check for common project indicators
		for _, indicator := range []string{".git", "go.mod", "package.json", "Cargo.toml", "pyproject.toml"} {
			if _, err := os.Stat(filepath.Join(dir, indicator)); err == nil {
				return dir
			}
		}

		parent := filepath.Dir(dir)
		if parent == dir {
			// Reached filesystem root
			break
		}
		dir = parent
	}
	
	// Default to starting directory
	return startDir
}

func isTerminal(f *os.File) bool {
	stat, err := f.Stat()
	if err != nil {
		return false
	}
	return (stat.Mode() & os.ModeCharDevice) != 0
}

// Session manager methods

func (sm *SessionManager) CreateOrResumeSession() (*Session, error) {
	// TODO: Implement session creation/resumption logic
	return &Session{
		ID:        "temp-session",
		StartTime: 0,
	}, nil
}

func (sm *SessionManager) CreateTemporarySession() (*Session, error) {
	// TODO: Implement temporary session creation
	return &Session{
		ID:        "temp-session",
		StartTime: 0,
	}, nil
}

func (sm *SessionManager) GetLastSession() (*Session, error) {
	// TODO: Implement last session retrieval
	return &Session{
		ID:        "last-session",
		StartTime: 0,
	}, nil
}

func (sm *SessionManager) GetSession(sessionID string) (*Session, error) {
	// TODO: Implement session retrieval by ID
	return &Session{
		ID:        sessionID,
		StartTime: 0,
	}, nil
}