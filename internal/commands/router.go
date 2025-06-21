package commands

import (
	"fmt"
	"strings"

	"github.com/aircher/aircher/internal/config"
	"github.com/rs/zerolog"
)

// Router handles slash command routing and execution
type Router struct {
	config          *config.Config
	projectRoot     string
	builtInCommands map[string]Command
	projectCommands map[string]CustomCommand
	userCommands    map[string]CustomCommand
	middleware      []MiddlewareFunc
	logger          zerolog.Logger
	core            AircherCore // Interface to avoid circular imports
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

// Command represents a built-in slash command
type Command interface {
	Name() string
	Description() string
	Usage() string
	Execute(ctx *CommandContext) error
}

// CustomCommand represents a user or project-defined command
type CustomCommand struct {
	Name      string                 `json:"name"`
	Scope     CommandScope           `json:"scope"`
	FilePath  string                 `json:"file_path"`
	Content   string                 `json:"content"`
	Arguments []CommandArgument      `json:"arguments,omitempty"`
	Metadata  map[string]interface{} `json:"metadata,omitempty"`
}

// CommandScope defines where a command is available
type CommandScope string

const (
	ScopeBuiltIn CommandScope = "builtin"
	ScopeProject CommandScope = "project"
	ScopeUser    CommandScope = "user"
)

// CommandArgument represents a command argument definition
type CommandArgument struct {
	Name        string      `json:"name"`
	Type        string      `json:"type"`
	Required    bool        `json:"required"`
	Default     interface{} `json:"default,omitempty"`
	Description string      `json:"description,omitempty"`
}

// CommandContext provides context for command execution
type CommandContext struct {
	Command   string
	Arguments []string
	RawInput  string
	Core      AircherCore
	Logger    zerolog.Logger
}

// MiddlewareFunc represents command middleware
type MiddlewareFunc func(*CommandContext, func() error) error

// Core built-in commands
var coreCommands = map[string]Command{
	"help":   &HelpCommand{},
	"clear":  &ClearCommand{},
	"config": &ConfigCommand{},
	"cost":   &CostCommand{},
	"memory": &MemoryCommand{},
	"search": &SearchCommand{},
	"think":  &ThinkCommand{},
	"mcp":    &MCPCommand{},
	"tools":  &ToolsCommand{},
}

// NewRouter creates a new command router
func NewRouter(cfg *config.Config, projectRoot string, logger zerolog.Logger) (*Router, error) {
	router := &Router{
		config:          cfg,
		projectRoot:     projectRoot,
		builtInCommands: make(map[string]Command),
		projectCommands: make(map[string]CustomCommand),
		userCommands:    make(map[string]CustomCommand),
		middleware:      make([]MiddlewareFunc, 0),
		logger:          logger.With().Str("component", "commands").Logger(),
	}

	// Load built-in commands
	for name, cmd := range coreCommands {
		router.builtInCommands[name] = cmd
	}

	// TODO: Load custom commands from files
	if err := router.loadCustomCommands(); err != nil {
		router.logger.Warn().Err(err).Msg("Failed to load custom commands")
	}

	return router, nil
}

// SetCore sets the Aircher core instance
func (r *Router) SetCore(core AircherCore) {
	r.core = core
}

// Route routes and executes a slash command
func (r *Router) Route(input string) error {
	if !strings.HasPrefix(input, "/") {
		return fmt.Errorf("not a slash command")
	}

	// Parse command and arguments
	parts := strings.Fields(input)
	if len(parts) == 0 {
		return fmt.Errorf("empty command")
	}

	cmdName := strings.TrimPrefix(parts[0], "/")
	args := parts[1:]

	// Create command context
	ctx := &CommandContext{
		Command:   cmdName,
		Arguments: args,
		RawInput:  input,
		Core:      r.core,
		Logger:    r.logger.With().Str("command", cmdName).Logger(),
	}

	// Execute command with middleware
	return r.executeWithMiddleware(ctx)
}

// executeWithMiddleware executes a command with middleware
func (r *Router) executeWithMiddleware(ctx *CommandContext) error {
	var execute func() error

	execute = func() error {
		return r.executeCommand(ctx)
	}

	// Apply middleware in reverse order
	for i := len(r.middleware) - 1; i >= 0; i-- {
		middleware := r.middleware[i]
		prevExecute := execute
		execute = func() error {
			return middleware(ctx, prevExecute)
		}
	}

	return execute()
}

// executeCommand executes the actual command
func (r *Router) executeCommand(ctx *CommandContext) error {
	// Try built-in commands first
	if cmd, exists := r.builtInCommands[ctx.Command]; exists {
		return cmd.Execute(ctx)
	}

	// Try project commands
	if cmd, exists := r.projectCommands[ctx.Command]; exists {
		return r.executeCustomCommand(ctx, &cmd)
	}

	// Try user commands
	if cmd, exists := r.userCommands[ctx.Command]; exists {
		return r.executeCustomCommand(ctx, &cmd)
	}

	return fmt.Errorf("unknown command: /%s", ctx.Command)
}

// executeCustomCommand executes a custom command
func (r *Router) executeCustomCommand(ctx *CommandContext, cmd *CustomCommand) error {
	// TODO: Implement custom command execution
	ctx.Logger.Info().Str("scope", string(cmd.Scope)).Msg("Executing custom command (stub)")
	return fmt.Errorf("custom command execution not yet implemented")
}

// loadCustomCommands loads custom commands from files
func (r *Router) loadCustomCommands() error {
	// TODO: Implement custom command loading from .aircher/commands/
	r.logger.Debug().Msg("Custom command loading not yet implemented")
	return nil
}

// GetAvailableCommands returns list of available commands
func (r *Router) GetAvailableCommands() map[string]Command {
	commands := make(map[string]Command)

	// Add built-in commands
	for name, cmd := range r.builtInCommands {
		commands[name] = cmd
	}

	// TODO: Add custom commands

	return commands
}

// AddMiddleware adds middleware to the command router
func (r *Router) AddMiddleware(middleware MiddlewareFunc) {
	r.middleware = append(r.middleware, middleware)
}

// Built-in command implementations

// HelpCommand shows available commands
type HelpCommand struct{}

func (c *HelpCommand) Name() string        { return "help" }
func (c *HelpCommand) Description() string { return "Show available commands" }
func (c *HelpCommand) Usage() string       { return "/help" }

func (c *HelpCommand) Execute(ctx *CommandContext) error {
	// TODO: Implement help display
	fmt.Println("Help command not yet implemented")
	return nil
}

// ClearCommand clears the conversation
type ClearCommand struct{}

func (c *ClearCommand) Name() string        { return "clear" }
func (c *ClearCommand) Description() string { return "Clear conversation" }
func (c *ClearCommand) Usage() string       { return "/clear" }

func (c *ClearCommand) Execute(ctx *CommandContext) error {
	// TODO: Implement conversation clearing
	fmt.Println("Clear command not yet implemented")
	return nil
}

// ConfigCommand manages settings
type ConfigCommand struct{}

func (c *ConfigCommand) Name() string        { return "config" }
func (c *ConfigCommand) Description() string { return "Settings management" }
func (c *ConfigCommand) Usage() string       { return "/config [key] [value]" }

func (c *ConfigCommand) Execute(ctx *CommandContext) error {
	// TODO: Implement config management
	fmt.Println("Config command not yet implemented")
	return nil
}

// CostCommand shows usage statistics
type CostCommand struct{}

func (c *CostCommand) Name() string        { return "cost" }
func (c *CostCommand) Description() string { return "Usage and cost statistics" }
func (c *CostCommand) Usage() string       { return "/cost" }

func (c *CostCommand) Execute(ctx *CommandContext) error {
	// TODO: Implement cost display
	fmt.Println("Cost command not yet implemented")
	return nil
}

// MemoryCommand manages project memory
type MemoryCommand struct{}

func (c *MemoryCommand) Name() string        { return "memory" }
func (c *MemoryCommand) Description() string { return "Edit AGENTS.md memory" }
func (c *MemoryCommand) Usage() string       { return "/memory [add|edit|show] [content]" }

func (c *MemoryCommand) Execute(ctx *CommandContext) error {
	// TODO: Implement memory management
	fmt.Println("Memory command not yet implemented")
	return nil
}

// SearchCommand performs web search
type SearchCommand struct{}

func (c *SearchCommand) Name() string        { return "search" }
func (c *SearchCommand) Description() string { return "Force web search" }
func (c *SearchCommand) Usage() string       { return "/search <query>" }

func (c *SearchCommand) Execute(ctx *CommandContext) error {
	if len(ctx.Arguments) == 0 {
		return fmt.Errorf("search command requires a query")
	}
	// TODO: Implement search
	fmt.Printf("Search command not yet implemented for: %s\n", strings.Join(ctx.Arguments, " "))
	return nil
}

// ThinkCommand toggles thinking mode
type ThinkCommand struct{}

func (c *ThinkCommand) Name() string        { return "think" }
func (c *ThinkCommand) Description() string { return "Enable thinking mode" }
func (c *ThinkCommand) Usage() string       { return "/think" }

func (c *ThinkCommand) Execute(ctx *CommandContext) error {
	// TODO: Implement thinking mode toggle
	fmt.Println("Think command not yet implemented")
	return nil
}

// MCPCommand manages MCP servers
type MCPCommand struct{}

func (c *MCPCommand) Name() string        { return "mcp" }
func (c *MCPCommand) Description() string { return "MCP server management" }
func (c *MCPCommand) Usage() string       { return "/mcp <subcommand>" }

func (c *MCPCommand) Execute(ctx *CommandContext) error {
	// TODO: Implement MCP management
	fmt.Println("MCP command not yet implemented")
	return nil
}

// ToolsCommand lists available tools
type ToolsCommand struct{}

func (c *ToolsCommand) Name() string        { return "tools" }
func (c *ToolsCommand) Description() string { return "List available MCP tools" }
func (c *ToolsCommand) Usage() string       { return "/tools" }

func (c *ToolsCommand) Execute(ctx *CommandContext) error {
	// TODO: Implement tools listing
	fmt.Println("Tools command not yet implemented")
	return nil
}
