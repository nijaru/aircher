/// Plan Mode implementation for safe code exploration
///
/// Inspired by Claude Code's Plan Mode, this provides:
/// - Read-only codebase exploration
/// - Task planning without execution
/// - Safe analysis before making changes
/// - User approval before switching to execution mode

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info};

/// Plan Mode execution state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlanMode {
    /// Normal execution mode (default)
    Normal,
    /// Plan-only mode - read operations only
    Planning,
    /// Auto-accept mode - no approvals needed
    AutoAccept,
}

impl Default for PlanMode {
    fn default() -> Self {
        PlanMode::Normal
    }
}

/// A planned task that hasn't been executed yet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedTask {
    pub id: String,
    pub description: String,
    pub task_type: PlannedTaskType,
    pub dependencies: Vec<String>, // IDs of tasks this depends on
    pub estimated_effort: TaskEffort,
    pub safety_level: TaskSafety,
    pub rationale: String, // Why this task is needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlannedTaskType {
    /// Read and analyze files
    Analysis {
        files: Vec<PathBuf>,
        purpose: String,
    },
    /// Modify existing files
    Modification {
        files: Vec<PathBuf>,
        changes_summary: String,
    },
    /// Create new files
    Creation {
        files: Vec<PathBuf>,
        purpose: String,
    },
    /// Delete files or code
    Deletion {
        files: Vec<PathBuf>,
        reason: String,
    },
    /// Run commands or tests
    Execution {
        commands: Vec<String>,
        purpose: String,
    },
    /// Refactoring operations
    Refactoring {
        scope: String,
        approach: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskEffort {
    Trivial,    // < 5 minutes
    Small,      // 5-15 minutes
    Medium,     // 15-60 minutes
    Large,      // 1-4 hours
    ExtraLarge, // > 4 hours
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskSafety {
    /// No risk - read operations only
    Safe,
    /// Low risk - minor changes, easily reversible
    LowRisk,
    /// Medium risk - significant changes, some complexity
    MediumRisk,
    /// High risk - major changes, potential for issues
    HighRisk,
    /// Critical - could break functionality or data
    Critical,
}

/// Plan generation and management
pub struct PlanGenerator {
    mode: PlanMode,
    current_plan: Option<ExecutionPlan>,
    task_id_counter: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub id: String,
    pub title: String,
    pub description: String,
    pub tasks: Vec<PlannedTask>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub estimated_total_time: std::time::Duration,
    pub risk_assessment: PlanRiskAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanRiskAssessment {
    pub overall_risk: TaskSafety,
    pub risk_factors: Vec<String>,
    pub mitigation_strategies: Vec<String>,
    pub rollback_plan: String,
}

impl PlanGenerator {
    pub fn new() -> Self {
        Self {
            mode: PlanMode::default(),
            current_plan: None,
            task_id_counter: 1,
        }
    }

    pub fn set_mode(&mut self, mode: PlanMode) {
        info!("Plan mode changed: {:?} -> {:?}", self.mode, mode);
        self.mode = mode;
    }

    pub fn get_mode(&self) -> PlanMode {
        self.mode.clone()
    }

    pub fn is_planning_mode(&self) -> bool {
        matches!(self.mode, PlanMode::Planning)
    }

    pub fn is_auto_accept_mode(&self) -> bool {
        matches!(self.mode, PlanMode::AutoAccept)
    }

    /// Generate a plan for the given user request
    pub async fn generate_plan(&mut self, user_request: &str, context: &PlanningContext) -> Result<ExecutionPlan> {
        debug!("Generating plan for request: {}", user_request);

        let plan_id = format!("plan_{}", chrono::Utc::now().timestamp());
        let mut tasks = Vec::new();

        // Analyze the request and break it down into tasks
        let analysis_tasks = self.create_analysis_tasks(user_request, context).await?;
        tasks.extend(analysis_tasks);

        let implementation_tasks = self.create_implementation_tasks(user_request, context).await?;
        tasks.extend(implementation_tasks);

        let validation_tasks = self.create_validation_tasks(user_request, context).await?;
        tasks.extend(validation_tasks);

        // Calculate overall risk and timing
        let risk_assessment = self.assess_plan_risk(&tasks);
        let estimated_time = self.estimate_total_time(&tasks);

        let plan = ExecutionPlan {
            id: plan_id,
            title: self.generate_plan_title(user_request),
            description: format!("Plan to: {}", user_request),
            tasks,
            created_at: chrono::Utc::now(),
            estimated_total_time: estimated_time,
            risk_assessment,
        };

        self.current_plan = Some(plan.clone());
        Ok(plan)
    }

    /// Create analysis tasks to understand the codebase
    async fn create_analysis_tasks(&mut self, request: &str, context: &PlanningContext) -> Result<Vec<PlannedTask>> {
        let mut tasks = Vec::new();

        // Always start with understanding the current state
        tasks.push(PlannedTask {
            id: self.next_task_id(),
            description: "Analyze current codebase structure".to_string(),
            task_type: PlannedTaskType::Analysis {
                files: context.relevant_files.clone(),
                purpose: "Understand existing implementation and identify modification points".to_string(),
            },
            dependencies: Vec::new(),
            estimated_effort: TaskEffort::Small,
            safety_level: TaskSafety::Safe,
            rationale: "Need to understand current state before making changes".to_string(),
        });

        // Add request-specific analysis
        if request.contains("test") || request.contains("bug") {
            tasks.push(PlannedTask {
                id: self.next_task_id(),
                description: "Examine test coverage and failure patterns".to_string(),
                task_type: PlannedTaskType::Analysis {
                    files: context.test_files.clone(),
                    purpose: "Identify testing gaps and failure scenarios".to_string(),
                },
                dependencies: vec![tasks[0].id.clone()],
                estimated_effort: TaskEffort::Medium,
                safety_level: TaskSafety::Safe,
                rationale: "Testing context is crucial for safe modifications".to_string(),
            });
        }

        Ok(tasks)
    }

    /// Create implementation tasks based on the request
    async fn create_implementation_tasks(&mut self, request: &str, _context: &PlanningContext) -> Result<Vec<PlannedTask>> {
        let mut tasks = Vec::new();

        // Determine task type based on request keywords
        if request.contains("add") || request.contains("implement") || request.contains("create") {
            tasks.push(PlannedTask {
                id: self.next_task_id(),
                description: "Implement new functionality".to_string(),
                task_type: PlannedTaskType::Creation {
                    files: vec![PathBuf::from("src/new_feature.rs")], // Placeholder
                    purpose: "Add requested functionality".to_string(),
                },
                dependencies: Vec::new(),
                estimated_effort: TaskEffort::Medium,
                safety_level: TaskSafety::MediumRisk,
                rationale: "New functionality needed as requested".to_string(),
            });
        } else if request.contains("fix") || request.contains("bug") {
            tasks.push(PlannedTask {
                id: self.next_task_id(),
                description: "Fix identified issues".to_string(),
                task_type: PlannedTaskType::Modification {
                    files: Vec::new(), // Would be filled from analysis
                    changes_summary: "Fix bugs and issues".to_string(),
                },
                dependencies: Vec::new(),
                estimated_effort: TaskEffort::Small,
                safety_level: TaskSafety::LowRisk,
                rationale: "Bug fixes improve system stability".to_string(),
            });
        } else if request.contains("refactor") || request.contains("clean") {
            tasks.push(PlannedTask {
                id: self.next_task_id(),
                description: "Refactor and improve code quality".to_string(),
                task_type: PlannedTaskType::Refactoring {
                    scope: "Identified code areas".to_string(),
                    approach: "Incremental refactoring with tests".to_string(),
                },
                dependencies: Vec::new(),
                estimated_effort: TaskEffort::Large,
                safety_level: TaskSafety::MediumRisk,
                rationale: "Refactoring improves maintainability".to_string(),
            });
        }

        Ok(tasks)
    }

    /// Create validation tasks to ensure changes work
    async fn create_validation_tasks(&mut self, _request: &str, _context: &PlanningContext) -> Result<Vec<PlannedTask>> {
        let mut tasks = Vec::new();

        // Always validate with tests
        tasks.push(PlannedTask {
            id: self.next_task_id(),
            description: "Run tests to validate changes".to_string(),
            task_type: PlannedTaskType::Execution {
                commands: vec!["cargo test".to_string()],
                purpose: "Ensure changes don't break existing functionality".to_string(),
            },
            dependencies: Vec::new(), // Would depend on implementation tasks
            estimated_effort: TaskEffort::Small,
            safety_level: TaskSafety::Safe,
            rationale: "Testing ensures quality and catches regressions".to_string(),
        });

        Ok(tasks)
    }

    fn assess_plan_risk(&self, tasks: &[PlannedTask]) -> PlanRiskAssessment {
        let max_risk = tasks.iter()
            .map(|t| &t.safety_level)
            .max_by_key(|risk| match risk {
                TaskSafety::Safe => 0,
                TaskSafety::LowRisk => 1,
                TaskSafety::MediumRisk => 2,
                TaskSafety::HighRisk => 3,
                TaskSafety::Critical => 4,
            })
            .cloned()
            .unwrap_or(TaskSafety::Safe);

        let risk_factors = tasks.iter()
            .filter_map(|t| match t.safety_level {
                TaskSafety::MediumRisk | TaskSafety::HighRisk | TaskSafety::Critical => {
                    Some(format!("{}: {}", t.description, t.rationale))
                }
                _ => None,
            })
            .collect();

        PlanRiskAssessment {
            overall_risk: max_risk,
            risk_factors,
            mitigation_strategies: vec![
                "Create backup before major changes".to_string(),
                "Run tests after each step".to_string(),
                "Use git commits for rollback points".to_string(),
            ],
            rollback_plan: "Use git to revert changes, restore from backup if needed".to_string(),
        }
    }

    fn estimate_total_time(&self, tasks: &[PlannedTask]) -> std::time::Duration {
        let total_minutes: u64 = tasks.iter()
            .map(|t| match t.estimated_effort {
                TaskEffort::Trivial => 3,
                TaskEffort::Small => 10,
                TaskEffort::Medium => 30,
                TaskEffort::Large => 120,
                TaskEffort::ExtraLarge => 300,
            })
            .sum();

        std::time::Duration::from_secs(total_minutes * 60)
    }

    fn generate_plan_title(&self, request: &str) -> String {
        if request.len() > 50 {
            format!("{}...", &request[..47])
        } else {
            request.to_string()
        }
    }

    fn next_task_id(&mut self) -> String {
        let id = format!("task_{}", self.task_id_counter);
        self.task_id_counter += 1;
        id
    }

    pub fn get_current_plan(&self) -> Option<&ExecutionPlan> {
        self.current_plan.as_ref()
    }

    pub fn clear_plan(&mut self) {
        self.current_plan = None;
    }
}

/// Context information for plan generation
pub struct PlanningContext {
    pub relevant_files: Vec<PathBuf>,
    pub test_files: Vec<PathBuf>,
    pub recent_changes: Vec<String>,
    pub project_structure: HashMap<String, Vec<PathBuf>>,
    pub dependencies: Vec<String>,
}

impl Default for PlanningContext {
    fn default() -> Self {
        Self {
            relevant_files: Vec::new(),
            test_files: Vec::new(),
            recent_changes: Vec::new(),
            project_structure: HashMap::new(),
            dependencies: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plan_generation() {
        let mut generator = PlanGenerator::new();
        let context = PlanningContext::default();

        let plan = generator.generate_plan("Add user authentication system", &context).await.unwrap();

        assert!(!plan.tasks.is_empty());
        assert!(plan.title.contains("authentication"));
        assert_eq!(plan.risk_assessment.overall_risk, TaskSafety::MediumRisk);
    }

    #[test]
    fn test_mode_switching() {
        let mut generator = PlanGenerator::new();

        assert_eq!(generator.get_mode(), PlanMode::Normal);

        generator.set_mode(PlanMode::Planning);
        assert!(generator.is_planning_mode());

        generator.set_mode(PlanMode::AutoAccept);
        assert!(generator.is_auto_accept_mode());
    }
}