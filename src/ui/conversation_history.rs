use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use chrono::{DateTime, Utc};

use crate::providers::{Message, MessageRole};

pub struct ConversationHistoryModal {
    visible: bool,
    messages: Vec<DisplayMessage>,
    selected_index: usize,
    list_state: ListState,
    show_details: bool,
}

#[derive(Debug, Clone)]
struct DisplayMessage {
    index: usize,
    role: MessageRole,
    preview: String,
    full_content: String,
    timestamp: DateTime<Utc>,
    is_current: bool,
    is_compacted: bool,
}

impl ConversationHistoryModal {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Self {
            visible: false,
            messages: Vec::new(),
            selected_index: 0,
            list_state,
            show_details: false,
        }
    }

    pub fn show(&mut self, messages: &[Message], current_position: usize) {
        self.visible = true;
        self.show_details = false;
        self.build_display_messages(messages, current_position);
        
        // Find the display index that corresponds to the current position
        let display_index = self.messages.iter()
            .position(|msg| msg.index == current_position)
            .unwrap_or(self.messages.len().saturating_sub(1));
        
        self.selected_index = display_index;
        self.list_state.select(Some(self.selected_index));
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.messages.clear();
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    fn build_display_messages(&mut self, messages: &[Message], current_position: usize) {
        self.messages.clear();
        
        for (index, message) in messages.iter().enumerate() {
            // Skip system messages and tool calls for cleaner display
            if matches!(message.role, MessageRole::System | MessageRole::Tool) {
                continue;
            }

            let preview = if message.content.len() > 80 {
                format!("{}...", &message.content[..77])
            } else {
                message.content.clone()
            };

            self.messages.push(DisplayMessage {
                index, // Keep the original message index for correct forking
                role: message.role.clone(),
                preview,
                full_content: message.content.clone(),
                timestamp: message.timestamp,
                is_current: index == current_position,
                is_compacted: false, // TODO: Implement compaction detection
            });
        }
    }

    pub fn move_up(&mut self) {
        if !self.messages.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index < self.messages.len().saturating_sub(1) {
            self.selected_index += 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }

    pub fn get_selected_message_index(&self) -> Option<usize> {
        if self.messages.is_empty() {
            None
        } else {
            Some(self.messages[self.selected_index].index)
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate modal size (60% width, 70% height, centered)
        let popup_area = Self::centered_rect(60, 70, area);

        // Clear background
        f.render_widget(Clear, popup_area);

        // Main layout: title + content + footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(popup_area);

        // Title
        let title = if self.show_details {
            " Message Details (ESC ESC) "
        } else {
            " Jump to previous message (ESC ESC) "
        };
        
        let title_block = Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title(title)
            .border_style(Style::default().fg(Color::Cyan));
        
        let subtitle = Paragraph::new("This will fork the conversation from the selected point")
            .block(title_block)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(subtitle, chunks[0]);

        // Content area
        if self.show_details {
            self.render_details(f, chunks[1]);
        } else {
            self.render_message_list(f, chunks[1]);
        }

        // Footer with controls
        let footer_text = if self.show_details {
            "↑/↓ navigate • Tab list view • ESC cancel"
        } else {
            "↑/↓ navigate • Enter fork • Tab details • ESC cancel"
        };
        
        let footer = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(footer, chunks[2]);
    }

    fn render_message_list(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.messages
            .iter()
            .enumerate()
            .map(|(i, msg)| {
                let role_icon = match msg.role {
                    MessageRole::User => "👤",
                    MessageRole::Assistant => "🤖", 
                    MessageRole::System => "⚙️",
                    MessageRole::Tool => "🔧",
                };

                let prefix = if msg.is_current {
                    format!("  {} (current)", msg.index + 1)
                } else {
                    format!("  {}", msg.index + 1)
                };

                let style = if i == self.selected_index {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else if msg.is_current {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };

                let content = if msg.is_compacted {
                    format!("{} {} > [Compacted: multiple messages]", prefix, "📋")
                } else {
                    format!("{} {} > {}", prefix, role_icon, msg.preview)
                };

                ListItem::new(Line::from(Span::styled(content, style)))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::LEFT | Borders::RIGHT))
            .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_stateful_widget(list, area, &mut self.list_state.clone());
    }

    fn render_details(&self, f: &mut Frame, area: Rect) {
        if let Some(msg) = self.messages.get(self.selected_index) {
            let role_name = match msg.role {
                MessageRole::User => "User",
                MessageRole::Assistant => "Assistant",
                MessageRole::System => "System",
                MessageRole::Tool => "Tool",
            };

            let details = format!(
                "Message {}\nRole: {}\nTimestamp: {}\n\n{}",
                msg.index + 1,
                role_name,
                msg.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                msg.full_content
            );

            let paragraph = Paragraph::new(details)
                .block(Block::default().borders(Borders::LEFT | Borders::RIGHT))
                .style(Style::default().fg(Color::White))
                .wrap(ratatui::widgets::Wrap { trim: true });

            f.render_widget(paragraph, area);
        }
    }

    // Helper function to create centered rectangle
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

impl Default for ConversationHistoryModal {
    fn default() -> Self {
        Self::new()
    }
}