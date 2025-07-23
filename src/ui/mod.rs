use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io::stdout;
use std::rc::Rc;
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::config::ConfigManager;
use crate::providers::{ChatRequest, Message, MessageRole, ProviderManager};
use crate::project::ProjectManager;
use crate::sessions::{SessionManager, Session};
use crate::storage::DatabaseManager;
use crate::intelligence::tui_tools::TuiIntelligenceTools;
use crate::intelligence::tools::IntelligenceTools;
use crate::intelligence::file_monitor;
use crate::semantic_search::SemanticCodeSearch;
use crate::agent::{AgentController, conversation::ProgrammingLanguage};

pub mod chat;
pub mod components;
pub mod selection;
pub mod enhanced_selection;
pub mod settings;
pub mod help;
pub mod autocomplete;
pub mod slash_commands;
pub mod typeahead;
pub mod model_selection;

use selection::SelectionModal;
use settings::SettingsModal;
use help::HelpModal;
use autocomplete::AutocompleteEngine;
use model_selection::ModelSelectionOverlay;
use slash_commands::{parse_slash_command, format_help};

pub struct TuiManager {
    config: ConfigManager,
    provider_name: String,
    model: String,
    messages: Vec<Message>,
    input: String,
    cursor_position: usize,
    scroll_offset: u16,
    session_cost: f64,
    session_tokens: u32,
    // Session management
    _project_manager: ProjectManager,
    session_manager: SessionManager,
    current_session: Option<Session>,
    intelligence_tools: TuiIntelligenceTools,
    file_monitor: Option<file_monitor::FileMonitor>,
    // Semantic search
    semantic_search: SemanticCodeSearch,
    // AI Agent
    agent_controller: Option<AgentController>,
    // Modals
    selection_modal: SelectionModal,
    settings_modal: SettingsModal,
    help_modal: HelpModal,
    model_selection_overlay: ModelSelectionOverlay,
    // Autocomplete
    autocomplete: AutocompleteEngine,
    // Authentication state
    providers: Option<Rc<ProviderManager>>,
    auth_required: bool,
    show_auth_setup: bool,
    // State
    budget_warning_shown: bool,
    cost_warnings: Vec<String>,
    should_quit: bool,
}

impl TuiManager {
    /// Create TUI manager with authentication state handling
    pub async fn new_with_auth_state(
        config: &ConfigManager, 
        providers: Option<Rc<ProviderManager>>,
        provider_name: String,
        model: String
    ) -> Result<Self> {
        let auth_required = providers.is_none();
        
        // Initialize project manager
        let mut project_manager = ProjectManager::new()?;
        
        // Initialize project if needed
        if !project_manager.is_project_initialized() {
            project_manager.initialize_project()?;
            info!("Initialized new .aircher project");
        }
        
        // Initialize session manager
        let database_manager = DatabaseManager::new(config).await?;
        let session_manager = SessionManager::new(&database_manager).await?;
        
        // Initialize intelligence tools
        let mut intelligence_tools = TuiIntelligenceTools::new()?;
        intelligence_tools.initialize_project()?;
        
        // Initialize semantic search (works without API keys)
        let mut semantic_search = crate::semantic_search::SemanticCodeSearch::new();
        semantic_search.ensure_model_available().await?;
        
        // Start background file monitoring with semantic search integration
        let semantic_search_monitor = crate::semantic_search::SemanticCodeSearch::new();
        let file_monitor = if !auth_required {
            Some(file_monitor::start_background_monitoring(
                project_manager.clone(),
                intelligence_tools.clone(),
                semantic_search_monitor,
            ).await?)
        } else {
            // In auth mode, we can still monitor files but without provider integration
            Some(file_monitor::start_background_monitoring(
                project_manager.clone(),
                intelligence_tools.clone(),
                semantic_search_monitor,
            ).await?)
        };
        
        // Create or continue session (if providers available)
        let current_session = if let Some(ref _providers) = providers {
            let project_info = project_manager.get_project_info();
            let session_title = format!("{} - TUI Session", project_info.name);
            
            Some(session_manager.create_session(
                session_title,
                provider_name.clone(),
                model.clone(),
                Some("TUI session for project".to_string()),
                vec!["tui".to_string()],
            ).await?)
        } else {
            None
        };
        
        info!("TUI Manager initialized with auth_required: {}", auth_required);
        
        Ok(Self {
            config: config.clone(),
            provider_name,
            model,
            messages: Vec::new(),
            input: String::new(),
            cursor_position: 0,
            scroll_offset: 0,
            session_cost: 0.0,
            session_tokens: 0,
            // Session management
            _project_manager: project_manager,
            session_manager,
            current_session,
            intelligence_tools,
            file_monitor,
            // Semantic search
            semantic_search,
            // AI Agent
            agent_controller: None, // Will be initialized when needed
            // Modals
            selection_modal: if let Some(ref providers) = providers {
                SelectionModal::new(providers.as_ref(), config)
            } else {
                // In demo mode, create a selection modal from config only
                SelectionModal::from_config(config)
            },
            settings_modal: SettingsModal::new(config),
            help_modal: HelpModal::new(),
            model_selection_overlay: if let Some(ref providers) = providers {
                ModelSelectionOverlay::with_providers(config, providers.as_ref())
            } else {
                ModelSelectionOverlay::new(config)
            },
            // Autocomplete
            autocomplete: AutocompleteEngine::new(),
            // Authentication state
            providers,
            auth_required,
            show_auth_setup: false, // Always start with normal interface
            // State
            budget_warning_shown: false,
            cost_warnings: Vec::new(),
            should_quit: false,
        })
    }

    pub async fn new(config: &ConfigManager, providers: &ProviderManager) -> Result<Self> {
        // Initialize project manager
        let mut project_manager = ProjectManager::new()?;
        
        // Initialize project if needed
        if !project_manager.is_project_initialized() {
            project_manager.initialize_project()?;
            info!("Initialized new .aircher project");
        }
        
        // Initialize session manager
        let database_manager = DatabaseManager::new(config).await?;
        let session_manager = SessionManager::new(&database_manager).await?;
        
        // Initialize intelligence tools
        let mut intelligence_tools = TuiIntelligenceTools::new()?;
        intelligence_tools.initialize_project()?;
        
        // Initialize semantic search for background monitoring
        let mut semantic_search_monitor = crate::semantic_search::SemanticCodeSearch::new();
        semantic_search_monitor.ensure_model_available().await?;
        
        // Initialize semantic search for TUI
        let mut semantic_search = crate::semantic_search::SemanticCodeSearch::new();
        semantic_search.ensure_model_available().await?;
        
        // Start background file monitoring with semantic search integration
        let file_monitor = file_monitor::start_background_monitoring(
            project_manager.clone(),
            intelligence_tools.clone(),
            semantic_search_monitor,
        ).await?;
        
        // Create or continue session
        let project_info = project_manager.get_project_info();
        let session_title = format!("{} - TUI Session", project_info.name);
        
        let current_session = session_manager.create_session(
            session_title,
            config.global.default_provider.clone(),
            config.global.default_model.clone(),
            Some("TUI session for project".to_string()),
            vec!["tui".to_string()],
        ).await?;
        
        info!("Created session: {}", current_session.id);
        
        // Load session messages
        let session_messages = session_manager.load_session_messages(&current_session.id).await?;
        
        // Convert session messages to provider messages
        let mut messages = Vec::new();
        for session_msg in session_messages {
            let provider_role = match session_msg.role {
                crate::sessions::MessageRole::User => MessageRole::User,
                crate::sessions::MessageRole::Assistant => MessageRole::Assistant,
                crate::sessions::MessageRole::System => MessageRole::System,
                crate::sessions::MessageRole::Tool => MessageRole::Tool,
            };
            
            let mut provider_msg = Message::new(provider_role, session_msg.content);
            provider_msg.tokens_used = session_msg.tokens_used;
            provider_msg.cost = session_msg.cost;
            
            messages.push(provider_msg);
        }
        
        Ok(Self {
            config: config.clone(),
            provider_name: config.global.default_provider.clone(),
            model: config.global.default_model.clone(),
            messages,
            input: String::new(),
            cursor_position: 0,
            scroll_offset: 0,
            session_cost: current_session.total_cost,
            session_tokens: current_session.total_tokens,
            // Session management
            _project_manager: project_manager,
            session_manager,
            current_session: Some(current_session),
            intelligence_tools,
            file_monitor: Some(file_monitor),
            // Semantic search
            semantic_search,
            // AI Agent
            agent_controller: None, // Will be initialized when needed
            // Initialize modals
            selection_modal: SelectionModal::new(providers, config),
            settings_modal: SettingsModal::new(config),
            help_modal: HelpModal::new(),
            model_selection_overlay: ModelSelectionOverlay::with_providers(config, providers),
            // Initialize autocomplete
            autocomplete: AutocompleteEngine::new(),
            // Authentication state (providers available in this constructor)
            providers: Some(Rc::new(ProviderManager::new(config).await?)),
            auth_required: false,
            show_auth_setup: false,
            // Initialize state
            budget_warning_shown: false,
            cost_warnings: Vec::new(),
            should_quit: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting TUI interface (auth_required: {})", self.auth_required);

        // Setup terminal
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        // Create channel for async communication
        let (_tx, mut rx) = mpsc::channel::<String>(10);

        // Main TUI loop
        loop {
            // Check if we should exit
            if self.should_quit {
                break;
            }
            
            // Draw the UI
            terminal.draw(|f| self.draw(f))?;

            // Handle events with timeout
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        // Check if any modal is handling the event
                        if self.handle_modal_events(key)? {
                            continue;
                        }

                        // Handle auth setup events if in auth mode
                        if self.show_auth_setup {
                            if self.handle_auth_setup_events(key).await? {
                                continue;
                            }
                        }

                        // Handle main interface events
                        match key.code {
                            KeyCode::Char('c')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                break;
                            }
                            KeyCode::F(1) => {
                                self.help_modal.toggle();
                            }
                            KeyCode::F(2) => {
                                self.settings_modal.toggle();
                            }
                            KeyCode::Tab => {
                                self.selection_modal.toggle();
                            }
                            KeyCode::Enter => {
                                // Check if autocomplete is visible and accept suggestion
                                if self.autocomplete.is_visible() {
                                    if let Some(completion) = self.autocomplete.accept_suggestion() {
                                        self.input = completion;
                                        self.cursor_position = self.input.len();
                                    }
                                } else if !self.input.is_empty() {
                                    let message = self.input.clone();
                                    self.input.clear();
                                    self.cursor_position = 0;
                                    
                                    debug!("Processing user message: '{}'", message);

                                    // Check for slash commands
                                    if let Some((command, args)) = parse_slash_command(&message) {
                                        match command {
                                            "/model" => {
                                                self.model_selection_overlay.show();
                                            }
                                            "/search" => {
                                                if !args.is_empty() {
                                                    // Add user message showing the search command
                                                    self.messages.push(Message::user(message.clone()));
                                                    
                                                    // Perform semantic search
                                                    if let Err(e) = self.handle_search_command(args).await {
                                                        self.messages.push(Message::new(
                                                            MessageRole::System,
                                                            format!("Search error: {}", e),
                                                        ));
                                                    }
                                                } else {
                                                    self.messages.push(Message::new(
                                                        MessageRole::System,
                                                        "Usage: /search <query>".to_string(),
                                                    ));
                                                }
                                            }
                                            "/help" => {
                                                // Add each help line as a separate message for proper display
                                                for line in format_help() {
                                                    self.messages.push(Message::new(
                                                        MessageRole::System,
                                                        line,
                                                    ));
                                                }
                                            }
                                            "/clear" => {
                                                self.messages.clear();
                                                self.messages.push(Message::new(
                                                    MessageRole::System,
                                                    "Conversation cleared. Context reset.".to_string(),
                                                ));
                                            }
                                            "/config" => {
                                                self.settings_modal.toggle();
                                            }
                                            "/sessions" => {
                                                // TODO: Implement session browser
                                                self.messages.push(Message::new(
                                                    MessageRole::System,
                                                    "Session browser coming soon!".to_string(),
                                                ));
                                            }
                                            "/compact" => {
                                                // TODO: Implement conversation compaction
                                                self.messages.push(Message::new(
                                                    MessageRole::System,
                                                    "Conversation compaction coming soon!".to_string(),
                                                ));
                                            }
                                            "/quit" => {
                                                self.should_quit = true;
                                            }
                                            _ => {
                                                self.messages.push(Message::new(
                                                    MessageRole::System,
                                                    format!("Unknown command: {}. Type /help for available commands.", command),
                                                ));
                                            }
                                        }
                                    } else if message.starts_with("/") {
                                        // Unknown slash command
                                        self.messages.push(Message::new(
                                            MessageRole::System,
                                            "Unknown command. Type /help for available commands.".to_string(),
                                        ));
                                    } else {
                                        // Handle regular message based on auth state
                                        if self.providers.is_some() {
                                            // Add user message first
                                            self.messages.push(Message::user(message.clone()));
                                            
                                            // Send to AI (methods will handle the borrowing internally)
                                            if let Err(e) = self.handle_ai_message(message).await {
                                                self.messages.push(Message::new(
                                                    MessageRole::System,
                                                    format!("Error: {}", e),
                                                ));
                                            }
                                        } else {
                                            // Demo mode - show that AI features require API key
                                            debug!("No providers configured, showing demo mode message");
                                            self.messages.push(Message::user(message.clone()));
                                            self.messages.push(Message::new(
                                                MessageRole::System,
                                                "⚠️ No AI provider configured. Type /model to select one or /config to set up API keys.".to_string(),
                                            ));
                                        }
                                    }
                                }
                            }
                            KeyCode::Char(' ') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Ctrl+Space to manually trigger autocomplete
                                let _ = self.autocomplete.generate_suggestions(&self.input, self.cursor_position);
                                if self.autocomplete.has_suggestions() {
                                    self.autocomplete.show();
                                }
                            }
                            KeyCode::Char(c) => {
                                // Insert character at cursor position
                                self.input.insert(self.cursor_position, c);
                                self.cursor_position += 1;
                                
                                // Generate autocomplete suggestions
                                let _ = self.autocomplete.generate_suggestions(&self.input, self.cursor_position);
                            }
                            KeyCode::Backspace => {
                                if self.cursor_position > 0 {
                                    self.input.remove(self.cursor_position - 1);
                                    self.cursor_position -= 1;
                                    
                                    // Update autocomplete suggestions
                                    if self.input.is_empty() {
                                        self.autocomplete.hide();
                                    } else {
                                        let _ = self.autocomplete.generate_suggestions(&self.input, self.cursor_position);
                                    }
                                }
                            }
                            KeyCode::Up => {
                                if self.autocomplete.is_visible() {
                                    self.autocomplete.move_selection_up();
                                } else if self.scroll_offset > 0 {
                                    self.scroll_offset -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if self.autocomplete.is_visible() {
                                    self.autocomplete.move_selection_down();
                                } else {
                                    // Limit scroll to available messages
                                    let visible_height = 20; // Approximate, could be calculated more precisely
                                    let max_scroll = if self.messages.len() > visible_height {
                                        (self.messages.len() - visible_height) as u16
                                    } else {
                                        0
                                    };
                                    if self.scroll_offset < max_scroll {
                                        self.scroll_offset += 1;
                                    }
                                }
                            }
                            KeyCode::Left => {
                                if self.cursor_position > 0 {
                                    self.cursor_position -= 1;
                                    self.autocomplete.hide();
                                }
                            }
                            KeyCode::Right => {
                                if self.cursor_position < self.input.len() {
                                    self.cursor_position += 1;
                                    self.autocomplete.hide();
                                }
                            }
                            KeyCode::Esc => {
                                if self.autocomplete.is_visible() {
                                    self.autocomplete.hide();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Handle async messages
            if let Ok(msg) = rx.try_recv() {
                debug!("Received async message: {:?}", msg);
            }
        }

        // Cleanup
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        
        // Stop file monitoring
        if let Some(monitor) = &self.file_monitor {
            monitor.stop().await;
        }

        info!("TUI interface closed");
        Ok(())
    }

    fn draw(&self, f: &mut Frame) {
        // Show auth setup screen if needed
        if self.show_auth_setup {
            self.draw_auth_setup_screen(f);
            return;
        }

        // Let terminal handle background colors

        // Minimal margins like Claude Code
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(if self.messages.is_empty() { 5 } else { 0 }), // Welcome box
                Constraint::Length(if self.messages.is_empty() { 1 } else { 0 }), // Tip line
                Constraint::Min(1),    // Chat area
                Constraint::Length(4), // Input box area
                Constraint::Length(1), // Status line
            ])
            .split(f.area());

        // Show welcome box and tip only when chat is empty
        if self.messages.is_empty() {
            self.draw_welcome_box(f, chunks[0]);
            self.draw_tip_line(f, chunks[1]);
        }

        // Chat area
        let chat_area = if self.messages.is_empty() { chunks[2] } else { chunks[0] };
        self.draw_chat_area(f, chat_area);

        // Input box area  
        let input_area = if self.messages.is_empty() { chunks[3] } else { chunks[1] };
        self.draw_input_box(f, input_area);

        // Status line
        let status_area = if self.messages.is_empty() { chunks[4] } else { chunks[2] };
        self.draw_status_bar(f, status_area);

        // Render autocomplete suggestions
        if self.autocomplete.is_visible() {
            self.autocomplete.render(f, input_area);
        }

        // Render modals (on top of everything)
        self.selection_modal.render(f, f.area());
        self.settings_modal.render(f, f.area());
        self.help_modal.render(f, f.area());
        self.model_selection_overlay.render(f, f.area());
    }

    fn draw_welcome_box(&self, f: &mut Frame, area: Rect) {
        // Create a centered welcome box like Claude Code
        let welcome_width = 55;
        let welcome_height = 5;
        let x = (area.width.saturating_sub(welcome_width)) / 2;
        let y = 0;
        
        let welcome_area = Rect::new(
            area.x + x,
            area.y + y,
            welcome_width.min(area.width),
            welcome_height.min(area.height)
        );

        let welcome_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(139, 92, 246))); // Purple border, no background override

        let welcome_content = vec![
            Line::from(vec![
                Span::styled("🏹 Welcome to Aircher!", 
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)) // White and bold like Claude
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  /help", Style::default().fg(Color::Rgb(139, 92, 246))), // Purple highlights
                Span::styled(" for help, ", Style::default().fg(Color::Rgb(163, 136, 186))), // Low-sat purple like Claude's beige
                Span::styled("/model", Style::default().fg(Color::Rgb(139, 92, 246))),
                Span::styled(" to select AI model", Style::default().fg(Color::Rgb(163, 136, 186))),
            ]),
        ];

        let welcome_paragraph = Paragraph::new(welcome_content)
            .block(welcome_block)
            .alignment(Alignment::Center);
        
        f.render_widget(welcome_paragraph, welcome_area);
    }
    
    fn draw_tip_line(&self, f: &mut Frame, area: Rect) {
        // Tip line like Claude Code's "※ Tip: Hit shift+tab..."
        let tip_text = Line::from(vec![
            Span::styled(" ※ Tip: Type ", Style::default().fg(Color::Rgb(107, 114, 128))), // Gray like Claude's tips
            Span::styled("/", Style::default().fg(Color::Rgb(139, 92, 246))),
            Span::styled(" to see available commands", Style::default().fg(Color::Rgb(107, 114, 128))),
        ]);
        
        let tip_paragraph = Paragraph::new(tip_text)
            .alignment(Alignment::Left);
        
        f.render_widget(tip_paragraph, area);
    }

    fn draw_chat_area(&self, f: &mut Frame, area: Rect) {
        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .map(|msg| {
                let prefix = match msg.role {
                    MessageRole::User => "You: ",
                    MessageRole::Assistant => "Aircher: ",
                    MessageRole::System => "",
                    MessageRole::Tool => "Tool: ",
                };

                let style = match msg.role {
                    MessageRole::User => Style::default().fg(Color::Rgb(167, 139, 250)), // Light purple
                    MessageRole::Assistant => Style::default().fg(Color::Rgb(229, 231, 235)), // Light gray
                    MessageRole::System => Style::default().fg(Color::Rgb(156, 163, 175)), // Gray
                    MessageRole::Tool => Style::default().fg(Color::Rgb(251, 191, 36)), // Amber
                };

                ListItem::new(Line::from(vec![
                    Span::styled(prefix, style.add_modifier(Modifier::BOLD)),
                    Span::raw(&msg.content),
                ]))
            })
            .collect();

        // Clean list without borders - like Claude Code's clean chat
        let messages_list = List::new(messages)
            .block(Block::default())
            .style(Style::default());

        // Implement scrolling
        let mut state = ListState::default();
        if !self.messages.is_empty() {
            // Calculate which message should be at the top based on scroll_offset
            let visible_height = area.height as usize;
            let total_messages = self.messages.len();
            
            if total_messages > visible_height {
                // Auto-scroll to bottom unless user has scrolled up
                let max_scroll = total_messages.saturating_sub(visible_height);
                let actual_scroll = (max_scroll as u16).saturating_sub(self.scroll_offset);
                state.select(Some(actual_scroll as usize));
            } else {
                state.select(Some(0));
            }
        }

        f.render_stateful_widget(messages_list, area, &mut state);
    }

    fn draw_input_box(&self, f: &mut Frame, area: Rect) {
        // Split area for input and bottom info line
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Input box (expandable)
                Constraint::Length(1), // Bottom info line
            ])
            .split(area);

        // Input box with subtle border and placeholder
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(163, 136, 186))); // Low-sat purple like Claude's beige border

        let input_inner = input_block.inner(chunks[0]);
        f.render_widget(input_block, chunks[0]);

        // Add padding inside the input box like Claude Code
        let padded_area = Rect {
            x: input_inner.x + 1, // Left padding
            y: input_inner.y,
            width: input_inner.width.saturating_sub(2), // Account for left and right padding
            height: input_inner.height,
        };

        // Handle multi-line input display or show placeholder
        if self.input.is_empty() && !self.autocomplete.is_visible() {
            // Show placeholder text in input box with padding
            let placeholder = Paragraph::new("Type your message and press Enter")
                .style(Style::default().fg(Color::Rgb(107, 114, 128))) // Gray placeholder
                .alignment(Alignment::Left);
            f.render_widget(placeholder, padded_area);
        } else {
            let lines: Vec<&str> = self.input.split('\n').collect();
            let _line_count = lines.len();
            let visible_lines = input_inner.height as usize;
            
            // Calculate which lines to show based on cursor position
            let cursor_line = self.input[..self.cursor_position].matches('\n').count();
            let start_line = if cursor_line >= visible_lines {
                cursor_line - visible_lines + 1
            } else {
                0
            };
            
            let visible_text = lines.iter()
                .skip(start_line)
                .take(visible_lines)
                .cloned()
                .collect::<Vec<&str>>()
                .join("\n");

            // Input text using terminal's default text color with padding
            let input_text = Paragraph::new(visible_text)
                .style(Style::default().fg(Color::White)) // Use terminal white
                .wrap(ratatui::widgets::Wrap { trim: false });
            f.render_widget(input_text, padded_area);

            // Calculate cursor position for multi-line
            let cursor_line_in_view = cursor_line.saturating_sub(start_line);
            let cursor_col = if cursor_line < lines.len() {
                let line_start = lines[..cursor_line].iter().map(|l| l.len() + 1).sum::<usize>();
                self.cursor_position.saturating_sub(line_start)
            } else {
                0
            };

            // Set cursor position with padding
            f.set_cursor_position((
                padded_area.x + cursor_col as u16,
                padded_area.y + cursor_line_in_view as u16
            ));
        }

        // Bottom info line with shortcuts on left and model info on right
        self.draw_input_info_line(f, chunks[1]);
    }
    
    fn draw_input_info_line(&self, f: &mut Frame, area: Rect) {
        // Split the line: left side for shortcuts, right side for model info
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Left side
                Constraint::Percentage(40), // Right side
            ])
            .split(area);

        // Left side: shortcuts and help
        let shortcuts = if self.autocomplete.is_visible() {
            "↑↓ navigate • Enter accept • Esc cancel"
        } else {
            "? for shortcuts | Ctrl+C to quit"
        };
        
        // Add left padding to shortcuts like Claude Code
        let padded_shortcuts_area = Rect {
            x: chunks[0].x + 1, // Left padding
            y: chunks[0].y,
            width: chunks[0].width.saturating_sub(1),
            height: chunks[0].height,
        };
        
        let shortcuts_text = Paragraph::new(shortcuts)
            .style(Style::default().fg(Color::Rgb(107, 114, 128))) // Comment gray
            .alignment(Alignment::Left);
        f.render_widget(shortcuts_text, padded_shortcuts_area);

        // Right side: model info and context usage
        let mut right_parts = vec![];
        
        // Model info
        right_parts.push(format!("{} ({})", self.model, self.provider_name));
        
        // Context usage percentage (if we have session data)
        if self.session_tokens > 0 {
            // Assume a reasonable context window (adjust based on actual model)
            let estimated_context_window = 100000; // 100k tokens as example
            let usage_percent = (self.session_tokens as f32 / estimated_context_window as f32 * 100.0).min(100.0);
            right_parts.push(format!("{}% context", usage_percent as u32));
        }
        
        // Cost if significant
        if self.session_cost > 0.001 {
            right_parts.push(format!("${:.3}", self.session_cost));
        }
        
        let right_text = right_parts.join(" • ");
        let right_paragraph = Paragraph::new(right_text)
            .style(Style::default().fg(Color::Rgb(107, 114, 128))) // Comment gray
            .alignment(Alignment::Right);
        f.render_widget(right_paragraph, chunks[1]);
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect) {
        // Simple status like Claude Code's bottom status (often empty unless there's an alert)
        let mut status_parts = vec![];
        
        // Budget warning if applicable
        if let Some(limit) = self.config.global.budget_limit {
            if self.session_cost > limit * 0.8 {
                status_parts.push("⚠️ Approaching budget limit".to_string());
            }
        }
        
        // Only show status if there's something important
        if !status_parts.is_empty() {
            let status_text = status_parts.join(" | ");
            let status = Paragraph::new(status_text)
                .style(Style::default().fg(Color::Rgb(107, 114, 128))) // Comment gray
                .alignment(Alignment::Left);
            
            f.render_widget(status, area);
        }
    }

    fn handle_modal_events(&mut self, key: ratatui::crossterm::event::KeyEvent) -> Result<bool> {
        // Model selection overlay
        if self.model_selection_overlay.is_visible() {
            match key.code {
                KeyCode::Esc => {
                    self.model_selection_overlay.hide();
                    return Ok(true);
                }
                KeyCode::Tab => {
                    self.model_selection_overlay.switch_mode();
                    return Ok(true);
                }
                KeyCode::Enter => {
                    if let Some((provider, model)) = self.model_selection_overlay.get_selected() {
                        self.provider_name = provider;
                        self.model = model;
                        self.model_selection_overlay.hide();
                    }
                    return Ok(true);
                }
                KeyCode::Up => {
                    self.model_selection_overlay.move_selection_up();
                    return Ok(true);
                }
                KeyCode::Down => {
                    self.model_selection_overlay.move_selection_down();
                    return Ok(true);
                }
                KeyCode::Left => {
                    self.model_selection_overlay.move_cursor_left();
                    return Ok(true);
                }
                KeyCode::Right => {
                    self.model_selection_overlay.move_cursor_right();
                    return Ok(true);
                }
                KeyCode::Backspace => {
                    self.model_selection_overlay.handle_backspace();
                    return Ok(true);
                }
                KeyCode::Char(c) => {
                    self.model_selection_overlay.handle_char(c);
                    return Ok(true);
                }
                _ => return Ok(true), // Consume all other events
            }
        }

        // Help modal
        if self.help_modal.is_visible() {
            match key.code {
                KeyCode::Esc => {
                    self.help_modal.hide();
                    return Ok(true);
                }
                KeyCode::Up => {
                    self.help_modal.scroll_up();
                    return Ok(true);
                }
                KeyCode::Down => {
                    self.help_modal.scroll_down();
                    return Ok(true);
                }
                _ => return Ok(true), // Consume all other events
            }
        }

        // Settings modal
        if self.settings_modal.is_visible() {
            match key.code {
                KeyCode::Esc => {
                    if self.settings_modal.is_editing() {
                        self.settings_modal.cancel_editing();
                    } else {
                        self.settings_modal.hide();
                    }
                    return Ok(true);
                }
                KeyCode::Enter => {
                    if self.settings_modal.is_editing() {
                        self.settings_modal.finish_editing();
                    } else {
                        self.settings_modal.start_editing();
                    }
                    return Ok(true);
                }
                KeyCode::Up => {
                    if !self.settings_modal.is_editing() {
                        self.settings_modal.move_up();
                    }
                    return Ok(true);
                }
                KeyCode::Down => {
                    if !self.settings_modal.is_editing() {
                        self.settings_modal.move_down();
                    }
                    return Ok(true);
                }
                KeyCode::Left => {
                    if !self.settings_modal.is_editing() {
                        self.settings_modal.move_left();
                    }
                    return Ok(true);
                }
                KeyCode::Right => {
                    if !self.settings_modal.is_editing() {
                        self.settings_modal.move_right();
                    }
                    return Ok(true);
                }
                KeyCode::Char(c) => {
                    if self.settings_modal.is_editing() {
                        self.settings_modal.add_char(c);
                    } else if c == 's' || c == 'S' {
                        // Save configuration
                        self.config = self.settings_modal.get_config().clone();
                        self.messages.push(Message::new(
                            MessageRole::System,
                            "Configuration saved".to_string(),
                        ));
                    }
                    return Ok(true);
                }
                KeyCode::Backspace => {
                    if self.settings_modal.is_editing() {
                        self.settings_modal.remove_char();
                    }
                    return Ok(true);
                }
                _ => return Ok(true),
            }
        }

        // Selection modal
        if self.selection_modal.is_visible() {
            match key.code {
                KeyCode::Esc => {
                    self.selection_modal.hide();
                    return Ok(true);
                }
                KeyCode::Enter => {
                    if let Some(provider) = self.selection_modal.get_selected_provider() {
                        self.provider_name = provider.to_string();
                    }
                    if let Some(model) = self.selection_modal.get_selected_model() {
                        self.model = model.to_string();
                    }
                    self.selection_modal.hide();
                    return Ok(true);
                }
                KeyCode::Tab => {
                    // Tab to confirm selection
                    if let Some(provider) = self.selection_modal.get_selected_provider() {
                        self.provider_name = provider.to_string();
                    }
                    if let Some(model) = self.selection_modal.get_selected_model() {
                        self.model = model.to_string();
                    }
                    self.selection_modal.hide();
                    return Ok(true);
                }
                KeyCode::Up => {
                    self.selection_modal.move_up();
                    return Ok(true);
                }
                KeyCode::Down => {
                    self.selection_modal.move_down();
                    return Ok(true);
                }
                KeyCode::Left => {
                    self.selection_modal.move_left();
                    return Ok(true);
                }
                KeyCode::Right => {
                    self.selection_modal.move_right();
                    return Ok(true);
                }
                _ => return Ok(true),
            }
        }

        Ok(false)
    }

    async fn check_budget_limits(&mut self, providers: &ProviderManager) -> Result<bool> {
        // Check if we have a budget limit
        if let Some(limit) = self.config.global.budget_limit {
            // Get estimated cost for current provider/model
            if let Some(provider) = providers.get_provider_or_host(&self.provider_name) {
                // Estimate tokens for current input (rough estimate)
                let estimated_tokens = self.input.len() as u32 / 3; // Rough estimate
                
                if let Some(cost) = provider.calculate_cost(estimated_tokens, estimated_tokens) {
                    let total_cost = self.session_cost + cost;
                    
                    if total_cost > limit {
                        self.messages.push(Message::new(
                            MessageRole::System,
                            format!("🚫 Budget limit exceeded! Cost would be ${:.4}, limit is ${:.2}", 
                                total_cost, limit),
                        ));
                        return Ok(false);
                    }
                    
                    if total_cost > limit * 0.9 && !self.budget_warning_shown {
                        self.messages.push(Message::new(
                            MessageRole::System,
                            format!("⚠️  Warning: Approaching budget limit (${:.4}/${:.2})", 
                                total_cost, limit),
                        ));
                        self.budget_warning_shown = true;
                    }
                }
            }
        }
        
        Ok(true)
    }

    async fn send_message(&mut self, message: String, providers: &ProviderManager) -> Result<()> {
        // Ensure agent controller is initialized
        self.ensure_agent_initialized(providers).await?;
        
        // Try to use agent controller for enhanced functionality
        if let Some(ref mut agent) = self.agent_controller {
            // Get provider for agent
            let provider = providers
                .get_provider_or_host(&self.provider_name)
                .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", self.provider_name))?;
            
            info!("Using agent controller for enhanced AI assistance");
            
            // Process message through agent
            match agent.process_message(&message, provider).await {
                Ok(response) => {
                    // Add user message to local display
                    let user_msg = Message::new(MessageRole::User, message.clone());
                    self.messages.push(user_msg);
                    
                    // Add agent response to local display
                    let assistant_msg = Message::new(MessageRole::Assistant, response.clone());
                    self.messages.push(assistant_msg);
                    
                    // Update session if available
                    if let Some(ref session) = self.current_session {
                        // Convert providers::Message to sessions::Message for storage
                        let session_message = crate::sessions::Message {
                            id: uuid::Uuid::new_v4().to_string(),
                            role: crate::sessions::MessageRole::Assistant,
                            content: response,
                            timestamp: chrono::Utc::now(),
                            tokens_used: None,
                            cost: None,
                        };
                        self.session_manager.add_message(&session.id.to_string(), &session_message).await?;
                    }
                    
                    Ok(())
                }
                Err(e) => {
                    info!("Agent processing failed, falling back to direct provider: {}", e);
                    self.send_message_fallback(message, providers).await
                }
            }
        } else {
            // Fallback to direct provider if agent initialization failed
            self.send_message_fallback(message, providers).await
        }
    }

    /// Fallback message sending using direct provider (current implementation)
    async fn send_message_fallback(&mut self, message: String, providers: &ProviderManager) -> Result<()> {
        // Add user message to local messages
        let user_msg = Message::new(MessageRole::User, message.clone());
        self.messages.push(user_msg);

        // Get provider
        let provider = providers
            .get_provider_or_host(&self.provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", self.provider_name))?;

        // Get intelligence context for the user's message
        let context = self.intelligence_tools.get_development_context(&message).await;
        
        // Create enhanced system prompt with context
        let system_prompt = self.create_enhanced_system_prompt(&context).await?;
        
        // Create chat request with enhanced context
        let mut enhanced_messages = vec![Message::system(system_prompt)];
        enhanced_messages.extend(self.messages.clone());
        
        let request = ChatRequest::new(enhanced_messages, self.model.clone());

        // Send request
        match provider.chat(&request).await {
            Ok(response) => {
                // Create assistant message
                let assistant_msg = Message::new(MessageRole::Assistant, response.content);
                
                // Add assistant response to local messages
                self.messages.push(assistant_msg.clone());

                // Update session stats
                self.session_tokens += response.tokens_used;
                if let Some(cost) = response.cost {
                    self.session_cost += cost;
                }
                
                // Persist messages to session storage
                if let Some(session) = &self.current_session {
                    // Save user message
                    let user_session_msg = crate::sessions::Message {
                        id: uuid::Uuid::new_v4().to_string(),
                        role: crate::sessions::MessageRole::User,
                        content: message,
                        timestamp: chrono::Utc::now(),
                        tokens_used: None,
                        cost: None,
                    };
                    
                    self.session_manager.add_message(&session.id, &user_session_msg).await?;
                    
                    // Save assistant message
                    let assistant_session_msg = crate::sessions::Message {
                        id: uuid::Uuid::new_v4().to_string(),
                        role: crate::sessions::MessageRole::Assistant,
                        content: assistant_msg.content,
                        timestamp: chrono::Utc::now(),
                        tokens_used: Some(response.tokens_used),
                        cost: response.cost,
                    };
                    
                    self.session_manager.add_message(&session.id, &assistant_session_msg).await?;
                    
                    // Update session stats in database
                    self.session_manager.save_session(&Session {
                        id: session.id.clone(),
                        title: session.title.clone(),
                        created_at: session.created_at,
                        updated_at: chrono::Utc::now(),
                        provider: self.provider_name.clone(),
                        model: self.model.clone(),
                        total_cost: self.session_cost,
                        total_tokens: self.session_tokens,
                        message_count: self.messages.len() as u32,
                        tags: session.tags.clone(),
                        is_archived: false,
                        description: session.description.clone(),
                    }).await?;
                }
            }
            Err(e) => {
                return Err(e);
            }
        }

        Ok(())
    }
    
    /// Initialize agent controller for coding assistance  
    async fn ensure_agent_initialized(&mut self, _providers: &ProviderManager) -> Result<()> {
        if self.agent_controller.is_none() {
            // Create database manager for intelligence engine
            let database_manager = DatabaseManager::new(&self.config).await?;
            
            // Create intelligence engine
            let intelligence = crate::intelligence::IntelligenceEngine::new(&self.config, &database_manager).await?;
            
            // Detect project language and create project context
            let root_path = std::env::current_dir()?;
            let language = detect_language(&root_path);
            let project_context = crate::agent::conversation::ProjectContext {
                root_path,
                language,
                framework: None,
                recent_changes: Vec::new(),
            };
            
            // Initialize agent controller
            let mut controller = AgentController::new(intelligence, project_context)?;
            
            // Register default tools
            controller.register_tool(Box::new(crate::agent::tools::file_ops::ReadFileTool::new()));
            controller.register_tool(Box::new(crate::agent::tools::file_ops::WriteFileTool::new()));
            controller.register_tool(Box::new(crate::agent::tools::file_ops::ListFilesTool::new()));
            controller.register_tool(Box::new(crate::agent::tools::code_analysis::SearchCodeTool::new()));
            controller.register_tool(Box::new(crate::agent::tools::code_analysis::FindDefinitionTool::new()));
            controller.register_tool(Box::new(crate::agent::tools::system_ops::RunCommandTool::new()));
            
            self.agent_controller = Some(controller);
            info!("Agent controller initialized successfully");
        }
        Ok(())
    }
    
    async fn create_enhanced_system_prompt(&self, context: &crate::intelligence::ContextualInsight) -> Result<String> {
        // Load AI configuration
        let ai_config = self.intelligence_tools.load_ai_configuration().await;
        
        let mut prompt = String::new();
        
        // Add project instructions if available
        if let Some(project_instructions) = &ai_config.project_instructions {
            prompt.push_str("# Project Instructions\n\n");
            prompt.push_str(project_instructions);
            prompt.push_str("\n\n");
        }
        
        // Add current development context
        prompt.push_str("# Current Development Context\n\n");
        prompt.push_str(&format!("**Development Phase**: {}\n", context.development_phase));
        prompt.push_str(&format!("**Current Focus**: {}\n", context.active_story));
        prompt.push_str(&format!("**Confidence**: {:.1}%\n\n", context.confidence * 100.0));
        
        // Add key files information
        if !context.key_files.is_empty() {
            prompt.push_str("## Key Files in Context\n\n");
            for file in &context.key_files {
                prompt.push_str(&format!("- **{}**: {} (relevance: {:.1})\n", 
                    file.path, file.purpose, file.relevance.total_score()));
            }
            prompt.push_str("\n");
        }
        
        // Add architectural context
        if !context.architectural_context.is_empty() {
            prompt.push_str("## Recent Architectural Decisions\n\n");
            for decision in &context.architectural_context {
                prompt.push_str(&format!("- **{}**: {}\n", decision.decision, decision.rationale));
            }
            prompt.push_str("\n");
        }
        
        // Add suggested next actions
        if !context.suggested_next_actions.is_empty() {
            prompt.push_str("## Suggested Next Actions\n\n");
            for action in &context.suggested_next_actions {
                prompt.push_str(&format!("- {}: {} (confidence: {:.1}%)\n", 
                    action.action_type, action.description, action.confidence * 100.0));
            }
            prompt.push_str("\n");
        }
        
        // Add final instructions
        prompt.push_str("## Instructions\n\n");
        prompt.push_str("You are an AI assistant helping with software development. ");
        prompt.push_str("Use the context above to provide accurate, relevant assistance. ");
        prompt.push_str("Be concise and focus on the user's specific request.\n");
        
        Ok(prompt)
    }
    
    /// Handle /search command for semantic code search with optional filters
    async fn handle_search_command(&mut self, query: &str) -> Result<()> {
        info!("Performing semantic search for: '{}'", query);
        
        // Parse search command and filters
        let (search_query, filters) = self.parse_search_command(query);
        let limit = filters.limit.unwrap_or(10);
        
        match self.semantic_search.search(&search_query, limit * 3).await {
            Ok((mut results, mut metrics)) => {
                let original_count = results.len();
                
                // Apply advanced filters
                results = self.apply_search_filters(
                    results,
                    &filters.file_types,
                    &filters.languages,
                    &filters.scope,
                    &filters.chunk_types,
                    filters.min_similarity,
                    filters.max_similarity,
                    &filters.exclude,
                    &filters.include,
                    filters.debug_filters
                );
                
                // Limit results after filtering
                results.truncate(limit);
                
                // Update metrics with filter effectiveness
                if original_count != results.len() {
                    metrics.filtered_results_count = Some(results.len());
                }
                
                if results.is_empty() {
                    let mut message = format!("🔍 No search results found ({})", metrics.format_summary());
                    if original_count > 0 {
                        message.push_str(&format!("\n💡 {} results were filtered out - try adjusting filters", original_count));
                    }
                    self.messages.push(Message::new(
                        MessageRole::System,
                        message,
                    ));
                } else {
                    // Format search results for display
                    let mut result_text = format!("🔍 Found {} search results ({}):\n\n", results.len(), metrics.format_summary());
                    
                    if filters.debug_filters {
                        result_text.push_str(&format!("⏱️ {}\n\n", metrics.format_detailed()));
                    }
                    
                    for (i, result) in results.iter().enumerate() {
                        result_text.push_str(&format!(
                            "{}. **{}** (similarity: {:.2})\n",
                            i + 1,
                            result.file_path.display(),
                            result.similarity_score
                        ));
                        
                        result_text.push_str(&format!(
                            "   Lines {}-{}: {:?}\n",
                            result.chunk.start_line,
                            result.chunk.end_line,
                            result.chunk.chunk_type
                        ));
                        
                        // Show a preview of the content (first 100 characters)
                        let preview = if result.chunk.content.len() > 100 {
                            format!("{}...", &result.chunk.content[..100])
                        } else {
                            result.chunk.content.clone()
                        };
                        
                        result_text.push_str(&format!("   Preview: {}\n\n", preview));
                    }
                    
                    self.messages.push(Message::new(
                        MessageRole::System,
                        result_text,
                    ));
                }
            }
            Err(e) => {
                self.messages.push(Message::new(
                    MessageRole::System,
                    format!("Search failed: {}", e),
                ));
            }
        }
        
        Ok(())
    }
    
    /// Parse search command with optional filters
    fn parse_search_command(&self, input: &str) -> (String, SearchFilters) {
        let mut filters = SearchFilters::default();
        let parts: Vec<&str> = input.split_whitespace().collect();
        let mut query_parts = Vec::new();
        let mut i = 0;
        
        while i < parts.len() {
            let part = parts[i];
            
            if part.starts_with("--") {
                match part {
                    "--file-types" | "--ft" => {
                        if i + 1 < parts.len() {
                            filters.file_types = Some(parts[i + 1].split(',').map(|s| s.to_string()).collect());
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--scope" | "-s" => {
                        if i + 1 < parts.len() {
                            filters.scope = Some(parts[i + 1].split(',').map(|s| s.to_string()).collect());
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--min-similarity" | "--min" => {
                        if i + 1 < parts.len() {
                            if let Ok(val) = parts[i + 1].parse::<f32>() {
                                filters.min_similarity = Some(val);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--exclude" | "-e" => {
                        if i + 1 < parts.len() {
                            filters.exclude = Some(parts[i + 1].split(',').map(|s| s.to_string()).collect());
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--limit" | "-l" => {
                        if i + 1 < parts.len() {
                            if let Ok(val) = parts[i + 1].parse::<usize>() {
                                filters.limit = Some(val);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--debug" => {
                        filters.debug_filters = true;
                        i += 1;
                    }
                    _ => {
                        // Unknown flag, include in query
                        query_parts.push(part);
                        i += 1;
                    }
                }
            } else {
                query_parts.push(part);
                i += 1;
            }
        }
        
        let query = query_parts.join(" ");
        (query, filters)
    }
    
    /// Apply search filters (copied from CLI implementation)
    fn apply_search_filters(
        &self,
        mut results: Vec<crate::semantic_search::SearchResult>,
        file_types: &Option<Vec<String>>,
        languages: &Option<Vec<String>>,
        scope: &Option<Vec<String>>,
        chunk_types: &Option<Vec<String>>,
        min_similarity: Option<f32>,
        max_similarity: Option<f32>,
        exclude: &Option<Vec<String>>,
        include: &Option<Vec<String>>,
        debug_filters: bool,
    ) -> Vec<crate::semantic_search::SearchResult> {
        let original_count = results.len();
        
        // Filter by similarity thresholds
        if let Some(min_sim) = min_similarity {
            results.retain(|r| r.similarity_score >= min_sim);
            if debug_filters {
                debug!("After min similarity filter: {} results", results.len());
            }
        }
        
        if let Some(max_sim) = max_similarity {
            results.retain(|r| r.similarity_score <= max_sim);
            if debug_filters {
                debug!("After max similarity filter: {} results", results.len());
            }
        }
        
        // Filter by file types/extensions
        if let Some(ref types) = file_types {
            let normalized_types: Vec<String> = types.iter()
                .map(|t| normalize_file_type(t))
                .collect();
            
            results.retain(|r| {
                if let Some(ext) = r.file_path.extension().and_then(|e| e.to_str()) {
                    normalized_types.contains(&ext.to_lowercase()) ||
                    normalized_types.contains(&language_from_extension(ext))
                } else {
                    false
                }
            });
            
            if debug_filters {
                debug!("After file type filter: {} results", results.len());
            }
        }
        
        // Filter by languages
        if let Some(ref langs) = languages {
            let normalized_langs: Vec<String> = langs.iter()
                .map(|l| l.to_lowercase())
                .collect();
            
            results.retain(|r| {
                if let Some(ext) = r.file_path.extension().and_then(|e| e.to_str()) {
                    let lang = language_from_extension(ext);
                    normalized_langs.contains(&lang)
                } else {
                    false
                }
            });
            
            if debug_filters {
                debug!("After language filter: {} results", results.len());
            }
        }
        
        // Filter by chunk types
        if let Some(ref chunks) = chunk_types {
            let normalized_chunks: Vec<String> = chunks.iter()
                .map(|c| c.to_lowercase())
                .collect();
            
            results.retain(|r| {
                let chunk_type_str = match r.chunk.chunk_type {
                    crate::vector_search::ChunkType::Function => "function",
                    crate::vector_search::ChunkType::Class => "class",
                    crate::vector_search::ChunkType::Module => "module",
                    crate::vector_search::ChunkType::Comment => "comment",
                    crate::vector_search::ChunkType::Generic => "generic",
                }.to_string();
                
                normalized_chunks.contains(&chunk_type_str)
            });
            
            if debug_filters {
                debug!("After chunk type filter: {} results", results.len());
            }
        }
        
        // Filter by scope (functions, classes, modules, etc.)
        if let Some(ref scopes) = scope {
            let normalized_scopes: Vec<String> = scopes.iter()
                .map(|s| s.to_lowercase())
                .collect();
            
            results.retain(|r| {
                let chunk_type_str = match r.chunk.chunk_type {
                    crate::vector_search::ChunkType::Function => "function",
                    crate::vector_search::ChunkType::Class => "class",
                    crate::vector_search::ChunkType::Module => "module",
                    crate::vector_search::ChunkType::Comment => "comment",
                    crate::vector_search::ChunkType::Generic => "generic",
                }.to_string();
                
                // Check if scope matches chunk type or if "functions" matches "function"
                normalized_scopes.contains(&chunk_type_str) ||
                (chunk_type_str == "function" && normalized_scopes.contains(&"functions".to_string())) ||
                (chunk_type_str == "class" && normalized_scopes.contains(&"classes".to_string())) ||
                (chunk_type_str == "module" && normalized_scopes.contains(&"modules".to_string()))
            });
            
            if debug_filters {
                debug!("After scope filter: {} results", results.len());
            }
        }
        
        // Apply exclude patterns
        if let Some(ref excl_patterns) = exclude {
            results.retain(|r| {
                let path_str = r.file_path.to_string_lossy().to_lowercase();
                !excl_patterns.iter().any(|pattern| {
                    let pattern_lower = pattern.to_lowercase();
                    path_str.contains(&pattern_lower) ||
                    r.chunk.content.to_lowercase().contains(&pattern_lower)
                })
            });
            
            if debug_filters {
                debug!("After exclude filter: {} results", results.len());
            }
        }
        
        // Apply include patterns
        if let Some(ref incl_patterns) = include {
            results.retain(|r| {
                let path_str = r.file_path.to_string_lossy().to_lowercase();
                incl_patterns.iter().any(|pattern| {
                    let pattern_lower = pattern.to_lowercase();
                    path_str.contains(&pattern_lower)
                })
            });
            
            if debug_filters {
                debug!("After include filter: {} results", results.len());
            }
        }
        
        if debug_filters && results.len() != original_count {
            info!("🔍 Filtered search results: {} → {}", original_count, results.len());
        }
        
        results
    }

    /// Handle auth setup events
    async fn handle_auth_setup_events(&mut self, key: ratatui::crossterm::event::KeyEvent) -> Result<bool> {
        // Skip auth setup screen immediately
        self.show_auth_setup = false;
        
        // Allow Ctrl+C to quit
        if matches!(key.code, KeyCode::Char('c')) && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(false);
        }
        
        Ok(true)
    }

    /// Draw the auth setup screen
    fn draw_auth_setup_screen(&self, f: &mut Frame) {
        // Simplified welcome like Claude Code
        let area = f.area();
        
        // Show regular UI with welcome message
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title bar
                Constraint::Min(0),    // Chat area
                Constraint::Length(3), // Input box
                Constraint::Length(1), // Status bar
            ])
            .split(area);

        // Title bar
        let title = Paragraph::new(format!(
            "🏹 Aircher - {} - {} | F1: Help | F2: Settings | /help for commands",
            self.provider_name, self.model
        ))
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(title, chunks[0]);

        // Chat area with welcome message
        if self.messages.is_empty() {
            // Show welcome box in center of chat area
            let welcome_width = 60.min(chunks[1].width - 4);
            let welcome_height = 12.min(chunks[1].height - 4);
            
            let x = chunks[1].x + (chunks[1].width - welcome_width) / 2;
            let y = chunks[1].y + (chunks[1].height - welcome_height) / 2;
            
            let welcome_area = Rect::new(x, y, welcome_width, welcome_height);
            
            // Clear and draw welcome box
            f.render_widget(Clear, welcome_area);
            
            let welcome_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            
            let inner = welcome_block.inner(welcome_area);
            f.render_widget(welcome_block, welcome_area);
            
            // Welcome content
            let welcome_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2), // Title
                    Constraint::Length(1), // Space
                    Constraint::Length(2), // Instructions
                    Constraint::Length(1), // Space
                    Constraint::Length(2), // CWD
                    Constraint::Min(0),    // Space
                ])
                .split(inner);
            
            // Title
            let welcome_title = Paragraph::new("🏹 Welcome to Aircher!")
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);
            f.render_widget(welcome_title, welcome_chunks[0]);
            
            // Instructions based on auth state
            let instructions = if self.providers.is_none() {
                vec![
                    Line::from("  /help for help, /config to set API keys"),
                    Line::from(""),
                ]
            } else {
                vec![
                    Line::from("  /help for help, /model to select model"),
                    Line::from(""),
                ]
            };
            let instructions_widget = Paragraph::new(instructions)
                .style(Style::default())
                .alignment(Alignment::Left);
            f.render_widget(instructions_widget, welcome_chunks[2]);
            
            // Current working directory
            let cwd = std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "unknown".to_string());
            let cwd_text = format!("  cwd: {}", cwd);
            let cwd_widget = Paragraph::new(cwd_text)
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(cwd_widget, welcome_chunks[4]);
            
            // Tip below the welcome box
            let tip_y = welcome_area.y + welcome_area.height + 1;
            if tip_y < chunks[1].y + chunks[1].height - 1 {
                let tip_area = Rect::new(chunks[1].x, tip_y, chunks[1].width, 1);
                let tip = if self.providers.is_none() {
                    " ※ Tip: Configure a provider with /config or F2"
                } else {
                    " ※ Tip: Use /model to change models, /search to explore code"
                };
                let tip_widget = Paragraph::new(tip)
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center);
                f.render_widget(tip_widget, tip_area);
            }
        } else {
            // Draw normal chat area
            self.draw_chat_area(f, chunks[1]);
        }

        // Input box
        self.draw_input_box(f, chunks[2]);

        // Status bar
        self.draw_status_bar(f, chunks[3]);

        // Render autocomplete suggestions
        if self.autocomplete.is_visible() {
            self.autocomplete.render(f, chunks[2]);
        }

        // Render modals
        self.selection_modal.render(f, f.area());
        self.settings_modal.render(f, f.area());
        self.help_modal.render(f, f.area());
        self.model_selection_overlay.render(f, f.area());
    }

    /// Handle AI message sending with proper borrowing
    async fn handle_ai_message(&mut self, message: String) -> Result<()> {
        // Clone the Rc to avoid borrowing issues
        if let Some(providers) = self.providers.clone() {
            // Check budget limits
            if self.check_budget_limits(&providers).await? {
                // Send message to AI
                self.send_message(message, &providers).await?;
            }
        }
        Ok(())
    }
}

/// Detect programming language from project directory
fn detect_language(root: &std::path::Path) -> ProgrammingLanguage {
    if root.join("Cargo.toml").exists() {
        ProgrammingLanguage::Rust
    } else if root.join("package.json").exists() {
        if root.join("tsconfig.json").exists() {
            ProgrammingLanguage::TypeScript
        } else {
            ProgrammingLanguage::JavaScript
        }
    } else if root.join("requirements.txt").exists() || root.join("pyproject.toml").exists() {
        ProgrammingLanguage::Python
    } else if root.join("go.mod").exists() {
        ProgrammingLanguage::Go
    } else if root.join("pom.xml").exists() {
        ProgrammingLanguage::Java
    } else {
        ProgrammingLanguage::Other("Unknown".to_string())
    }
}

/// Search filters for TUI search commands
#[derive(Default)]
struct SearchFilters {
    file_types: Option<Vec<String>>,
    languages: Option<Vec<String>>,
    scope: Option<Vec<String>>,
    chunk_types: Option<Vec<String>>,
    min_similarity: Option<f32>,
    max_similarity: Option<f32>,
    exclude: Option<Vec<String>>,
    include: Option<Vec<String>>,
    debug_filters: bool,
    limit: Option<usize>,
}

/// Normalize file type input (e.g., "rs" -> "rs", "rust" -> "rs")
fn normalize_file_type(file_type: &str) -> String {
    match file_type.to_lowercase().as_str() {
        "rust" => "rs".to_string(),
        "python" => "py".to_string(),
        "javascript" => "js".to_string(),
        "typescript" => "ts".to_string(),
        "c++" | "cpp" => "cpp".to_string(),
        "c#" | "csharp" => "cs".to_string(),
        "golang" | "go" => "go".to_string(),
        other => other.to_string(),
    }
}

/// Get language name from file extension
fn language_from_extension(ext: &str) -> String {
    match ext.to_lowercase().as_str() {
        "rs" => "rust".to_string(),
        "py" => "python".to_string(),
        "js" => "javascript".to_string(),
        "jsx" => "javascript".to_string(),
        "ts" => "typescript".to_string(),
        "tsx" => "typescript".to_string(),
        "cpp" | "cc" | "cxx" => "cpp".to_string(),
        "c" => "c".to_string(),
        "h" | "hpp" => "c".to_string(),
        "cs" => "csharp".to_string(),
        "go" => "go".to_string(),
        "java" => "java".to_string(),
        "rb" => "ruby".to_string(),
        "php" => "php".to_string(),
        "swift" => "swift".to_string(),
        "kt" => "kotlin".to_string(),
        other => other.to_string(),
    }
}
