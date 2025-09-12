use crate::config::ConfigManager;
use crate::storage::DatabaseManager;
use crate::semantic_search::SemanticCodeSearch;
use anyhow::Result;
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tracing::warn;

pub mod context;
pub mod narrative;
pub mod memory;
pub mod persistent_memory;
// pub mod simple_lance_memory;  // Disabled due to arrow dependency conflict
// pub mod intelligent_memory;   // Disabled due to arrow dependency conflict
pub mod duckdb_memory;
pub mod tools;
pub mod tui_tools;
pub mod file_monitor;
pub mod mcp_integration;
pub mod mcp_examples;
pub mod ast_analysis;

pub use context::*;
pub use narrative::*;
pub use memory::*;
pub use persistent_memory::*;
// pub use simple_lance_memory::*;  // Disabled due to arrow dependency conflict
// pub use intelligent_memory::*;   // Disabled due to arrow dependency conflict
pub use duckdb_memory::*;
pub use tools::*;
pub use mcp_integration::*;
pub use ast_analysis::*;

/// Intelligence Engine - Context-aware development assistant for AI agents
pub struct IntelligenceEngine {
    _config: ConfigManager,
    _storage: DatabaseManager,
    context_engine: ContextualRelevanceEngine,
    narrative_tracker: DevelopmentNarrativeTracker,
    memory_system: ConversationalMemorySystem,
    duckdb_memory: Option<Arc<Mutex<DuckDBMemory>>>,
    semantic_search: Option<Arc<RwLock<SemanticCodeSearch>>>,
    ast_analyzer: Arc<RwLock<ASTAnalyzer>>,
}

impl IntelligenceEngine {
    pub async fn new(config: &ConfigManager, storage: &DatabaseManager) -> Result<Self> {
        let context_engine = ContextualRelevanceEngine::new(config).await?;
        let narrative_tracker = DevelopmentNarrativeTracker::new(config).await?;
        let memory_system = ConversationalMemorySystem::new(storage).await?;
        let ast_analyzer = Arc::new(RwLock::new(ASTAnalyzer::new()?));

        Ok(Self {
            _config: config.clone(),
            _storage: storage.clone(),
            context_engine,
            narrative_tracker,
            memory_system,
            duckdb_memory: None,
            semantic_search: None,
            ast_analyzer,
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

        Ok(Self {
            _config: config.clone(),
            _storage: storage.clone(),
            context_engine,
            narrative_tracker,
            memory_system,
            duckdb_memory: None,
            semantic_search: Some(Arc::new(RwLock::new(semantic_search))),
            ast_analyzer,
        })
    }
    
    /// Add semantic search to existing intelligence engine
    pub fn set_semantic_search(&mut self, semantic_search: SemanticCodeSearch) {
        self.semantic_search = Some(Arc::new(RwLock::new(semantic_search)));
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
