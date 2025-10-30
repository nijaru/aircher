/// Unified Intelligence Engine - Automatic middleware for transparent intelligence enhancement
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::debug;
use anyhow::Result;

use crate::intelligence::IntelligenceEngine;

/// Enhanced context for intelligent processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedContext {
    pub original_request: String,
    pub detected_intent: UserIntent,
    pub relevant_context: Vec<ContextItem>,
    pub intelligence_insights: Vec<IntelligenceInsight>,
    pub suggested_approach: String,
    pub confidence: f32,
}

/// User intent classification for automatic intelligence selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserIntent {
    CodeReading {
        files_mentioned: Vec<String>,
        analysis_depth: AnalysisDepth,
    },
    CodeWriting {
        target_files: Vec<String>,
        code_type: CodeType,
    },
    ProjectFixing {
        error_indicators: Vec<String>,
        urgency_level: UrgencyLevel,
    },
    ProjectExploration {
        scope: ExplorationScope,
    },
    Mixed {
        primary_intent: Box<UserIntent>,
        secondary_intents: Vec<UserIntent>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalysisDepth {
    Surface,     // Basic file overview
    Detailed,    // Function-level analysis
    Architectural, // System-wide relationships
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CodeType {
    NewFeature,
    BugFix,
    Refactoring,
    Test,
    Documentation,
    Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UrgencyLevel {
    Critical,    // Production down
    High,        // Blocking development
    Medium,      // Normal priority
    Low,         // Nice to have
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExplorationScope {
    SingleFile,
    Module,
    Architecture,
    FullProject,
}

/// Task analysis result with intelligence insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAnalysis {
    pub intent: UserIntent,
    pub complexity_score: f32,
    pub confidence_score: f32,
    pub has_multiple_solution_paths: bool,
    pub is_critical: bool,
    pub requires_codebase_search: bool,
}

/// Task complexity classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskComplexity {
    Simple,      // Single file, straightforward changes
    Moderate,    // Multiple files, some interdependencies
    Complex,     // Architecture changes, many dependencies
    Critical,    // System-wide impacts, high risk
}

/// Context item with relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    pub content: String,
    pub source: ContextSource,
    pub relevance_score: f32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextSource {
    FileAnalysis(String),
    PatternLearning(String),
    PreviousInteraction(String),
    SemanticSearch(String),
    ProjectMemory(String),
}

/// Intelligence insight for enhanced understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceInsight {
    pub insight_type: InsightType,
    pub description: String,
    pub confidence: f32,
    pub suggested_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightType {
    CodePurpose,
    ArchitecturalPattern,
    QualityIssue,
    PerformanceOpportunity,
    SecurityConcern,
    DebugGuidance,
    PatternApplication,
    ContextualExplanation,
}

/// Enhanced response with intelligence improvements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedResponse {
    pub original_response: String,
    pub intelligence_additions: Vec<IntelligenceAddition>,
    pub final_response: String,
    pub learning_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceAddition {
    pub addition_type: AdditionType,
    pub content: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdditionType {
    ContextualExplanation,
    QualityImprovement,
    AlternativeApproach,
    WarningOrCaution,
    FollowUpSuggestion,
    RelatedPattern,
}

/// Unified Intelligence Engine - Transparent middleware for all interactions
pub struct UnifiedIntelligenceEngine {
    // Core intelligence components (internal)
    base_intelligence: Arc<IntelligenceEngine>,

    // Cache for performance
    intent_cache: Arc<tokio::sync::RwLock<HashMap<String, UserIntent>>>,
    context_cache: Arc<tokio::sync::RwLock<HashMap<String, EnhancedContext>>>,

    // Learning state
    learning_enabled: bool,
}

impl UnifiedIntelligenceEngine {
    /// Create a new unified intelligence engine from an existing engine
    /// Takes Arc to allow sharing the same engine instance across components
    pub fn new(base_intelligence: Arc<IntelligenceEngine>) -> Self {
        Self {
            base_intelligence,
            intent_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            context_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            learning_enabled: true,
        }
    }

    /// Automatically enhance user request with intelligence
    pub async fn enhance_request_understanding(&self, user_input: &str) -> Result<EnhancedContext> {
        debug!("Automatically enhancing request understanding for: {}", user_input);

        // Check cache first
        {
            let cache = self.context_cache.read().await;
            if let Some(cached) = cache.get(user_input) {
                debug!("Using cached enhanced context");
                return Ok(cached.clone());
            }
        }

        // 1. Analyze user intent
        let intent = self.analyze_user_intent(user_input).await?;

        // 2. Gather relevant context based on intent
        let relevant_context = self.gather_relevant_context(user_input, &intent).await?;

        // 3. Generate intelligence insights
        let insights = self.generate_intelligence_insights(user_input, &intent, &relevant_context).await?;

        // 4. Suggest approach
        let suggested_approach = self.suggest_intelligent_approach(&intent, &insights).await?;

        // 5. Calculate confidence
        let confidence = self.calculate_enhancement_confidence(&intent, &relevant_context, &insights).await?;

        let enhanced_context = EnhancedContext {
            original_request: user_input.to_string(),
            detected_intent: intent,
            relevant_context,
            intelligence_insights: insights,
            suggested_approach,
            confidence,
        };

        // Cache the result
        {
            let mut cache = self.context_cache.write().await;
            cache.insert(user_input.to_string(), enhanced_context.clone());
        }

        Ok(enhanced_context)
    }

    /// Automatically improve response quality with intelligence
    pub async fn enhance_response_quality(&self, response: &str, original_input: &str, enhanced_context: &EnhancedContext) -> Result<EnhancedResponse> {
        debug!("Automatically enhancing response quality");

        // 1. Analyze response for improvement opportunities
        let improvement_opportunities = self.analyze_response_for_improvements(response, enhanced_context).await?;

        // 2. Generate intelligence additions
        let mut intelligence_additions = Vec::new();

        for opportunity in improvement_opportunities {
            match opportunity {
                ImprovementOpportunity::AddContext => {
                    if let Some(addition) = self.add_contextual_explanation(response, enhanced_context).await? {
                        intelligence_additions.push(addition);
                    }
                },
                ImprovementOpportunity::SuggestAlternatives => {
                    if let Some(addition) = self.suggest_alternative_approaches(response, enhanced_context).await? {
                        intelligence_additions.push(addition);
                    }
                },
                ImprovementOpportunity::AddQualityGuidance => {
                    if let Some(addition) = self.add_quality_improvements(response, enhanced_context).await? {
                        intelligence_additions.push(addition);
                    }
                },
                ImprovementOpportunity::AddWarnings => {
                    if let Some(addition) = self.add_warnings_and_cautions(response, enhanced_context).await? {
                        intelligence_additions.push(addition);
                    }
                },
            }
        }

        // 3. Construct enhanced response
        let final_response = self.construct_enhanced_response(response, &intelligence_additions).await?;

        // 4. Learn from this interaction
        let learning_notes = if self.learning_enabled {
            self.learn_from_interaction(original_input, response, enhanced_context).await?
        } else {
            Vec::new()
        };

        Ok(EnhancedResponse {
            original_response: response.to_string(),
            intelligence_additions,
            final_response,
            learning_notes,
        })
    }

    /// Generate intelligent system prompt enhancements
    pub async fn enhance_system_prompt(&self, base_prompt: &str, enhanced_context: &EnhancedContext) -> Result<String> {
        let mut enhanced_prompt = base_prompt.to_string();

        // Add intent-specific guidance
        enhanced_prompt.push_str(&self.generate_intent_specific_guidance(&enhanced_context.detected_intent).await?);

        // Add relevant context
        if !enhanced_context.relevant_context.is_empty() {
            enhanced_prompt.push_str("\n\n## Relevant Context from Intelligence:\n");
            for item in &enhanced_context.relevant_context {
                if item.relevance_score > 0.7 {
                    enhanced_prompt.push_str(&format!("- {} (confidence: {:.1}%)\n",
                        item.content, item.relevance_score * 100.0));
                }
            }
        }

        // Add intelligence insights
        if !enhanced_context.intelligence_insights.is_empty() {
            enhanced_prompt.push_str("\n\n## Intelligence Insights:\n");
            for insight in &enhanced_context.intelligence_insights {
                if insight.confidence > 0.6 {
                    enhanced_prompt.push_str(&format!("- {}: {}\n",
                        self.insight_type_to_string(&insight.insight_type),
                        insight.description));
                }
            }
        }

        // Add suggested approach
        if !enhanced_context.suggested_approach.is_empty() {
            enhanced_prompt.push_str(&format!("\n\n## Intelligent Approach Suggestion:\n{}\n",
                enhanced_context.suggested_approach));
        }

        Ok(enhanced_prompt)
    }

    // Internal implementation methods

    async fn analyze_user_intent(&self, input: &str) -> Result<UserIntent> {
        // Check cache first
        {
            let cache = self.intent_cache.read().await;
            if let Some(cached_intent) = cache.get(input) {
                return Ok(cached_intent.clone());
            }
        }

        let input_lower = input.to_lowercase();

        // Simple intent detection based on keywords and patterns
        let intent = if input_lower.contains("read") || input_lower.contains("understand") ||
                        input_lower.contains("what does") || input_lower.contains("explain") ||
                        input_lower.contains("analyze") {

            let files_mentioned = self.extract_file_mentions(input).await?;
            let analysis_depth = if input_lower.contains("architecture") || input_lower.contains("system") {
                AnalysisDepth::Architectural
            } else if input_lower.contains("function") || input_lower.contains("method") || input_lower.contains("detailed") {
                AnalysisDepth::Detailed
            } else {
                AnalysisDepth::Surface
            };

            UserIntent::CodeReading { files_mentioned, analysis_depth }

        } else if input_lower.contains("write") || input_lower.contains("create") ||
                  input_lower.contains("add") || input_lower.contains("implement") ||
                  input_lower.contains("generate") {

            let target_files = self.extract_file_mentions(input).await?;
            let code_type = if input_lower.contains("test") {
                CodeType::Test
            } else if input_lower.contains("fix") || input_lower.contains("bug") {
                CodeType::BugFix
            } else if input_lower.contains("refactor") {
                CodeType::Refactoring
            } else if input_lower.contains("config") {
                CodeType::Configuration
            } else if input_lower.contains("doc") {
                CodeType::Documentation
            } else {
                CodeType::NewFeature
            };

            UserIntent::CodeWriting { target_files, code_type }

        } else if input_lower.contains("error") || input_lower.contains("bug") ||
                  input_lower.contains("fix") || input_lower.contains("debug") ||
                  input_lower.contains("broken") || input_lower.contains("failing") {

            let error_indicators = self.extract_error_indicators(input).await?;
            let urgency_level = if input_lower.contains("critical") || input_lower.contains("production") {
                UrgencyLevel::Critical
            } else if input_lower.contains("urgent") || input_lower.contains("blocking") {
                UrgencyLevel::High
            } else if input_lower.contains("minor") || input_lower.contains("nice") {
                UrgencyLevel::Low
            } else {
                UrgencyLevel::Medium
            };

            UserIntent::ProjectFixing { error_indicators, urgency_level }

        } else {
            let scope = if input_lower.contains("project") || input_lower.contains("codebase") {
                ExplorationScope::FullProject
            } else if input_lower.contains("architecture") {
                ExplorationScope::Architecture
            } else if input_lower.contains("module") {
                ExplorationScope::Module
            } else {
                ExplorationScope::SingleFile
            };

            UserIntent::ProjectExploration { scope }
        };

        // Cache the result
        {
            let mut cache = self.intent_cache.write().await;
            cache.insert(input.to_string(), intent.clone());
        }

        Ok(intent)
    }

    async fn gather_relevant_context(&self, input: &str, intent: &UserIntent) -> Result<Vec<ContextItem>> {
        let mut context_items = Vec::new();

        match intent {
            UserIntent::CodeReading { files_mentioned, .. } => {
                // Get enhanced context for code reading
                if let Ok(enhanced_context) = self.base_intelligence.get_enhanced_development_context(input, None).await {
                    context_items.push(ContextItem {
                        content: enhanced_context,
                        source: ContextSource::FileAnalysis("enhanced_context".to_string()),
                        relevance_score: 0.9,
                        metadata: HashMap::new(),
                    });
                }

                // Add file-specific context for mentioned files
                for file in files_mentioned {
                    if let Ok(file_content) = tokio::fs::read_to_string(file).await {
                        let preview = file_content.lines().take(10).collect::<Vec<_>>().join("\n");
                        context_items.push(ContextItem {
                            content: format!("File preview: {}", preview),
                            source: ContextSource::FileAnalysis(file.clone()),
                            relevance_score: 0.8,
                            metadata: HashMap::new(),
                        });
                    }
                }
            },

            UserIntent::CodeWriting { .. } => {
                // Get pattern learning context
                if let Ok(patterns) = self.base_intelligence.get_pattern_summary().await {
                    context_items.push(ContextItem {
                        content: patterns,
                        source: ContextSource::PatternLearning("project_patterns".to_string()),
                        relevance_score: 0.85,
                        metadata: HashMap::new(),
                    });
                }
            },

            UserIntent::ProjectFixing { error_indicators, .. } => {
                // Get debugging intelligence context
                // COMMENTED OUT: get_quick_fix_recommendations method is temporarily disabled
                // for _error in error_indicators {
                //     if let Ok(recommendations) = self.base_intelligence.get_quick_fix_recommendations(error).await {
                //         context_items.push(ContextItem {
                //             content: format!("Quick fix recommendations: {:?}", recommendations),
                //             source: ContextSource::PreviousInteraction("debug_knowledge".to_string()),
                //             relevance_score: 0.9,
                //             metadata: HashMap::new(),
                //         });
                //     }
                // }
                let _ = error_indicators; // Suppress unused warning
            },

            _ => {
                // General context
                if let Ok(suggestions) = self.base_intelligence.get_suggestions(input, None).await {
                    if !suggestions.trim().is_empty() && suggestions != "Intelligence memory not initialized" {
                        context_items.push(ContextItem {
                            content: suggestions,
                            source: ContextSource::ProjectMemory("general_suggestions".to_string()),
                            relevance_score: 0.7,
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }

        Ok(context_items)
    }

    async fn generate_intelligence_insights(&self, input: &str, intent: &UserIntent, context: &[ContextItem]) -> Result<Vec<IntelligenceInsight>> {
        let mut insights = Vec::new();

        // Generate intent-specific insights
        match intent {
            UserIntent::CodeReading { analysis_depth, .. } => {
                insights.push(IntelligenceInsight {
                    insight_type: InsightType::CodePurpose,
                    description: format!("Analyzing code with {:?} depth for comprehensive understanding", analysis_depth),
                    confidence: 0.8,
                    suggested_actions: vec![
                        "Focus on business logic and architectural patterns".to_string(),
                        "Identify key abstractions and relationships".to_string(),
                    ],
                });
            },

            UserIntent::CodeWriting { code_type, .. } => {
                insights.push(IntelligenceInsight {
                    insight_type: InsightType::PatternApplication,
                    description: format!("Generating {:?} code using learned project patterns", code_type),
                    confidence: 0.85,
                    suggested_actions: vec![
                        "Apply consistent naming conventions".to_string(),
                        "Follow existing architectural patterns".to_string(),
                        "Include appropriate error handling".to_string(),
                    ],
                });
            },

            UserIntent::ProjectFixing { urgency_level, .. } => {
                insights.push(IntelligenceInsight {
                    insight_type: InsightType::DebugGuidance,
                    description: format!("Debugging with {:?} urgency - applying systematic analysis", urgency_level),
                    confidence: 0.9,
                    suggested_actions: vec![
                        "Analyze root cause before applying fixes".to_string(),
                        "Consider system-wide impact of changes".to_string(),
                        "Generate multiple fix strategies with risk assessment".to_string(),
                    ],
                });
            },

            _ => {}
        }

        // Add context-based insights
        for item in context {
            if item.relevance_score > 0.8 {
                insights.push(IntelligenceInsight {
                    insight_type: InsightType::ContextualExplanation,
                    description: format!("High-relevance context available from {:?}", item.source),
                    confidence: item.relevance_score,
                    suggested_actions: vec!["Use this context to enhance understanding".to_string()],
                });
            }
        }

        Ok(insights)
    }

    async fn suggest_intelligent_approach(&self, intent: &UserIntent, insights: &[IntelligenceInsight]) -> Result<String> {
        let base_suggestion = match intent {
            UserIntent::CodeReading { analysis_depth, .. } => {
                match analysis_depth {
                    AnalysisDepth::Surface => "Provide a high-level overview focusing on main purpose and key components",
                    AnalysisDepth::Detailed => "Analyze function-level details including implementation patterns and logic flow",
                    AnalysisDepth::Architectural => "Examine system-wide architecture, patterns, and component relationships",
                }
            },
            UserIntent::CodeWriting { code_type, .. } => {
                match code_type {
                    CodeType::NewFeature => "Design and implement following established patterns and architecture",
                    CodeType::BugFix => "Identify root cause and apply minimal, targeted fixes",
                    CodeType::Refactoring => "Improve structure while preserving functionality and existing interfaces",
                    CodeType::Test => "Create comprehensive tests covering edge cases and error conditions",
                    CodeType::Documentation => "Provide clear, practical documentation with examples",
                    CodeType::Configuration => "Configure following project conventions and security best practices",
                }
            },
            UserIntent::ProjectFixing { urgency_level, .. } => {
                match urgency_level {
                    UrgencyLevel::Critical => "Apply immediate hotfix while planning proper long-term solution",
                    UrgencyLevel::High => "Implement targeted fix with thorough testing before deployment",
                    UrgencyLevel::Medium => "Analyze thoroughly and implement comprehensive solution",
                    UrgencyLevel::Low => "Consider this as an opportunity for improvement and refactoring",
                }
            },
            UserIntent::ProjectExploration { scope } => {
                match scope {
                    ExplorationScope::SingleFile => "Focus on understanding this specific file's role and implementation",
                    ExplorationScope::Module => "Explain module structure, responsibilities, and interfaces",
                    ExplorationScope::Architecture => "Describe overall system design, patterns, and component interactions",
                    ExplorationScope::FullProject => "Provide comprehensive project overview including technologies, patterns, and structure",
                }
            },
            UserIntent::Mixed { primary_intent, .. } => {
                return Box::pin(self.suggest_intelligent_approach(primary_intent, insights)).await;
            }
        };

        // Enhance with insight-based suggestions
        let mut enhanced_approach = base_suggestion.to_string();

        let high_confidence_insights: Vec<_> = insights.iter()
            .filter(|i| i.confidence > 0.8)
            .collect();

        if !high_confidence_insights.is_empty() {
            enhanced_approach.push_str("\n\nIntelligence recommendations: ");
            for insight in high_confidence_insights {
                enhanced_approach.push_str(&format!("{} ", insight.description));
            }
        }

        Ok(enhanced_approach)
    }

    async fn calculate_enhancement_confidence(&self, intent: &UserIntent, context: &[ContextItem], insights: &[IntelligenceInsight]) -> Result<f32> {
        let mut confidence = 0.5_f32; // Base confidence

        // Intent clarity boosts confidence
        match intent {
            UserIntent::Mixed { .. } => confidence += 0.0, // Mixed intent is harder
            _ => confidence += 0.2, // Clear intent helps
        }

        // Context availability boosts confidence
        let high_relevance_context = context.iter().filter(|c| c.relevance_score > 0.7).count();
        confidence += (high_relevance_context as f32 * 0.1).min(0.3);

        // High-confidence insights boost overall confidence
        let high_confidence_insights = insights.iter().filter(|i| i.confidence > 0.8).count();
        confidence += (high_confidence_insights as f32 * 0.05).min(0.2);

        Ok(confidence.min(1.0))
    }


}

/// Response improvement opportunities
#[derive(Debug)]
enum ImprovementOpportunity {
    AddContext,
    SuggestAlternatives,
    AddQualityGuidance,
    AddWarnings,
}

impl UnifiedIntelligenceEngine {
    // Response enhancement methods

    async fn analyze_response_for_improvements(&self, response: &str, context: &EnhancedContext) -> Result<Vec<ImprovementOpportunity>> {
        let mut opportunities = Vec::new();

        // Check if response could benefit from additional context
        if response.len() < 200 && !context.relevant_context.is_empty() {
            opportunities.push(ImprovementOpportunity::AddContext);
        }

        // Check if we should suggest alternatives
        match &context.detected_intent {
            UserIntent::CodeWriting { .. } => {
                opportunities.push(ImprovementOpportunity::SuggestAlternatives);
            },
            UserIntent::ProjectFixing { urgency_level: UrgencyLevel::Critical, .. } => {
                opportunities.push(ImprovementOpportunity::AddWarnings);
            },
            _ => {}
        }

        // Always consider quality guidance for code-related responses
        if response.contains("```") || response.contains("function") || response.contains("class") {
            opportunities.push(ImprovementOpportunity::AddQualityGuidance);
        }

        Ok(opportunities)
    }

    async fn add_contextual_explanation(&self, response: &str, context: &EnhancedContext) -> Result<Option<IntelligenceAddition>> {
        if let Some(highest_relevance_item) = context.relevant_context.iter().max_by(|a, b|
            a.relevance_score.partial_cmp(&b.relevance_score).unwrap()) {

            if highest_relevance_item.relevance_score > 0.8 {
                return Ok(Some(IntelligenceAddition {
                    addition_type: AdditionType::ContextualExplanation,
                    content: format!("Additional context: {}", highest_relevance_item.content),
                    confidence: highest_relevance_item.relevance_score,
                }));
            }
        }
        Ok(None)
    }

    async fn suggest_alternative_approaches(&self, response: &str, context: &EnhancedContext) -> Result<Option<IntelligenceAddition>> {
        match &context.detected_intent {
            UserIntent::CodeWriting { code_type, .. } => {
                let alternative = match code_type {
                    CodeType::NewFeature => "Consider using a feature flag for gradual rollout",
                    CodeType::BugFix => "Alternative: Add comprehensive tests before fixing to prevent regression",
                    CodeType::Refactoring => "Alternative: Refactor incrementally to reduce risk",
                    _ => "Consider different implementation approaches for better maintainability",
                };

                Ok(Some(IntelligenceAddition {
                    addition_type: AdditionType::AlternativeApproach,
                    content: alternative.to_string(),
                    confidence: 0.7,
                }))
            },
            _ => Ok(None)
        }
    }

    async fn add_quality_improvements(&self, response: &str, _context: &EnhancedContext) -> Result<Option<IntelligenceAddition>> {
        if response.contains("```") {
            Ok(Some(IntelligenceAddition {
                addition_type: AdditionType::QualityImprovement,
                content: "Remember to include error handling, input validation, and documentation comments in production code".to_string(),
                confidence: 0.8,
            }))
        } else {
            Ok(None)
        }
    }

    async fn add_warnings_and_cautions(&self, response: &str, context: &EnhancedContext) -> Result<Option<IntelligenceAddition>> {
        match &context.detected_intent {
            UserIntent::ProjectFixing { urgency_level: UrgencyLevel::Critical, .. } => {
                Ok(Some(IntelligenceAddition {
                    addition_type: AdditionType::WarningOrCaution,
                    content: "⚠️ Critical fix: Test thoroughly in staging environment before production deployment. Consider rollback plan.".to_string(),
                    confidence: 0.9,
                }))
            },
            _ => Ok(None)
        }
    }

    async fn construct_enhanced_response(&self, original: &str, additions: &[IntelligenceAddition]) -> Result<String> {
        let mut enhanced = original.to_string();

        // Add intelligence additions at appropriate points
        for addition in additions {
            match addition.addition_type {
                AdditionType::ContextualExplanation => {
                    enhanced.push_str(&format!("\n\n**Context**: {}", addition.content));
                },
                AdditionType::AlternativeApproach => {
                    enhanced.push_str(&format!("\n\n**Alternative Approach**: {}", addition.content));
                },
                AdditionType::QualityImprovement => {
                    enhanced.push_str(&format!("\n\n**Quality Note**: {}", addition.content));
                },
                AdditionType::WarningOrCaution => {
                    enhanced.push_str(&format!("\n\n{}", addition.content));
                },
                AdditionType::FollowUpSuggestion => {
                    enhanced.push_str(&format!("\n\n**Next Steps**: {}", addition.content));
                },
                AdditionType::RelatedPattern => {
                    enhanced.push_str(&format!("\n\n**Related Pattern**: {}", addition.content));
                },
            }
        }

        Ok(enhanced)
    }

    async fn learn_from_interaction(&self, input: &str, response: &str, context: &EnhancedContext) -> Result<Vec<String>> {
        let mut learning_notes = Vec::new();

        // Learn from successful patterns
        if context.confidence > 0.8 {
            learning_notes.push(format!("High confidence interaction with intent: {:?}", context.detected_intent));
        }

        // Learn from context usage
        let high_relevance_context = context.relevant_context.iter()
            .filter(|c| c.relevance_score > 0.8)
            .count();

        if high_relevance_context > 0 {
            learning_notes.push(format!("Successfully used {} high-relevance context items", high_relevance_context));
        }

        // Note response characteristics for future improvement
        if response.len() > 500 {
            learning_notes.push("Generated detailed response - user may prefer comprehensive answers".to_string());
        }

        Ok(learning_notes)
    }

    // Utility methods

    async fn extract_file_mentions(&self, input: &str) -> Result<Vec<String>> {
        let mut files = Vec::new();

        // Simple pattern matching for file paths
        let patterns = [
            r"src/[a-zA-Z_/\.]+\.rs",
            r"[a-zA-Z_/\.]+\.rs",
            r"[a-zA-Z_/\.]+\.py",
            r"[a-zA-Z_/\.]+\.js",
            r"[a-zA-Z_/\.]+\.ts",
        ];

        for pattern in &patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                for mat in regex.find_iter(input) {
                    files.push(mat.as_str().to_string());
                }
            }
        }

        Ok(files)
    }

    async fn extract_error_indicators(&self, input: &str) -> Result<Vec<String>> {
        let mut indicators = Vec::new();

        let error_patterns = [
            "error[E",
            "panic",
            "exception",
            "failed",
            "broken",
            "not working",
            "doesn't work",
        ];

        for pattern in &error_patterns {
            if input.to_lowercase().contains(pattern) {
                indicators.push(pattern.to_string());
            }
        }

        Ok(indicators)
    }

    async fn generate_intent_specific_guidance(&self, intent: &UserIntent) -> Result<String> {
        let guidance = match intent {
            UserIntent::CodeReading { .. } => {
                "\n\nFocus on explaining the code's purpose, architecture, and key design decisions. Provide context about how this code fits into the larger system."
            },
            UserIntent::CodeWriting { .. } => {
                "\n\nGenerate code that follows project conventions and patterns. Consider error handling, testing, and documentation. Ensure the code integrates well with existing systems."
            },
            UserIntent::ProjectFixing { .. } => {
                "\n\nAnalyze the problem systematically. Consider root causes, not just symptoms. Provide multiple solution approaches with trade-offs. Think about testing and validation."
            },
            UserIntent::ProjectExploration { .. } => {
                "\n\nProvide a clear overview that helps the user understand the project structure, key components, and how they work together."
            },
            UserIntent::Mixed { .. } => {
                "\n\nAddress all aspects of the request systematically. Start with the most important part and provide comprehensive guidance."
            }
        };

        Ok(guidance.to_string())
    }

    fn insight_type_to_string(&self, insight_type: &InsightType) -> &'static str {
        match insight_type {
            InsightType::CodePurpose => "Code Purpose",
            InsightType::ArchitecturalPattern => "Architecture",
            InsightType::QualityIssue => "Quality",
            InsightType::PerformanceOpportunity => "Performance",
            InsightType::SecurityConcern => "Security",
            InsightType::DebugGuidance => "Debug Guidance",
            InsightType::PatternApplication => "Pattern Application",
            InsightType::ContextualExplanation => "Contextual Explanation",
        }
    }
}