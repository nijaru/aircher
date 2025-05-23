package repl

import (
	"context"
	"fmt"
	"strings"
	"time"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/bubbles/textinput"
	"github.com/charmbracelet/bubbles/viewport"
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
	input    textinput.Model
	viewport viewport.Model
	
	// Application state
	messages     []Message
	width        int
	height       int
	ready        bool
	
	// UI state
	thinking     bool
	searching    bool
	streaming    bool
	showHelp     bool
	showContext  bool
	
	// Current session
	session      *Session
	currentProvider string
	
	// Active streaming
	streamChan <-chan *providers.StreamChunk
	
	// Styling
	styles       Styles
	renderer     *glamour.TermRenderer
	
	// Context
	logger       zerolog.Logger
	core         AircherCore
}

// Styles holds all the styling definitions
type Styles struct {
	// Layout styles
	App        lipgloss.Style
	Header     lipgloss.Style
	Footer     lipgloss.Style
	Content    lipgloss.Style
	Sidebar    lipgloss.Style
	
	// Message styles
	UserMsg    lipgloss.Style
	AssistMsg  lipgloss.Style
	SystemMsg  lipgloss.Style
	ErrorMsg   lipgloss.Style
	
	// Status styles
	StatusBar  lipgloss.Style
	Provider   lipgloss.Style
	Cost       lipgloss.Style
	Thinking   lipgloss.Style
	Searching  lipgloss.Style
	
	// Input styles
	Input      lipgloss.Style
	Prompt     lipgloss.Style
	
	// Help styles
	Help       lipgloss.Style
	HelpKey    lipgloss.Style
	HelpDesc   lipgloss.Style
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
	
	thinkingMsg struct {
		active bool
	}
	
	searchingMsg struct {
		active bool
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
	input := textinput.New()
	input.Placeholder = "Ask anything... (type /help for commands)"
	input.Focus()
	input.CharLimit = 2000
	input.Width = 80
	
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
		input:     input,
		viewport:  vp,
		messages:  make([]Message, 0),
		session:   session,
		styles:    styles,
		renderer:  renderer,
		logger:    r.logger,
		core:      r.core,
		currentProvider: "ollama", // Default to available provider
	}
	
	// Add welcome message
	model.addSystemMessage("Welcome to Aircher! 🏹")
	model.addSystemMessage("Type your question or use /help for commands.")
	
	// Process initial prompt if provided
	if initialPrompt != "" {
		model.addUserMessage(initialPrompt)
		model.addAssistantMessage("Initial prompt processing not yet implemented.")
	}
	
	return model
}

// Init initializes the Bubble Tea model
func (m Model) Init() tea.Cmd {
	return textinput.Blink
}

// Update handles Bubble Tea messages
func (m Model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	var cmds []tea.Cmd
	
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height
		
		// Update component sizes
		m.input.Width = m.width - 4
		m.viewport.Width = m.width - 4
		m.viewport.Height = m.height - 8
		m.ready = true
		
		// Update content
		m.updateViewportContent()
		
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "esc":
			return m, tea.Quit
			
		case "ctrl+h":
			m.showHelp = !m.showHelp
			m.updateViewportContent()
			
		case "ctrl+t":
			m.showContext = !m.showContext
			m.updateViewportContent()
			
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
		
	case thinkingMsg:
		m.thinking = msg.active
		
	case searchingMsg:
		m.searching = msg.active
		
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
	if m.thinking {
		status = append(status, m.styles.Thinking.Render("🤔 Thinking"))
	}
	if m.searching {
		status = append(status, m.styles.Searching.Render("🔍 Searching"))
	}
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
	
	mainContent := m.styles.Content.Width(m.width-2).Height(m.height-6).Render(m.viewport.View())
	
	if m.showContext {
		contextPanel := m.renderContextPanel()
		return lipgloss.JoinHorizontal(
			lipgloss.Top,
			mainContent,
			contextPanel,
		)
	}
	
	return mainContent
}

// renderFooter renders the input area and help hints
func (m Model) renderFooter() string {
	inputArea := m.styles.Input.Width(m.width-2).Render(
		lipgloss.JoinHorizontal(
			lipgloss.Left,
			m.styles.Prompt.Render("> "),
			m.input.View(),
		),
	)
	
	hints := m.styles.Help.Render("Ctrl+H: Help • Ctrl+T: Context • Ctrl+C: Exit")
	
	return m.styles.Footer.Width(m.width).Render(
		lipgloss.JoinVertical(
			lipgloss.Top,
			inputArea,
			hints,
		),
	)
}

// renderHelp renders the help panel
func (m Model) renderHelp() string {
	helpContent := []string{
		m.styles.HelpKey.Render("Available Commands:"),
		"",
		m.styles.HelpKey.Render("/help") + " - " + m.styles.HelpDesc.Render("Show this help"),
		m.styles.HelpKey.Render("/clear") + " - " + m.styles.HelpDesc.Render("Clear conversation"),
		m.styles.HelpKey.Render("/config") + " - " + m.styles.HelpDesc.Render("Settings management"),
		m.styles.HelpKey.Render("/cost") + " - " + m.styles.HelpDesc.Render("Usage and cost statistics"),
		m.styles.HelpKey.Render("/memory") + " - " + m.styles.HelpDesc.Render("Edit AIRCHER.md memory"),
		m.styles.HelpKey.Render("/search <query>") + " - " + m.styles.HelpDesc.Render("Force web search"),
		m.styles.HelpKey.Render("/think") + " - " + m.styles.HelpDesc.Render("Enable thinking mode"),
		m.styles.HelpKey.Render("/mcp") + " - " + m.styles.HelpDesc.Render("MCP server management"),
		m.styles.HelpKey.Render("/tools") + " - " + m.styles.HelpDesc.Render("List available MCP tools"),
		"",
		m.styles.HelpKey.Render("Keyboard Shortcuts:"),
		"",
		m.styles.HelpKey.Render("Ctrl+H") + " - " + m.styles.HelpDesc.Render("Toggle this help"),
		m.styles.HelpKey.Render("Ctrl+T") + " - " + m.styles.HelpDesc.Render("Toggle context panel"),
		m.styles.HelpKey.Render("Ctrl+C") + " - " + m.styles.HelpDesc.Render("Exit Aircher"),
		m.styles.HelpKey.Render("Esc") + " - " + m.styles.HelpDesc.Render("Exit Aircher"),
	}
	
	return m.styles.Help.Width(m.width-4).Height(m.height-6).Render(
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
		m.styles.HelpKey.Render("🔧 Tools Available:"),
		m.styles.HelpDesc.Render("• Filesystem operations"),
		m.styles.HelpDesc.Render("• Git integration"),
		m.styles.HelpDesc.Render("• Database queries"),
		"",
		m.styles.HelpKey.Render("📊 Statistics:"),
		m.styles.HelpDesc.Render(fmt.Sprintf("• Total cost: $%.4f", m.calculateTotalCost())),
		m.styles.HelpDesc.Render(fmt.Sprintf("• Total tokens: %d", m.calculateTotalTokens())),
	}
	
	return m.styles.Sidebar.Width(30).Height(m.height-6).Render(
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
	
	switch cmd {
	case "/help":
		m.showHelp = !m.showHelp
		return func() tea.Msg {
			return statusMsg{"Help toggled"}
		}
		
	case "/clear":
		m.messages = m.messages[:0]
		m.addSystemMessage("Conversation cleared")
		return nil
		
	case "/config":
		return func() tea.Msg {
			return statusMsg{"Configuration not yet implemented"}
		}
		
	case "/cost":
		cost := m.calculateTotalCost()
		tokens := m.calculateTotalTokens()
		return func() tea.Msg {
			return statusMsg{fmt.Sprintf("Total cost: $%.4f, Total tokens: %d", cost, tokens)}
		}
		
	case "/think":
		m.thinking = !m.thinking
		status := "disabled"
		if m.thinking {
			status = "enabled"
		}
		return func() tea.Msg {
			return statusMsg{fmt.Sprintf("Thinking mode %s", status)}
		}
		
	default:
		return func() tea.Msg {
			return errorMsg{fmt.Errorf("unknown command: %s", cmd)}
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
	
	return m.startLLMStream()
}

func (m Model) startLLMStream() tea.Cmd {
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

	// Create chat request
	request := &providers.ChatRequest{
		Messages:    messages,
		Model:       model,
		Provider:    provider,
		Temperature: 0.7,
		Stream:      true,
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