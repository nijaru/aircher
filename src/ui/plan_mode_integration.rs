use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::agent::plan_mode::{ExecutionPlan, PlannedTask, TaskSafety, TaskEffort, PlannedTaskType};

/// UI widget for displaying execution plans
pub struct PlanDisplayWidget {
    visible: bool,
    current_plan: Option<ExecutionPlan>,
    selected_task: usize,
    show_details: bool,
}

impl PlanDisplayWidget {
    pub fn new() -> Self {
        Self {
            visible: false,
            current_plan: None,
            selected_task: 0,
            show_details: false,
        }
    }

    pub fn show_plan(&mut self, plan: ExecutionPlan) {
        self.current_plan = Some(plan);
        self.visible = true;
        self.selected_task = 0;
        self.show_details = false;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.current_plan = None;
        self.selected_task = 0;
        self.show_details = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn select_next_task(&mut self) {
        if let Some(plan) = &self.current_plan {
            if self.selected_task < plan.tasks.len().saturating_sub(1) {
                self.selected_task += 1;
            }
        }
    }

    pub fn select_previous_task(&mut self) {
        if self.selected_task > 0 {
            self.selected_task -= 1;
        }
    }

    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        let Some(plan) = &self.current_plan else {
            return;
        };

        // Create overlay background
        let overlay_area = centered_rect(90, 80, area);
        f.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .title("⏸ Plan Mode - Execution Plan")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(Color::Cyan)),
            overlay_area,
        );

        // Split into header, tasks, and controls
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(4), // Header
                Constraint::Min(10),   // Tasks
                Constraint::Length(3), // Controls
            ])
            .split(overlay_area);

        // Render plan header
        self.render_plan_header(f, chunks[0], plan);

        // Render task list
        self.render_task_list(f, chunks[1], plan);

        // Render controls
        self.render_controls(f, chunks[2]);
    }

    fn render_plan_header(&self, f: &mut Frame, area: Rect, plan: &ExecutionPlan) {
        let header_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title
                Constraint::Length(1), // Description
                Constraint::Length(1), // Stats
                Constraint::Length(1), // Risk
            ])
            .split(area);

        // Plan title
        let title = Paragraph::new(plan.title.clone())
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            .wrap(Wrap { trim: true });
        f.render_widget(title, header_chunks[0]);

        // Description
        let description = Paragraph::new(plan.description.clone())
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: true });
        f.render_widget(description, header_chunks[1]);

        // Stats
        let total_minutes = plan.estimated_total_time.as_secs() / 60;
        let stats_text = format!(
            "Tasks: {} | Estimated time: {}h {}m | Created: {}",
            plan.tasks.len(),
            total_minutes / 60,
            total_minutes % 60,
            plan.created_at.format("%Y-%m-%d %H:%M")
        );
        let stats = Paragraph::new(stats_text)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(stats, header_chunks[2]);

        // Risk assessment
        let risk_color = match plan.risk_assessment.overall_risk {
            TaskSafety::Safe => Color::Green,
            TaskSafety::LowRisk => Color::Yellow,
            TaskSafety::MediumRisk => Color::Magenta,
            TaskSafety::HighRisk => Color::Red,
            TaskSafety::Critical => Color::Red,
        };
        let risk_text = format!("Risk: {:?}", plan.risk_assessment.overall_risk);
        let risk = Paragraph::new(risk_text)
            .style(Style::default().fg(risk_color).add_modifier(Modifier::BOLD));
        f.render_widget(risk, header_chunks[3]);
    }

    fn render_task_list(&self, f: &mut Frame, area: Rect, plan: &ExecutionPlan) {
        let items: Vec<ListItem> = plan.tasks.iter().enumerate().map(|(i, task)| {
            let is_selected = i == self.selected_task;

            // Task number and status
            let task_num = format!("{}.", i + 1);
            let task_desc = &task.description;

            // Effort indicator
            let effort_indicator = match task.estimated_effort {
                TaskEffort::Trivial => "⚡",
                TaskEffort::Small => "🔹",
                TaskEffort::Medium => "🔸",
                TaskEffort::Large => "🔶",
                TaskEffort::ExtraLarge => "🔴",
            };

            // Safety indicator
            let safety_color = match task.safety_level {
                TaskSafety::Safe => Color::Green,
                TaskSafety::LowRisk => Color::Yellow,
                TaskSafety::MediumRisk => Color::Magenta,
                TaskSafety::HighRisk => Color::Red,
                TaskSafety::Critical => Color::Red,
            };

            // Type indicator
            let type_indicator = match &task.task_type {
                PlannedTaskType::Analysis { .. } => "🔍",
                PlannedTaskType::Modification { .. } => "✏️",
                PlannedTaskType::Creation { .. } => "📄",
                PlannedTaskType::Deletion { .. } => "🗑️",
                PlannedTaskType::Execution { .. } => "⚙️",
                PlannedTaskType::Refactoring { .. } => "🔧",
            };

            let style = if is_selected {
                Style::default()
                    .bg(Color::Rgb(50, 50, 50))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", task_num), style),
                Span::styled(type_indicator, style),
                Span::styled(" ", style),
                Span::styled(effort_indicator, style),
                Span::styled(" ", style),
                Span::styled(task_desc, style),
                Span::styled(" ", style),
                Span::styled("●", Style::default().fg(safety_color)),
            ]);

            ListItem::new(line)
        }).collect();

        let task_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Tasks")
                    .title_style(Style::default().fg(Color::Cyan))
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(0, 100, 100))
                    .add_modifier(Modifier::BOLD)
            );

        f.render_widget(task_list, area);

        // Show task details if requested
        if self.show_details && self.selected_task < plan.tasks.len() {
            let task = &plan.tasks[self.selected_task];
            self.render_task_details(f, area, task);
        }
    }

    fn render_task_details(&self, f: &mut Frame, area: Rect, task: &PlannedTask) {
        // Create a popup for task details
        let popup_area = centered_rect(70, 50, area);

        // Background
        f.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .title("Task Details")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(Color::Cyan)),
            popup_area,
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // Description
                Constraint::Length(1), // Type info
                Constraint::Length(1), // Effort & Safety
                Constraint::Min(2),    // Rationale
            ])
            .split(popup_area);

        // Description
        let description = Paragraph::new(task.description.clone())
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
        f.render_widget(description, chunks[0]);

        // Type information
        let type_info = match &task.task_type {
            PlannedTaskType::Analysis { files, purpose } => {
                format!("Analysis: {} files - {}", files.len(), purpose)
            }
            PlannedTaskType::Modification { files, changes_summary } => {
                format!("Modify: {} files - {}", files.len(), changes_summary)
            }
            PlannedTaskType::Creation { files, purpose } => {
                format!("Create: {} files - {}", files.len(), purpose)
            }
            PlannedTaskType::Deletion { files, reason } => {
                format!("Delete: {} files - {}", files.len(), reason)
            }
            PlannedTaskType::Execution { commands, purpose } => {
                format!("Execute: {} commands - {}", commands.len(), purpose)
            }
            PlannedTaskType::Refactoring { scope, approach } => {
                format!("Refactor: {} - {}", scope, approach)
            }
        };
        let type_paragraph = Paragraph::new(type_info)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(type_paragraph, chunks[1]);

        // Effort and safety
        let effort_safety = format!(
            "Effort: {:?} | Safety: {:?} | Dependencies: {}",
            task.estimated_effort,
            task.safety_level,
            task.dependencies.len()
        );
        let effort_safety_paragraph = Paragraph::new(effort_safety)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(effort_safety_paragraph, chunks[2]);

        // Rationale
        let rationale = Paragraph::new(task.rationale.clone())
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .title("Rationale")
                    .title_style(Style::default().fg(Color::Cyan))
            );
        f.render_widget(rationale, chunks[3]);
    }

    fn render_controls(&self, f: &mut Frame, area: Rect) {
        let controls_text = if self.show_details {
            "↑/↓: Navigate | d: Hide details | Enter: Execute plan | Esc: Cancel"
        } else {
            "↑/↓: Navigate | d: Show details | Enter: Execute plan | Esc: Cancel"
        };

        let controls = Paragraph::new(controls_text)
            .style(Style::default().fg(Color::Rgb(107, 114, 128)))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .title("Controls")
                    .title_style(Style::default().fg(Color::Cyan))
            );
        f.render_widget(controls, area);
    }
}

/// Helper function to create centered rect
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