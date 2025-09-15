use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// use crate::agent::conversation::{Message, MessageRole}; // For future integration
use crate::intelligence::{IntelligenceEngine, tools::IntelligenceTools};
use crate::semantic_search::SemanticCodeSearch;

/// Dynamic Context Manager - An intelligent agent that actively manages context
/// This is our innovation: instead of static context or sub-agents, we have a
/// smart context manager that dynamically adjusts what's in the working memory
pub struct DynamicContextManager {
    /// Intelligence engine for understanding code relationships
    intelligence: Arc<IntelligenceEngine>,
    /// Semantic search for finding relevant code
    search: Arc<RwLock<SemanticCodeSearch>>,
    /// Current working context
    working_context: RwLock<WorkingContext>,
    /// Context cache for quick retrieval
    context_cache: RwLock<ContextCache>,
    /// Prediction model for anticipating needs
    predictor: ContextPredictor,
    /// Configuration
    config: ContextConfig,
}

/// The active working context - what's currently "on the desk"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingContext {
    /// Current task being worked on
    pub current_task: String,
    /// Active context items with relevance scores
    pub active_items: HashMap<ContextItemId, ContextItem>,
    /// Total token usage
    pub token_usage: usize,
    /// Recent access patterns for learning
    pub access_history: VecDeque<ContextAccess>,
    /// Current focus point (file:line)
    pub focus_point: Option<FocusPoint>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextItemId(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    pub id: ContextItemId,
    pub item_type: ContextItemType,
    pub content: String,
    pub relevance_score: f32,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub access_count: usize,
    pub token_size: usize,
    pub relationships: Vec<ContextItemId>,
}

impl ContextItem {
    /// Generate a summary of this context item for display
    pub fn summary(&self) -> String {
        match &self.item_type {
            ContextItemType::FileContent { path, lines } => {
                if let Some((start, end)) = lines {
                    format!("{}:{}-{}", path, start, end)
                } else {
                    path.clone()
                }
            }
            ContextItemType::FunctionDefinition { name, file } => {
                format!("Function {} in {}", name, file)
            }
            ContextItemType::ClassDefinition { name, file } => {
                format!("Class {} in {}", name, file)
            }
            ContextItemType::Documentation { topic } => {
                format!("Docs: {}", topic)
            }
            ContextItemType::ErrorContext { error, .. } => {
                format!("Error: {}", error)
            }
            ContextItemType::TestCase { name, file } => {
                format!("Test {} in {}", name, file)
            }
            ContextItemType::Dependency { package, version } => {
                format!("{}@{}", package, version)
            }
            ContextItemType::GitHistory { file, .. } => {
                format!("Git history for {}", file)
            }
            ContextItemType::Conversation { .. } => {
                "Previous conversation context".to_string()
            }
            ContextItemType::TaskRequirement { requirement } => {
                format!("Requirement: {}", requirement)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextItemType {
    FileContent { path: String, lines: Option<(usize, usize)> },
    FunctionDefinition { name: String, file: String },
    ClassDefinition { name: String, file: String },
    Documentation { topic: String },
    ErrorContext { error: String, stack_trace: Vec<String> },
    TestCase { name: String, file: String },
    Dependency { package: String, version: String },
    GitHistory { file: String, commits: Vec<String> },
    Conversation { messages: Vec<String> },
    TaskRequirement { requirement: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusPoint {
    pub file: String,
    pub line: Option<usize>,
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAccess {
    pub item_id: ContextItemId,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: AccessAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessAction {
    Read,
    Modified,
    Referenced,
    Searched,
}

/// Cache for quick context retrieval
pub struct ContextCache {
    /// Recently evicted items that might be needed again
    evicted_items: VecDeque<ContextItem>,
    /// Preloaded items likely to be needed
    prefetch_queue: VecDeque<ContextItem>,
    /// Relationship graph for finding related context
    relationship_graph: HashMap<ContextItemId, HashSet<ContextItemId>>,
}

/// Predicts what context will be needed next
pub struct ContextPredictor {
    /// Patterns learned from usage
    learned_patterns: HashMap<String, Vec<PredictionPattern>>,
    /// Current predictions
    predictions: RwLock<Vec<ContextPrediction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionPattern {
    pub trigger: String,
    pub likely_needs: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPrediction {
    pub item_type: String,
    pub reason: String,
    pub confidence: f32,
    pub predicted_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Maximum tokens in working context
    pub max_tokens: usize,
    /// Minimum relevance score to keep
    pub min_relevance: f32,
    /// How many items to prefetch
    pub prefetch_count: usize,
    /// How many evicted items to keep
    pub eviction_cache_size: usize,
    /// Enable predictive loading
    pub predictive_loading: bool,
    /// Aggressiveness of pruning (0.0 - 1.0)
    pub pruning_aggressiveness: f32,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_tokens: 8000,
            min_relevance: 0.3,
            prefetch_count: 5,
            eviction_cache_size: 20,
            predictive_loading: true,
            pruning_aggressiveness: 0.5,
        }
    }
}

impl DynamicContextManager {
    pub fn new(
        intelligence: Arc<IntelligenceEngine>,
        search: Arc<RwLock<SemanticCodeSearch>>,
    ) -> Self {
        Self {
            intelligence,
            search,
            working_context: RwLock::new(WorkingContext {
                current_task: String::new(),
                active_items: HashMap::new(),
                token_usage: 0,
                access_history: VecDeque::with_capacity(100),
                focus_point: None,
            }),
            context_cache: RwLock::new(ContextCache {
                evicted_items: VecDeque::with_capacity(20),
                prefetch_queue: VecDeque::with_capacity(10),
                relationship_graph: HashMap::new(),
            }),
            predictor: ContextPredictor {
                learned_patterns: HashMap::new(),
                predictions: RwLock::new(Vec::new()),
            },
            config: ContextConfig::default(),
        }
    }

    /// Update context based on current activity
    pub async fn update_context(&self, activity: &str) -> Result<ContextUpdate> {
        info!("Dynamically updating context based on: {}", activity);

        let mut update = ContextUpdate {
            added: Vec::new(),
            removed: Vec::new(),
            relevance_changes: Vec::new(),
        };

        // Step 1: Analyze what the activity needs
        let needed_context = self.analyze_context_needs(activity).await?;

        // Step 2: Remove irrelevant context
        let removed = self.prune_irrelevant_context(&needed_context).await?;
        update.removed = removed;

        // Step 3: Add newly relevant context
        let added = self.fetch_relevant_context(&needed_context).await?;
        update.added = added;

        // Step 4: Adjust relevance scores
        self.update_relevance_scores(activity).await?;

        // Step 5: Predictive prefetching
        if self.config.predictive_loading {
            self.prefetch_predicted_context(activity).await?;
        }

        // Step 6: Ensure we're within token limits
        self.enforce_token_limits().await?;

        Ok(update)
    }

    /// Analyze what context is needed for current activity
    async fn analyze_context_needs(&self, activity: &str) -> Result<ContextNeeds> {
        let mut needs = ContextNeeds {
            required_files: Vec::new(),
            required_symbols: Vec::new(),
            required_types: Vec::new(),
            search_queries: Vec::new(),
        };

        // Use intelligence to understand the activity
        let insight = self.intelligence.get_development_context(activity).await;

        // Extract file needs
        needs.required_files = insight.key_files.iter()
            .map(|f| f.path.clone())
            .collect();

        // Determine needed context types based on activity
        let activity_lower = activity.to_lowercase();

        if activity_lower.contains("error") || activity_lower.contains("bug") {
            needs.required_types.push(ContextItemType::ErrorContext {
                error: String::new(),
                stack_trace: Vec::new(),
            });
        }

        if activity_lower.contains("test") {
            needs.required_types.push(ContextItemType::TestCase {
                name: String::new(),
                file: String::new(),
            });
        }

        if activity_lower.contains("implement") || activity_lower.contains("create") {
            needs.required_types.push(ContextItemType::TaskRequirement {
                requirement: String::new(),
            });
        }

        // Generate search queries for finding relevant code
        if activity_lower.contains("function") || activity_lower.contains("method") {
            needs.search_queries.push(format!("function {}", extract_identifier(activity)));
        }

        Ok(needs)
    }

    /// Remove context that's no longer relevant
    async fn prune_irrelevant_context(&self, needs: &ContextNeeds) -> Result<Vec<ContextItemId>> {
        let mut removed = Vec::new();
        let mut context = self.working_context.write().await;

        // Calculate relevance threshold based on aggressiveness
        let threshold = self.config.min_relevance +
            (1.0 - self.config.min_relevance) * self.config.pruning_aggressiveness;

        // Identify items to remove
        let items_to_remove: Vec<ContextItemId> = context.active_items
            .iter()
            .filter_map(|(id, item)| {
                // Don't remove recently accessed items
                let age = chrono::Utc::now() - item.last_accessed;
                if age.num_seconds() < 30 {
                    return None;
                }

                // Don't remove required files
                if let ContextItemType::FileContent { path, .. } = &item.item_type {
                    if needs.required_files.contains(path) {
                        return None;
                    }
                }

                // Remove if relevance is too low
                if item.relevance_score < threshold {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();

        // Move items to eviction cache
        let mut cache = self.context_cache.write().await;
        for id in &items_to_remove {
            if let Some(item) = context.active_items.remove(id) {
                context.token_usage -= item.token_size;

                // Add to eviction cache for potential retrieval
                cache.evicted_items.push_front(item.clone());
                if cache.evicted_items.len() > self.config.eviction_cache_size {
                    cache.evicted_items.pop_back();
                }

                removed.push(id.clone());
            }
        }

        info!("Pruned {} irrelevant context items", removed.len());
        Ok(removed)
    }

    /// Fetch newly relevant context
    async fn fetch_relevant_context(&self, needs: &ContextNeeds) -> Result<Vec<ContextItemId>> {
        let mut added = Vec::new();
        let mut context = self.working_context.write().await;
        let mut cache = self.context_cache.write().await;

        // Check eviction cache first
        for file in &needs.required_files {
            let mut found_in_cache = false;

            // Look in evicted items
            if let Some(pos) = cache.evicted_items.iter().position(|item| {
                matches!(&item.item_type, ContextItemType::FileContent { path, .. } if path == file)
            }) {
                let item = cache.evicted_items.remove(pos).unwrap();
                let id = item.id.clone();
                context.active_items.insert(id.clone(), item);
                added.push(id);
                found_in_cache = true;
            }

            // If not in cache, fetch from file system
            if !found_in_cache {
                if let Ok(content) = tokio::fs::read_to_string(file).await {
                    let id = ContextItemId(format!("file:{}", file));
                    let item = ContextItem {
                        id: id.clone(),
                        item_type: ContextItemType::FileContent {
                            path: file.clone(),
                            lines: None,
                        },
                        content: content.clone(),
                        relevance_score: 1.0, // High relevance for requested files
                        last_accessed: chrono::Utc::now(),
                        access_count: 1,
                        token_size: estimate_tokens(&content),
                        relationships: Vec::new(),
                    };

                    context.token_usage += item.token_size;
                    context.active_items.insert(id.clone(), item);
                    added.push(id);
                }
            }
        }

        // Search for relevant code using semantic search
        for query in &needs.search_queries {
            let mut search = self.search.write().await;
            if let Ok((results, _metrics)) = search.search(query, 3).await {
                for result in results {
                    let id = ContextItemId(format!("search:{}:{}",
                        result.file_path.display(), result.chunk.start_line));

                    if !context.active_items.contains_key(&id) {
                        let item = ContextItem {
                            id: id.clone(),
                            item_type: ContextItemType::FileContent {
                                path: result.file_path.to_string_lossy().into_owned(),
                                lines: Some((result.chunk.start_line, result.chunk.end_line)),
                            },
                            content: result.chunk.content.clone(),
                            relevance_score: result.similarity_score,
                            last_accessed: chrono::Utc::now(),
                            access_count: 1,
                            token_size: estimate_tokens(&result.chunk.content),
                            relationships: Vec::new(),
                        };

                        context.token_usage += item.token_size;
                        context.active_items.insert(id.clone(), item);
                        added.push(id);
                    }
                }
            }
        }

        info!("Added {} relevant context items", added.len());
        Ok(added)
    }

    /// Update relevance scores based on current activity
    async fn update_relevance_scores(&self, activity: &str) -> Result<()> {
        let mut context = self.working_context.write().await;

        // Decay all scores slightly
        for item in context.active_items.values_mut() {
            item.relevance_score *= 0.95;
        }

        // Boost scores for items related to current activity
        let activity_lower = activity.to_lowercase();
        for item in context.active_items.values_mut() {
            match &item.item_type {
                ContextItemType::FileContent { path, .. } => {
                    if activity_lower.contains(&path.to_lowercase()) {
                        item.relevance_score = (item.relevance_score + 0.2).min(1.0);
                    }
                }
                ContextItemType::FunctionDefinition { name, .. } => {
                    if activity_lower.contains(&name.to_lowercase()) {
                        item.relevance_score = (item.relevance_score + 0.3).min(1.0);
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Prefetch context predicted to be needed soon
    async fn prefetch_predicted_context(&self, activity: &str) -> Result<()> {
        // Look for patterns in activity
        let predictions = self.predictor.predict_needs(activity).await?;

        let _cache = self.context_cache.write().await;

        for prediction in predictions.iter().take(self.config.prefetch_count) {
            // Prefetch predicted items into cache
            debug!("Prefetching {} (confidence: {})",
                prediction.item_type, prediction.confidence);

            // This would fetch the actual content and add to prefetch queue
            // Implementation depends on specific prediction type
        }

        Ok(())
    }

    /// Ensure we stay within token limits
    async fn enforce_token_limits(&self) -> Result<()> {
        let mut context = self.working_context.write().await;

        while context.token_usage > self.config.max_tokens {
            // Find least relevant item
            let least_relevant = context.active_items
                .iter()
                .min_by(|a, b| {
                    a.1.relevance_score.partial_cmp(&b.1.relevance_score).unwrap()
                })
                .map(|(id, _)| id.clone());

            if let Some(id) = least_relevant {
                if let Some(item) = context.active_items.remove(&id) {
                    context.token_usage -= item.token_size;
                    warn!("Evicted {} to stay within token limit", id.0);

                    // Add to eviction cache
                    let mut cache = self.context_cache.write().await;
                    cache.evicted_items.push_front(item);
                    if cache.evicted_items.len() > self.config.eviction_cache_size {
                        cache.evicted_items.pop_back();
                    }
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    /// Track context access for learning patterns
    pub async fn track_access(&self, item_id: &ContextItemId, action: AccessAction) {
        let mut context = self.working_context.write().await;

        // Update item metadata
        if let Some(item) = context.active_items.get_mut(item_id) {
            item.last_accessed = chrono::Utc::now();
            item.access_count += 1;

            // Boost relevance for accessed items
            item.relevance_score = (item.relevance_score + 0.1).min(1.0);
        }

        // Record access history
        context.access_history.push_front(ContextAccess {
            item_id: item_id.clone(),
            timestamp: chrono::Utc::now(),
            action,
        });

        // Keep history bounded
        if context.access_history.len() > 100 {
            context.access_history.pop_back();
        }
    }

    /// Set the current focus point
    pub async fn set_focus(&self, file: String, line: Option<usize>, symbol: Option<String>) {
        let mut context = self.working_context.write().await;
        context.focus_point = Some(FocusPoint { file, line, symbol });
    }

    /// Get current working context summary
    pub async fn get_context_summary(&self) -> ContextSummary {
        let context = self.working_context.read().await;

        ContextSummary {
            total_items: context.active_items.len(),
            token_usage: context.token_usage,
            token_limit: self.config.max_tokens,
            utilization: (context.token_usage as f32 / self.config.max_tokens as f32) * 100.0,
            focus: context.focus_point.clone(),
            top_items: context.active_items
                .values()
                .map(|item| (item.id.clone(), item.relevance_score))
                .collect(),
        }
    }

    /// Get the actual context content for LLM
    pub async fn get_context_for_llm(&self) -> String {
        let context = self.working_context.read().await;
        let mut output = String::new();

        // Sort by relevance
        let mut items: Vec<_> = context.active_items.values().collect();
        items.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        // Build context string
        for item in items {
            match &item.item_type {
                ContextItemType::FileContent { path, lines } => {
                    output.push_str(&format!("\n## File: {}\n", path));
                    if let Some((start, end)) = lines {
                        output.push_str(&format!("Lines {}-{}:\n", start, end));
                    }
                    output.push_str(&item.content);
                    output.push_str("\n");
                }
                ContextItemType::FunctionDefinition { name, file } => {
                    output.push_str(&format!("\n## Function: {} (from {})\n", name, file));
                    output.push_str(&item.content);
                    output.push_str("\n");
                }
                ContextItemType::ErrorContext { error, .. } => {
                    output.push_str(&format!("\n## Error Context: {}\n", error));
                    output.push_str(&item.content);
                    output.push_str("\n");
                }
                _ => {
                    // Add other types as needed
                }
            }
        }

        output
    }

    /// Get the most relevant context items up to a limit
    pub async fn get_relevant_context(&self, limit: usize) -> Result<Vec<ContextItem>> {
        let context = self.working_context.read().await;

        // Sort by relevance and recency
        let mut items: Vec<_> = context.active_items.values().cloned().collect();
        items.sort_by(|a, b| {
            // First by relevance, then by last accessed
            match b.relevance_score.partial_cmp(&a.relevance_score).unwrap() {
                std::cmp::Ordering::Equal => b.last_accessed.cmp(&a.last_accessed),
                other => other,
            }
        });

        // Take up to limit items
        Ok(items.into_iter().take(limit).collect())
    }

    /// Track file access by path (helper for tool execution)
    pub async fn track_file_access(&self, path: &str, action: AccessAction) {
        // Create a context item ID from the path
        let item_id = ContextItemId(format!("file:{}", path));

        // Check if we have this file in context
        let mut context = self.working_context.write().await;

        // If not in context, add it
        if !context.active_items.contains_key(&item_id) {
            let item = ContextItem {
                id: item_id.clone(),
                item_type: ContextItemType::FileContent {
                    path: path.to_string(),
                    lines: None,
                },
                content: String::new(), // Content would be loaded when needed
                relevance_score: 0.5,
                last_accessed: chrono::Utc::now(),
                access_count: 1,
                token_size: 0,
                relationships: Vec::new(),
            };
            context.active_items.insert(item_id.clone(), item);
        }

        // Now track the access
        drop(context); // Release write lock
        self.track_access(&item_id, action).await;
    }
}

/// Needs analysis result
#[derive(Debug)]
struct ContextNeeds {
    required_files: Vec<String>,
    required_symbols: Vec<String>,
    required_types: Vec<ContextItemType>,
    search_queries: Vec<String>,
}

/// Update result
#[derive(Debug, Serialize, Deserialize)]
pub struct ContextUpdate {
    pub added: Vec<ContextItemId>,
    pub removed: Vec<ContextItemId>,
    pub relevance_changes: Vec<(ContextItemId, f32)>,
}

/// Context summary for UI
#[derive(Debug, Serialize, Deserialize)]
pub struct ContextSummary {
    pub total_items: usize,
    pub token_usage: usize,
    pub token_limit: usize,
    pub utilization: f32,
    pub focus: Option<FocusPoint>,
    pub top_items: Vec<(ContextItemId, f32)>,
}

impl ContextPredictor {
    async fn predict_needs(&self, activity: &str) -> Result<Vec<ContextPrediction>> {
        let mut predictions = Vec::new();

        // Simple pattern matching for now
        // In production, this would use learned patterns
        let activity_lower = activity.to_lowercase();

        if activity_lower.contains("test") {
            predictions.push(ContextPrediction {
                item_type: "test_files".to_string(),
                reason: "Activity mentions testing".to_string(),
                confidence: 0.8,
                predicted_at: chrono::Utc::now(),
            });
        }

        if activity_lower.contains("error") || activity_lower.contains("fix") {
            predictions.push(ContextPrediction {
                item_type: "error_logs".to_string(),
                reason: "Activity suggests debugging".to_string(),
                confidence: 0.7,
                predicted_at: chrono::Utc::now(),
            });
        }

        Ok(predictions)
    }
}

/// Helper functions
fn estimate_tokens(content: &str) -> usize {
    // Rough estimate: 4 chars = 1 token
    content.len() / 4
}

fn extract_identifier(text: &str) -> String {
    // Extract likely identifier from text
    // This is simplified - would use proper parsing in production
    text.split_whitespace()
        .find(|w| w.chars().all(|c| c.is_alphanumeric() || c == '_'))
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dynamic_context_management() {
        // Would need proper mocks
        todo!("Implement dynamic context tests")
    }
}