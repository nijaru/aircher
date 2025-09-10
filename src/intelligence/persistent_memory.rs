//! Persistent Project Memory System
//!
//! Provides long-term memory capabilities for the intelligence engine,
//! storing successful patterns, user preferences, and project-specific
//! context that persists across sessions.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{PathBuf};
use tokio::fs;
use super::{Outcome};

/// Project-specific persistent memory system (File-based implementation)
pub struct PersistentProjectMemory {
    project_root: PathBuf,
    memory_dir: PathBuf,
    session_cache: HashMap<String, ProjectSession>,
    patterns_cache: HashMap<String, Vec<LearnedPattern>>,
}

/// Session data that persists across restarts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSession {
    pub session_id: String,
    pub project_path: PathBuf,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub conversation_context: Vec<ConversationTurn>,
    pub successful_patterns: Vec<LearnedPattern>,
    pub user_preferences: UserPreferences,
    pub project_insights: ProjectInsights,
}

/// Individual conversation turn for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub timestamp: DateTime<Utc>,
    pub user_query: String,
    pub query_type: String,
    pub files_involved: Vec<String>,
    pub tools_used: Vec<String>,
    pub outcome_quality: f64,
    pub lessons_learned: Vec<String>,
}

/// Patterns learned from successful interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub pattern_id: String,
    pub pattern_type: PatternType,
    pub description: String,
    pub context: String,
    pub success_rate: f64,
    pub usage_count: u32,
    pub first_learned: DateTime<Utc>,
    pub last_reinforced: DateTime<Utc>,
    pub files_involved: Vec<String>,
    pub tools_involved: Vec<String>,
    pub conditions: Vec<String>,
}

/// Types of patterns the system can learn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    SuccessfulFileCombo,     // These files work well together
    EffectiveToolSequence,   // This sequence of tools solves problems
    UserPreference,          // User prefers this approach
    ArchitecturalPattern,    // This code pattern is used in this project
    DebuggingStrategy,       // This approach fixes these types of issues
    TestingPattern,          // This testing approach works for this codebase
    ConfigurationStrategy,   // This configuration works for this project type
}

/// User preferences learned over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub coding_style: HashMap<String, String>,    // "indent" -> "spaces", etc.
    pub preferred_tools: Vec<String>,             // Tools user likes to use
    pub communication_style: String,              // "concise", "detailed", etc.
    pub explanation_depth: String,                // "brief", "comprehensive", etc.
    pub preferred_languages: Vec<String>,         // Programming languages
    pub workflow_patterns: Vec<String>,           // Common workflow preferences
}

/// Project-specific insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInsights {
    pub project_type: String,                     // "rust-cli", "web-app", etc.
    pub key_directories: Vec<String>,             // Important directories
    pub architectural_patterns: Vec<String>,      // Patterns used in this project
    pub common_tasks: Vec<String>,                // Frequent user tasks
    pub performance_hotspots: Vec<String>,        // Performance-critical areas
    pub testing_strategy: String,                 // How this project is tested
    pub build_system: String,                     // "cargo", "npm", "make", etc.
    pub dependencies: Vec<String>,                // Key dependencies
}

impl PersistentProjectMemory {
    pub async fn new(project_root: PathBuf) -> Result<Self> {
        // Create memory directory structure
        let memory_dir = project_root.join(".aircher").join("memory");
        fs::create_dir_all(&memory_dir).await?;

        let mut memory = Self {
            project_root,
            memory_dir,
            session_cache: HashMap::new(),
            patterns_cache: HashMap::new(),
        };
        
        // Load existing sessions
        memory.load_existing_sessions().await?;
        
        // Load existing patterns
        memory.load_existing_patterns().await?;

        Ok(memory)
    }

    /// Load existing sessions from files
    async fn load_existing_sessions(&mut self) -> Result<()> {
        let sessions_dir = self.memory_dir.join("sessions");
        fs::create_dir_all(&sessions_dir).await?;
        
        let mut dir_entries = fs::read_dir(&sessions_dir).await?;
        while let Some(entry) = dir_entries.next_entry().await? {
            if entry.file_type().await?.is_file() && 
               entry.path().extension().map_or(false, |ext| ext == "json") {
                
                if let Ok(content) = fs::read_to_string(entry.path()).await {
                    if let Ok(session) = serde_json::from_str::<ProjectSession>(&content) {
                        self.session_cache.insert(session.session_id.clone(), session);
                    }
                }
            }
        }
        
        tracing::info!("Loaded {} existing sessions for project", self.session_cache.len());
        Ok(())
    }
    
    /// Load existing patterns from files
    async fn load_existing_patterns(&mut self) -> Result<()> {
        let patterns_dir = self.memory_dir.join("patterns");
        fs::create_dir_all(&patterns_dir).await?;
        
        let mut dir_entries = fs::read_dir(&patterns_dir).await?;
        while let Some(entry) = dir_entries.next_entry().await? {
            if entry.file_type().await?.is_file() && 
               entry.path().extension().map_or(false, |ext| ext == "json") {
                
                if let Ok(content) = fs::read_to_string(entry.path()).await {
                    if let Ok(patterns) = serde_json::from_str::<Vec<LearnedPattern>>(&content) {
                        let path = entry.path();
                        let file_stem = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("default");
                        self.patterns_cache.insert(file_stem.to_string(), patterns);
                    }
                }
            }
        }
        
        tracing::info!("Loaded {} pattern categories", self.patterns_cache.len());
        Ok(())
    }

    /// Start or resume a project session
    pub async fn start_session(&mut self, session_id: Option<String>) -> Result<String> {
        let session_id = session_id.unwrap_or_else(|| {
            format!("session_{}", Utc::now().timestamp())
        });

        let should_create_new = !self.session_cache.contains_key(&session_id);
        
        if !should_create_new {
            // Resume existing session
            if let Some(existing_session) = self.session_cache.get_mut(&session_id) {
                existing_session.last_activity = Utc::now();
            }
            // Clone the session to avoid borrow conflicts
            if let Some(existing_session) = self.session_cache.get(&session_id).cloned() {
                self.save_session(&existing_session).await?;
            }
            tracing::info!("Resumed session: {}", session_id);
        } else {
            // Create new session
            let new_session = ProjectSession {
                session_id: session_id.clone(),
                project_path: self.project_root.clone(),
                started_at: Utc::now(),
                last_activity: Utc::now(),
                conversation_context: Vec::new(),
                successful_patterns: Vec::new(),
                user_preferences: UserPreferences::default(),
                project_insights: self.analyze_project_structure().await?,
            };

            self.session_cache.insert(session_id.clone(), new_session.clone());
            self.save_session(&new_session).await?;
            tracing::info!("Created new session: {}", session_id);
        }

        Ok(session_id)
    }

    /// Record a conversation turn for learning
    pub async fn record_conversation_turn(
        &mut self,
        session_id: &str,
        user_query: &str,
        files_involved: &[String],
        tools_used: &[String],
        outcome: &Outcome,
    ) -> Result<()> {
        let turn = ConversationTurn {
            timestamp: Utc::now(),
            user_query: user_query.to_string(),
            query_type: self.classify_query(user_query),
            files_involved: files_involved.to_vec(),
            tools_used: tools_used.to_vec(),
            outcome_quality: outcome.success_rating,
            lessons_learned: outcome.identified_gaps.clone(),
        };

        // Extract pattern outside of mutable borrow if needed
        let pattern = if turn.outcome_quality > 0.8 {
            Some(self.extract_pattern_from_turn(&turn).await?)
        } else {
            None
        };
        
        // Update session in steps to avoid borrow conflicts
        let mut session_to_save = None;
        
        if let Some(session) = self.session_cache.get_mut(session_id) {
            session.conversation_context.push(turn.clone());
            session.last_activity = Utc::now();
            
            // Add pattern if we learned one
            if let Some(pattern) = pattern {
                session.successful_patterns.push(pattern);
            }
            
            // Update user preferences directly
            Self::update_user_preferences_inline(session, &turn).await?;
            
            // Clone session for saving
            session_to_save = Some(session.clone());
        }
        
        // Save session outside the mutable borrow
        if let Some(session) = session_to_save {
            self.save_session(&session).await?;
        }

        Ok(())
    }

    /// Get relevant patterns for a query
    pub async fn get_relevant_patterns(&self, query: &str, session_id: &str) -> Result<Vec<LearnedPattern>> {
        let mut relevant_patterns = Vec::new();

        // Get patterns from current session
        if let Some(session) = self.session_cache.get(session_id) {
            let query_type = self.classify_query(query);
            
            for pattern in &session.successful_patterns {
                if self.pattern_matches_query(pattern, query, &query_type) {
                    relevant_patterns.push(pattern.clone());
                }
            }
        }

        // Get patterns from patterns cache
        let query_type = self.classify_query(query);
        
        // Search across all pattern categories
        for patterns_list in self.patterns_cache.values() {
            for pattern in patterns_list {
                if self.pattern_matches_query(pattern, query, &query_type) {
                    relevant_patterns.push(pattern.clone());
                }
            }
        }

        // Sort by relevance
        relevant_patterns.sort_by(|a, b| {
            b.success_rate.partial_cmp(&a.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.usage_count.cmp(&a.usage_count))
        });

        relevant_patterns.truncate(5); // Top 5 most relevant
        Ok(relevant_patterns)
    }

    /// Save session to both cache and file
    async fn save_session(&self, session: &ProjectSession) -> Result<()> {
        // Save session as JSON file
        let sessions_dir = self.memory_dir.join("sessions");
        fs::create_dir_all(&sessions_dir).await?;
        
        let session_file = sessions_dir.join(format!("{}.json", session.session_id));
        let session_json = serde_json::to_string_pretty(session)?;
        fs::write(&session_file, session_json).await?;

        // Also export as human-readable markdown
        self.export_memory_markdown(session).await?;

        Ok(())
    }

    /// Export memory as human-readable markdown
    async fn export_memory_markdown(&self, session: &ProjectSession) -> Result<()> {
        let memory_file = self.memory_dir.join(format!("{}.md", session.session_id));
        
        let mut content = format!(
            "# Project Memory - Session {}\n\n",
            session.session_id
        );
        
        content.push_str(&format!(
            "**Project**: {}\n**Started**: {}\n**Last Activity**: {}\n\n",
            session.project_path.display(),
            session.started_at.format("%Y-%m-%d %H:%M:%S UTC"),
            session.last_activity.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Project insights
        content.push_str("## Project Insights\n\n");
        content.push_str(&format!("- **Type**: {}\n", session.project_insights.project_type));
        content.push_str(&format!("- **Build System**: {}\n", session.project_insights.build_system));
        if !session.project_insights.key_directories.is_empty() {
            content.push_str(&format!("- **Key Directories**: {}\n", 
                session.project_insights.key_directories.join(", ")));
        }
        content.push_str("\n");

        // Successful patterns
        if !session.successful_patterns.is_empty() {
            content.push_str("## Learned Patterns\n\n");
            for pattern in &session.successful_patterns {
                content.push_str(&format!(
                    "### {}\n**Success Rate**: {:.1}% | **Used**: {} times\n\n{}\n\n",
                    pattern.description,
                    pattern.success_rate * 100.0,
                    pattern.usage_count,
                    pattern.context
                ));
            }
        }

        // User preferences
        content.push_str("## User Preferences\n\n");
        if !session.user_preferences.preferred_tools.is_empty() {
            content.push_str(&format!("- **Preferred Tools**: {}\n", 
                session.user_preferences.preferred_tools.join(", ")));
        }
        content.push_str(&format!("- **Communication Style**: {}\n", 
            session.user_preferences.communication_style));
        content.push_str(&format!("- **Explanation Depth**: {}\n\n", 
            session.user_preferences.explanation_depth));

        // Recent conversations
        if !session.conversation_context.is_empty() {
            content.push_str("## Recent Conversations\n\n");
            let recent_turns: Vec<_> = session.conversation_context.iter()
                .rev()
                .take(5)
                .collect();
            
            for (i, turn) in recent_turns.iter().enumerate() {
                content.push_str(&format!(
                    "### {} - {} (Quality: {:.1}/10)\n**Query**: {}\n**Files**: {}\n**Tools**: {}\n\n",
                    i + 1,
                    turn.timestamp.format("%m/%d %H:%M"),
                    turn.outcome_quality * 10.0,
                    turn.user_query,
                    turn.files_involved.join(", "),
                    turn.tools_used.join(", ")
                ));
            }
        }

        fs::write(&memory_file, content).await?;
        Ok(())
    }

    /// Extract learned pattern from successful conversation turn
    async fn extract_pattern_from_turn(&self, turn: &ConversationTurn) -> Result<LearnedPattern> {
        let pattern = LearnedPattern {
            pattern_id: format!("pattern_{}", Utc::now().timestamp_millis()),
            pattern_type: self.infer_pattern_type(&turn.query_type, &turn.tools_used),
            description: format!("Successful {} using {}", 
                turn.query_type, turn.tools_used.join(", ")),
            context: turn.user_query.clone(),
            success_rate: turn.outcome_quality,
            usage_count: 1,
            first_learned: turn.timestamp,
            last_reinforced: turn.timestamp,
            files_involved: turn.files_involved.clone(),
            tools_involved: turn.tools_used.clone(),
            conditions: self.extract_conditions_from_turn(turn),
        };

        Ok(pattern)
    }

    /// Analyze project structure to generate insights
    async fn analyze_project_structure(&self) -> Result<ProjectInsights> {
        let mut insights = ProjectInsights {
            project_type: "unknown".to_string(),
            key_directories: Vec::new(),
            architectural_patterns: Vec::new(),
            common_tasks: Vec::new(),
            performance_hotspots: Vec::new(),
            testing_strategy: "unknown".to_string(),
            build_system: "unknown".to_string(),
            dependencies: Vec::new(),
        };

        // Detect project type and build system
        if self.project_root.join("Cargo.toml").exists() {
            insights.project_type = "rust".to_string();
            insights.build_system = "cargo".to_string();
        } else if self.project_root.join("package.json").exists() {
            insights.project_type = "javascript".to_string();
            insights.build_system = "npm".to_string();
        } else if self.project_root.join("pyproject.toml").exists() {
            insights.project_type = "python".to_string();
            insights.build_system = "python".to_string();
        }

        // Identify key directories
        let common_dirs = ["src", "lib", "tests", "docs", "examples", "bin"];
        for dir in &common_dirs {
            if self.project_root.join(dir).exists() {
                insights.key_directories.push(dir.to_string());
            }
        }

        Ok(insights)
    }

    // Helper methods
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

    fn pattern_matches_query(&self, pattern: &LearnedPattern, query: &str, query_type: &str) -> bool {
        let pattern_type_str = format!("{:?}", pattern.pattern_type).to_lowercase();
        query_type.contains(&pattern_type_str) || 
        pattern.description.to_lowercase().contains(&query.to_lowercase()) ||
        pattern.context.to_lowercase().contains(&query.to_lowercase())
    }

    fn infer_pattern_type(&self, query_type: &str, tools_used: &[String]) -> PatternType {
        match query_type {
            "debugging" => PatternType::DebuggingStrategy,
            "testing" => PatternType::TestingPattern,
            "configuration" => PatternType::ConfigurationStrategy,
            _ => {
                if tools_used.len() > 1 {
                    PatternType::EffectiveToolSequence
                } else {
                    PatternType::SuccessfulFileCombo
                }
            }
        }
    }

    fn extract_conditions_from_turn(&self, turn: &ConversationTurn) -> Vec<String> {
        let mut conditions = Vec::new();
        
        if !turn.files_involved.is_empty() {
            conditions.push(format!("files: {}", turn.files_involved.join(", ")));
        }
        
        if turn.outcome_quality > 0.9 {
            conditions.push("high_quality_outcome".to_string());
        }
        
        conditions
    }

    async fn update_user_preferences_inline(session: &mut ProjectSession, turn: &ConversationTurn) -> Result<()> {
        // Update communication style based on query patterns
        if turn.user_query.len() < 50 {
            session.user_preferences.communication_style = "concise".to_string();
        } else {
            session.user_preferences.communication_style = "detailed".to_string();
        }

        // Track preferred tools
        for tool in &turn.tools_used {
            if !session.user_preferences.preferred_tools.contains(tool) {
                session.user_preferences.preferred_tools.push(tool.clone());
            }
        }

        Ok(())
    }

}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            coding_style: HashMap::new(),
            preferred_tools: Vec::new(),
            communication_style: "balanced".to_string(),
            explanation_depth: "moderate".to_string(),
            preferred_languages: Vec::new(),
            workflow_patterns: Vec::new(),
        }
    }
}

impl Default for ProjectInsights {
    fn default() -> Self {
        Self {
            project_type: "unknown".to_string(),
            key_directories: Vec::new(),
            architectural_patterns: Vec::new(),
            common_tasks: Vec::new(),
            performance_hotspots: Vec::new(),
            testing_strategy: "unknown".to_string(),
            build_system: "unknown".to_string(),
            dependencies: Vec::new(),
        }
    }
}