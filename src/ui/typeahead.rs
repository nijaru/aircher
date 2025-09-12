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
    pub hide_input: bool,  // Hide input for simple selection
}

#[derive(Clone, Debug)]
pub struct TypeaheadItem {
    pub label: String,
    pub value: String,
    pub description: Option<String>,
    pub available: bool,
    // Rich metadata for enhanced model selection
    pub metadata: Option<TypeaheadMetadata>,
}

#[derive(Clone, Debug)]
pub struct TypeaheadMetadata {
    pub context_window: Option<u32>,    // Context window size (e.g., 200000)
    pub pricing: Option<ModelPricing>,  // Input/output pricing
    pub capabilities: Vec<String>,      // ["tools", "streaming", etc.]
    pub provider_info: Option<String>,  // Additional provider-specific info
    pub is_default: bool,              // ⭐ Default/recommended model
    pub is_large_context: bool,        // 🧠 Large context models (1M+ tokens)
}

#[derive(Clone, Debug)]
pub struct ModelPricing {
    pub input_cost: f64,    // Cost per 1K input tokens
    pub output_cost: f64,   // Cost per 1K output tokens  
    pub currency: String,   // "USD"
}

impl TypeaheadItem {
    /// Create a simple item without metadata
    pub fn new(label: String, value: String, description: Option<String>, available: bool) -> Self {
        Self {
            label,
            value,
            description,
            available,
            metadata: None,
        }
    }

    /// Create an enhanced item with metadata
    pub fn with_metadata(
        label: String,
        value: String,
        description: Option<String>,
        available: bool,
        metadata: TypeaheadMetadata,
    ) -> Self {
        Self {
            label,
            value,
            description,
            available,
            metadata: Some(metadata),
        }
    }

    /// Format context window for display (e.g., "200k ctx", "2M ctx")
    pub fn format_context_window(&self) -> Option<String> {
        if let Some(metadata) = &self.metadata {
            if let Some(context) = metadata.context_window {
                return Some(if context >= 1_000_000 {
                    format!("{}M ctx", context / 1_000_000)
                } else if context >= 1_000 {
                    format!("{}k ctx", context / 1_000)
                } else {
                    format!("{} ctx", context)
                });
            }
        }
        None
    }

    /// Format pricing for display (e.g., "$3.00⇄$15.00", "Free")
    pub fn format_pricing(&self) -> Option<String> {
        if let Some(metadata) = &self.metadata {
            if let Some(pricing) = &metadata.pricing {
                if pricing.input_cost == 0.0 && pricing.output_cost == 0.0 {
                    return Some("Free".to_string());
                } else {
                    return Some(format!(
                        "${:.2}⇄${:.2}",
                        pricing.input_cost, pricing.output_cost
                    ));
                }
            }
        }
        None
    }

    /// Get capability icons with spacing for better readability
    pub fn get_capability_icons(&self) -> String {
        if let Some(metadata) = &self.metadata {
            let mut icons = Vec::new();
            for capability in &metadata.capabilities {
                match capability.as_str() {
                    "tools" | "function_calling" => icons.push("🔧"),
                    "streaming" => icons.push("⚡"),
                    "vision" => icons.push("👁"),
                    "reasoning" => icons.push("🎯"),
                    "multimodal" => icons.push("🖼"),
                    _ => {}
                }
            }
            return if icons.is_empty() { String::new() } else { icons.join("") };
        }
        String::new()
    }

    /// Get status icons with spacing and priority ordering
    pub fn get_status_icons(&self) -> String {
        if let Some(metadata) = &self.metadata {
            let mut icons = Vec::new();
            if metadata.is_default {
                icons.push("⭐");
            }
            if metadata.is_large_context {
                icons.push("🧠");
            }
            return if icons.is_empty() { String::new() } else { icons.join("") };
        }
        String::new()
    }
    
    /// Get enhanced model quality indicator based on model name and capabilities
    pub fn get_quality_indicator(&self) -> Option<String> {
        if let Some(_metadata) = &self.metadata {
            let name_lower = self.value.to_lowercase();
            
            // Premium/flagship models
            if name_lower.contains("claude-3-5-sonnet") || name_lower.contains("gpt-4o") || 
               name_lower.contains("gemini-2.0-flash-exp") || name_lower.contains("o1-pro") {
                return Some("🏆".to_string()); // Premium quality
            }
            
            // High-quality models
            if name_lower.contains("claude-3-opus") || name_lower.contains("gpt-4") || 
               name_lower.contains("gemini-pro") || name_lower.contains("deepseek-r1") {
                return Some("💎".to_string()); // High quality
            }
            
            // Fast/efficient models
            if name_lower.contains("haiku") || name_lower.contains("turbo") || 
               name_lower.contains("flash") || name_lower.contains("mini") {
                return Some("🚀".to_string()); // Fast/efficient
            }
            
            // Local models
            if name_lower.contains("llama") || name_lower.contains("qwen") || 
               name_lower.contains("mistral") || name_lower.contains("phi") {
                return Some("💾".to_string()); // Local/private
            }
        }
        None
    }
    
    /// Get cost efficiency rating (💰 for very efficient)
    pub fn get_cost_efficiency(&self) -> Option<String> {
        if let Some(metadata) = &self.metadata {
            if let Some(pricing) = &metadata.pricing {
                let avg_cost = (pricing.input_cost + pricing.output_cost) / 2.0;
                
                if avg_cost == 0.0 {
                    return Some("🎉".to_string()); // Free (party emoji)
                } else if avg_cost < 1.0 {
                    return Some("💰".to_string()); // Very efficient
                }
                // Expensive models get no efficiency indicator
            }
        }
        None
    }
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
            hide_input: false,
        }
    }

    pub fn show(&mut self) {
        use tracing::debug;
        debug!("TypeaheadOverlay::show() called, items: {}, filtered: {}", 
               self.items.len(), self.filtered_items.len());
        self.visible = true;
        self.filter_items();
        debug!("After show() filter_items, filtered: {}", self.filtered_items.len());
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.input.clear();
        self.cursor_position = 0;
        self.selected_index = 0;
    }

    pub fn set_items(&mut self, items: Vec<TypeaheadItem>) {
        use tracing::debug;
        debug!("TypeaheadOverlay::set_items called with {} items: {:?}", 
               items.len(), items.iter().map(|item| &item.label).collect::<Vec<_>>());
        self.items = items;
        self.filter_items();
        debug!("After filter_items, filtered_items count: {}", self.filtered_items.len());
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

    pub fn filter_items(&mut self) {
        let query = self.input.to_lowercase();
        
        if query.is_empty() {
            // When there's no query, show all items in their original order
            self.filtered_items = self.items.clone();
        } else {
            // First, separate current item and other items
            let (current_items, other_items): (Vec<_>, Vec<_>) = self.items.iter()
                .cloned()
                .partition(|item| {
                    self.current_value.as_ref().map_or(false, |cv| cv == &item.value)
                });
            
            // Filter other items while preserving the original order
            let filtered: Vec<_> = other_items.into_iter()
                .filter(|item| {
                    item.label.to_lowercase().contains(&query) ||
                    item.value.to_lowercase().contains(&query) ||
                    item.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query))
                })
                .collect();
            
            // Note: We don't re-sort here to preserve the careful ordering from ModelSelectionOverlay
            // The items are already sorted by authentication status and provider priority
            
            // Put current item at top if it matches
            self.filtered_items = current_items.into_iter()
                .filter(|item| {
                    item.label.to_lowercase().contains(&query) ||
                    item.value.to_lowercase().contains(&query)
            })
            .chain(filtered)
            .take(10) // Limit to 10 items
            .collect();
        }
        
        // Reset selection if needed
        if self.selected_index >= self.filtered_items.len() {
            self.selected_index = 0;
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate overlay size - responsive but with min/max constraints
        let width = (area.width * 70 / 100).max(50).min(80);
        let height = (area.height * 60 / 100).max(15).min(30);
        
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

        // Layout - adjust based on whether we show input
        let chunks = if self.hide_input {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Description
                    Constraint::Min(5),    // List (more space without input)
                    Constraint::Length(1), // Help
                ])
                .split(inner)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Description
                    Constraint::Length(3), // Input
                    Constraint::Min(5),    // List
                    Constraint::Length(1), // Help
                ])
                .split(inner)
        };

        // Description
        let desc_paragraph = Paragraph::new(self.description.as_str())
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Left);
        f.render_widget(desc_paragraph, chunks[0]);

        // Input field (only if not hidden)
        let list_chunk_idx = if !self.hide_input {
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
            
            2 // List is at index 2 when input is shown
        } else {
            1 // List is at index 1 when input is hidden
        };

        // Filtered items list
        use tracing::debug;
        debug!("RENDER: Creating ListItems from {} filtered_items: {:?}", 
               self.filtered_items.len(), 
               self.filtered_items.iter().map(|i| &i.label).collect::<Vec<_>>());
        let items: Vec<ListItem> = self.filtered_items.iter().enumerate().map(|(i, item)| {
            let mut spans = vec![];
            
            // Check if this is the current value (active/selected)
            let is_current = self.current_value.as_ref().map_or(false, |cv| cv == &item.value);
            
            // Add indicators: current (✓) takes priority over navigation (▶)
            if is_current {
                // Use different color for checkmark when highlighted to ensure visibility
                let checkmark_color = if i == self.selected_index {
                    Color::Yellow // Yellow stands out on cyan background
                } else {
                    Color::Green  // Green on normal background
                };
                spans.push(Span::styled("✓ ", Style::default().fg(checkmark_color).add_modifier(Modifier::BOLD)));
            } else if i == self.selected_index {
                spans.push(Span::styled("▶ ", Style::default().fg(Color::Yellow))); // Make arrow more visible too
            } else {
                spans.push(Span::raw("  "));
            }
            
            // Add label with status icons
            let label_style = if item.available {
                if is_current {
                    Style::default().add_modifier(Modifier::BOLD) // Bold for current selection
                } else {
                    Style::default()
                }
            } else {
                Style::default().fg(Color::DarkGray)
            };
            
            // Add quality indicator first (most important)
            if let Some(quality) = item.get_quality_indicator() {
                spans.push(Span::styled(format!("{} ", quality), Style::default()));
            }
            
            // Add status icons (⭐ for default, 🧠 for large context)
            let status_icons = item.get_status_icons();
            if !status_icons.is_empty() {
                spans.push(Span::styled(format!("{} ", status_icons), Style::default()));
            }
            
            spans.push(Span::styled(&item.label, label_style));
            
            // Add capability icons after label with enhanced styling
            let capability_icons = item.get_capability_icons();
            if !capability_icons.is_empty() {
                spans.push(Span::styled(format!(" {}", capability_icons), Style::default().fg(Color::Yellow)));
            }
            
            // Add context window info with enhanced styling
            if let Some(context) = item.format_context_window() {
                spans.push(Span::styled(
                    format!(" • {}", context), 
                    Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
                ));
            }
            
            // Add pricing info with cost efficiency indicator
            if let Some(pricing) = item.format_pricing() {
                let pricing_style = if pricing == "Free" {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                };
                
                spans.push(Span::styled(
                    format!(" • {}", pricing), 
                    pricing_style
                ));
                
                // Add cost efficiency indicator for extra context
                if let Some(efficiency) = item.get_cost_efficiency() {
                    spans.push(Span::styled(
                        format!(" {}", efficiency), 
                        Style::default().fg(Color::Green)
                    ));
                }
            }
            
            // Add description if present (kept for backwards compatibility)
            if let Some(desc) = &item.description {
                spans.push(Span::styled(
                    format!(" - {}", desc), 
                    Style::default().fg(Color::DarkGray)
                ));
            }
            
            let style = if i == self.selected_index {
                // Use black text on cyan background for better contrast
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            ListItem::new(Line::from(spans)).style(style)
        }).collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(list, chunks[list_chunk_idx]);

        // Help text
        let help_chunk_idx = if self.hide_input { 2 } else { 3 };
        let help = if self.filtered_items.is_empty() && !self.input.is_empty() {
            // Only show "No matches" if user is actively filtering
            "No matches found. Press Esc to cancel."
        } else {
            // Always show navigation help when not filtering or when items exist
            "↑/↓ Navigate • Enter Select • Tab Switch • Esc Cancel"
        };
        let help_text = Paragraph::new(help)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(help_text, chunks[help_chunk_idx]);
    }
}