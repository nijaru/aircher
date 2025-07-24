use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

#[derive(Debug, Clone)]
pub enum DiffLineType {
    Context,    // Unchanged line
    Addition,   // Line added (green)
    Deletion,   // Line removed (red)
    Modified,   // Line changed (yellow)
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub old_line_num: Option<usize>,
    pub new_line_num: Option<usize>,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub filename: String,
    pub old_content: String,
    pub new_content: String,
    pub diff_lines: Vec<DiffLine>,
}

pub struct DiffViewer {
    visible: bool,
    diffs: Vec<FileDiff>,
    current_diff_index: usize,
    scroll_offset: usize,
    list_state: ListState,
    show_line_numbers: bool,
}

impl DiffViewer {
    pub fn new() -> Self {
        Self {
            visible: false,
            diffs: Vec::new(),
            current_diff_index: 0,
            scroll_offset: 0,
            list_state: ListState::default(),
            show_line_numbers: true,
        }
    }

    pub fn show_diff(&mut self, diff: FileDiff) {
        self.diffs = vec![diff];
        self.current_diff_index = 0;
        self.scroll_offset = 0;
        self.list_state.select(Some(0));
        self.visible = true;
    }

    pub fn show_diffs(&mut self, diffs: Vec<FileDiff>) {
        self.diffs = diffs;
        self.current_diff_index = 0;
        self.scroll_offset = 0;
        self.list_state.select(Some(0));
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn next_file(&mut self) {
        if self.current_diff_index < self.diffs.len().saturating_sub(1) {
            self.current_diff_index += 1;
            self.scroll_offset = 0;
            self.list_state.select(Some(0));
        }
    }

    pub fn prev_file(&mut self) {
        if self.current_diff_index > 0 {
            self.current_diff_index -= 1;
            self.scroll_offset = 0;
            self.list_state.select(Some(0));
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        let current_diff = &self.diffs[self.current_diff_index];
        let max_scroll = current_diff.diff_lines.len().saturating_sub(20); // Assume 20 visible lines
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }

    pub fn get_current_diff(&self) -> Option<&FileDiff> {
        self.diffs.get(self.current_diff_index)
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        if !self.visible || self.diffs.is_empty() {
            return;
        }

        // Calculate centered area (90% width, 80% height)
        let width = (area.width as f32 * 0.9) as u16;
        let height = (area.height as f32 * 0.8) as u16;
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;
        
        let popup_area = Rect { x, y, width, height };

        // Clear background
        f.render_widget(Clear, popup_area);

        let current_diff = &self.diffs[self.current_diff_index];

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3), // Header
                Constraint::Min(5),    // Diff content
                Constraint::Length(3), // Footer/help
            ])
            .split(popup_area);

        // Main block
        let block = Block::default()
            .title(format!(" Diff: {} ({}/{}) ", 
                current_diff.filename, 
                self.current_diff_index + 1, 
                self.diffs.len()
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        f.render_widget(block, popup_area);

        // Inner area for content
        let inner = Block::default().borders(Borders::ALL).inner(popup_area);
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1), // File info
                Constraint::Min(5),    // Diff lines
                Constraint::Length(2), // Help text
            ])
            .split(inner);

        // File info
        let additions = current_diff.diff_lines.iter()
            .filter(|line| matches!(line.line_type, DiffLineType::Addition))
            .count();
        let deletions = current_diff.diff_lines.iter()
            .filter(|line| matches!(line.line_type, DiffLineType::Deletion))
            .count();
        
        let file_info = format!("  +{} -{} lines", additions, deletions);
        let info_paragraph = Paragraph::new(file_info)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(info_paragraph, inner_chunks[0]);

        // Diff lines
        let visible_lines: Vec<ListItem> = current_diff.diff_lines
            .iter()
            .skip(self.scroll_offset)
            .take(inner_chunks[1].height as usize)
            .map(|diff_line| self.format_diff_line(diff_line))
            .collect();

        let diff_list = List::new(visible_lines)
            .block(Block::default().borders(Borders::NONE));

        f.render_widget(diff_list, inner_chunks[1]);

        // Help text
        let help_text = if self.diffs.len() > 1 {
            "↑/↓: Scroll  ←/→: Previous/Next file  Enter: Accept  Esc: Cancel"
        } else {
            "↑/↓: Scroll  Enter: Accept changes  Esc: Cancel"
        };
        
        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(help_paragraph, inner_chunks[2]);
    }

    fn format_diff_line(&self, diff_line: &DiffLine) -> ListItem {
        let (prefix, style) = match diff_line.line_type {
            DiffLineType::Addition => ("+", Style::default().fg(Color::Green)),
            DiffLineType::Deletion => ("-", Style::default().fg(Color::Red)),
            DiffLineType::Modified => ("~", Style::default().fg(Color::Yellow)),
            DiffLineType::Context => (" ", Style::default().fg(Color::White)),
        };

        let line_numbers = if self.show_line_numbers {
            match (&diff_line.old_line_num, &diff_line.new_line_num) {
                (Some(old), Some(new)) => format!("{:4}|{:4} ", old, new),
                (Some(old), None) => format!("{:4}|     ", old),
                (None, Some(new)) => format!("    |{:4} ", new),
                (None, None) => "    |     ".to_string(),
            }
        } else {
            String::new()
        };

        let content = format!("{}{}{}", line_numbers, prefix, diff_line.content);
        
        ListItem::new(Line::from(Span::styled(content, style)))
    }
}

/// Generate diff from old and new content
pub fn generate_diff(filename: String, old_content: String, new_content: String) -> FileDiff {
    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();
    
    let mut diff_lines = Vec::new();
    
    // Simple line-by-line diff algorithm
    // This is a basic implementation - for production, consider using a proper diff library
    let mut old_idx = 0;
    let mut new_idx = 0;
    
    while old_idx < old_lines.len() || new_idx < new_lines.len() {
        if old_idx >= old_lines.len() {
            // Only new lines remaining (additions)
            diff_lines.push(DiffLine {
                line_type: DiffLineType::Addition,
                old_line_num: None,
                new_line_num: Some(new_idx + 1),
                content: new_lines[new_idx].to_string(),
            });
            new_idx += 1;
        } else if new_idx >= new_lines.len() {
            // Only old lines remaining (deletions)
            diff_lines.push(DiffLine {
                line_type: DiffLineType::Deletion,
                old_line_num: Some(old_idx + 1),
                new_line_num: None,
                content: old_lines[old_idx].to_string(),
            });
            old_idx += 1;
        } else if old_lines[old_idx] == new_lines[new_idx] {
            // Lines are the same (context)
            diff_lines.push(DiffLine {
                line_type: DiffLineType::Context,
                old_line_num: Some(old_idx + 1),
                new_line_num: Some(new_idx + 1),
                content: old_lines[old_idx].to_string(),
            });
            old_idx += 1;
            new_idx += 1;
        } else {
            // Lines are different - check if it's a modification or insertion/deletion
            if old_idx + 1 < old_lines.len() && old_lines[old_idx + 1] == new_lines[new_idx] {
                // Deletion: old line doesn't exist in new
                diff_lines.push(DiffLine {
                    line_type: DiffLineType::Deletion,
                    old_line_num: Some(old_idx + 1),
                    new_line_num: None,
                    content: old_lines[old_idx].to_string(),
                });
                old_idx += 1;
            } else if new_idx + 1 < new_lines.len() && old_lines[old_idx] == new_lines[new_idx + 1] {
                // Addition: new line doesn't exist in old
                diff_lines.push(DiffLine {
                    line_type: DiffLineType::Addition,
                    old_line_num: None,
                    new_line_num: Some(new_idx + 1),
                    content: new_lines[new_idx].to_string(),
                });
                new_idx += 1;
            } else {
                // Modification: lines are different
                diff_lines.push(DiffLine {
                    line_type: DiffLineType::Deletion,
                    old_line_num: Some(old_idx + 1),
                    new_line_num: None,
                    content: old_lines[old_idx].to_string(),
                });
                diff_lines.push(DiffLine {
                    line_type: DiffLineType::Addition,
                    old_line_num: None,
                    new_line_num: Some(new_idx + 1),
                    content: new_lines[new_idx].to_string(),
                });
                old_idx += 1;
                new_idx += 1;
            }
        }
    }
    
    FileDiff {
        filename,
        old_content,
        new_content,
        diff_lines,
    }
}