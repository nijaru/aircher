use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub struct ModelSelectionModal {
    visible: bool,
    selected_index: usize,
    providers: Vec<String>,
    models: Vec<String>,
}

impl ModelSelectionModal {
    pub fn new() -> Self {
        Self {
            visible: false,
            selected_index: 0,
            providers: vec![
                "claude".to_string(),
                "gemini".to_string(),
                "openrouter".to_string(),
            ],
            models: vec![
                "claude-3-5-sonnet-20241022".to_string(),
                "claude-3-haiku-20240307".to_string(),
                "gemini-2.0-flash-exp".to_string(),
                "gemini-1.5-pro".to_string(),
            ],
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index < self.models.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn get_selected_model(&self) -> &str {
        &self.models[self.selected_index]
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate modal area (centered)
        let modal_area = centered_rect(60, 50, area);

        // Clear the area
        f.render_widget(Clear, modal_area);

        // Render modal background
        let modal_block = Block::default()
            .borders(Borders::ALL)
            .title("Select AI Model")
            .style(Style::default().bg(Color::Black));

        f.render_widget(modal_block, modal_area);

        // Inner area
        let inner_area = Rect {
            x: modal_area.x + 1,
            y: modal_area.y + 1,
            width: modal_area.width.saturating_sub(2),
            height: modal_area.height.saturating_sub(2),
        };

        // Split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Provider selection
                Constraint::Min(0),    // Model list
                Constraint::Length(2), // Instructions
            ])
            .split(inner_area);

        // Provider selection
        let provider_text = format!("Provider: {}", self.providers[0]);
        let provider_widget = Paragraph::new(provider_text)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(provider_widget, chunks[0]);

        // Model list
        let models: Vec<ListItem> = self
            .models
            .iter()
            .enumerate()
            .map(|(i, model)| {
                let style = if i == self.selected_index {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                ListItem::new(format!("  {}", model)).style(style)
            })
            .collect();

        let models_list = List::new(models).block(Block::default().borders(Borders::NONE));

        f.render_widget(models_list, chunks[1]);

        // Instructions
        let instructions = Paragraph::new("↑/↓ to navigate, Enter to select, Esc to cancel")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(instructions, chunks[2]);
    }
}

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
