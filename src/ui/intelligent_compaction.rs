use anyhow::Result;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use crate::ui::Message;
use crate::ui::compaction_analyzer::{ConversationAnalyzer, CompactionContext};
use crate::intelligence::{IntelligenceEngine, tools::IntelligenceTools};

/// Intelligence-enhanced compaction context with deeper insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentCompactionContext {
    /// Base context from conversation analysis
    pub base_context: CompactionContext,
    /// Intelligence insights about development context
    pub development_phase: String,
    /// Active development story/narrative
    pub active_story: String,
    /// Predicted files that might need attention
    pub predicted_files: Vec<String>,
    /// Intelligent suggestions for what to preserve
    pub preservation_priorities: Vec<String>,
    /// Cross-project patterns that might apply
    pub relevant_patterns: Vec<String>,
    /// Intelligence confidence score
    pub intelligence_confidence: f64,
    /// Project momentum insights
    pub momentum_insights: Vec<String>,
}

/// Intelligence-enhanced conversation analyzer
pub struct IntelligentCompactionAnalyzer {
    base_analyzer: ConversationAnalyzer,
    intelligence: Option<Arc<IntelligenceEngine>>,
}

impl IntelligentCompactionAnalyzer {
    /// Create new intelligent analyzer without intelligence (fallback mode)
    pub fn new() -> Result<Self> {
        Ok(Self {
            base_analyzer: ConversationAnalyzer::new()?,
            intelligence: None,
        })
    }

    /// Create with intelligence engine integration
    pub fn with_intelligence(intelligence: Arc<IntelligenceEngine>) -> Result<Self> {
        Ok(Self {
            base_analyzer: ConversationAnalyzer::new()?,
            intelligence: Some(intelligence),
        })
    }

    /// Analyze conversation with intelligence enhancement
    pub async fn analyze_conversation(&self, messages: &[Message]) -> Result<IntelligentCompactionContext> {
        info!("Starting intelligent compaction analysis of {} messages", messages.len());

        // Get base context analysis
        let base_context = self.base_analyzer.analyze_conversation(messages)?;

        // If no intelligence engine, return base context wrapped
        let Some(intelligence) = &self.intelligence else {
            debug!("No intelligence engine available, using base analysis only");
            return Ok(IntelligentCompactionContext {
                base_context,
                development_phase: "Unknown".to_string(),
                active_story: "No intelligence context available".to_string(),
                predicted_files: Vec::new(),
                preservation_priorities: Vec::new(),
                relevant_patterns: Vec::new(),
                intelligence_confidence: 0.0,
                momentum_insights: Vec::new(),
            });
        };

        info!("Enhancing compaction with intelligence insights");

        // Get intelligence-enhanced insights
        let development_context = intelligence.get_development_context(&base_context.current_task).await;

        // Extract predicted files from intelligence
        let predicted_files = if !base_context.recent_files.is_empty() {
            intelligence.predict_file_changes(&base_context.recent_files[0]).await
                .unwrap_or_else(|e| {
                    warn!("Failed to get file predictions: {}", e);
                    Vec::new()
                })
        } else {
            Vec::new()
        };

        // Get relevant patterns from intelligence memory
        let relevant_patterns = intelligence.get_relevant_patterns(&base_context.current_task, "current").await
            .unwrap_or_else(|e| {
                warn!("Failed to get relevant patterns: {}", e);
                Vec::new()
            });

        // Generate preservation priorities using intelligence
        let preservation_priorities = self.generate_intelligent_priorities(
            &base_context,
            &development_context,
            &predicted_files
        ).await;

        // Get project momentum insights
        let momentum = intelligence.get_project_momentum().await;
        let momentum_insights = vec![
            momentum.recent_focus.clone(),
            momentum.architectural_direction.clone(),
        ];

        let enhanced_context = IntelligentCompactionContext {
            base_context,
            development_phase: development_context.development_phase,
            active_story: development_context.active_story,
            predicted_files,
            preservation_priorities,
            relevant_patterns,
            intelligence_confidence: development_context.confidence,
            momentum_insights,
        };

        info!("Intelligence analysis complete - confidence: {:.1}%, predicted files: {}, patterns: {}",
              enhanced_context.intelligence_confidence * 100.0,
              enhanced_context.predicted_files.len(),
              enhanced_context.relevant_patterns.len());

        Ok(enhanced_context)
    }

    /// Generate intelligent preservation priorities
    async fn generate_intelligent_priorities(
        &self,
        base_context: &CompactionContext,
        development_context: &crate::intelligence::ContextualInsight,
        predicted_files: &[String],
    ) -> Vec<String> {
        let mut priorities = Vec::new();

        // High priority: Current task context
        if !base_context.current_task.is_empty() {
            priorities.push(format!("Current task progress: {}", base_context.current_task));
        }

        // High priority: Files intelligence thinks are important
        for file_context in &development_context.key_files {
            if file_context.relevance.total_score() > 0.5 {
                priorities.push(format!("Critical context for: {} ({})",
                    file_context.path, file_context.purpose));
            }
        }

        // Medium priority: Predicted file relationships
        if !predicted_files.is_empty() {
            priorities.push(format!("Files that may need changes: {}",
                predicted_files.join(", ")));
        }

        // Medium priority: Architectural decisions
        for decision in &development_context.architectural_context {
            priorities.push(format!("Architectural decision: {}",
                decision.decision.chars().take(80).collect::<String>()));
        }

        // Medium priority: Recent patterns from intelligence
        for pattern in &development_context.recent_patterns {
            if pattern.confidence > 0.7 {
                priorities.push(format!("Important pattern: {} ({}% confidence)",
                    pattern.description, (pattern.confidence * 100.0) as u8));
            }
        }

        // Low priority: Suggested next actions
        for action in &development_context.suggested_next_actions {
            if action.confidence > 0.6 {
                priorities.push(format!("Suggested action: {}", action.description));
            }
        }

        // Sort by intelligence-determined importance and limit
        priorities.truncate(8);
        priorities
    }
}

impl IntelligentCompactionContext {
    /// Generate hyper-intelligent compaction prompt
    pub fn generate_intelligent_prompt(&self, _conversation: &str) -> String {
        let mut prompt = String::new();

        // Start with intelligence-enhanced task focus
        if self.intelligence_confidence > 0.5 {
            prompt.push_str(&format!(
                "Create an INTELLIGENT summary optimized for seamless continuation of current work.\n\n"
            ));

            prompt.push_str(&format!(
                "📊 INTELLIGENCE CONTEXT ({}% confidence):\n",
                (self.intelligence_confidence * 100.0) as u8
            ));

            prompt.push_str(&format!(
                "🎯 Development Phase: {}\n",
                self.development_phase
            ));

            prompt.push_str(&format!(
                "📖 Active Story: {}\n\n",
                self.active_story
            ));
        } else {
            // Fallback to base analysis if intelligence confidence is low
            return self.base_context.generate_smart_prompt(_conversation);
        }

        // Add intelligence-driven preservation priorities
        if !self.preservation_priorities.is_empty() {
            prompt.push_str("🧠 INTELLIGENT PRESERVATION PRIORITIES:\n");
            for (i, priority) in self.preservation_priorities.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, priority));
            }
            prompt.push_str("\n");
        }

        // Add predictive context
        if !self.predicted_files.is_empty() {
            prompt.push_str(&format!(
                "🔮 FILES LIKELY TO NEED CHANGES: {}\n\n",
                self.predicted_files.join(", ")
            ));
        }

        // Add learned patterns context
        if !self.relevant_patterns.is_empty() {
            prompt.push_str("🎓 RELEVANT LEARNED PATTERNS:\n");
            for pattern in &self.relevant_patterns {
                let truncated = pattern.chars().take(100).collect::<String>();
                prompt.push_str(&format!("• {}\n", truncated));
            }
            prompt.push_str("\n");
        }

        // Add momentum insights
        if !self.momentum_insights.is_empty() {
            prompt.push_str("🚀 PROJECT MOMENTUM:\n");
            for insight in &self.momentum_insights {
                if !insight.trim().is_empty() {
                    prompt.push_str(&format!("• {}\n", insight));
                }
            }
            prompt.push_str("\n");
        }

        // Add base context priorities with intelligence enhancement
        prompt.push_str("📁 KEY FILES AND CONTEXT:\n");
        if !self.base_context.recent_files.is_empty() {
            prompt.push_str(&format!(
                "• Primary files: {}\n",
                self.base_context.recent_files.join(", ")
            ));
        }

        if !self.base_context.active_tools.is_empty() {
            prompt.push_str(&format!(
                "• Tools in active use: {}\n",
                self.base_context.active_tools.join(", ")
            ));
        }

        if !self.base_context.unresolved_issues.is_empty() {
            prompt.push_str(&format!(
                "• Unresolved issues: {}\n",
                self.base_context.unresolved_issues.len()
            ));
        }
        prompt.push_str("\n");

        // Add project-specific intelligence enhancements
        if let Some(project_type) = &self.base_context.project_type {
            prompt.push_str(&format!("🏗️ {} PROJECT INTELLIGENCE:\n", project_type.to_uppercase()));
            match project_type.as_str() {
                "rust" => {
                    prompt.push_str("• Preserve: compilation states, cargo dependencies, performance implications\n");
                    prompt.push_str("• Focus: type safety, memory management patterns, concurrency issues\n");
                }
                "node" => {
                    prompt.push_str("• Preserve: package.json changes, npm/yarn states, async patterns\n");
                    prompt.push_str("• Focus: dependency management, build processes, testing strategies\n");
                }
                "python" => {
                    prompt.push_str("• Preserve: virtual env setup, import hierarchies, pip dependencies\n");
                    prompt.push_str("• Focus: package structure, testing frameworks, performance bottlenecks\n");
                }
                _ => {
                    prompt.push_str("• Preserve: build configurations, dependency states, test results\n");
                }
            }
            prompt.push_str("\n");
        }

        prompt.push_str("💡 COMPACTION STRATEGY:\n");
        prompt.push_str("1. Maintain ALL context needed for seamless work continuation\n");
        prompt.push_str("2. Preserve technical decision rationale and error states\n");
        prompt.push_str("3. Keep tool usage patterns and successful approaches\n");
        prompt.push_str("4. Condense repetitive exploration, expand successful patterns\n");
        prompt.push_str("5. Preserve file relationships and architectural insights\n\n");

        prompt.push_str("Generate a summary that feels like continuing the same session with full context.\n\n");
        prompt.push_str("Conversation to summarize:\n\n");

        prompt
    }

    /// Check if intelligent analysis was successful
    pub fn has_intelligence_insights(&self) -> bool {
        self.intelligence_confidence > 0.3 &&
        (!self.predicted_files.is_empty() ||
         !self.relevant_patterns.is_empty() ||
         !self.preservation_priorities.is_empty())
    }

    /// Get a human-readable summary of the analysis
    pub fn get_analysis_summary(&self) -> String {
        if self.has_intelligence_insights() {
            format!(
                "🧠 Intelligent analysis complete ({}% confidence)\n\
                📁 {} files analyzed, {} predictions\n\
                🎯 {} preservation priorities identified\n\
                🎓 {} relevant patterns found\n\
                Phase: {} | Story: {}",
                (self.intelligence_confidence * 100.0) as u8,
                self.base_context.recent_files.len(),
                self.predicted_files.len(),
                self.preservation_priorities.len(),
                self.relevant_patterns.len(),
                self.development_phase,
                if self.active_story.len() > 50 {
                    format!("{}...", self.active_story.chars().take(47).collect::<String>())
                } else {
                    self.active_story.clone()
                }
            )
        } else {
            format!(
                "📊 Basic analysis complete\n\
                📁 {} files, {} tools, task: {}",
                self.base_context.recent_files.len(),
                self.base_context.active_tools.len(),
                if self.base_context.current_task.len() > 40 {
                    format!("{}...", self.base_context.current_task.chars().take(37).collect::<String>())
                } else {
                    self.base_context.current_task.clone()
                }
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigManager;
    use crate::storage::DatabaseManager;

    #[tokio::test]
    async fn test_intelligent_compaction_without_intelligence() -> Result<()> {
        let analyzer = IntelligentCompactionAnalyzer::new()?;
        let messages = vec![
            Message::user("I'm implementing authentication in src/auth.rs".to_string()),
            Message::assistant("Let me help you with that.".to_string()),
        ];

        let context = analyzer.analyze_conversation(&messages).await?;

        // Should fall back to base analysis
        assert!(!context.base_context.current_task.is_empty());
        assert_eq!(context.intelligence_confidence, 0.0);
        assert!(!context.has_intelligence_insights());

        Ok(())
    }

    #[tokio::test]
    async fn test_intelligent_prompt_generation() -> Result<()> {
        // Create mock context with intelligence insights
        let base_context = CompactionContext {
            current_task: "Implementing authentication".to_string(),
            recent_files: vec!["src/auth.rs".to_string()],
            project_type: Some("rust".to_string()),
            ..Default::default()
        };

        let context = IntelligentCompactionContext {
            base_context,
            development_phase: "Feature Implementation".to_string(),
            active_story: "User authentication system".to_string(),
            predicted_files: vec!["src/models/user.rs".to_string()],
            preservation_priorities: vec!["JWT token handling".to_string()],
            relevant_patterns: vec!["Authentication pattern from session X".to_string()],
            intelligence_confidence: 0.85,
            momentum_insights: vec!["Security-focused development".to_string()],
        };

        let prompt = context.generate_intelligent_prompt("");

        assert!(prompt.contains("INTELLIGENCE CONTEXT"));
        assert!(prompt.contains("85% confidence"));
        assert!(prompt.contains("Feature Implementation"));
        assert!(prompt.contains("PRESERVATION PRIORITIES"));
        assert!(prompt.contains("FILES LIKELY TO NEED CHANGES"));
        assert!(prompt.contains("RUST PROJECT INTELLIGENCE"));

        Ok(())
    }
}