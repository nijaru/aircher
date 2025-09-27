use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

use crate::intelligence::ast_analysis::ASTAnalyzer;
use crate::semantic_search::SemanticCodeSearch;

/// Purpose Analysis Engine - Understands what code does and why it exists
/// Goes beyond syntax to extract business logic and intent
#[derive(Clone)]
pub struct PurposeAnalysisEngine {
    ast_analyzer: Arc<tokio::sync::RwLock<ASTAnalyzer>>,
    semantic_search: Option<Arc<tokio::sync::RwLock<SemanticCodeSearch>>>,
    pattern_cache: Arc<tokio::sync::RwLock<HashMap<String, CachedPurposeAnalysis>>>,
}

/// Comprehensive analysis of code purpose and business context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePurposeAnalysis {
    /// Primary business purpose of the code
    pub primary_purpose: BusinessPurpose,
    /// Confidence in the analysis (0.0 - 1.0)
    pub confidence: f32,
    /// Extracted business rules and logic
    pub business_rules: Vec<BusinessRule>,
    /// Domain concepts identified in the code
    pub domain_concepts: Vec<DomainConcept>,
    /// Criticality assessment
    pub criticality: CodeCriticality,
    /// Quality assessment and issues
    pub quality_assessment: QualityAssessment,
    /// Dependencies and relationships
    pub dependencies: DependencyAnalysis,
    /// Suggested improvements
    pub improvements: Vec<ImprovementSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BusinessPurpose {
    /// Core business logic implementation
    CoreLogic {
        domain: String,
        operation: String,
        business_value: String,
    },
    /// Data processing and transformation
    DataProcessing {
        input_type: String,
        output_type: String,
        transformation_logic: String,
    },
    /// User interface and interaction
    UserInterface {
        ui_type: String,
        user_actions: Vec<String>,
        data_displayed: Vec<String>,
    },
    /// Integration and communication
    Integration {
        integration_type: String,
        external_systems: Vec<String>,
        data_exchanged: Vec<String>,
    },
    /// Infrastructure and utilities
    Infrastructure {
        infrastructure_type: String,
        services_provided: Vec<String>,
    },
    /// Testing and validation
    Testing {
        test_type: String,
        code_under_test: String,
        scenarios_covered: Vec<String>,
    },
    /// Configuration and setup
    Configuration {
        config_type: String,
        settings_managed: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRule {
    pub rule_description: String,
    pub rule_type: BusinessRuleType,
    pub enforcement_level: EnforcementLevel,
    pub code_location: String,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BusinessRuleType {
    Validation,
    Authorization,
    BusinessLogic,
    DataConstraint,
    WorkflowRule,
    PolicyEnforcement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementLevel {
    Strict,      // Must always be enforced
    Conditional, // Enforced under certain conditions
    Advisory,    // Warning or guidance
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConcept {
    pub concept_name: String,
    pub concept_type: ConceptType,
    pub description: String,
    pub relationships: Vec<ConceptRelationship>,
    pub code_representations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConceptType {
    Entity,     // Core business entity
    ValueObject, // Value object in DDD terms
    Service,    // Business service
    Process,    // Business process
    Rule,       // Business rule
    Event,      // Domain event
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptRelationship {
    pub related_concept: String,
    pub relationship_type: RelationshipType,
    pub strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    Uses,
    UsedBy,
    Contains,
    ContainedBy,
    Depends,
    DependedBy,
    Triggers,
    TriggeredBy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodeCriticality {
    Critical,   // Core business functionality
    Important,  // Significant feature
    Standard,   // Regular functionality
    Utility,    // Helper/utility code
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessment {
    pub overall_quality: QualityLevel,
    pub code_smells: Vec<CodeSmell>,
    pub maintainability_score: f32,
    pub readability_score: f32,
    pub test_coverage_assessment: TestCoverageAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityLevel {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSmell {
    pub smell_type: String,
    pub description: String,
    pub severity: SmellSeverity,
    pub location: String,
    pub suggested_refactor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmellSeverity {
    Critical,
    Major,
    Minor,
    Suggestion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCoverageAssessment {
    pub has_tests: bool,
    pub test_types: Vec<String>,
    pub coverage_gaps: Vec<String>,
    pub test_quality: QualityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    pub internal_dependencies: Vec<InternalDependency>,
    pub external_dependencies: Vec<ExternalDependency>,
    pub circular_dependencies: Vec<String>,
    pub dependency_health: DependencyHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalDependency {
    pub module_path: String,
    pub dependency_type: DependencyType,
    pub coupling_strength: CouplingStrength,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDependency {
    pub library_name: String,
    pub usage_purpose: String,
    pub version_info: Option<String>,
    pub security_assessment: SecurityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    DirectUsage,
    Inheritance,
    Composition,
    Interface,
    Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CouplingStrength {
    Tight,   // Highly coupled
    Moderate, // Some coupling
    Loose,   // Loosely coupled
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyHealth {
    Healthy,
    Concerning,
    Problematic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Trusted,
    Verified,
    Unknown,
    Suspicious,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementSuggestion {
    pub suggestion_type: ImprovementType,
    pub description: String,
    pub priority: Priority,
    pub estimated_effort: EstimatedEffort,
    pub expected_benefit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImprovementType {
    Refactoring,
    Performance,
    Security,
    Maintainability,
    Testing,
    Documentation,
    Architecture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EstimatedEffort {
    Minimal,   // < 1 hour
    Small,     // 1-4 hours
    Medium,    // 1-2 days
    Large,     // 3-5 days
    XLarge,    // > 1 week
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedPurposeAnalysis {
    analysis: CodePurposeAnalysis,
    timestamp: chrono::DateTime<chrono::Utc>,
    file_hash: String,
}

impl PurposeAnalysisEngine {
    pub fn new(
        ast_analyzer: Arc<tokio::sync::RwLock<ASTAnalyzer>>,
        semantic_search: Option<Arc<tokio::sync::RwLock<SemanticCodeSearch>>>,
    ) -> Self {
        Self {
            ast_analyzer,
            semantic_search,
            pattern_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Analyze the purpose and business context of code
    pub async fn analyze_code_purpose(
        &self,
        file_path: &str,
        code_content: &str,
    ) -> Result<CodePurposeAnalysis> {
        info!("Analyzing purpose of code in: {}", file_path);

        // Check cache first
        let file_hash = self.calculate_content_hash(code_content);
        if let Some(cached) = self.get_cached_analysis(file_path, &file_hash).await {
            debug!("Using cached purpose analysis for {}", file_path);
            return Ok(cached.analysis);
        }

        // Step 1: Extract structural information from AST
        let ast_info = self.extract_ast_information(file_path, code_content).await?;

        // Step 2: Analyze business purpose
        let primary_purpose = self.determine_primary_purpose(file_path, code_content, &ast_info).await?;

        // Step 3: Extract business rules
        let business_rules = self.extract_business_rules(code_content, &ast_info).await?;

        // Step 4: Identify domain concepts
        let domain_concepts = self.identify_domain_concepts(code_content, &ast_info).await?;

        // Step 5: Assess criticality
        let criticality = self.assess_code_criticality(file_path, code_content, &ast_info).await?;

        // Step 6: Quality assessment
        let quality_assessment = self.assess_code_quality(code_content, &ast_info).await?;

        // Step 7: Dependency analysis
        let dependencies = self.analyze_dependencies(file_path, code_content, &ast_info).await?;

        // Step 8: Generate improvement suggestions
        let improvements = self.generate_improvement_suggestions(&quality_assessment, &dependencies).await?;

        let analysis = CodePurposeAnalysis {
            primary_purpose,
            confidence: 0.85, // TODO: Calculate actual confidence based on analysis quality
            business_rules,
            domain_concepts,
            criticality,
            quality_assessment,
            dependencies,
            improvements,
        };

        // Cache the analysis
        self.cache_analysis(file_path, &file_hash, &analysis).await?;

        Ok(analysis)
    }

    /// Get business context summary for quick understanding
    pub async fn get_business_context_summary(
        &self,
        file_path: &str,
        code_content: &str,
    ) -> Result<String> {
        let analysis = self.analyze_code_purpose(file_path, code_content).await?;

        let mut summary = String::new();

        // Primary purpose
        summary.push_str(&format!("**Purpose**: {}\n", self.format_business_purpose(&analysis.primary_purpose)));

        // Criticality
        summary.push_str(&format!("**Criticality**: {:?}\n", analysis.criticality));

        // Key business rules
        if !analysis.business_rules.is_empty() {
            summary.push_str("**Key Business Rules**:\n");
            for rule in analysis.business_rules.iter().take(3) {
                summary.push_str(&format!("- {}\n", rule.rule_description));
            }
        }

        // Quality highlights
        summary.push_str(&format!("**Quality**: {:?}", analysis.quality_assessment.overall_quality));
        if !analysis.quality_assessment.code_smells.is_empty() {
            let critical_smells: Vec<_> = analysis.quality_assessment.code_smells
                .iter()
                .filter(|s| matches!(s.severity, SmellSeverity::Critical | SmellSeverity::Major))
                .collect();
            if !critical_smells.is_empty() {
                summary.push_str(&format!(" ({} issues)", critical_smells.len()));
            }
        }
        summary.push('\n');

        // Top improvement suggestion
        if let Some(top_improvement) = analysis.improvements.iter()
            .filter(|i| matches!(i.priority, Priority::Critical | Priority::High))
            .next() {
            summary.push_str(&format!("**Recommended**: {}\n", top_improvement.description));
        }

        Ok(summary)
    }

    // Helper methods for analysis implementation

    async fn extract_ast_information(
        &self,
        file_path: &str,
        _code_content: &str,
    ) -> Result<serde_json::Value> {
        let mut analyzer = self.ast_analyzer.write().await;
        let path = std::path::Path::new(file_path);
        let analysis = analyzer.analyze_file(path).await?;
        Ok(serde_json::to_value(analysis)?)
    }

    async fn determine_primary_purpose(
        &self,
        file_path: &str,
        code_content: &str,
        _ast_info: &serde_json::Value,
    ) -> Result<BusinessPurpose> {
        // Analyze file path and content patterns to determine purpose
        let file_path_lower = file_path.to_lowercase();
        let content_lower = code_content.to_lowercase();

        // Check for common patterns
        if file_path_lower.contains("test") || content_lower.contains("#[test]") || content_lower.contains("fn test_") {
            Ok(BusinessPurpose::Testing {
                test_type: self.determine_test_type(&content_lower),
                code_under_test: self.extract_code_under_test(file_path),
                scenarios_covered: self.extract_test_scenarios(&content_lower),
            })
        } else if file_path_lower.contains("config") || content_lower.contains("configuration") {
            Ok(BusinessPurpose::Configuration {
                config_type: self.determine_config_type(&content_lower),
                settings_managed: self.extract_config_settings(&content_lower),
            })
        } else if file_path_lower.contains("ui") || file_path_lower.contains("view") || content_lower.contains("render") {
            Ok(BusinessPurpose::UserInterface {
                ui_type: self.determine_ui_type(&content_lower),
                user_actions: self.extract_user_actions(&content_lower),
                data_displayed: self.extract_displayed_data(&content_lower),
            })
        } else if content_lower.contains("api") || content_lower.contains("client") || content_lower.contains("request") {
            Ok(BusinessPurpose::Integration {
                integration_type: "API Integration".to_string(),
                external_systems: self.extract_external_systems(&content_lower),
                data_exchanged: self.extract_data_exchanged(&content_lower),
            })
        } else if self.is_infrastructure_code(&file_path_lower, &content_lower) {
            Ok(BusinessPurpose::Infrastructure {
                infrastructure_type: self.determine_infrastructure_type(&content_lower),
                services_provided: self.extract_services_provided(&content_lower),
            })
        } else if self.is_data_processing(&content_lower) {
            Ok(BusinessPurpose::DataProcessing {
                input_type: self.extract_input_type(&content_lower),
                output_type: self.extract_output_type(&content_lower),
                transformation_logic: self.extract_transformation_logic(&content_lower),
            })
        } else {
            // Default to core logic analysis
            Ok(BusinessPurpose::CoreLogic {
                domain: self.extract_domain_from_path(file_path),
                operation: self.extract_primary_operation(&content_lower),
                business_value: self.infer_business_value(&content_lower),
            })
        }
    }

    async fn extract_business_rules(
        &self,
        code_content: &str,
        _ast_info: &serde_json::Value,
    ) -> Result<Vec<BusinessRule>> {
        let mut rules = Vec::new();

        // Look for validation patterns
        if let Some(rule) = self.extract_validation_rules(code_content) {
            rules.extend(rule);
        }

        // Look for authorization patterns
        if let Some(rule) = self.extract_authorization_rules(code_content) {
            rules.extend(rule);
        }

        // Look for business logic constraints
        if let Some(rule) = self.extract_business_logic_rules(code_content) {
            rules.extend(rule);
        }

        Ok(rules)
    }

    async fn identify_domain_concepts(
        &self,
        code_content: &str,
        _ast_info: &serde_json::Value,
    ) -> Result<Vec<DomainConcept>> {
        let mut concepts = Vec::new();

        // Extract entities (structs, classes)
        concepts.extend(self.extract_entity_concepts(code_content));

        // Extract services (modules, functions with business logic)
        concepts.extend(self.extract_service_concepts(code_content));

        // Extract processes (workflows, state machines)
        concepts.extend(self.extract_process_concepts(code_content));

        Ok(concepts)
    }

    async fn assess_code_criticality(
        &self,
        file_path: &str,
        code_content: &str,
        _ast_info: &serde_json::Value,
    ) -> Result<CodeCriticality> {
        let file_path_lower = file_path.to_lowercase();
        let content_lower = code_content.to_lowercase();

        // Assess based on various factors
        if file_path_lower.contains("core") || file_path_lower.contains("main") ||
           content_lower.contains("critical") || content_lower.contains("essential") {
            Ok(CodeCriticality::Critical)
        } else if self.has_business_logic_indicators(&content_lower) {
            Ok(CodeCriticality::Important)
        } else if file_path_lower.contains("util") || file_path_lower.contains("helper") {
            Ok(CodeCriticality::Utility)
        } else {
            Ok(CodeCriticality::Standard)
        }
    }

    async fn assess_code_quality(
        &self,
        code_content: &str,
        _ast_info: &serde_json::Value,
    ) -> Result<QualityAssessment> {
        let code_smells = self.detect_code_smells(code_content);
        let maintainability_score = self.calculate_maintainability_score(code_content);
        let readability_score = self.calculate_readability_score(code_content);
        let test_coverage_assessment = self.assess_test_coverage(code_content);

        let overall_quality = self.determine_overall_quality(
            &code_smells,
            maintainability_score,
            readability_score,
            &test_coverage_assessment,
        );

        Ok(QualityAssessment {
            overall_quality,
            code_smells,
            maintainability_score,
            readability_score,
            test_coverage_assessment,
        })
    }

    async fn analyze_dependencies(
        &self,
        file_path: &str,
        code_content: &str,
        _ast_info: &serde_json::Value,
    ) -> Result<DependencyAnalysis> {
        let internal_dependencies = self.extract_internal_dependencies(file_path, code_content);
        let external_dependencies = self.extract_external_dependencies(code_content);
        let circular_dependencies = self.detect_circular_dependencies(file_path, &internal_dependencies);
        let dependency_health = self.assess_dependency_health(&internal_dependencies, &external_dependencies);

        Ok(DependencyAnalysis {
            internal_dependencies,
            external_dependencies,
            circular_dependencies,
            dependency_health,
        })
    }

    async fn generate_improvement_suggestions(
        &self,
        quality_assessment: &QualityAssessment,
        dependencies: &DependencyAnalysis,
    ) -> Result<Vec<ImprovementSuggestion>> {
        let mut suggestions = Vec::new();

        // Generate suggestions based on code smells
        for smell in &quality_assessment.code_smells {
            if matches!(smell.severity, SmellSeverity::Critical | SmellSeverity::Major) {
                suggestions.push(ImprovementSuggestion {
                    suggestion_type: ImprovementType::Refactoring,
                    description: format!("Address {}: {}", smell.smell_type, smell.suggested_refactor),
                    priority: match smell.severity {
                        SmellSeverity::Critical => Priority::Critical,
                        SmellSeverity::Major => Priority::High,
                        _ => Priority::Medium,
                    },
                    estimated_effort: self.estimate_refactor_effort(&smell.smell_type),
                    expected_benefit: format!("Improve code maintainability and reduce {}", smell.smell_type),
                });
            }
        }

        // Generate suggestions based on dependency issues
        if matches!(dependencies.dependency_health, DependencyHealth::Problematic) {
            suggestions.push(ImprovementSuggestion {
                suggestion_type: ImprovementType::Architecture,
                description: "Refactor dependency structure to reduce coupling".to_string(),
                priority: Priority::High,
                estimated_effort: EstimatedEffort::Large,
                expected_benefit: "Improve modularity and testability".to_string(),
            });
        }

        // Generate testing suggestions
        if !quality_assessment.test_coverage_assessment.has_tests {
            suggestions.push(ImprovementSuggestion {
                suggestion_type: ImprovementType::Testing,
                description: "Add unit tests to improve code reliability".to_string(),
                priority: Priority::High,
                estimated_effort: EstimatedEffort::Medium,
                expected_benefit: "Increase confidence in code changes and prevent regressions".to_string(),
            });
        }

        Ok(suggestions)
    }

    // Utility methods for cache management
    async fn get_cached_analysis(
        &self,
        file_path: &str,
        file_hash: &str,
    ) -> Option<CachedPurposeAnalysis> {
        let cache = self.pattern_cache.read().await;
        cache.get(file_path).and_then(|cached| {
            if cached.file_hash == file_hash {
                Some(cached.clone())
            } else {
                None
            }
        })
    }

    async fn cache_analysis(
        &self,
        file_path: &str,
        file_hash: &str,
        analysis: &CodePurposeAnalysis,
    ) -> Result<()> {
        let mut cache = self.pattern_cache.write().await;
        cache.insert(file_path.to_string(), CachedPurposeAnalysis {
            analysis: analysis.clone(),
            timestamp: chrono::Utc::now(),
            file_hash: file_hash.to_string(),
        });
        Ok(())
    }

    fn calculate_content_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    // Placeholder implementations for helper methods
    // These would be implemented with sophisticated pattern matching and analysis

    fn format_business_purpose(&self, purpose: &BusinessPurpose) -> String {
        match purpose {
            BusinessPurpose::CoreLogic { domain, operation, .. } => {
                format!("{} - {}", domain, operation)
            }
            BusinessPurpose::DataProcessing { input_type, output_type, .. } => {
                format!("Data processing: {} → {}", input_type, output_type)
            }
            BusinessPurpose::UserInterface { ui_type, .. } => {
                format!("User interface: {}", ui_type)
            }
            BusinessPurpose::Integration { integration_type, .. } => {
                format!("Integration: {}", integration_type)
            }
            BusinessPurpose::Infrastructure { infrastructure_type, .. } => {
                format!("Infrastructure: {}", infrastructure_type)
            }
            BusinessPurpose::Testing { test_type, .. } => {
                format!("Testing: {}", test_type)
            }
            BusinessPurpose::Configuration { config_type, .. } => {
                format!("Configuration: {}", config_type)
            }
        }
    }

    // Additional helper methods would be implemented here with sophisticated
    // pattern matching, regex analysis, and semantic understanding
    fn determine_test_type(&self, _content: &str) -> String { "Unit Test".to_string() }
    fn extract_code_under_test(&self, file_path: &str) -> String {
        file_path.replace("test_", "").replace("_test", "")
    }
    fn extract_test_scenarios(&self, _content: &str) -> Vec<String> { vec!["Happy path".to_string()] }
    fn determine_config_type(&self, _content: &str) -> String { "Application Configuration".to_string() }
    fn extract_config_settings(&self, _content: &str) -> Vec<String> { vec![] }
    fn determine_ui_type(&self, _content: &str) -> String { "Web Interface".to_string() }
    fn extract_user_actions(&self, _content: &str) -> Vec<String> { vec![] }
    fn extract_displayed_data(&self, _content: &str) -> Vec<String> { vec![] }
    fn extract_external_systems(&self, _content: &str) -> Vec<String> { vec![] }
    fn extract_data_exchanged(&self, _content: &str) -> Vec<String> { vec![] }
    fn is_infrastructure_code(&self, _file_path: &str, _content: &str) -> bool { false }
    fn determine_infrastructure_type(&self, _content: &str) -> String { "System Infrastructure".to_string() }
    fn extract_services_provided(&self, _content: &str) -> Vec<String> { vec![] }
    fn is_data_processing(&self, _content: &str) -> bool {
        _content.contains("transform") || _content.contains("process") || _content.contains("parse")
    }
    fn extract_input_type(&self, _content: &str) -> String { "Input Data".to_string() }
    fn extract_output_type(&self, _content: &str) -> String { "Output Data".to_string() }
    fn extract_transformation_logic(&self, _content: &str) -> String { "Data transformation".to_string() }
    fn extract_domain_from_path(&self, file_path: &str) -> String {
        file_path.split('/').nth(1).unwrap_or("Unknown").to_string()
    }
    fn extract_primary_operation(&self, _content: &str) -> String { "Business operation".to_string() }
    fn infer_business_value(&self, _content: &str) -> String { "Provides business functionality".to_string() }
    fn extract_validation_rules(&self, _content: &str) -> Option<Vec<BusinessRule>> { None }
    fn extract_authorization_rules(&self, _content: &str) -> Option<Vec<BusinessRule>> { None }
    fn extract_business_logic_rules(&self, _content: &str) -> Option<Vec<BusinessRule>> { None }
    fn extract_entity_concepts(&self, _content: &str) -> Vec<DomainConcept> { vec![] }
    fn extract_service_concepts(&self, _content: &str) -> Vec<DomainConcept> { vec![] }
    fn extract_process_concepts(&self, _content: &str) -> Vec<DomainConcept> { vec![] }
    fn has_business_logic_indicators(&self, _content: &str) -> bool { false }
    fn detect_code_smells(&self, _content: &str) -> Vec<CodeSmell> { vec![] }
    fn calculate_maintainability_score(&self, _content: &str) -> f32 { 0.8 }
    fn calculate_readability_score(&self, _content: &str) -> f32 { 0.8 }
    fn assess_test_coverage(&self, _content: &str) -> TestCoverageAssessment {
        TestCoverageAssessment {
            has_tests: false,
            test_types: vec![],
            coverage_gaps: vec![],
            test_quality: QualityLevel::Fair,
        }
    }
    fn determine_overall_quality(&self, _smells: &[CodeSmell], _maint: f32, _read: f32, _test: &TestCoverageAssessment) -> QualityLevel {
        QualityLevel::Good
    }
    fn extract_internal_dependencies(&self, _file_path: &str, _content: &str) -> Vec<InternalDependency> { vec![] }
    fn extract_external_dependencies(&self, _content: &str) -> Vec<ExternalDependency> { vec![] }
    fn detect_circular_dependencies(&self, _file_path: &str, _deps: &[InternalDependency]) -> Vec<String> { vec![] }
    fn assess_dependency_health(&self, _internal: &[InternalDependency], _external: &[ExternalDependency]) -> DependencyHealth {
        DependencyHealth::Healthy
    }
    fn estimate_refactor_effort(&self, _smell_type: &str) -> EstimatedEffort { EstimatedEffort::Small }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intelligence::ast_analysis::ASTAnalyzer;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_purpose_analysis_engine_creation() {
        let ast_analyzer = Arc::new(tokio::sync::RwLock::new(ASTAnalyzer::new().unwrap()));
        let engine = PurposeAnalysisEngine::new(ast_analyzer, None);

        // Basic functionality test
        let result = engine.get_business_context_summary(
            "src/test.rs",
            "fn test_example() { assert_eq!(1, 1); }"
        ).await;

        assert!(result.is_ok());
        let summary = result.unwrap();
        assert!(summary.contains("Purpose"));
    }

    #[tokio::test]
    async fn test_test_file_detection() {
        let ast_analyzer = Arc::new(tokio::sync::RwLock::new(ASTAnalyzer::new().unwrap()));
        let engine = PurposeAnalysisEngine::new(ast_analyzer, None);

        let analysis = engine.analyze_code_purpose(
            "src/test_user.rs",
            "#[test] fn test_user_creation() { }"
        ).await.unwrap();

        match analysis.primary_purpose {
            BusinessPurpose::Testing { .. } => {
                // Test correctly identified
            },
            _ => panic!("Should have identified as testing code"),
        }
    }
}