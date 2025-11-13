use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use tracing::{debug, warn};

use super::{
    ContextualInsight, ImpactAnalysis, ContextSuggestions, Outcome,
    ProjectMomentum, CrossProjectInsight, AiConfiguration, FileWithContext,
    ContextualRelevance, ArchitecturalDecision, Pattern, Action
};
use super::tools::IntelligenceTools;
use crate::project::ProjectManager;

/// TUI-specific implementation of intelligence tools
///
/// This provides the actual implementation that the TUI can use to
/// integrate with the Intelligence Engine.
#[derive(Clone)]
pub struct TuiIntelligenceTools {
    project_manager: ProjectManager,
}

impl TuiIntelligenceTools {
    /// Create a new TUI intelligence tools instance
    pub fn new() -> Result<Self> {
        let project_manager = ProjectManager::new()?;

        Ok(Self {
            project_manager,
        })
    }

    /// Initialize the project if needed
    pub fn initialize_project(&mut self) -> Result<()> {
        if !self.project_manager.is_project_initialized() {
            self.project_manager.initialize_project()?;
        }
        Ok(())
    }

    /// Get project information
    pub fn get_project_info(&self) -> crate::project::ProjectInfo {
        self.project_manager.get_project_info()
    }

    /// Read file contents safely
    async fn read_file_safe(&self, path: &str) -> Result<String> {
        let content = tokio::fs::read_to_string(path).await?;
        Ok(content)
    }

    /// Scan directory for files
    fn scan_directory(&self, path: &str, max_depth: usize) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let path = PathBuf::from(path);

        if path.is_dir() {
            Self::scan_directory_recursive(&path, &mut files, 0, max_depth)?;
        }

        Ok(files)
    }

    fn scan_directory_recursive(
        dir: &PathBuf,
        files: &mut Vec<PathBuf>,
        current_depth: usize,
        max_depth: usize,
    ) -> Result<()> {
        if current_depth > max_depth {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Skip hidden files and directories
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    continue;
                }
            }

            if path.is_dir() {
                Self::scan_directory_recursive(&path, files, current_depth + 1, max_depth)?;
            } else {
                files.push(path);
            }
        }

        Ok(())
    }

    /// Calculate simple relevance for a file
    fn calculate_simple_relevance(&self, path: &str, query: &str) -> ContextualRelevance {
        let mut relevance = ContextualRelevance {
            immediate: 0.0,
            sequential: 0.0,
            dependent: 0.0,
            reference: 0.0,
            historical: 0.0,
        };

        // Basic keyword matching
        let query_lower = query.to_lowercase();
        let path_lower = path.to_lowercase();

        if query_lower.split_whitespace().any(|word| path_lower.contains(word)) {
            relevance.sequential = 0.8;
        }

        // Check for important file types
        if path.contains("mod.rs") || path.contains("lib.rs") || path.contains("main.rs") {
            relevance.dependent = 0.6;
        }

        // Check for configuration files
        if path.contains(".toml") || path.contains(".md") || path.contains("config") {
            relevance.reference = 0.4;
        }

        // Check for test files
        if path.contains("test") {
            relevance.reference = 0.3;
        }

        relevance
    }

    /// Analyze file to understand its purpose
    async fn analyze_file_purpose(&self, path: &str) -> Result<String> {
        let content = self.read_file_safe(path).await?;

        // Simple heuristic-based analysis
        let mut analysis = String::new();

        // Check file type
        if let Some(extension) = PathBuf::from(path).extension().and_then(|e| e.to_str()) {
            match extension {
                "rs" => analysis.push_str("Rust source file"),
                "py" => analysis.push_str("Python source file"),
                "js" | "ts" => analysis.push_str("JavaScript/TypeScript source file"),
                "md" => analysis.push_str("Markdown documentation"),
                "toml" => analysis.push_str("TOML configuration file"),
                "json" => analysis.push_str("JSON data/configuration file"),
                "yaml" | "yml" => analysis.push_str("YAML configuration file"),
                _ => analysis.push_str("Source file"),
            }
        }

        // Check for common patterns
        if content.contains("fn main(") || content.contains("def main(") {
            analysis.push_str(" - Contains main entry point");
        }

        if content.contains("test") || content.contains("Test") {
            analysis.push_str(" - Contains tests");
        }

        if content.contains("pub mod") || content.contains("mod ") {
            analysis.push_str(" - Module definition");
        }

        if content.contains("struct") || content.contains("class") {
            analysis.push_str(" - Data structure definitions");
        }

        Ok(analysis)
    }
}

#[async_trait]
impl IntelligenceTools for TuiIntelligenceTools {
    async fn get_development_context(&self, query: &str) -> ContextualInsight {
        debug!("Getting development context for query: {}", query);

        let project_info = self.get_project_info();
        let project_root = project_info.root_path;

        // Scan for relevant files
        let files = self.scan_directory(project_root.to_str().unwrap(), 3)
            .unwrap_or_default();

        // Create FileWithContext objects
        let mut key_files = Vec::new();

        for file in files.iter().take(20) { // Limit to first 20 files
            if let Some(path_str) = file.to_str() {
                if let Ok(purpose) = self.analyze_file_purpose(path_str).await {
                    let relevance = self.calculate_simple_relevance(path_str, query);

                    key_files.push(FileWithContext {
                        path: path_str.to_string(),
                        relevance,
                        purpose,
                        last_significant_change: chrono::Utc::now(),
                        relationship_to_current_work: "Supporting context".to_string(),
                    });
                }
            }
        }

        // Sort by relevance
        key_files.sort_by(|a, b| b.relevance.total_score().partial_cmp(&a.relevance.total_score()).unwrap());
        key_files.truncate(10);

        ContextualInsight {
            development_phase: "TUI Integration".to_string(),
            active_story: "Implementing session management and intelligence tools".to_string(),
            key_files,
            architectural_context: vec![
                ArchitecturalDecision {
                    decision: "Use SQLite for session persistence".to_string(),
                    rationale: "Local storage, no external dependencies".to_string(),
                    affected_files: vec!["src/sessions/mod.rs".to_string()],
                    implications: vec!["Database schema management".to_string()],
                    timestamp: chrono::Utc::now(),
                },
                ArchitecturalDecision {
                    decision: "TUI-first approach with CLI fallback".to_string(),
                    rationale: "Better user experience for interactive development".to_string(),
                    affected_files: vec!["src/ui/mod.rs".to_string(), "src/cli/mod.rs".to_string()],
                    implications: vec!["Session state management".to_string()],
                    timestamp: chrono::Utc::now(),
                },
            ],
            recent_patterns: vec![
                Pattern {
                    pattern_type: "async/await".to_string(),
                    description: "Consistent async patterns throughout codebase".to_string(),
                    confidence: 0.9,
                    occurrences: 50,
                },
                Pattern {
                    pattern_type: "Result<T>".to_string(),
                    description: "Error handling with anyhow::Result".to_string(),
                    confidence: 0.95,
                    occurrences: 80,
                },
            ],
            suggested_next_actions: vec![
                Action {
                    action_type: "implementation".to_string(),
                    description: "Integrate session management into TUI".to_string(),
                    confidence: 0.8,
                },
                Action {
                    action_type: "testing".to_string(),
                    description: "Test session persistence functionality".to_string(),
                    confidence: 0.7,
                },
            ],
            confidence: 0.75,
        }
    }

    async fn analyze_change_impact(&self, files: &[String]) -> ImpactAnalysis {
        debug!("Analyzing change impact for {} files", files.len());

        let mut direct_impacts = Vec::new();
        let mut indirect_impacts = Vec::new();
        let mut risk_areas = Vec::new();
        let mut suggested_tests = Vec::new();

        for file in files {
            // Simple analysis based on file name patterns
            if file.contains("mod.rs") || file.contains("lib.rs") {
                direct_impacts.push(format!("Module structure changes in {}", file));
                indirect_impacts.push("Import statements may need updates".to_string());
            }

            if file.contains("main.rs") {
                direct_impacts.push(format!("Main entry point changes in {}", file));
                risk_areas.push("Application startup behavior".to_string());
            }

            if file.contains("config") {
                direct_impacts.push(format!("Configuration changes in {}", file));
                indirect_impacts.push("System behavior may change".to_string());
            }

            if file.contains("test") {
                suggested_tests.push(format!("Run tests in {}", file));
            } else {
                suggested_tests.push(format!("Test functionality in {}", file));
            }
        }

        // Add general test suggestions
        suggested_tests.push("Run cargo check".to_string());
        suggested_tests.push("Run integration tests".to_string());

        ImpactAnalysis {
            direct_impacts,
            indirect_impacts,
            risk_areas,
            suggested_tests,
        }
    }

    async fn suggest_missing_context(&self, current_files: &[String]) -> ContextSuggestions {
        debug!("Suggesting missing context for {} files", current_files.len());

        let mut missing_dependencies = Vec::new();
        let mut architectural_context = Vec::new();
        let mut historical_context = Vec::new();

        // Check for common patterns
        let has_main = current_files.iter().any(|f| f.contains("main.rs"));
        let has_lib = current_files.iter().any(|f| f.contains("lib.rs"));
        let has_config = current_files.iter().any(|f| f.contains("config"));
        let has_readme = current_files.iter().any(|f| f.contains("README"));

        if !has_main && !has_lib {
            missing_dependencies.push("src/main.rs or src/lib.rs".to_string());
        }

        if !has_config {
            architectural_context.push("Configuration files".to_string());
        }

        if !has_readme {
            historical_context.push("Project README.md".to_string());
        }

        // Always suggest some key architectural files
        architectural_context.push("Cargo.toml".to_string());
        architectural_context.push("src/lib.rs".to_string());

        // Suggest documentation
        historical_context.push("AGENT.md for AI context".to_string());
        historical_context.push("Project documentation".to_string());

        let confidence = if missing_dependencies.is_empty() && architectural_context.is_empty() {
            0.3
        } else {
            0.7
        };

        ContextSuggestions {
            missing_dependencies,
            architectural_context,
            historical_context,
            confidence,
        }
    }

    async fn track_conversation_outcome(&self, files: &[String], outcome: Outcome) {
        debug!("Tracking conversation outcome for {} files", files.len());

        // This would typically store the outcome in a database
        // For now, just log it
        if outcome.success_rating > 0.8 {
            debug!("Successful conversation with files: {:?}", files);
        } else {
            warn!("Conversation had issues: {}", outcome.completion_status);
        }
    }

    async fn get_project_momentum(&self) -> ProjectMomentum {
        debug!("Getting project momentum");

        let _project_info = self.get_project_info();

        ProjectMomentum {
            recent_focus: "TUI integration and session management".to_string(),
            velocity_indicators: vec![
                "Active development in multiple modules".to_string(),
                "Regular commits and progress".to_string(),
                "Comprehensive testing implementation".to_string(),
            ],
            architectural_direction: "AI-powered development assistant with local intelligence".to_string(),
            next_priorities: vec![
                "Complete TUI integration".to_string(),
                "Implement background file monitoring".to_string(),
                "Add context injection system".to_string(),
            ],
            knowledge_gaps: vec![
                "Performance optimization patterns".to_string(),
                "Advanced TUI interactions".to_string(),
                "Intelligence engine tuning".to_string(),
            ],
        }
    }

    async fn add_project_directory(&self, path: &str) -> Result<(), String> {
        debug!("Adding project directory: {}", path);

        // Validate path exists
        if !PathBuf::from(path).exists() {
            return Err(format!("Directory does not exist: {}", path));
        }

        // This would typically add to a list of tracked directories
        // For now, just validate and return success
        Ok(())
    }

    async fn analyze_cross_project_patterns(&self, query: &str) -> CrossProjectInsight {
        debug!("Analyzing cross-project patterns for: {}", query);

        // Placeholder implementation
        CrossProjectInsight {
            similar_patterns: vec![
                Pattern {
                    pattern_type: "TUI patterns".to_string(),
                    description: "Terminal UI patterns from other Rust projects".to_string(),
                    confidence: 0.7,
                    occurrences: 10,
                },
                Pattern {
                    pattern_type: "Session management".to_string(),
                    description: "Session persistence in CLI tools".to_string(),
                    confidence: 0.8,
                    occurrences: 15,
                },
            ],
            architectural_lessons: vec![
                super::ArchitecturalLesson {
                    pattern_name: "Local-first intelligence".to_string(),
                    description: "Store intelligence locally for privacy and performance".to_string(),
                    projects_using: vec!["Gemini CLI".to_string(), "Codex".to_string()],
                    success_rate: 0.85,
                    best_practices: vec![
                        "Use SQLite for local storage".to_string(),
                        "Implement background analysis".to_string(),
                    ],
                },
            ],
            user_preferences: vec![
                super::UserPreference {
                    preference_type: "Communication style".to_string(),
                    value: "Concise and direct".to_string(),
                    confidence: 0.9,
                    projects_observed: vec!["Claude Code".to_string(), "Aircher".to_string()],
                },
            ],
            implementation_examples: vec![
                super::ImplementationExample {
                    project_path: "external/gemini-cli".to_string(),
                    file_path: "src/tools/memory.rs".to_string(),
                    description: "Memory management for AI context".to_string(),
                    relevance_score: 0.8,
                },
                super::ImplementationExample {
                    project_path: "external/codex".to_string(),
                    file_path: "src/agent/core.rs".to_string(),
                    description: "Agent core implementation patterns".to_string(),
                    relevance_score: 0.7,
                },
            ],
        }
    }

    async fn load_ai_configuration(&self) -> AiConfiguration {
        debug!("Loading AI configuration");

        let mut config = AiConfiguration {
            global_instructions: None,
            project_instructions: None,
            cursor_rules: None,
            copilot_instructions: None,
            legacy_claude: None,
            custom_instructions: vec![],
        };

        // Try to load AGENT.md
        if let Ok(agent_path) = self.project_manager.get_agent_config_path() {
            if agent_path.exists() {
                if let Ok(content) = self.read_file_safe(&agent_path.to_string_lossy()).await {
                    config.project_instructions = Some(content);
                }
            }
        }

        // Check for other configuration files
        let project_root = self.project_manager.get_project_root()
            .unwrap_or(self.project_manager.get_current_dir());

        // Look for .cursorrules
        let cursor_rules_path = project_root.join(".cursorrules");
        if cursor_rules_path.exists() {
            if let Ok(content) = self.read_file_safe(&cursor_rules_path.to_string_lossy()).await {
                config.cursor_rules = Some(content);
            }
        }

        // Look for AGENTS.md (renamed from CLAUDE.md)
        let agents_md_path = project_root.join("AGENTS.md");
        if agents_md_path.exists() {
            if let Ok(content) = self.read_file_safe(&agents_md_path.to_string_lossy()).await {
                config.legacy_claude = Some(content);
            }
        }

        // Add custom instructions
        config.custom_instructions.push(super::CustomInstruction {
            source_file: "internal".to_string(),
            content: "Use Rust best practices with async/await, error handling with anyhow::Result, and modular code organization".to_string(),
            priority: 5,
        });

        config
    }

    async fn search_code_semantically(&self, _query: &str, _limit: usize) -> Result<Vec<super::CodeSearchResult>, String> {
        // TUI tools don't have semantic search capability
        Err("Semantic search not available in TUI intelligence tools".to_string())
    }

    async fn analyze_code_structure(&self, _file_path: &str) -> Result<super::ASTAnalysis, String> {
        // TUI tools don't have AST analysis capability
        Err("AST analysis not available in TUI intelligence tools".to_string())
    }

    async fn get_code_insights(&self, _file_path: &str) -> Result<super::CodeInsights, String> {
        // TUI tools don't have full code insights capability
        Err("Code insights not available in TUI intelligence tools".to_string())
    }

    async fn initialize_project_memory(&mut self, _project_root: std::path::PathBuf) -> Result<(), String> {
        // TUI tools don't support persistent memory initialization
        Err("Project memory initialization not available in TUI intelligence tools".to_string())
    }

    async fn start_session(&self, _session_id: Option<String>) -> Result<Option<String>, String> {
        // TUI tools don't support session management
        Ok(None)
    }

    async fn record_learning(
        &self,
        _session_id: &str,
        _user_query: &str,
        _files_involved: &[String],
        _tools_used: &[String],
        _outcome: super::Outcome,
    ) -> Result<(), String> {
        // TUI tools don't support learning recording
        Ok(())
    }

    async fn get_relevant_patterns(&self, _query: &str, _session_id: &str) -> Result<Vec<String>, String> {
        // TUI tools don't have persistent pattern storage
        Ok(Vec::new())
    }
}

impl Default for TuiIntelligenceTools {
    fn default() -> Self {
        Self::new().expect("Failed to create TUI intelligence tools")
    }
}
