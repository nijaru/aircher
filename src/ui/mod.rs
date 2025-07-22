use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
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

pub mod chat;
pub mod components;
pub mod selection;
pub mod enhanced_selection;
pub mod settings;
pub mod help;
pub mod autocomplete;

use selection::SelectionModal;
use settings::SettingsModal;
use help::HelpModal;
use autocomplete::AutocompleteEngine;

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
    // Modals
    selection_modal: SelectionModal,
    settings_modal: SettingsModal,
    help_modal: HelpModal,
    // Autocomplete
    autocomplete: AutocompleteEngine,
    // Authentication state
    providers: Option<Rc<ProviderManager>>,
    auth_required: bool,
    show_auth_setup: bool,
    // State
    budget_warning_shown: bool,
    cost_warnings: Vec<String>,
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
            // Modals
            selection_modal: if let Some(ref providers) = providers {
                SelectionModal::new(providers.as_ref(), config)
            } else {
                // In demo mode, create a selection modal from config only
                SelectionModal::from_config(config)
            },
            settings_modal: SettingsModal::new(config),
            help_modal: HelpModal::new(),
            // Autocomplete
            autocomplete: AutocompleteEngine::new(),
            // Authentication state
            providers,
            auth_required,
            show_auth_setup: auth_required,
            // State
            budget_warning_shown: false,
            cost_warnings: Vec::new(),
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
            // Initialize modals
            selection_modal: SelectionModal::new(providers, config),
            settings_modal: SettingsModal::new(config),
            help_modal: HelpModal::new(),
            // Initialize autocomplete
            autocomplete: AutocompleteEngine::new(),
            // Authentication state (providers available in this constructor)
            providers: Some(Rc::new(ProviderManager::new(config).await?)),
            auth_required: false,
            show_auth_setup: false,
            // Initialize state
            budget_warning_shown: false,
            cost_warnings: Vec::new(),
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

                                    // Check if it's a search command
                                    if message.starts_with("/search ") {
                                        let query = message.strip_prefix("/search ").unwrap_or("").trim();
                                        if !query.is_empty() {
                                            // Add user message showing the search command
                                            self.messages.push(Message::user(message.clone()));
                                            
                                            // Perform semantic search
                                            if let Err(e) = self.handle_search_command(query).await {
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
                                    } else {
                                        // Handle message based on auth state
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
                                            self.messages.push(Message::user(message.clone()));
                                            self.messages.push(Message::new(
                                                MessageRole::System,
                                                "\u{1f4a1} Demo Mode: AI chat requires API key configuration. Press F2 for Settings or type '/search <query>' to explore the codebase.".to_string(),
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
                                    self.scroll_offset += 1;
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

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title bar
                Constraint::Min(0),    // Chat area
                Constraint::Length(3), // Input box
                Constraint::Length(1), // Status bar
            ])
            .split(f.area());

        // Title bar with auth status
        let auth_status = if self.auth_required { "[Demo Mode]" } else { "" };
        let title = Paragraph::new(format!(
            "🏹 Aircher {} - {} - {} | F1: Help | F2: Settings | Tab: Select | /search <query> [--filters]",
            auth_status, self.provider_name, self.model
        ))
        .style(Style::default().fg(if self.auth_required { Color::Yellow } else { Color::Cyan }))
        .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(title, chunks[0]);

        // Chat area
        self.draw_chat_area(f, chunks[1]);

        // Input box
        self.draw_input_box(f, chunks[2]);

        // Status bar
        self.draw_status_bar(f, chunks[3]);

        // Render autocomplete suggestions (above input box)
        if self.autocomplete.is_visible() {
            self.autocomplete.render(f, chunks[2]);
        }

        // Render modals (on top of everything)
        self.selection_modal.render(f, f.area());
        self.settings_modal.render(f, f.area());
        self.help_modal.render(f, f.area());
    }

    fn draw_chat_area(&self, f: &mut Frame, area: Rect) {
        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .map(|msg| {
                let prefix = match msg.role {
                    MessageRole::User => "👤 You: ",
                    MessageRole::Assistant => "🤖 AI: ",
                    MessageRole::System => "⚙️ System: ",
                    MessageRole::Tool => "🔧 Tool: ",
                };

                let style = match msg.role {
                    MessageRole::User => Style::default().fg(Color::Green),
                    MessageRole::Assistant => Style::default().fg(Color::Blue),
                    MessageRole::System => Style::default().fg(Color::Red),
                    MessageRole::Tool => Style::default().fg(Color::Yellow),
                };

                ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::raw(&msg.content),
                ]))
            })
            .collect();

        let messages_list =
            List::new(messages).block(Block::default().borders(Borders::ALL).title("Chat"));

        f.render_widget(messages_list, area);
    }

    fn draw_input_box(&self, f: &mut Frame, area: Rect) {
        // Create input display with cursor and preview
        let mut input_display = self.input.clone();
        
        // Add inline preview if available
        if let Some(preview) = self.autocomplete.get_inline_preview() {
            input_display.push_str(&preview);
        }
        
        // Create title with autocomplete hint
        let title = if self.autocomplete.is_visible() {
            "Message (↑↓ navigate, Enter accept, Esc cancel)"
        } else {
            "Message (Ctrl+Space for suggestions)"
        };
        
        let input_style = if self.autocomplete.is_visible() {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Yellow)
        };
        
        let input = Paragraph::new(input_display.as_str())
            .style(input_style)
            .block(Block::default().borders(Borders::ALL).title(title));
        f.render_widget(input, area);
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect) {
        let mut status_parts = vec![];
        
        // Cost and tokens
        if self.session_cost > 0.0 {
            status_parts.push(format!("💰 ${:.4}", self.session_cost));
            status_parts.push(format!("📊 {} tokens", self.session_tokens));
        }
        
        // Budget warning
        if let Some(limit) = self.config.global.budget_limit {
            if self.session_cost > limit * 0.8 {
                status_parts.push("⚠️  Approaching budget limit".to_string());
            }
        }
        
        // Add basic instructions
        if status_parts.is_empty() {
            status_parts.push("Type your message and press Enter | Ctrl+C to quit".to_string());
        } else {
            status_parts.push("Ctrl+C to quit".to_string());
        }
        
        let status_text = status_parts.join(" | ");
        let status_color = if self.cost_warnings.is_empty() {
            Color::Gray
        } else {
            Color::Yellow
        };
        
        let status = Paragraph::new(status_text).style(Style::default().fg(status_color));
        f.render_widget(status, area);
    }

    fn handle_modal_events(&mut self, key: ratatui::crossterm::event::KeyEvent) -> Result<bool> {
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
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Allow Ctrl+C to quit even in auth setup
                return Ok(false);
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // 's' to skip auth setup and enter demo mode
                self.show_auth_setup = false;
                self.messages.push(Message::new(
                    MessageRole::System,
                    "🚀 Entering Demo Mode! You can explore the codebase with '/search <query>' commands. API chat features require configuration.".to_string(),
                ));
                return Ok(true);
            }
            KeyCode::F(2) => {
                // F2 to go to settings for API key setup
                self.settings_modal.toggle();
                self.show_auth_setup = false;
                return Ok(true);
            }
            KeyCode::Esc => {
                // Esc to skip auth setup
                self.show_auth_setup = false;
                self.messages.push(Message::new(
                    MessageRole::System,
                    "🚀 Welcome to Demo Mode! Use '/search <query>' to explore code or press F2 for Settings.".to_string(),
                ));
                return Ok(true);
            }
            _ => return Ok(true), // Consume other events in auth mode
        }
    }

    /// Draw the auth setup screen
    fn draw_auth_setup_screen(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Instructions
            ])
            .split(f.area());

        // Title
        let title = Paragraph::new("🏹 Welcome to Aircher!")
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Main content area
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Status
                Constraint::Length(4), // Demo mode info
                Constraint::Length(4), // API setup info
                Constraint::Min(0),    // Available features
            ])
            .split(chunks[1]);

        // API key status
        let status_text = if self.auth_required {
            "❌ No API keys configured"
        } else {
            "✅ API keys configured"
        };
        let status = Paragraph::new(status_text)
            .style(Style::default().fg(if self.auth_required { Color::Red } else { Color::Green }))
            .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(status, content_chunks[0]);

        // Demo mode info
        let demo_info = Paragraph::new(
            "🚀 Demo Mode Available:\n\
             • Semantic code search with '/search <query>'\n\
             • File monitoring and project analysis\n\
             • Full TUI interface exploration"
        )
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Demo Features"));
        f.render_widget(demo_info, content_chunks[1]);

        // API setup info
        let api_info = Paragraph::new(
            "🔑 Full Features (requires API keys):\n\
             • AI chat assistance\n\
             • Code generation and analysis\n\
             • Intelligent context suggestions"
        )
        .style(Style::default().fg(Color::Blue))
        .block(Block::default().borders(Borders::ALL).title("Full Features"));
        f.render_widget(api_info, content_chunks[2]);

        // Available features in demo mode
        let features = vec![
            "✅ Semantic Search - Find code by meaning",
            "✅ File Monitoring - Track project changes", 
            "✅ Intelligence Tools - Project analysis",
            "✅ TUI Interface - Full terminal experience",
            "⚙️  Settings Panel - Configure API keys",
        ];

        let feature_items: Vec<ratatui::widgets::ListItem> = features
            .iter()
            .map(|f| ratatui::widgets::ListItem::new(*f))
            .collect();

        let features_list = ratatui::widgets::List::new(feature_items)
            .block(Block::default().borders(Borders::ALL).title("Available Now"));
        f.render_widget(features_list, content_chunks[3]);

        // Instructions
        let instructions = Paragraph::new(
            "📋 Options:\n\
             [S] Start Demo Mode | [F2] Configure API Keys | [Ctrl+C] Exit | [Esc] Skip Setup"
        )
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Instructions"));
        f.render_widget(instructions, chunks[2]);
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
