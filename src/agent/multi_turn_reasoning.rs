/// Multi-Turn Reasoning Engine - Real systematic problem solving for coding tasks
///
/// This replaces theatrical intelligence with actual reasoning that:
/// 1. Plans systematically before acting
/// 2. Explores codebases methodically
/// 3. Learns from failures and iterates
/// 4. Validates changes through testing
///
/// Designed for SWE-bench performance, not just demo capabilities.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::agent::tools::ToolRegistry;
use crate::providers::LLMProvider;
use crate::agent::intelligent_strategy_selection::IntelligentStrategySelector;
use crate::intelligence::{IntelligenceEngine, ContextItem};

/// Represents a systematic plan for solving a coding problem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningPlan {
    pub task_id: String,
    pub objective: String,
    pub phases: Vec<ReasoningPhase>,
    pub current_phase: usize,
    pub state: PlanState,
    pub learned_context: HashMap<String, String>,
    pub failed_attempts: Vec<FailedAttempt>,
    pub max_iterations: usize,
    pub current_iteration: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningPhase {
    pub name: String,
    pub description: String,
    pub actions: Vec<PlannedAction>,
    pub success_criteria: Vec<String>,
    pub completed: bool,
    pub results: Option<PhaseResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedAction {
    pub action_type: ActionType,
    pub description: String,
    pub tool: String,
    pub parameters: serde_json::Value,
    pub expected_outcome: String,
    pub retry_count: usize,
    pub max_retries: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ActionType {
    Explore,    // Understand codebase structure
    Analyze,    // Deep dive into specific code
    Test,       // Run tests to understand current state
    Implement,  // Make actual changes
    Validate,   // Check if changes work
    Debug,      // Fix issues found during validation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanState {
    Planning,
    Executing,
    Validating,
    Complete,
    Failed,
    Iterating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseResult {
    pub success: bool,
    pub findings: Vec<String>,
    pub artifacts: HashMap<String, String>, // file_path -> content
    pub next_actions: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedAttempt {
    pub phase: String,
    pub action: String,
    pub error: String,
    pub learning: String, // What we learned from this failure
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Multi-turn reasoning engine that actually solves coding problems systematically
pub struct MultiTurnReasoningEngine {
    tools: Arc<ToolRegistry>,
    active_plans: HashMap<String, ReasoningPlan>,
    execution_queue: VecDeque<(String, usize)>, // (plan_id, action_index)
    memory: ReasoningMemory,
    intelligent_selector: IntelligentStrategySelector,
}

#[derive(Debug, Clone)]
pub struct ReasoningMemory {
    pub codebase_structure: HashMap<String, CodebaseInfo>,
    pub test_results: HashMap<String, TestResult>,
    pub successful_patterns: Vec<SuccessfulPattern>,
    pub failure_patterns: Vec<FailurePattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseInfo {
    pub files: Vec<String>,
    pub dependencies: HashMap<String, Vec<String>>,
    pub test_files: Vec<String>,
    pub entry_points: Vec<String>,
    pub language: String,
    pub framework: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub parsed_failures: Vec<TestFailure>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFailure {
    pub test_name: String,
    pub failure_type: String,
    pub error_message: String,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessfulPattern {
    pub problem_type: String,
    pub approach: String,
    pub key_actions: Vec<String>,
    pub success_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    pub problem_type: String,
    pub failed_approach: String,
    pub error_indicators: Vec<String>,
    pub lessons_learned: Vec<String>,
}

impl MultiTurnReasoningEngine {
    pub fn new(tools: Arc<ToolRegistry>, intelligence: Arc<IntelligenceEngine>) -> Result<Self> {
        let intelligent_selector = IntelligentStrategySelector::new(intelligence)
            .unwrap_or_else(|e| {
                warn!("Failed to create intelligent strategy selector: {}", e);
                panic!("Failed to create intelligent strategy selector: {}", e);
            });

        Ok(Self {
            tools,
            active_plans: HashMap::new(),
            execution_queue: VecDeque::new(),
            memory: ReasoningMemory {
                codebase_structure: HashMap::new(),
                test_results: HashMap::new(),
                successful_patterns: Vec::new(),
                failure_patterns: Vec::new(),
            },
            intelligent_selector,
        })
    }

    /// Create a systematic plan for solving a coding problem
    pub async fn create_reasoning_plan(
        &mut self,
        objective: &str,
        _provider: &dyn LLMProvider,
        _model: &str
    ) -> Result<String> {
        info!("Creating intelligent reasoning plan for: {}", objective);

        let task_id = Uuid::new_v4().to_string();

        // Gather context for intelligence analysis
        let context = self.gather_context_for_task(objective).await?;

        // Use intelligent strategy selection
        let adapted_strategy = self.intelligent_selector
            .select_intelligent_strategy(objective, &context)
            .await?;

        info!(
            "Selected strategy: {} with adaptations: confidence={:.2}, depth={}, reflection={}, tree_search={}",
            adapted_strategy.base_strategy.name,
            adapted_strategy.confidence_threshold,
            adapted_strategy.max_exploration_depth,
            adapted_strategy.should_use_reflection,
            adapted_strategy.should_use_tree_search
        );

        // Generate strategy-specific phases based on the selected strategy
        let mut phases = self.create_strategy_specific_phases(
            &adapted_strategy.base_strategy.name,
            objective,
            &adapted_strategy
        )?;

        // Apply intelligence adaptations to phases
        self.apply_intelligence_adaptations(&mut phases, &adapted_strategy);

        // For exploration phase, add specific actions based on objective
        if let Some(exploration) = phases.iter_mut().find(|p| p.name.contains("Exploration") || p.name.contains("Research")) {
            exploration.actions = self.plan_exploration_actions(objective);
        }

        let plan = ReasoningPlan {
            task_id: task_id.clone(),
            objective: objective.to_string(),
            phases,
            current_phase: 0,
            state: PlanState::Planning,
            learned_context: HashMap::new(),
            failed_attempts: Vec::new(),
            max_iterations: adapted_strategy.base_strategy.max_iterations.max(3),
            current_iteration: 0,
        };

        self.active_plans.insert(task_id.clone(), plan);

        // Queue up the first phase for execution
        self.queue_phase_execution(&task_id, 0)?;

        Ok(task_id)
    }

    /// Gather context for task analysis
    async fn gather_context_for_task(&self, objective: &str) -> Result<Vec<ContextItem>> {
        let mut context = Vec::new();

        // Add project context from memory
        for (path, info) in &self.memory.codebase_structure {
            context.push(ContextItem {
                content: format!("File: {}, Language: {}", path, info.language),
                source: crate::intelligence::ContextSource::FileAnalysis(path.clone()),
                relevance_score: 0.5, // Base relevance
                metadata: HashMap::new(),
            });
        }

        // Add test results context
        for (test, result) in &self.memory.test_results {
            context.push(ContextItem {
                content: format!("Test: {}, Exit Code: {}", test, result.exit_code),
                source: crate::intelligence::ContextSource::PreviousInteraction(format!("test:{}", test)),
                relevance_score: if result.exit_code != 0 { 0.9 } else { 0.3 }, // Failed tests more relevant
                metadata: HashMap::new(),
            });
        }

        // Add successful patterns
        for pattern in &self.memory.successful_patterns {
            context.push(ContextItem {
                content: format!("Successful Pattern: {} - {}", pattern.problem_type, pattern.approach),
                source: crate::intelligence::ContextSource::PatternLearning("successful_patterns".to_string()),
                relevance_score: 0.7,
                metadata: HashMap::new(),
            });
        }

        Ok(context)
    }

    /// Create strategy-specific phases based on the selected strategy type
    fn create_strategy_specific_phases(
        &self,
        strategy_name: &str,
        objective: &str,
        adapted_strategy: &crate::agent::intelligent_strategy_selection::AdaptedStrategy,
    ) -> Result<Vec<ReasoningPhase>> {
        match strategy_name.to_lowercase().as_str() {
            "react" => self.create_react_phases(objective, adapted_strategy),
            "reflexion" => self.create_reflexion_phases(objective, adapted_strategy),
            "tree_of_thoughts" | "tree of thoughts" => self.create_tree_of_thoughts_phases(objective, adapted_strategy),
            "interactive_planning" | "interactive planning" => self.create_interactive_planning_phases(objective, adapted_strategy),
            "swe_bench_strategy" | "swe-bench strategy" => self.create_swe_bench_phases(objective, adapted_strategy),
            "workflow_orchestration" | "workflow orchestration" => self.create_workflow_phases(objective, adapted_strategy),
            _ => {
                warn!("Unknown strategy '{}', falling back to default phases", strategy_name);
                self.create_default_phases(objective)
            }
        }
    }

    /// Create ReAct strategy phases (Think-Act-Observe loop)
    fn create_react_phases(&self, objective: &str, adapted_strategy: &crate::agent::intelligent_strategy_selection::AdaptedStrategy) -> Result<Vec<ReasoningPhase>> {
        let mut phases = Vec::new();

        // ReAct uses iterative Think-Act-Observe cycles
        let iterations = adapted_strategy.max_exploration_depth.min(5);

        for i in 0..iterations {
            // Think phase
            phases.push(ReasoningPhase {
                name: format!("Think_{}", i + 1),
                description: format!("Reason about the current state and plan next action (iteration {})", i + 1),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Analyze,
                        description: "Analyze current context and determine next step".to_string(),
                        tool: "reflect".to_string(),
                        parameters: serde_json::json!({"iteration": i + 1}),
                        expected_outcome: "Clear reasoning about next action".to_string(),
                        retry_count: 0,
                        max_retries: 1,
                    }
                ],
                success_criteria: vec![
                    "Identified what needs to be done next".to_string(),
                    "Generated hypothesis or plan".to_string(),
                ],
                completed: false,
                results: None,
            });

            // Act phase
            phases.push(ReasoningPhase {
                name: format!("Act_{}", i + 1),
                description: format!("Execute the planned action (iteration {})", i + 1),
                actions: vec![
                    PlannedAction {
                        action_type: if i == 0 { ActionType::Explore } else { ActionType::Implement },
                        description: "Execute planned action based on thinking".to_string(),
                        tool: if i == 0 { "search_code".to_string() } else { "edit_file".to_string() },
                        parameters: serde_json::json!({}),
                        expected_outcome: "Action executed successfully".to_string(),
                        retry_count: 0,
                        max_retries: 2,
                    }
                ],
                success_criteria: vec![
                    "Action completed".to_string(),
                    "Results available for observation".to_string(),
                ],
                completed: false,
                results: None,
            });

            // Observe phase
            phases.push(ReasoningPhase {
                name: format!("Observe_{}", i + 1),
                description: format!("Observe results and update understanding (iteration {})", i + 1),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Validate,
                        description: "Observe and validate action results".to_string(),
                        tool: "run_command".to_string(),
                        parameters: serde_json::json!({"command": "echo 'Observing results'"}),
                        expected_outcome: "Results understood and integrated".to_string(),
                        retry_count: 0,
                        max_retries: 1,
                    }
                ],
                success_criteria: vec![
                    "Results observed and understood".to_string(),
                    "Learning extracted from observation".to_string(),
                ],
                completed: false,
                results: None,
            });
        }

        Ok(phases)
    }

    /// Create Reflexion strategy phases (with explicit reflection)
    fn create_reflexion_phases(&self, objective: &str, _adapted_strategy: &crate::agent::intelligent_strategy_selection::AdaptedStrategy) -> Result<Vec<ReasoningPhase>> {
        Ok(vec![
            // Initial attempt phase
            ReasoningPhase {
                name: "Initial_Attempt".to_string(),
                description: "Make initial attempt to solve the problem".to_string(),
                actions: self.plan_exploration_actions(objective),
                success_criteria: vec![
                    "Initial approach attempted".to_string(),
                    "Results available for reflection".to_string(),
                ],
                completed: false,
                results: None,
            },
            // Reflection phase
            ReasoningPhase {
                name: "Reflection".to_string(),
                description: "Reflect on the attempt and identify improvements".to_string(),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Analyze,
                        description: "Analyze what worked and what didn't".to_string(),
                        tool: "reflect".to_string(),
                        parameters: serde_json::json!({"deep_analysis": true}),
                        expected_outcome: "Clear understanding of successes and failures".to_string(),
                        retry_count: 0,
                        max_retries: 1,
                    },
                    PlannedAction {
                        action_type: ActionType::Debug,
                        description: "Identify root causes of any issues".to_string(),
                        tool: "analyze_errors".to_string(),
                        parameters: serde_json::json!({}),
                        expected_outcome: "Root causes identified".to_string(),
                        retry_count: 0,
                        max_retries: 2,
                    }
                ],
                success_criteria: vec![
                    "Reflection complete".to_string(),
                    "Improvements identified".to_string(),
                    "New approach formulated".to_string(),
                ],
                completed: false,
                results: None,
            },
            // Refined attempt phase
            ReasoningPhase {
                name: "Refined_Attempt".to_string(),
                description: "Apply learnings from reflection to solve the problem".to_string(),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Implement,
                        description: "Implement solution with reflection insights".to_string(),
                        tool: "edit_file".to_string(),
                        parameters: serde_json::json!({}),
                        expected_outcome: "Improved solution implemented".to_string(),
                        retry_count: 0,
                        max_retries: 2,
                    }
                ],
                success_criteria: vec![
                    "Solution implemented with improvements".to_string(),
                    "Previous issues addressed".to_string(),
                ],
                completed: false,
                results: None,
            },
            // Validation phase
            ReasoningPhase {
                name: "Validation".to_string(),
                description: "Validate the refined solution".to_string(),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Test,
                        description: "Run tests to validate solution".to_string(),
                        tool: "run_command".to_string(),
                        parameters: serde_json::json!({"command": "cargo test"}),
                        expected_outcome: "Tests pass".to_string(),
                        retry_count: 0,
                        max_retries: 2,
                    }
                ],
                success_criteria: vec![
                    "Solution validated".to_string(),
                    "All tests pass".to_string(),
                ],
                completed: false,
                results: None,
            }
        ])
    }

    /// Create Tree of Thoughts phases (exploring multiple solution paths)
    fn create_tree_of_thoughts_phases(&self, objective: &str, adapted_strategy: &crate::agent::intelligent_strategy_selection::AdaptedStrategy) -> Result<Vec<ReasoningPhase>> {
        let beam_width = 3; // Number of parallel paths to explore
        let mut phases = Vec::new();

        // Initial exploration phase
        phases.push(ReasoningPhase {
            name: "Generate_Solution_Paths".to_string(),
            description: "Generate multiple possible solution approaches".to_string(),
            actions: vec![
                PlannedAction {
                    action_type: ActionType::Analyze,
                    description: format!("Generate {} different approaches", beam_width),
                    tool: "brainstorm".to_string(),
                    parameters: serde_json::json!({"count": beam_width}),
                    expected_outcome: "Multiple solution paths identified".to_string(),
                    retry_count: 0,
                    max_retries: 1,
                }
            ],
            success_criteria: vec![
                format!("At least {} solution paths generated", beam_width),
                "Each path is distinct and viable".to_string(),
            ],
            completed: false,
            results: None,
        });

        // Explore each path in parallel
        for i in 0..beam_width {
            phases.push(ReasoningPhase {
                name: format!("Explore_Path_{}", i + 1),
                description: format!("Explore solution path {} in detail", i + 1),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Explore,
                        description: format!("Deep dive into approach {}", i + 1),
                        tool: "search_code".to_string(),
                        parameters: serde_json::json!({"path_id": i + 1}),
                        expected_outcome: format!("Path {} fully explored", i + 1),
                        retry_count: 0,
                        max_retries: 2,
                    },
                    PlannedAction {
                        action_type: ActionType::Analyze,
                        description: format!("Evaluate viability of path {}", i + 1),
                        tool: "evaluate".to_string(),
                        parameters: serde_json::json!({"path_id": i + 1}),
                        expected_outcome: "Path viability assessed".to_string(),
                        retry_count: 0,
                        max_retries: 1,
                    }
                ],
                success_criteria: vec![
                    format!("Path {} explored", i + 1),
                    "Pros and cons identified".to_string(),
                    "Feasibility assessed".to_string(),
                ],
                completed: false,
                results: None,
            });
        }

        // Selection phase
        phases.push(ReasoningPhase {
            name: "Select_Best_Path".to_string(),
            description: "Compare paths and select the most promising one".to_string(),
            actions: vec![
                PlannedAction {
                    action_type: ActionType::Analyze,
                    description: "Compare all explored paths".to_string(),
                    tool: "compare_solutions".to_string(),
                    parameters: serde_json::json!({"paths": beam_width}),
                    expected_outcome: "Best path selected".to_string(),
                    retry_count: 0,
                    max_retries: 1,
                }
            ],
            success_criteria: vec![
                "All paths compared".to_string(),
                "Best path selected with justification".to_string(),
            ],
            completed: false,
            results: None,
        });

        // Implementation phase
        phases.push(ReasoningPhase {
            name: "Implement_Selected_Path".to_string(),
            description: "Implement the selected solution path".to_string(),
            actions: vec![
                PlannedAction {
                    action_type: ActionType::Implement,
                    description: "Implement the chosen solution".to_string(),
                    tool: "edit_file".to_string(),
                    parameters: serde_json::json!({}),
                    expected_outcome: "Solution implemented".to_string(),
                    retry_count: 0,
                    max_retries: 3,
                }
            ],
            success_criteria: vec![
                "Solution fully implemented".to_string(),
                "Code compiles/runs".to_string(),
            ],
            completed: false,
            results: None,
        });

        Ok(phases)
    }

    /// Create Interactive Planning phases (with checkpoints)
    fn create_interactive_planning_phases(&self, objective: &str, _adapted_strategy: &crate::agent::intelligent_strategy_selection::AdaptedStrategy) -> Result<Vec<ReasoningPhase>> {
        Ok(vec![
            // Planning phase
            ReasoningPhase {
                name: "Create_Plan".to_string(),
                description: "Create detailed plan with user checkpoints".to_string(),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Analyze,
                        description: "Analyze requirements and create plan".to_string(),
                        tool: "plan".to_string(),
                        parameters: serde_json::json!({"interactive": true}),
                        expected_outcome: "Detailed plan created".to_string(),
                        retry_count: 0,
                        max_retries: 1,
                    }
                ],
                success_criteria: vec![
                    "Plan created with clear steps".to_string(),
                    "Checkpoints identified".to_string(),
                ],
                completed: false,
                results: None,
            },
            // First implementation checkpoint
            ReasoningPhase {
                name: "Implementation_Phase_1".to_string(),
                description: "Implement first part of the plan".to_string(),
                actions: self.plan_exploration_actions(objective),
                success_criteria: vec![
                    "First phase implemented".to_string(),
                    "Ready for review".to_string(),
                ],
                completed: false,
                results: None,
            },
            // Review checkpoint
            ReasoningPhase {
                name: "Review_Checkpoint_1".to_string(),
                description: "Review progress and adjust plan if needed".to_string(),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Validate,
                        description: "Review implementation so far".to_string(),
                        tool: "review".to_string(),
                        parameters: serde_json::json!({"checkpoint": 1}),
                        expected_outcome: "Progress reviewed".to_string(),
                        retry_count: 0,
                        max_retries: 1,
                    }
                ],
                success_criteria: vec![
                    "Progress reviewed".to_string(),
                    "Adjustments identified if needed".to_string(),
                ],
                completed: false,
                results: None,
            },
            // Second implementation phase
            ReasoningPhase {
                name: "Implementation_Phase_2".to_string(),
                description: "Complete implementation based on feedback".to_string(),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Implement,
                        description: "Complete remaining implementation".to_string(),
                        tool: "edit_file".to_string(),
                        parameters: serde_json::json!({}),
                        expected_outcome: "Implementation complete".to_string(),
                        retry_count: 0,
                        max_retries: 2,
                    }
                ],
                success_criteria: vec![
                    "Implementation complete".to_string(),
                    "All requirements met".to_string(),
                ],
                completed: false,
                results: None,
            },
            // Final validation
            ReasoningPhase {
                name: "Final_Validation".to_string(),
                description: "Validate complete implementation".to_string(),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Test,
                        description: "Run comprehensive tests".to_string(),
                        tool: "run_command".to_string(),
                        parameters: serde_json::json!({"command": "cargo test"}),
                        expected_outcome: "All tests pass".to_string(),
                        retry_count: 0,
                        max_retries: 2,
                    }
                ],
                success_criteria: vec![
                    "All tests pass".to_string(),
                    "Solution validated".to_string(),
                ],
                completed: false,
                results: None,
            }
        ])
    }

    /// Create SWE-bench strategy phases (systematic bug fixing)
    fn create_swe_bench_phases(&self, objective: &str, _adapted_strategy: &crate::agent::intelligent_strategy_selection::AdaptedStrategy) -> Result<Vec<ReasoningPhase>> {
        Ok(vec![
            // Bug reproduction phase
            ReasoningPhase {
                name: "Reproduce_Bug".to_string(),
                description: "Reproduce the reported bug or issue".to_string(),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Test,
                        description: "Run existing tests to see failures".to_string(),
                        tool: "run_command".to_string(),
                        parameters: serde_json::json!({"command": "cargo test"}),
                        expected_outcome: "Bug reproduced".to_string(),
                        retry_count: 0,
                        max_retries: 2,
                    },
                    PlannedAction {
                        action_type: ActionType::Analyze,
                        description: "Analyze test output and error messages".to_string(),
                        tool: "analyze_errors".to_string(),
                        parameters: serde_json::json!({}),
                        expected_outcome: "Error patterns identified".to_string(),
                        retry_count: 0,
                        max_retries: 1,
                    }
                ],
                success_criteria: vec![
                    "Bug successfully reproduced".to_string(),
                    "Failure conditions understood".to_string(),
                ],
                completed: false,
                results: None,
            },
            // Localization phase
            ReasoningPhase {
                name: "Localize_Bug".to_string(),
                description: "Locate the source of the bug in the codebase".to_string(),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Explore,
                        description: "Search for error-related code".to_string(),
                        tool: "search_code".to_string(),
                        parameters: serde_json::json!({"query": "error"}),
                        expected_outcome: "Relevant code found".to_string(),
                        retry_count: 0,
                        max_retries: 2,
                    },
                    PlannedAction {
                        action_type: ActionType::Analyze,
                        description: "Trace error through call stack".to_string(),
                        tool: "trace_error".to_string(),
                        parameters: serde_json::json!({}),
                        expected_outcome: "Bug location identified".to_string(),
                        retry_count: 0,
                        max_retries: 2,
                    },
                    PlannedAction {
                        action_type: ActionType::Debug,
                        description: "Analyze root cause".to_string(),
                        tool: "debug_analyze".to_string(),
                        parameters: serde_json::json!({}),
                        expected_outcome: "Root cause understood".to_string(),
                        retry_count: 0,
                        max_retries: 1,
                    }
                ],
                success_criteria: vec![
                    "Bug location identified".to_string(),
                    "Root cause understood".to_string(),
                ],
                completed: false,
                results: None,
            },
            // Fix generation phase
            ReasoningPhase {
                name: "Generate_Fix".to_string(),
                description: "Generate and apply fix for the bug".to_string(),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Implement,
                        description: "Implement bug fix".to_string(),
                        tool: "edit_file".to_string(),
                        parameters: serde_json::json!({}),
                        expected_outcome: "Fix implemented".to_string(),
                        retry_count: 0,
                        max_retries: 3,
                    },
                    PlannedAction {
                        action_type: ActionType::Test,
                        description: "Test the fix locally".to_string(),
                        tool: "run_command".to_string(),
                        parameters: serde_json::json!({"command": "cargo test"}),
                        expected_outcome: "Local tests pass".to_string(),
                        retry_count: 0,
                        max_retries: 2,
                    }
                ],
                success_criteria: vec![
                    "Fix implemented".to_string(),
                    "Bug no longer reproduces".to_string(),
                ],
                completed: false,
                results: None,
            },
            // Validation phase
            ReasoningPhase {
                name: "Validate_Fix".to_string(),
                description: "Validate fix doesn't break other functionality".to_string(),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Test,
                        description: "Run full test suite".to_string(),
                        tool: "run_command".to_string(),
                        parameters: serde_json::json!({"command": "cargo test --all"}),
                        expected_outcome: "All tests pass".to_string(),
                        retry_count: 0,
                        max_retries: 2,
                    },
                    PlannedAction {
                        action_type: ActionType::Validate,
                        description: "Check for regressions".to_string(),
                        tool: "check_regressions".to_string(),
                        parameters: serde_json::json!({}),
                        expected_outcome: "No regressions found".to_string(),
                        retry_count: 0,
                        max_retries: 1,
                    }
                ],
                success_criteria: vec![
                    "All tests pass".to_string(),
                    "No regressions introduced".to_string(),
                ],
                completed: false,
                results: None,
            }
        ])
    }

    /// Create Workflow Orchestration phases
    fn create_workflow_phases(&self, objective: &str, _adapted_strategy: &crate::agent::intelligent_strategy_selection::AdaptedStrategy) -> Result<Vec<ReasoningPhase>> {
        Ok(vec![
            // Setup phase
            ReasoningPhase {
                name: "Setup_Workflow".to_string(),
                description: "Set up workflow dependencies and environment".to_string(),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Explore,
                        description: "Check environment and dependencies".to_string(),
                        tool: "check_environment".to_string(),
                        parameters: serde_json::json!({}),
                        expected_outcome: "Environment ready".to_string(),
                        retry_count: 0,
                        max_retries: 2,
                    }
                ],
                success_criteria: vec![
                    "Environment configured".to_string(),
                    "Dependencies available".to_string(),
                ],
                completed: false,
                results: None,
            },
            // Parallel execution phase
            ReasoningPhase {
                name: "Parallel_Tasks".to_string(),
                description: "Execute independent tasks in parallel".to_string(),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Implement,
                        description: "Execute task 1".to_string(),
                        tool: "execute_task".to_string(),
                        parameters: serde_json::json!({"task_id": 1, "parallel": true}),
                        expected_outcome: "Task 1 complete".to_string(),
                        retry_count: 0,
                        max_retries: 2,
                    },
                    PlannedAction {
                        action_type: ActionType::Implement,
                        description: "Execute task 2".to_string(),
                        tool: "execute_task".to_string(),
                        parameters: serde_json::json!({"task_id": 2, "parallel": true}),
                        expected_outcome: "Task 2 complete".to_string(),
                        retry_count: 0,
                        max_retries: 2,
                    },
                    PlannedAction {
                        action_type: ActionType::Implement,
                        description: "Execute task 3".to_string(),
                        tool: "execute_task".to_string(),
                        parameters: serde_json::json!({"task_id": 3, "parallel": true}),
                        expected_outcome: "Task 3 complete".to_string(),
                        retry_count: 0,
                        max_retries: 2,
                    }
                ],
                success_criteria: vec![
                    "All parallel tasks complete".to_string(),
                    "Results collected".to_string(),
                ],
                completed: false,
                results: None,
            },
            // Integration phase
            ReasoningPhase {
                name: "Integrate_Results".to_string(),
                description: "Integrate results from parallel tasks".to_string(),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Implement,
                        description: "Merge task results".to_string(),
                        tool: "merge_results".to_string(),
                        parameters: serde_json::json!({}),
                        expected_outcome: "Results integrated".to_string(),
                        retry_count: 0,
                        max_retries: 2,
                    }
                ],
                success_criteria: vec![
                    "All results integrated".to_string(),
                    "Final output generated".to_string(),
                ],
                completed: false,
                results: None,
            },
            // Verification phase
            ReasoningPhase {
                name: "Verify_Workflow".to_string(),
                description: "Verify complete workflow execution".to_string(),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Validate,
                        description: "Verify workflow output".to_string(),
                        tool: "verify_output".to_string(),
                        parameters: serde_json::json!({}),
                        expected_outcome: "Output verified".to_string(),
                        retry_count: 0,
                        max_retries: 1,
                    }
                ],
                success_criteria: vec![
                    "Workflow complete".to_string(),
                    "Output meets requirements".to_string(),
                ],
                completed: false,
                results: None,
            }
        ])
    }

    /// Create default phases for unknown strategies
    fn create_default_phases(&self, objective: &str) -> Result<Vec<ReasoningPhase>> {
        Ok(vec![
            ReasoningPhase {
                name: "Exploration".to_string(),
                description: "Explore and understand the problem space".to_string(),
                actions: self.plan_exploration_actions(objective),
                success_criteria: vec![
                    "Problem space understood".to_string(),
                    "Key files and patterns identified".to_string(),
                ],
                completed: false,
                results: None,
            },
            ReasoningPhase {
                name: "Analysis".to_string(),
                description: "Analyze findings and plan approach".to_string(),
                actions: vec![],
                success_criteria: vec![
                    "Root cause identified".to_string(),
                    "Solution approach planned".to_string(),
                ],
                completed: false,
                results: None,
            },
            ReasoningPhase {
                name: "Implementation".to_string(),
                description: "Implement the solution".to_string(),
                actions: vec![],
                success_criteria: vec![
                    "Solution implemented".to_string(),
                    "Code compiles/runs".to_string(),
                ],
                completed: false,
                results: None,
            },
            ReasoningPhase {
                name: "Validation".to_string(),
                description: "Validate the solution works correctly".to_string(),
                actions: vec![],
                success_criteria: vec![
                    "Tests pass".to_string(),
                    "Solution validated".to_string(),
                ],
                completed: false,
                results: None,
            }
        ])
    }

    /// Apply intelligence adaptations to phases
    fn apply_intelligence_adaptations(
        &self,
        phases: &mut Vec<ReasoningPhase>,
        adapted_strategy: &crate::agent::intelligent_strategy_selection::AdaptedStrategy,
    ) {
        // Add reflection phase if enabled
        if adapted_strategy.should_use_reflection {
            let reflection_phase = ReasoningPhase {
                name: "Self-Reflection".to_string(),
                description: "Analyze progress and adapt approach based on results".to_string(),
                actions: vec![
                    PlannedAction {
                        action_type: ActionType::Analyze,
                        description: "Review current progress and identify issues".to_string(),
                        tool: "reflect".to_string(),
                        parameters: serde_json::json!({}),
                        expected_outcome: "Clear understanding of what's working and what isn't".to_string(),
                        retry_count: 0,
                        max_retries: 1,
                    }
                ],
                success_criteria: vec![
                    "Identified what's working well".to_string(),
                    "Identified what needs improvement".to_string(),
                    "Generated actionable insights".to_string(),
                ],
                completed: false,
                results: None,
            };

            // Insert reflection phase between analysis and implementation
            if phases.len() >= 2 {
                phases.insert(2, reflection_phase);
            } else {
                phases.push(reflection_phase);
            }
        }

        // Modify exploration depth based on intelligence
        for phase in phases.iter_mut() {
            if phase.name.contains("Exploration") || phase.name.contains("Research") {
                // Adjust number of exploration actions based on max_exploration_depth
                let target_actions = adapted_strategy.max_exploration_depth.min(8);
                if phase.actions.len() < target_actions {
                    // Add more exploration actions for complex tasks
                    for i in phase.actions.len()..target_actions {
                        phase.actions.push(PlannedAction {
                            action_type: ActionType::Explore,
                            description: format!("Deep exploration step {}", i + 1),
                            tool: "search_code".to_string(),
                            parameters: serde_json::json!({"query": "related patterns"}),
                            expected_outcome: "Additional relevant code discovered".to_string(),
                            retry_count: 0,
                            max_retries: 2,
                        });
                    }
                }
            }
        }

        // Add tree search capabilities if enabled
        if adapted_strategy.should_use_tree_search {
            for phase in phases.iter_mut() {
                if phase.name.contains("Analysis") || phase.name.contains("Implementation") {
                    phase.description = format!("{} (with multi-path exploration)", phase.description);

                    // Add backtracking action
                    phase.actions.push(PlannedAction {
                        action_type: ActionType::Analyze,
                        description: "Evaluate multiple solution paths and backtrack if needed".to_string(),
                        tool: "tree_search".to_string(),
                        parameters: serde_json::json!({
                            "beam_width": 3,
                            "max_depth": adapted_strategy.max_exploration_depth
                        }),
                        expected_outcome: "Best solution path identified".to_string(),
                        retry_count: 0,
                        max_retries: 1,
                    });
                }
            }
        }
    }

    /// Plan exploration actions based on the objective
    fn plan_exploration_actions(&self, objective: &str) -> Vec<PlannedAction> {
        let mut actions = Vec::new();

        // Always start by understanding the project structure
        actions.push(PlannedAction {
            action_type: ActionType::Explore,
            description: "List project root to understand overall structure".to_string(),
            tool: "list_files".to_string(),
            parameters: serde_json::json!({
                "path": ".",
                "recursive": false
            }),
            expected_outcome: "Understand top-level project organization".to_string(),
            retry_count: 0,
            max_retries: 2,
        });

        // Look for common configuration files
        actions.push(PlannedAction {
            action_type: ActionType::Explore,
            description: "Check for package.json, Cargo.toml, requirements.txt, etc.".to_string(),
            tool: "run_command".to_string(),
            parameters: serde_json::json!({
                "command": "find . -maxdepth 2 -name 'package.json' -o -name 'Cargo.toml' -o -name 'requirements.txt' -o -name 'pyproject.toml' -o -name 'go.mod' -o -name 'pom.xml' | head -10"
            }),
            expected_outcome: "Identify project language and build system".to_string(),
            retry_count: 0,
            max_retries: 2,
        });

        // Search for relevant files based on objective keywords
        if let Some(search_terms) = self.extract_search_terms(objective) {
            for term in search_terms {
                actions.push(PlannedAction {
                    action_type: ActionType::Explore,
                    description: format!("Search for files related to '{}'", term),
                    tool: "search_code".to_string(),
                    parameters: serde_json::json!({
                        "query": term,
                        "limit": 10
                    }),
                    expected_outcome: format!("Find files containing '{}'", term),
                    retry_count: 0,
                    max_retries: 2,
                });
            }
        }

        // Look for test directories
        actions.push(PlannedAction {
            action_type: ActionType::Explore,
            description: "Find test directories and files".to_string(),
            tool: "run_command".to_string(),
            parameters: serde_json::json!({
                "command": "find . -type d -name '*test*' -o -name '*spec*' | head -10"
            }),
            expected_outcome: "Locate test suites for validation".to_string(),
            retry_count: 0,
            max_retries: 2,
        });

        actions
    }

    /// Extract search terms from the objective for targeted exploration
    fn extract_search_terms(&self, objective: &str) -> Option<Vec<String>> {
        let objective_lower = objective.to_lowercase();
        let mut terms = Vec::new();

        // Extract quoted strings
        if let Some(start) = objective_lower.find('"') {
            if let Some(end) = objective_lower[start + 1..].find('"') {
                let quoted_term = &objective_lower[start + 1..start + 1 + end];
                terms.push(quoted_term.to_string());
            }
        }

        // Look for common patterns
        if objective_lower.contains("bug") || objective_lower.contains("fix") || objective_lower.contains("error") {
            // Extract error-related terms
            for word in objective_lower.split_whitespace() {
                if word.len() > 4 && (word.contains("error") || word.contains("exception") || word.contains("fail")) {
                    terms.push(word.to_string());
                }
            }
        }

        // Extract function/class names (CamelCase or snake_case identifiers)
        for word in objective.split_whitespace() {
            if word.len() > 3 && (word.contains('_') || (word.chars().any(char::is_uppercase) && word.chars().any(char::is_lowercase))) {
                terms.push(word.to_string());
            }
        }

        if terms.is_empty() {
            None
        } else {
            Some(terms)
        }
    }

    /// Queue a phase for execution
    fn queue_phase_execution(&mut self, plan_id: &str, phase_index: usize) -> Result<()> {
        if let Some(plan) = self.active_plans.get(plan_id) {
            if phase_index < plan.phases.len() {
                // Queue all actions in this phase
                for (action_index, _) in plan.phases[phase_index].actions.iter().enumerate() {
                    self.execution_queue.push_back((plan_id.to_string(), action_index));
                }
                info!("Queued {} actions for phase {} of plan {}",
                     plan.phases[phase_index].actions.len(), phase_index, plan_id);
            } else {
                return Err(anyhow!("Phase index {} out of bounds for plan {}", phase_index, plan_id));
            }
        } else {
            return Err(anyhow!("Plan {} not found", plan_id));
        }
        Ok(())
    }

    /// Execute the next action in the queue
    pub async fn execute_next_action(&mut self, provider: &dyn LLMProvider, model: &str) -> Result<Option<ActionResult>> {
        if let Some((plan_id, action_index)) = self.execution_queue.pop_front() {
            self.execute_planned_action(&plan_id, action_index, provider, model).await
        } else {
            Ok(None) // No actions queued
        }
    }

    /// Execute a specific planned action
    async fn execute_planned_action(
        &mut self,
        plan_id: &str,
        action_index: usize,
        provider: &dyn LLMProvider,
        model: &str
    ) -> Result<Option<ActionResult>> {
        let plan = self.active_plans.get_mut(plan_id)
            .ok_or_else(|| anyhow!("Plan {} not found", plan_id))?;

        let current_phase_index = plan.current_phase;
        if current_phase_index >= plan.phases.len() {
            return Err(anyhow!("Current phase index out of bounds"));
        }

        let action = plan.phases[current_phase_index].actions.get(action_index)
            .ok_or_else(|| anyhow!("Action index {} out of bounds", action_index))?
            .clone();

        info!("Executing action: {} - {}", action.action_type as u8, action.description);

        // Execute the tool
        if let Some(tool) = self.tools.get(&action.tool) {
            match tool.execute(action.parameters.clone()).await {
                Ok(output) => {
                    if output.success {
                        let result = ActionResult {
                            action: action.clone(),
                            success: true,
                            output: Some(output.result.to_string()),
                            error: None,
                            learnings: self.extract_learnings(&action, &output.result.to_string()),
                        };

                        // Update plan with results
                        self.update_plan_with_result(plan_id, &result).await?;

                        Ok(Some(result))
                    } else {
                        let error_msg = output.error.unwrap_or_else(|| "Unknown error".to_string());
                        let result = ActionResult {
                            action: action.clone(),
                            success: false,
                            output: None,
                            error: Some(error_msg.clone()),
                            learnings: self.extract_error_learnings(&action, &error_msg),
                        };

                        self.handle_action_failure(plan_id, action_index, &result).await?;

                        Ok(Some(result))
                    }
                }
                Err(e) => {
                    let result = ActionResult {
                        action: action.clone(),
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                        learnings: self.extract_error_learnings(&action, &e.to_string()),
                    };

                    self.handle_action_failure(plan_id, action_index, &result).await?;

                    Ok(Some(result))
                }
            }
        } else {
            Err(anyhow!("Tool '{}' not found", action.tool))
        }
    }

    /// Extract learnings from successful action execution
    fn extract_learnings(&self, action: &PlannedAction, output: &str) -> Vec<String> {
        let mut learnings = Vec::new();

        match action.action_type {
            ActionType::Explore => {
                if output.contains("package.json") {
                    learnings.push("Project is Node.js-based".to_string());
                }
                if output.contains("Cargo.toml") {
                    learnings.push("Project is Rust-based".to_string());
                }
                if output.contains("requirements.txt") || output.contains("pyproject.toml") {
                    learnings.push("Project is Python-based".to_string());
                }
                if output.lines().count() > 10 {
                    learnings.push("Large project with many files".to_string());
                }
            }
            ActionType::Test => {
                if output.contains("PASSED") {
                    learnings.push("Some tests are currently passing".to_string());
                }
                if output.contains("FAILED") {
                    learnings.push("Some tests are currently failing".to_string());
                }
                if output.contains("ERROR") {
                    learnings.push("Test execution has errors".to_string());
                }
            }
            _ => {
                // Generic learnings
                if output.len() > 1000 {
                    learnings.push("Action produced substantial output".to_string());
                } else if output.is_empty() {
                    learnings.push("Action produced no output".to_string());
                }
            }
        }

        learnings
    }

    /// Extract learnings from failed action execution
    fn extract_error_learnings(&self, action: &PlannedAction, error: &str) -> Vec<String> {
        let mut learnings = Vec::new();

        if error.contains("permission denied") || error.contains("Permission denied") {
            learnings.push("Need appropriate file permissions".to_string());
        }
        if error.contains("command not found") || error.contains("No such file") {
            learnings.push("Required tool or file not available".to_string());
        }
        if error.contains("timeout") {
            learnings.push("Operation took too long, may need different approach".to_string());
        }

        learnings.push(format!("Action '{}' failed: {}", action.description, error));
        learnings
    }

    /// Update plan with successful action result
    async fn update_plan_with_result(&mut self, plan_id: &str, result: &ActionResult) -> Result<()> {
        // First, update the plan's learned context
        {
            let plan = self.active_plans.get_mut(plan_id)
                .ok_or_else(|| anyhow!("Plan {} not found", plan_id))?;

            // Store learnings in plan context
            for learning in &result.learnings {
                plan.learned_context.insert(
                    format!("{}_{}", result.action.action_type as u8, plan.learned_context.len()),
                    learning.clone()
                );
            }
        }

        // Check if current phase is complete
        let (current_phase_complete, current_phase_name, should_move_to_next_phase, new_phase_name) = {
            let plan = self.active_plans.get(plan_id)
                .ok_or_else(|| anyhow!("Plan {} not found", plan_id))?;

            let current_phase = &plan.phases[plan.current_phase];
            let completed_actions = current_phase.actions.iter()
                .filter(|a| a.retry_count < a.max_retries) // Actions that succeeded or gave up
                .count();

            let is_complete = completed_actions == current_phase.actions.len();
            let should_move = is_complete && (plan.current_phase + 1 < plan.phases.len());
            let new_phase_name = if should_move {
                Some(plan.phases[plan.current_phase + 1].name.clone())
            } else {
                None
            };

            (is_complete, current_phase.name.clone(), should_move, new_phase_name)
        };

        if current_phase_complete {
            info!("Phase '{}' completed for plan {}", current_phase_name, plan_id);

            if should_move_to_next_phase {
                // Move to next phase
                {
                    let plan = self.active_plans.get_mut(plan_id)
                        .ok_or_else(|| anyhow!("Plan {} not found", plan_id))?;
                    plan.current_phase += 1;
                }

                info!("Moving to phase '{}' for plan {}", new_phase_name.unwrap_or_default(), plan_id);

                // Plan the next phase based on what we've learned
                self.plan_next_phase(plan_id).await?;

                // Queue the next phase
                let current_phase_index = {
                    let plan = self.active_plans.get(plan_id)
                        .ok_or_else(|| anyhow!("Plan {} not found", plan_id))?;
                    plan.current_phase
                };
                self.queue_phase_execution(plan_id, current_phase_index)?;
            } else {
                // Plan is complete!
                let plan = self.active_plans.get_mut(plan_id)
                    .ok_or_else(|| anyhow!("Plan {} not found", plan_id))?;
                plan.state = PlanState::Complete;
                info!("Plan {} completed successfully!", plan_id);
            }
        }

        Ok(())
    }

    /// Handle action failure with retry logic
    async fn handle_action_failure(&mut self, plan_id: &str, action_index: usize, result: &ActionResult) -> Result<()> {
        // Get plan info without holding the mutable reference
        let (current_phase_index, current_retry_count, max_retries, action_description, phase_name, should_continue) = {
            let plan = self.active_plans.get_mut(plan_id)
                .ok_or_else(|| anyhow!("Plan {} not found", plan_id))?;

            let current_phase_index = plan.current_phase;
            let action = &mut plan.phases[current_phase_index].actions[action_index];

            action.retry_count += 1;

            let action_type = action.action_type; // Copy the enum value
            let should_continue_after_failure = if action.retry_count >= action.max_retries {
                Self::should_continue_despite_failure_static_pure(&action_type)
            } else {
                true // Will retry
            };

            (
                current_phase_index,
                action.retry_count,
                action.max_retries,
                action.description.clone(),
                plan.phases[current_phase_index].name.clone(),
                should_continue_after_failure,
            )
        };

        if current_retry_count < max_retries {
            info!("Action failed, retrying ({}/{}): {}", current_retry_count, max_retries, action_description);
            // Re-queue this action for retry
            self.execution_queue.push_front((plan_id.to_string(), action_index));
        } else {
            warn!("Action failed permanently after {} retries: {}", max_retries, action_description);

            // Record the failure
            {
                let plan = self.active_plans.get_mut(plan_id)
                    .ok_or_else(|| anyhow!("Plan {} not found", plan_id))?;

                plan.failed_attempts.push(FailedAttempt {
                    phase: phase_name.clone(),
                    action: action_description.clone(),
                    error: result.error.as_ref().unwrap_or(&"Unknown error".to_string()).clone(),
                    learning: result.learnings.join("; "),
                    timestamp: chrono::Utc::now(),
                });
            }

            // Decide whether to continue or fail the entire plan
            if should_continue {
                info!("Continuing plan despite failed action: {}", action_description);
            } else {
                warn!("Plan {} failed due to critical action failure", plan_id);
                let plan = self.active_plans.get_mut(plan_id)
                    .ok_or_else(|| anyhow!("Plan {} not found", plan_id))?;
                plan.state = PlanState::Failed;
            }
        }

        Ok(())
    }

    /// Determine if we should continue the plan despite an action failure
    fn should_continue_despite_failure(&self, _plan: &ReasoningPlan, failed_action: &PlannedAction) -> bool {
        self.should_continue_despite_failure_static(&failed_action.action_type)
    }

    /// Static version for determining if we should continue despite failure (avoids borrowing issues)
    fn should_continue_despite_failure_static(&self, action_type: &ActionType) -> bool {
        Self::should_continue_despite_failure_static_pure(action_type)
    }

    /// Pure static function for determining if we should continue despite failure
    fn should_continue_despite_failure_static_pure(action_type: &ActionType) -> bool {
        match action_type {
            ActionType::Explore => true,  // Exploration failures are usually non-critical
            ActionType::Analyze => true,  // Analysis failures can often be worked around
            ActionType::Test => false,    // Test failures are usually critical for validation
            ActionType::Implement => false, // Implementation failures are critical
            ActionType::Validate => false,  // Validation failures are critical
            ActionType::Debug => true,   // Debug failures might be acceptable
        }
    }

    /// Plan the next phase based on learnings from previous phases
    async fn plan_next_phase(&mut self, plan_id: &str) -> Result<()> {
        // Get phase info without holding mutable reference
        let (next_phase_index, phase_name, learned_context) = {
            let plan = self.active_plans.get(plan_id)
                .ok_or_else(|| anyhow!("Plan {} not found", plan_id))?;

            let next_phase_index = plan.current_phase;
            if next_phase_index >= plan.phases.len() {
                return Ok(()); // No more phases to plan
            }

            let phase_name = plan.phases[next_phase_index].name.clone();
            let learned_context = plan.learned_context.clone();

            (next_phase_index, phase_name, learned_context)
        };

        // Plan actions for the next phase based on what we learned
        let new_actions = match phase_name.as_str() {
            "Analysis" => {
                self.plan_analysis_actions(&learned_context)
            }
            "Testing" => {
                self.plan_testing_actions(&learned_context)
            }
            "Implementation" => {
                self.plan_implementation_actions(&learned_context)
            }
            "Validation" => {
                self.plan_validation_actions(&learned_context)
            }
            _ => {
                // Phase already has actions planned
                return Ok(());
            }
        };

        // Update the plan with new actions
        {
            let plan = self.active_plans.get_mut(plan_id)
                .ok_or_else(|| anyhow!("Plan {} not found", plan_id))?;
            plan.phases[next_phase_index].actions = new_actions;
        }

        Ok(())
    }

    /// Plan analysis actions based on exploration results
    fn plan_analysis_actions(&self, learned_context: &HashMap<String, String>) -> Vec<PlannedAction> {
        let mut actions = Vec::new();

        // Determine what files to analyze based on exploration
        let mut files_to_analyze = Vec::new();
        for (_, learning) in learned_context {
            if learning.contains("Rust-based") {
                files_to_analyze.push("Cargo.toml");
                files_to_analyze.push("src/main.rs");
                files_to_analyze.push("src/lib.rs");
            }
            if learning.contains("Node.js-based") {
                files_to_analyze.push("package.json");
                files_to_analyze.push("index.js");
                files_to_analyze.push("src/index.js");
            }
            if learning.contains("Python-based") {
                files_to_analyze.push("requirements.txt");
                files_to_analyze.push("main.py");
                files_to_analyze.push("__init__.py");
            }
        }

        // Read and analyze key files
        for file in files_to_analyze.iter().take(5) { // Limit to prevent overwhelming
            actions.push(PlannedAction {
                action_type: ActionType::Analyze,
                description: format!("Analyze {}", file),
                tool: "read_file".to_string(),
                parameters: serde_json::json!({
                    "path": file
                }),
                expected_outcome: format!("Understand structure and purpose of {}", file),
                retry_count: 0,
                max_retries: 2,
            });
        }

        actions
    }

    /// Plan testing actions based on exploration and analysis
    fn plan_testing_actions(&self, learned_context: &HashMap<String, String>) -> Vec<PlannedAction> {
        let mut actions = Vec::new();

        // Determine test commands based on project type
        for (_, learning) in learned_context {
            if learning.contains("Rust-based") {
                actions.push(PlannedAction {
                    action_type: ActionType::Test,
                    description: "Run Rust tests".to_string(),
                    tool: "run_command".to_string(),
                    parameters: serde_json::json!({
                        "command": "cargo test"
                    }),
                    expected_outcome: "See current test status".to_string(),
                    retry_count: 0,
                    max_retries: 1,
                });
            }
            if learning.contains("Node.js-based") {
                actions.push(PlannedAction {
                    action_type: ActionType::Test,
                    description: "Run Node.js tests".to_string(),
                    tool: "run_command".to_string(),
                    parameters: serde_json::json!({
                        "command": "npm test"
                    }),
                    expected_outcome: "See current test status".to_string(),
                    retry_count: 0,
                    max_retries: 1,
                });
            }
            if learning.contains("Python-based") {
                actions.push(PlannedAction {
                    action_type: ActionType::Test,
                    description: "Run Python tests".to_string(),
                    tool: "run_command".to_string(),
                    parameters: serde_json::json!({
                        "command": "python -m pytest -v"
                    }),
                    expected_outcome: "See current test status".to_string(),
                    retry_count: 0,
                    max_retries: 1,
                });
            }
        }

        actions
    }

    /// Plan implementation actions (placeholder - would be customized per problem)
    fn plan_implementation_actions(&self, _learned_context: &HashMap<String, String>) -> Vec<PlannedAction> {
        vec![
            PlannedAction {
                action_type: ActionType::Implement,
                description: "Implementation will be planned based on specific problem analysis".to_string(),
                tool: "read_file".to_string(), // Placeholder
                parameters: serde_json::json!({}),
                expected_outcome: "Custom implementation based on problem".to_string(),
                retry_count: 0,
                max_retries: 2,
            }
        ]
    }

    /// Plan validation actions
    fn plan_validation_actions(&self, learned_context: &HashMap<String, String>) -> Vec<PlannedAction> {
        // Similar to testing actions but focused on validating the changes work
        self.plan_testing_actions(learned_context)
    }

    /// Get the status of all active plans
    pub fn get_plan_status(&self, plan_id: &str) -> Option<&ReasoningPlan> {
        self.active_plans.get(plan_id)
    }

    /// Get all active plan IDs
    pub fn get_active_plan_ids(&self) -> Vec<String> {
        self.active_plans.keys().cloned().collect()
    }

    /// Check if there are queued actions to execute
    pub fn has_queued_actions(&self) -> bool {
        !self.execution_queue.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action: PlannedAction,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub learnings: Vec<String>,
}