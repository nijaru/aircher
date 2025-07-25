use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind, EnableMouseCapture, DisableMouseCapture},
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
use std::sync::Arc;
use std::time::{Duration, Instant};
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
use crate::auth::AuthManager;

#[derive(Debug, Clone, PartialEq)]
enum NetworkStatus {
    Online,
    Offline,
    Error,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
enum StreamingState {
    /// Not streaming
    Idle,
    /// Uploading/sending message to API
    Hustling {
        start_time: Instant,
        tokens_sent: u32,
    },
    /// Using tools/processing
    Schlepping {
        start_time: Instant,
        tokens_used: u32,
    },
    /// Receiving response stream
    Streaming {
        start_time: Instant,
        tokens_received: u32,
    },
}

pub mod auth_wizard;
pub mod chat;
pub mod command_approval;
pub mod components;
pub mod diff_viewer;
pub mod selection;
pub mod enhanced_selection;
pub mod session_browser;
pub mod settings;
pub mod help;
pub mod autocomplete;
pub mod slash_commands;
pub mod typeahead;
pub mod model_selection;
pub mod syntax_highlight;
pub mod spinners;

use auth_wizard::AuthWizard;
use selection::SelectionModal;
use session_browser::SessionBrowser;
use settings::SettingsModal;
use help::HelpModal;
use autocomplete::AutocompleteEngine;
use command_approval::{CommandApprovalModal, ApprovalChoice};
use model_selection::ModelSelectionOverlay;
use diff_viewer::{DiffViewer, generate_diff};
use slash_commands::{parse_slash_command, format_help};
use syntax_highlight::SyntaxHighlighter;
use spinners::{BRAILLE_SPINNER, THINKING_SPINNER, STAR_PULSE, GROWING_PILLAR};

/// Fun streaming messages for different states
const HUSTLING_MESSAGES: &[&str] = &[
    "Drawing bow",
    "Nocking arrow",
    "Taking aim",
    "Sighting target",
    "Focusing shot",
    "Preparing volley",
    "Loading quiver",
    "Positioning stance",
];

const SCHLEPPING_MESSAGES: &[&str] = &[
    "Calculating trajectory",
    "Analyzing wind patterns",
    "Processing target data",
    "Computing ballistics",
    "Evaluating conditions",
    "Strategizing approach",
    "Calibrating sights",
    "Adjusting for distance",
];

const STREAMING_MESSAGES: &[&str] = &[
    "Receiving data",
    "Collecting responses",
    "Gathering results",
    "Retrieving content",
    "Capturing output",
    "Assembling reply",
    "Downloading stream",
    "Acquiring tokens",
];

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
    session_browser: SessionBrowser,
    diff_viewer: DiffViewer,
    command_approval: CommandApprovalModal,
    auth_wizard: AuthWizard,
    // Autocomplete
    autocomplete: AutocompleteEngine,
    // Syntax highlighting
    syntax_highlighter: SyntaxHighlighter,
    // Permissions
    permissions_manager: crate::permissions::PermissionsManager,
    // Authentication state
    auth_manager: Arc<AuthManager>,
    providers: Option<Rc<ProviderManager>>,
    auth_required: bool,
    show_auth_setup: bool,
    // State
    budget_warning_shown: bool,
    cost_warnings: Vec<String>,
    should_quit: bool,
    // Streaming state
    streaming_state: StreamingState,
    // Ctrl+C handling
    last_ctrl_c_time: Option<Instant>,
    // UI modes (affect next message sent)
    auto_accept_edits: bool,
    plan_mode: bool,
    turbo_mode: bool,
    turbo_mode_start: Option<Instant>,
    // Message history
    message_history: Vec<String>,
    history_index: Option<usize>,
    // Error handling
    recent_app_error: Option<(Instant, String)>,
    network_status: NetworkStatus,
    // Channel for triggering UI updates
    update_tx: Option<mpsc::Sender<String>>,
    // Channel for permission requests from tools
    permission_rx: Option<crate::agent::tools::permission_channel::PermissionRequestReceiver>,
    permission_tx: Option<crate::agent::tools::permission_channel::PermissionRequestSender>,
    // Message queuing during agent processing
    message_queue: Vec<QueuedMessage>,
    agent_processing: bool,
}

#[derive(Debug, Clone)]
struct QueuedMessage {
    content: String,
    timestamp: Instant,
}

impl TuiManager {
    /// Calculate dynamic input height based on content and available space
    fn calculate_input_height(&self, available_height: u16) -> u16 {
        let lines: Vec<&str> = self.input.split('\n').collect();
        let content_lines = lines.len() as u16;
        
        // Calculate reasonable limits based on screen size
        let min_lines = 1u16;
        let max_lines = {
            // Reserve space for: title(1) + chat(min 3) + status(1) + info_line(1) = 6 minimum
            let reserved_space = 6u16;
            let available_for_input = available_height.saturating_sub(reserved_space);
            // Use up to 40% of screen height, but at least 3 lines, max 20 lines
            (available_for_input * 2 / 5).max(3).min(20)
        };
        
        let actual_lines = content_lines.max(min_lines).min(max_lines);
        actual_lines + 2 // +2 for borders
    }
    /// Create TUI manager with authentication state handling
    pub async fn new_with_auth_state(
        config: &ConfigManager,
        auth_manager: Arc<AuthManager>,
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
            model_selection_overlay: {
                let mut overlay = ModelSelectionOverlay::with_auth_manager(config, auth_manager.clone());
                if let Some(ref providers) = providers {
                    // Update provider availability while keeping auth manager
                    overlay.update_provider_availability(providers.as_ref());
                    // IMPORTANT: Update dynamic models from providers (especially for Ollama)
                    overlay.update_dynamic_models(providers.as_ref());
                }
                overlay
            },
            session_browser: SessionBrowser::new(),
            diff_viewer: DiffViewer::new(),
            command_approval: CommandApprovalModal::new(),
            auth_wizard: AuthWizard::new(),
            // Autocomplete
            autocomplete: AutocompleteEngine::new(),
            // Syntax highlighting
            syntax_highlighter: SyntaxHighlighter::new(),
            // Permissions
            permissions_manager: crate::permissions::PermissionsManager::new()?,
            // Authentication state
            auth_manager,
            providers,
            auth_required,
            show_auth_setup: false, // Always start with normal interface
            // State
            budget_warning_shown: false,
            cost_warnings: Vec::new(),
            should_quit: false,
            // Initialize streaming state
            streaming_state: StreamingState::Idle,
            // Initialize Ctrl+C handling
            last_ctrl_c_time: None,
            // Initialize UI modes (session-based, reset on restart)
            auto_accept_edits: false,
            plan_mode: false,
            turbo_mode: false,
            turbo_mode_start: None,
            message_history: Vec::new(),
            history_index: None,
            recent_app_error: None,
            network_status: NetworkStatus::Unknown,
            update_tx: None,
            permission_rx: None,
            permission_tx: None,
            message_queue: Vec::new(),
            agent_processing: false,
        })
    }

    pub async fn new(config: &ConfigManager, auth_manager: Arc<AuthManager>, providers: &ProviderManager) -> Result<Self> {
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
        
        // Initialize permissions manager
        let permissions_manager = crate::permissions::PermissionsManager::new()?;
        
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
            model_selection_overlay: {
                let mut overlay = ModelSelectionOverlay::with_auth_manager(config, auth_manager.clone());
                overlay.update_provider_availability(providers);
                // IMPORTANT: Update dynamic models from providers (especially for Ollama)
                overlay.update_dynamic_models(providers);
                overlay
            },
            session_browser: SessionBrowser::new(),
            diff_viewer: DiffViewer::new(),
            command_approval: CommandApprovalModal::new(),
            auth_wizard: AuthWizard::new(),
            // Initialize autocomplete
            autocomplete: AutocompleteEngine::new(),
            // Initialize syntax highlighting
            syntax_highlighter: SyntaxHighlighter::new(),
            // Initialize permissions
            permissions_manager,
            // Authentication state (providers available in this constructor)
            auth_manager: auth_manager.clone(),
            providers: Some(Rc::new(ProviderManager::new(config, auth_manager).await?)),
            auth_required: false,
            show_auth_setup: false,
            // Initialize state
            budget_warning_shown: false,
            cost_warnings: Vec::new(),
            should_quit: false,
            // Initialize streaming state
            streaming_state: StreamingState::Idle,
            // Initialize Ctrl+C handling
            last_ctrl_c_time: None,
            // Initialize UI modes (session-based, reset on restart)
            auto_accept_edits: false,
            plan_mode: false,
            turbo_mode: false,
            turbo_mode_start: None,
            message_history: Vec::new(),
            history_index: None,
            recent_app_error: None,
            network_status: NetworkStatus::Unknown,
            update_tx: None,
            permission_rx: None,
            permission_tx: None,
            message_queue: Vec::new(),
            agent_processing: false,
        })
    }

    /// Handle application errors - show in chat and update status
    fn handle_app_error(&mut self, error: String) {
        // Show error in conversation
        self.add_message(Message::new(
            MessageRole::System,
            format!("⚠️ Error: {}", error),
        ));
        
        // Update status for indicator
        self.recent_app_error = Some((Instant::now(), error.clone()));
        
        // Log to file
        self.log_error_to_file(&error);
    }
    
    /// Handle network/API errors - show in chat and update network status
    fn handle_network_error(&mut self, error: String) {
        // Show error in conversation
        self.add_message(Message::new(
            MessageRole::System,
            format!("🔴 Network Error: {}", error),
        ));
        
        // Update network status
        self.network_status = NetworkStatus::Error;
        
        // Log to file
        self.log_error_to_file(&error);
    }
    
    /// Log errors to a file for debugging
    fn log_error_to_file(&self, error: &str) {
        use std::fs::OpenOptions;
        use std::io::Write;
        
        // Get log file path - using data dir for cross-platform compatibility
        let log_file = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("aircher")
            .join("tui-errors.log");
            
        // Create directory if needed
        if let Some(parent) = log_file.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        
        // Append error to log file with timestamp
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
        {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            writeln!(file, "[{}] {}", timestamp, error).ok();
        }
    }
    
    /// Check if error should still be displayed (expires after 5 seconds)
    fn should_show_error(&self) -> bool {
        if let Some((timestamp, _)) = &self.recent_app_error {
            Instant::now().duration_since(*timestamp) < Duration::from_secs(5)
        } else {
            false
        }
    }
    
    /// Check if we have any recent errors (for status bar indicator)
    fn has_recent_error(&self) -> bool {
        if let Some((timestamp, _)) = &self.recent_app_error {
            // Show indicator for 30 seconds after error
            Instant::now().duration_since(*timestamp) < Duration::from_secs(30)
        } else {
            false
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting TUI interface (auth_required: {})", self.auth_required);

        // Initialize auth status for UI components
        self.model_selection_overlay.initialize_auth_status(&self.config).await;

        // Setup terminal
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        stdout().execute(EnableMouseCapture)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        // Create channel for async communication
        let (tx, mut rx) = mpsc::channel::<String>(10);
        self.update_tx = Some(tx);
        
        // Create permission channel
        let (perm_tx, mut perm_rx) = crate::agent::tools::permission_channel::create_permission_channel();
        self.permission_tx = Some(perm_tx.clone());

        // Main TUI loop
        loop {
            // Check if we should exit
            if self.should_quit {
                break;
            }
            
            // Process model updates from async tasks
            self.model_selection_overlay.process_model_updates();
            
            // Draw the UI
            terminal.draw(|f| self.draw(f))?;

            // Handle events with timeout (short timeout for responsive streaming)
            if event::poll(std::time::Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
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
                                // 2-stage Ctrl+C like Claude Code: first clears input, second quits
                                let now = Instant::now();
                                if let Some(last_time) = self.last_ctrl_c_time {
                                    // If less than 2 seconds since last Ctrl+C, quit
                                    if now.duration_since(last_time) < Duration::from_secs(2) {
                                        break;
                                    }
                                }
                                
                                // First Ctrl+C or too much time passed
                                if !self.input.is_empty() {
                                    // Clear input if there's text
                                    self.input.clear();
                                    self.cursor_position = 0;
                                    self.autocomplete.hide();
                                    self.last_ctrl_c_time = Some(now);
                                } else {
                                    // No text to clear, quit immediately
                                    break;
                                }
                            }
                            KeyCode::F(1) => {
                                self.help_modal.toggle();
                            }
                            KeyCode::F(2) => {
                                self.settings_modal.toggle();
                            }
                            KeyCode::Tab => {
                                // Shift+Tab cycles modes, regular Tab opens selection modal
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    self.cycle_modes();
                                } else {
                                    self.selection_modal.toggle();
                                }
                            }
                            KeyCode::BackTab => {
                                // BackTab (Shift+Tab) cycles modes
                                self.cycle_modes();
                            }
                            KeyCode::Enter => {
                                // Check if Alt+Enter or Shift+Enter was pressed for newline
                                if key.modifiers.contains(KeyModifiers::ALT) || key.modifiers.contains(KeyModifiers::SHIFT) {
                                    // Add newline to input
                                    self.input.insert(self.cursor_position, '\n');
                                    self.cursor_position += 1;
                                } else if self.autocomplete.is_visible() {
                                    // Accept autocomplete suggestion
                                    if let Some(completion) = self.autocomplete.accept_suggestion() {
                                        // Handle @ file completions differently
                                        if self.input.contains('@') {
                                            // Find the @ position to replace from
                                            if let Some(at_pos) = self.input.rfind('@') {
                                                // Replace everything after @ with the completion
                                                self.input.truncate(at_pos + 1);
                                                self.input.push_str(&completion);
                                                self.cursor_position = self.input.len();
                                            }
                                        } else {
                                            // Normal slash command completion
                                            self.input = completion.clone();
                                            self.cursor_position = self.input.len();
                                        }
                                        
                                        // If it's a complete slash command, execute it immediately
                                        if let Some((command, args)) = parse_slash_command(&completion) {
                                            match command {
                                                "/model" => {
                                                    self.input.clear();
                                                    self.cursor_position = 0;
                                                    self.show_model_selection_with_auth_check().await;
                                                }
                                                "/search" => {
                                                    self.input.clear();
                                                    self.cursor_position = 0;
                                                    self.handle_search_command(args).await?;
                                                }
                                                "/help" => {
                                                    self.input.clear();
                                                    self.cursor_position = 0;
                                                    self.help_modal.toggle();
                                                }
                                                "/clear" => {
                                                    self.input.clear();
                                                    self.cursor_position = 0;
                                                    self.messages.clear();
                                                    self.add_message(Message::new(
                                                        MessageRole::System,
                                                        "Conversation cleared".to_string(),
                                                    ));
                                                }
                                                "/quit" | "/exit" | "/q" => {
                                                    self.should_quit = true;
                                                }
                                                "/auth" | "/login" => {
                                                    self.input.clear();
                                                    self.cursor_position = 0;
                                                    self.auth_wizard.show(&self.config, &self.auth_manager).await;
                                                }
                                                "/sessions" => {
                                                    self.input.clear();
                                                    self.cursor_position = 0;
                                                    self.session_browser.show();
                                                }
                                                "/config" | "/settings" => {
                                                    self.input.clear();
                                                    self.cursor_position = 0;
                                                    self.settings_modal.toggle();
                                                }
                                                _ => {
                                                    // For unknown commands, leave the input as-is so user can modify
                                                }
                                            }
                                        }
                                    }
                                } else if self.auth_wizard.is_visible() {
                                    // Handle auth wizard Enter key
                                    let providers = self.providers.as_ref().map(|p| p.as_ref());
                                    self.auth_wizard.handle_enter(&self.auth_manager, &self.config, providers).await?;
                                    
                                    // Check if auth wizard completed successfully and refresh providers
                                    if self.auth_wizard.is_completed_successfully() {
                                        self.refresh_providers_after_auth().await?;
                                    }
                                } else if !self.input.is_empty() {
                                    let message = self.input.clone();
                                    self.input.clear();
                                    self.cursor_position = 0;
                                    
                                    // Add to message history (but not duplicates of the last message)
                                    if self.message_history.is_empty() || 
                                       self.message_history.last() != Some(&message) {
                                        self.message_history.push(message.clone());
                                        // Keep history size reasonable (e.g., last 1000 messages)
                                        if self.message_history.len() > 1000 {
                                            self.message_history.remove(0);
                                        }
                                    }
                                    // Reset history navigation
                                    self.history_index = None;
                                    
                                    debug!("Processing user message: '{}'", message);

                                    // Check for help shortcut
                                    if message.trim() == "?" {
                                        // Show help
                                        for line in format_help() {
                                            self.add_message(Message::new(
                                                MessageRole::System,
                                                line,
                                            ));
                                        }
                                    }
                                    // Check for slash commands
                                    else if let Some((command, args)) = parse_slash_command(&message) {
                                        match command {
                                            "/model" => {
                                                self.show_model_selection_with_auth_check().await;
                                            }
                                            "/search" => {
                                                if !args.is_empty() {
                                                    // Add user message showing the search command
                                                    self.add_message(Message::user(message.clone()));
                                                    
                                                    // Perform semantic search
                                                    if let Err(e) = self.handle_search_command(args).await {
                                                        self.add_message(Message::new(
                                                            MessageRole::System,
                                                            format!("Search failed: {}", e),
                                                        ));
                                                    }
                                                } else {
                                                    self.add_message(Message::new(
                                                        MessageRole::System,
                                                        "Usage: /search <query>".to_string(),
                                                    ));
                                                }
                                            }
                                            "/init" => {
                                                if let Err(e) = self.handle_init_command().await {
                                                    self.add_message(Message::new(
                                                        MessageRole::System,
                                                        format!("Init failed: {}", e),
                                                    ));
                                                }
                                            }
                                            "/help" => {
                                                // Add each help line as a separate message for proper display
                                                for line in format_help() {
                                                    self.add_message(Message::new(
                                                        MessageRole::System,
                                                        line,
                                                    ));
                                                }
                                            }
                                            "/clear" => {
                                                self.messages.clear();
                                                self.scroll_offset = 0; // Reset scroll on clear
                                                self.add_message(Message::new(
                                                    MessageRole::System,
                                                    "Conversation cleared. Context reset.".to_string(),
                                                ));
                                            }
                                            "/config" => {
                                                self.settings_modal.toggle();
                                            }
                                            "/auth" => {
                                                // Show auth setup wizard
                                                if let Err(e) = self.handle_auth_command().await {
                                                    self.add_message(Message::new(
                                                        MessageRole::System,
                                                        format!("Auth setup failed: {}", e),
                                                    ));
                                                }
                                            }
                                            "/sessions" => {
                                                // Show session browser
                                                self.session_browser.show();
                                                self.load_sessions().await;
                                            }
                                            "/compact" => {
                                                // Add user message showing the compact command
                                                self.add_message(Message::user(message.clone()));
                                                
                                                // Perform conversation compaction
                                                if let Err(e) = self.handle_compact_command(args).await {
                                                    self.add_message(Message::new(
                                                        MessageRole::System,
                                                        format!("Compaction failed: {}", e),
                                                    ));
                                                }
                                            }
                                            "/turbo" => {
                                                // Toggle turbo mode
                                                if self.turbo_mode {
                                                    // Turn off turbo mode, return to default
                                                    self.turbo_mode = false;
                                                    self.turbo_mode_start = None;
                                                    self.auto_accept_edits = false;
                                                    self.plan_mode = false;
                                                    self.add_message(Message::new(
                                                        MessageRole::System,
                                                        "Default mode restored. Will prompt for approval before making changes.".to_string(),
                                                    ));
                                                } else {
                                                    // Enable turbo mode
                                                    self.auto_accept_edits = false;
                                                    self.plan_mode = false;
                                                    self.turbo_mode = true;
                                                    self.turbo_mode_start = Some(Instant::now());
                                                    self.add_message(Message::new(
                                                        MessageRole::System,
                                                        "🚀 Turbo mode activated! I'll autonomously execute complex tasks with full file permissions.".to_string(),
                                                    ));
                                                }
                                            }
                                            "/quit" => {
                                                self.should_quit = true;
                                            }
                                            _ => {
                                                self.add_message(Message::new(
                                                    MessageRole::System,
                                                    format!("Unknown command: {}. Type /help for available commands.", command),
                                                ));
                                            }
                                        }
                                    } else if message.starts_with("/") {
                                        // Unknown slash command
                                        self.add_message(Message::new(
                                            MessageRole::System,
                                            "Unknown command. Type /help for available commands.".to_string(),
                                        ));
                                    } else {
                                        // Handle regular message based on auth state
                                        if self.providers.is_some() {
                                            // Add user message first
                                            self.add_message(Message::user(message.clone()));
                                            
                                            // Send to AI (methods will handle the borrowing internally)
                                            if let Err(e) = self.handle_ai_message(message).await {
                                                self.add_message(Message::new(
                                                    MessageRole::System,
                                                    format!("Error: {}", e),
                                                ));
                                            }
                                        } else {
                                            // Demo mode - show that AI features require API key
                                            debug!("No providers configured, showing demo mode message");
                                            self.add_message(Message::user(message.clone()));
                                            self.add_message(Message::new(
                                                MessageRole::System,
                                                "No AI provider configured. Type /model to select one or /config to set up API keys.".to_string(),
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
                            KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Ctrl+L to jump to bottom (like terminal clear)
                                self.scroll_offset = 0;
                            }
                            KeyCode::Char(c) => {
                                // Insert character at cursor position
                                self.input.insert(self.cursor_position, c);
                                self.cursor_position += 1;
                                
                                // Reset Ctrl+C timer when user starts typing
                                self.last_ctrl_c_time = None;
                                
                                // Reset history navigation when user types
                                self.history_index = None;
                                
                                // Generate autocomplete suggestions
                                use std::fs::OpenOptions;
                                use std::io::Write;
                                let mut file = OpenOptions::new().create(true).append(true).open("/tmp/autocomplete_debug.log").unwrap();
                                writeln!(file, "DEBUG: Typed '{}', input now: '{}'", c, self.input).unwrap();
                                
                                let _result = self.autocomplete.generate_suggestions(&self.input, self.cursor_position);
                                writeln!(file, "DEBUG: Autocomplete visible: {}, suggestions: {}", 
                                         self.autocomplete.is_visible(), self.autocomplete.suggestions.len()).unwrap();
                                if !self.autocomplete.suggestions.is_empty() {
                                    writeln!(file, "DEBUG: Suggestions: {:?}", 
                                             self.autocomplete.suggestions.iter().map(|s| &s.completion).collect::<Vec<_>>()).unwrap();
                                }
                            }
                            KeyCode::Backspace => {
                                if self.cursor_position > 0 {
                                    self.input.remove(self.cursor_position - 1);
                                    self.cursor_position -= 1;
                                    
                                    // Reset Ctrl+C timer when user edits
                                    self.last_ctrl_c_time = None;
                                    
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
                                } else if !self.message_history.is_empty() {
                                    // Navigate through message history
                                    match self.history_index {
                                        None => {
                                            // Start from the most recent message
                                            self.history_index = Some(self.message_history.len() - 1);
                                            self.input = self.message_history[self.message_history.len() - 1].clone();
                                            self.cursor_position = self.input.len();
                                        }
                                        Some(idx) if idx > 0 => {
                                            // Go to older message
                                            self.history_index = Some(idx - 1);
                                            self.input = self.message_history[idx - 1].clone();
                                            self.cursor_position = self.input.len();
                                        }
                                        _ => {} // Already at oldest message
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if self.autocomplete.is_visible() {
                                    self.autocomplete.move_selection_down();
                                } else if let Some(idx) = self.history_index {
                                    // Navigate through message history
                                    if idx < self.message_history.len() - 1 {
                                        // Go to newer message
                                        self.history_index = Some(idx + 1);
                                        self.input = self.message_history[idx + 1].clone();
                                        self.cursor_position = self.input.len();
                                    } else {
                                        // Reached the newest, clear input to allow new typing
                                        self.history_index = None;
                                        self.input.clear();
                                        self.cursor_position = 0;
                                    }
                                }
                            }
                            KeyCode::Left => {
                                if self.cursor_position > 0 {
                                    self.cursor_position -= 1;
                                    // Re-generate suggestions at new cursor position
                                    let _ = self.autocomplete.generate_suggestions(&self.input, self.cursor_position);
                                }
                            }
                            KeyCode::Right => {
                                if self.cursor_position < self.input.len() {
                                    self.cursor_position += 1;
                                    // Re-generate suggestions at new cursor position
                                    let _ = self.autocomplete.generate_suggestions(&self.input, self.cursor_position);
                                }
                            }
                            KeyCode::Esc => {
                                if self.autocomplete.is_visible() {
                                    self.autocomplete.hide();
                                } else if self.streaming_state != StreamingState::Idle {
                                    // Interrupt streaming
                                    self.stop_streaming();
                                    self.add_message(Message::new(
                                        MessageRole::System,
                                        "Streaming interrupted by user".to_string(),
                                    ));
                                }
                            }
                            KeyCode::PageUp => {
                                // Scroll up with overlap (80% of visible height)
                                let visible_height = 20; // Approximate chat area height
                                let scroll_amount = (visible_height * 4 / 5).max(1);
                                self.scroll_offset = self.scroll_offset.saturating_add(scroll_amount);
                            }
                            KeyCode::PageDown => {
                                // Scroll down with overlap
                                let visible_height = 20; // Approximate chat area height
                                let scroll_amount = (visible_height * 4 / 5).max(1);
                                self.scroll_offset = self.scroll_offset.saturating_sub(scroll_amount);
                            }
                            KeyCode::End => {
                                // Jump to bottom
                                self.scroll_offset = 0;
                            }
                            _ => {}
                        }
                    }
                    Event::Mouse(mouse) => {
                        match mouse.kind {
                            MouseEventKind::ScrollUp => {
                                // Scroll up by 3 lines
                                self.scroll_offset = self.scroll_offset.saturating_add(3);
                            }
                            MouseEventKind::ScrollDown => {
                                // Scroll down by 3 lines
                                self.scroll_offset = self.scroll_offset.saturating_sub(3);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            // Handle async messages (trigger redraw on streaming updates)
            while let Ok(msg) = rx.try_recv() {
                debug!("Received async message: {:?}", msg);
                if msg == "update" {
                    // Trigger redraw for streaming updates
                    terminal.draw(|f| self.draw(f))?;
                }
            }
            
            // Handle permission requests
            while let Ok((request, response_tx)) = perm_rx.try_recv() {
                    // Show command approval modal
                    let full_command = if request.args.is_empty() {
                        request.command.clone()
                    } else {
                        format!("{} {}", request.command, request.args.join(" "))
                    };
                    
                    self.command_approval.show(full_command, request.description);
                    terminal.draw(|f| self.draw(f))?;
                    
                    // Wait for user response
                    loop {
                        if let Event::Key(key) = event::read()? {
                            if key.kind == KeyEventKind::Press {
                                match key.code {
                                    KeyCode::Esc => {
                                        let _ = response_tx.send(crate::agent::tools::permission_channel::PermissionResponse::Denied);
                                        self.command_approval.hide();
                                        break;
                                    }
                                    KeyCode::Enter => {
                                        let choice = self.command_approval.get_selected();
                                        let response = match choice {
                                            crate::ui::command_approval::ApprovalChoice::Yes => {
                                                crate::agent::tools::permission_channel::PermissionResponse::Approved
                                            }
                                            crate::ui::command_approval::ApprovalChoice::YesForSession => {
                                                crate::agent::tools::permission_channel::PermissionResponse::ApprovedSimilar
                                            }
                                            crate::ui::command_approval::ApprovalChoice::EditFeedback => {
                                                crate::agent::tools::permission_channel::PermissionResponse::Denied
                                            }
                                            crate::ui::command_approval::ApprovalChoice::Abort => {
                                                crate::agent::tools::permission_channel::PermissionResponse::Denied
                                            }
                                            crate::ui::command_approval::ApprovalChoice::No => {
                                                crate::agent::tools::permission_channel::PermissionResponse::Denied
                                            }
                                        };
                                        let _ = response_tx.send(response);
                                        self.command_approval.hide();
                                        break;
                                    }
                                    KeyCode::Tab | KeyCode::Right => {
                                        self.command_approval.select_next();
                                        terminal.draw(|f| self.draw(f))?;
                                    }
                                    KeyCode::Left => {
                                        self.command_approval.select_prev();
                                        terminal.draw(|f| self.draw(f))?;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    
                // Redraw after modal is hidden
                terminal.draw(|f| self.draw(f))?;
            }
            
            // Process queued messages if agent is not currently processing
            if !self.agent_processing && !self.message_queue.is_empty() {
                if let Some(providers) = self.providers.clone() {
                    let queued = self.message_queue.remove(0);
                    debug!("Processing queued message from main loop: {}", queued.content);
                    if let Err(e) = self.send_message(queued.content, &providers).await {
                        tracing::error!("Failed to process queued message: {}", e);
                        self.add_message(Message::new(
                            MessageRole::System, 
                            format!("❌ Failed to process queued message: {}", e)
                        ));
                    }
                }
            }
        }

        // Cleanup
        disable_raw_mode()?;
        stdout().execute(DisableMouseCapture)?;
        stdout().execute(LeaveAlternateScreen)?;
        
        // Stop file monitoring
        if let Some(monitor) = &self.file_monitor {
            monitor.stop().await;
        }

        info!("TUI interface closed");
        Ok(())
    }

    fn draw(&mut self, f: &mut Frame) {
        // Show auth setup screen if needed
        if self.show_auth_setup {
            self.draw_auth_setup_screen(f);
            return;
        }

        // Let terminal handle background colors

        // Minimal margins like Claude Code
        let screen_height = f.area().height;
        let input_area_height = self.calculate_input_height(screen_height) + 1; // +1 for info line
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(if self.messages.is_empty() { 5 } else { 0 }), // Welcome box
                Constraint::Length(if self.messages.is_empty() { 1 } else { 0 }), // Tip line
                Constraint::Min(1),    // Chat area
                Constraint::Length(input_area_height), // Dynamic input box area
                Constraint::Length(1), // Status line
            ])
            .split(f.area());

        // Show welcome box only when chat is empty
        if self.messages.is_empty() {
            self.draw_welcome_box(f, chunks[0]);
        }

        // Chat area is always at index 2
        self.draw_chat_area(f, chunks[2]);

        // Input box area is always at index 3
        self.draw_input_box(f, chunks[3]);

        // Status line is always at index 4
        self.draw_status_bar(f, chunks[4]);

        // Render modals (on top of everything)
        self.selection_modal.render(f, f.area());
        self.settings_modal.render(f, f.area());
        self.help_modal.render(f, f.area());
        self.model_selection_overlay.render(f, f.area());
        self.session_browser.render(f, f.area());
        self.diff_viewer.render(f, f.area());
        self.command_approval.render(f, f.area());
        self.auth_wizard.render(f, f.area());

        // Render autocomplete suggestions (on top of modals)
        if self.autocomplete.is_visible() {
            // Calculate where the input box is for proper positioning
            let screen_height = f.area().height;
            let input_area_height = self.calculate_input_height(screen_height) + 1;
            let input_y_position = screen_height - input_area_height - 1; // -1 for status bar
            
            // Create a rect representing the input area position
            let input_area = Rect {
                x: 1,
                y: input_y_position,
                width: f.area().width - 2,
                height: input_area_height,
            };
            
            self.autocomplete.render(f, input_area);
        }
    }

    fn draw_welcome_box(&self, f: &mut Frame, area: Rect) {
        // Create a left-aligned welcome box like Claude Code
        let welcome_width = 55;
        let welcome_height = 5;
        let x = 0; // Left-aligned like Claude Code
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
                Span::styled("  Type ", Style::default().fg(Color::Rgb(107, 114, 128))), // Gray like tips
                Span::styled("/", Style::default().fg(Color::Rgb(139, 92, 246))), // Purple highlight
                Span::styled(" to see available commands", Style::default().fg(Color::Rgb(107, 114, 128))),
            ]),
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
    

    fn draw_chat_area(&self, f: &mut Frame, area: Rect) {
        let mut messages: Vec<ListItem> = Vec::new();
        
        for (i, msg) in self.messages.iter().enumerate() {
            // Add subtle spacing before each message (except the first)
            if i > 0 {
                messages.push(ListItem::new(Line::from("")));
            }
            
            // Add message content with role-specific styling and indentation
            let mut message_lines = match msg.role {
                MessageRole::User => {
                    // User messages: Comment-colored like Claude Code
                    let comment_color = Style::default().fg(Color::Rgb(107, 114, 128)); // Comment gray like Claude Code
                    vec![ListItem::new(Line::from(vec![
                        Span::styled("  > ", comment_color), // Slight indentation
                        Span::styled(&msg.content, comment_color),
                    ]))]
                },
                MessageRole::Assistant => {
                    // Assistant messages: Regular font colors with syntax highlighting (like Claude Code)
                    let highlighted_lines = self.syntax_highlighter.highlight_message(&msg.content);
                    highlighted_lines.into_iter().map(|line| {
                        // Add subtle left indentation to create visual separation
                        let mut spans = vec![Span::styled("  ", Style::default())]; // 2-space indent
                        spans.extend(line.spans);
                        ListItem::new(Line::from(spans))
                    }).collect()
                },
                MessageRole::System => {
                    // System messages: Comment-colored like thinking/system messages in Claude Code
                    let comment_color = Style::default().fg(Color::Rgb(107, 114, 128)); // Same as user messages
                    vec![ListItem::new(Line::from(vec![
                        Span::styled("    ℹ ", comment_color), // More indentation for system messages
                        Span::styled(&msg.content, comment_color.add_modifier(Modifier::ITALIC)),
                    ]))]
                },
                MessageRole::Tool => {
                    // Tool messages: Regular font colors like LLM messages, not comment colored
                    let tool_color = Style::default().fg(Color::Rgb(240, 240, 235)); // Off-white like assistant messages
                    vec![ListItem::new(Line::from(vec![
                        Span::styled("    🔧 ", tool_color), // More indentation for tool messages
                        Span::styled(&msg.content, tool_color),
                    ]))]
                },
            };
            
            messages.append(&mut message_lines);
        }

        // Add queued messages with ">" prefix to show they're waiting
        for queued_msg in &self.message_queue {
            let style = Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC);
            messages.push(ListItem::new(Line::from(vec![
                Span::styled("> ", style),
                Span::styled(&queued_msg.content, style),
            ])));
        }

        // Check if we're scrolled and need to show indicator
        let is_scrolled = self.scroll_offset > 0;
        let block = if is_scrolled {
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Line::from(vec![
                    Span::raw(" "),
                    Span::styled("⟱ More below ", Style::default().fg(Color::Yellow)),
                    Span::styled("(Ctrl+L to bottom)", Style::default().fg(Color::DarkGray)),
                    Span::raw(" "),
                ]).alignment(Alignment::Center))
        } else {
            Block::default()
        };

        // List with optional scroll indicator
        let messages_list = List::new(messages)
            .block(block)
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
        
        // Draw error box if we have a recent error
        if self.should_show_error() {
            if let Some((_, error)) = &self.recent_app_error {
                // Position error box at top of chat area
                let error_height = 3;
                let error_area = Rect {
                    x: area.x + 2,
                    y: area.y + 1, // Near top with small margin
                    width: area.width.saturating_sub(4),
                    height: error_height,
                };
                
                // Create error message with word wrapping
                let error_text = vec![
                    Line::from(error.as_str()),
                ];
                
                let error_widget = Paragraph::new(error_text)
                    .block(Block::default()
                        .title(" ⚠ Error ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Red))
                        .style(Style::default().bg(Color::Black)))
                    .style(Style::default().fg(Color::Yellow))
                    .wrap(ratatui::widgets::Wrap { trim: true });
                
                // Clear background and render error
                f.render_widget(Clear, error_area);
                f.render_widget(error_widget, error_area);
            }
        }
    }

    fn draw_input_box(&self, f: &mut Frame, area: Rect) {
        // Use consistent dynamic height calculation
        let input_height = self.calculate_input_height(f.area().height);
        
        // Split area for input and bottom info line
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(input_height), // Dynamic height based on content
                Constraint::Length(1), // Bottom info line
            ])
            .split(area);

        // Input box with rounded corners using Unicode characters
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(163, 136, 186))) // Low-sat purple like Claude's beige border
            .border_set(ratatui::symbols::border::Set {
                top_left: "╭",     // ╭
                top_right: "╮",    // ╮
                bottom_left: "╰",  // ╰
                bottom_right: "╯", // ╯
                vertical_left: "│",   // │
                vertical_right: "│",  // │
                horizontal_top: "─",  // ─
                horizontal_bottom: "─", // ─
            });

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
            let placeholder = Paragraph::new("Type your message and press Enter (Alt+Enter for newlines)")
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

        // Left side: dynamic status based on conversation state and modes
        let shortcuts_line = if self.autocomplete.is_visible() {
            Line::from(Span::styled(
                "↑↓ navigate • Enter accept • Esc cancel",
                Style::default().fg(Color::Rgb(107, 114, 128)) // Comment gray
            ))
        } else if self.messages.is_empty() {
            // Show help discovery when chat is empty
            Line::from(Span::styled(
                "? for shortcuts",
                Style::default().fg(Color::Rgb(107, 114, 128)) // Comment gray
            ))
        } else {
            // Show current mode during conversation with colors
            if self.turbo_mode {
                let elapsed_ms = if let Some(start) = self.turbo_mode_start {
                    start.elapsed().as_millis()
                } else {
                    0
                };
                let spinner_frame = GROWING_PILLAR.get_frame(elapsed_ms);
                Line::from(vec![
                    Span::styled(format!("{} ", spinner_frame), Style::default().fg(Color::Red)), // Animated pillar
                    Span::styled("🚀 turbo mode on", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::styled(" (shift+tab to cycle)", Style::default().fg(Color::Rgb(107, 114, 128)))
                ])
            } else if self.plan_mode {
                Line::from(vec![
                    Span::styled("⏸ ", Style::default().fg(Color::Cyan)), // Turquoise for plan
                    Span::styled("plan mode on", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(" (shift+tab to cycle)", Style::default().fg(Color::Rgb(107, 114, 128)))
                ])
            } else if self.auto_accept_edits {
                Line::from(vec![
                    Span::styled("⏵⏵ ", Style::default().fg(Color::Magenta)), // Purple for auto-accept
                    Span::styled("auto-accept edits on", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    Span::styled(" (shift+tab to cycle)", Style::default().fg(Color::Rgb(107, 114, 128)))
                ])
            } else {
                // Default mode
                Line::from(Span::styled(
                    "shift+tab to cycle modes",
                    Style::default().fg(Color::Rgb(107, 114, 128)) // Comment gray
                ))
            }
        };
        
        // Add left padding to shortcuts like Claude Code
        let padded_shortcuts_area = Rect {
            x: chunks[0].x + 1, // Left padding
            y: chunks[0].y,
            width: chunks[0].width.saturating_sub(1),
            height: chunks[0].height,
        };
        
        let shortcuts_text = Paragraph::new(shortcuts_line)
            .alignment(Alignment::Left);
        f.render_widget(shortcuts_text, padded_shortcuts_area);

        // Right side: model info and context usage (responsive for narrow terminals)
        
        // Clean up model name (remove dates for status bar)
        let clean_model_name = self.model
            .replace("-20241022", "")
            .replace("-20250104", "")
            .replace("-20250114", "");
        
        // Fix provider name mapping
        let display_provider = match self.provider_name.as_str() {
            "claude" => "anthropic",
            provider => provider,
        };
        
        // Context usage percentage - always show when not 100%, show 100% too for clarity
        let estimated_context_window = match self.model.as_str() {
            m if m.contains("gpt-4") => 128000,
            m if m.contains("gpt-3.5") => 16000,
            m if m.contains("claude-3-5-sonnet") => 200000,
            m if m.contains("claude-3-opus") => 200000,
            m if m.contains("claude-3-haiku") => 200000,
            _ => 100000, // Default fallback
        };
        
        let usage_percent = if self.session_tokens > 0 {
            (self.session_tokens as f32 / estimated_context_window as f32 * 100.0).min(100.0)
        } else {
            0.0
        };
        
        let remaining_percent = if usage_percent >= 100.0 {
            0
        } else {
            ((1.0 - usage_percent / 100.0) * 100.0) as u32
        };
        
        // Build right side with model info and context percentage
        let mut parts: Vec<String> = Vec::new();
        
        // Add error indicator first if there's a recent error
        if self.has_recent_error() {
            parts.push("⚠️".to_string()); // Warning triangle
        }
        
        // Model info (always show if space allows)
        let available_width = chunks[1].width as usize;
        if available_width >= 20 {
            if available_width >= 40 {
                // Full format
                parts.push(format!("{} ({})", clean_model_name, display_provider));
            } else {
                // Compact format - just model name
                parts.push(clean_model_name);
            }
        }
        
        // Context percentage (always show)
        parts.push(format!("{}%", remaining_percent));
        
        // Cost if significant and space allows
        if self.session_cost > 0.001 && available_width >= 60 {
            parts.push(format!("${:.3}", self.session_cost));
        }
        
        let right_text = parts.join(" • ");
        
        // Choose color based on context remaining - graduated warning system
        let text_color = if remaining_percent < 5 {
            Color::Red  // Critical red when very low
        } else if remaining_percent < 20 {
            Color::Rgb(255, 165, 0)  // Orange warning when low
        } else {
            Color::Rgb(107, 114, 128)  // Normal comment gray
        };
        
        let right_paragraph = Paragraph::new(right_text)
            .style(Style::default().fg(text_color))
            .alignment(Alignment::Right);
        f.render_widget(right_paragraph, chunks[1]);
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect) {
        // Simple status like Claude Code's bottom status (often empty unless there's an alert)
        let mut status_parts = vec![];
        
        // Streaming indicator if active
        if let Some(streaming_display) = self.get_streaming_display() {
            status_parts.push(streaming_display);
        }
        
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
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.model_selection_overlay.hide();
                    return Ok(true);
                }
                KeyCode::Tab => {
                    self.model_selection_overlay.switch_mode();
                    return Ok(true);
                }
                KeyCode::Enter => {
                    // First check if the current provider is authenticated
                    if !self.model_selection_overlay.is_current_provider_authenticated() {
                        // Provider is not authenticated, show auth wizard
                        self.model_selection_overlay.hide();
                        // Mark that auth setup should be shown - handle async operations outside modal handler
                        self.show_auth_setup = true;
                        return Ok(false); // Let the main event loop handle the async auth operations
                    }
                    
                    if let Some((provider, model)) = self.model_selection_overlay.get_selected() {
                        let old_provider = self.provider_name.clone();
                        let old_model = self.model.clone();
                        self.provider_name = provider.clone();
                        self.model = model.clone();
                        self.model_selection_overlay.hide();
                        
                        // Show confirmation message
                        if old_provider != provider || old_model != model {
                            self.add_message(Message::new(
                                MessageRole::System,
                                format!("Model changed to {} ({})", model, provider),
                            ));
                        }
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

        // Auth wizard
        if self.auth_wizard.is_visible() {
            match key.code {
                KeyCode::Esc => {
                    self.auth_wizard.handle_escape();
                    return Ok(true);
                }
                KeyCode::Enter => {
                    // Mark that Enter was pressed - handle async operations outside modal handler
                    // This is a workaround since handle_modal_events can't be async
                    return Ok(false); // Let the main event loop handle the async auth operations
                }
                KeyCode::Up => {
                    self.auth_wizard.move_selection_up();
                    return Ok(true);
                }
                KeyCode::Down => {
                    self.auth_wizard.move_selection_down();
                    return Ok(true);
                }
                KeyCode::Left => {
                    self.auth_wizard.move_cursor_left();
                    return Ok(true);
                }
                KeyCode::Right => {
                    self.auth_wizard.move_cursor_right();
                    return Ok(true);
                }
                KeyCode::Backspace => {
                    self.auth_wizard.handle_backspace();
                    return Ok(true);
                }
                KeyCode::Char(c) => {
                    self.auth_wizard.handle_char(c);
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

        // Session browser
        if self.session_browser.is_visible() {
            match key.code {
                KeyCode::Esc => {
                    self.session_browser.hide();
                    return Ok(true);
                }
                KeyCode::Enter => {
                    if let Some(_session) = self.session_browser.get_selected() {
                        // Load the selected session
                        // Note: Would need to handle session loading asynchronously
                        // For now, just switch the session browser state
                        self.session_browser.hide();
                    }
                    return Ok(true);
                }
                KeyCode::Up => {
                    self.session_browser.move_up();
                    return Ok(true);
                }
                KeyCode::Down => {
                    self.session_browser.move_down();
                    return Ok(true);
                }
                KeyCode::Char('/') => {
                    // Start filtering - for now just hide browser, would need input handling
                    return Ok(true);
                }
                KeyCode::Char(c) => {
                    // Add to filter
                    let mut current_filter = self.session_browser.get_filter().to_string();
                    current_filter.push(c);
                    self.session_browser.set_filter(current_filter);
                    return Ok(true);
                }
                KeyCode::Backspace => {
                    // Remove from filter
                    let mut current_filter = self.session_browser.get_filter().to_string();
                    current_filter.pop();
                    self.session_browser.set_filter(current_filter);
                    return Ok(true);
                }
                _ => return Ok(true), // Consume all other events
            }
        }

        // Command approval modal
        if self.command_approval.is_visible() {
            match key.code {
                KeyCode::Esc => {
                    self.command_approval.hide();
                    return Ok(true);
                }
                KeyCode::Enter => {
                    let choice = self.command_approval.get_selected();
                    let command = self.command_approval.get_command().to_string();
                    
                    match choice {
                        ApprovalChoice::Yes => {
                            // Just approve this command
                            self.add_message(Message::new(
                                MessageRole::System,
                                format!("✓ Approved command: {}", command),
                            ));
                            // TODO: Actually execute the command
                        }
                        ApprovalChoice::YesForSession => {
                            // Approve this command and similar ones
                            let pattern = crate::permissions::PermissionsManager::get_command_pattern(&command);
                            if let Err(e) = self.permissions_manager.approve_pattern(pattern.clone()) {
                                self.add_message(Message::new(
                                    MessageRole::System,
                                    format!("Failed to save permission: {}", e),
                                ));
                            } else {
                                self.add_message(Message::new(
                                    MessageRole::System,
                                    format!("✓ Approved command pattern: {}*", pattern),
                                ));
                            }
                            // TODO: Actually execute the command
                        }
                        ApprovalChoice::EditFeedback => {
                            let feedback = self.command_approval.get_feedback();
                            self.add_message(Message::new(
                                MessageRole::System,
                                format!("✗ Denied command with feedback: {} ({})", command, feedback),
                            ));
                        }
                        ApprovalChoice::No => {
                            self.add_message(Message::new(
                                MessageRole::System,
                                format!("✗ Denied command: {}", command),
                            ));
                        }
                        ApprovalChoice::Abort => {
                            self.add_message(Message::new(
                                MessageRole::System,
                                format!("✗ Aborted command execution: {}", command),
                            ));
                        }
                    }
                    
                    self.command_approval.hide();
                    return Ok(true);
                }
                KeyCode::Left => {
                    self.command_approval.select_prev();
                    return Ok(true);
                }
                KeyCode::Right => {
                    self.command_approval.select_next();
                    return Ok(true);
                }
                KeyCode::Tab => {
                    self.command_approval.select_next();
                    return Ok(true);
                }
                KeyCode::BackTab => {
                    self.command_approval.select_prev();
                    return Ok(true);
                }
                _ => return Ok(true), // Consume all other events
            }
        }

        // Diff viewer
        if self.diff_viewer.is_visible() {
            match key.code {
                KeyCode::Esc => {
                    self.diff_viewer.hide();
                    return Ok(true);
                }
                KeyCode::Enter => {
                    // Accept the diff - this would apply the changes
                    if let Some(diff) = self.diff_viewer.get_current_diff() {
                        // TODO: Implement actual file writing
                        self.add_message(Message::new(
                            MessageRole::System,
                            format!("Would apply changes to: {}", diff.filename),
                        ));
                    }
                    self.diff_viewer.hide();
                    return Ok(true);
                }
                KeyCode::Up => {
                    self.diff_viewer.scroll_up();
                    return Ok(true);
                }
                KeyCode::Down => {
                    self.diff_viewer.scroll_down();
                    return Ok(true);
                }
                KeyCode::Left => {
                    self.diff_viewer.prev_file();
                    return Ok(true);
                }
                KeyCode::Right => {
                    self.diff_viewer.next_file();
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
                        self.add_message(Message::new(
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
                        self.add_message(Message::new(
                            MessageRole::System,
                            format!("🚫 Budget limit exceeded! Cost would be ${:.4}, limit is ${:.2}", 
                                total_cost, limit),
                        ));
                        return Ok(false);
                    }
                    
                    if total_cost > limit * 0.9 && !self.budget_warning_shown {
                        self.add_message(Message::new(
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
        // Start hustling animation (uploading message)
        // Estimate tokens based on message length (rough approximation)
        let input_tokens = (message.len() / 4) as u32;
        self.start_hustling(input_tokens);
        
        // Ensure agent controller is initialized
        self.ensure_agent_initialized(providers).await?;
        
        // Try to use agent controller for enhanced functionality
        // Switch to schlepping mode before agent processing
        let should_use_agent = self.agent_controller.is_some();
        if should_use_agent {
            self.start_schlepping();
        }
        
        if let Some(ref mut agent) = self.agent_controller {
            // Get provider for agent
            let provider = providers
                .get_provider_or_host(&self.provider_name)
                .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", self.provider_name))?;
            
            info!("Using agent controller for enhanced AI assistance");
            
            // Set agent processing state
            self.agent_processing = true;
            
            // Process message through agent with streaming
            match agent.process_message_streaming(&message, provider, &self.model).await {
                Ok(mut agent_stream) => {
                    // Add user message to local display
                    let user_msg = Message::new(MessageRole::User, message.clone());
                    self.add_message(user_msg);
                    
                    // Start streaming animation
                    self.start_streaming();
                    
                    // Process agent updates
                    let mut assistant_content = String::new();
                    let assistant_msg_id = uuid::Uuid::new_v4().to_string();
                    let mut total_tokens = 0u32;
                    
                    // Add empty assistant message that we'll update
                    let assistant_msg = Message::new(MessageRole::Assistant, String::new());
                    self.add_message(assistant_msg);
                    let assistant_index = self.messages.len() - 1;
                    
                    // Process streaming updates
                    while let Some(update_result) = agent_stream.recv().await {
                        match update_result {
                            Ok(update) => {
                                match update {
                                    crate::agent::streaming::AgentUpdate::ToolStatus(status) => {
                                        // Add tool status message
                                        let tool_msg = Message::new(MessageRole::Tool, status);
                                        self.add_message(tool_msg);
                                    }
                                    crate::agent::streaming::AgentUpdate::TextChunk { content, delta: _, tokens_used } => {
                                        // Update streaming content
                                        assistant_content.push_str(&content);
                                        if let Some(tokens) = tokens_used {
                                            total_tokens = tokens;
                                            self.update_streaming_tokens(tokens);
                                        }
                                        
                                        // Update the assistant message in place
                                        if let Some(msg) = self.messages.get_mut(assistant_index) {
                                            msg.content = assistant_content.clone();
                                        }
                                    }
                                    crate::agent::streaming::AgentUpdate::Complete { total_tokens: final_tokens, tool_status_messages: _ } => {
                                        total_tokens = final_tokens;
                                        break;
                                    }
                                    crate::agent::streaming::AgentUpdate::Error(error) => {
                                        let error_msg = Message::new(MessageRole::System, format!("❌ Agent error: {}", error));
                                        self.add_message(error_msg);
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                let error_msg = Message::new(MessageRole::System, format!("❌ Stream error: {}", e));
                                self.add_message(error_msg);
                                break;
                            }
                        }
                    }
                    
                    // Stop streaming animation
                    self.stop_streaming();
                    
                    // Update session stats
                    self.session_tokens += input_tokens + total_tokens;
                    
                    // Calculate cost for the response (input + output tokens)
                    if let Some(cost) = provider.calculate_cost(input_tokens, total_tokens) {
                        self.session_cost += cost;
                    }
                    
                    // Update session if available
                    if let Some(ref session) = self.current_session {
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
                            id: assistant_msg_id,
                            role: crate::sessions::MessageRole::Assistant,
                            content: assistant_content,
                            timestamp: chrono::Utc::now(),
                            tokens_used: Some(total_tokens),
                            cost: provider.calculate_cost(input_tokens, total_tokens),
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
                    
                    // Agent processing complete - reset state
                    self.agent_processing = false;
                    
                    Ok(())
                }
                Err(e) => {
                    info!("Agent processing failed, falling back to direct provider: {}", e);
                    self.agent_processing = false;
                    self.send_message_fallback(message, providers).await
                }
            }
        } else {
            // Fallback to direct provider if agent initialization failed
            self.send_message_fallback(message, providers).await
        }
    }

    /// Queue a message to be processed later when agent is free
    fn queue_message(&mut self, message: String) {
        debug!("Queuing message while agent is processing: {}", message);
        self.message_queue.push(QueuedMessage {
            content: message,
            timestamp: Instant::now(),
        });
    }


    /// Fallback message sending using direct provider (current implementation)
    async fn send_message_fallback(&mut self, message: String, providers: &ProviderManager) -> Result<()> {
        // Estimate input tokens
        let input_tokens = (message.len() / 4) as u32;
        
        // Add user message to local messages
        let user_msg = Message::new(MessageRole::User, message.clone());
        self.add_message(user_msg);

        // Get provider
        let provider = providers
            .get_provider_or_host(&self.provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", self.provider_name))?;

        // Get intelligence context for the user's message
        let context = self.intelligence_tools.get_development_context(&message).await;
        
        // Create enhanced system prompt with context and mode-specific instructions
        let system_prompt = self.create_enhanced_system_prompt_with_mode(&context).await?;
        
        // Create chat request with enhanced context
        let mut enhanced_messages = vec![Message::system(system_prompt)];
        enhanced_messages.extend(self.messages.clone());
        
        let mut request = ChatRequest::new(enhanced_messages, self.model.clone());
        request.stream = true; // Enable streaming

        // Send streaming request with retry logic
        let mut retry_count = 0;
        const MAX_RETRIES: u32 = 2;
        
        loop {
            match provider.stream(&request).await {
            Ok(mut stream) => {
                // Create initial assistant message
                let mut assistant_content = String::new();
                let assistant_msg_id = uuid::Uuid::new_v4().to_string();
                
                // Add empty assistant message that we'll update
                self.add_message(Message::new(MessageRole::Assistant, String::new()));
                let assistant_idx = self.messages.len() - 1;
                
                // Switch to streaming state
                self.start_streaming();
                
                // Process streaming chunks
                let mut total_tokens = 0u32;
                while let Some(chunk_result) = stream.recv().await {
                    match chunk_result {
                        Ok(chunk) => {
                            // Append content
                            assistant_content.push_str(&chunk.content);
                            
                            // Update the message in place
                            self.messages[assistant_idx] = Message::new(
                                MessageRole::Assistant, 
                                assistant_content.clone()
                            );
                            
                            // Track tokens and update streaming display
                            if let Some(tokens) = chunk.tokens_used {
                                total_tokens = tokens;
                                self.update_streaming_tokens(total_tokens);
                            }
                            
                            // Check if stream is complete
                            if chunk.finish_reason.is_some() {
                                break;
                            }
                            
                            // Trigger UI refresh
                            if let Some(tx) = &self.update_tx {
                                let _ = tx.send("update".to_string()).await;
                            }
                        }
                        Err(e) => {
                            // Handle streaming error
                            self.add_message(Message::new(
                                MessageRole::System,
                                format!("Streaming error: {}", e),
                            ));
                            self.stop_streaming();
                            return Err(e);
                        }
                    }
                }
                
                // Update session stats
                self.session_tokens += input_tokens + total_tokens;
                
                // Calculate cost for the response (input + output tokens)
                if let Some(cost) = provider.calculate_cost(input_tokens, total_tokens) {
                    self.session_cost += cost;
                }
                
                // Stop streaming animation
                self.stop_streaming();
                
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
                        id: assistant_msg_id,
                        role: crate::sessions::MessageRole::Assistant,
                        content: assistant_content,
                        timestamp: chrono::Utc::now(),
                        tokens_used: Some(total_tokens),
                        cost: provider.calculate_cost(input_tokens, total_tokens),
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
                
                // Success - break out of retry loop
                break;
            }
            Err(e) => {
                // Stop streaming animation on error
                self.stop_streaming();
                
                // Determine error type and provide helpful message
                let error_msg = match e.to_string().to_lowercase() {
                    msg if msg.contains("timeout") => {
                        format!("Request timed out. Try again or use /model to switch providers.")
                    }
                    msg if msg.contains("unauthorized") || msg.contains("401") => {
                        format!("Authentication failed. Check your API key with /config.")
                    }
                    msg if msg.contains("rate limit") || msg.contains("429") => {
                        format!("Rate limit exceeded. Please wait a moment before trying again.")
                    }
                    msg if msg.contains("network") || msg.contains("connection") => {
                        format!("Network error: {}. Check your connection.", e)
                    }
                    _ => format!("Error: {}", e)
                };
                
                // Check if we should retry with the same provider
                if Self::is_retryable_error(&e) && retry_count < MAX_RETRIES {
                    retry_count += 1;
                    self.add_message(Message::new(
                        MessageRole::System,
                        format!("Retrying... (attempt {}/{})", retry_count + 1, MAX_RETRIES + 1),
                    ));
                    // Brief delay before retry
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    continue; // Try again
                } else {
                    // Try provider fallback if configured
                    if let Some(fallback_provider) = self.get_fallback_provider(&e) {
                        self.add_message(Message::new(
                            MessageRole::System,
                            format!("Falling back to {} due to error: {}", fallback_provider, error_msg),
                        ));
                        
                        // Update provider and try with fallback
                        let original_provider = self.provider_name.clone();
                        let fallback_provider_name = fallback_provider.clone();
                        self.provider_name = fallback_provider;
                        
                        // Get fallback provider and retry
                        if let Some(fallback_provider_instance) = providers.get_provider_or_host(&self.provider_name) {
                            match fallback_provider_instance.stream(&request).await {
                                Ok(_fallback_stream) => {
                                    // Handle fallback stream success - continue with outer logic
                                    // This is a bit complex to handle cleanly, so for now just log success
                                    self.add_message(Message::new(
                                        MessageRole::System,
                                        format!("✓ Successfully switched to {} provider", self.provider_name),
                                    ));
                                    // Reset to process the fallback stream
                                    // For now, just break and let user retry manually
                                    break;
                                }
                                Err(fallback_error) => {
                                    // Fallback also failed, restore original provider
                                    self.provider_name = original_provider;
                                    self.add_message(Message::new(
                                        MessageRole::System,
                                        format!("Fallback to {} also failed: {}", fallback_provider_name, fallback_error),
                                    ));
                                }
                            }
                        }
                    }
                    
                    // Non-retryable error or max retries exceeded or fallback failed
                    self.add_message(Message::new(
                        MessageRole::System,
                        error_msg,
                    ));
                    break; // Exit retry loop
                }
            }
            } // Close the match arm
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
            let mut controller = AgentController::new(intelligence, self.auth_manager.clone(), project_context)?;
            
            // Register default tools
            controller.register_tool(Box::new(crate::agent::tools::file_ops::ReadFileTool::new()));
            controller.register_tool(Box::new(crate::agent::tools::file_ops::WriteFileTool::new()));
            controller.register_tool(Box::new(crate::agent::tools::file_ops::ListFilesTool::new()));
            controller.register_tool(Box::new(crate::agent::tools::code_analysis::SearchCodeTool::new()));
            controller.register_tool(Box::new(crate::agent::tools::code_analysis::FindDefinitionTool::new()));
            
            // Create RunCommandTool with permissions
            if let Some(perm_tx) = &self.permission_tx {
                let permissions_arc = std::sync::Arc::new(tokio::sync::Mutex::new(self.permissions_manager.clone()));
                let run_command_tool = crate::agent::tools::system_ops::RunCommandTool::with_permissions(
                    permissions_arc,
                    Some(perm_tx.clone())
                );
                controller.register_tool(Box::new(run_command_tool));
            } else {
                controller.register_tool(Box::new(crate::agent::tools::system_ops::RunCommandTool::new()));
            }
            
            self.agent_controller = Some(controller);
            info!("Agent controller initialized successfully");
        }
        Ok(())
    }
    
    async fn create_enhanced_system_prompt_with_mode(&self, context: &crate::intelligence::ContextualInsight) -> Result<String> {
        let mut prompt = self.create_enhanced_system_prompt(context).await?;
        
        // Add mode-specific instructions
        match (self.auto_accept_edits, self.plan_mode, self.turbo_mode) {
            // Turbo mode - autonomous execution like Google Jules
            (_, _, true) => {
                prompt.push_str("\n\n## TURBO MODE - AUTONOMOUS EXECUTION\n");
                prompt.push_str("You are in Turbo Mode - operate like Google's Jules with full autonomy.\n");
                prompt.push_str("- AUTOMATICALLY read files you need to understand the codebase\n");
                prompt.push_str("- AUTOMATICALLY write files when making changes\n"); 
                prompt.push_str("- Create and execute comprehensive plans to complete tasks\n");
                prompt.push_str("- Make multiple file changes in sequence without asking\n");
                prompt.push_str("- Show your reasoning and progress as you work\n");
                prompt.push_str("- Only ask for clarification on ambiguous requirements\n");
                prompt.push_str("- Use available tools liberally to accomplish the task\n");
            }
            // Plan mode - create execution plans first
            (_, true, _) => {
                prompt.push_str("\n\n## PLAN MODE\n");
                prompt.push_str("You are in Plan Mode. For complex tasks:\n");
                prompt.push_str("1. First create a detailed execution plan\n");
                prompt.push_str("2. Break down the task into specific steps\n");
                prompt.push_str("3. Identify files that need to be read or modified\n");
                prompt.push_str("4. Ask for approval before executing the plan\n");
                prompt.push_str("5. Execute step-by-step after approval\n");
            }
            // Auto-accept mode - apply changes without confirmation
            (true, _, _) => {
                prompt.push_str("\n\n## AUTO-ACCEPT MODE\n");
                prompt.push_str("Auto-accept is enabled. You can:\n");
                prompt.push_str("- Read files automatically when needed\n");
                prompt.push_str("- Write files directly without asking for confirmation\n");
                prompt.push_str("- Make multiple file changes in sequence\n");
                prompt.push_str("- Show diffs for transparency but apply changes immediately\n");
            }
            // Default mode - ask for permissions
            _ => {
                prompt.push_str("\n\n## DEFAULT MODE\n");
                prompt.push_str("You are in default mode. Always:\n");
                prompt.push_str("- Ask before reading files\n");
                prompt.push_str("- Ask before writing or modifying files\n");
                prompt.push_str("- Show planned changes and wait for approval\n");
                prompt.push_str("- Respect user's explicit consent for file operations\n");
            }
        }
        
        Ok(prompt)
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
    
    /// Handle /read command to display file contents
    async fn handle_read_command(&mut self, filename: &str) -> Result<()> {
        let filename = filename.trim();
        info!("Reading file: '{}'", filename);
        
        // Handle relative paths from current directory
        let path = if filename.starts_with('/') {
            std::path::PathBuf::from(filename)
        } else {
            std::env::current_dir()?.join(filename)
        };
        
        // Check if file exists and is readable
        if !path.exists() {
            return Err(anyhow::anyhow!("File not found: {}", filename));
        }
        
        if !path.is_file() {
            return Err(anyhow::anyhow!("Path is not a file: {}", filename));
        }
        
        // Read file contents
        let contents = match std::fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to read file '{}': {}", filename, e));
            }
        };
        
        // Get file extension for potential syntax highlighting hint
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("txt");
        
        // Format file contents for display
        let line_count = contents.lines().count();
        let size_kb = contents.len() as f64 / 1024.0;
        
        // Create header message
        let header = format!(
            "📄 **{}** ({} lines, {:.1} KB, .{})", 
            filename, line_count, size_kb, extension
        );
        
        self.add_message(Message::new(MessageRole::System, header));
        
        // Display file contents with line numbers for readability
        if contents.is_empty() {
            self.add_message(Message::new(
                MessageRole::System, 
                "File is empty.".to_string()
            ));
        } else {
            // Split into chunks if file is very large (>100 lines)
            let lines: Vec<&str> = contents.lines().collect();
            
            if lines.len() > 100 {
                // Show first 50 lines, then summary, then last 50 lines
                let first_chunk: String = lines.iter().take(50)
                    .enumerate()
                    .map(|(i, line)| format!("{:4} | {}", i + 1, line))
                    .collect::<Vec<_>>()
                    .join("\n");
                
                self.add_message(Message::new(MessageRole::Assistant, first_chunk));
                
                let middle_lines = lines.len() - 100;
                self.add_message(Message::new(
                    MessageRole::System,
                    format!("... ({} lines omitted) ...", middle_lines)
                ));
                
                let last_chunk: String = lines.iter().skip(lines.len() - 50)
                    .enumerate()
                    .map(|(i, line)| format!("{:4} | {}", lines.len() - 50 + i + 1, line))
                    .collect::<Vec<_>>()
                    .join("\n");
                
                self.add_message(Message::new(MessageRole::Assistant, last_chunk));
            } else {
                // Show complete file with line numbers
                let formatted_contents: String = lines.iter()
                    .enumerate()
                    .map(|(i, line)| format!("{:4} | {}", i + 1, line))
                    .collect::<Vec<_>>()
                    .join("\n");
                
                self.add_message(Message::new(MessageRole::Assistant, formatted_contents));
            }
        }
        
        Ok(())
    }
    
    /// Show example diff for testing the diff viewer
    fn show_example_diff(&mut self) {
        let old_content = r#"function greet(name) {
    console.log("Hello " + name);
    return "Hi there!";
}

function farewell() {
    console.log("Goodbye!");
}"#.to_string();

        let new_content = r#"function greet(name, greeting = "Hello") {
    console.log(greeting + " " + name + "!");
    return `${greeting} there, ${name}!`;
}

function welcome(name) {
    return greet(name, "Welcome");
}

function farewell(name) {
    console.log(`Goodbye, ${name}!`);
    return "See you later!";
}"#.to_string();

        let diff = generate_diff("example.js".to_string(), old_content, new_content);
        
        self.add_message(Message::new(
            MessageRole::System,
            "Showing example diff - use ↑/↓ to scroll, Enter to accept, Esc to cancel".to_string(),
        ));
        
        self.diff_viewer.show_diff(diff);
    }
    
    /// Show model selection overlay with proper auth status checking
    async fn show_model_selection_with_auth_check(&mut self) {
        // Update with real auth status first before showing
        self.model_selection_overlay.initialize_auth_status(&self.config).await;
        
        // Then show the overlay with correct status
        self.model_selection_overlay.show();
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
                    self.add_message(Message::new(
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
                    
                    self.add_message(Message::new(
                        MessageRole::System,
                        result_text,
                    ));
                }
            }
            Err(e) => {
                self.add_message(Message::new(
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
        // If show_auth_setup is true but auth wizard is not visible, show it
        if !self.auth_wizard.is_visible() {
            self.auth_wizard.show(&self.config, &self.auth_manager).await;
            self.show_auth_setup = false;
        }
        
        // Allow Ctrl+C to quit (single press since no input to clear in auth screen)
        if matches!(key.code, KeyCode::Char('c')) && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(false);
        }
        
        Ok(true)
    }

    /// Draw the auth setup screen
    fn draw_auth_setup_screen(&mut self, f: &mut Frame) {
        // Simplified welcome like Claude Code
        let area = f.area();
        
        // Show regular UI with welcome message
        let input_height = self.calculate_input_height(area.height) + 1; // +1 for info line
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title bar
                Constraint::Min(0),    // Chat area
                Constraint::Length(input_height), // Dynamic input box height
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
            
        } else {
            // Draw normal chat area
            self.draw_chat_area(f, chunks[1]);
        }

        // Input box
        self.draw_input_box(f, chunks[2]);

        // Status bar
        self.draw_status_bar(f, chunks[3]);

        // Render modals
        self.selection_modal.render(f, f.area());
        self.settings_modal.render(f, f.area());
        self.help_modal.render(f, f.area());
        self.model_selection_overlay.render(f, f.area());
        self.session_browser.render(f, f.area());
        self.diff_viewer.render(f, f.area());

        // Note: Autocomplete rendering is handled in the main draw method above
        // to ensure proper z-order (on top of modals)
    }

    /// Handle AI message sending with proper borrowing
    async fn handle_ai_message(&mut self, message: String) -> Result<()> {
        // Clone the Rc to avoid borrowing issues
        if let Some(providers) = self.providers.clone() {
            // Check budget limits
            if self.check_budget_limits(&providers).await? {
                // Check if agent is processing - if so, queue the message
                if self.agent_processing {
                    self.queue_message(message);
                    info!("Message queued - agent is currently processing");
                } else {
                    // Send message to AI normally
                    self.send_message(message, &providers).await?;
                }
            }
        }
        Ok(())
    }

    /// Start hustling (uploading/sending)
    pub fn start_hustling(&mut self, tokens: u32) {
        self.streaming_state = StreamingState::Hustling {
            start_time: Instant::now(),
            tokens_sent: tokens,
        };
    }
    
    /// Start schlepping (using tools)
    pub fn start_schlepping(&mut self) {
        self.streaming_state = StreamingState::Schlepping {
            start_time: Instant::now(),
            tokens_used: 0,
        };
    }
    
    /// Start streaming (receiving response)
    pub fn start_streaming(&mut self) {
        self.streaming_state = StreamingState::Streaming {
            start_time: Instant::now(),
            tokens_received: 0,
        };
    }
    
    /// Update token count during streaming
    pub fn update_streaming_tokens(&mut self, tokens: u32) {
        match &mut self.streaming_state {
            StreamingState::Hustling { tokens_sent, .. } => *tokens_sent = tokens,
            StreamingState::Schlepping { tokens_used, .. } => *tokens_used = tokens,
            StreamingState::Streaming { tokens_received, .. } => *tokens_received = tokens,
            _ => {}
        }
    }
    
    /// Stop streaming
    pub fn stop_streaming(&mut self) {
        self.streaming_state = StreamingState::Idle;
    }
    
    /// Get current streaming display
    pub fn get_streaming_display(&self) -> Option<String> {
        match &self.streaming_state {
            StreamingState::Idle => None,
            StreamingState::Hustling { start_time, tokens_sent } => {
                let elapsed = start_time.elapsed();
                let spinner = BRAILLE_SPINNER.get_frame(elapsed.as_millis());
                // Pick a message based on time
                let msg_index = (elapsed.as_secs() as usize / 3) % HUSTLING_MESSAGES.len();
                let message = HUSTLING_MESSAGES[msg_index];
                Some(format!(
                    "{} {}… ({}s · ↑ {}k tokens · esc to interrupt)",
                    spinner, message, elapsed.as_secs(), (*tokens_sent as f32 / 1000.0).max(0.1)
                ))
            }
            StreamingState::Schlepping { start_time, tokens_used } => {
                let elapsed = start_time.elapsed();
                let spinner = THINKING_SPINNER.get_frame(elapsed.as_millis());
                // Pick a message based on time
                let msg_index = (elapsed.as_secs() as usize / 3) % SCHLEPPING_MESSAGES.len();
                let message = SCHLEPPING_MESSAGES[msg_index];
                Some(format!(
                    "{} {}… ({}s · ⚒ {}k tokens · esc to interrupt)",
                    spinner, message, elapsed.as_secs(), (*tokens_used as f32 / 1000.0).max(0.1)
                ))
            }
            StreamingState::Streaming { start_time, tokens_received } => {
                let elapsed = start_time.elapsed();
                let spinner = STAR_PULSE.get_frame(elapsed.as_millis());
                // Pick a message based on time
                let msg_index = (elapsed.as_secs() as usize / 3) % STREAMING_MESSAGES.len();
                let message = STREAMING_MESSAGES[msg_index];
                Some(format!(
                    "{} {}… ({}s · ↓ {}k tokens · esc to interrupt)",
                    spinner, message, elapsed.as_secs(), (*tokens_received as f32 / 1000.0).max(0.1)
                ))
            }
        }
    }

    /// Refresh providers after successful authentication
    pub async fn refresh_providers_after_auth(&mut self) -> Result<()> {
        // Create new ProviderManager with updated authentication
        let provider_manager = ProviderManager::new(&self.config, self.auth_manager.clone()).await?;
        self.providers = Some(Rc::new(provider_manager));
        
        // Update auth state
        self.auth_required = false;
        self.show_auth_setup = false;
        
        // Refresh dependent UI components
        if let Some(providers) = &self.providers {
            self.selection_modal = SelectionModal::new(providers, &self.config);
            
            // Create model selection overlay with auth manager and update with auth status
            self.model_selection_overlay = ModelSelectionOverlay::with_auth_manager(&self.config, self.auth_manager.clone());
            self.model_selection_overlay.update_items_with_auth(&self.config).await;
            // IMPORTANT: Update dynamic models from providers (especially for Ollama)
            self.model_selection_overlay.update_dynamic_models(providers.as_ref());
        }
        
        // Add success message
        self.add_message(Message::new(
            MessageRole::System,
            "✅ Authentication successful! Providers are now available for model selection.".to_string(),
        ));
        
        Ok(())
    }

    /// Cycle through UI modes (like Claude Code's shift+tab)
    fn cycle_modes(&mut self) {
        match (self.auto_accept_edits, self.plan_mode, self.turbo_mode) {
            // Default -> Plan mode
            (false, false, false) => {
                self.plan_mode = true;
                self.add_message(Message::new(
                    MessageRole::System,
                    "⏸ Plan mode enabled. Will create execution plans before making changes.".to_string(),
                ));
            }
            // Plan mode -> Auto-accept
            (false, true, false) => {
                self.plan_mode = false;
                self.auto_accept_edits = true;
                self.add_message(Message::new(
                    MessageRole::System,
                    "⏵⏵ Auto-accept edits enabled. File changes will be applied automatically.".to_string(),
                ));
            }
            // Auto-accept -> Turbo mode
            (true, false, false) => {
                self.auto_accept_edits = false;
                self.turbo_mode = true;
                self.turbo_mode_start = Some(Instant::now());
                self.add_message(Message::new(
                    MessageRole::System,
                    "🚀 Turbo mode enabled. Will autonomously execute complex tasks with full permissions.".to_string(),
                ));
            }
            // Turbo mode -> Default
            _ => {
                self.auto_accept_edits = false;
                self.plan_mode = false;
                self.turbo_mode = false;
                self.turbo_mode_start = None;
                self.add_message(Message::new(
                    MessageRole::System,
                    "Default mode. Will prompt for approval before making changes.".to_string(),
                ));
            }
        }
    }

    // Helper method to add a message and reset scroll to bottom
    fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.scroll_offset = 0; // Reset to bottom
    }
    
    // Helper to check if error is retryable
    fn is_retryable_error(error: &anyhow::Error) -> bool {
        let msg = error.to_string().to_lowercase();
        msg.contains("timeout") || 
        msg.contains("network") || 
        msg.contains("connection") ||
        msg.contains("503") || // Service unavailable
        msg.contains("502") || // Bad gateway
        msg.contains("500")    // Internal server error
    }

    /// Get fallback provider based on error type and configuration
    fn get_fallback_provider(&self, error: &anyhow::Error) -> Option<String> {
        let error_msg = error.to_string().to_lowercase();
        
        // Get provider fallback configuration
        if let Some(fallback_config) = self.config.multi_provider.provider_fallbacks.get(&self.provider_name) {
            // Check if this error type should trigger fallback
            let should_fallback = match () {
                _ if (error_msg.contains("rate limit") || error_msg.contains("429")) && fallback_config.rate_limit_fallback => true,
                _ if (error_msg.contains("timeout") || error_msg.contains("network") || error_msg.contains("connection")) && fallback_config.connection_fallback => true,
                _ if (error_msg.contains("usage") || error_msg.contains("quota") || error_msg.contains("limit")) && fallback_config.max_usage_fallback => true,
                _ => false,
            };
            
            if should_fallback && !fallback_config.fallback_to.is_empty() {
                // Return the first fallback provider
                // TODO: Implement smarter selection based on strategy and auth status
                return Some(fallback_config.fallback_to[0].clone());
            }
        }
        
        None
    }
    
    // Load sessions for the session browser
    async fn load_sessions(&mut self) {
        let filter = crate::sessions::SessionFilter::default();
        match self.session_manager.search_sessions(&filter, Some(50)).await {
            Ok(sessions) => {
                self.session_browser.set_sessions(sessions);
                self.session_browser.set_loading(false);
            }
            Err(e) => {
                self.session_browser.set_error(format!("Failed to load sessions: {}", e));
            }
        }
    }
    
    // Load a specific session and replace current conversation
    async fn load_session(&mut self, session_id: String) -> Result<()> {
        // Load session data
        match self.session_manager.load_session(&session_id).await? {
            Some(session) => {
                // Load session messages
                let session_messages = self.session_manager.load_session_messages(&session_id).await?;
                
                // Clear current messages
                self.messages.clear();
                self.scroll_offset = 0;
                
                // Convert session messages to provider messages
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
                    
                    self.messages.push(provider_msg);
                }
                
                // Update current session and model info
                self.current_session = Some(session.clone());
                self.provider_name = session.provider;
                self.model = session.model;
                self.session_cost = session.total_cost;
                self.session_tokens = session.total_tokens;
                
                // Show confirmation
                self.add_message(Message::new(
                    MessageRole::System,
                    format!("Loaded session: {} ({} messages)", session.title, session.message_count),
                ));
                
                Ok(())
            }
            None => {
                Err(anyhow::anyhow!("Session not found"))
            }
        }
    }

    /// Handle /init command to create or update AGENT.md
    async fn handle_init_command(&mut self) -> Result<()> {
        self.add_message(Message::new(
            MessageRole::System,
            "🎯 Analyzing your codebase to create AGENT.md configuration...".to_string(),
        ));

        // Check if AGENT.md already exists
        let agent_path = if let Ok(path) = self._project_manager.get_agent_config_path() {
            path
        } else {
            std::env::current_dir()?.join("AGENT.md")
        };

        if agent_path.exists() {
            self.add_message(Message::new(
                MessageRole::System,
                format!("✅ AGENT.md already exists at: {}", agent_path.display()),
            ));
            self.add_message(Message::new(
                MessageRole::System,
                "Use your editor to customize it, or delete it to regenerate.".to_string(),
            ));
            return Ok(());
        }

        // Analyze the codebase
        let project_root = std::env::current_dir()?;
        let project_name = project_root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Detect language and build tools
        let (language, build_commands) = self.detect_project_setup(&project_root)?;

        // Create AGENT.md content
        let agent_content = self.generate_agent_md_content(&project_name, &language, &build_commands)?;

        // Write AGENT.md
        std::fs::write(&agent_path, agent_content)?;

        self.add_message(Message::new(
            MessageRole::System,
            format!("✅ Created AGENT.md at: {}", agent_path.display()),
        ));
        self.add_message(Message::new(
            MessageRole::System,
            "This file will be automatically loaded in future sessions for consistent AI context.".to_string(),
        ));

        Ok(())
    }

    /// Handle /compact command to summarize and compress conversation history
    async fn handle_compact_command(&mut self, custom_instructions: &str) -> Result<()> {
        if self.messages.is_empty() {
            self.add_message(Message::new(
                MessageRole::System,
                "No conversation to compact.".to_string(),
            ));
            return Ok(());
        }

        // Check if we have an LLM provider available
        let providers = match &self.providers {
            Some(providers) => providers.clone(),
            None => {
                self.add_message(Message::new(
                    MessageRole::System,
                    "No AI provider available for conversation compaction.".to_string(),
                ));
                return Ok(());
            }
        };

        let provider = providers
            .get_provider_or_host(&self.provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", self.provider_name))?;

        info!("Starting conversation compaction with {} messages", self.messages.len());

        // Show compaction progress
        self.add_message(Message::new(
            MessageRole::System,
            "🗜️ Compacting conversation history...".to_string(),
        ));

        // Build conversation history for summarization (exclude system messages)
        let conversation_text = self.messages
            .iter()
            .filter(|msg| !matches!(msg.role, MessageRole::System))
            .map(|msg| {
                let role_prefix = match msg.role {
                    MessageRole::User => "User",
                    MessageRole::Assistant => "Assistant", 
                    MessageRole::Tool => "Tool",
                    MessageRole::System => "System", // Won't be included due to filter
                };
                format!("{}: {}", role_prefix, msg.content)
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        // Create summarization prompt
        let mut prompt = format!(
            "Please create a concise summary of this conversation that captures:\n\
            1. The main topics discussed\n\
            2. Key decisions or conclusions reached\n\
            3. Important context that should be preserved\n\
            4. Current project state or progress\n\n\
            Keep the summary detailed enough to maintain continuity but concise enough to save context.\n\n"
        );

        // Add custom instructions if provided
        if !custom_instructions.is_empty() {
            prompt.push_str(&format!(
                "Special instructions for this compaction: {}\n\n",
                custom_instructions
            ));
        }

        prompt.push_str("Conversation to summarize:\n\n");
        prompt.push_str(&conversation_text);

        // Create chat request for summarization
        let messages = vec![Message::user(prompt)];
        let mut request = ChatRequest::new(messages, self.model.clone());
        request.temperature = Some(0.3); // Lower temperature for consistent summaries
        request.max_tokens = Some(1000); // Reasonable limit for summaries

        // Get summary from LLM
        match provider.chat(&request).await {
            Ok(response) => {
                // Calculate token savings
                let original_tokens = self.session_tokens;
                let summary_tokens = response.tokens_used;
                let tokens_saved = original_tokens.saturating_sub(summary_tokens);

                // Create summary message
                let summary_content = format!(
                    "📋 **Conversation Summary** (saved {} tokens)\n\n{}",
                    tokens_saved,
                    response.content
                );

                // Add custom instructions as context if provided
                let final_summary = if !custom_instructions.is_empty() {
                    format!(
                        "{}\n\n---\n**Additional Context**: {}",
                        summary_content,
                        custom_instructions
                    )
                } else {
                    summary_content
                };

                // Clear conversation except for the last few messages and replace with summary
                let recent_messages_to_keep = 3; // Keep last 3 messages for immediate context
                let messages_to_keep = if self.messages.len() > recent_messages_to_keep {
                    self.messages.split_off(self.messages.len() - recent_messages_to_keep)
                } else {
                    std::mem::take(&mut self.messages)
                };

                // Start fresh with summary
                self.messages.clear();
                self.messages.push(Message::new(MessageRole::System, final_summary));
                
                // Add back recent messages
                self.messages.extend(messages_to_keep);

                // Update session stats
                self.session_tokens = summary_tokens + (recent_messages_to_keep as u32 * 50); // Rough estimate

                self.add_message(Message::new(
                    MessageRole::System,
                    format!(
                        "✅ Conversation compacted successfully. Saved approximately {} tokens.",
                        tokens_saved
                    ),
                ));

                info!("Conversation compacted: {} → {} tokens saved", original_tokens, tokens_saved);
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to generate conversation summary: {}", e));
            }
        }

        Ok(())
    }

    /// Handle /auth command to show authentication setup wizard
    async fn handle_auth_command(&mut self) -> Result<()> {
        // Show the auth wizard using shared auth manager
        self.auth_wizard.show(&self.config, &self.auth_manager).await;
        
        Ok(())
    }

    /// Detect project setup and build commands
    fn detect_project_setup(&self, root: &std::path::Path) -> Result<(String, Vec<String>)> {
        let mut language = "Unknown".to_string();
        let mut build_commands = Vec::new();

        // Detect language and build system
        if root.join("Cargo.toml").exists() {
            language = "Rust".to_string();
            build_commands.push("cargo build".to_string());
            build_commands.push("cargo test".to_string());
            build_commands.push("cargo check".to_string());
        } else if root.join("package.json").exists() {
            language = "JavaScript/TypeScript".to_string();
            build_commands.push("npm install".to_string());
            build_commands.push("npm run build".to_string());
            build_commands.push("npm test".to_string());
        } else if root.join("requirements.txt").exists() || root.join("pyproject.toml").exists() {
            language = "Python".to_string();
            build_commands.push("pip install -r requirements.txt".to_string());
            build_commands.push("python -m pytest".to_string());
        } else if root.join("go.mod").exists() {
            language = "Go".to_string();
            build_commands.push("go build".to_string());
            build_commands.push("go test ./...".to_string());
        }

        Ok((language, build_commands))
    }

    /// Generate AGENT.md content based on project analysis
    fn generate_agent_md_content(&self, project_name: &str, language: &str, build_commands: &[String]) -> Result<String> {
        let mut content = String::new();
        
        content.push_str(&format!("# {} Agent Configuration\n\n", project_name));
        content.push_str("This file provides context and instructions for AI agents working with this project.\n\n");
        
        content.push_str("## Project Overview\n\n");
        content.push_str(&format!("{} is a {} project.\n\n", project_name, language));
        
        if !build_commands.is_empty() {
            content.push_str("## Build Commands\n\n");
            for cmd in build_commands {
                content.push_str(&format!("- `{}`\n", cmd));
            }
            content.push_str("\n");
        }
        
        content.push_str("## Development Guidelines\n\n");
        content.push_str("### Code Quality\n");
        content.push_str("- Write clean, readable code with proper error handling\n");
        content.push_str("- Follow existing code style and conventions\n");
        content.push_str("- Add tests for new functionality\n\n");
        
        content.push_str("### Architecture\n");
        content.push_str("- Maintain separation of concerns\n");
        content.push_str("- Use appropriate design patterns\n");
        content.push_str("- Document complex logic and decisions\n\n");
        
        content.push_str("## Notes for AI Agents\n\n");
        content.push_str("- Prioritize code quality and maintainability\n");
        content.push_str("- Consider performance implications of changes\n");
        content.push_str("- Preserve existing functionality unless explicitly changing it\n");
        content.push_str("- Ask for clarification when requirements are unclear\n\n");
        
        content.push_str("## Quick Commands\n\n");
        content.push_str("```bash\n");
        for cmd in build_commands {
            content.push_str(&format!("# {}\n", cmd));
        }
        content.push_str("```\n");

        Ok(content)
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
