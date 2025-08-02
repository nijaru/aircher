use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap, Padding},
    Frame,
};
use std::io;

use crate::auth::{AuthManager, AuthStatus};
use crate::config::ConfigManager;
use crate::providers::ProviderManager;

pub struct AuthWizard {
    visible: bool,
    current_step: WizardStep,
    current_provider: Option<String>,
    api_key_input: String,
    cursor_position: usize,
    available_providers: Vec<ProviderInfo>,
    selected_provider_index: usize,
    error_message: Option<String>,
    success_message: Option<String>,
    oauth_url: Option<String>,
}

#[derive(Clone, Debug)]
struct ProviderInfo {
    name: String,
    display_name: String,
    description: String,
    needs_auth: bool,
    auth_status: AuthStatus,
    env_var: String,
}

#[derive(Clone, PartialEq)]
enum WizardStep {
    ProviderSelection,
    ApiKeyEntry,
    OAuth,
    Testing,
    Complete,
}

impl AuthWizard {
    pub fn new() -> Self {
        Self {
            visible: false,
            current_step: WizardStep::ProviderSelection,
            current_provider: None,
            api_key_input: String::new(),
            cursor_position: 0,
            available_providers: Vec::new(),
            selected_provider_index: 0,
            error_message: None,
            success_message: None,
            oauth_url: None,
        }
    }

    pub async fn show(&mut self, config: &ConfigManager, auth_manager: &AuthManager) {
        self.show_with_provider(config, auth_manager, None).await;
    }

    pub async fn show_with_provider(&mut self, config: &ConfigManager, auth_manager: &AuthManager, specific_provider: Option<String>) {
        self.visible = true;
        self.current_step = WizardStep::ProviderSelection;
        self.error_message = None;
        self.success_message = None;
        self.load_providers(config, auth_manager).await;
        
        // If a specific provider is requested, skip to auth for that provider
        if let Some(provider_name) = specific_provider {
            if let Some((index, provider)) = self.available_providers.iter().enumerate()
                .find(|(_, p)| p.name == provider_name) {
                
                self.selected_provider_index = index;
                
                // If provider doesn't need auth, mark as complete
                if !provider.needs_auth {
                    self.success_message = Some(format!("{} doesn't require authentication", provider.display_name));
                    self.current_step = WizardStep::Complete;
                } else if provider.auth_status == AuthStatus::Authenticated {
                    self.success_message = Some(format!("{} is already authenticated", provider.display_name));
                    self.current_step = WizardStep::Complete;
                } else {
                    // Jump directly to API key entry for this provider
                    self.current_provider = Some(provider.name.clone());
                    self.current_step = WizardStep::ApiKeyEntry;
                    self.api_key_input.clear();
                    self.cursor_position = 0;
                }
            }
        }
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.reset_state();
    }

    /// Check if the wizard just completed successfully
    pub fn is_completed_successfully(&self) -> bool {
        matches!(self.current_step, WizardStep::Complete) && self.success_message.is_some()
    }

    /// Get the provider that was just authenticated (if any)
    pub fn get_authenticated_provider(&self) -> Option<String> {
        if self.is_completed_successfully() {
            self.current_provider.clone()
        } else {
            None
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn handle_char(&mut self, c: char) {
        match self.current_step {
            WizardStep::ApiKeyEntry => {
                self.api_key_input.insert(self.cursor_position, c);
                self.cursor_position += 1;
                self.error_message = None;
            }
            _ => {}
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.current_step {
            WizardStep::ApiKeyEntry => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.api_key_input.remove(self.cursor_position);
                    self.error_message = None;
                }
            }
            _ => {}
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.current_step == WizardStep::ApiKeyEntry && self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.current_step == WizardStep::ApiKeyEntry && self.cursor_position < self.api_key_input.len() {
            self.cursor_position += 1;
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.current_step == WizardStep::ProviderSelection && self.selected_provider_index > 0 {
            self.selected_provider_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.current_step == WizardStep::ProviderSelection 
            && self.selected_provider_index < self.available_providers.len().saturating_sub(1) {
            self.selected_provider_index += 1;
        }
    }

    pub async fn handle_enter(&mut self, auth_manager: &AuthManager, config: &ConfigManager, _providers: Option<&ProviderManager>) -> io::Result<()> {
        match self.current_step {
            WizardStep::ProviderSelection => {
                if let Some(provider) = self.available_providers.get(self.selected_provider_index) {
                    if !provider.needs_auth {
                        self.success_message = Some(format!("{} doesn't require authentication", provider.display_name));
                        self.current_step = WizardStep::Complete;
                    } else if provider.auth_status == AuthStatus::Authenticated {
                        self.success_message = Some(format!("{} is already authenticated", provider.display_name));
                        self.current_step = WizardStep::Complete;
                    } else {
                        self.current_provider = Some(provider.name.clone());
                        
                        // Check if this is an OAuth provider
                        if provider.env_var.is_empty() && (provider.name == "anthropic-pro" || provider.name == "anthropic-max") {
                            // Start OAuth flow
                            self.current_step = WizardStep::OAuth;
                            self.error_message = None;
                            
                            // Start OAuth flow
                            match auth_manager.start_oauth_flow(&provider.name).await {
                                Ok(url) => {
                                    self.oauth_url = Some(url.clone());
                                    
                                    // Check if we're in SSH session
                                    use crate::auth::oauth::OAuthHandler;
                                    if OAuthHandler::is_ssh_session() {
                                        // In SSH, show URL for manual copy
                                        self.error_message = Some("SSH session detected - manual authentication required".to_string());
                                    } else {
                                        self.success_message = Some("Opening browser for authentication...".to_string());
                                    }
                                }
                                Err(e) => {
                                    self.error_message = Some(format!("Failed to start OAuth: {}", e));
                                    self.current_step = WizardStep::ProviderSelection;
                                }
                            }
                        } else {
                            // Regular API key entry
                            self.current_step = WizardStep::ApiKeyEntry;
                            self.api_key_input.clear();
                            self.cursor_position = 0;
                        }
                    }
                }
            }
            WizardStep::ApiKeyEntry => {
                if self.api_key_input.trim().is_empty() {
                    self.error_message = Some("API key cannot be empty".to_string());
                    return Ok(());
                }

                if let Some(provider_name) = &self.current_provider {
                    self.current_step = WizardStep::Testing;
                    self.error_message = None;

                    // Store the API key
                    match auth_manager.store_api_key(provider_name, &self.api_key_input).await {
                        Ok(()) => {
                            // Test the API key
                            match auth_manager.test_provider_auth(provider_name, config).await {
                                Ok(auth_info) => {
                                    match auth_info.status {
                                        AuthStatus::Authenticated => {
                                            self.success_message = Some(format!("✓ {} authentication successful!", 
                                                self.get_provider_display_name(provider_name)));
                                            self.current_step = WizardStep::Complete;
                                        }
                                        _ => {
                                            self.error_message = Some(format!("Authentication failed: {}", 
                                                auth_info.error_message.unwrap_or_else(|| "Unknown error".to_string())));
                                            self.current_step = WizardStep::ApiKeyEntry;
                                        }
                                    }
                                }
                                Err(e) => {
                                    self.error_message = Some(format!("Failed to test authentication: {}", e));
                                    self.current_step = WizardStep::ApiKeyEntry;
                                }
                            }
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Failed to store API key: {}", e));
                            self.current_step = WizardStep::ApiKeyEntry;
                        }
                    }
                }
            }
            WizardStep::OAuth => {
                // OAuth is handled automatically in background
                // User can close this window or wait for completion
                self.hide();
            }
            WizardStep::Complete => {
                self.hide();
            }
            _ => {}
        }
        
        Ok(())
    }

    pub fn handle_escape(&mut self) {
        match self.current_step {
            WizardStep::ProviderSelection => {
                self.hide();
            }
            WizardStep::ApiKeyEntry => {
                self.current_step = WizardStep::ProviderSelection;
                self.api_key_input.clear();
                self.cursor_position = 0;
                self.error_message = None;
            }
            WizardStep::Complete => {
                self.hide();
            }
            _ => {}
        }
    }

    async fn load_providers(&mut self, config: &ConfigManager, auth_manager: &AuthManager) {
        let mut providers = Vec::new();
        let auth_statuses = auth_manager.get_all_provider_statuses(config).await;

        for (name, provider_config) in &config.providers {
            let auth_info = auth_statuses.get(name);
            let display_name = self.get_provider_display_name(name);
            let description = self.get_provider_description(name);
            
            providers.push(ProviderInfo {
                name: name.clone(),
                display_name,
                description,
                needs_auth: !provider_config.api_key_env.is_empty(),
                auth_status: auth_info.map(|i| i.status.clone()).unwrap_or(AuthStatus::NotConfigured),
                env_var: provider_config.api_key_env.clone(),
            });
        }

        // Sort providers: authenticated first, then unauthenticated, alphabetical within each group
        providers.sort_by(|a, b| {
            match (&a.auth_status, &b.auth_status) {
                (AuthStatus::Authenticated, AuthStatus::Authenticated) => a.name.cmp(&b.name),
                (AuthStatus::Authenticated, _) => std::cmp::Ordering::Less,
                (_, AuthStatus::Authenticated) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        self.available_providers = providers;
        self.selected_provider_index = 0;
    }

    fn get_provider_display_name(&self, provider: &str) -> String {
        match provider {
            "claude" | "anthropic-api" => "Anthropic Claude".to_string(),
            "anthropic-pro" | "anthropic-max" => "Anthropic Claude Pro/Max".to_string(),
            "openai" => "OpenAI".to_string(),
            "gemini" => "Google Gemini".to_string(),
            "ollama" => "Ollama".to_string(),
            "openrouter" => "OpenRouter".to_string(),
            _ => provider.to_string(),
        }
    }

    fn get_provider_description(&self, provider: &str) -> String {
        match provider {
            "claude" | "anthropic-api" => "Best for complex reasoning and coding tasks".to_string(),
            "anthropic-pro" | "anthropic-max" => "Claude Pro/Max subscription (OAuth)".to_string(),
            "openai" => "GPT models with broad capabilities".to_string(),
            "gemini" => "Google's multimodal AI models".to_string(),
            "ollama" => "Run models locally on your machine".to_string(),
            "openrouter" => "Access multiple providers through one API".to_string(),
            _ => String::new(),
        }
    }

    fn get_auth_status_icon(&self, status: &AuthStatus) -> &'static str {
        match status {
            AuthStatus::Authenticated => "✓",
            AuthStatus::NotConfigured => "✗",
            AuthStatus::Invalid => "✗",
            AuthStatus::Expired => "⚠",
            AuthStatus::RateLimited => "⚠",
            AuthStatus::NetworkError => "⚠",
        }
    }
    
    fn get_auth_status_icon_for_provider(&self, provider: &ProviderInfo) -> &'static str {
        // Special handling for local providers (like Ollama)
        if provider.name == "ollama" {
            match provider.auth_status {
                AuthStatus::Authenticated => "⚡", // Local provider available
                AuthStatus::NetworkError => "✗", // Local provider not found
                _ => self.get_auth_status_icon(&provider.auth_status),
            }
        } else if !provider.needs_auth && matches!(provider.auth_status, AuthStatus::Authenticated) {
            "⚡" // Other local providers
        } else {
            self.get_auth_status_icon(&provider.auth_status)
        }
    }

    fn get_auth_status_color(&self, status: &AuthStatus) -> Color {
        match status {
            AuthStatus::Authenticated => Color::Green,
            AuthStatus::NotConfigured => Color::Yellow,
            AuthStatus::Invalid => Color::Red,
            AuthStatus::Expired => Color::Yellow,
            AuthStatus::RateLimited => Color::Yellow,
            AuthStatus::NetworkError => Color::Red,
        }
    }

    fn reset_state(&mut self) {
        self.current_step = WizardStep::ProviderSelection;
        self.current_provider = None;
        self.api_key_input.clear();
        self.cursor_position = 0;
        self.selected_provider_index = 0;
        self.error_message = None;
        self.success_message = None;
        self.oauth_url = None;
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Create centered overlay (same responsive size as model selection modal)
        let width = (area.width * 70 / 100).max(50).min(80);
        let height = (area.height * 60 / 100).max(15).min(30);
        
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;
        
        let popup_area = Rect::new(x, y, width, height);
        
        // Clear the background
        f.render_widget(Clear, popup_area);

        match self.current_step {
            WizardStep::ProviderSelection => self.render_provider_selection(f, popup_area),
            WizardStep::ApiKeyEntry => self.render_api_key_entry(f, popup_area),
            WizardStep::OAuth => self.render_oauth(f, popup_area),
            WizardStep::Testing => self.render_testing(f, popup_area),
            WizardStep::Complete => self.render_complete(f, popup_area),
        }
    }

    fn render_provider_selection(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(5),    // Provider list
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Authentication Setup")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Provider list
        let items: Vec<ListItem> = self.available_providers
            .iter()
            .enumerate()
            .map(|(i, provider)| {
                let icon = self.get_auth_status_icon_for_provider(provider);
                let color = self.get_auth_status_color(&provider.auth_status);
                
                let status_text = if provider.needs_auth {
                    match provider.auth_status {
                        AuthStatus::Authenticated => "authenticated",
                        _ => "needs setup",
                    }
                } else {
                    "local (no auth needed)"
                };

                let line = Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(color)),
                    Span::styled(&provider.display_name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::raw(" - "),
                    Span::styled(&provider.description, Style::default().fg(Color::Gray)),
                    Span::raw(" ("),
                    Span::styled(status_text, Style::default().fg(color)),
                    Span::raw(")"),
                ]);

                let mut item = ListItem::new(line);
                if i == self.selected_provider_index {
                    item = item.style(Style::default().bg(Color::DarkGray));
                }
                item
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Select a Provider"));
        f.render_widget(list, chunks[1]);

        // Instructions
        let instructions = Paragraph::new("↑↓ Navigate • Enter: Select • Esc: Cancel")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(instructions, chunks[2]);
    }


    fn render_api_key_entry(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(4), // Provider info
                Constraint::Length(3), // Input field
                Constraint::Min(3),    // Error message or instructions
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        // Title
        let provider_name = self.current_provider.as_ref()
            .map(|p| self.get_provider_display_name(p))
            .unwrap_or_else(|| "Provider".to_string());
        let title = Paragraph::new(format!("Setup {}", provider_name))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Provider info
        if let Some(provider) = self.available_providers.iter().find(|p| Some(&p.name) == self.current_provider.as_ref()) {
            let env_info = if !provider.env_var.is_empty() {
                format!("You can also set the {} environment variable instead.", provider.env_var)
            } else {
                String::new()
            };
            
            let info_text = format!("{}\n{}", provider.description, env_info);
            let info = Paragraph::new(info_text)
                .style(Style::default().fg(Color::Gray))
                .wrap(Wrap { trim: true })
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(info, chunks[1]);
        }

        // Input field
        let masked_input = "*".repeat(self.api_key_input.len());
        let input = Paragraph::new(masked_input.as_str())
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("API Key"));
        f.render_widget(input, chunks[2]);

        // Error message or help
        if let Some(error) = &self.error_message {
            let error_widget = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red))
                .wrap(Wrap { trim: true })
                .block(Block::default().borders(Borders::ALL).title("Error"));
            f.render_widget(error_widget, chunks[3]);
        } else {
            let help = Paragraph::new("Enter your API key. It will be stored securely in ~/.aircher/auth.json")
                .style(Style::default().fg(Color::Gray))
                .wrap(Wrap { trim: true })
                .block(Block::default().borders(Borders::ALL).title("Help"));
            f.render_widget(help, chunks[3]);
        }

        // Instructions
        let instructions = Paragraph::new("Enter: Save and test • Esc: Back • Type to enter key")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(instructions, chunks[4]);
    }

    fn render_testing(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(5),    // Testing message
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Testing Authentication")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Testing message
        let message = Paragraph::new("Testing your API key...\n\nThis may take a few seconds.")
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(message, chunks[1]);

        // Instructions
        let instructions = Paragraph::new("Please wait...")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(instructions, chunks[2]);
    }

    fn render_oauth(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(5),    // Content
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        // Title
        let title = Paragraph::new("OAuth Authentication")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Content
        let mut lines = Vec::new();
        
        if let Some(url) = &self.oauth_url {
            use crate::auth::oauth::OAuthHandler;
            if OAuthHandler::is_ssh_session() {
                lines.push(Line::from(vec![
                    Span::styled("SSH session detected - Manual authentication required", Style::default().fg(Color::Yellow))
                ]));
                lines.push(Line::from(""));
                lines.push(Line::from("Please open this URL in your browser:"));
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled(url, Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED))
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("✓ ", Style::default().fg(Color::Green)),
                    Span::from("Opening browser for authentication...")
                ]));
                lines.push(Line::from(""));
                lines.push(Line::from("Please complete the authentication in your browser."));
                lines.push(Line::from(""));
                lines.push(Line::from("This window will close automatically when done."));
            }
        } else {
            lines.push(Line::from(vec![
                Span::styled("Starting OAuth authentication...", Style::default().fg(Color::Gray))
            ]));
        }
        
        let content = List::new(lines)
            .block(Block::default().borders(Borders::ALL).padding(Padding::new(2, 2, 1, 1)));
        f.render_widget(content, chunks[1]);

        // Instructions
        let instructions = Paragraph::new("Press Enter or Esc to close this window")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(instructions, chunks[2]);
    }

    fn render_complete(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(5),    // Message
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        // Title
        let (title_text, title_color) = if self.success_message.is_some() {
            ("Setup Complete", Color::Green)
        } else {
            ("Setup Failed", Color::Red)
        };
        
        let title = Paragraph::new(title_text)
            .style(Style::default().fg(title_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Message
        let default_status = "Unknown status".to_string();
        let message_text = self.success_message.as_ref()
            .or(self.error_message.as_ref())
            .unwrap_or(&default_status);
        
        let message_color = if self.success_message.is_some() {
            Color::Green
        } else {
            Color::Red
        };

        let message = Paragraph::new(message_text.as_str())
            .style(Style::default().fg(message_color))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(message, chunks[1]);

        // Instructions
        let instructions = Paragraph::new("Enter: Close • Esc: Close")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(instructions, chunks[2]);
    }

}