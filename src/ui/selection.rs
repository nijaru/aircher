use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};
use std::collections::HashMap;

use crate::config::{ConfigManager, ModelConfig};
use crate::providers::ProviderManager;

pub struct SelectionModal {
    visible: bool,
    selected_tab: usize,
    selected_provider: usize,
    selected_model: usize,
    providers: Vec<String>,
    models: HashMap<String, Vec<ModelConfig>>,
}

impl SelectionModal {
    pub fn new(providers: &ProviderManager, config: &ConfigManager) -> Self {
        let provider_names = providers.list_providers();
        let mut models = HashMap::new();
        
        // Get models for each provider
        for provider_name in &provider_names {
            if let Some(provider_config) = config.providers.get(provider_name) {
                models.insert(provider_name.clone(), provider_config.models.clone());
            }
        }

        Self {
            visible: false,
            selected_tab: 0,
            selected_provider: 0,
            selected_model: 0,
            providers: provider_names,
            models,
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
        match self.selected_tab {
            0 => {
                // Provider selection
                if self.selected_provider > 0 {
                    self.selected_provider -= 1;
                }
            }
            1 => {
                // Model selection
                if self.selected_model > 0 {
                    self.selected_model -= 1;
                }
            }
            _ => {}
        }
    }

    pub fn move_down(&mut self) {
        match self.selected_tab {
            0 => {
                // Provider selection
                if self.selected_provider < self.providers.len().saturating_sub(1) {
                    self.selected_provider += 1;
                }
            }
            1 => {
                // Model selection
                if let Some(current_provider) = self.providers.get(self.selected_provider) {
                    if let Some(models) = self.models.get(current_provider) {
                        if self.selected_model < models.len().saturating_sub(1) {
                            self.selected_model += 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn move_left(&mut self) {
        if self.selected_tab > 0 {
            self.selected_tab -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.selected_tab < 1 {
            self.selected_tab += 1;
        }
    }

    pub fn get_selected_provider(&self) -> Option<&str> {
        self.providers.get(self.selected_provider).map(|s| s.as_str())
    }

    pub fn get_selected_model(&self) -> Option<&str> {
        if let Some(current_provider) = self.providers.get(self.selected_provider) {
            if let Some(models) = self.models.get(current_provider) {
                return models.get(self.selected_model).map(|m| m.name.as_str());
            }
        }
        None
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate modal area (centered, larger)
        let modal_area = centered_rect(80, 70, area);

        // Clear the area
        f.render_widget(Clear, modal_area);

        // Render modal background
        let modal_block = Block::default()
            .borders(Borders::ALL)
            .title("Provider & Model Selection")
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
        let tab_titles = vec!["Providers", "Models"];
        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::BOTTOM))
            .highlight_style(Style::default().fg(Color::Blue))
            .select(self.selected_tab);
        f.render_widget(tabs, chunks[0]);

        // Content based on selected tab
        match self.selected_tab {
            0 => self.render_providers(f, chunks[1]),
            1 => self.render_models(f, chunks[1]),
            _ => {}
        }

        // Instructions
        let instructions = match self.selected_tab {
            0 => "←/→ Switch tabs, ↑/↓ Navigate, Enter to select, Tab to confirm, Esc to cancel",
            1 => "←/→ Switch tabs, ↑/↓ Navigate, Enter to select, Tab to confirm, Esc to cancel",
            _ => "",
        };
        
        let instructions_widget = Paragraph::new(instructions)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(instructions_widget, chunks[2]);
    }

    fn render_providers(&self, f: &mut Frame, area: Rect) {
        let providers: Vec<ListItem> = self
            .providers
            .iter()
            .enumerate()
            .map(|(i, provider)| {
                let style = if i == self.selected_provider {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                let icon = match provider.as_str() {
                    "claude" => "🤖",
                    "gemini" => "⭐",
                    "openrouter" => "🌐",
                    _ => "📡",
                };

                ListItem::new(format!("  {} {}", icon, provider)).style(style)
            })
            .collect();

        let providers_list = List::new(providers)
            .block(Block::default().borders(Borders::ALL).title("Available Providers"));

        f.render_widget(providers_list, area);
    }

    fn render_models(&self, f: &mut Frame, area: Rect) {
        if let Some(current_provider) = self.providers.get(self.selected_provider) {
            if let Some(models) = self.models.get(current_provider) {
                let model_items: Vec<ListItem> = models
                    .iter()
                    .enumerate()
                    .map(|(i, model)| {
                        let style = if i == self.selected_model {
                            Style::default().bg(Color::Blue).fg(Color::White)
                        } else {
                            Style::default()
                        };

                        let cost_info = format!(
                            "  {} ({}k ctx, ${:.2}/$1M input)",
                            model.name,
                            model.context_window / 1000,
                            model.input_cost_per_1m
                        );

                        ListItem::new(cost_info).style(style)
                    })
                    .collect();

                let models_list = List::new(model_items)
                    .block(Block::default().borders(Borders::ALL).title(format!("Models for {}", current_provider)));

                f.render_widget(models_list, area);
            } else {
                let no_models = Paragraph::new("No models configured for this provider")
                    .style(Style::default().fg(Color::Red))
                    .block(Block::default().borders(Borders::ALL).title("Models"));
                f.render_widget(no_models, area);
            }
        }
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