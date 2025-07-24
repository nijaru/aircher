use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use chrono::{DateTime, Local};

use crate::sessions::Session;

pub struct SessionBrowser {
    visible: bool,
    sessions: Vec<Session>,
    filtered_sessions: Vec<Session>,
    selected_index: usize,
    list_state: ListState,
    filter: String,
    loading: bool,
    error: Option<String>,
}

impl SessionBrowser {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Self {
            visible: false,
            sessions: Vec::new(),
            filtered_sessions: Vec::new(),
            selected_index: 0,
            list_state,
            filter: String::new(),
            loading: false,
            error: None,
        }
    }

    pub fn set_sessions(&mut self, sessions: Vec<Session>) {
        self.sessions = sessions;
        self.apply_filter();
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.loading = true;
        self.error = None;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.loading = false;
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn move_up(&mut self) {
        if self.filtered_sessions.is_empty() {
            return;
        }
        
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    pub fn move_down(&mut self) {
        if self.filtered_sessions.is_empty() {
            return;
        }
        
        if self.selected_index < self.filtered_sessions.len() - 1 {
            self.selected_index += 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    pub fn get_selected(&self) -> Option<&Session> {
        if self.filtered_sessions.is_empty() {
            None
        } else {
            self.filtered_sessions.get(self.selected_index)
        }
    }

    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
        self.apply_filter();
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }

    pub fn get_filter(&self) -> &str {
        &self.filter
    }

    fn apply_filter(&mut self) {
        if self.filter.is_empty() {
            self.filtered_sessions = self.sessions.clone();
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.filtered_sessions = self.sessions
                .iter()
                .filter(|session| {
                    session.title.to_lowercase().contains(&filter_lower) ||
                    session.model.to_lowercase().contains(&filter_lower) ||
                    session.provider.to_lowercase().contains(&filter_lower) ||
                    session.description.as_ref()
                        .map(|desc| desc.to_lowercase().contains(&filter_lower))
                        .unwrap_or(false)
                })
                .cloned()
                .collect();
        }
    }

    fn format_session_item(session: &Session) -> ListItem {
        let local_time: DateTime<Local> = session.updated_at.with_timezone(&Local);
        let time_str = local_time.format("%b %d %H:%M").to_string();
        
        let cost_str = if session.total_cost > 0.0 {
            format!(" ${:.4}", session.total_cost)
        } else {
            String::new()
        };
        
        let title = if session.title.len() > 50 {
            format!("{}...", &session.title[..47])
        } else {
            session.title.clone()
        };
        
        let line = Line::from(vec![
            Span::styled(title, Style::default().fg(Color::White)),
            Span::raw(" "),
            Span::styled(
                format!("({})", session.model),
                Style::default().fg(Color::DarkGray)
            ),
            Span::styled(cost_str, Style::default().fg(Color::Yellow)),
            Span::raw(" - "),
            Span::styled(time_str, Style::default().fg(Color::DarkGray)),
        ]);
        
        ListItem::new(line)
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate centered area (80% width, 70% height)
        let width = (area.width as f32 * 0.8) as u16;
        let height = (area.height as f32 * 0.7) as u16;
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;
        
        let popup_area = Rect {
            x,
            y,
            width,
            height,
        };

        // Clear background
        f.render_widget(Clear, popup_area);

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3), // Search box
                Constraint::Min(5),    // Session list
                Constraint::Length(3), // Help text
            ])
            .split(popup_area);

        // Main block
        let block = Block::default()
            .title(" Browse Sessions ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta));
        f.render_widget(block, popup_area);

        // Inner area for content
        let inner = Block::default().borders(Borders::ALL).inner(popup_area);
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3), // Search box
                Constraint::Min(5),    // Session list
                Constraint::Length(2), // Help text
            ])
            .split(inner);

        // Search box
        let search_block = Block::default()
            .title("Filter")
            .borders(Borders::ALL);
        let search_text = Paragraph::new(self.filter.as_str())
            .block(search_block);
        f.render_widget(search_text, inner_chunks[0]);

        // Session list or loading/error state
        if self.loading {
            let loading_text = Paragraph::new("Loading sessions...")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(loading_text, inner_chunks[1]);
        } else if let Some(error) = &self.error {
            let error_text = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(error_text, inner_chunks[1]);
        } else if self.filtered_sessions.is_empty() {
            let empty_text = if self.sessions.is_empty() {
                "No sessions found"
            } else {
                "No sessions match filter"
            };
            let empty_paragraph = Paragraph::new(empty_text)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(empty_paragraph, inner_chunks[1]);
        } else {
            // Create session list
            let items: Vec<ListItem> = self.filtered_sessions
                .iter()
                .map(Self::format_session_item)
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::NONE))
                .highlight_style(
                    Style::default()
                        .bg(Color::Rgb(60, 60, 60))
                        .add_modifier(Modifier::BOLD)
                )
                .highlight_symbol("▶ ");

            f.render_stateful_widget(list, inner_chunks[1], &mut self.list_state);
        }

        // Help text
        let help_text = Paragraph::new(
            "↑/↓: Navigate  Enter: Load  /: Filter  Esc: Cancel"
        )
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
        f.render_widget(help_text, inner_chunks[2]);
    }
}