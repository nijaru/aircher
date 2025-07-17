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

pub mod chat;
pub mod components;
pub mod selection;
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
    project_manager: ProjectManager,
    session_manager: SessionManager,
    current_session: Option<Session>,
    intelligence_tools: TuiIntelligenceTools,
    file_monitor: Option<file_monitor::FileMonitor>,
    // Modals
    selection_modal: SelectionModal,
    settings_modal: SettingsModal,
    help_modal: HelpModal,
    // Autocomplete
    autocomplete: AutocompleteEngine,
    // State
    budget_warning_shown: bool,
    cost_warnings: Vec<String>,
}

impl TuiManager {
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
        
        // Start background file monitoring
        let file_monitor = file_monitor::start_background_monitoring(
            project_manager.clone(),
            intelligence_tools.clone(),
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
            project_manager,
            session_manager,
            current_session: Some(current_session),
            intelligence_tools,
            file_monitor: Some(file_monitor),
            // Initialize modals
            selection_modal: SelectionModal::new(providers, config),
            settings_modal: SettingsModal::new(config),
            help_modal: HelpModal::new(),
            // Initialize autocomplete
            autocomplete: AutocompleteEngine::new(),
            // Initialize state
            budget_warning_shown: false,
            cost_warnings: Vec::new(),
        })
    }

    pub async fn run(&mut self, providers: &ProviderManager) -> Result<()> {
        info!("Starting TUI interface");

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

                                    // Check budget before sending
                                    if self.check_budget_limits(providers).await? {
                                        // Add user message
                                        self.messages.push(Message::user(message.clone()));

                                        // Send request to AI
                                        if let Err(e) = self.send_message(message, providers).await {
                                            self.messages.push(Message::new(
                                                MessageRole::System,
                                                format!("Error: {}", e),
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
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title bar
                Constraint::Min(0),    // Chat area
                Constraint::Length(3), // Input box
                Constraint::Length(1), // Status bar
            ])
            .split(f.area());

        // Title bar
        let title = Paragraph::new(format!(
            "🏹 Aircher - {} - {} | F1: Help | F2: Settings | Tab: Select",
            self.provider_name, self.model
        ))
        .style(Style::default().fg(Color::Cyan))
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
}
