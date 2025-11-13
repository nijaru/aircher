use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::intelligence::tools::IntelligenceTools;
use crate::agent::tools::ToolRegistry;
use crate::intelligence::IntelligenceEngine;

/// Context Engineering: A better alternative to sub-agents
/// Based on 2025 research showing structured context management outperforms autonomous sub-agents
pub struct ContextEngine {
    /// Intelligence engine for context analysis
    intelligence: Arc<IntelligenceEngine>,
    /// Tool registry
    #[allow(dead_code)]
    tools: Arc<ToolRegistry>,
    /// Active context windows
    contexts: RwLock<HashMap<String, TaskContext>>,
    /// Context templates for different task types
    templates: RwLock<HashMap<TaskType, ContextTemplate>>,
    /// Maximum context size before compaction
    max_context_tokens: usize,
}

/// Represents a focused task context without autonomous sub-agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    /// Unique context identifier
    pub id: String,
    /// Task type for specialized handling
    pub task_type: TaskType,
    /// Current focus area
    pub focus: FocusArea,
    /// Relevant files and code
    pub relevant_code: Vec<CodeContext>,
    /// Task requirements and constraints
    pub requirements: TaskRequirements,
    /// Execution plan (not autonomous)
    pub plan: Option<ExecutionPlan>,
    /// Current state
    pub state: ContextState,
    /// Token usage tracking
    pub token_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    Feature,      // New feature implementation
    BugFix,       // Bug fixing
    Refactor,     // Code refactoring
    Testing,      // Test writing
    Documentation,// Documentation updates
    Review,       // Code review
    Performance,  // Performance optimization
    Security,     // Security improvements
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusArea {
    /// Primary files being modified
    pub primary_files: Vec<String>,
    /// Secondary files for context
    pub context_files: Vec<String>,
    /// Specific code sections (file:line_start:line_end)
    pub code_sections: Vec<String>,
    /// Relevant symbols (functions, classes)
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContext {
    pub file_path: String,
    pub relevant_sections: Vec<CodeSection>,
    pub importance: ContextImportance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSection {
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub purpose: String, // Why this section is relevant
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContextImportance {
    Critical,  // Must be in context
    High,      // Should be in context
    Medium,    // Include if space allows
    Low,       // Optional context
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequirements {
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub constraints: Vec<String>,
    pub examples: Vec<Example>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub description: String,
    pub input: String,
    pub expected_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub steps: Vec<PlannedStep>,
    pub current_step: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedStep {
    pub description: String,
    pub tools_needed: Vec<String>,
    pub expected_changes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContextState {
    Analyzing,    // Understanding the task
    Planning,     // Creating execution plan
    Executing,    // Working on the task
    Verifying,    // Checking results
    Completed,    // Task done
    Compacting,   // Reducing context size
}

/// Template for creating focused contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextTemplate {
    pub task_type: TaskType,
    pub required_context: Vec<String>,
    pub optional_context: Vec<String>,
    pub max_tokens: usize,
    pub prompt_template: String,
}

impl ContextEngine {
    pub fn new(
        intelligence: Arc<IntelligenceEngine>,
        tools: Arc<ToolRegistry>,
    ) -> Self {
        let mut templates = HashMap::new();

        // Initialize templates for different task types
        templates.insert(TaskType::Feature, ContextTemplate {
            task_type: TaskType::Feature,
            required_context: vec![
                "requirements".to_string(),
                "related_code".to_string(),
                "dependencies".to_string(),
            ],
            optional_context: vec![
                "examples".to_string(),
                "tests".to_string(),
            ],
            max_tokens: 8000,
            prompt_template: "Implement the following feature with focus on the requirements and existing patterns in the codebase.".to_string(),
        });

        templates.insert(TaskType::BugFix, ContextTemplate {
            task_type: TaskType::BugFix,
            required_context: vec![
                "error_message".to_string(),
                "failing_code".to_string(),
                "stack_trace".to_string(),
            ],
            optional_context: vec![
                "recent_changes".to_string(),
                "related_tests".to_string(),
            ],
            max_tokens: 6000,
            prompt_template: "Fix the following bug by analyzing the error and implementing a solution that doesn't break existing functionality.".to_string(),
        });

        Self {
            intelligence,
            tools,
            contexts: RwLock::new(HashMap::new()),
            templates: RwLock::new(templates),
            max_context_tokens: 10000,
        }
    }

    /// Create a new focused context for a task
    pub async fn create_context(
        &self,
        task_description: &str,
        task_type: TaskType,
    ) -> Result<TaskContext> {
        info!("Creating focused context for {:?} task", task_type);

        let id = uuid::Uuid::new_v4().to_string();

        // Analyze task to determine focus area
        let focus = self.analyze_focus_area(task_description).await?;

        // Gather relevant code context
        let relevant_code = self.gather_relevant_code(&focus).await?;

        // Extract requirements
        let requirements = self.extract_requirements(task_description).await?;

        let context = TaskContext {
            id: id.clone(),
            task_type,
            focus,
            relevant_code,
            requirements,
            plan: None,
            state: ContextState::Analyzing,
            token_count: 0,
        };

        // Store context
        let mut contexts = self.contexts.write().await;
        contexts.insert(id.clone(), context.clone());

        Ok(context)
    }

    /// Intentionally compact context to maintain focus
    pub async fn compact_context(&self, context_id: &str) -> Result<TaskContext> {
        info!("Performing intentional context compaction for {}", context_id);

        let mut contexts = self.contexts.write().await;
        let context = contexts.get_mut(context_id)
            .ok_or_else(|| anyhow::anyhow!("Context not found"))?;

        context.state = ContextState::Compacting;

        // Remove low-importance context
        context.relevant_code.retain(|code| {
            code.importance != ContextImportance::Low
        });

        // Truncate medium importance if over limit
        if context.token_count > self.max_context_tokens {
            context.relevant_code.retain(|code| {
                code.importance == ContextImportance::Critical ||
                code.importance == ContextImportance::High
            });
        }

        // Update token count
        context.token_count = self.estimate_tokens(context);

        context.state = ContextState::Executing;
        Ok(context.clone())
    }

    /// Switch context focus without creating new agents
    pub async fn switch_focus(
        &self,
        context_id: &str,
        new_focus: FocusArea,
    ) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        let context = contexts.get_mut(context_id)
            .ok_or_else(|| anyhow::anyhow!("Context not found"))?;

        info!("Switching focus for context {} to {:?}", context_id, new_focus);

        context.focus = new_focus.clone();

        // Re-gather relevant code for new focus
        context.relevant_code = self.gather_relevant_code(&new_focus).await?;

        Ok(())
    }

    /// Create structured execution plan
    pub async fn create_plan(
        &self,
        context_id: &str,
    ) -> Result<ExecutionPlan> {
        let contexts = self.contexts.read().await;
        let context = contexts.get(context_id)
            .ok_or_else(|| anyhow::anyhow!("Context not found"))?;

        info!("Creating structured execution plan for {:?} task", context.task_type);

        // Generate plan based on task type and requirements
        let steps = self.generate_plan_steps(context).await?;

        let plan = ExecutionPlan {
            steps,
            current_step: 0,
        };

        // Update context with plan
        drop(contexts);
        let mut contexts = self.contexts.write().await;
        if let Some(ctx) = contexts.get_mut(context_id) {
            ctx.plan = Some(plan.clone());
            ctx.state = ContextState::Planning;
        }

        Ok(plan)
    }

    /// Get specialized prompt for current context
    pub async fn get_specialized_prompt(
        &self,
        context_id: &str,
    ) -> Result<String> {
        let contexts = self.contexts.read().await;
        let context = contexts.get(context_id)
            .ok_or_else(|| anyhow::anyhow!("Context not found"))?;

        let templates = self.templates.read().await;
        let template = templates.get(&context.task_type)
            .ok_or_else(|| anyhow::anyhow!("No template for task type"))?;

        // Build focused prompt with only relevant context
        let mut prompt = template.prompt_template.clone();
        prompt.push_str("\n\n## Task Requirements:\n");
        prompt.push_str(&context.requirements.description);

        prompt.push_str("\n\n## Relevant Code Context:\n");
        for code in &context.relevant_code {
            if code.importance == ContextImportance::Critical {
                prompt.push_str(&format!("\n### {}\n", code.file_path));
                for section in &code.relevant_sections {
                    prompt.push_str(&format!("Lines {}-{}: {}\n",
                        section.start_line, section.end_line, section.purpose));
                    prompt.push_str(&section.content);
                    prompt.push_str("\n");
                }
            }
        }

        if let Some(plan) = &context.plan {
            prompt.push_str("\n\n## Execution Plan:\n");
            for (i, step) in plan.steps.iter().enumerate() {
                let marker = if i == plan.current_step { "→" } else { " " };
                prompt.push_str(&format!("{} {}. {}\n", marker, i + 1, step.description));
            }
        }

        Ok(prompt)
    }

    // Helper methods

    async fn analyze_focus_area(&self, task: &str) -> Result<FocusArea> {
        // Use intelligence engine to determine focus
        let context = self.intelligence.get_development_context(task).await;

        Ok(FocusArea {
            primary_files: context.key_files.iter()
                .map(|f| f.path.clone())
                .collect(),
            context_files: Vec::new(), // TODO: Extract from recent_patterns when integrated
            code_sections: Vec::new(),
            symbols: Vec::new(),
        })
    }

    async fn gather_relevant_code(&self, focus: &FocusArea) -> Result<Vec<CodeContext>> {
        let mut relevant_code = Vec::new();

        for file in &focus.primary_files {
            relevant_code.push(CodeContext {
                file_path: file.clone(),
                relevant_sections: Vec::new(), // Would be populated from actual file analysis
                importance: ContextImportance::Critical,
            });
        }

        for file in &focus.context_files {
            relevant_code.push(CodeContext {
                file_path: file.clone(),
                relevant_sections: Vec::new(),
                importance: ContextImportance::Medium,
            });
        }

        Ok(relevant_code)
    }

    async fn extract_requirements(&self, task: &str) -> Result<TaskRequirements> {
        Ok(TaskRequirements {
            description: task.to_string(),
            acceptance_criteria: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
        })
    }

    async fn generate_plan_steps(&self, context: &TaskContext) -> Result<Vec<PlannedStep>> {
        let mut steps = Vec::new();

        match context.task_type {
            TaskType::Feature => {
                steps.push(PlannedStep {
                    description: "Analyze requirements and existing patterns".to_string(),
                    tools_needed: vec!["read_file".to_string(), "search_code".to_string()],
                    expected_changes: vec![],
                });
                steps.push(PlannedStep {
                    description: "Implement core functionality".to_string(),
                    tools_needed: vec!["write_file".to_string(), "edit_file".to_string()],
                    expected_changes: context.focus.primary_files.clone(),
                });
                steps.push(PlannedStep {
                    description: "Add tests".to_string(),
                    tools_needed: vec!["write_file".to_string(), "run_tests".to_string()],
                    expected_changes: vec!["tests/".to_string()],
                });
            }
            TaskType::BugFix => {
                steps.push(PlannedStep {
                    description: "Reproduce and understand the bug".to_string(),
                    tools_needed: vec!["read_file".to_string(), "run_command".to_string()],
                    expected_changes: vec![],
                });
                steps.push(PlannedStep {
                    description: "Implement fix".to_string(),
                    tools_needed: vec!["edit_file".to_string()],
                    expected_changes: context.focus.primary_files.clone(),
                });
                steps.push(PlannedStep {
                    description: "Verify fix and test".to_string(),
                    tools_needed: vec!["run_tests".to_string()],
                    expected_changes: vec![],
                });
            }
            _ => {
                steps.push(PlannedStep {
                    description: "Execute task".to_string(),
                    tools_needed: vec!["read_file".to_string(), "edit_file".to_string()],
                    expected_changes: context.focus.primary_files.clone(),
                });
            }
        }

        Ok(steps)
    }

    fn estimate_tokens(&self, context: &TaskContext) -> usize {
        // Rough estimation: 4 chars = 1 token
        let mut chars = 0;

        chars += context.requirements.description.len();
        for code in &context.relevant_code {
            for section in &code.relevant_sections {
                chars += section.content.len();
            }
        }

        chars / 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_creation() {
        // Would need proper mocks
        todo!("Implement context engine tests")
    }
}
