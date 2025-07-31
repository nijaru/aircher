use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::config::ConfigManager;

pub struct SettingsModal {
    visible: bool,
    selected_setting: usize,
    config: ConfigManager,
    editing_field: Option<String>,
    edit_buffer: String,
}

impl SettingsModal {
    pub fn new(config: &ConfigManager) -> Self {
        Self {
            visible: false,
            selected_setting: 0,
            config: config.clone(),
            editing_field: None,
            edit_buffer: String::new(),
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
        if self.selected_setting > 0 {
            self.selected_setting -= 1;
        }
    }

    pub fn move_down(&mut self) {
        let max_items: usize = 5; // Number of UI settings
        
        if self.selected_setting < max_items.saturating_sub(1) {
            self.selected_setting += 1;
        }
    }

    pub fn move_left(&mut self) {
        // No tabs anymore, so this is a no-op
    }

    pub fn move_right(&mut self) {
        // No tabs anymore, so this is a no-op
    }

    pub fn start_editing(&mut self) {
        if self.editing_field.is_none() {
            // Only UI Settings now
            match self.selected_setting {
                0 => {
                    self.editing_field = Some("theme".to_string());
                    self.edit_buffer = self.config.ui.theme.clone();
                }
                1 => {
                    self.editing_field = Some("refresh_rate".to_string());
                    self.edit_buffer = self.config.ui.refresh_rate_ms.to_string();
                }
                _ => {}
            }
        }
    }

    pub fn finish_editing(&mut self) {
        if let Some(field) = &self.editing_field {
            match field.as_str() {
                "theme" => {
                    self.config.ui.theme = self.edit_buffer.clone();
                }
                "refresh_rate" => {
                    if let Ok(rate) = self.edit_buffer.parse::<u64>() {
                        self.config.ui.refresh_rate_ms = rate;
                    }
                }
                _ => {}
            }
        }
        
        self.editing_field = None;
        self.edit_buffer.clear();
    }

    pub fn cancel_editing(&mut self) {
        self.editing_field = None;
        self.edit_buffer.clear();
    }

    pub fn add_char(&mut self, c: char) {
        if self.editing_field.is_some() {
            self.edit_buffer.push(c);
        }
    }

    pub fn remove_char(&mut self) {
        if self.editing_field.is_some() {
            self.edit_buffer.pop();
        }
    }

    pub fn is_editing(&self) -> bool {
        self.editing_field.is_some()
    }

    pub fn get_config(&self) -> &ConfigManager {
        &self.config
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate modal area (centered, smaller since we only have UI settings now)
        let modal_area = centered_rect(60, 50, area);

        // Clear the area
        f.render_widget(Clear, modal_area);

        // Render modal background
        let modal_block = Block::default()
            .borders(Borders::ALL)
            .title("Settings")
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
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Instructions
            ])
            .split(inner_area);

        // Render UI settings (only content now)
        self.render_ui_settings(f, chunks[0]);

        // Instructions
        let instructions = if self.editing_field.is_some() {
            "Type to edit, Enter to save, Esc to cancel"
        } else {
            "↑/↓ Navigate, Enter to edit, S to save config, Esc or Ctrl+C to close"
        };
        
        let instructions_widget = Paragraph::new(instructions)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(instructions_widget, chunks[1]);
    }

    fn render_ui_settings(&self, f: &mut Frame, area: Rect) {
        let settings = vec![
            ("Theme", self.config.ui.theme.clone()),
            ("Refresh Rate (ms)", self.config.ui.refresh_rate_ms.to_string()),
            ("Enable Mouse", self.config.ui.enable_mouse.to_string()),
            ("Show Token Count", self.config.ui.show_token_count.to_string()),
            ("Show Cost Estimate", self.config.ui.show_cost_estimate.to_string()),
        ];

        let setting_items: Vec<ListItem> = settings
            .iter()
            .enumerate()
            .map(|(i, (name, value))| {
                let is_editing = self.editing_field.as_ref()
                    .map(|field| match i {
                        0 => field == "theme",
                        1 => field == "refresh_rate",
                        _ => false,
                    })
                    .unwrap_or(false);

                let display_value = if is_editing {
                    &self.edit_buffer
                } else {
                    value
                };

                let line = Line::from(vec![
                    Span::styled(format!("  {}: ", name), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled(display_value, Style::default().fg(Color::Gray)),
                ]);

                let mut item = ListItem::new(line);
                if i == self.selected_setting {
                    item = item.style(Style::default().bg(Color::DarkGray));
                }
                item
            })
            .collect();

        let settings_list = List::new(setting_items)
            .block(Block::default().borders(Borders::ALL).title("UI Settings"));

        f.render_widget(settings_list, area);
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