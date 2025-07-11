use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};
use std::collections::HashMap;

use crate::config::ConfigManager;

pub struct SettingsModal {
    visible: bool,
    selected_tab: usize,
    selected_setting: usize,
    config: ConfigManager,
    api_keys: HashMap<String, String>,
    editing_field: Option<String>,
    edit_buffer: String,
}

impl SettingsModal {
    pub fn new(config: &ConfigManager) -> Self {
        let mut api_keys = HashMap::new();
        
        // Load API keys from environment (masked for display)
        for (provider_name, provider_config) in &config.providers {
            if let Ok(key) = std::env::var(&provider_config.api_key_env) {
                let masked = if key.len() > 8 {
                    format!("{}...{}", &key[..4], &key[key.len()-4..])
                } else {
                    "*".repeat(key.len())
                };
                api_keys.insert(provider_name.clone(), masked);
            } else {
                api_keys.insert(provider_name.clone(), "Not set".to_string());
            }
        }

        Self {
            visible: false,
            selected_tab: 0,
            selected_setting: 0,
            config: config.clone(),
            api_keys,
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
        let max_items = match self.selected_tab {
            0 => self.api_keys.len(),
            1 => 6, // Number of UI settings
            2 => 3, // Number of budget settings
            _ => 0,
        };
        
        if self.selected_setting < max_items.saturating_sub(1) {
            self.selected_setting += 1;
        }
    }

    pub fn move_left(&mut self) {
        if self.selected_tab > 0 {
            self.selected_tab -= 1;
            self.selected_setting = 0;
        }
    }

    pub fn move_right(&mut self) {
        if self.selected_tab < 2 {
            self.selected_tab += 1;
            self.selected_setting = 0;
        }
    }

    pub fn start_editing(&mut self) {
        if self.editing_field.is_none() {
            match self.selected_tab {
                0 => {
                    // API Keys
                    let providers: Vec<String> = self.api_keys.keys().cloned().collect();
                    if let Some(provider) = providers.get(self.selected_setting) {
                        self.editing_field = Some(format!("api_key_{}", provider));
                        self.edit_buffer.clear();
                    }
                }
                1 => {
                    // UI Settings
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
                2 => {
                    // Budget Settings
                    match self.selected_setting {
                        0 => {
                            self.editing_field = Some("budget_limit".to_string());
                            self.edit_buffer = self.config.global.budget_limit
                                .map(|b| b.to_string())
                                .unwrap_or_else(|| "0.0".to_string());
                        }
                        1 => {
                            self.editing_field = Some("max_context_tokens".to_string());
                            self.edit_buffer = self.config.global.max_context_tokens.to_string();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    pub fn finish_editing(&mut self) {
        if let Some(field) = &self.editing_field {
            match field.as_str() {
                field if field.starts_with("api_key_") => {
                    let provider = field.strip_prefix("api_key_").unwrap();
                    if let Some(provider_config) = self.config.providers.get(provider) {
                        std::env::set_var(&provider_config.api_key_env, &self.edit_buffer);
                        let masked = if self.edit_buffer.len() > 8 {
                            format!("{}...{}", &self.edit_buffer[..4], &self.edit_buffer[self.edit_buffer.len()-4..])
                        } else {
                            "*".repeat(self.edit_buffer.len())
                        };
                        self.api_keys.insert(provider.to_string(), masked);
                    }
                }
                "theme" => {
                    self.config.ui.theme = self.edit_buffer.clone();
                }
                "refresh_rate" => {
                    if let Ok(rate) = self.edit_buffer.parse::<u64>() {
                        self.config.ui.refresh_rate_ms = rate;
                    }
                }
                "budget_limit" => {
                    if let Ok(limit) = self.edit_buffer.parse::<f64>() {
                        self.config.global.budget_limit = Some(limit);
                    }
                }
                "max_context_tokens" => {
                    if let Ok(tokens) = self.edit_buffer.parse::<u32>() {
                        self.config.global.max_context_tokens = tokens;
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

        // Calculate modal area (centered, large)
        let modal_area = centered_rect(90, 80, area);

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
                Constraint::Length(3), // Tab selection
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Instructions
            ])
            .split(inner_area);

        // Tab selection
        let tab_titles = vec!["API Keys", "UI Settings", "Budget"];
        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::BOTTOM))
            .highlight_style(Style::default().fg(Color::Blue))
            .select(self.selected_tab);
        f.render_widget(tabs, chunks[0]);

        // Content based on selected tab
        match self.selected_tab {
            0 => self.render_api_keys(f, chunks[1]),
            1 => self.render_ui_settings(f, chunks[1]),
            2 => self.render_budget_settings(f, chunks[1]),
            _ => {}
        }

        // Instructions
        let instructions = if self.editing_field.is_some() {
            "Type to edit, Enter to save, Esc to cancel"
        } else {
            "←/→ Switch tabs, ↑/↓ Navigate, Enter to edit, S to save config, Esc to close"
        };
        
        let instructions_widget = Paragraph::new(instructions)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(instructions_widget, chunks[2]);
    }

    fn render_api_keys(&self, f: &mut Frame, area: Rect) {
        let key_items: Vec<ListItem> = self
            .api_keys
            .iter()
            .enumerate()
            .map(|(i, (provider, key))| {
                let style = if i == self.selected_setting {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                let is_editing = self.editing_field.as_ref()
                    .map(|field| field == &format!("api_key_{}", provider))
                    .unwrap_or(false);

                let display_value = if is_editing {
                    &self.edit_buffer
                } else {
                    key
                };

                let icon = match provider.as_str() {
                    "claude" => "🤖",
                    "gemini" => "⭐",
                    "openrouter" => "🌐",
                    _ => "📡",
                };

                ListItem::new(format!("  {} {}: {}", icon, provider, display_value)).style(style)
            })
            .collect();

        let keys_list = List::new(key_items)
            .block(Block::default().borders(Borders::ALL).title("API Keys"));

        f.render_widget(keys_list, area);
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
                let style = if i == self.selected_setting {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

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

                ListItem::new(format!("  {}: {}", name, display_value)).style(style)
            })
            .collect();

        let settings_list = List::new(setting_items)
            .block(Block::default().borders(Borders::ALL).title("UI Settings"));

        f.render_widget(settings_list, area);
    }

    fn render_budget_settings(&self, f: &mut Frame, area: Rect) {
        let budget_limit = self.config.global.budget_limit
            .map(|b| format!("${:.2}", b))
            .unwrap_or_else(|| "No limit".to_string());

        let settings = vec![
            ("Budget Limit", budget_limit),
            ("Max Context Tokens", self.config.global.max_context_tokens.to_string()),
            ("Default Provider", self.config.global.default_provider.clone()),
        ];

        let setting_items: Vec<ListItem> = settings
            .iter()
            .enumerate()
            .map(|(i, (name, value))| {
                let style = if i == self.selected_setting {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                let is_editing = self.editing_field.as_ref()
                    .map(|field| match i {
                        0 => field == "budget_limit",
                        1 => field == "max_context_tokens",
                        _ => false,
                    })
                    .unwrap_or(false);

                let display_value = if is_editing {
                    &self.edit_buffer
                } else {
                    value
                };

                ListItem::new(format!("  {}: {}", name, display_value)).style(style)
            })
            .collect();

        let settings_list = List::new(setting_items)
            .block(Block::default().borders(Borders::ALL).title("Budget & Performance"));

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