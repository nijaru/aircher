use crate::config::ConfigManager;
use crate::storage::DatabaseManager;
use anyhow::Result;
use chrono::{DateTime, Utc};
use async_trait::async_trait;

pub mod context;
pub mod narrative;
pub mod memory;
pub mod tools;
pub mod tui_tools;
pub mod file_monitor;

pub use context::*;
pub use narrative::*;
pub use memory::*;
pub use tools::*;

/// Intelligence Engine - Context-aware development assistant for AI agents
pub struct IntelligenceEngine {
    config: ConfigManager,
    storage: DatabaseManager,
    context_engine: ContextualRelevanceEngine,
    narrative_tracker: DevelopmentNarrativeTracker,
    memory_system: ConversationalMemorySystem,
}

impl IntelligenceEngine {
    pub async fn new(config: &ConfigManager, storage: &DatabaseManager) -> Result<Self> {
        let context_engine = ContextualRelevanceEngine::new(config).await?;
        let narrative_tracker = DevelopmentNarrativeTracker::new(config).await?;
        let memory_system = ConversationalMemorySystem::new(storage).await?;

        Ok(Self {
            config: config.clone(),
            storage: storage.clone(),
            context_engine,
            narrative_tracker,
            memory_system,
        })
    }
}

#[async_trait]
impl IntelligenceTools for IntelligenceEngine {
    async fn get_development_context(&self, query: &str) -> ContextualInsight {
        // Combine all intelligence sources to provide comprehensive context
        let narrative = self.narrative_tracker.get_current_narrative().await;
        let relevance = self.context_engine.analyze_relevance(query).await;
        let memory = self.memory_system.get_relevant_patterns(query).await;

        ContextualInsight {
            development_phase: narrative.current_epic,
            active_story: narrative.recent_focus,
            key_files: relevance.ranked_files,
            architectural_context: narrative.recent_decisions,
            recent_patterns: memory.patterns,
            suggested_next_actions: relevance.predicted_actions,
            confidence: relevance.confidence,
        }
    }

    async fn analyze_change_impact(&self, files: &[String]) -> ImpactAnalysis {
        self.context_engine.analyze_impact(files).await
    }

    async fn suggest_missing_context(&self, current_files: &[String]) -> ContextSuggestions {
        self.context_engine.suggest_additional_context(current_files).await
    }

    async fn track_conversation_outcome(&self, files: &[String], outcome: Outcome) -> () {
        self.memory_system.record_outcome(files, outcome).await;
    }

    async fn get_project_momentum(&self) -> ProjectMomentum {
        self.narrative_tracker.get_momentum().await
    }

    async fn add_project_directory(&self, _path: &str) -> Result<(), String> {
        // TODO: Implement cross-project directory support
        Ok(())
    }

    async fn analyze_cross_project_patterns(&self, _query: &str) -> CrossProjectInsight {
        // TODO: Implement cross-project pattern analysis
        CrossProjectInsight {
            similar_patterns: vec![],
            architectural_lessons: vec![],
            user_preferences: vec![],
            implementation_examples: vec![],
        }
    }

    async fn load_ai_configuration(&self) -> AiConfiguration {
        // TODO: Implement AI configuration loading from AGENT.md, .cursorrules, etc.
        AiConfiguration {
            global_instructions: None,
            project_instructions: None,
            cursor_rules: None,
            copilot_instructions: None,
            legacy_claude: None,
            custom_instructions: vec![],
        }
    }
}

/// Core data structures for Intelligence Engine
#[derive(Debug, Clone)]
pub struct ContextualRelevance {
    pub immediate: f64,
    pub sequential: f64,
    pub dependent: f64,
    pub reference: f64,
    pub historical: f64,
}

impl ContextualRelevance {
    /// Calculate total relevance score across all dimensions
    pub fn total_score(&self) -> f64 {
        self.immediate + self.sequential + self.dependent + self.reference + self.historical
    }
}

#[derive(Debug, Clone)]
pub struct ContextualInsight {
    pub development_phase: String,
    pub active_story: String,
    pub key_files: Vec<FileWithContext>,
    pub architectural_context: Vec<ArchitecturalDecision>,
    pub recent_patterns: Vec<Pattern>,
    pub suggested_next_actions: Vec<Action>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct FileWithContext {
    pub path: String,
    pub relevance: ContextualRelevance,
    pub purpose: String,
    pub last_significant_change: DateTime<Utc>,
    pub relationship_to_current_work: String,
}

#[derive(Debug, Clone)]
pub struct ArchitecturalDecision {
    pub decision: String,
    pub rationale: String,
    pub affected_files: Vec<String>,
    pub implications: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub pattern_type: String,
    pub description: String,
    pub confidence: f64,
    pub occurrences: u32,
}

#[derive(Debug, Clone)]
pub struct Action {
    pub action_type: String,
    pub description: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct ImpactAnalysis {
    pub direct_impacts: Vec<String>,
    pub indirect_impacts: Vec<String>,
    pub risk_areas: Vec<String>,
    pub suggested_tests: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ContextSuggestions {
    pub missing_dependencies: Vec<String>,
    pub architectural_context: Vec<String>,
    pub historical_context: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct Outcome {
    pub success_rating: f64,
    pub completion_status: String,
    pub user_feedback: Option<String>,
    pub identified_gaps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectMomentum {
    pub recent_focus: String,
    pub velocity_indicators: Vec<String>,
    pub architectural_direction: String,
    pub next_priorities: Vec<String>,
    pub knowledge_gaps: Vec<String>,
}

/// Cross-project intelligence data structures
#[derive(Debug, Clone)]
pub struct CrossProjectInsight {
    pub similar_patterns: Vec<Pattern>,
    pub architectural_lessons: Vec<ArchitecturalLesson>,
    pub user_preferences: Vec<UserPreference>,
    pub implementation_examples: Vec<ImplementationExample>,
}

#[derive(Debug, Clone)]
pub struct ArchitecturalLesson {
    pub pattern_name: String,
    pub description: String,
    pub projects_using: Vec<String>,
    pub success_rate: f64,
    pub best_practices: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UserPreference {
    pub preference_type: String,
    pub value: String,
    pub confidence: f64,
    pub projects_observed: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ImplementationExample {
    pub project_path: String,
    pub file_path: String,
    pub description: String,
    pub relevance_score: f64,
}

/// AI Configuration from various files
#[derive(Debug, Clone)]
pub struct AiConfiguration {
    pub global_instructions: Option<String>,  // ~/.agent/AGENT.md
    pub project_instructions: Option<String>, // ./AGENT.md
    pub cursor_rules: Option<String>,         // ./.cursorrules
    pub copilot_instructions: Option<String>, // ./.copilot
    pub legacy_claude: Option<String>,        // ./CLAUDE.md
    pub custom_instructions: Vec<CustomInstruction>,
}

#[derive(Debug, Clone)]
pub struct CustomInstruction {
    pub source_file: String,
    pub content: String,
    pub priority: u8,
}
