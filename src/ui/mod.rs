use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
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

pub mod chat;
pub mod components;

pub struct TuiManager {
    config: ConfigManager,
    provider_name: String,
    model: String,
    messages: Vec<Message>,
    input: String,
    scroll_offset: u16,
    session_cost: f64,
    session_tokens: u32,
}

impl TuiManager {
    pub async fn new(config: &ConfigManager) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            provider_name: "claude".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            messages: Vec::new(),
            input: String::new(),
            scroll_offset: 0,
            session_cost: 0.0,
            session_tokens: 0,
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
                        match key.code {
                            KeyCode::Char('c')
                                if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                            {
                                break;
                            }
                            KeyCode::Enter => {
                                if !self.input.is_empty() {
                                    let message = self.input.clone();
                                    self.input.clear();

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
                            KeyCode::Char(c) => {
                                self.input.push(c);
                            }
                            KeyCode::Backspace => {
                                self.input.pop();
                            }
                            KeyCode::Up => {
                                if self.scroll_offset > 0 {
                                    self.scroll_offset -= 1;
                                }
                            }
                            KeyCode::Down => {
                                self.scroll_offset += 1;
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
            "🏹 Aircher - {} - {}",
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
        let input = Paragraph::new(self.input.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Message"));
        f.render_widget(input, area);
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect) {
        let status_text = if self.session_cost > 0.0 {
            format!(
                "💰 ${:.4} | 📊 {} tokens | Ctrl+C to quit",
                self.session_cost, self.session_tokens
            )
        } else {
            "Type your message and press Enter | Ctrl+C to quit".to_string()
        };

        let status = Paragraph::new(status_text).style(Style::default().fg(Color::Gray));
        f.render_widget(status, area);
    }

    async fn send_message(&mut self, _message: String, providers: &ProviderManager) -> Result<()> {
        // Get provider
        let provider = providers
            .get_provider_or_host(&self.provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", self.provider_name))?;

        // Create chat request
        let request = ChatRequest::new(self.messages.clone(), self.model.clone());

        // Send request
        match provider.chat(&request).await {
            Ok(response) => {
                // Add assistant response
                self.messages
                    .push(Message::new(MessageRole::Assistant, response.content));

                // Update session stats
                self.session_tokens += response.tokens_used;
                if let Some(cost) = response.cost {
                    self.session_cost += cost;
                }
            }
            Err(e) => {
                return Err(e);
            }
        }

        Ok(())
    }
}
