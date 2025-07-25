use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub struct TypeaheadOverlay {
    pub visible: bool,
    pub title: String,
    pub description: String,
    pub items: Vec<TypeaheadItem>,
    pub filtered_items: Vec<TypeaheadItem>,
    pub input: String,
    pub cursor_position: usize,
    pub selected_index: usize,
    pub current_value: Option<String>,
}

#[derive(Clone, Debug)]
pub struct TypeaheadItem {
    pub label: String,
    pub value: String,
    pub description: Option<String>,
    pub available: bool,
}

impl TypeaheadOverlay {
    pub fn new(title: String, description: String) -> Self {
        Self {
            visible: false,
            title,
            description,
            items: Vec::new(),
            filtered_items: Vec::new(),
            input: String::new(),
            cursor_position: 0,
            selected_index: 0,
            current_value: None,
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.filter_items();
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.input.clear();
        self.cursor_position = 0;
        self.selected_index = 0;
    }

    pub fn set_items(&mut self, items: Vec<TypeaheadItem>) {
        self.items = items;
        self.filter_items();
    }

    pub fn set_current_value(&mut self, value: Option<String>) {
        self.current_value = value;
        self.filter_items();
    }

    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
        self.filter_items();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
            self.filter_items();
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

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if !self.filtered_items.is_empty() {
            self.selected_index = self.filtered_items.len() - 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if !self.filtered_items.is_empty() {
            if self.selected_index < self.filtered_items.len() - 1 {
                self.selected_index += 1;
            } else {
                self.selected_index = 0;
            }
        }
    }

    pub fn get_selected(&self) -> Option<&TypeaheadItem> {
        self.filtered_items.get(self.selected_index)
    }

    fn filter_items(&mut self) {
        let query = self.input.to_lowercase();
        
        // First, separate current item and other items
        let (current_items, other_items): (Vec<_>, Vec<_>) = self.items.iter()
            .cloned()
            .partition(|item| {
                self.current_value.as_ref().map_or(false, |cv| cv == &item.value)
            });
        
        // Filter other items
        let mut filtered: Vec<_> = other_items.into_iter()
            .filter(|item| {
                item.label.to_lowercase().contains(&query) ||
                item.value.to_lowercase().contains(&query) ||
                item.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query))
            })
            .collect();
        
        // Sort by relevance (items starting with query first)
        filtered.sort_by(|a, b| {
            let a_starts = a.label.to_lowercase().starts_with(&query);
            let b_starts = b.label.to_lowercase().starts_with(&query);
            match (a_starts, b_starts) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.label.cmp(&b.label),
            }
        });
        
        // Put current item at top if it matches
        self.filtered_items = current_items.into_iter()
            .filter(|item| {
                query.is_empty() || 
                item.label.to_lowercase().contains(&query) ||
                item.value.to_lowercase().contains(&query)
            })
            .chain(filtered)
            .take(10) // Limit to 10 items
            .collect();
        
        // Reset selection if needed
        if self.selected_index >= self.filtered_items.len() {
            self.selected_index = 0;
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate overlay size
        let width = 60.min(area.width - 4);
        let height = 20.min(area.height - 4);
        
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;
        
        let overlay_area = Rect::new(x, y, width, height);

        // Clear the area
        f.render_widget(Clear, overlay_area);

        // Create the main block
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray));

        let inner = block.inner(overlay_area);
        f.render_widget(block, overlay_area);

        // Layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Description
                Constraint::Length(3), // Input
                Constraint::Min(5),    // List
                Constraint::Length(1), // Help
            ])
            .split(inner);

        // Description
        let desc_paragraph = Paragraph::new(self.description.as_str())
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Left);
        f.render_widget(desc_paragraph, chunks[0]);

        // Input field
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));
        
        let input_area = input_block.inner(chunks[1]);
        f.render_widget(input_block, chunks[1]);
        
        let input_text = Paragraph::new(self.input.as_str())
            .style(Style::default().fg(Color::White));
        f.render_widget(input_text, input_area);

        // Show cursor
        f.set_cursor_position((
            input_area.x + self.cursor_position as u16,
            input_area.y
        ));

        // Filtered items list
        let items: Vec<ListItem> = self.filtered_items.iter().enumerate().map(|(i, item)| {
            let mut spans = vec![];
            
            // Add checkmark for current value
            if self.current_value.as_ref().map_or(false, |cv| cv == &item.value) {
                spans.push(Span::styled("✓ ", Style::default().fg(Color::Green)));
            } else {
                spans.push(Span::raw("  "));
            }
            
            // Add label
            let label_style = if item.available {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            };
            spans.push(Span::styled(&item.label, label_style));
            
            // Note: Availability status is already shown in the label with icons
            
            // Add description if present
            if let Some(desc) = &item.description {
                spans.push(Span::styled(
                    format!(" - {}", desc), 
                    Style::default().fg(Color::DarkGray)
                ));
            }
            
            let style = if i == self.selected_index {
                Style::default().fg(Color::White).bg(Color::Blue).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            ListItem::new(Line::from(spans)).style(style)
        }).collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(list, chunks[2]);

        // Help text
        let help = if self.filtered_items.is_empty() {
            "No matches found. Press Esc to cancel."
        } else {
            "↑/↓ Navigate • Enter Select • Esc Cancel"
        };
        let help_text = Paragraph::new(help)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(help_text, chunks[3]);
    }
}