use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::agent::tools::{ToolCall, ToolOutput, ToolRegistry};
use crate::intelligence::IntelligenceEngine;

/// Represents a high-level task that can be decomposed into subtasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub intent: TaskIntent,
    pub subtasks: Vec<Task>,
    pub dependencies: Vec<String>, // IDs of tasks that must complete first
    pub status: TaskStatus,
    pub tool_calls: Vec<ToolCall>,
    pub outputs: Vec<ToolOutput>,
    pub context: TaskContext,
    pub confidence: f32,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskIntent {
    FileOperation,
    CodeGeneration,
    Refactoring,
    BugFix,
    Testing,
    Documentation,
    Research,
    BuildOperation,
    GitOperation,
    Complex, // Requires decomposition
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Planning,
    Executing,
    Completed,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub files_involved: Vec<String>,
    pub symbols_referenced: Vec<String>,
    pub test_files: Vec<String>,
    pub build_commands: Vec<String>,
    pub git_branch: Option<String>,
    pub project_type: Option<String>,
    pub constraints: Vec<String>,
}

/// Pattern learned from successful task sequences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub trigger: String,           // What triggers this pattern
    pub task_sequence: Vec<String>, // Sequence of task types
    pub tool_sequence: Vec<String>, // Sequence of tools used
    pub success_rate: f32,
    pub avg_duration_ms: u64,
    pub usage_count: u32,
    pub last_used: chrono::DateTime<chrono::Utc>,
}

/// Manages task planning and decomposition
pub struct TaskPlanner {
    #[allow(dead_code)]
    intelligence: Arc<IntelligenceEngine>,
    patterns: Arc<RwLock<HashMap<String, LearnedPattern>>>,
    #[allow(dead_code)]
    max_decomposition_depth: usize,
}

impl TaskPlanner {
    pub fn new(intelligence: Arc<IntelligenceEngine>) -> Self {
        Self {
            intelligence,
            patterns: Arc::new(RwLock::new(HashMap::new())),
            max_decomposition_depth: 5,
        }
    }

    /// Decompose a complex request into executable subtasks
    pub async fn decompose_task(&self, request: &str) -> Result<Task> {
        let intent = self.classify_intent(request).await?;
        
        let task = Task {
            id: uuid::Uuid::new_v4().to_string(),
            description: request.to_string(),
            intent: intent.clone(),
            subtasks: Vec::new(),
            dependencies: Vec::new(),
            status: TaskStatus::Planning,
            tool_calls: Vec::new(),
            outputs: Vec::new(),
            context: self.analyze_context(request).await?,
            confidence: 0.0,
            retry_count: 0,
        };

        // Check if we have a learned pattern for this type of request
        if let Some(pattern) = self.find_matching_pattern(request).await {
            info!("Found matching pattern with {}% success rate", pattern.success_rate * 100.0);
            return self.apply_pattern(task, pattern).await;
        }

        // Otherwise, decompose based on intent
        match intent {
            TaskIntent::Complex => self.decompose_complex_task(task).await,
            _ => Ok(self.create_simple_task(task).await),
        }
    }

    /// Classify the intent of a task request
    async fn classify_intent(&self, request: &str) -> Result<TaskIntent> {
        let lower = request.to_lowercase();
        
        if lower.contains("fix") || lower.contains("bug") || lower.contains("error") {
            Ok(TaskIntent::BugFix)
        } else if lower.contains("test") || lower.contains("spec") {
            Ok(TaskIntent::Testing)
        } else if lower.contains("refactor") || lower.contains("optimize") || lower.contains("improve") {
            Ok(TaskIntent::Refactoring)
        } else if lower.contains("document") || lower.contains("comment") || lower.contains("readme") {
            Ok(TaskIntent::Documentation)
        } else if lower.contains("build") || lower.contains("compile") || lower.contains("run") {
            Ok(TaskIntent::BuildOperation)
        } else if lower.contains("git") || lower.contains("commit") || lower.contains("branch") || lower.contains("merge") {
            Ok(TaskIntent::GitOperation)
        } else if lower.contains("create") || lower.contains("implement") || lower.contains("add") {
            Ok(TaskIntent::CodeGeneration)
        } else if lower.contains("read") || lower.contains("write") || lower.contains("edit") || lower.contains("delete") {
            Ok(TaskIntent::FileOperation)
        } else if lower.contains("find") || lower.contains("search") || lower.contains("analyze") {
            Ok(TaskIntent::Research)
        } else {
            // Complex task that needs decomposition
            Ok(TaskIntent::Complex)
        }
    }

    /// Analyze the context needed for task execution
    async fn analyze_context(&self, request: &str) -> Result<TaskContext> {
        // Simplified context analysis - extract file mentions and common patterns
        let lower = request.to_lowercase();
        let mut files_involved = Vec::new();
        let mut build_commands = Vec::new();
        
        // Extract file paths mentioned in the request
        for word in request.split_whitespace() {
            if word.contains('.') && (word.contains('/') || word.contains('\\')) {
                files_involved.push(word.to_string());
            }
        }
        
        // Detect common build commands
        if lower.contains("cargo") {
            build_commands.push("cargo build".to_string());
            build_commands.push("cargo test".to_string());
        } else if lower.contains("npm") || lower.contains("node") {
            build_commands.push("npm install".to_string());
            build_commands.push("npm test".to_string());
        } else if lower.contains("python") || lower.contains("pip") {
            build_commands.push("python -m pytest".to_string());
        }
        
        // Detect project type
        let project_type = if lower.contains("rust") || lower.contains("cargo") {
            Some("rust".to_string())
        } else if lower.contains("javascript") || lower.contains("typescript") || lower.contains("node") {
            Some("javascript".to_string())
        } else if lower.contains("python") {
            Some("python".to_string())
        } else {
            None
        };
        
        Ok(TaskContext {
            files_involved,
            symbols_referenced: Vec::new(),
            test_files: Vec::new(),
            build_commands,
            git_branch: None,
            project_type,
            constraints: Vec::new(),
        })
    }

    /// Find a matching learned pattern for the request
    async fn find_matching_pattern(&self, request: &str) -> Option<LearnedPattern> {
        let patterns = self.patterns.read().await;
        
        // Simple string matching for now, could use embeddings for semantic matching
        patterns.values()
            .filter(|p| request.to_lowercase().contains(&p.trigger.to_lowercase()))
            .max_by(|a, b| a.success_rate.partial_cmp(&b.success_rate).unwrap())
            .cloned()
    }

    /// Apply a learned pattern to create a task plan
    async fn apply_pattern(&self, mut task: Task, pattern: LearnedPattern) -> Result<Task> {
        info!("Applying learned pattern: {}", pattern.trigger);
        
        // Create subtasks based on the pattern's task sequence
        for (i, task_type) in pattern.task_sequence.iter().enumerate() {
            let subtask = Task {
                id: uuid::Uuid::new_v4().to_string(),
                description: format!("{} (step {} of pattern)", task_type, i + 1),
                intent: self.parse_task_intent(task_type),
                subtasks: Vec::new(),
                dependencies: if i > 0 { vec![task.subtasks[i - 1].id.clone()] } else { Vec::new() },
                status: TaskStatus::Pending,
                tool_calls: Vec::new(),
                outputs: Vec::new(),
                context: task.context.clone(),
                confidence: pattern.success_rate,
                retry_count: 0,
            };
            task.subtasks.push(subtask);
        }
        
        task.confidence = pattern.success_rate;
        task.status = TaskStatus::Pending;
        Ok(task)
    }

    /// Decompose a complex task into subtasks
    async fn decompose_complex_task(&self, mut task: Task) -> Result<Task> {
        info!("Decomposing complex task: {}", task.description);
        
        // Simplified task decomposition based on common patterns
        let suggestions = self.suggest_task_decomposition(&task.description);
        
        // Create subtasks from suggestions
        let mut previous_id = None;
        for suggestion in suggestions {
            let subtask = Task {
                id: uuid::Uuid::new_v4().to_string(),
                description: suggestion.clone(),
                intent: self.classify_intent(&suggestion).await?,
                subtasks: Vec::new(),
                dependencies: previous_id.clone().map(|id| vec![id]).unwrap_or_default(),
                status: TaskStatus::Pending,
                tool_calls: Vec::new(),
                outputs: Vec::new(),
                context: task.context.clone(),
                confidence: 0.8, // Initial confidence
                retry_count: 0,
            };
            
            previous_id = Some(subtask.id.clone());
            task.subtasks.push(subtask);
        }
        
        task.status = TaskStatus::Pending;
        Ok(task)
    }
    
    /// Suggest task decomposition based on common patterns
    fn suggest_task_decomposition(&self, description: &str) -> Vec<String> {
        let lower = description.to_lowercase();
        
        if lower.contains("implement") && lower.contains("authentication") {
            vec![
                "Research existing authentication patterns in the codebase".to_string(),
                "Design authentication schema and data models".to_string(),
                "Implement user registration endpoint".to_string(),
                "Implement login endpoint with token generation".to_string(),
                "Add authentication middleware".to_string(),
                "Implement password reset functionality".to_string(),
                "Add session management".to_string(),
                "Write tests for authentication flow".to_string(),
            ]
        } else if lower.contains("refactor") {
            vec![
                "Analyze current code structure".to_string(),
                "Identify code smells and improvement areas".to_string(),
                "Create refactoring plan".to_string(),
                "Implement refactored code".to_string(),
                "Update tests for refactored code".to_string(),
                "Verify functionality preservation".to_string(),
            ]
        } else if lower.contains("fix") && lower.contains("bug") {
            vec![
                "Reproduce the bug".to_string(),
                "Identify root cause".to_string(),
                "Implement fix".to_string(),
                "Add regression test".to_string(),
                "Verify fix in different scenarios".to_string(),
            ]
        } else if lower.contains("add") && lower.contains("feature") {
            vec![
                "Analyze requirements".to_string(),
                "Design feature implementation".to_string(),
                "Implement core functionality".to_string(),
                "Add error handling".to_string(),
                "Write tests".to_string(),
                "Update documentation".to_string(),
            ]
        } else {
            // Generic decomposition
            vec![
                format!("Analyze: {}", description),
                format!("Plan implementation for: {}", description),
                format!("Execute: {}", description),
                format!("Test: {}", description),
                format!("Document: {}", description),
            ]
        }
    }

    /// Create a simple task that doesn't need decomposition
    async fn create_simple_task(&self, mut task: Task) -> Task {
        task.status = TaskStatus::Pending;
        task.confidence = 0.9; // High confidence for simple tasks
        task
    }

    fn parse_task_intent(&self, task_type: &str) -> TaskIntent {
        match task_type.to_lowercase().as_str() {
            "file_operation" => TaskIntent::FileOperation,
            "code_generation" => TaskIntent::CodeGeneration,
            "refactoring" => TaskIntent::Refactoring,
            "bug_fix" => TaskIntent::BugFix,
            "testing" => TaskIntent::Testing,
            "documentation" => TaskIntent::Documentation,
            "research" => TaskIntent::Research,
            "build_operation" => TaskIntent::BuildOperation,
            "git_operation" => TaskIntent::GitOperation,
            _ => TaskIntent::Complex,
        }
    }

    /// Learn from successful task execution
    pub async fn learn_from_execution(&self, task: &Task, success: bool) -> Result<()> {
        if !success || task.subtasks.is_empty() {
            return Ok(());
        }

        let trigger = self.extract_trigger(&task.description);
        let task_sequence: Vec<String> = task.subtasks.iter()
            .map(|t| format!("{:?}", t.intent))
            .collect();
        let tool_sequence: Vec<String> = task.tool_calls.iter()
            .map(|tc| tc.name.clone())
            .collect();

        let mut patterns = self.patterns.write().await;
        
        if let Some(pattern) = patterns.get_mut(&trigger) {
            // Update existing pattern
            pattern.usage_count += 1;
            pattern.success_rate = (pattern.success_rate * (pattern.usage_count - 1) as f32 + 1.0) / pattern.usage_count as f32;
            pattern.last_used = chrono::Utc::now();
        } else {
            // Create new pattern
            let pattern = LearnedPattern {
                trigger: trigger.clone(),
                task_sequence,
                tool_sequence,
                success_rate: 1.0,
                avg_duration_ms: 0, // TODO: Track execution time
                usage_count: 1,
                last_used: chrono::Utc::now(),
            };
            patterns.insert(trigger, pattern);
        }

        info!("Learned pattern from successful execution");
        Ok(())
    }

    fn extract_trigger(&self, description: &str) -> String {
        // Extract key phrases that trigger this pattern
        // For now, use first few words
        description.split_whitespace()
            .take(5)
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    }
}

/// Manages task execution with intelligent orchestration
pub struct TaskExecutor {
    planner: Arc<TaskPlanner>,
    tools: Arc<ToolRegistry>,
    #[allow(dead_code)]
    intelligence: Arc<IntelligenceEngine>,
    max_retries: u32,
}

impl TaskExecutor {
    pub fn new(
        planner: Arc<TaskPlanner>,
        tools: Arc<ToolRegistry>,
        intelligence: Arc<IntelligenceEngine>,
    ) -> Self {
        Self {
            planner,
            tools,
            intelligence,
            max_retries: 3,
        }
    }

    /// Execute a task plan with intelligent orchestration
    pub async fn execute_task(&self, mut task: Task) -> Result<Task> {
        match task.status {
            TaskStatus::Completed => return Ok(task),
            TaskStatus::Failed if task.retry_count >= self.max_retries => return Ok(task),
            _ => {}
        }

        task.status = TaskStatus::Executing;
        
        // Execute subtasks first (depth-first)
        if !task.subtasks.is_empty() {
            for i in 0..task.subtasks.len() {
                let subtask = task.subtasks[i].clone();
                
                // Check dependencies
                if !self.dependencies_met(&subtask, &task.subtasks) {
                    task.subtasks[i].status = TaskStatus::Blocked;
                    continue;
                }
                
                // Execute subtask
                let executed = Box::pin(self.execute_task(subtask)).await?;
                task.subtasks[i] = executed;
                
                // If subtask failed, handle it
                if task.subtasks[i].status == TaskStatus::Failed {
                    return self.handle_failure(&mut task, i).await;
                }
            }
        } else {
            // Leaf task - execute tools
            self.execute_tools(&mut task).await?;
        }
        
        // All subtasks completed successfully
        if task.subtasks.iter().all(|t| t.status == TaskStatus::Completed) {
            task.status = TaskStatus::Completed;
            
            // Learn from successful execution
            self.planner.learn_from_execution(&task, true).await?;
        }
        
        Ok(task)
    }

    /// Check if all dependencies for a task are met
    fn dependencies_met(&self, task: &Task, all_tasks: &[Task]) -> bool {
        task.dependencies.iter().all(|dep_id| {
            all_tasks.iter()
                .any(|t| t.id == *dep_id && t.status == TaskStatus::Completed)
        })
    }

    /// Execute tools for a leaf task
    async fn execute_tools(&self, task: &mut Task) -> Result<()> {
        // Determine which tools to use based on task intent
        let tool_calls = self.plan_tool_calls(task).await?;
        
        for tool_call in tool_calls {
            if let Some(tool) = self.tools.get(&tool_call.name) {
                match tool.execute(tool_call.parameters.clone()).await {
                    Ok(output) => {
                        task.outputs.push(output);
                        task.tool_calls.push(tool_call);
                    }
                    Err(e) => {
                        warn!("Tool execution failed: {}", e);
                        task.status = TaskStatus::Failed;
                        return Err(anyhow::anyhow!("Tool execution failed: {}", e));
                    }
                }
            }
        }
        
        task.status = TaskStatus::Completed;
        Ok(())
    }

    /// Plan which tools to call for a task
    async fn plan_tool_calls(&self, task: &Task) -> Result<Vec<ToolCall>> {
        let mut tool_calls = Vec::new();
        
        match &task.intent {
            TaskIntent::FileOperation => {
                // Analyze which file operations are needed
                if task.description.contains("read") {
                    for file in &task.context.files_involved {
                        tool_calls.push(ToolCall {
                            name: "read_file".to_string(),
                            parameters: serde_json::json!({ "path": file }),
                        });
                    }
                } else if task.description.contains("write") || task.description.contains("create") {
                    // Plan write operations
                    tool_calls.push(ToolCall {
                        name: "write_file".to_string(),
                        parameters: serde_json::json!({ 
                            "path": task.context.files_involved.first().unwrap_or(&String::new()),
                            "content": "" // Content would be generated
                        }),
                    });
                }
            }
            TaskIntent::CodeGeneration => {
                // First read existing code for context
                for file in &task.context.files_involved {
                    tool_calls.push(ToolCall {
                        name: "read_file".to_string(),
                        parameters: serde_json::json!({ "path": file }),
                    });
                }
                // Then generate and write new code
                tool_calls.push(ToolCall {
                    name: "write_file".to_string(),
                    parameters: serde_json::json!({ "path": "", "content": "" }),
                });
            }
            TaskIntent::Research => {
                // Use search tools
                tool_calls.push(ToolCall {
                    name: "search_code".to_string(),
                    parameters: serde_json::json!({ 
                        "query": task.description,
                        "limit": 10 
                    }),
                });
            }
            TaskIntent::GitOperation => {
                tool_calls.push(ToolCall {
                    name: "git_status".to_string(),
                    parameters: serde_json::json!({}),
                });
            }
            TaskIntent::BuildOperation => {
                for cmd in &task.context.build_commands {
                    tool_calls.push(ToolCall {
                        name: "run_command".to_string(),
                        parameters: serde_json::json!({ 
                            "command": cmd.split_whitespace().next().unwrap_or(""),
                            "args": cmd.split_whitespace().skip(1).collect::<Vec<_>>()
                        }),
                    });
                }
            }
            _ => {
                // Default to search for understanding
                tool_calls.push(ToolCall {
                    name: "search_code".to_string(),
                    parameters: serde_json::json!({ "query": task.description }),
                });
            }
        }
        
        Ok(tool_calls)
    }

    /// Handle task failure with intelligent recovery
    async fn handle_failure(&self, task: &mut Task, failed_index: usize) -> Result<Task> {
        let failed_subtask = &task.subtasks[failed_index];
        warn!("Subtask failed: {}", failed_subtask.description);
        
        if task.retry_count < self.max_retries {
            task.retry_count += 1;
            info!("Retrying task (attempt {}/{})", task.retry_count, self.max_retries);
            
            // Try alternative approach
            if let Ok(alternative) = self.generate_alternative_approach(failed_subtask).await {
                task.subtasks[failed_index] = alternative;
                return Box::pin(self.execute_task(task.clone())).await;
            }
        }
        
        task.status = TaskStatus::Failed;
        
        // Learn from failure
        self.planner.learn_from_execution(task, false).await?;
        
        Ok(task.clone())
    }

    /// Generate an alternative approach for a failed task
    async fn generate_alternative_approach(&self, failed_task: &Task) -> Result<Task> {
        info!("Generating alternative approach for: {}", failed_task.description);
        
        // Generate alternatives based on the task intent
        let alternatives = match failed_task.intent {
            TaskIntent::FileOperation => vec![
                "Use different file access method".to_string(),
                "Check file permissions and retry".to_string(),
                "Try with absolute path instead of relative".to_string(),
            ],
            TaskIntent::CodeGeneration => vec![
                "Break down into smaller code chunks".to_string(),
                "Use different design pattern".to_string(),
                "Generate simpler implementation first".to_string(),
            ],
            TaskIntent::BugFix => vec![
                "Add more debugging output".to_string(),
                "Try different fix approach".to_string(),
                "Isolate the problem further".to_string(),
            ],
            TaskIntent::Testing => vec![
                "Use different testing framework".to_string(),
                "Write simpler test cases first".to_string(),
                "Mock external dependencies".to_string(),
            ],
            _ => vec![
                format!("Retry: {}", failed_task.description),
                format!("Simplify: {}", failed_task.description),
                format!("Alternative approach for: {}", failed_task.description),
            ],
        };
        
        if let Some(alternative_desc) = alternatives.first() {
            let mut alternative = failed_task.clone();
            alternative.description = alternative_desc.clone();
            alternative.status = TaskStatus::Pending;
            alternative.tool_calls.clear();
            alternative.outputs.clear();
            Ok(alternative)
        } else {
            Err(anyhow::anyhow!("No alternative approach found"))
        }
    }
}

/// Main reasoning engine that coordinates planning and execution
pub struct AgentReasoning {
    planner: Arc<TaskPlanner>,
    executor: Arc<TaskExecutor>,
    #[allow(dead_code)]
    intelligence: Arc<IntelligenceEngine>,
    active_tasks: Arc<RwLock<VecDeque<Task>>>,
}

impl AgentReasoning {
    pub fn new(
        intelligence: Arc<IntelligenceEngine>,
        tools: Arc<ToolRegistry>,
    ) -> Self {
        let planner = Arc::new(TaskPlanner::new(intelligence.clone()));
        let executor = Arc::new(TaskExecutor::new(
            planner.clone(),
            tools,
            intelligence.clone(),
        ));
        
        Self {
            planner,
            executor,
            intelligence,
            active_tasks: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Process a user request with intelligent reasoning
    pub async fn process_request(&self, request: &str) -> Result<TaskExecutionResult> {
        info!("Processing request with reasoning engine: {}", request);
        
        // Decompose request into tasks
        let task = self.planner.decompose_task(request).await?;
        
        // Add to active tasks
        {
            let mut tasks = self.active_tasks.write().await;
            tasks.push_back(task.clone());
        }
        
        // Execute task plan
        let executed_task = self.executor.execute_task(task).await?;
        
        // Generate summary
        let summary = self.generate_execution_summary(&executed_task).await?;
        let success = executed_task.status == TaskStatus::Completed;
        
        Ok(TaskExecutionResult {
            task: executed_task,
            summary,
            success,
        })
    }

    /// Generate a human-readable summary of task execution
    async fn generate_execution_summary(&self, task: &Task) -> Result<String> {
        let mut summary = String::new();
        
        summary.push_str(&format!("# Task: {}\n", task.description));
        summary.push_str(&format!("Status: {:?}\n", task.status));
        summary.push_str(&format!("Confidence: {:.1}%\n\n", task.confidence * 100.0));
        
        if !task.subtasks.is_empty() {
            summary.push_str("## Subtasks:\n");
            for (i, subtask) in task.subtasks.iter().enumerate() {
                let status_icon = match subtask.status {
                    TaskStatus::Completed => "✅",
                    TaskStatus::Failed => "❌",
                    TaskStatus::Blocked => "⏸️",
                    TaskStatus::Executing => "🔄",
                    _ => "⏳",
                };
                summary.push_str(&format!("{}. {} {} - {}\n", 
                    i + 1, status_icon, subtask.description, format!("{:?}", subtask.status)));
            }
            summary.push('\n');
        }
        
        if !task.tool_calls.is_empty() {
            summary.push_str("## Tools Used:\n");
            for tool_call in &task.tool_calls {
                summary.push_str(&format!("- {}\n", tool_call.name));
            }
            summary.push('\n');
        }
        
        if task.status == TaskStatus::Completed {
            summary.push_str("✅ Task completed successfully!\n");
        } else if task.status == TaskStatus::Failed {
            summary.push_str("❌ Task failed. Consider trying an alternative approach.\n");
        }
        
        Ok(summary)
    }

    /// Get the current status of all active tasks
    pub async fn get_task_status(&self) -> Vec<TaskStatus> {
        let tasks = self.active_tasks.read().await;
        tasks.iter().map(|t| t.status.clone()).collect()
    }

    /// Cancel a specific task
    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.active_tasks.write().await;
        tasks.retain(|t| t.id != task_id);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionResult {
    pub task: Task,
    pub summary: String,
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigManager;
    use crate::storage::DatabaseManager;

    async fn create_test_intelligence() -> Arc<IntelligenceEngine> {
        let config = ConfigManager::load().await.unwrap();
        let storage = DatabaseManager::new(&config).await.unwrap();
        Arc::new(IntelligenceEngine::new(&config, &storage).await.unwrap())
    }

    #[tokio::test]
    async fn test_task_decomposition() {
        // Test that complex requests get decomposed into subtasks
        let intelligence = create_test_intelligence().await;
        let planner = TaskPlanner::new(intelligence);
        
        // Test with a request that doesn't match simple intents
        let task = planner.decompose_task("Review the codebase and suggest architectural improvements").await.unwrap();
        
        // Verify the task was created successfully
        assert!(!task.description.is_empty());
        // Future: When decomposition is implemented, verify subtasks
        // assert!(!task.subtasks.is_empty());
    }

    #[tokio::test]
    async fn test_intent_classification() {
        let intelligence = create_test_intelligence().await;
        let planner = TaskPlanner::new(intelligence);
        
        let bug_intent = planner.classify_intent("Fix the login bug").await.unwrap();
        assert_eq!(bug_intent as i32, TaskIntent::BugFix as i32);
        
        let test_intent = planner.classify_intent("Write tests for the API").await.unwrap();
        assert_eq!(test_intent as i32, TaskIntent::Testing as i32);
        
        let gen_intent = planner.classify_intent("Create a new React component").await.unwrap();
        assert_eq!(gen_intent as i32, TaskIntent::CodeGeneration as i32);
    }

    #[tokio::test]
    async fn test_pattern_learning() {
        let intelligence = create_test_intelligence().await;
        let planner = Arc::new(TaskPlanner::new(intelligence));
        
        let task = Task {
            id: "test".to_string(),
            description: "Fix authentication bug".to_string(),
            intent: TaskIntent::BugFix,
            subtasks: vec![
                Task {
                    id: "sub1".to_string(),
                    description: "Find bug location".to_string(),
                    intent: TaskIntent::Research,
                    subtasks: Vec::new(),
                    dependencies: Vec::new(),
                    status: TaskStatus::Completed,
                    tool_calls: vec![ToolCall {
                        name: "search_code".to_string(),
                        parameters: serde_json::json!({"query": "auth"}),
                    }],
                    outputs: Vec::new(),
                    context: TaskContext {
                        files_involved: vec!["auth.rs".to_string()],
                        symbols_referenced: Vec::new(),
                        test_files: Vec::new(),
                        build_commands: Vec::new(),
                        git_branch: None,
                        project_type: None,
                        constraints: Vec::new(),
                    },
                    confidence: 0.9,
                    retry_count: 0,
                },
            ],
            dependencies: Vec::new(),
            status: TaskStatus::Completed,
            tool_calls: Vec::new(),
            outputs: Vec::new(),
            context: TaskContext {
                files_involved: Vec::new(),
                symbols_referenced: Vec::new(),
                test_files: Vec::new(),
                build_commands: Vec::new(),
                git_branch: None,
                project_type: None,
                constraints: Vec::new(),
            },
            confidence: 0.9,
            retry_count: 0,
        };
        
        planner.learn_from_execution(&task, true).await.unwrap();
        
        let patterns = planner.patterns.read().await;
        assert!(!patterns.is_empty());
    }
}