use async_trait::async_trait;
use super::{ContextualInsight, ImpactAnalysis, ContextSuggestions, Outcome, ProjectMomentum, CrossProjectInsight, AiConfiguration};

/// Tool interface for AI agents to interact with the Intelligence Engine
#[async_trait]
pub trait IntelligenceTools: Send + Sync {
    /// Get comprehensive context for current development work
    /// 
    /// This is the primary tool for AI agents to understand what's happening
    /// in the codebase and get relevant context for user requests.
    async fn get_development_context(&self, query: &str) -> ContextualInsight;

    /// Analyze the potential impact of changing specific files
    /// 
    /// Helps AI agents understand ripple effects and dependencies
    /// before suggesting code changes.
    async fn analyze_change_impact(&self, files: &[String]) -> ImpactAnalysis;

    /// Suggest additional context that might be helpful
    /// 
    /// When an AI agent has some files in context, this suggests
    /// what else might be relevant or missing.
    async fn suggest_missing_context(&self, current_files: &[String]) -> ContextSuggestions;

    /// Record the outcome of a conversation for learning
    /// 
    /// AI agents should call this after completing tasks to help
    /// the Intelligence Engine learn what context was effective.
    async fn track_conversation_outcome(&self, files: &[String], outcome: Outcome) -> ();

    /// Get overall project momentum and direction
    /// 
    /// Provides high-level context about where the project is heading
    /// and what patterns are emerging.
    async fn get_project_momentum(&self) -> ProjectMomentum;

    /// Add external project directory for cross-project intelligence
    /// 
    /// Allows AI agents to include context from other projects,
    /// following the Claude Code pattern of multi-directory sessions.
    async fn add_project_directory(&self, path: &str) -> Result<(), String>;

    /// Analyze patterns across multiple projects
    /// 
    /// Provides insights from other projects that might be relevant
    /// to the current development context.
    async fn analyze_cross_project_patterns(&self, query: &str) -> CrossProjectInsight;

    /// Load global and project-specific AI configuration
    /// 
    /// Discovers and loads AGENT.md, .cursorrules, and other AI config files
    /// to provide consistent context across tools.
    async fn load_ai_configuration(&self) -> AiConfiguration;
}

/// Example tool usage patterns for AI agents
pub mod usage_examples {
    use super::*;
    
    /// Example: Starting a conversation with a user request
    pub async fn handle_user_request(
        intelligence: &dyn IntelligenceTools,
        user_request: &str,
    ) -> ContextualInsight {
        // Get context for the user's request
        intelligence.get_development_context(user_request).await
    }
    
    /// Example: Before making code changes
    pub async fn before_code_changes(
        intelligence: &dyn IntelligenceTools,
        files_to_change: &[String],
    ) -> ImpactAnalysis {
        // Understand what might be affected
        intelligence.analyze_change_impact(files_to_change).await
    }
    
    /// Example: After completing a task
    pub async fn after_task_completion(
        intelligence: &dyn IntelligenceTools,
        files_used: &[String],
        success_rating: f64,
    ) {
        let outcome = Outcome {
            success_rating,
            completion_status: "completed".to_string(),
            user_feedback: None,
            identified_gaps: vec![],
        };
        
        intelligence.track_conversation_outcome(files_used, outcome).await;
    }
}