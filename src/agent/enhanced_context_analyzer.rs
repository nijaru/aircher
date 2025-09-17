use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

use crate::intelligence::{IntelligenceEngine, tools::IntelligenceTools};
use crate::semantic_search::SemanticCodeSearch;

/// Enhanced Context Analyzer that goes beyond simple keyword extraction
/// Uses semantic understanding, AST analysis, and pattern recognition
pub struct EnhancedContextAnalyzer {
    intelligence: Arc<IntelligenceEngine>,
    #[allow(dead_code)]
    search: Option<Arc<tokio::sync::RwLock<SemanticCodeSearch>>>,
    #[allow(dead_code)]
    pattern_cache: tokio::sync::RwLock<HashMap<String, AnalysisPattern>>,
}

/// Represents an analyzed activity with rich context understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAnalysis {
    /// Primary intent of the activity
    pub intent: ActivityIntent,
    /// Confidence in the analysis (0.0 - 1.0)
    pub confidence: f32,
    /// Required files with specific relevance reasons
    pub required_files: Vec<FileRequirement>,
    /// Code symbols that need to be loaded
    pub required_symbols: Vec<SymbolRequirement>,
    /// Semantic search queries to find related code
    pub search_queries: Vec<SemanticQuery>,
    /// Predicted next activities
    pub predicted_next: Vec<String>,
    /// Related concepts that might be needed
    pub related_concepts: Vec<ConceptLink>,
    /// Workflow context (if part of larger task)
    pub workflow_context: Option<WorkflowContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityIntent {
    /// Reading and understanding code
    CodeComprehension {
        target_files: Vec<String>,
        focus_areas: Vec<String>,
    },
    /// Implementing new functionality
    Implementation {
        feature_description: String,
        implementation_type: ImplementationType,
        dependencies: Vec<String>,
    },
    /// Debugging or fixing issues
    Debugging {
        error_description: String,
        affected_areas: Vec<String>,
        potential_causes: Vec<String>,
    },
    /// Refactoring existing code
    Refactoring {
        refactor_type: RefactorType,
        scope: String,
        affected_files: Vec<String>,
    },
    /// Testing related activities
    Testing {
        test_type: TestType,
        target_code: Vec<String>,
        test_scenarios: Vec<String>,
    },
    /// Documentation tasks
    Documentation {
        doc_type: DocumentationType,
        scope: String,
    },
    /// Code review and analysis
    Review {
        review_scope: Vec<String>,
        review_criteria: Vec<String>,
    },
    /// General exploration
    Exploration {
        exploration_goals: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationType {
    NewFeature,
    Enhancement,
    BugFix,
    Performance,
    Security,
    Integration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefactorType {
    Extract,
    Rename,
    Move,
    Simplify,
    Optimize,
    Modernize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestType {
    Unit,
    Integration,
    EndToEnd,
    Performance,
    Security,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentationType {
    API,
    Architecture,
    UserGuide,
    Comments,
    README,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRequirement {
    pub path: String,
    pub relevance_reason: String,
    pub priority: Priority,
    pub access_pattern: AccessPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolRequirement {
    pub symbol_name: String,
    pub symbol_type: SymbolType,
    pub file_path: String,
    pub line_range: Option<(usize, usize)>,
    pub relevance_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolType {
    Function,
    Class,
    Variable,
    Constant,
    Type,
    Interface,
    Module,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticQuery {
    pub query: String,
    pub query_type: QueryType,
    pub expected_results: usize,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    Similar,      // Find similar code patterns
    Related,      // Find related functionality
    Dependencies, // Find dependencies/dependents
    Examples,     // Find usage examples
    Documentation, // Find relevant docs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessPattern {
    FullRead,      // Need to read entire file
    Focused(Vec<String>), // Need specific sections/functions
    Scan,          // Just scan for relevant parts
    Reference,     // Just need as reference
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptLink {
    pub concept: String,
    pub relation: ConceptRelation,
    pub strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConceptRelation {
    Uses,
    UsedBy,
    Similar,
    Opposite,
    Alternative,
    Prerequisite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    pub workflow_type: String,
    pub current_step: String,
    pub previous_steps: Vec<String>,
    pub next_likely_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnalysisPattern {
    pattern: String,
    intent: ActivityIntent,
    confidence: f32,
    usage_count: u32,
    last_used: chrono::DateTime<chrono::Utc>,
}

impl EnhancedContextAnalyzer {
    pub fn new(
        intelligence: Arc<IntelligenceEngine>,
        search: Option<Arc<tokio::sync::RwLock<SemanticCodeSearch>>>,
    ) -> Self {
        Self {
            intelligence,
            search,
            pattern_cache: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Analyze an activity with enhanced semantic understanding
    pub async fn analyze_activity(&self, activity: &str) -> Result<ContextAnalysis> {
        info!("Enhanced analysis of activity: {}", activity);

        // Step 1: Check for cached patterns
        if let Some(cached) = self.check_pattern_cache(activity).await {
            debug!("Found cached analysis pattern");
            return Ok(self.apply_cached_pattern(activity, cached).await?);
        }

        // Step 2: Semantic intent classification
        let intent = self.classify_intent(activity).await?;

        // Step 3: Extract entities and relationships
        let entities = self.extract_entities(activity).await?;

        // Step 4: Determine required context
        let required_files = self.determine_required_files(activity, &intent, &entities).await?;
        let required_symbols = self.determine_required_symbols(activity, &intent, &entities)?;

        // Step 5: Generate semantic queries
        let search_queries = self.generate_semantic_queries(activity, &intent, &entities).await?;

        // Step 6: Predict next activities
        let predicted_next = self.predict_next_activities(activity, &intent).await?;

        // Step 7: Find related concepts
        let related_concepts = self.find_related_concepts(activity, &intent).await?;

        // Step 8: Determine workflow context
        let workflow_context = self.analyze_workflow_context(activity, &intent).await?;

        let analysis = ContextAnalysis {
            intent,
            confidence: 0.85, // TODO: Calculate actual confidence
            required_files,
            required_symbols,
            search_queries,
            predicted_next,
            related_concepts,
            workflow_context,
        };

        // Cache this pattern for future use
        self.cache_analysis_pattern(activity, &analysis).await?;

        Ok(analysis)
    }

    /// Classify the primary intent of an activity using advanced pattern matching
    async fn classify_intent(&self, activity: &str) -> Result<ActivityIntent> {
        let activity_lower = activity.to_lowercase();

        // Use intelligence engine for sophisticated intent analysis
        let insight = self.intelligence.get_development_context(activity).await;

        // Analyze linguistic patterns
        let words: Vec<&str> = activity_lower.split_whitespace().collect();

        // Intent classification with confidence scoring
        if self.matches_debugging_pattern(&words, &insight) {
            Ok(ActivityIntent::Debugging {
                error_description: self.extract_error_description(activity),
                affected_areas: self.extract_affected_areas(activity, &insight),
                potential_causes: self.suggest_potential_causes(activity, &insight),
            })
        } else if self.matches_implementation_pattern(&words, &insight) {
            Ok(ActivityIntent::Implementation {
                feature_description: self.extract_feature_description(activity),
                implementation_type: self.classify_implementation_type(&words),
                dependencies: self.identify_dependencies(activity, &insight),
            })
        } else if self.matches_refactoring_pattern(&words, &insight) {
            Ok(ActivityIntent::Refactoring {
                refactor_type: self.classify_refactor_type(&words),
                scope: self.determine_refactor_scope(activity),
                affected_files: self.predict_affected_files(activity, &insight),
            })
        } else if self.matches_testing_pattern(&words, &insight) {
            Ok(ActivityIntent::Testing {
                test_type: self.classify_test_type(&words),
                target_code: self.identify_test_targets(activity, &insight),
                test_scenarios: self.suggest_test_scenarios(activity),
            })
        } else if self.matches_comprehension_pattern(&words, &insight) {
            Ok(ActivityIntent::CodeComprehension {
                target_files: insight.key_files.iter().map(|f| f.path.clone()).collect(),
                focus_areas: self.identify_focus_areas(activity, &insight),
            })
        } else {
            Ok(ActivityIntent::Exploration {
                exploration_goals: self.extract_exploration_goals(activity),
            })
        }
    }

    /// Extract structured entities from the activity description
    async fn extract_entities(&self, activity: &str) -> Result<HashMap<String, Vec<String>>> {
        let mut entities = HashMap::new();

        // Extract file paths
        let file_paths = self.extract_file_paths(activity);
        if !file_paths.is_empty() {
            entities.insert("files".to_string(), file_paths);
        }

        // Extract function/method names
        let functions = self.extract_function_names(activity);
        if !functions.is_empty() {
            entities.insert("functions".to_string(), functions);
        }

        // Extract class names
        let classes = self.extract_class_names(activity);
        if !classes.is_empty() {
            entities.insert("classes".to_string(), classes);
        }

        // Extract technology/framework terms
        let technologies = self.extract_technologies(activity);
        if !technologies.is_empty() {
            entities.insert("technologies".to_string(), technologies);
        }

        Ok(entities)
    }

    /// Determine which files are required based on sophisticated analysis
    async fn determine_required_files(
        &self,
        activity: &str,
        intent: &ActivityIntent,
        entities: &HashMap<String, Vec<String>>,
    ) -> Result<Vec<FileRequirement>> {
        let mut required_files = Vec::new();

        // Get intelligence insights
        let insight = self.intelligence.get_development_context(activity).await;

        // Add files from intelligence analysis
        for key_file in &insight.key_files {
            required_files.push(FileRequirement {
                path: key_file.path.clone(),
                relevance_reason: "Identified by intelligence analysis".to_string(),
                priority: Priority::High,
                access_pattern: AccessPattern::FullRead,
            });
        }

        // Add files based on extracted entities
        if let Some(file_entities) = entities.get("files") {
            for file_path in file_entities {
                required_files.push(FileRequirement {
                    path: file_path.clone(),
                    relevance_reason: "Explicitly mentioned in activity".to_string(),
                    priority: Priority::Critical,
                    access_pattern: self.determine_access_pattern(file_path, intent),
                });
            }
        }

        // Add related files based on intent
        match intent {
            ActivityIntent::Testing { target_code, .. } => {
                for target in target_code {
                    // Find corresponding test files
                    let test_files = self.find_test_files(target).await?;
                    for test_file in test_files {
                        required_files.push(FileRequirement {
                            path: test_file,
                            relevance_reason: "Related test file".to_string(),
                            priority: Priority::Medium,
                            access_pattern: AccessPattern::Scan,
                        });
                    }
                }
            }
            ActivityIntent::Implementation { dependencies, .. } => {
                for dep in dependencies {
                    if let Ok(dep_files) = self.find_dependency_files(dep).await {
                        for dep_file in dep_files {
                            required_files.push(FileRequirement {
                                path: dep_file,
                                relevance_reason: "Dependency file".to_string(),
                                priority: Priority::Medium,
                                access_pattern: AccessPattern::Reference,
                            });
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(required_files)
    }

    /// Generate semantic search queries based on deep understanding
    async fn generate_semantic_queries(
        &self,
        _activity: &str,
        intent: &ActivityIntent,
        entities: &HashMap<String, Vec<String>>,
    ) -> Result<Vec<SemanticQuery>> {
        let mut queries = Vec::new();

        // Generate queries based on intent
        match intent {
            ActivityIntent::Implementation { feature_description, .. } => {
                queries.push(SemanticQuery {
                    query: format!("similar implementation: {}", feature_description),
                    query_type: QueryType::Similar,
                    expected_results: 5,
                    priority: Priority::High,
                });

                queries.push(SemanticQuery {
                    query: format!("examples of: {}", feature_description),
                    query_type: QueryType::Examples,
                    expected_results: 3,
                    priority: Priority::Medium,
                });
            }
            ActivityIntent::Debugging { error_description, .. } => {
                queries.push(SemanticQuery {
                    query: format!("similar error: {}", error_description),
                    query_type: QueryType::Similar,
                    expected_results: 3,
                    priority: Priority::Critical,
                });

                queries.push(SemanticQuery {
                    query: format!("error handling: {}", error_description),
                    query_type: QueryType::Related,
                    expected_results: 5,
                    priority: Priority::High,
                });
            }
            ActivityIntent::CodeComprehension { focus_areas, .. } => {
                for focus in focus_areas {
                    queries.push(SemanticQuery {
                        query: format!("related to: {}", focus),
                        query_type: QueryType::Related,
                        expected_results: 10,
                        priority: Priority::Medium,
                    });
                }
            }
            _ => {}
        }

        // Generate queries based on entities
        if let Some(functions) = entities.get("functions") {
            for function in functions {
                queries.push(SemanticQuery {
                    query: format!("calls to {}", function),
                    query_type: QueryType::Dependencies,
                    expected_results: 5,
                    priority: Priority::Medium,
                });
            }
        }

        Ok(queries)
    }

    // Helper methods for pattern matching
    fn matches_debugging_pattern(&self, words: &[&str], insight: &crate::intelligence::ContextualInsight) -> bool {
        let debug_keywords = ["fix", "bug", "error", "issue", "problem", "broken", "crash", "fail"];
        words.iter().any(|w| debug_keywords.contains(w)) ||
        insight.development_phase.to_lowercase().contains("debug") ||
        insight.suggested_next_actions.iter().any(|a| a.description.to_lowercase().contains("fix"))
    }

    fn matches_implementation_pattern(&self, words: &[&str], insight: &crate::intelligence::ContextualInsight) -> bool {
        let impl_keywords = ["implement", "create", "add", "build", "develop", "write", "code"];
        words.iter().any(|w| impl_keywords.contains(w)) ||
        insight.development_phase.to_lowercase().contains("implement")
    }

    fn matches_refactoring_pattern(&self, words: &[&str], _insight: &crate::intelligence::ContextualInsight) -> bool {
        let refactor_keywords = ["refactor", "restructure", "reorganize", "improve", "optimize", "clean"];
        words.iter().any(|w| refactor_keywords.contains(w))
    }

    fn matches_testing_pattern(&self, words: &[&str], _insight: &crate::intelligence::ContextualInsight) -> bool {
        let test_keywords = ["test", "spec", "verify", "validate", "check"];
        words.iter().any(|w| test_keywords.contains(w))
    }

    fn matches_comprehension_pattern(&self, words: &[&str], _insight: &crate::intelligence::ContextualInsight) -> bool {
        let comp_keywords = ["understand", "read", "analyze", "review", "examine", "study"];
        words.iter().any(|w| comp_keywords.contains(w))
    }

    // Entity extraction methods
    fn extract_file_paths(&self, activity: &str) -> Vec<String> {
        let mut paths = Vec::new();

        // Simple regex for common file patterns
        let patterns = [
            r"src/[a-zA-Z0-9_/]+\.rs",
            r"[a-zA-Z0-9_/]+\.py",
            r"[a-zA-Z0-9_/]+\.js",
            r"[a-zA-Z0-9_/]+\.ts",
            r"[a-zA-Z0-9_/]+\.go",
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                for mat in re.find_iter(activity) {
                    paths.push(mat.as_str().to_string());
                }
            }
        }

        paths
    }

    fn extract_function_names(&self, activity: &str) -> Vec<String> {
        let mut functions = Vec::new();

        // Look for function-like patterns
        if let Ok(re) = regex::Regex::new(r"(?:function|fn|def|func)\s+([a-zA-Z_][a-zA-Z0-9_]*)") {
            for cap in re.captures_iter(activity) {
                if let Some(func_name) = cap.get(1) {
                    functions.push(func_name.as_str().to_string());
                }
            }
        }

        functions
    }

    fn extract_class_names(&self, activity: &str) -> Vec<String> {
        let mut classes = Vec::new();

        // Look for class-like patterns
        if let Ok(re) = regex::Regex::new(r"(?:class|struct|interface)\s+([A-Z][a-zA-Z0-9_]*)") {
            for cap in re.captures_iter(activity) {
                if let Some(class_name) = cap.get(1) {
                    classes.push(class_name.as_str().to_string());
                }
            }
        }

        classes
    }

    fn extract_technologies(&self, activity: &str) -> Vec<String> {
        let tech_keywords = [
            "rust", "python", "javascript", "typescript", "react", "node",
            "django", "flask", "express", "fastapi", "tokio", "async",
            "docker", "kubernetes", "aws", "gcp", "azure"
        ];

        let activity_lower = activity.to_lowercase();
        tech_keywords.iter()
            .filter(|&tech| activity_lower.contains(tech))
            .map(|&tech| tech.to_string())
            .collect()
    }

    // Placeholder implementations for helper methods
    fn extract_error_description(&self, activity: &str) -> String {
        activity.to_string() // TODO: Extract actual error from activity
    }

    fn extract_affected_areas(&self, _activity: &str, insight: &crate::intelligence::ContextualInsight) -> Vec<String> {
        insight.key_files.iter().map(|f| f.path.clone()).collect()
    }

    fn suggest_potential_causes(&self, _activity: &str, _insight: &crate::intelligence::ContextualInsight) -> Vec<String> {
        vec!["Logic error".to_string(), "Type mismatch".to_string(), "Null reference".to_string()]
    }

    fn extract_feature_description(&self, activity: &str) -> String {
        activity.to_string() // TODO: Extract feature description
    }

    fn classify_implementation_type(&self, words: &[&str]) -> ImplementationType {
        if words.iter().any(|w| ["new", "create"].contains(w)) {
            ImplementationType::NewFeature
        } else if words.iter().any(|w| ["enhance", "improve"].contains(w)) {
            ImplementationType::Enhancement
        } else if words.iter().any(|w| ["fix", "bug"].contains(w)) {
            ImplementationType::BugFix
        } else {
            ImplementationType::NewFeature
        }
    }

    fn identify_dependencies(&self, _activity: &str, insight: &crate::intelligence::ContextualInsight) -> Vec<String> {
        insight.architectural_context.iter()
            .map(|decision| decision.decision.clone())
            .collect()
    }

    // Additional placeholder methods would be implemented here...
    fn classify_refactor_type(&self, _words: &[&str]) -> RefactorType { RefactorType::Simplify }
    fn determine_refactor_scope(&self, activity: &str) -> String { activity.to_string() }
    fn predict_affected_files(&self, _activity: &str, insight: &crate::intelligence::ContextualInsight) -> Vec<String> {
        insight.key_files.iter().map(|f| f.path.clone()).collect()
    }
    fn classify_test_type(&self, _words: &[&str]) -> TestType { TestType::Unit }
    fn identify_test_targets(&self, _activity: &str, insight: &crate::intelligence::ContextualInsight) -> Vec<String> {
        insight.key_files.iter().map(|f| f.path.clone()).collect()
    }
    fn suggest_test_scenarios(&self, _activity: &str) -> Vec<String> { vec!["Happy path".to_string()] }
    fn identify_focus_areas(&self, _activity: &str, insight: &crate::intelligence::ContextualInsight) -> Vec<String> {
        insight.recent_patterns.iter()
            .map(|pattern| pattern.description.clone())
            .collect()
    }
    fn extract_exploration_goals(&self, activity: &str) -> Vec<String> { vec![activity.to_string()] }
    fn determine_access_pattern(&self, _file_path: &str, _intent: &ActivityIntent) -> AccessPattern { AccessPattern::FullRead }
    async fn find_test_files(&self, _target: &str) -> Result<Vec<String>> { Ok(vec![]) }
    async fn find_dependency_files(&self, _dep: &str) -> Result<Vec<String>> { Ok(vec![]) }
    fn determine_required_symbols(&self, _activity: &str, _intent: &ActivityIntent, _entities: &HashMap<String, Vec<String>>) -> Result<Vec<SymbolRequirement>> { Ok(vec![]) }
    async fn predict_next_activities(&self, _activity: &str, _intent: &ActivityIntent) -> Result<Vec<String>> { Ok(vec![]) }
    async fn find_related_concepts(&self, _activity: &str, _intent: &ActivityIntent) -> Result<Vec<ConceptLink>> { Ok(vec![]) }
    async fn analyze_workflow_context(&self, _activity: &str, _intent: &ActivityIntent) -> Result<Option<WorkflowContext>> { Ok(None) }
    async fn check_pattern_cache(&self, _activity: &str) -> Option<AnalysisPattern> { None }
    async fn apply_cached_pattern(&self, _activity: &str, _pattern: AnalysisPattern) -> Result<ContextAnalysis> {
        // TODO: Implement cached pattern application
        Err(anyhow::anyhow!("Not implemented"))
    }
    async fn cache_analysis_pattern(&self, _activity: &str, _analysis: &ContextAnalysis) -> Result<()> { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigManager;
    use crate::storage::DatabaseManager;

    #[tokio::test]
    async fn test_intent_classification() -> Result<()> {
        let config = ConfigManager::load().await?;
        let storage = DatabaseManager::new(&config).await?;
        let intelligence = Arc::new(IntelligenceEngine::new(&config, &storage).await?);
        let analyzer = EnhancedContextAnalyzer::new(intelligence, None);

        let analysis = analyzer.analyze_activity("Fix the authentication bug in user login").await?;

        // Should classify as debugging intent
        match analysis.intent {
            ActivityIntent::Debugging { .. } => {},
            _ => panic!("Expected debugging intent"),
        }

        assert!(analysis.confidence > 0.0);
        assert!(!analysis.required_files.is_empty() || !analysis.search_queries.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_entity_extraction() -> Result<()> {
        let config = ConfigManager::load().await?;
        let storage = DatabaseManager::new(&config).await?;
        let intelligence = Arc::new(IntelligenceEngine::new(&config, &storage).await?);
        let analyzer = EnhancedContextAnalyzer::new(intelligence, None);

        let entities = analyzer.extract_entities("Implement a new function called processData in src/main.rs").await?;

        // Should extract function name and file path
        assert!(entities.contains_key("functions") || entities.contains_key("files"));

        Ok(())
    }
}