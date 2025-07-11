use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub struct HelpModal {
    visible: bool,
    scroll_offset: usize,
}

impl HelpModal {
    pub fn new() -> Self {
        Self {
            visible: false,
            scroll_offset: 0,
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

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate modal area (centered, large)
        let modal_area = centered_rect(80, 85, area);

        // Clear the area
        f.render_widget(Clear, modal_area);

        // Render modal background
        let modal_block = Block::default()
            .borders(Borders::ALL)
            .title("🏹 Aircher Help")
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
                Constraint::Length(2), // Instructions
            ])
            .split(inner_area);

        // Render help content
        self.render_help_content(f, chunks[0]);

        // Instructions
        let instructions = Paragraph::new("↑/↓ to scroll, Esc to close")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(instructions, chunks[1]);
    }

    fn render_help_content(&self, f: &mut Frame, area: Rect) {
        let help_items = vec![
            ("🎯 BASIC NAVIGATION", ""),
            ("Enter", "Send message"),
            ("Ctrl+C", "Quit application"),
            ("↑/↓", "Scroll chat history"),
            ("", ""),
            ("🔧 MODALS & SETTINGS", ""),
            ("Tab", "Open provider/model selection"),
            ("F2", "Open settings panel"),
            ("F1", "Show this help (you are here!)"),
            ("Esc", "Close any modal"),
            ("", ""),
            ("📡 PROVIDER SELECTION", ""),
            ("←/→", "Switch between providers/models tabs"),
            ("↑/↓", "Navigate lists"),
            ("Enter", "Select item"),
            ("", ""),
            ("⚙️ SETTINGS PANEL", ""),
            ("←/→", "Switch between tabs"),
            ("↑/↓", "Navigate settings"),
            ("Enter", "Edit setting"),
            ("S", "Save configuration"),
            ("", ""),
            ("🤖 SUPPORTED PROVIDERS", ""),
            ("Claude", "Anthropic's Claude models (API key required)"),
            ("Gemini", "Google's Gemini models (API key required)"),
            ("OpenRouter", "Multi-provider hub (API key required)"),
            ("", ""),
            ("💰 COST TRACKING", ""),
            ("", "Real-time cost and token usage displayed"),
            ("", "Set budget limits in settings"),
            ("", "Cost warnings when approaching limits"),
            ("", ""),
            ("🎨 INTERFACE FEATURES", ""),
            ("", "• Responsive terminal UI"),
            ("", "• Real-time streaming responses"),
            ("", "• Conversation history"),
            ("", "• Model/provider switching"),
            ("", "• Cost and token tracking"),
            ("", "• Configuration persistence"),
            ("", ""),
            ("🔑 API KEY SETUP", ""),
            ("", "Set environment variables:"),
            ("ANTHROPIC_API_KEY", "For Claude models"),
            ("GOOGLE_API_KEY", "For Gemini models"),
            ("OPENROUTER_API_KEY", "For OpenRouter access"),
            ("", ""),
            ("", "Or use the Settings panel (F2) to configure"),
            ("", ""),
            ("📚 TIPS & TRICKS", ""),
            ("", "• Use Tab to quickly switch models"),
            ("", "• Check cost estimates before sending"),
            ("", "• Set budget limits to control spending"),
            ("", "• Use ↑/↓ to review conversation history"),
            ("", "• Press F1 anytime for help"),
            ("", ""),
            ("🏹 About Aircher", ""),
            ("", "Advanced AI terminal assistant"),
            ("", "Built with Rust and Ratatui"),
            ("", "Version 0.1.0-dev"),
        ];

        let items: Vec<ListItem> = help_items
            .iter()
            .enumerate()
            .map(|(i, (key, description))| {
                if key.starts_with("🎯") || key.starts_with("🔧") || key.starts_with("📡") || 
                   key.starts_with("⚙️") || key.starts_with("🤖") || key.starts_with("💰") || 
                   key.starts_with("🎨") || key.starts_with("🔑") || key.starts_with("📚") || 
                   key.starts_with("🏹") {
                    // Section headers
                    ListItem::new(Line::from(vec![
                        Span::styled(*key, Style::default().fg(Color::Yellow)),
                        Span::raw(" "),
                        Span::styled(*description, Style::default().fg(Color::Yellow)),
                    ]))
                } else if key.is_empty() {
                    // Empty lines
                    ListItem::new("")
                } else if description.is_empty() {
                    // Single column items
                    ListItem::new(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(*key, Style::default().fg(Color::Gray)),
                    ]))
                } else {
                    // Key-value pairs
                    ListItem::new(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(*key, Style::default().fg(Color::Cyan)),
                        Span::raw(": "),
                        Span::styled(*description, Style::default().fg(Color::White)),
                    ]))
                }
            })
            .skip(self.scroll_offset)
            .collect();

        let help_list = List::new(items)
            .block(Block::default().borders(Borders::NONE));

        f.render_widget(help_list, area);
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