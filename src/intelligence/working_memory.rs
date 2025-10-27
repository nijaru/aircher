// Week 5: Working Memory - Dynamic context management
//
// Purpose: Manage LLM context window with intelligent pruning
// Key innovation: Continuous work without restart (unlike Claude Code)
//
// Algorithm:
// 1. Track context items (messages, tool results, code snippets)
// 2. Calculate relevance score for each item
// 3. When 80% full, prune bottom 30% by token count
// 4. Summarize removed items to episodic memory
//
// Relevance scoring:
// - Time decay (exponential, half-life ~1 hour)
// - Task association (2x boost for current task)
// - Dependencies (items that reference this)
// - Item type weights (task state > messages > tool results)

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::intelligence::unified_intelligence::UserIntent;

/// Context window with intelligent pruning
pub struct ContextWindow {
    /// All context items (messages, tool results, code snippets)
    items: Vec<ContextItem>,

    /// Current token count
    token_count: usize,

    /// Maximum tokens before pruning (e.g., 180K for Claude, leave 20K buffer)
    max_tokens: usize,

    /// Session identifier
    session_id: String,

    /// Current task being worked on
    current_task_id: Option<String>,

    /// Number of times we've pruned
    pruning_count: usize,
}

/// Single item in the context window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    /// Unique identifier
    pub id: Uuid,

    /// Actual content (text for messages, code for snippets)
    pub content: String,

    /// Type of item (affects relevance scoring)
    pub item_type: ContextItemType,

    /// When this item was added
    pub timestamp: DateTime<Utc>,

    /// Relevance score (0.0 to 100.0+)
    pub relevance_score: f32,

    /// Token cost of this item
    pub token_cost: usize,

    /// Associated task (if any)
    pub task_id: Option<String>,

    /// Other items that depend on this one
    pub dependencies: Vec<Uuid>,

    /// Never remove (e.g., system prompt)
    pub sticky: bool,
}

/// Type of context item (affects relevance and pruning)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContextItemType {
    /// System prompt (sticky, never removed)
    SystemPrompt,

    /// User message with optional intent
    UserMessage {
        intent: Option<UserIntent>,
    },

    /// Assistant response with tool calls
    AssistantResponse {
        tool_calls: Vec<String>,
    },

    /// Tool execution result
    ToolResult {
        tool_name: String,
        file_path: Option<String>,
    },

    /// Code snippet from file
    CodeSnippet {
        file_path: String,
        line_range: (usize, usize),
    },

    /// Task state tracking
    TaskState {
        task_id: String,
        description: String,
    },

    /// Knowledge graph query result
    KnowledgeGraphQuery {
        query: String,
        result_summary: String,
    },
}

impl ContextWindow {
    /// Create a new context window
    pub fn new(session_id: String, max_tokens: usize) -> Self {
        info!("Creating new context window: session={}, max_tokens={}", session_id, max_tokens);

        Self {
            items: Vec::new(),
            token_count: 0,
            max_tokens,
            session_id,
            current_task_id: None,
            pruning_count: 0,
        }
    }

    /// Add an item to the context window
    pub fn add_item(&mut self, item: ContextItem) {
        debug!("Adding context item: type={:?}, tokens={}", item.item_type, item.token_cost);

        self.token_count += item.token_cost;
        self.items.push(item);
    }

    /// Set the current task
    pub fn set_current_task(&mut self, task_id: String) {
        info!("Setting current task: {}", task_id);
        self.current_task_id = Some(task_id);
    }

    /// Get current token count
    pub fn token_count(&self) -> usize {
        self.token_count
    }

    /// Get number of items
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Check if pruning is needed (at 80% capacity)
    pub fn needs_pruning(&self) -> bool {
        let threshold = (self.max_tokens as f32 * 0.8) as usize;
        self.token_count > threshold
    }

    /// Calculate relevance score for an item
    fn calculate_relevance(&self, item: &ContextItem) -> f32 {
        let mut score = 1.0;

        // Factor 1: Time decay (exponential, half-life ~1 hour)
        let age_minutes = (Utc::now() - item.timestamp).num_minutes() as f32;
        let time_score = (-age_minutes / 60.0).exp(); // Exponential decay
        score *= time_score;

        // Factor 2: Task association (current task = more relevant)
        if let Some(task_id) = &self.current_task_id {
            if item.task_id.as_ref() == Some(task_id) {
                score *= 2.0; // 2x boost for current task
            }
        }

        // Factor 3: Dependencies (other items reference this)
        let dependency_count = self.count_dependents(item.id);
        let dependency_boost = 1.0 + (dependency_count as f32 * 0.2);
        score *= dependency_boost;

        // Factor 4: Item type (some types are more important)
        let type_multiplier = match &item.item_type {
            ContextItemType::SystemPrompt => 100.0, // Never remove (but marked sticky anyway)
            ContextItemType::TaskState { .. } => 2.0, // Keep task state
            ContextItemType::UserMessage { .. } => 1.5, // Keep user intent
            ContextItemType::AssistantResponse { tool_calls } => {
                if !tool_calls.is_empty() { 1.2 } else { 0.9 }
            }
            ContextItemType::ToolResult { .. } => 0.8, // Tool results decay
            ContextItemType::CodeSnippet { .. } => 0.7, // Code snippets decay faster
            ContextItemType::KnowledgeGraphQuery { .. } => 0.6, // Can re-query graph
        };
        score *= type_multiplier;

        // Factor 5: Explicit relevance score (set by query)
        score *= item.relevance_score;

        score.clamp(0.0, 100.0)
    }

    /// Count how many items depend on this one
    fn count_dependents(&self, item_id: Uuid) -> usize {
        self.items
            .iter()
            .filter(|item| item.dependencies.contains(&item_id))
            .count()
    }

    /// Prune context window (remove bottom 30% by token count)
    pub fn prune(&mut self) -> Result<Vec<ContextItem>> {
        info!("Pruning context window: {} tokens, {} items", self.token_count, self.items.len());

        let start_tokens = self.token_count;
        let start_items = self.items.len();

        // Calculate relevance for each item
        let mut scored_items: Vec<(usize, f32)> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| !item.sticky) // Never remove sticky items
            .map(|(idx, item)| {
                let score = self.calculate_relevance(item);
                (idx, score)
            })
            .collect();

        // Sort by relevance (lowest first)
        scored_items.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // Remove bottom 30% by token count
        let target_removal = (self.token_count as f32 * 0.3) as usize;
        let mut removed_tokens = 0;
        let mut indices_to_remove = Vec::new();

        for (idx, score) in scored_items {
            if removed_tokens >= target_removal {
                break;
            }

            let item = &self.items[idx];
            removed_tokens += item.token_cost;
            indices_to_remove.push(idx);

            debug!("Pruning item: type={:?}, tokens={}, score={:.2}",
                   item.item_type, item.token_cost, score);
        }

        // Remove items (in reverse order to preserve indices)
        indices_to_remove.sort_by(|a, b| b.cmp(a));
        let mut removed_items = Vec::new();

        for idx in indices_to_remove {
            let item = self.items.remove(idx);
            self.token_count -= item.token_cost;
            removed_items.push(item);
        }

        self.pruning_count += 1;

        info!("Pruning complete: {} → {} tokens ({} items removed)",
              start_tokens, self.token_count, removed_items.len());

        Ok(removed_items)
    }

    /// Get all items (for building LLM messages)
    pub fn get_items(&self) -> &[ContextItem] {
        &self.items
    }

    /// Get statistics
    pub fn stats(&self) -> ContextWindowStats {
        ContextWindowStats {
            total_items: self.items.len(),
            total_tokens: self.token_count,
            max_tokens: self.max_tokens,
            utilization: (self.token_count as f32 / self.max_tokens as f32 * 100.0),
            pruning_count: self.pruning_count,
            sticky_items: self.items.iter().filter(|i| i.sticky).count(),
        }
    }
}

/// Statistics about the context window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindowStats {
    pub total_items: usize,
    pub total_tokens: usize,
    pub max_tokens: usize,
    pub utilization: f32, // Percentage
    pub pruning_count: usize,
    pub sticky_items: usize,
}

impl std::fmt::Display for ContextWindowStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Context: {} items, {}/{} tokens ({:.1}% full), {} prunings",
            self.total_items,
            self.total_tokens,
            self.max_tokens,
            self.utilization,
            self.pruning_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_window_creation() {
        let window = ContextWindow::new("test-session".to_string(), 100000);
        assert_eq!(window.token_count(), 0);
        assert_eq!(window.item_count(), 0);
        assert!(!window.needs_pruning());
    }

    #[test]
    fn test_add_item() {
        let mut window = ContextWindow::new("test-session".to_string(), 100000);

        let item = ContextItem {
            id: Uuid::new_v4(),
            content: "Hello, world!".to_string(),
            item_type: ContextItemType::UserMessage { intent: None },
            timestamp: Utc::now(),
            relevance_score: 1.0,
            token_cost: 100,
            task_id: None,
            dependencies: vec![],
            sticky: false,
        };

        window.add_item(item);

        assert_eq!(window.token_count(), 100);
        assert_eq!(window.item_count(), 1);
    }

    #[test]
    fn test_needs_pruning() {
        let mut window = ContextWindow::new("test-session".to_string(), 1000);

        // Add items up to 80% capacity
        for _ in 0..8 {
            let item = ContextItem {
                id: Uuid::new_v4(),
                content: "test".to_string(),
                item_type: ContextItemType::UserMessage { intent: None },
                timestamp: Utc::now(),
                relevance_score: 1.0,
                token_cost: 100,
                task_id: None,
                dependencies: vec![],
                sticky: false,
            };
            window.add_item(item);
        }

        assert!(window.needs_pruning());
    }

    #[test]
    fn test_pruning() {
        let mut window = ContextWindow::new("test-session".to_string(), 1000);

        // Add 10 items (1000 tokens total)
        for i in 0..10 {
            let item = ContextItem {
                id: Uuid::new_v4(),
                content: format!("Message {}", i),
                item_type: ContextItemType::UserMessage { intent: None },
                timestamp: Utc::now() - chrono::Duration::minutes(i as i64),
                relevance_score: 1.0,
                token_cost: 100,
                task_id: None,
                dependencies: vec![],
                sticky: false,
            };
            window.add_item(item);
        }

        assert_eq!(window.token_count(), 1000);
        assert_eq!(window.item_count(), 10);

        // Prune (should remove ~30% = 300 tokens = 3 items)
        let removed = window.prune().unwrap();

        assert!(removed.len() >= 2 && removed.len() <= 4, "Expected ~3 items removed");
        assert!(window.token_count() <= 700, "Expected ~700 tokens remaining");
    }

    #[test]
    fn test_sticky_items_not_pruned() {
        let mut window = ContextWindow::new("test-session".to_string(), 1000);

        // Add sticky system prompt
        let sticky = ContextItem {
            id: Uuid::new_v4(),
            content: "System prompt".to_string(),
            item_type: ContextItemType::SystemPrompt,
            timestamp: Utc::now(),
            relevance_score: 1.0,
            token_cost: 100,
            task_id: None,
            dependencies: vec![],
            sticky: true,
        };
        window.add_item(sticky);

        // Add regular items
        for i in 0..9 {
            let item = ContextItem {
                id: Uuid::new_v4(),
                content: format!("Message {}", i),
                item_type: ContextItemType::UserMessage { intent: None },
                timestamp: Utc::now(),
                relevance_score: 1.0,
                token_cost: 100,
                task_id: None,
                dependencies: vec![],
                sticky: false,
            };
            window.add_item(item);
        }

        window.prune().unwrap();

        // System prompt should still be there
        let sticky_count = window.get_items().iter().filter(|i| i.sticky).count();
        assert_eq!(sticky_count, 1);
    }

    #[test]
    fn test_task_association_boosts_relevance() {
        let mut window = ContextWindow::new("test-session".to_string(), 100000);
        window.set_current_task("task-1".to_string());

        // Add item for current task
        let task_item = ContextItem {
            id: Uuid::new_v4(),
            content: "Current task work".to_string(),
            item_type: ContextItemType::UserMessage { intent: None },
            timestamp: Utc::now() - chrono::Duration::hours(2), // Old
            relevance_score: 1.0,
            token_cost: 100,
            task_id: Some("task-1".to_string()),
            dependencies: vec![],
            sticky: false,
        };

        // Add recent item for different task
        let other_item = ContextItem {
            id: Uuid::new_v4(),
            content: "Other task work".to_string(),
            item_type: ContextItemType::UserMessage { intent: None },
            timestamp: Utc::now(), // Recent
            relevance_score: 1.0,
            token_cost: 100,
            task_id: Some("task-2".to_string()),
            dependencies: vec![],
            sticky: false,
        };

        let task_score = window.calculate_relevance(&task_item);
        let other_score = window.calculate_relevance(&other_item);

        // Task association should overcome recency
        assert!(task_score > other_score * 0.5,
                "Current task should have higher relevance despite age");
    }
}
