use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};

use crate::agent::approval_modes::{PendingChange, ChangeType, ApprovalMode, ChangeApprovalManager};

/// Individual change approval choices
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChangeApprovalChoice {
    Apply,              // y - Apply this change
    ApplyAll,          // A - Apply all remaining changes
    Skip,              // n - Skip this change
    SkipAll,           // N - Skip all remaining changes
    Edit,              // e - Edit the change
    ViewDiff,          // d - View full diff
    Abort,             // q - Abort all changes
    ApplyPattern,      // a - Always apply similar changes this session
}

/// Mode selector for approval workflow
#[derive(Debug)]
pub struct ApprovalModeSelector {
    visible: bool,
    selected_mode: usize,
    modes: Vec<(ApprovalMode, &'static str, &'static str)>,
}

impl ApprovalModeSelector {
    pub fn new() -> Self {
        Self {
            visible: false,
            selected_mode: 1, // Default to Review mode
            modes: vec![
                (ApprovalMode::Auto, "Auto", "Apply all changes automatically"),
                (ApprovalMode::Review, "Review", "Review each change before applying (default)"),
                (ApprovalMode::Smart, "Smart", "Auto-approve safe operations, review destructive ones"),
                (ApprovalMode::DiffOnly, "Diff Only", "Show diffs only, never apply changes"),
            ],
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ApprovalMode> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_mode > 0 {
                    self.selected_mode -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_mode < self.modes.len() - 1 {
                    self.selected_mode += 1;
                }
            }
            KeyCode::Enter => {
                self.visible = false;
                return Some(self.modes[self.selected_mode].0);
            }
            KeyCode::Esc => {
                self.visible = false;
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let idx = c.to_digit(10).unwrap() as usize - 1;
                if idx < self.modes.len() {
                    self.selected_mode = idx;
                    self.visible = false;
                    return Some(self.modes[idx].0);
                }
            }
            _ => {}
        }
        None
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        let width = 60.min(area.width - 4);
        let height = 10.min(area.height - 4);
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;

        let popup_area = Rect { x, y, width, height };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(" Approval Mode ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(popup_area);
        f.render_widget(block, popup_area);

        let items: Vec<ListItem> = self.modes
            .iter()
            .enumerate()
            .map(|(idx, (_, name, desc))| {
                let selected = idx == self.selected_mode;
                let prefix = if selected { "▶ " } else { "  " };
                let number = format!("{}. ", idx + 1);

                let content = vec![
                    Line::from(vec![
                        Span::styled(prefix, if selected {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default()
                        }),
                        Span::styled(number, Style::default().fg(Color::DarkGray)),
                        Span::styled(*name, if selected {
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().add_modifier(Modifier::BOLD)
                        }),
                    ]),
                    Line::from(vec![
                        Span::raw("     "),
                        Span::styled(*desc, Style::default().fg(Color::Gray)),
                    ]),
                ];

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items);
        f.render_widget(list, inner);
    }
}

/// Main change approval UI widget
pub struct ChangeApprovalWidget {
    visible: bool,
    current_change: Option<PendingChange>,
    diff_lines: Vec<String>,
    scroll_offset: usize,
    scroll_state: ScrollbarState,
    show_full_diff: bool,
    manager: ChangeApprovalManager,
    batch_count: usize, // Number of changes in current batch
}

impl ChangeApprovalWidget {
    pub fn new(mode: ApprovalMode) -> Self {
        Self {
            visible: false,
            current_change: None,
            diff_lines: Vec::new(),
            scroll_offset: 0,
            scroll_state: ScrollbarState::default(),
            show_full_diff: false,
            manager: ChangeApprovalManager::new(mode),
            batch_count: 0,
        }
    }

    pub fn set_mode(&mut self, mode: ApprovalMode) {
        self.manager.set_mode(mode);
    }

    pub fn get_mode(&self) -> ApprovalMode {
        self.manager.get_mode()
    }

    pub fn queue_change(&mut self, change: PendingChange) {
        let _ = self.manager.queue_change(change);
        self.batch_count = self.manager.get_all_pending().len();

        // If in review mode and not currently showing, show the first change
        if self.get_mode() == ApprovalMode::Review && !self.visible {
            self.show_next_change();
        }
    }

    pub fn queue_changes(&mut self, changes: Vec<PendingChange>) {
        for change in changes {
            let _ = self.manager.queue_change(change);
        }
        self.batch_count = self.manager.get_all_pending().len();

        if self.get_mode() == ApprovalMode::Review && !self.visible {
            self.show_next_change();
        }
    }

    fn show_next_change(&mut self) {
        if let Some(change) = self.manager.get_next_pending() {
            let diff = change.generate_diff();
            self.diff_lines = diff.lines().map(String::from).collect();
            self.scroll_offset = 0;
            self.scroll_state = ScrollbarState::default()
                .content_length(self.diff_lines.len())
                .position(0);
            self.current_change = Some(change);
            self.visible = true;
        } else {
            self.visible = false;
            self.batch_count = 0;
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ChangeApprovalChoice> {
        if !self.visible {
            return None;
        }

        match key.code {
            // Approval actions
            KeyCode::Char('y') => {
                if let Some(change) = &self.current_change {
                    let _ = self.manager.approve_change(&change.id);
                    self.show_next_change();
                    return Some(ChangeApprovalChoice::Apply);
                }
            }
            KeyCode::Char('A') => {
                let _ = self.manager.approve_all();
                self.visible = false;
                self.batch_count = 0;
                return Some(ChangeApprovalChoice::ApplyAll);
            }
            KeyCode::Char('n') => {
                if let Some(change) = &self.current_change {
                    let _ = self.manager.reject_change(&change.id);
                    self.show_next_change();
                    return Some(ChangeApprovalChoice::Skip);
                }
            }
            KeyCode::Char('N') => {
                let _ = self.manager.reject_all();
                self.visible = false;
                self.batch_count = 0;
                return Some(ChangeApprovalChoice::SkipAll);
            }
            KeyCode::Char('a') => {
                // Add pattern for session approval
                if let Some(change) = &self.current_change {
                    if let ChangeType::RunCommand { command, .. } = &change.change_type {
                        let pattern = command.split_whitespace().next().unwrap_or("").to_string();
                        self.manager.add_session_pattern(pattern);
                        let _ = self.manager.approve_change(&change.id);
                        self.show_next_change();
                        return Some(ChangeApprovalChoice::ApplyPattern);
                    }
                }
            }

            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                    self.scroll_state = self.scroll_state.position(self.scroll_offset);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.scroll_offset < self.diff_lines.len().saturating_sub(10) {
                    self.scroll_offset += 1;
                    self.scroll_state = self.scroll_state.position(self.scroll_offset);
                }
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
                self.scroll_state = self.scroll_state.position(self.scroll_offset);
            }
            KeyCode::PageDown => {
                let max_offset = self.diff_lines.len().saturating_sub(10);
                self.scroll_offset = (self.scroll_offset + 10).min(max_offset);
                self.scroll_state = self.scroll_state.position(self.scroll_offset);
            }

            // View options
            KeyCode::Char('d') => {
                self.show_full_diff = !self.show_full_diff;
                return Some(ChangeApprovalChoice::ViewDiff);
            }
            KeyCode::Char('e') => {
                // TODO: Implement edit functionality
                return Some(ChangeApprovalChoice::Edit);
            }

            // Abort
            KeyCode::Esc | KeyCode::Char('q') => {
                let _ = self.manager.reject_all();
                self.visible = false;
                self.batch_count = 0;
                return Some(ChangeApprovalChoice::Abort);
            }

            _ => {}
        }

        None
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.visible || self.current_change.is_none() {
            return;
        }

        let change = self.current_change.as_ref().unwrap();

        // Calculate layout
        let width = (area.width as f32 * 0.9).min(120.0) as u16;
        let height = (area.height as f32 * 0.8).min(40.0) as u16;
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;

        let popup_area = Rect { x, y, width, height };

        // Clear background
        f.render_widget(Clear, popup_area);

        // Main block
        let title = format!(
            " {} Change ({}/{}) - {} ",
            match self.manager.get_mode() {
                ApprovalMode::Auto => "Auto",
                ApprovalMode::Review => "Review",
                ApprovalMode::Smart => "Smart",
                ApprovalMode::DiffOnly => "Diff",
            },
            self.manager.get_all_pending().len() + 1,
            self.batch_count,
            change.tool_name
        );

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(popup_area);
        f.render_widget(block, popup_area);

        // Split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Description
                Constraint::Min(5),     // Diff
                Constraint::Length(4),  // Controls
            ])
            .split(inner);

        // Description section
        let desc_text = vec![
            Line::from(vec![
                Span::styled("Change: ", Style::default().fg(Color::Gray)),
                Span::styled(&change.description, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    match &change.change_type {
                        ChangeType::CreateFile { .. } => "Create File",
                        ChangeType::ModifyFile { .. } => "Modify File",
                        ChangeType::DeleteFile { .. } => "Delete File",
                        ChangeType::RunCommand { .. } => "Run Command",
                    },
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];

        let desc = Paragraph::new(desc_text)
            .wrap(Wrap { trim: true });
        f.render_widget(desc, chunks[0]);

        // Diff section with scrollbar
        let visible_lines = chunks[1].height as usize;
        let diff_content: Vec<Line> = self.diff_lines
            .iter()
            .skip(self.scroll_offset)
            .take(visible_lines)
            .map(|line| {
                if line.starts_with('+') && !line.starts_with("+++") {
                    Line::styled(line.clone(), Style::default().fg(Color::Green))
                } else if line.starts_with('-') && !line.starts_with("---") {
                    Line::styled(line.clone(), Style::default().fg(Color::Red))
                } else if line.starts_with("@@") {
                    Line::styled(line.clone(), Style::default().fg(Color::Cyan))
                } else {
                    Line::from(line.clone())
                }
            })
            .collect();

        let diff = Paragraph::new(diff_content)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(diff, chunks[1]);

        // Scrollbar
        if self.diff_lines.len() > visible_lines {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            f.render_stateful_widget(
                scrollbar,
                chunks[1].inner(Margin { vertical: 1, horizontal: 0 }),
                &mut self.scroll_state.clone(),
            );
        }

        // Controls section
        let controls = if change.is_safe {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(" [y]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(" Apply  "),
                    Span::styled("[n]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::raw(" Skip  "),
                    Span::styled("[A]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(" Apply All  "),
                    Span::styled("[N]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::raw(" Skip All  "),
                    Span::styled("[d]", Style::default().fg(Color::Blue)),
                    Span::raw(" Diff  "),
                    Span::styled("[q]", Style::default().fg(Color::Yellow)),
                    Span::raw(" Abort"),
                ]),
                Line::from(vec![
                    Span::styled(" ⚡ ", Style::default().fg(Color::Green)),
                    Span::styled("Safe operation - can be auto-approved in Smart mode",
                        Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
                ]),
            ]
        } else {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(" [y]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(" Apply  "),
                    Span::styled("[n]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::raw(" Skip  "),
                    Span::styled("[A]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(" Apply All  "),
                    Span::styled("[N]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::raw(" Skip All  "),
                    Span::styled("[d]", Style::default().fg(Color::Blue)),
                    Span::raw(" Diff  "),
                    Span::styled("[q]", Style::default().fg(Color::Yellow)),
                    Span::raw(" Abort"),
                ]),
                Line::from(vec![
                    Span::styled(" ⚠️  ", Style::default().fg(Color::Yellow)),
                    Span::styled("Requires manual review",
                        Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
                ]),
            ]
        };

        let controls_widget = Paragraph::new(controls);
        f.render_widget(controls_widget, chunks[2]);
    }

    pub fn get_stats(&self) -> (usize, usize, usize) {
        let stats = self.manager.get_stats();
        (stats.pending, stats.applied, stats.rejected)
    }
}