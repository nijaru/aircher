use crate::config::ConfigManager;
use crate::storage::DatabaseManager;
use crate::semantic_search::SemanticCodeSearch;
use anyhow::Result;
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tracing::{warn, info};

pub mod context;
pub mod narrative;
pub mod memory;
pub mod persistent_memory;
// pub mod simple_lance_memory;  // Disabled due to arrow dependency conflict
// pub mod intelligent_memory;   // Disabled due to arrow dependency conflict
pub mod duckdb_memory;
pub mod knowledge_graph;  // Week 4: Knowledge graph for codebase structure
pub mod graph_builder;    // Week 4: Build knowledge graph from tree-sitter
pub mod working_memory;   // Week 5: Dynamic context management
pub mod dynamic_context;  // Week 5: Integration of all memory systems
pub mod tools;
pub mod tui_tools;
pub mod file_monitor;
pub mod mcp_integration;
pub mod mcp_examples;
pub mod ast_analysis;
pub mod purpose_analysis;
pub mod pattern_aware_generation;
pub mod intelligent_debugging;
pub mod unified_intelligence;

pub use context::*;
pub use narrative::*;
pub use memory::*;
pub use persistent_memory::*;
// pub use simple_lance_memory::*;  // Disabled due to arrow dependency conflict
// pub use intelligent_memory::*;   // Disabled due to arrow dependency conflict
pub use duckdb_memory::*;
pub use knowledge_graph::*;
pub use graph_builder::*;
pub use working_memory::*;
pub use dynamic_context::*;
pub use tools::*;
pub use mcp_integration::*;
pub use ast_analysis::*;
// Use wildcard imports but rename conflicting types to avoid ambiguity
pub use purpose_analysis::*;
pub use pattern_aware_generation::*;
pub use intelligent_debugging::{
    IntelligentDebuggingEngine,
    // Rename conflicting types from intelligent_debugging to avoid conflicts with purpose_analysis
    DependencyType as DebuggingDependencyType,
    CouplingStrength as DebuggingCouplingStrength,
    EstimatedEffort as DebuggingEstimatedEffort,
};
pub use unified_intelligence::{
    UnifiedIntelligenceEngine,
    EnhancedContext,
    UserIntent,
    AnalysisDepth,
    CodeType,
    UrgencyLevel,
    ExplorationScope,
    ContextItem,
    ContextSource,
    IntelligenceInsight,
    TaskAnalysis,
    TaskComplexity,
};

/// Intelligence Engine - Context-aware development assistant for AI agents
pub struct IntelligenceEngine {
    _config: ConfigManager,
    _storage: DatabaseManager,
    context_engine: ContextualRelevanceEngine,
    narrative_tracker: DevelopmentNarrativeTracker,
    memory_system: ConversationalMemorySystem,
    duckdb_memory: Option<Arc<Mutex<DuckDBMemory>>>,
    knowledge_graph: Option<Arc<Mutex<KnowledgeGraph>>>,  // Week 4: Knowledge graph
    semantic_search: Option<Arc<RwLock<SemanticCodeSearch>>>,
    ast_analyzer: Arc<RwLock<ASTAnalyzer>>,
    purpose_analyzer: PurposeAnalysisEngine,
    pattern_generator: PatternAwareGenerationEngine,
    debugging_engine: IntelligentDebuggingEngine,
}

impl IntelligenceEngine {
    pub async fn new(config: &ConfigManager, storage: &DatabaseManager) -> Result<Self> {
        let context_engine = ContextualRelevanceEngine::new(config).await?;
        let narrative_tracker = DevelopmentNarrativeTracker::new(config).await?;
        let memory_system = ConversationalMemorySystem::new(storage).await?;
        let ast_analyzer = Arc::new(RwLock::new(ASTAnalyzer::new()?));
        let purpose_analyzer = PurposeAnalysisEngine::new(ast_analyzer.clone(), None);
        let pattern_generator = PatternAwareGenerationEngine::new(
            Arc::new(purpose_analyzer.clone()),
            ast_analyzer.clone(),
            None,
        );
        let debugging_engine = IntelligentDebuggingEngine::new(
            ast_analyzer.clone(),
            None,
            Arc::new(purpose_analyzer.clone()),
        );

        Ok(Self {
            _config: config.clone(),
            _storage: storage.clone(),
            context_engine,
            narrative_tracker,
            memory_system,
            duckdb_memory: None,
            knowledge_graph: None,
            semantic_search: None,
            ast_analyzer,
            purpose_analyzer,
            pattern_generator,
            debugging_engine,
        })
    }
    
    /// Create intelligence engine with semantic search integration
    pub async fn with_semantic_search(
        config: &ConfigManager,
        storage: &DatabaseManager,
        semantic_search: SemanticCodeSearch,
    ) -> Result<Self> {
        let context_engine = ContextualRelevanceEngine::new(config).await?;
        let narrative_tracker = DevelopmentNarrativeTracker::new(config).await?;
        let memory_system = ConversationalMemorySystem::new(storage).await?;
        let ast_analyzer = Arc::new(RwLock::new(ASTAnalyzer::new()?));
        let semantic_search_arc = Arc::new(RwLock::new(semantic_search));
        let purpose_analyzer = PurposeAnalysisEngine::new(ast_analyzer.clone(), Some(semantic_search_arc.clone()));
        let pattern_generator = PatternAwareGenerationEngine::new(
            Arc::new(purpose_analyzer.clone()),
            ast_analyzer.clone(),
            Some(semantic_search_arc.clone()),
        );
        let debugging_engine = IntelligentDebuggingEngine::new(
            ast_analyzer.clone(),
            Some(semantic_search_arc.clone()),
            Arc::new(purpose_analyzer.clone()),
        );

        Ok(Self {
            _config: config.clone(),
            _storage: storage.clone(),
            context_engine,
            narrative_tracker,
            memory_system,
            duckdb_memory: None,
            knowledge_graph: None,
            semantic_search: Some(semantic_search_arc),
            ast_analyzer,
            purpose_analyzer,
            pattern_generator,
            debugging_engine,
        })
    }
    
    /// Add semantic search to existing intelligence engine
    pub fn set_semantic_search(&mut self, semantic_search: SemanticCodeSearch) {
        let semantic_search_arc = Arc::new(RwLock::new(semantic_search));
        self.semantic_search = Some(semantic_search_arc.clone());
        // Update purpose analyzer with semantic search
        self.purpose_analyzer = PurposeAnalysisEngine::new(self.ast_analyzer.clone(), Some(semantic_search_arc.clone()));
        // Update pattern generator with semantic search
        self.pattern_generator = PatternAwareGenerationEngine::new(
            Arc::new(self.purpose_analyzer.clone()),
            self.ast_analyzer.clone(),
            Some(semantic_search_arc),
        );
    }
    
    /// Create an MCP-enhanced version of this intelligence engine
    pub async fn with_mcp_enhancement(self) -> Result<McpEnhancedIntelligenceEngine<Self>> {
        McpEnhancedIntelligenceEngine::new(self).await
    }
    
    /// Initialize DuckDB memory for a project
    pub async fn initialize_duckdb_memory(&mut self, project_root: std::path::PathBuf) -> Result<()> {
        let duckdb_memory = DuckDBMemory::new(&project_root).await?;
        
        self.duckdb_memory = Some(Arc::new(Mutex::new(duckdb_memory)));
        Ok(())
    }
    
    /// Start a new session with intelligent memory
    pub async fn start_memory_session(&self, session_id: Option<String>) -> Result<Option<String>> {
        // For now, return the provided session_id or generate a new one
        Ok(session_id.or_else(|| Some(uuid::Uuid::new_v4().to_string())))
    }
    
    /// Record a pattern for learning
    pub async fn record_pattern(
        &self,
        pattern: duckdb_memory::Pattern,
    ) -> Result<()> {
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;
            memory_guard.record_pattern(pattern).await?;
        }
        Ok(())
    }

    // WEEK 3: Episodic memory recording methods

    /// Record a tool execution to episodic memory
    pub async fn record_tool_execution(&self, execution: duckdb_memory::ToolExecution) -> Result<()> {
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;
            memory_guard.record_tool_execution(execution).await?;
        }
        Ok(())
    }

    /// Record a file interaction to episodic memory
    pub async fn record_file_interaction(&self, interaction: duckdb_memory::FileInteraction) -> Result<()> {
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;
            memory_guard.record_file_interaction(interaction).await?;
        }
        Ok(())
    }

    /// Record a task to episodic memory
    pub async fn record_task(&self, task: duckdb_memory::TaskRecord) -> Result<()> {
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;
            memory_guard.record_task(task).await?;
        }
        Ok(())
    }

    /// Update task status in episodic memory
    pub async fn update_task_status(&self, task_id: &str, status: &str, outcome: Option<String>) -> Result<()> {
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;
            memory_guard.update_task_status(task_id, status, outcome).await?;
        }
        Ok(())
    }

    /// Record a context snapshot to episodic memory
    pub async fn record_context_snapshot(&self, snapshot: duckdb_memory::ContextSnapshot) -> Result<()> {
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;
            memory_guard.record_context_snapshot(snapshot).await?;
        }
        Ok(())
    }

    // WEEK 3 DAY 5-7: Episodic memory QUERY methods

    /// Get tool execution history for current session
    pub async fn get_tool_executions(&self, session_id: &str, limit: usize) -> Result<Vec<duckdb_memory::ToolExecution>> {
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;
            memory_guard.get_tool_executions(session_id, limit).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get file interaction history - "Have I worked on this file before?"
    pub async fn get_file_history(&self, file_path: &str, limit: usize) -> Result<Vec<duckdb_memory::FileInteraction>> {
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;
            memory_guard.get_file_interactions(file_path, limit).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Find files that are often edited together (co-edit patterns)
    pub async fn find_co_edit_patterns(&self, time_window_minutes: i32) -> Result<Vec<duckdb_memory::LearnedPattern>> {
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;
            memory_guard.find_co_edit_patterns(time_window_minutes).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get learned patterns by type
    pub async fn get_learned_patterns(&self, pattern_type: &str, limit: usize) -> Result<Vec<duckdb_memory::LearnedPattern>> {
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;
            memory_guard.get_learned_patterns(pattern_type, limit).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get tool success rate analytics
    pub async fn get_tool_success_rate(&self, session_id: &str, tool_name: Option<&str>) -> Result<(usize, usize, f64)> {
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;
            let executions = memory_guard.get_tool_executions(session_id, 1000).await?;

            let filtered: Vec<_> = if let Some(name) = tool_name {
                executions.into_iter().filter(|e| e.tool_name == name).collect()
            } else {
                executions
            };

            let total = filtered.len();
            let successful = filtered.iter().filter(|e| e.success).count();
            let rate = if total > 0 {
                successful as f64 / total as f64
            } else {
                0.0
            };

            Ok((total, successful, rate))
        } else {
            Ok((0, 0, 0.0))
        }
    }

    /// Check if file has been worked on before and get context
    pub async fn check_file_context(&self, file_path: &str) -> Result<String> {
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;
            let interactions = memory_guard.get_file_interactions(file_path, 5).await?;

            if interactions.is_empty() {
                return Ok(format!("First time working with {}", file_path));
            }

            let mut context = format!("Previous work on {}:\n", file_path);
            for interaction in interactions.iter().take(3) {
                context.push_str(&format!(
                    "- {} at {} ({})\n",
                    interaction.operation,
                    interaction.timestamp.format("%Y-%m-%d %H:%M"),
                    if interaction.success { "success" } else { "failed" }
                ));
                if let Some(changes) = &interaction.changes_summary {
                    context.push_str(&format!("  {}\n", changes));
                }
            }

            Ok(context)
        } else {
            Ok("Memory not initialized".to_string())
        }
    }

    /// Get suggestions based on what files are often edited together
    pub async fn suggest_related_files(&self, current_file: &str) -> Result<Vec<String>> {
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;

            // Get co-edit patterns
            let patterns = memory_guard.find_co_edit_patterns(300).await?; // 5 minute window

            // Find patterns involving current file
            let mut related_files = Vec::new();
            for pattern in patterns {
                if let Some(files) = pattern.pattern_data.get("files").and_then(|f| f.as_array()) {
                    let file_strings: Vec<String> = files
                        .iter()
                        .filter_map(|f| f.as_str().map(String::from))
                        .collect();

                    if file_strings.iter().any(|f| f == current_file) {
                        for file in file_strings {
                            if file != current_file && !related_files.contains(&file) {
                                related_files.push(file);
                            }
                        }
                    }
                }
            }

            Ok(related_files)
        } else {
            Ok(Vec::new())
        }
    }

    // ========== Week 4: Knowledge Graph Methods ==========

    /// Build knowledge graph from project root
    pub async fn build_knowledge_graph(&mut self, project_root: PathBuf) -> Result<()> {
        info!("Building knowledge graph for {:?}", project_root);

        let mut builder = GraphBuilder::new(project_root.clone())?;
        let graph = builder.build_graph()?;

        let stats = graph.stats();
        info!("Knowledge graph built: {}", stats);

        self.knowledge_graph = Some(Arc::new(Mutex::new(graph)));

        Ok(())
    }

    /// Load knowledge graph from file
    pub async fn load_knowledge_graph(&mut self, path: &Path) -> Result<()> {
        info!("Loading knowledge graph from {:?}", path);

        let graph = KnowledgeGraph::load(path)?;

        let stats = graph.stats();
        info!("Knowledge graph loaded: {}", stats);

        self.knowledge_graph = Some(Arc::new(Mutex::new(graph)));

        Ok(())
    }

    /// Save knowledge graph to file
    pub async fn save_knowledge_graph(&self, path: &Path) -> Result<()> {
        if let Some(graph) = &self.knowledge_graph {
            let graph_guard = graph.lock().await;
            graph_guard.save(path)?;
            Ok(())
        } else {
            anyhow::bail!("Knowledge graph not initialized")
        }
    }

    /// Get all functions and classes in a file
    pub async fn get_file_structure(&self, file_path: &Path) -> Result<Vec<NodeType>> {
        if let Some(graph) = &self.knowledge_graph {
            let graph_guard = graph.lock().await;
            graph_guard.get_file_contents(file_path)
        } else {
            Ok(Vec::new())
        }
    }

    /// Find what calls a function
    pub async fn find_callers(&self, function_name: &str) -> Result<Vec<NodeType>> {
        if let Some(graph) = &self.knowledge_graph {
            let graph_guard = graph.lock().await;
            graph_guard.get_callers(function_name)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get file dependencies (imports)
    pub async fn get_file_dependencies(&self, file_path: &Path) -> Result<Vec<NodeType>> {
        if let Some(graph) = &self.knowledge_graph {
            let graph_guard = graph.lock().await;
            graph_guard.get_dependencies(file_path)
        } else {
            Ok(Vec::new())
        }
    }

    /// Find symbol definition
    pub async fn find_symbol_definition(&self, symbol_name: &str) -> Result<Vec<NodeType>> {
        if let Some(graph) = &self.knowledge_graph {
            let graph_guard = graph.lock().await;
            graph_guard.find_symbol(symbol_name)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get knowledge graph statistics
    pub async fn get_knowledge_graph_stats(&self) -> Option<GraphStats> {
        if let Some(graph) = &self.knowledge_graph {
            let graph_guard = graph.lock().await;
            Some(graph_guard.stats())
        } else {
            None
        }
    }

    /// Get embeddings for text using semantic search
    pub async fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        if let Some(semantic) = &self.semantic_search {
            let mut search = semantic.write().await;
            match search.generate_embedding(text).await {
                Ok(embedding) => Ok(embedding),
                Err(e) => {
                    warn!("Failed to generate embedding: {}, falling back to empty vector", e);
                    Ok(vec![])
                }
            }
        } else {
            Ok(vec![])
        }
    }
    
    /// Get intelligent suggestions based on context
    pub async fn get_suggestions(&self, context: &str, _embedding: Option<&[f32]>) -> Result<String> {
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;
            memory_guard.suggest_next(context).await
        } else {
            Ok("Intelligence memory not initialized".to_string())
        }
    }
    
    /// Predict what files might need changes
    pub async fn predict_file_changes(&self, current_file: &str) -> Result<Vec<String>> {
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;
            memory_guard.check_related_files(current_file).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Analyze code purpose and business context (internal use only - automatic via UnifiedIntelligenceEngine)
    pub(crate) async fn analyze_code_purpose(
        &self,
        file_path: &str,
        code_content: &str,
    ) -> Result<CodePurposeAnalysis> {
        self.purpose_analyzer.analyze_code_purpose(file_path, code_content).await
    }

    /// Get business context summary for quick understanding (internal use only - automatic via UnifiedIntelligenceEngine)
    pub(crate) async fn get_business_context_summary(
        &self,
        file_path: &str,
        code_content: &str,
    ) -> Result<String> {
        self.purpose_analyzer.get_business_context_summary(file_path, code_content).await
    }

    /// Learn patterns from project files (internal use only - automatic via UnifiedIntelligenceEngine)
    pub(crate) async fn learn_project_patterns(&self, project_files: Vec<String>) -> Result<()> {
        self.pattern_generator.learn_project_patterns(project_files).await
    }

    /// Generate code that follows project patterns (internal use only - automatic via UnifiedIntelligenceEngine)
    pub(crate) async fn generate_contextual_code(&self, request: CodeGenerationRequest) -> Result<GeneratedCode> {
        self.pattern_generator.generate_contextual_code(request).await
    }

    /// Get a summary of learned patterns (internal use only - automatic via UnifiedIntelligenceEngine)
    pub(crate) async fn get_pattern_summary(&self) -> Result<String> {
        self.pattern_generator.get_pattern_summary().await
    }

    // === INTELLIGENT DEBUGGING METHODS (internal use only - automatic via UnifiedIntelligenceEngine) ===

    /// Perform comprehensive error analysis (internal use only - automatic via UnifiedIntelligenceEngine)
    pub async fn analyze_error(&self, request: ErrorAnalysisRequest) -> Result<ErrorAnalysis> {
        self.debugging_engine.analyze_error(request).await
    }

    /// Generate multiple fix strategies for an error (internal use only - automatic via UnifiedIntelligenceEngine)
    pub async fn generate_fix_strategies(&self, analysis: &ErrorAnalysis) -> Result<Vec<FixStrategy>> {
        self.debugging_engine.generate_fix_strategies(analysis).await
    }

    /// Generate comprehensive fix with code changes, tests, and validation plan (internal use only - automatic via UnifiedIntelligenceEngine)
    pub async fn generate_comprehensive_fix(&self, request: ErrorAnalysisRequest) -> Result<FixResult> {
        self.debugging_engine.generate_comprehensive_fix(request).await
    }

    /// Quick error analysis for simple cases (internal use only - automatic via UnifiedIntelligenceEngine)
    pub async fn quick_analyze_error(&self, error_message: &str, file_path: &str) -> Result<ErrorAnalysis> {
        self.debugging_engine.quick_analyze(error_message, file_path).await
    }

    /// Get quick fix recommendations based on error pattern matching (internal use only - automatic via UnifiedIntelligenceEngine)
    pub async fn get_quick_fix_recommendations(&self, error_message: &str) -> Result<Vec<String>> {
        self.debugging_engine.get_quick_fix_recommendations(error_message).await
    }

    /// Analyze project-wide error patterns (internal use only - automatic via UnifiedIntelligenceEngine)
    pub async fn analyze_project_error_patterns(&self, project_files: Vec<String>) -> Result<std::collections::HashMap<String, Vec<String>>> {
        self.debugging_engine.analyze_project_error_patterns(project_files).await
    }

    /// Learn from fix results to improve future debugging (internal use only - automatic via UnifiedIntelligenceEngine)
    pub async fn learn_from_fix(&self, fix_result: &FixResult, success: bool, feedback: Option<String>) -> Result<()> {
        self.debugging_engine.learn_from_fix(fix_result, success, feedback).await
    }

    /// Analyze task for intelligence-enhanced strategy selection
    pub async fn analyze_task(&self, task: &str, context: &[ContextItem]) -> Result<TaskAnalysis> {
        // Classify user intent
        let intent = self.classify_user_intent(task).await;

        // Calculate complexity based on task description and context
        let complexity_score = self.calculate_task_complexity(task, context).await;

        // Calculate confidence based on available context and patterns
        let confidence_score = self.calculate_confidence(task, context).await;

        // Analyze task characteristics
        let has_multiple_solution_paths = task.contains("optimize") || task.contains("improve") ||
                                         task.contains("refactor") || complexity_score > 0.7;

        let is_critical = task.contains("fix") && (task.contains("critical") || task.contains("production") ||
                         task.contains("urgent")) || complexity_score > 0.9;

        let requires_codebase_search = task.contains("find") || task.contains("search") ||
                                      task.contains("where") || task.contains("how") ||
                                      complexity_score > 0.5;

        Ok(TaskAnalysis {
            intent,
            complexity_score,
            confidence_score,
            has_multiple_solution_paths,
            is_critical,
            requires_codebase_search,
        })
    }

    /// Classify user intent from task description
    async fn classify_user_intent(&self, task: &str) -> UserIntent {
        let task_lower = task.to_lowercase();

        if task_lower.contains("fix") || task_lower.contains("bug") || task_lower.contains("error") {
            let urgency = if task_lower.contains("critical") || task_lower.contains("urgent") {
                UrgencyLevel::Critical
            } else if task_lower.contains("important") {
                UrgencyLevel::High
            } else {
                UrgencyLevel::Medium
            };

            UserIntent::ProjectFixing {
                error_indicators: vec![task.to_string()],
                urgency_level: urgency,
            }
        } else if task_lower.contains("generate") || task_lower.contains("create") || task_lower.contains("write") {
            let code_type = if task_lower.contains("test") {
                CodeType::Test
            } else if task_lower.contains("doc") {
                CodeType::Documentation
            } else if task_lower.contains("config") {
                CodeType::Configuration
            } else if task_lower.contains("fix") {
                CodeType::BugFix
            } else if task_lower.contains("refactor") {
                CodeType::Refactoring
            } else {
                CodeType::NewFeature
            };

            UserIntent::CodeWriting {
                target_files: vec![],
                code_type,
            }
        } else if task_lower.contains("explain") || task_lower.contains("understand") || task_lower.contains("read") {
            let depth = if task_lower.contains("deep") || task_lower.contains("detail") {
                AnalysisDepth::Detailed
            } else if task_lower.contains("architecture") || task_lower.contains("design") {
                AnalysisDepth::Architectural
            } else {
                AnalysisDepth::Surface
            };

            UserIntent::CodeReading {
                files_mentioned: vec![],
                analysis_depth: depth,
            }
        } else if task_lower.contains("explore") || task_lower.contains("search") {
            let scope = if task_lower.contains("file") {
                ExplorationScope::SingleFile
            } else if task_lower.contains("module") {
                ExplorationScope::Module
            } else if task_lower.contains("architecture") {
                ExplorationScope::Architecture
            } else {
                ExplorationScope::FullProject
            };

            UserIntent::ProjectExploration {
                scope,
            }
        } else {
            // Default to code reading for unknown tasks
            UserIntent::CodeReading {
                files_mentioned: vec![],
                analysis_depth: AnalysisDepth::Surface,
            }
        }
    }

    /// Calculate task complexity score (0.0 to 1.0)
    async fn calculate_task_complexity(&self, task: &str, context: &[ContextItem]) -> f32 {
        let mut complexity = 0.0;

        let task_lower = task.to_lowercase();

        // Base complexity from task type
        if task_lower.contains("refactor") || task_lower.contains("architecture") {
            complexity += 0.4;
        } else if task_lower.contains("optimize") || task_lower.contains("performance") {
            complexity += 0.3;
        } else if task_lower.contains("fix") || task_lower.contains("bug") {
            complexity += 0.2;
        } else if task_lower.contains("create") || task_lower.contains("implement") {
            complexity += 0.3;
        }

        // Add complexity based on scope indicators
        if task_lower.contains("system") || task_lower.contains("codebase") {
            complexity += 0.3;
        } else if task_lower.contains("module") || task_lower.contains("component") {
            complexity += 0.2;
        }

        // Add complexity based on context size
        complexity += (context.len() as f32 * 0.1).min(0.3);

        // Add complexity for multi-step tasks
        if task_lower.contains(" and ") || task_lower.contains(", ") {
            complexity += 0.2;
        }

        // Cap at 1.0
        complexity.min(1.0)
    }

    /// Calculate confidence score based on available context (0.0 to 1.0)
    async fn calculate_confidence(&self, task: &str, context: &[ContextItem]) -> f32 {
        let mut confidence = 0.5; // Base confidence

        // Increase confidence with more context
        confidence += (context.len() as f32 * 0.1).min(0.3);

        // Increase confidence for specific, well-defined tasks
        let task_lower = task.to_lowercase();
        if task_lower.contains("specific") || task_lower.contains("exactly") {
            confidence += 0.2;
        }

        // Decrease confidence for vague tasks
        if task_lower.contains("somehow") || task_lower.contains("maybe") || task_lower.contains("might") {
            confidence -= 0.3;
        }

        // Increase confidence if semantic search is available
        if self.semantic_search.is_some() {
            confidence += 0.1;
        }

        // Cap between 0.0 and 1.0
        confidence.max(0.0).min(1.0)
    }

    /// Method for learning from strategy execution outcomes
    pub async fn learn_from_execution(
        &self,
        task: &str,
        strategy_name: &str,
        success: bool,
        learned_patterns: Vec<String>,
    ) -> Result<()> {
        // Record learning in DuckDB memory if available
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;

            let pattern = duckdb_memory::Pattern {
                id: uuid::Uuid::new_v4().to_string(),
                description: format!("Strategy: {} for task: {}", strategy_name, task),
                context: task.to_string(),
                actions: vec![duckdb_memory::AgentAction {
                    tool: "strategy_execution".to_string(),
                    params: serde_json::json!({"strategy": strategy_name}),
                    success,
                    duration_ms: 0,
                    result_summary: format!("Strategy {} execution {}", strategy_name, if success { "succeeded" } else { "failed" }),
                }],
                files_involved: vec![],
                success,
                timestamp: chrono::Utc::now(),
                session_id: "strategy_learning".to_string(),
                embedding_text: format!("{} {}", strategy_name, task),
                embedding: vec![],
            };

            memory_guard.record_pattern(pattern).await?;
        }

        info!(
            "Recorded strategy learning: strategy='{}', task='{}', success={}, patterns={}",
            strategy_name, task, success, learned_patterns.len()
        );

        Ok(())
    }

    /// Get enhanced context that includes purpose analysis (internal use only - automatic via UnifiedIntelligenceEngine)
    pub(crate) async fn get_enhanced_development_context(&self, query: &str, file_path: Option<&str>) -> Result<String> {
        let mut context = String::new();

        // Get base development context
        let base_context = self.get_development_context(query).await;
        context.push_str(&format!("**Development Phase**: {}\n", base_context.development_phase));
        context.push_str(&format!("**Confidence**: {:.1}%\n\n", base_context.confidence * 100.0));

        // Add purpose analysis if file path provided
        if let Some(path) = file_path {
            if let Ok(file_content) = tokio::fs::read_to_string(path).await {
                match self.get_business_context_summary(path, &file_content).await {
                    Ok(purpose_summary) => {
                        context.push_str("**Code Purpose Analysis**:\n");
                        context.push_str(&purpose_summary);
                        context.push_str("\n");
                    }
                    Err(e) => {
                        context.push_str(&format!("**Purpose Analysis Error**: {}\n", e));
                    }
                }
            }
        }

        // Add key files
        if !base_context.key_files.is_empty() {
            context.push_str("**Key Files**:\n");
            for file in &base_context.key_files {
                context.push_str(&format!("- {} ({})\n", file.path, file.relationship_to_current_work));
            }
            context.push_str("\n");
        }

        // Add architectural context
        if !base_context.architectural_context.is_empty() {
            context.push_str("**Architectural Context**:\n");
            for arch in &base_context.architectural_context {
                context.push_str(&format!("- {}\n", arch.decision));
                if !arch.rationale.is_empty() {
                    context.push_str(&format!("  Rationale: {}\n", arch.rationale));
                }
            }
            context.push_str("\n");
        }

        // Add suggested actions
        if !base_context.suggested_next_actions.is_empty() {
            context.push_str("**Suggested Actions**:\n");
            for action in base_context.suggested_next_actions.iter().take(3) {
                context.push_str(&format!("- {} ({})\n", action.description, action.action_type));
            }
        }

        Ok(context)
    }
}

#[async_trait]
impl IntelligenceTools for IntelligenceEngine {
    async fn get_development_context(&self, query: &str) -> ContextualInsight {
        // Combine all intelligence sources to provide comprehensive context
        let narrative = self.narrative_tracker.get_current_narrative().await;
        let relevance = self.context_engine.analyze_relevance(query).await;
        let memory = self.memory_system.get_relevant_patterns(query).await;
        
        // Enhance with semantic search results if available
        let mut enhanced_files = relevance.ranked_files;
        let mut enhanced_confidence = relevance.confidence;
        
        if let Some(semantic_search) = &self.semantic_search {
            let mut search_engine = semantic_search.write().await;
            match search_engine.search(query, 5).await {
                Ok((results, _metrics)) => {
                    let avg_similarity = if !results.is_empty() {
                        results.iter().map(|r| r.similarity_score as f64).sum::<f64>() / results.len() as f64
                    } else {
                        0.0
                    };
                    
                    // Add semantic search results to file context
                    for result in &results {
                        let semantic_file = FileWithContext {
                            path: result.file_path.to_string_lossy().to_string(),
                            relevance: ContextualRelevance {
                                immediate: result.similarity_score as f64,
                                sequential: 0.0,
                                dependent: 0.0,
                                reference: 0.0,
                                historical: 0.0,
                            },
                            purpose: format!("Semantic match: {}", 
                                result.chunk.content.lines().take(2).collect::<Vec<_>>().join(" ").chars().take(100).collect::<String>()),
                            last_significant_change: chrono::Utc::now(), // TODO: Get actual file modification time
                            relationship_to_current_work: format!("Semantic similarity: {:.2}%", result.similarity_score * 100.0),
                        };
                        enhanced_files.push(semantic_file);
                    }
                    
                    // Boost confidence if semantic search found good matches
                    if avg_similarity > 0.0 {
                        enhanced_confidence = (enhanced_confidence + avg_similarity) / 2.0;
                    }
                }
                Err(_) => {
                    // Semantic search failed, continue with existing results
                }
            }
        }

        // Enhance with persistent memory patterns if available
        let enhanced_patterns = memory.patterns;
        let enhanced_actions = relevance.predicted_actions;
        
        if let Some(_duckdb_memory) = &self.duckdb_memory {
            // Try to get patterns with a default session ID if none provided
            let _session_id = "current"; // In practice, this would come from the session context
            
            // Skip learned patterns for now - would need to integrate with DuckDBMemory
            // This would use duckdb_memory.find_similar_patterns() in a real implementation
            // TODO: Integrate DuckDB memory patterns here
        }

        ContextualInsight {
            development_phase: narrative.current_epic,
            active_story: narrative.recent_focus,
            key_files: enhanced_files,
            architectural_context: narrative.recent_decisions,
            recent_patterns: enhanced_patterns,
            suggested_next_actions: enhanced_actions,
            confidence: enhanced_confidence,
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
    
    async fn search_code_semantically(&self, query: &str, limit: usize) -> Result<Vec<CodeSearchResult>, String> {
        if let Some(semantic_search) = &self.semantic_search {
            let mut search_engine = semantic_search.write().await;
            match search_engine.search(query, limit).await {
                Ok((results, _metrics)) => {
                    let code_results = results.into_iter().map(|result| {
                        CodeSearchResult {
                            file_path: result.file_path.to_string_lossy().to_string(),
                            line_start: result.chunk.start_line,
                            line_end: result.chunk.end_line,
                            content: result.chunk.content,
                            similarity_score: result.similarity_score,
                            chunk_type: match result.chunk.chunk_type {
                                crate::vector_search::ChunkType::Function => "function".to_string(),
                                crate::vector_search::ChunkType::Class => "class".to_string(),
                                crate::vector_search::ChunkType::Module => "module".to_string(),
                                crate::vector_search::ChunkType::Comment => "comment".to_string(),
                                crate::vector_search::ChunkType::Generic => "generic".to_string(),
                            },
                            context_lines: result.context_lines,
                        }
                    }).collect();
                    Ok(code_results)
                }
                Err(e) => Err(format!("Semantic search failed: {}", e))
            }
        } else {
            Err("Semantic search not available - engine not initialized with search capability".to_string())
        }
    }
    
    async fn analyze_code_structure(&self, file_path: &str) -> Result<ASTAnalysis, String> {
        let path = std::path::Path::new(file_path);
        let mut analyzer = self.ast_analyzer.write().await;
        
        match analyzer.analyze_file(path).await {
            Ok(Some(analysis)) => Ok(analysis),
            Ok(None) => Err(format!("Unsupported file type or failed to parse: {}", file_path)),
            Err(e) => Err(format!("AST analysis failed: {}", e))
        }
    }
    
    async fn get_code_insights(&self, file_path: &str) -> Result<CodeInsights, String> {
        let analysis = self.analyze_code_structure(file_path).await?;
        
        // Calculate quality score
        let quality_score = crate::intelligence::ast_analysis::utils::calculate_quality_score(&analysis);
        
        // Extract key functions
        let key_functions = crate::intelligence::ast_analysis::utils::extract_function_signatures(&analysis);
        
        // Extract dependencies
        let dependencies = crate::intelligence::ast_analysis::utils::extract_dependencies(&analysis);
        
        // Generate complexity summary
        let complexity_summary = format!(
            "Cyclomatic: {}, Cognitive: {}, Depth: {}, LOC: {}, Comments: {:.1}%",
            analysis.complexity_metrics.cyclomatic_complexity,
            analysis.complexity_metrics.cognitive_complexity,
            analysis.complexity_metrics.nesting_depth,
            analysis.complexity_metrics.lines_of_code,
            analysis.complexity_metrics.comment_ratio * 100.0
        );
        
        // Extract patterns
        let patterns = analysis.patterns.iter()
            .map(|p| format!("{}: {}", p.pattern_type, p.description))
            .collect();
        
        // Generate suggestions based on analysis
        let mut suggestions = Vec::new();
        
        if analysis.complexity_metrics.cyclomatic_complexity > 10 {
            suggestions.push("Consider refactoring complex functions to reduce cyclomatic complexity".to_string());
        }
        
        if analysis.complexity_metrics.comment_ratio < 0.1 {
            suggestions.push("Add more comments to improve code documentation".to_string());
        }
        
        if analysis.functions.len() > 20 {
            suggestions.push("Consider splitting this file into smaller modules".to_string());
        }
        
        Ok(CodeInsights {
            file_path: file_path.to_string(),
            language: analysis.language.clone(),
            quality_score,
            complexity_summary,
            key_functions,
            dependencies,
            patterns,
            suggestions,
            ast_analysis: Some(analysis),
        })
    }
    
    async fn initialize_project_memory(&mut self, project_root: std::path::PathBuf) -> Result<(), String> {
        self.initialize_duckdb_memory(project_root).await
            .map_err(|e| format!("Failed to initialize project memory: {}", e))
    }
    
    async fn start_session(&self, session_id: Option<String>) -> Result<Option<String>, String> {
        self.start_memory_session(session_id).await
            .map_err(|e| format!("Failed to start memory session: {}", e))
    }
    
    async fn record_learning(
        &self,
        session_id: &str,
        user_query: &str,
        files_involved: &[String],
        tools_used: &[String],
        outcome: Outcome,
    ) -> Result<(), String> {
        // Record pattern to DuckDB memory
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;
            
            // Convert to pattern format
            let actions = tools_used.iter().map(|tool| AgentAction {
                tool: tool.clone(),
                params: serde_json::json!({}),
                success: outcome.success_rating > 0.5,
                duration_ms: 0,
                result_summary: format!("{:?}", outcome),
            }).collect();
            
            let pattern = duckdb_memory::Pattern {
                id: uuid::Uuid::new_v4().to_string(),
                description: user_query.to_string(),
                context: user_query.to_string(),
                actions,
                files_involved: files_involved.to_vec(),
                success: outcome.success_rating > 0.5,
                timestamp: Utc::now(),
                session_id: session_id.to_string(),
                embedding_text: user_query.to_string(),
                embedding: vec![], // Legacy code path, no embedding generation
            };
            
            memory_guard.record_pattern(pattern).await
                .map_err(|e| format!("Failed to record pattern: {}", e))
        } else {
            Ok(())
        }
    }
    
    async fn get_relevant_patterns(&self, query: &str, _session_id: &str) -> Result<Vec<String>, String> {
        // Use DuckDB memory to find similar patterns
        if let Some(memory) = &self.duckdb_memory {
            let memory_guard = memory.lock().await;
            let suggestions = memory_guard.suggest_next(query).await
                .map_err(|e| format!("Failed to get patterns: {}", e))?;
            Ok(vec![suggestions])
        } else {
            Ok(vec![])
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
pub struct CodeSearchResult {
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub content: String,
    pub similarity_score: f32,
    pub chunk_type: String,
    pub context_lines: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeInsights {
    pub file_path: String,
    pub language: String,
    pub quality_score: f32,
    pub complexity_summary: String,
    pub key_functions: Vec<String>,
    pub dependencies: Vec<String>,
    pub patterns: Vec<String>,
    pub suggestions: Vec<String>,
    pub ast_analysis: Option<ASTAnalysis>,
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
    pub legacy_claude: Option<String>,        // ./AGENTS.md (legacy field name)
    pub custom_instructions: Vec<CustomInstruction>,
}

#[derive(Debug, Clone)]
pub struct CustomInstruction {
    pub source_file: String,
    pub content: String,
    pub priority: u8,
}
