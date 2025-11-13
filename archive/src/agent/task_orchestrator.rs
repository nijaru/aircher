use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};

use crate::agent::core::Agent;
use crate::agent::reasoning::AgentReasoning;
use crate::agent::dynamic_context::DynamicContextManager;
use crate::intelligence::IntelligenceEngine;
use crate::providers::LLMProvider;

/// Multi-turn task orchestration using context engineering instead of sub-agents
///
/// This orchestrator provides the same functionality as the old sub-agent approach
/// but uses our unified Agent with intelligent context management for better performance
/// and avoiding the 19% degradation and tunnel vision issues of sub-agents.
pub struct TaskOrchestrator {
    /// Unified agent for all task execution
    agent: Arc<Agent>,
    /// Reasoning engine for task planning
    reasoning: Arc<AgentReasoning>,
    /// Context manager for intelligent context switching
    context_manager: Arc<DynamicContextManager>,
    /// Intelligence engine for enhanced analysis
    #[allow(dead_code)]
    intelligence: Arc<IntelligenceEngine>,
    /// Task execution history
    history: RwLock<Vec<OrchestrationStep>>,
    /// Current execution plan
    execution_plan: RwLock<Option<ExecutionPlan>>,
    /// Max iterations for autonomous execution
    max_iterations: usize,
}

/// Represents a complete execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// Original user request
    pub request: String,
    /// High-level goal
    pub goal: String,
    /// Decomposed steps
    pub steps: Vec<PlannedStep>,
    /// Current step index
    pub current_step: usize,
    /// Overall status
    pub status: PlanStatus,
    /// Accumulated results
    pub results: Vec<StepResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedStep {
    /// Step identifier
    pub id: String,
    /// Step description
    pub description: String,
    /// Context focus for this step (replaces agent specialization)
    pub context_focus: ContextFocus,
    /// Tools likely needed
    pub expected_tools: Vec<String>,
    /// Dependencies on other steps
    pub depends_on: Vec<String>,
    /// Whether this can be parallelized
    pub can_parallelize: bool,
    /// Estimated complexity (1-10)
    pub complexity: u8,
}

/// Defines what context the agent should focus on for a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextFocus {
    Frontend { ui_files: Vec<String>, components: Vec<String> },
    Backend { api_files: Vec<String>, models: Vec<String> },
    Testing { test_files: Vec<String>, target_code: Vec<String> },
    DevOps { config_files: Vec<String>, deploy_scripts: Vec<String> },
    Security { auth_files: Vec<String>, security_configs: Vec<String> },
    Performance { bottleneck_files: Vec<String>, metrics: Vec<String> },
    CodeReview { review_files: Vec<String>, patterns: Vec<String> },
    Research { exploration_areas: Vec<String>, investigation_queries: Vec<String> },
    General { relevant_files: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub success: bool,
    pub output: String,
    pub tools_used: Vec<String>,
    pub context_focus: String,
    pub duration_ms: u64,
    pub files_modified: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlanStatus {
    Planning,
    Executing,
    Completed,
    Failed(String),
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationStep {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub step_type: StepType,
    pub description: String,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    Planning,
    ContextSwitching,
    Execution,
    Verification,
    Iteration,
    Completion,
}

impl TaskOrchestrator {
    /// Create a new task orchestrator with context engineering
    pub fn new(
        agent: Arc<Agent>,
        reasoning: Arc<AgentReasoning>,
        context_manager: Arc<DynamicContextManager>,
        intelligence: Arc<IntelligenceEngine>,
    ) -> Self {
        Self {
            agent,
            reasoning,
            context_manager,
            intelligence,
            history: RwLock::new(Vec::new()),
            execution_plan: RwLock::new(None),
            max_iterations: 15, // Slightly higher than old limit for complex tasks
        }
    }

    /// Execute a complex multi-turn task using context engineering
    pub async fn execute_task(
        &self,
        request: &str,
        provider: &dyn LLMProvider,
        model: &str,
    ) -> Result<TaskResult> {
        info!("Starting context-engineered multi-turn task orchestration for: {}", request);

        // Step 1: Generate execution plan
        let plan = self.generate_plan(request, provider, model).await?;

        // Store the plan
        {
            let mut plan_lock = self.execution_plan.write().await;
            *plan_lock = Some(plan.clone());
        }

        // Step 2: Execute plan steps with context switching
        let results = self.execute_plan(plan, provider, model).await?;

        // Step 3: Synthesize results
        let final_result = self.synthesize_results(results, provider, model).await?;

        Ok(final_result)
    }

    /// Generate an execution plan using reasoning engine
    async fn generate_plan(
        &self,
        request: &str,
        _provider: &dyn LLMProvider,
        _model: &str,
    ) -> Result<ExecutionPlan> {
        info!("Generating execution plan for task");

        // Record planning step
        self.add_history_step(OrchestrationStep {
            timestamp: chrono::Utc::now(),
            step_type: StepType::Planning,
            description: format!("Analyzing task: {}", request),
            result: None,
        }).await;

        // Use reasoning engine to decompose task
        let task_result = self.reasoning.process_request(request).await?;

        // Convert reasoning tasks to planned steps with context focus
        let mut steps = Vec::new();
        for (i, subtask) in task_result.task.subtasks.iter().enumerate() {
            // Determine context focus based on task content
            let context_focus = self.determine_context_focus(&subtask.description).await?;

            // Determine expected tools
            let expected_tools = self.predict_required_tools(&subtask.description).await;

            steps.push(PlannedStep {
                id: format!("step_{}", i + 1),
                description: subtask.description.clone(),
                context_focus,
                expected_tools,
                depends_on: if i > 0 { vec![format!("step_{}", i)] } else { vec![] },
                can_parallelize: false, // Conservative default
                complexity: self.estimate_complexity(&subtask.description),
            });
        }

        let plan = ExecutionPlan {
            request: request.to_string(),
            goal: task_result.task.description,
            steps,
            current_step: 0,
            status: PlanStatus::Planning,
            results: Vec::new(),
        };

        // Record plan completion
        self.add_history_step(OrchestrationStep {
            timestamp: chrono::Utc::now(),
            step_type: StepType::Planning,
            description: format!("Generated {} step execution plan", plan.steps.len()),
            result: Some(serde_json::to_string(&plan)?),
        }).await;

        Ok(plan)
    }

    /// Execute a plan step by step with intelligent context switching
    async fn execute_plan(
        &self,
        mut plan: ExecutionPlan,
        provider: &dyn LLMProvider,
        model: &str,
    ) -> Result<Vec<StepResult>> {
        info!("Executing plan with {} steps using context engineering", plan.steps.len());
        plan.status = PlanStatus::Executing;

        let mut results = Vec::new();
        let mut iterations = 0;

        while plan.current_step < plan.steps.len() && iterations < self.max_iterations {
            let step = &plan.steps[plan.current_step];
            let start_time = std::time::Instant::now();

            info!("Executing step {}: {}", step.id, step.description);

            // Check dependencies
            if !self.dependencies_met(&step.depends_on, &results).await {
                warn!("Dependencies not met for step {}, skipping", step.id);
                plan.current_step += 1;
                continue;
            }

            // Switch context focus for this step (replaces agent switching)
            self.prepare_context_for_step(step).await?;

            // Execute step with focused context
            let step_result = match self.execute_single_step(step, provider, model).await {
                Ok((output, files_modified)) => StepResult {
                    step_id: step.id.clone(),
                    success: true,
                    output,
                    tools_used: step.expected_tools.clone(), // TODO: Track actual usage
                    context_focus: format!("{:?}", step.context_focus),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    files_modified,
                },
                Err(e) => {
                    error!("Step {} failed: {}", step.id, e);
                    StepResult {
                        step_id: step.id.clone(),
                        success: false,
                        output: format!("Error: {}", e),
                        tools_used: Vec::new(),
                        context_focus: format!("{:?}", step.context_focus),
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        files_modified: Vec::new(),
                    }
                }
            };

            // Record step completion
            self.add_history_step(OrchestrationStep {
                timestamp: chrono::Utc::now(),
                step_type: StepType::Execution,
                description: format!("Completed step {}: {}", step.id, step.description),
                result: Some(step_result.output.clone()),
            }).await;

            results.push(step_result);
            plan.current_step += 1;
            iterations += 1;

            // Check if we should continue or need user input
            if iterations >= self.max_iterations {
                warn!("Reached max iterations limit");
                plan.status = PlanStatus::Paused;
                break;
            }
        }

        if plan.current_step >= plan.steps.len() {
            plan.status = PlanStatus::Completed;
        }

        Ok(results)
    }

    /// Prepare context manager for executing a specific step
    async fn prepare_context_for_step(&self, step: &PlannedStep) -> Result<()> {
        info!("Preparing context for step: {:?}", step.context_focus);

        self.add_history_step(OrchestrationStep {
            timestamp: chrono::Utc::now(),
            step_type: StepType::ContextSwitching,
            description: format!("Switching context focus for step {}", step.id),
            result: None,
        }).await;

        // Update context based on the step's focus area
        let context_activity = match &step.context_focus {
            ContextFocus::Frontend { ui_files, components } => {
                format!("Frontend development task focusing on UI files: {} and components: {}",
                    ui_files.join(", "), components.join(", "))
            }
            ContextFocus::Backend { api_files, models } => {
                format!("Backend development task focusing on API files: {} and models: {}",
                    api_files.join(", "), models.join(", "))
            }
            ContextFocus::Testing { test_files, target_code } => {
                format!("Testing task focusing on test files: {} and target code: {}",
                    test_files.join(", "), target_code.join(", "))
            }
            ContextFocus::General { relevant_files } => {
                format!("General development task with relevant files: {}",
                    relevant_files.join(", "))
            }
            _ => format!("Task: {}", step.description),
        };

        // Update context manager with focused activity
        let _ = self.context_manager.update_context(&context_activity).await
            .map_err(|e| debug!("Context update failed: {}", e));

        Ok(())
    }

    /// Execute a single step with the prepared context
    async fn execute_single_step(
        &self,
        step: &PlannedStep,
        provider: &dyn LLMProvider,
        model: &str,
    ) -> Result<(String, Vec<String>)> {
        // Use the unified agent to process this step directly (bypass orchestration to avoid recursion)
        let (response, tool_status) = self.agent.process_message_direct(&step.description, provider, model).await?;

        // Extract file modifications from tool status (simplified)
        let files_modified = tool_status.iter()
            .filter_map(|status| {
                if status.contains("write_file") || status.contains("edit_file") {
                    // Extract file path from status message (simplified parsing)
                    None // TODO: Implement proper file tracking
                } else {
                    None
                }
            })
            .collect();

        Ok((response, files_modified))
    }

    /// Synthesize results from all steps
    async fn synthesize_results(
        &self,
        results: Vec<StepResult>,
        _provider: &dyn LLMProvider,
        _model: &str,
    ) -> Result<TaskResult> {
        info!("Synthesizing results from {} steps", results.len());

        let successful = results.iter().filter(|r| r.success).count();
        let failed = results.len() - successful;

        let summary = if failed == 0 {
            format!("Successfully completed all {} steps", successful)
        } else {
            format!("Completed {} of {} steps ({} failed)", successful, results.len(), failed)
        };

        // Combine all outputs
        let mut combined_output = String::new();
        for result in &results {
            combined_output.push_str(&format!(
                "\n## Step {}: {}\n{}\n",
                result.step_id,
                if result.success { "✓" } else { "✗" },
                result.output
            ));
        }

        // Record completion
        self.add_history_step(OrchestrationStep {
            timestamp: chrono::Utc::now(),
            step_type: StepType::Completion,
            description: summary.clone(),
            result: Some(combined_output.clone()),
        }).await;

        Ok(TaskResult {
            success: failed == 0,
            summary,
            details: combined_output,
            steps_completed: successful,
            steps_total: results.len(),
            execution_time_ms: results.iter().map(|r| r.duration_ms).sum(),
            files_modified: results.iter()
                .flat_map(|r| r.files_modified.iter())
                .cloned()
                .collect(),
        })
    }

    /// Determine context focus based on task content (replaces agent specialization)
    async fn determine_context_focus(&self, task: &str) -> Result<ContextFocus> {
        let task_lower = task.to_lowercase();

        Ok(if task_lower.contains("ui") || task_lower.contains("component") || task_lower.contains("frontend") {
            ContextFocus::Frontend {
                ui_files: vec!["src/ui/".to_string()],
                components: vec![]
            }
        } else if task_lower.contains("api") || task_lower.contains("database") || task_lower.contains("server") {
            ContextFocus::Backend {
                api_files: vec!["src/api/".to_string()],
                models: vec![]
            }
        } else if task_lower.contains("test") || task_lower.contains("spec") {
            ContextFocus::Testing {
                test_files: vec!["tests/".to_string()],
                target_code: vec![]
            }
        } else if task_lower.contains("deploy") || task_lower.contains("docker") {
            ContextFocus::DevOps {
                config_files: vec![],
                deploy_scripts: vec![]
            }
        } else if task_lower.contains("security") || task_lower.contains("auth") {
            ContextFocus::Security {
                auth_files: vec!["src/auth/".to_string()],
                security_configs: vec![]
            }
        } else if task_lower.contains("performance") || task_lower.contains("optimize") {
            ContextFocus::Performance {
                bottleneck_files: vec![],
                metrics: vec![]
            }
        } else if task_lower.contains("review") || task_lower.contains("refactor") {
            ContextFocus::CodeReview {
                review_files: vec![],
                patterns: vec![]
            }
        } else {
            ContextFocus::General {
                relevant_files: vec![]
            }
        })
    }

    /// Predict which tools will be needed for a task
    async fn predict_required_tools(&self, task: &str) -> Vec<String> {
        let mut tools = Vec::new();
        let task_lower = task.to_lowercase();

        // Always need file reading
        tools.push("read_file".to_string());

        if task_lower.contains("create") || task_lower.contains("write") || task_lower.contains("implement") {
            tools.push("write_file".to_string());
        }
        if task_lower.contains("edit") || task_lower.contains("modify") || task_lower.contains("update") {
            tools.push("edit_file".to_string());
        }
        if task_lower.contains("test") || task_lower.contains("run") {
            tools.push("run_command".to_string());
        }
        if task_lower.contains("search") || task_lower.contains("find") {
            tools.push("search_code".to_string());
        }

        tools
    }

    /// Estimate task complexity (1-10)
    fn estimate_complexity(&self, task: &str) -> u8 {
        let task_lower = task.to_lowercase();

        if task_lower.contains("refactor") || task_lower.contains("architecture") {
            8
        } else if task_lower.contains("implement") || task_lower.contains("create") {
            6
        } else if task_lower.contains("fix") || task_lower.contains("debug") {
            5
        } else if task_lower.contains("update") || task_lower.contains("modify") {
            4
        } else if task_lower.contains("read") || task_lower.contains("analyze") {
            3
        } else {
            5 // Default medium complexity
        }
    }

    /// Check if dependencies are met
    async fn dependencies_met(&self, deps: &[String], results: &[StepResult]) -> bool {
        for dep in deps {
            if !results.iter().any(|r| r.step_id == *dep && r.success) {
                return false;
            }
        }
        true
    }

    /// Add a step to the orchestration history
    async fn add_history_step(&self, step: OrchestrationStep) {
        let mut history = self.history.write().await;
        history.push(step);
    }

    /// Get the current execution plan
    pub async fn get_current_plan(&self) -> Option<ExecutionPlan> {
        let plan = self.execution_plan.read().await;
        plan.clone()
    }

    /// Get orchestration history
    pub async fn get_history(&self) -> Vec<OrchestrationStep> {
        let history = self.history.read().await;
        history.clone()
    }
}

/// Result of task orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub summary: String,
    pub details: String,
    pub steps_completed: usize,
    pub steps_total: usize,
    pub execution_time_ms: u64,
    pub files_modified: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_focus_detection() {
        // Would need proper mocks for comprehensive testing
        let orchestrator = create_test_orchestrator().await;

        let frontend_focus = orchestrator.determine_context_focus("build a React component").await.unwrap();
        assert!(matches!(frontend_focus, ContextFocus::Frontend { .. }));

        let backend_focus = orchestrator.determine_context_focus("create API endpoint").await.unwrap();
        assert!(matches!(backend_focus, ContextFocus::Backend { .. }));

        let testing_focus = orchestrator.determine_context_focus("write unit tests").await.unwrap();
        assert!(matches!(testing_focus, ContextFocus::Testing { .. }));
    }

    #[tokio::test]
    async fn test_tool_prediction() {
        let orchestrator = create_test_orchestrator().await;

        let tools = orchestrator.predict_required_tools("create a new file").await;
        assert!(tools.contains(&"write_file".to_string()));

        let tools = orchestrator.predict_required_tools("search for authentication").await;
        assert!(tools.contains(&"search_code".to_string()));
    }

    async fn create_test_orchestrator() -> TaskOrchestrator {
        // This would need proper mocks in a real test
        todo!("Implement test orchestrator with mocks")
    }
}
