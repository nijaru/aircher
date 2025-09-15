use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};

use crate::agent::sub_agents::{SubAgentManager, AgentSpecialization};
use crate::agent::reasoning::{Task, TaskStatus, AgentReasoning};
use crate::agent::tools::{ToolCall, ToolOutput, ToolRegistry};
use crate::intelligence::IntelligenceEngine;
use crate::providers::LLMProvider;

/// Multi-turn task orchestration for complex workflows
pub struct TaskOrchestrator {
    /// Sub-agent manager for specialized agents
    sub_agents: Arc<SubAgentManager>,
    /// Reasoning engine for task planning
    reasoning: Arc<AgentReasoning>,
    /// Tool registry for direct tool execution
    tools: Arc<ToolRegistry>,
    /// Intelligence engine for context
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
    /// Agent specialization needed
    pub required_agent: AgentSpecialization,
    /// Tools likely needed
    pub expected_tools: Vec<String>,
    /// Dependencies on other steps
    pub depends_on: Vec<String>,
    /// Whether this can be parallelized
    pub can_parallelize: bool,
    /// Estimated complexity (1-10)
    pub complexity: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub success: bool,
    pub output: String,
    pub tools_used: Vec<String>,
    pub agent_used: String,
    pub duration_ms: u64,
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
    AgentDelegation,
    ToolExecution,
    Verification,
    Iteration,
    Completion,
}

impl TaskOrchestrator {
    /// Create a new task orchestrator
    pub fn new(
        sub_agents: Arc<SubAgentManager>,
        reasoning: Arc<AgentReasoning>,
        tools: Arc<ToolRegistry>,
        intelligence: Arc<IntelligenceEngine>,
    ) -> Self {
        Self {
            sub_agents,
            reasoning,
            tools,
            intelligence,
            history: RwLock::new(Vec::new()),
            execution_plan: RwLock::new(None),
            max_iterations: 10,
        }
    }

    /// Execute a complex multi-turn task
    pub async fn execute_task(
        &self,
        request: &str,
        provider: &dyn LLMProvider,
        model: &str,
    ) -> Result<TaskResult> {
        info!("Starting multi-turn task orchestration for: {}", request);

        // Step 1: Generate execution plan
        let plan = self.generate_plan(request, provider, model).await?;

        // Store the plan
        {
            let mut plan_lock = self.execution_plan.write().await;
            *plan_lock = Some(plan.clone());
        }

        // Step 2: Execute plan steps
        let results = self.execute_plan(plan, provider, model).await?;

        // Step 3: Synthesize results
        let final_result = self.synthesize_results(results, provider, model).await?;

        Ok(final_result)
    }

    /// Generate an execution plan for the task
    async fn generate_plan(
        &self,
        request: &str,
        provider: &dyn LLMProvider,
        model: &str,
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

        // Convert reasoning tasks to planned steps
        let mut steps = Vec::new();
        for (i, subtask) in task_result.task.subtasks.iter().enumerate() {
            // Determine required agent based on task content
            let required_agent = self.determine_agent_specialization(&subtask.description).await?;

            // Determine expected tools
            let expected_tools = self.predict_required_tools(&subtask.description).await;

            steps.push(PlannedStep {
                id: format!("step_{}", i + 1),
                description: subtask.description.clone(),
                required_agent,
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

    /// Execute a plan step by step
    async fn execute_plan(
        &self,
        mut plan: ExecutionPlan,
        provider: &dyn LLMProvider,
        model: &str,
    ) -> Result<Vec<StepResult>> {
        info!("Executing plan with {} steps", plan.steps.len());
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

            // Select appropriate agent
            let agent_name = format!("{:?}", step.required_agent).to_lowercase();
            if let Err(e) = self.sub_agents.switch_agent(&agent_name).await {
                warn!("Failed to switch to agent {}: {}", agent_name, e);
                // Fall back to current agent
            }

            // Execute step with selected agent
            let step_result = match self.execute_single_step(step, provider, model).await {
                Ok(output) => StepResult {
                    step_id: step.id.clone(),
                    success: true,
                    output,
                    tools_used: step.expected_tools.clone(), // TODO: Track actual usage
                    agent_used: agent_name,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                },
                Err(e) => {
                    error!("Step {} failed: {}", step.id, e);
                    StepResult {
                        step_id: step.id.clone(),
                        success: false,
                        output: format!("Error: {}", e),
                        tools_used: Vec::new(),
                        agent_used: agent_name,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                    }
                }
            };

            // Record step completion
            self.add_history_step(OrchestrationStep {
                timestamp: chrono::Utc::now(),
                step_type: StepType::AgentDelegation,
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

    /// Execute a single step
    async fn execute_single_step(
        &self,
        step: &PlannedStep,
        provider: &dyn LLMProvider,
        model: &str,
    ) -> Result<String> {
        // Get the active agent
        let agent = self.sub_agents.get_active_agent().await
            .ok_or_else(|| anyhow::anyhow!("No active agent"))?;

        // Process the task with the agent
        let result = agent.process_task(&step.description, provider, model).await?;

        // If tools are expected, try to execute them
        if !step.expected_tools.is_empty() {
            debug!("Attempting to use tools: {:?}", step.expected_tools);
            // TODO: Integrate tool execution with agent processing
        }

        Ok(result)
    }

    /// Synthesize results from all steps
    async fn synthesize_results(
        &self,
        results: Vec<StepResult>,
        provider: &dyn LLMProvider,
        model: &str,
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
        })
    }

    /// Determine which agent specialization is best for a task
    async fn determine_agent_specialization(&self, task: &str) -> Result<AgentSpecialization> {
        let task_lower = task.to_lowercase();

        Ok(if task_lower.contains("ui") || task_lower.contains("component") || task_lower.contains("frontend") {
            AgentSpecialization::Frontend
        } else if task_lower.contains("api") || task_lower.contains("database") || task_lower.contains("server") {
            AgentSpecialization::Backend
        } else if task_lower.contains("test") || task_lower.contains("spec") {
            AgentSpecialization::Testing
        } else if task_lower.contains("deploy") || task_lower.contains("docker") {
            AgentSpecialization::DevOps
        } else if task_lower.contains("security") || task_lower.contains("auth") {
            AgentSpecialization::Security
        } else if task_lower.contains("performance") || task_lower.contains("optimize") {
            AgentSpecialization::Performance
        } else if task_lower.contains("review") || task_lower.contains("refactor") {
            AgentSpecialization::CodeReview
        } else {
            AgentSpecialization::Research
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
        if task_lower.contains("commit") || task_lower.contains("git") {
            tools.push("smart_commit".to_string());
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_specialization_detection() {
        let orchestrator = create_test_orchestrator().await;

        assert_eq!(
            orchestrator.determine_agent_specialization("build a React component").await.unwrap(),
            AgentSpecialization::Frontend
        );

        assert_eq!(
            orchestrator.determine_agent_specialization("create API endpoint").await.unwrap(),
            AgentSpecialization::Backend
        );

        assert_eq!(
            orchestrator.determine_agent_specialization("write unit tests").await.unwrap(),
            AgentSpecialization::Testing
        );
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