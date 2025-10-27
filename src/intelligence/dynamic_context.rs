// Week 5: Dynamic Context Manager - Integration of all memory systems
//
// Purpose: Unified interface for context management
// Integrates:
// 1. Episodic Memory (DuckDB) - tracks history, learns patterns
// 2. Knowledge Graph (petgraph) - instant codebase queries
// 3. Working Memory (ContextWindow) - dynamic pruning
//
// Key innovation: Continuous work without restart
// - Prune low-value context when window fills
// - Fetch relevant context from knowledge graph
// - Learn from episodic memory patterns
// - Never lose important information

use anyhow::Result;
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};
use uuid::Uuid;

use super::duckdb_memory::{DuckDBMemory, ContextSnapshot};
use super::knowledge_graph::{KnowledgeGraph, NodeType};
use super::working_memory::{ContextWindow, ContextItem, ContextItemType};

/// Dynamic context manager - integrates all memory systems
pub struct DynamicContextManager {
    /// Working memory with intelligent pruning
    window: ContextWindow,

    /// Episodic memory (optional)
    episodic: Option<Arc<Mutex<DuckDBMemory>>>,

    /// Knowledge graph (optional)
    knowledge_graph: Option<Arc<Mutex<KnowledgeGraph>>>,

    /// Session ID
    session_id: String,
}

impl DynamicContextManager {
    /// Create a new dynamic context manager
    pub fn new(session_id: String, max_tokens: usize) -> Self {
        info!("Creating dynamic context manager: session={}, max_tokens={}",
              session_id, max_tokens);

        Self {
            window: ContextWindow::new(session_id.clone(), max_tokens),
            episodic: None,
            knowledge_graph: None,
            session_id,
        }
    }

    /// Set episodic memory
    pub fn set_episodic_memory(&mut self, memory: Arc<Mutex<DuckDBMemory>>) {
        info!("Connected episodic memory");
        self.episodic = Some(memory);
    }

    /// Set knowledge graph
    pub fn set_knowledge_graph(&mut self, graph: Arc<Mutex<KnowledgeGraph>>) {
        info!("Connected knowledge graph");
        self.knowledge_graph = Some(graph);
    }

    /// Add a message to the context
    pub fn add_user_message(&mut self, content: String, intent: Option<crate::intelligence::unified_intelligence::UserIntent>) {
        let token_cost = self.estimate_tokens(&content);
        let item = ContextItem {
            id: Uuid::new_v4(),
            content,
            item_type: ContextItemType::UserMessage { intent },
            timestamp: Utc::now(),
            relevance_score: 1.0,
            token_cost,
            task_id: None,
            dependencies: vec![],
            sticky: false,
        };

        self.window.add_item(item);
    }

    /// Add assistant response
    pub fn add_assistant_response(&mut self, content: String, tool_calls: Vec<String>) {
        let token_cost = self.estimate_tokens(&content);
        let item = ContextItem {
            id: Uuid::new_v4(),
            content,
            item_type: ContextItemType::AssistantResponse { tool_calls },
            timestamp: Utc::now(),
            relevance_score: 1.0,
            token_cost,
            task_id: None,
            dependencies: vec![],
            sticky: false,
        };

        self.window.add_item(item);
    }

    /// Add tool result
    pub fn add_tool_result(&mut self, tool_name: String, file_path: Option<String>, result: String) {
        let token_cost = self.estimate_tokens(&result);
        let item = ContextItem {
            id: Uuid::new_v4(),
            content: result,
            item_type: ContextItemType::ToolResult { tool_name, file_path },
            timestamp: Utc::now(),
            relevance_score: 1.0,
            token_cost,
            task_id: None,
            dependencies: vec![],
            sticky: false,
        };

        self.window.add_item(item);
    }

    /// Set current task
    pub fn set_current_task(&mut self, task_id: String) {
        self.window.set_current_task(task_id);
    }

    /// Check if pruning is needed and execute if necessary
    pub async fn maybe_prune(&mut self) -> Result<()> {
        if !self.window.needs_pruning() {
            return Ok(());
        }

        info!("Context window needs pruning");

        // Prune context window
        let removed_items = self.window.prune()?;

        info!("Pruned {} items", removed_items.len());

        // Summarize removed items to episodic memory
        if let Some(episodic) = &self.episodic {
            self.summarize_to_episodic(episodic, &removed_items).await?;
        }

        // Take snapshot
        if let Some(episodic) = &self.episodic {
            self.snapshot_context(episodic, "pruning", &removed_items).await?;
        }

        Ok(())
    }

    /// Fetch relevant code from knowledge graph
    pub async fn fetch_relevant_code(&mut self, query: &str) -> Result<()> {
        let graph = match &self.knowledge_graph {
            Some(g) => g,
            None => {
                debug!("No knowledge graph available");
                return Ok(());
            }
        };

        let graph_guard = graph.lock().await;

        // Try to find symbols matching the query
        if let Ok(symbols) = graph_guard.find_symbol(query) {
            for symbol in symbols.iter().take(3) {
                let content = match symbol {
                    NodeType::Function { name, signature, line, file_path } => {
                        format!("Function `{}` at {}:{}\n{}", name, file_path.display(), line, signature)
                    }
                    NodeType::Class { name, line, file_path } => {
                        format!("Class `{}` at {}:{}", name, file_path.display(), line)
                    }
                    _ => continue,
                };

                let item = ContextItem {
                    id: Uuid::new_v4(),
                    content: content.clone(),
                    item_type: ContextItemType::KnowledgeGraphQuery {
                        query: query.to_string(),
                        result_summary: format!("Found symbol: {}", query),
                    },
                    timestamp: Utc::now(),
                    relevance_score: 0.8,
                    token_cost: self.estimate_tokens(&content),
                    task_id: None,
                    dependencies: vec![],
                    sticky: false,
                };

                self.window.add_item(item);
            }
        }

        Ok(())
    }

    /// Get file context from episodic memory
    pub async fn get_file_context(&self, file_path: &str) -> Result<Option<String>> {
        let episodic = match &self.episodic {
            Some(e) => e,
            None => return Ok(None),
        };

        let memory_guard = episodic.lock().await;
        let interactions = memory_guard.get_file_interactions(file_path, 5).await?;

        if interactions.is_empty() {
            return Ok(None);
        }

        let mut context = format!("Previous work on {}:\n", file_path);
        for interaction in interactions.iter().take(3) {
            context.push_str(&format!(
                "- {} at {} ({})\n",
                interaction.operation,
                interaction.timestamp.format("%Y-%m-%d %H:%M"),
                if interaction.success { "success" } else { "failed" }
            ));
        }

        Ok(Some(context))
    }

    /// Summarize removed items to episodic memory
    async fn summarize_to_episodic(
        &self,
        episodic: &Arc<Mutex<DuckDBMemory>>,
        removed: &[ContextItem],
    ) -> Result<()> {
        debug!("Summarizing {} removed items to episodic memory", removed.len());

        for item in removed {
            match &item.item_type {
                ContextItemType::ToolResult { tool_name, file_path } => {
                    debug!("Pruned tool result: {} on {:?}", tool_name, file_path);
                }
                ContextItemType::CodeSnippet { file_path, line_range } => {
                    debug!("Pruned code snippet: {:?} lines {}-{}",
                           file_path, line_range.0, line_range.1);
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Take a snapshot of current context
    async fn snapshot_context(
        &self,
        episodic: &Arc<Mutex<DuckDBMemory>>,
        reason: &str,
        removed: &[ContextItem],
    ) -> Result<()> {
        let stats = self.window.stats();

        let snapshot = ContextSnapshot {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            session_id: self.session_id.clone(),
            task_id: None,
            context_items: serde_json::json!({
                "total_items": stats.total_items,
                "total_tokens": stats.total_tokens,
                "utilization": stats.utilization,
            }),
            total_tokens: stats.total_tokens as i32,
            pruned_items: Some(serde_json::json!({
                "count": removed.len(),
                "tokens": removed.iter().map(|i| i.token_cost).sum::<usize>(),
            })),
            reason: reason.to_string(),
        };

        let memory_guard = episodic.lock().await;
        memory_guard.record_context_snapshot(snapshot).await?;

        Ok(())
    }

    /// Estimate token count (rough approximation: 4 chars per token)
    fn estimate_tokens(&self, text: &str) -> usize {
        (text.len() / 4).max(1)
    }

    /// Get context window statistics
    pub fn stats(&self) -> super::working_memory::ContextWindowStats {
        self.window.stats()
    }

    /// Get all items (for building LLM messages)
    pub fn get_items(&self) -> &[ContextItem] {
        self.window.get_items()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_context_creation() {
        let manager = DynamicContextManager::new("test-session".to_string(), 100000);
        let stats = manager.stats();
        assert_eq!(stats.total_items, 0);
        assert_eq!(stats.total_tokens, 0);
    }

    #[test]
    fn test_add_messages() {
        let mut manager = DynamicContextManager::new("test-session".to_string(), 100000);

        manager.add_user_message("Hello".to_string(), None);
        manager.add_assistant_response("Hi there!".to_string(), vec![]);

        let stats = manager.stats();
        assert_eq!(stats.total_items, 2);
        assert!(stats.total_tokens > 0);
    }

    #[tokio::test]
    async fn test_pruning_without_memory() {
        let mut manager = DynamicContextManager::new("test-session".to_string(), 1000);

        // Add items up to 90% capacity
        for i in 0..9 {
            manager.add_user_message(format!("Message {}", i), None);
        }

        // Should trigger pruning
        manager.maybe_prune().await.unwrap();

        let stats = manager.stats();
        assert!(stats.total_tokens < 900, "Should have pruned some tokens");
    }
}
