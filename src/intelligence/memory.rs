use crate::storage::DatabaseManager;
use anyhow::Result;
use super::{Pattern, Outcome};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Conversational Memory System - Learns from AI interactions
pub struct ConversationalMemorySystem {
    storage: DatabaseManager,
    pattern_cache: HashMap<String, Vec<Pattern>>,
    outcome_history: Vec<ConversationOutcome>,
}

/// Record of a conversation outcome for learning
#[derive(Debug, Clone)]
pub struct ConversationOutcome {
    pub files_included: Vec<String>,
    pub user_goal: String,
    pub outcome_quality: f64,
    pub missing_context_identified: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub query_type: String,
}

impl ConversationalMemorySystem {
    pub async fn new(storage: &DatabaseManager) -> Result<Self> {
        Ok(Self {
            storage: storage.clone(),
            pattern_cache: HashMap::new(),
            outcome_history: Vec::new(),
        })
    }

    /// Get patterns relevant to a query based on past conversations
    pub async fn get_relevant_patterns(&self, query: &str) -> MemoryInsight {
        let mut relevant_patterns = Vec::new();
        
        // Analyze query to determine type and context
        let query_type = self.classify_query(query);
        
        // Find similar successful conversations
        let similar_outcomes = self.find_similar_outcomes(query, &query_type);
        
        // Extract patterns from successful outcomes
        for outcome in similar_outcomes {
            if outcome.outcome_quality > 0.7 {
                // Create pattern from successful file combinations
                let pattern = Pattern {
                    pattern_type: "successful_context".to_string(),
                    description: format!("Files {} worked well for {}", 
                        outcome.files_included.join(", "), outcome.query_type),
                    confidence: outcome.outcome_quality,
                    occurrences: 1,
                };
                relevant_patterns.push(pattern);
            }
        }
        
        // Add general patterns from cache
        if let Some(cached_patterns) = self.pattern_cache.get(&query_type) {
            relevant_patterns.extend(cached_patterns.clone());
        }
        
        // Calculate overall confidence
        let confidence = if relevant_patterns.is_empty() {
            0.0
        } else {
            relevant_patterns.iter().map(|p| p.confidence).sum::<f64>() / relevant_patterns.len() as f64
        };
        
        MemoryInsight {
            patterns: relevant_patterns,
            confidence,
        }
    }
    
    /// Classify the type of query for pattern matching
    fn classify_query(&self, query: &str) -> String {
        let query_lower = query.to_lowercase();
        
        if query_lower.contains("implement") || query_lower.contains("add") {
            "implementation".to_string()
        } else if query_lower.contains("fix") || query_lower.contains("debug") {
            "debugging".to_string()
        } else if query_lower.contains("refactor") || query_lower.contains("improve") {
            "refactoring".to_string()
        } else if query_lower.contains("test") {
            "testing".to_string()
        } else if query_lower.contains("config") || query_lower.contains("setup") {
            "configuration".to_string()
        } else {
            "general".to_string()
        }
    }
    
    /// Find similar conversation outcomes
    fn find_similar_outcomes(&self, query: &str, query_type: &str) -> Vec<&ConversationOutcome> {
        let mut similar = Vec::new();
        
        for outcome in &self.outcome_history {
            let mut similarity_score = 0.0;
            
            // Type similarity
            if outcome.query_type == query_type {
                similarity_score += 0.5;
            }
            
            // Keyword similarity
            let query_words: Vec<&str> = query.split_whitespace().collect();
            let goal_words: Vec<&str> = outcome.user_goal.split_whitespace().collect();
            
            let common_words = query_words.iter()
                .filter(|word| goal_words.iter().any(|goal_word| goal_word.eq_ignore_ascii_case(word)))
                .count();
            
            if common_words > 0 {
                similarity_score += 0.3 * (common_words as f64 / query_words.len() as f64);
            }
            
            // Include if similarity is above threshold
            if similarity_score > 0.4 {
                similar.push(outcome);
            }
        }
        
        // Sort by similarity (approximated by outcome quality for now)
        similar.sort_by(|a, b| b.outcome_quality.partial_cmp(&a.outcome_quality).unwrap());
        similar.truncate(5); // Top 5 similar outcomes
        
        similar
    }

    /// Record the outcome of a conversation for learning
    pub async fn record_outcome(&self, files: &[String], outcome: Outcome) {
        let conversation_outcome = ConversationOutcome {
            files_included: files.to_vec(),
            user_goal: "Task completion".to_string(), // Would be extracted from conversation context
            outcome_quality: outcome.success_rating,
            missing_context_identified: outcome.identified_gaps,
            timestamp: Utc::now(),
            query_type: self.infer_query_type(files),
        };
        
        // In a full implementation, we'd store this in the database
        // For now, we'll just log it
        tracing::info!(
            "Conversation outcome recorded: {} files, quality: {:.2}, gaps: {}",
            conversation_outcome.files_included.len(),
            conversation_outcome.outcome_quality,
            conversation_outcome.missing_context_identified.len()
        );
        
        // Update learning patterns
        self.update_learning_patterns(&conversation_outcome).await;
    }
    
    /// Infer query type from files involved
    fn infer_query_type(&self, files: &[String]) -> String {
        for file in files {
            if file.contains("test") {
                return "testing".to_string();
            }
            if file.contains("config") {
                return "configuration".to_string();
            }
            if file.contains("ui/") || file.contains("tui/") {
                return "user_interface".to_string();
            }
            if file.contains("provider") {
                return "api_integration".to_string();
            }
            if file.contains("intelligence") {
                return "intelligence_engine".to_string();
            }
        }
        "general".to_string()
    }
    
    /// Update learning patterns based on conversation outcome
    async fn update_learning_patterns(&self, outcome: &ConversationOutcome) {
        // Extract successful patterns
        if outcome.outcome_quality > 0.7 {
            let pattern = Pattern {
                pattern_type: "file_combination".to_string(),
                description: format!("Files {} work well together for {}", 
                    outcome.files_included.join(", "), outcome.query_type),
                confidence: outcome.outcome_quality,
                occurrences: 1,
            };
            
            // In a full implementation, we'd update the database
            tracing::debug!("Learned successful pattern: {}", pattern.description);
        }
        
        // Learn from context gaps
        if !outcome.missing_context_identified.is_empty() {
            let gap_pattern = Pattern {
                pattern_type: "context_gap".to_string(),
                description: format!("Query type {} often needs additional context: {}", 
                    outcome.query_type, outcome.missing_context_identified.join(", ")),
                confidence: 0.8,
                occurrences: 1,
            };
            
            tracing::debug!("Learned context gap pattern: {}", gap_pattern.description);
        }
    }

    /// Get effectiveness metrics for different context strategies
    pub async fn get_effectiveness_metrics(&self) -> EffectivenessMetrics {
        // Analyze historical outcomes
        let total_outcomes = self.outcome_history.len();
        
        if total_outcomes == 0 {
            return EffectivenessMetrics {
                overall_success_rate: 0.0,
                context_accuracy_trend: vec![],
                optimal_context_size: 5, // Default recommendation
            };
        }
        
        // Calculate overall success rate
        let total_quality: f64 = self.outcome_history.iter()
            .map(|o| o.outcome_quality)
            .sum();
        let overall_success_rate = total_quality / total_outcomes as f64;
        
        // Calculate accuracy trend (last 10 conversations)
        let recent_outcomes = self.outcome_history.iter()
            .rev()
            .take(10)
            .collect::<Vec<_>>();
        
        let mut context_accuracy_trend = Vec::new();
        for outcome in recent_outcomes.iter() {
            context_accuracy_trend.push(outcome.outcome_quality);
        }
        
        // Determine optimal context size
        let mut size_quality = HashMap::new();
        for outcome in &self.outcome_history {
            let size = outcome.files_included.len();
            let entry = size_quality.entry(size).or_insert((0.0, 0));
            entry.0 += outcome.outcome_quality;
            entry.1 += 1;
        }
        
        let optimal_context_size = size_quality.iter()
            .map(|(size, (total_quality, count))| (*size, total_quality / *count as f64))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(size, _)| size)
            .unwrap_or(5);
        
        EffectivenessMetrics {
            overall_success_rate,
            context_accuracy_trend,
            optimal_context_size,
        }
    }
}

#[derive(Debug)]
pub struct MemoryInsight {
    pub patterns: Vec<Pattern>,
    pub confidence: f64,
}

#[derive(Debug)]
pub struct EffectivenessMetrics {
    pub overall_success_rate: f64,
    pub context_accuracy_trend: Vec<f64>,
    pub optimal_context_size: usize,
}