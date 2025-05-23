package repl

import (
	"bufio"
	"context"
	"fmt"
	"os"
	"strings"

	"github.com/aircher/aircher/internal/config"
	"github.com/rs/zerolog"
)

// REPL represents the interactive Read-Eval-Print Loop
type REPL struct {
	config *config.Config
	logger zerolog.Logger
	core   AircherCore // Interface to avoid circular imports
	reader *bufio.Reader
	ctx    context.Context
}

// AircherCore interface to avoid circular dependency
type AircherCore interface {
	GetConfig() interface{}
	GetProviderManager() interface{}
	GetContextEngine() interface{}
	GetSearchEngine() interface{}
	GetMemoryManager() interface{}
	GetMCPManager() interface{}
	GetCommandRouter() interface{}
}

// Session represents a conversation session
type Session struct {
	ID          string
	StartTime   int64
	LastUpdate  int64
	MessageCount int
	Provider    string
	Model       string
}

// New creates a new REPL instance
func New(cfg *config.Config, logger zerolog.Logger) (*REPL, error) {
	return &REPL{
		config: cfg,
		logger: logger.With().Str("component", "repl").Logger(),
		reader: bufio.NewReader(os.Stdin),
		ctx:    context.Background(),
	}, nil
}

// SetCore sets the Aircher core instance
func (r *REPL) SetCore(core AircherCore) {
	r.core = core
}

// Run starts the interactive REPL
func (r *REPL) Run(session *Session, initialPrompt string) error {
	r.logger.Info().Str("session_id", session.ID).Msg("Starting REPL")

	// Show initial prompt if provided
	if initialPrompt != "" {
		fmt.Printf("> %s\n", initialPrompt)
		// TODO: Process initial prompt
		fmt.Println("Response processing not yet implemented")
	}

	// Main REPL loop
	for {
		fmt.Print("> ")
		
		input, err := r.reader.ReadString('\n')
		if err != nil {
			return fmt.Errorf("error reading input: %w", err)
		}

		input = strings.TrimSpace(input)
		
		// Handle empty input
		if input == "" {
			continue
		}

		// Handle exit commands
		if input == "exit" || input == "quit" || input == "/quit" {
			fmt.Println("Goodbye!")
			break
		}

		// Handle slash commands
		if strings.HasPrefix(input, "/") {
			if err := r.handleSlashCommand(input); err != nil {
				fmt.Printf("Error: %v\n", err)
			}
			continue
		}

		// Process regular prompt
		if err := r.processPrompt(input); err != nil {
			fmt.Printf("Error: %v\n", err)
		}
	}

	return nil
}

// handleSlashCommand processes slash commands
func (r *REPL) handleSlashCommand(command string) error {
	parts := strings.Fields(command)
	if len(parts) == 0 {
		return fmt.Errorf("empty command")
	}

	cmd := parts[0]
	args := parts[1:]

	switch cmd {
	case "/help":
		r.showHelp()
	case "/clear":
		r.clearScreen()
	case "/config":
		return r.showConfig()
	case "/cost":
		return r.showCostStats()
	case "/memory":
		return r.editMemory()
	case "/search":
		if len(args) == 0 {
			return fmt.Errorf("search command requires a query")
		}
		return r.forceSearch(strings.Join(args, " "))
	case "/think":
		return r.toggleThinking()
	case "/mcp":
		return r.manageMCP(args)
	case "/tools":
		return r.listTools()
	default:
		return fmt.Errorf("unknown command: %s", cmd)
	}

	return nil
}

// processPrompt processes a regular user prompt
func (r *REPL) processPrompt(prompt string) error {
	// TODO: Implement actual prompt processing
	fmt.Printf("Processing: %s\n", prompt)
	fmt.Println("Prompt processing not yet implemented")
	return nil
}

// showHelp displays available commands
func (r *REPL) showHelp() {
	fmt.Println("Available commands:")
	fmt.Println("  /help                    - Show this help")
	fmt.Println("  /clear                   - Clear conversation")
	fmt.Println("  /config                  - Settings management")
	fmt.Println("  /cost                    - Usage and cost statistics")
	fmt.Println("  /memory                  - Edit AIRCHER.md memory")
	fmt.Println("  /search [query]          - Force web search")
	fmt.Println("  /think                   - Enable thinking mode")
	fmt.Println("  /mcp                     - MCP server management")
	fmt.Println("  /tools                   - List available MCP tools")
	fmt.Println("  exit, quit, /quit        - Exit Aircher")
}

// clearScreen clears the terminal screen
func (r *REPL) clearScreen() {
	fmt.Print("\033[2J\033[H")
}

// showConfig displays current configuration
func (r *REPL) showConfig() error {
	fmt.Println("Configuration display not yet implemented")
	return nil
}

// showCostStats displays usage and cost statistics
func (r *REPL) showCostStats() error {
	fmt.Println("Cost statistics not yet implemented")
	return nil
}

// editMemory opens memory editing interface
func (r *REPL) editMemory() error {
	fmt.Println("Memory editing not yet implemented")
	return nil
}

// forceSearch performs a forced web search
func (r *REPL) forceSearch(query string) error {
	fmt.Printf("Forced search for: %s\n", query)
	fmt.Println("Search not yet implemented")
	return nil
}

// toggleThinking toggles thinking mode
func (r *REPL) toggleThinking() error {
	fmt.Println("Thinking mode toggle not yet implemented")
	return nil
}

// manageMCP handles MCP server management
func (r *REPL) manageMCP(args []string) error {
	if len(args) == 0 {
		fmt.Println("MCP Commands:")
		fmt.Println("  /mcp list              - List installed MCP servers")
		fmt.Println("  /mcp install [server]  - Install an MCP server")
		fmt.Println("  /mcp enable [server]   - Enable an MCP server")
		fmt.Println("  /mcp disable [server]  - Disable an MCP server")
		fmt.Println("  /mcp status            - Show MCP server status")
		return nil
	}

	subcommand := args[0]
	switch subcommand {
	case "list":
		fmt.Println("MCP server listing not yet implemented")
	case "install":
		if len(args) < 2 {
			return fmt.Errorf("install command requires server name")
		}
		fmt.Printf("Installing MCP server: %s\n", args[1])
		fmt.Println("MCP installation not yet implemented")
	case "enable", "disable":
		if len(args) < 2 {
			return fmt.Errorf("%s command requires server name", subcommand)
		}
		fmt.Printf("%s MCP server: %s\n", strings.Title(subcommand), args[1])
		fmt.Printf("MCP %s not yet implemented\n", subcommand)
	case "status":
		fmt.Println("MCP status not yet implemented")
	default:
		return fmt.Errorf("unknown MCP command: %s", subcommand)
	}

	return nil
}

// listTools lists available MCP tools
func (r *REPL) listTools() error {
	fmt.Println("Tool listing not yet implemented")
	return nil
}