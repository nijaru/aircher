package repl

import (
	"context"
	"fmt"
	"os"
	"strings"
	"time"

	"github.com/charmbracelet/bubbles/textarea"
	"github.com/charmbracelet/bubbles/viewport"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/glamour"
	"github.com/charmbracelet/lipgloss"
	"github.com/rs/zerolog"

	"github.com/aircher/aircher/internal/config"
	"github.com/aircher/aircher/internal/providers"
)

// REPL represents the interactive Bubble Tea application
type REPL struct {
	config *config.Config
	logger zerolog.Logger
	core   AircherCore
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

// ProviderManager interface for accessing LLM providers
type ProviderManager interface {
	ChatStream(ctx context.Context, request *providers.ChatRequest) (<-chan *providers.StreamChunk, error)
	GetDefaultProvider() string
	GetDefaultModel(provider string) string
}

// Session represents a conversation session
type Session struct {
	ID           string
	StartTime    int64
	LastUpdate   int64
	MessageCount int
	Provider     string
	Model        string
}

// Message represents a conversation message
type Message struct {
	Role      string
	Content   string
	Timestamp time.Time
	Tokens    int
	Cost      float64
	Provider  string
}

// Model represents the Bubble Tea model
type Model struct {
	// Core components
	input    textarea.Model
	viewport viewport.Model

	// Application state
	messages []Message
	width    int
	height   int
	ready    bool

	// UI state
	streaming     bool
	showHelp      bool
	showContext   bool
	showShortcuts bool

	// Current session
	session         *Session
	currentProvider string

	// Active streaming
	streamChan <-chan *providers.StreamChunk

	// Styling
	styles   Styles
	renderer *glamour.TermRenderer

	// Context
	logger zerolog.Logger
	core   AircherCore
}

// Styles holds all the styling definitions
type Styles struct {
	// Layout styles
	App     lipgloss.Style
	Header  lipgloss.Style
	Footer  lipgloss.Style
	Content lipgloss.Style
	Sidebar lipgloss.Style

	// Message styles
	UserMsg   lipgloss.Style
	AssistMsg lipgloss.Style
	SystemMsg lipgloss.Style
	ErrorMsg  lipgloss.Style

	// Status styles
	StatusBar lipgloss.Style
	Provider  lipgloss.Style
	Cost      lipgloss.Style
	Thinking  lipgloss.Style
	Searching lipgloss.Style

	// Input styles
	Input  lipgloss.Style
	Prompt lipgloss.Style

	// Help styles
	Help     lipgloss.Style
	HelpKey  lipgloss.Style
	HelpDesc lipgloss.Style
}

// Colors
var (
	primaryColor   = lipgloss.Color("#7C3AED")
	secondaryColor = lipgloss.Color("#10B981")
	accentColor    = lipgloss.Color("#F59E0B")
	errorColor     = lipgloss.Color("#EF4444")
	mutedColor     = lipgloss.Color("#6B7280")
	bgColor        = lipgloss.Color("#1F2937")
	borderColor    = lipgloss.Color("#374151")
)

// Messages for Bubble Tea
type (
	streamMsg struct {
		content  string
		done     bool
		tokens   int
		cost     float64
		provider string
		model    string
		isError  bool
	}

	errorMsg struct {
		err error
	}

	statusMsg struct {
		text string
	}
)

// New creates a new REPL instance
func New(cfg *config.Config, logger zerolog.Logger) (*REPL, error) {
	return &REPL{
		config: cfg,
		logger: logger.With().Str("component", "repl").Logger(),
	}, nil
}

// SetCore sets the Aircher core instance
func (r *REPL) SetCore(core AircherCore) {
	r.core = core
}

// Run starts the Bubble Tea application
func (r *REPL) Run(session *Session, initialPrompt string) error {
	r.logger.Info().Str("session_id", session.ID).Msg("Starting TUI REPL")

	// Create the model
	model := r.createModel(session, initialPrompt)

	// Start Bubble Tea
	p := tea.NewProgram(
		model,
		tea.WithAltScreen(),
		tea.WithMouseCellMotion(),
	)

	_, err := p.Run()
	return err
}

// createModel creates the initial Bubble Tea model
func (r *REPL) createModel(session *Session, initialPrompt string) Model {
	// Create input field
	input := textarea.New()
	input.Placeholder = "Ask anything..."
	input.Prompt = "" // Remove prompt to avoid double ">"
	input.Focus()
	input.CharLimit = 2000
	input.SetWidth(80)
	input.SetHeight(1) // Start with single line
	input.ShowLineNumbers = false

	// Create viewport for conversation
	vp := viewport.New(80, 20)

	// Create markdown renderer
	renderer, _ := glamour.NewTermRenderer(
		glamour.WithAutoStyle(),
		glamour.WithWordWrap(78),
	)

	// Create styles
	styles := createStyles()

	// Create model
	model := Model{
		input:           input,
		viewport:        vp,
		messages:        make([]Message, 0),
		session:         session,
		styles:          styles,
		renderer:        renderer,
		logger:          r.logger,
		core:            r.core,
		currentProvider: "ollama", // Default to available provider
	}

	// Don't add welcome to chat history - it will be in header

	// Process initial prompt if provided
	if initialPrompt != "" {
		model.addUserMessage(initialPrompt)
		model.addAssistantMessage("Initial prompt processing not yet implemented.")
	}

	return model
}

// Init initializes the Bubble Tea model
func (m Model) Init() tea.Cmd {
	return textarea.Blink
}

// Update handles Bubble Tea messages
func (m Model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	var cmds []tea.Cmd

	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height

		// Update component sizes
		m.input.SetWidth(m.width - 4)
		if m.showContext {
			m.viewport.Width = m.width - 34
		} else {
			m.viewport.Width = m.width - 4
		}
		m.viewport.Height = m.height - 10
		m.ready = true

		// Update content
		m.updateViewportContent()

	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "esc":
			return m, tea.Quit

		case "ctrl+h":
			m.showShortcuts = !m.showShortcuts

		case "?":
			m.showShortcuts = !m.showShortcuts
			return m, nil

		case "ctrl+t":
			m.showContext = !m.showContext
			// Adjust viewport width when context panel toggles
			if m.showContext {
				m.viewport.Width = m.width - 34
			} else {
				m.viewport.Width = m.width - 4
			}
			m.updateViewportContent()
			return m, nil

		case "enter":
			if m.input.Value() == "" {
				break
			}

			userInput := strings.TrimSpace(m.input.Value())
			m.input.SetValue("")

			// Add user message
			m.addUserMessage(userInput)

			// Handle slash commands
			if strings.HasPrefix(userInput, "/") {
				return m, m.handleSlashCommand(userInput)
			}

			// Process regular input
			return m, m.processInput(userInput)

		case "shift+enter":
			// Allow multiline input - let textarea handle it
			var cmd tea.Cmd
			m.input, cmd = m.input.Update(msg)
			return m, cmd

		default:
			var cmd tea.Cmd
			m.input, cmd = m.input.Update(msg)
			return m, cmd
		}

	case streamMsg:
		// Handle streaming response
		if msg.isError {
			// Add error message
			errorMessage := Message{
				Role:      "assistant",
				Content:   msg.content,
				Timestamp: time.Now(),
				Provider:  "error",
			}
			m.messages = append(m.messages, errorMessage)
			m.streaming = false
		} else if len(m.messages) > 0 && m.messages[len(m.messages)-1].Role == "assistant" {
			// Update the last assistant message by appending new content
			if msg.content != "" {
				m.messages[len(m.messages)-1].Content += msg.content
			}
			if msg.done {
				m.messages[len(m.messages)-1].Tokens = msg.tokens
				m.messages[len(m.messages)-1].Cost = msg.cost
				m.messages[len(m.messages)-1].Provider = msg.provider
				m.streaming = false
				m.streamChan = nil
			}
		}
		m.updateViewportContent()

		if !msg.done && !msg.isError && m.streamChan != nil {
			// Continue reading from stream
			return m, m.streamCommand(m.streamChan)
		}

	case errorMsg:
		m.addErrorMessage(fmt.Sprintf("Error: %v", msg.err))
		m.updateViewportContent()

	case statusMsg:
		// Handle status updates
		m.addSystemMessage(msg.text)
		m.updateViewportContent()
	}

	// Update viewport
	var cmd tea.Cmd
	m.viewport, cmd = m.viewport.Update(msg)
	cmds = append(cmds, cmd)

	return m, tea.Batch(cmds...)
}

// View renders the Bubble Tea view
func (m Model) View() string {
	if !m.ready {
		return "Initializing Aircher..."
	}

	// Main layout
	header := m.renderHeader()
	content := m.renderContent()
	footer := m.renderFooter()

	return lipgloss.JoinVertical(
		lipgloss.Top,
		header,
		content,
		footer,
	)
}

// renderHeader renders the header with status information
func (m Model) renderHeader() string {
	title := m.styles.Header.Render("🏹 Aircher")

	status := []string{}

	// Provider status
	providerStyle := m.styles.Provider
	if m.currentProvider != "" {
		status = append(status, providerStyle.Render(fmt.Sprintf("Provider: %s", m.currentProvider)))
	}

	// Activity indicators
	if m.streaming {
		status = append(status, m.styles.Searching.Render("📡 Streaming"))
	}

	// Cost information
	totalCost := m.calculateTotalCost()
	if totalCost > 0 {
		status = append(status, m.styles.Cost.Render(fmt.Sprintf("Cost: $%.4f", totalCost)))
	}

	statusBar := strings.Join(status, " • ")

	return m.styles.StatusBar.Width(m.width).Render(
		lipgloss.JoinHorizontal(
			lipgloss.Left,
			title,
			lipgloss.NewStyle().Width(m.width-lipgloss.Width(title)-lipgloss.Width(statusBar)).Render(""),
			statusBar,
		),
	)
}

// renderContent renders the main content area
func (m Model) renderContent() string {
	if m.showHelp {
		return m.renderHelp()
	}

	// Show welcome message when no chat history exists
	if len(m.messages) == 0 {
		return m.renderWelcome()
	}

	if m.showContext {
		// Adjust width when context panel is shown
		mainWidth := m.width - 34 // Account for context panel width + margins
		mainContent := m.styles.Content.Width(mainWidth).Height(m.height - 8).Render(m.viewport.View())
		contextPanel := m.renderContextPanel()
		return lipgloss.JoinHorizontal(
			lipgloss.Top,
			mainContent,
			contextPanel,
		)
	}

	mainContent := m.styles.Content.Width(m.width - 2).Height(m.height - 8).Render(m.viewport.View())
	return mainContent
}

// renderWelcome renders the welcome message when no chat history exists
func (m Model) renderWelcome() string {
	welcomeTitle := m.styles.Header.Render("✻ Welcome to Aircher!")

	cwd, err := os.Getwd()
	if err != nil {
		cwd = "unknown"
	}

	instructions := []string{
		"",
		"/help for commands, /status for your current setup",
		"",
		fmt.Sprintf("cwd: %s", cwd),
		"",
	}

	welcomeBox := m.styles.Content.Width(m.width - 20).Render(
		lipgloss.JoinVertical(
			lipgloss.Left,
			welcomeTitle,
			strings.Join(instructions, "\n"),
		),
	)

	tips := []string{
		"Tips for getting started:",
		"",
		"1. Run /init to create an AGENTS.md file with instructions",
		"2. Use Aircher to help with file analysis, editing, and git operations",
		"3. Use MCP tools for enhanced capabilities like filesystem and git",
		"4. Be as specific as you would with another engineer for best results",
	}

	tipsContent := strings.Join(tips, "\n")

	exampleBox := m.styles.Help.Width(m.width - 20).Render(
		`Try "how do I log an error?"`,
	)

	content := lipgloss.JoinVertical(
		lipgloss.Center,
		"",
		welcomeBox,
		"",
		tipsContent,
		"",
		exampleBox,
		"",
	)

	return lipgloss.Place(
		m.width-2,
		m.height-8,
		lipgloss.Center,
		lipgloss.Center,
		content,
	)
}

// renderFooter renders the input area and help hints
func (m Model) renderFooter() string {
	// Input area with prompt
	prompt := m.styles.Prompt.Render("> ")
	inputArea := m.styles.Input.Width(m.width - 2).Render(
		lipgloss.JoinHorizontal(lipgloss.Left, prompt, m.input.View()),
	)

	// Footer content
	var footerHints string
	if m.showShortcuts {
		// Show shortcuts like Claude Code
		footerHints = m.styles.Help.Render("/ for commands        ! for bash mode        @ for file paths        # to memorize        \\ ⏎ for newline")
	} else {
		// Check if user is typing a slash command for autocomplete hints
		currentInput := m.input.Value()
		if strings.HasPrefix(currentInput, "/") && len(currentInput) > 1 {
			// Show matching commands with descriptions
			prefix := strings.ToLower(currentInput[1:])
			commands := map[string]string{
				"help": "show commands", "clear": "clear chat", "cost": "show usage",
				"config": "settings", "memory": "edit memory", "provider": "switch AI",
				"tools": "list tools", "status": "show setup", "init": "create AGENTS.md",
				"mcp": "manage servers", "session": "manage sessions", "save": "save chat",
				"load": "load chat", "export": "export chat",
			}

			var matches []string
			for cmd, desc := range commands {
				if strings.HasPrefix(cmd, prefix) {
					matches = append(matches, "/"+cmd+" "+desc)
				}
			}
			if len(matches) > 0 && len(matches) <= 3 {
				footerHints = m.styles.HelpDesc.Render(strings.Join(matches, "        "))
			} else if len(matches) > 3 {
				footerHints = m.styles.HelpDesc.Render(fmt.Sprintf("%d commands available", len(matches)))
			}
		} else {
			footerHints = m.styles.Help.Render("? for shortcuts")
		}
	}

	return m.styles.Footer.Width(m.width).Render(
		lipgloss.JoinVertical(
			lipgloss.Top,
			inputArea,
			footerHints,
		),
	)
}

// renderHelp renders the help panel
func (m Model) renderHelp() string {
	helpContent := []string{
		m.styles.HelpKey.Render("🏹 Aircher Commands:"),
		"",
		m.styles.HelpKey.Render("/help") + " - " + m.styles.HelpDesc.Render("Toggle this help panel"),
		m.styles.HelpKey.Render("/clear") + " - " + m.styles.HelpDesc.Render("Clear conversation history"),
		m.styles.HelpKey.Render("/cost") + " - " + m.styles.HelpDesc.Render("Show usage and cost statistics"),
		"",
		m.styles.HelpKey.Render("🔧 Configuration & Memory:"),
		"",
		m.styles.HelpKey.Render("/config <key> [value]") + " - " + m.styles.HelpDesc.Render("Manage settings"),
		m.styles.HelpKey.Render("/memory [add|edit|show]") + " - " + m.styles.HelpDesc.Render("Manage AGENTS.md memory"),
		"",
		m.styles.HelpKey.Render("🔧 Tools & Integration:"),
		"",
		m.styles.HelpKey.Render("/mcp") + " - " + m.styles.HelpDesc.Render("MCP server management"),
		m.styles.HelpKey.Render("/tools") + " - " + m.styles.HelpDesc.Render("List available MCP tools"),
		"",
		m.styles.HelpKey.Render("🔄 Provider & Model:"),
		"",
		m.styles.HelpKey.Render("/provider [name]") + " - " + m.styles.HelpDesc.Render("Switch AI provider (claude, openai, ollama)"),
		m.styles.HelpKey.Render("/model [name]") + " - " + m.styles.HelpDesc.Render("Switch model within current provider"),
		"",
		m.styles.HelpKey.Render("💾 Session Management:"),
		"",
		m.styles.HelpKey.Render("/session [list|new|load]") + " - " + m.styles.HelpDesc.Render("Manage conversation sessions"),
		m.styles.HelpKey.Render("/save [name]") + " - " + m.styles.HelpDesc.Render("Save current conversation"),
		m.styles.HelpKey.Render("/load <name>") + " - " + m.styles.HelpDesc.Render("Load saved conversation"),
		m.styles.HelpKey.Render("/export [format]") + " - " + m.styles.HelpDesc.Render("Export conversation (md, json, txt)"),
		"",
		m.styles.HelpKey.Render("⌨️  Keyboard Shortcuts:"),
		"",
		m.styles.HelpKey.Render("Ctrl+H") + " - " + m.styles.HelpDesc.Render("Toggle this help panel"),
		m.styles.HelpKey.Render("Ctrl+T") + " - " + m.styles.HelpDesc.Render("Toggle context panel"),
		m.styles.HelpKey.Render("Ctrl+C / Esc") + " - " + m.styles.HelpDesc.Render("Exit Aircher"),
		m.styles.HelpKey.Render("Enter") + " - " + m.styles.HelpDesc.Render("Send message"),
		m.styles.HelpKey.Render("Shift+Enter") + " - " + m.styles.HelpDesc.Render("New line (multiline input)"),
	}

	return m.styles.Help.Width(m.width - 4).Height(m.height - 6).Render(
		strings.Join(helpContent, "\n"),
	)
}

// renderContextPanel renders the context sidebar
func (m Model) renderContextPanel() string {
	contextContent := []string{
		m.styles.HelpKey.Render("📁 Context Panel"),
		"",
		m.styles.HelpDesc.Render("Session: ") + m.session.ID,
		m.styles.HelpDesc.Render("Messages: ") + fmt.Sprintf("%d", len(m.messages)),
		m.styles.HelpDesc.Render("Provider: ") + m.currentProvider,
		"",
		m.styles.HelpDesc.Render("🧠 Current State:"),
		m.styles.HelpDesc.Render(fmt.Sprintf("• Streaming: %v", m.streaming)),
		"",
		m.styles.HelpKey.Render("🔧 Available Tools:"),
		m.styles.HelpDesc.Render("• Filesystem operations (MCP)"),
		m.styles.HelpDesc.Render("• Git integration (MCP)"),
		m.styles.HelpDesc.Render("• Web search (automatic)"),
		m.styles.HelpDesc.Render("• Custom MCP tools"),
		"",
		m.styles.HelpKey.Render("📊 Session Stats:"),
		m.styles.HelpDesc.Render(fmt.Sprintf("• Cost: $%.4f", m.calculateTotalCost())),
		m.styles.HelpDesc.Render(fmt.Sprintf("• Tokens: %d", m.calculateTotalTokens())),
		"",
		m.styles.HelpDesc.Render("Press Ctrl+T to close"),
	}

	return m.styles.Sidebar.Width(30).Height(m.height - 8).Render(
		strings.Join(contextContent, "\n"),
	)
}

// Message handling methods
func (m *Model) addUserMessage(content string) {
	msg := Message{
		Role:      "user",
		Content:   content,
		Timestamp: time.Now(),
	}
	m.messages = append(m.messages, msg)
	m.updateViewportContent()
}

func (m *Model) addAssistantMessage(content string) {
	msg := Message{
		Role:      "assistant",
		Content:   content,
		Timestamp: time.Now(),
		Provider:  m.currentProvider,
	}
	m.messages = append(m.messages, msg)
	m.updateViewportContent()
}

func (m *Model) addSystemMessage(content string) {
	msg := Message{
		Role:      "system",
		Content:   content,
		Timestamp: time.Now(),
	}
	m.messages = append(m.messages, msg)
	m.updateViewportContent()
}

func (m *Model) addErrorMessage(content string) {
	msg := Message{
		Role:      "error",
		Content:   content,
		Timestamp: time.Now(),
	}
	m.messages = append(m.messages, msg)
	m.updateViewportContent()
}

// updateViewportContent updates the viewport with formatted messages
func (m *Model) updateViewportContent() {
	var content strings.Builder

	for i, msg := range m.messages {
		if i > 0 {
			content.WriteString("\n\n")
		}

		// Format timestamp
		timestamp := msg.Timestamp.Format("15:04:05")

		switch msg.Role {
		case "user":
			content.WriteString(m.styles.UserMsg.Render(fmt.Sprintf("👤 You [%s]", timestamp)))
			content.WriteString("\n")
			content.WriteString(msg.Content)

		case "assistant":
			providerInfo := ""
			if msg.Provider != "" {
				providerInfo = fmt.Sprintf(" via %s", msg.Provider)
			}
			content.WriteString(m.styles.AssistMsg.Render(fmt.Sprintf("🤖 Aircher%s [%s]", providerInfo, timestamp)))
			content.WriteString("\n")

			// Render as markdown if possible
			if m.renderer != nil {
				if rendered, err := m.renderer.Render(msg.Content); err == nil {
					content.WriteString(rendered)
				} else {
					content.WriteString(msg.Content)
				}
			} else {
				content.WriteString(msg.Content)
			}

		case "system":
			content.WriteString(m.styles.SystemMsg.Render(fmt.Sprintf("ℹ️ System [%s]", timestamp)))
			content.WriteString("\n")
			content.WriteString(msg.Content)

		case "error":
			content.WriteString(m.styles.ErrorMsg.Render(fmt.Sprintf("❌ Error [%s]", timestamp)))
			content.WriteString("\n")
			content.WriteString(msg.Content)
		}
	}

	m.viewport.SetContent(content.String())
	m.viewport.GotoBottom()
}

// Command and input processing
func (m Model) handleSlashCommand(command string) tea.Cmd {
	parts := strings.Fields(command)
	if len(parts) == 0 {
		return func() tea.Msg {
			return errorMsg{fmt.Errorf("empty command")}
		}
	}

	cmd := parts[0]
	args := parts[1:]

	switch cmd {
	case "/help":
		m.showShortcuts = !m.showShortcuts
		return nil

	case "/status":
		statusInfo := []string{
			fmt.Sprintf("Provider: %s", m.currentProvider),
			fmt.Sprintf("Messages: %d", len(m.messages)),
			fmt.Sprintf("Total cost: $%.4f", m.calculateTotalCost()),
			fmt.Sprintf("Total tokens: %d", m.calculateTotalTokens()),
		}
		return func() tea.Msg {
			return statusMsg{strings.Join(statusInfo, " • ")}
		}

	case "/init":
		return func() tea.Msg {
			return statusMsg{"AGENTS.md initialization not yet implemented"}
		}

	case "/clear":
		m.messages = m.messages[:0]
		m.addSystemMessage("Conversation cleared")
		return nil

	case "/config":
		if len(args) == 0 {
			return func() tea.Msg {
				return statusMsg{"Use: /config <key> [value] - Configuration management not yet fully implemented"}
			}
		}
		return func() tea.Msg {
			return statusMsg{"Configuration management not yet implemented"}
		}

	case "/cost":
		cost := m.calculateTotalCost()
		tokens := m.calculateTotalTokens()
		return func() tea.Msg {
			return statusMsg{fmt.Sprintf("Total cost: $%.4f, Total tokens: %d", cost, tokens)}
		}

	case "/memory":
		if len(args) == 0 {
			return func() tea.Msg {
				return statusMsg{"Use: /memory [add|edit|show] [content] - Memory management not yet implemented"}
			}
		}
		return func() tea.Msg {
			return statusMsg{"Memory management not yet implemented"}
		}

	case "/mcp":
		return func() tea.Msg {
			return statusMsg{"MCP server management not yet implemented"}
		}

	case "/tools":
		return func() tea.Msg {
			return statusMsg{"MCP tools listing not yet implemented"}
		}

	case "/provider":
		if len(args) == 0 {
			return func() tea.Msg {
				return statusMsg{fmt.Sprintf("Current provider: %s. Available: claude, openai, ollama, gemini", m.currentProvider)}
			}
		}
		provider := args[0]
		// TODO: Validate and switch provider
		return func() tea.Msg {
			return statusMsg{fmt.Sprintf("Provider switching not yet implemented. Requested: %s", provider)}
		}

	case "/model":
		if len(args) == 0 {
			return func() tea.Msg {
				return statusMsg{"Use: /model <name> - Model switching not yet implemented"}
			}
		}
		model := args[0]
		return func() tea.Msg {
			return statusMsg{fmt.Sprintf("Model switching not yet implemented. Requested: %s", model)}
		}

	case "/session":
		if len(args) == 0 {
			return func() tea.Msg {
				return statusMsg{"Use: /session [list|new|load] - Session management not yet implemented"}
			}
		}
		action := args[0]
		return func() tea.Msg {
			return statusMsg{fmt.Sprintf("Session %s not yet implemented", action)}
		}

	case "/save":
		name := "default"
		if len(args) > 0 {
			name = args[0]
		}
		return func() tea.Msg {
			return statusMsg{fmt.Sprintf("Conversation saving not yet implemented. Would save as: %s", name)}
		}

	case "/load":
		if len(args) == 0 {
			return func() tea.Msg {
				return errorMsg{fmt.Errorf("load command requires a name: /load <name>")}
			}
		}
		name := args[0]
		return func() tea.Msg {
			return statusMsg{fmt.Sprintf("Conversation loading not yet implemented. Would load: %s", name)}
		}

	case "/export":
		format := "md"
		if len(args) > 0 {
			format = args[0]
		}
		return func() tea.Msg {
			return statusMsg{fmt.Sprintf("Export not yet implemented. Would export as: %s", format)}
		}

	default:
		return func() tea.Msg {
			return errorMsg{fmt.Errorf("unknown command: %s. Use /help to see available commands", cmd)}
		}
	}
}

func (m Model) processInput(input string) tea.Cmd {
	m.streaming = true

	// Add user message to conversation immediately
	userMessage := Message{
		Role:      "user",
		Content:   input,
		Timestamp: time.Now(),
		Provider:  "user",
	}
	m.messages = append(m.messages, userMessage)

	// Add empty assistant message for streaming
	assistantMessage := Message{
		Role:      "assistant",
		Content:   "",
		Timestamp: time.Now(),
		Provider:  "",
	}
	m.messages = append(m.messages, assistantMessage)

	return m.startLLMStreamWithThinking(input)
}

// detectThinkingKeywords checks if the input contains keywords that suggest thinking mode should be enabled
func (m Model) detectThinkingKeywords(input string) bool {
	thinkingKeywords := []string{
		"think", "thinking", "reason", "reasoning", "analyze", "analysis",
		"consider", "evaluate", "examine", "plan", "planning", "strategy",
		"approach", "methodology", "step by step", "walk through", "break down",
		"pros and cons", "trade-offs", "implications", "consequences",
	}

	lowerInput := strings.ToLower(input)
	for _, keyword := range thinkingKeywords {
		if strings.Contains(lowerInput, keyword) {
			return true
		}
	}
	return false
}

func (m Model) startLLMStream() tea.Cmd {
	return m.startLLMStreamWithThinking("")
}

func (m Model) startLLMStreamWithThinking(userInput string) tea.Cmd {
	// Get provider manager from core
	providerMgrInterface := m.core.GetProviderManager()
	providerMgr, ok := providerMgrInterface.(*providers.Manager)
	if !ok {
		return func() tea.Msg {
			return streamMsg{
				content: "Error: Provider manager not available",
				done:    true,
				isError: true,
			}
		}
	}

	// Build conversation history (excluding the empty assistant message)
	var messages []providers.Message
	for i, msg := range m.messages {
		if i == len(m.messages)-1 && msg.Role == "assistant" && msg.Content == "" {
			continue // Skip the empty assistant message we just added
		}

		role := msg.Role
		if role == "user" {
			role = providers.RoleUser
		} else if role == "assistant" {
			role = providers.RoleAssistant
		}

		messages = append(messages, providers.Message{
			Role:    role,
			Content: msg.Content,
		})
	}

	// Determine provider and model
	provider := m.currentProvider
	if provider == "" {
		provider = providerMgr.GetDefaultProvider()
	}

	model := providerMgr.GetDefaultModel(provider)

	// Detect if thinking mode should be enabled
	enableThinking := false
	if userInput != "" {
		enableThinking = m.detectThinkingKeywords(userInput)
	}

	// Create chat request
	request := &providers.ChatRequest{
		Messages: messages,
		Model:    model,
		Stream:   true,
		Thinking: enableThinking,
	}

	// Start streaming
	return func() tea.Msg {
		ctx := context.Background()
		stream, err := providerMgr.ChatStream(ctx, request)
		if err != nil {
			return streamMsg{
				content: fmt.Sprintf("Error: %v", err),
				done:    true,
				isError: true,
			}
		}

		// Store stream reference and start reading
		m.streamChan = stream
		return m.streamCommand(stream)()
	}
}

func (m Model) streamCommand(stream <-chan *providers.StreamChunk) tea.Cmd {
	return func() tea.Msg {
		select {
		case chunk, ok := <-stream:
			if !ok {
				return streamMsg{
					done: true,
				}
			}

			if chunk.Error != nil {
				return streamMsg{
					content: fmt.Sprintf("Stream error: %v", chunk.Error),
					done:    true,
					isError: true,
				}
			}

			return streamMsg{
				content:  chunk.Delta.Content,
				done:     chunk.Done,
				tokens:   chunk.TokensUsed.TotalTokens,
				cost:     chunk.Cost,
				provider: chunk.Provider,
				model:    chunk.Model,
			}

		default:
			// No data available yet
			time.Sleep(time.Millisecond * 50)
			return streamMsg{
				content: "",
				done:    false,
			}
		}
	}
}

func (m Model) waitForStream() tea.Cmd {
	return tea.Tick(time.Millisecond*50, func(t time.Time) tea.Msg {
		return streamMsg{
			content: "",
			done:    false,
		}
	})
}

// Utility methods
func (m Model) calculateTotalCost() float64 {
	total := 0.0
	for _, msg := range m.messages {
		total += msg.Cost
	}
	return total
}

func (m Model) calculateTotalTokens() int {
	total := 0
	for _, msg := range m.messages {
		total += msg.Tokens
	}
	return total
}

// createStyles creates the styling definitions
func createStyles() Styles {
	return Styles{
		App: lipgloss.NewStyle().
			Padding(1).
			Background(bgColor),

		Header: lipgloss.NewStyle().
			Bold(true).
			Foreground(primaryColor).
			Padding(0, 1),

		Footer: lipgloss.NewStyle().
			Padding(1, 0),

		Content: lipgloss.NewStyle().
			Padding(1).
			Border(lipgloss.RoundedBorder()).
			BorderForeground(borderColor),

		Sidebar: lipgloss.NewStyle().
			Padding(1).
			Border(lipgloss.NormalBorder()).
			BorderForeground(borderColor).
			MarginLeft(1),

		UserMsg: lipgloss.NewStyle().
			Bold(true).
			Foreground(primaryColor),

		AssistMsg: lipgloss.NewStyle().
			Bold(true).
			Foreground(secondaryColor),

		SystemMsg: lipgloss.NewStyle().
			Bold(true).
			Foreground(mutedColor),

		ErrorMsg: lipgloss.NewStyle().
			Bold(true).
			Foreground(errorColor),

		StatusBar: lipgloss.NewStyle().
			Background(bgColor).
			Foreground(lipgloss.Color("#FFFFFF")).
			Padding(0, 1).
			Bold(true),

		Provider: lipgloss.NewStyle().
			Foreground(primaryColor).
			Bold(true),

		Cost: lipgloss.NewStyle().
			Foreground(accentColor),

		Thinking: lipgloss.NewStyle().
			Foreground(accentColor).
			Bold(true),

		Searching: lipgloss.NewStyle().
			Foreground(secondaryColor).
			Bold(true),

		Input: lipgloss.NewStyle().
			Border(lipgloss.RoundedBorder()).
			BorderForeground(borderColor).
			Padding(0, 1),

		Prompt: lipgloss.NewStyle().
			Foreground(primaryColor).
			Bold(true),

		Help: lipgloss.NewStyle().
			Padding(1).
			Border(lipgloss.RoundedBorder()).
			BorderForeground(borderColor),

		HelpKey: lipgloss.NewStyle().
			Bold(true).
			Foreground(primaryColor),

		HelpDesc: lipgloss.NewStyle().
			Foreground(mutedColor),
	}
}
