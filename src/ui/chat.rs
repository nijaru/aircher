use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::providers::{Message, MessageRole};

pub struct ChatWidget {
    messages: Vec<Message>,
    scroll_offset: u16,
}

impl ChatWidget {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll_offset: 0,
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.scroll_offset = 0;
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .skip(self.scroll_offset as usize)
            .map(|msg| {
                let (prefix, style) = match msg.role {
                    MessageRole::User => ("👤 You: ", Style::default().fg(Color::Green)),
                    MessageRole::Assistant => ("🤖 AI: ", Style::default().fg(Color::Blue)),
                    MessageRole::System => ("⚙️ System: ", Style::default().fg(Color::Red)),
                    MessageRole::Tool => ("🔧 Tool: ", Style::default().fg(Color::Yellow)),
                };

                // Handle long messages by wrapping them
                let content = if msg.content.len() > 100 {
                    format!("{}...", &msg.content[..100])
                } else {
                    msg.content.clone()
                };

                ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::raw(content),
                ]))
            })
            .collect();

        let messages_list =
            List::new(messages).block(Block::default().borders(Borders::ALL).title("Chat History"));

        f.render_widget(messages_list, area);
    }
}

pub struct InputWidget {
    input: String,
    cursor_position: usize,
}

impl InputWidget {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
        }
    }

    pub fn get_input(&self) -> &str {
        &self.input
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
    }

    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let input_text = if self.input.is_empty() {
            "Type your message here..."
        } else {
            &self.input
        };

        let input_style = if self.input.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Yellow)
        };

        let input_widget = Paragraph::new(input_text)
            .style(input_style)
            .block(Block::default().borders(Borders::ALL).title("Message"))
            .wrap(Wrap { trim: true });

        f.render_widget(input_widget, area);
    }
}

pub struct StatusWidget {
    provider: String,
    model: String,
    session_cost: f64,
    session_tokens: u32,
}

impl StatusWidget {
    pub fn new(provider: String, model: String) -> Self {
        Self {
            provider,
            model,
            session_cost: 0.0,
            session_tokens: 0,
        }
    }

    pub fn update_stats(&mut self, cost: f64, tokens: u32) {
        self.session_cost += cost;
        self.session_tokens += tokens;
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let status_text = if self.session_cost > 0.0 {
            format!(
                "Provider: {} | Model: {} | 💰 ${:.4} | 📊 {} tokens | Ctrl+C to quit",
                self.provider, self.model, self.session_cost, self.session_tokens
            )
        } else {
            format!(
                "Provider: {} | Model: {} | Type your message and press Enter | Ctrl+C to quit",
                self.provider, self.model
            )
        };

        let status = Paragraph::new(status_text)
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: true });

        f.render_widget(status, area);
    }
}
