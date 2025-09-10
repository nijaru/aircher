use chrono::{DateTime, Utc};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TodoStatus {
    Todo,      // ☐ - Pending task
    InProgress, // 🔄 - Currently being worked on
    Completed, // ✓ - Finished successfully  
    Failed,    // ✗ - Task failed or blocked
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TodoPriority {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub content: String,
    pub status: TodoStatus,
    pub priority: TodoPriority,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TodoItem {
    pub fn new(id: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            content,
            status: TodoStatus::Todo,
            priority: TodoPriority::Medium,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_priority(mut self, priority: TodoPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_status(mut self, status: TodoStatus) -> Self {
        self.status = status;
        self.updated_at = Utc::now();
        self
    }

    pub fn status_icon(&self) -> &'static str {
        match self.status {
            TodoStatus::Todo => "☐",
            TodoStatus::InProgress => "🔄",
            TodoStatus::Completed => "✓",
            TodoStatus::Failed => "✗",
        }
    }

    pub fn status_color(&self) -> Color {
        match self.status {
            TodoStatus::Todo => Color::White,
            TodoStatus::InProgress => Color::Yellow,
            TodoStatus::Completed => Color::Green,
            TodoStatus::Failed => Color::Red,
        }
    }

    pub fn priority_indicator(&self) -> &'static str {
        match self.priority {
            TodoPriority::High => "[High]",
            TodoPriority::Medium => "[Medium]",
            TodoPriority::Low => "[Low]",
        }
    }
}

pub struct TodoPanel {
    todos: HashMap<String, TodoItem>,
    visible: bool,
    compact_mode: bool,
}

impl Default for TodoPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl TodoPanel {
    pub fn new() -> Self {
        Self {
            todos: HashMap::new(),
            visible: true,
            compact_mode: true,
        }
    }

    pub fn add_todo(&mut self, id: String, content: String) -> &TodoItem {
        let todo = TodoItem::new(id.clone(), content);
        self.todos.insert(id.clone(), todo);
        &self.todos[&id]
    }

    pub fn add_todo_with_priority(&mut self, id: String, content: String, priority: TodoPriority) -> &TodoItem {
        let todo = TodoItem::new(id.clone(), content).with_priority(priority);
        self.todos.insert(id.clone(), todo);
        &self.todos[&id]
    }

    pub fn update_status(&mut self, id: &str, status: TodoStatus) -> bool {
        if let Some(todo) = self.todos.get_mut(id) {
            todo.status = status;
            todo.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn remove_todo(&mut self, id: &str) -> bool {
        self.todos.remove(id).is_some()
    }

    pub fn get_todos(&self) -> Vec<&TodoItem> {
        let mut todos: Vec<&TodoItem> = self.todos.values().collect();
        // Sort by priority (High -> Medium -> Low) then by created_at
        todos.sort_by(|a, b| {
            use std::cmp::Ordering;
            let priority_order = match (&a.priority, &b.priority) {
                (TodoPriority::High, TodoPriority::High) => Ordering::Equal,
                (TodoPriority::High, _) => Ordering::Less,
                (_, TodoPriority::High) => Ordering::Greater,
                (TodoPriority::Medium, TodoPriority::Medium) => Ordering::Equal,
                (TodoPriority::Medium, TodoPriority::Low) => Ordering::Less,
                (TodoPriority::Low, _) => Ordering::Greater,
            };
            
            if priority_order == Ordering::Equal {
                a.created_at.cmp(&b.created_at)
            } else {
                priority_order
            }
        });
        todos
    }

    pub fn count_by_status(&self) -> (usize, usize, usize, usize) {
        let mut todo = 0;
        let mut in_progress = 0;
        let mut completed = 0;
        let mut failed = 0;

        for item in self.todos.values() {
            match item.status {
                TodoStatus::Todo => todo += 1,
                TodoStatus::InProgress => in_progress += 1,
                TodoStatus::Completed => completed += 1,
                TodoStatus::Failed => failed += 1,
            }
        }

        (todo, in_progress, completed, failed)
    }

    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_compact_mode(&mut self, compact: bool) {
        self.compact_mode = compact;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible || area.height < 3 {
            return;
        }

        let todos = self.get_todos();
        let (todo_count, in_progress_count, completed_count, _failed_count) = self.count_by_status();

        if self.compact_mode {
            // Compact mode: show summary + active tasks only
            let active_todos: Vec<&TodoItem> = todos
                .iter()
                .filter(|t| matches!(t.status, TodoStatus::Todo | TodoStatus::InProgress))
                .take(3) // Show max 3 active todos
                .copied()
                .collect();

            if active_todos.is_empty() && completed_count == 0 {
                // No todos to show
                return;
            }

            let title = format!(
                "TODO Tasks ({} active, {} completed)",
                todo_count + in_progress_count,
                completed_count
            );

            let items: Vec<ListItem> = active_todos
                .iter()
                .map(|todo| {
                    let content = if todo.content.len() > 50 {
                        format!("{}…", &todo.content[..47])
                    } else {
                        todo.content.clone()
                    };

                    let item_text = format!("{} {} {}", todo.status_icon(), content, todo.priority_indicator());
                    
                    ListItem::new(item_text).style(
                        Style::default().fg(todo.status_color())
                    )
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(title)
                        .style(Style::default().fg(Color::Cyan))
                );

            frame.render_widget(list, area);
        } else {
            // Full mode: show all todos
            let items: Vec<ListItem> = todos
                .iter()
                .map(|todo| {
                    let item_text = format!(
                        "{} {} {}",
                        todo.status_icon(),
                        todo.content,
                        todo.priority_indicator()
                    );
                    
                    ListItem::new(item_text).style(
                        Style::default().fg(todo.status_color())
                    )
                })
                .collect();

            let title = format!(
                "TODO Tasks ({} total, {} active, {} completed)",
                self.todos.len(),
                todo_count + in_progress_count,
                completed_count
            );

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(title)
                        .style(Style::default().fg(Color::Cyan))
                );

            frame.render_widget(list, area);
        }
    }

    /// Integration with external TODO systems (like our todo_write tool)
    pub fn sync_from_external(&mut self, external_todos: Vec<(String, String, String, String)>) {
        // Clear existing todos
        self.todos.clear();
        
        // Add todos from external system
        // Format: (id, content, status, priority)
        for (id, content, status_str, priority_str) in external_todos {
            let status = match status_str.as_str() {
                "completed" => TodoStatus::Completed,
                "in-progress" => TodoStatus::InProgress,
                "failed" => TodoStatus::Failed,
                _ => TodoStatus::Todo,
            };
            
            let priority = match priority_str.as_str() {
                "high" => TodoPriority::High,
                "low" => TodoPriority::Low,
                _ => TodoPriority::Medium,
            };
            
            let todo = TodoItem::new(id.clone(), content)
                .with_status(status)
                .with_priority(priority);
            
            self.todos.insert(id, todo);
        }
    }
}
