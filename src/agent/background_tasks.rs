/// Background Task Execution System
///
/// Inspired by Cursor's 2025 background agents, this provides:
/// - Asynchronous task execution without blocking the UI
/// - Task queuing and prioritization
/// - Notification system for completion/approval requests
/// - Progress tracking and status updates

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Background task execution states
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is queued but not started
    Queued,
    /// Task is currently running
    Running {
        progress: f32, // 0.0 to 1.0
        current_step: String,
    },
    /// Task completed successfully
    Completed {
        result: TaskResult,
        duration: std::time::Duration,
    },
    /// Task failed with error
    Failed {
        error: String,
        retry_count: u32,
    },
    /// Task is paused waiting for user approval
    AwaitingApproval {
        approval_request: ApprovalRequest,
    },
    /// Task was cancelled by user
    Cancelled,
    /// Task is paused (can be resumed)
    Paused {
        reason: String,
    },
}

/// Priority levels for background tasks
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Urgent = 4,
    Critical = 5,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

/// A background task that can run independently
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundTask {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub task_type: BackgroundTaskType,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub dependencies: Vec<Uuid>, // Tasks that must complete first
    pub tags: Vec<String>,
    pub estimated_duration: Option<std::time::Duration>,
    pub max_retries: u32,
}

/// Types of background tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackgroundTaskType {
    /// Code analysis or exploration
    Analysis {
        target: AnalysisTarget,
        analysis_type: String,
    },
    /// File operations (with approval workflow)
    FileOperation {
        operation: FileOperationType,
        files: Vec<std::path::PathBuf>,
    },
    /// Build or test execution
    BuildTest {
        commands: Vec<String>,
        workspace: std::path::PathBuf,
    },
    /// Long-running refactoring
    Refactoring {
        scope: RefactoringScope,
        strategy: String,
    },
    /// Documentation generation
    Documentation {
        target: DocumentationTarget,
        format: String,
    },
    /// External tool integration
    ExternalTool {
        tool_name: String,
        arguments: Vec<String>,
    },
    /// Custom agent workflow
    AgentWorkflow {
        workflow_steps: Vec<WorkflowStep>,
        context: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisTarget {
    EntireCodebase,
    Directory(std::path::PathBuf),
    Files(Vec<std::path::PathBuf>),
    GitCommit(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileOperationType {
    Create { template: Option<String> },
    Modify { changes: Vec<String> },
    Delete { backup: bool },
    Move { destination: std::path::PathBuf },
    Rename { new_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefactoringScope {
    Function(String),
    Class(String),
    Module(std::path::PathBuf),
    Package(String),
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentationTarget {
    API,
    UserGuide,
    README,
    Changelog,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub action: String,
    pub parameters: HashMap<String, String>,
    pub depends_on: Vec<String>, // Names of previous steps
}

/// Result of a completed task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub output: String,
    pub files_created: Vec<std::path::PathBuf>,
    pub files_modified: Vec<std::path::PathBuf>,
    pub warnings: Vec<String>,
    pub metrics: HashMap<String, f64>,
}

/// Request for user approval during task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub task_id: Uuid,
    pub request_type: ApprovalType,
    pub description: String,
    pub details: String,
    pub options: Vec<ApprovalOption>,
    pub timeout: Option<std::time::Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalType {
    FileModification,
    CommandExecution,
    ResourceAccess,
    NetworkRequest,
    DataDeletion,
    ConfigChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalOption {
    pub id: String,
    pub label: String,
    pub description: String,
    pub is_destructive: bool,
}

/// Notification sent when task status changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNotification {
    pub task_id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub priority: NotificationPriority,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub actions: Vec<NotificationAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    ApprovalRequired,
    ProgressUpdate,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub id: String,
    pub label: String,
    pub action_type: ActionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Approve,
    Reject,
    ViewDetails,
    Retry,
    Cancel,
    Modify,
}

/// Background task execution manager
pub struct BackgroundTaskManager {
    tasks: Arc<RwLock<HashMap<Uuid, BackgroundTask>>>,
    task_queue: Arc<Mutex<VecDeque<Uuid>>>,
    running_tasks: Arc<Mutex<HashMap<Uuid, TaskHandle>>>,
    notification_sender: mpsc::UnboundedSender<TaskNotification>,
    approval_sender: mpsc::UnboundedSender<ApprovalRequest>,
    max_concurrent_tasks: usize,
    shutdown_signal: Arc<tokio::sync::Notify>,
}

/// Handle for a running task
struct TaskHandle {
    abort_handle: tokio::task::AbortHandle,
    progress_receiver: mpsc::UnboundedReceiver<TaskProgress>,
}

/// Progress update from a running task
#[derive(Debug, Clone)]
struct TaskProgress {
    task_id: Uuid,
    progress: f32,
    current_step: String,
    logs: Vec<String>,
}

impl BackgroundTaskManager {
    pub fn new(
        notification_sender: mpsc::UnboundedSender<TaskNotification>,
        approval_sender: mpsc::UnboundedSender<ApprovalRequest>,
    ) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            running_tasks: Arc::new(Mutex::new(HashMap::new())),
            notification_sender,
            approval_sender,
            max_concurrent_tasks: 3, // Configurable
            shutdown_signal: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Queue a new background task
    pub async fn queue_task(&self, mut task: BackgroundTask) -> Result<Uuid> {
        let task_id = task.id;
        task.status = TaskStatus::Queued;
        task.created_at = chrono::Utc::now();

        // Store the task
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id, task.clone());
        }

        // Add to queue (sorted by priority)
        {
            let mut queue = self.task_queue.lock().await;
            self.insert_by_priority(&mut queue, task_id, &task).await;
        }

        // Send notification
        self.send_notification(TaskNotification {
            task_id,
            notification_type: NotificationType::TaskStarted,
            title: format!("Task Queued: {}", task.title),
            message: format!("Task '{}' has been added to the background queue", task.title),
            priority: NotificationPriority::Normal,
            created_at: chrono::Utc::now(),
            actions: vec![
                NotificationAction {
                    id: "view".to_string(),
                    label: "View Details".to_string(),
                    action_type: ActionType::ViewDetails,
                },
                NotificationAction {
                    id: "cancel".to_string(),
                    label: "Cancel".to_string(),
                    action_type: ActionType::Cancel,
                },
            ],
        }).await?;

        // Trigger queue processing
        self.process_queue().await?;

        info!("Background task queued: {} - {}", task_id, task.title);
        Ok(task_id)
    }

    /// Insert task into queue maintaining priority order
    async fn insert_by_priority(&self, queue: &mut VecDeque<Uuid>, task_id: Uuid, task: &BackgroundTask) {
        // Find insertion point to maintain priority order
        let mut insert_index = queue.len();

        let tasks = self.tasks.read().await;
        for (i, existing_id) in queue.iter().enumerate() {
            if let Some(existing_task) = tasks.get(existing_id) {
                if task.priority > existing_task.priority {
                    insert_index = i;
                    break;
                }
            }
        }

        queue.insert(insert_index, task_id);
    }

    /// Process the task queue, starting tasks up to the concurrency limit
    async fn process_queue(&self) -> Result<()> {
        let running_count = {
            let running = self.running_tasks.lock().await;
            running.len()
        };

        if running_count >= self.max_concurrent_tasks {
            debug!("Max concurrent tasks reached ({}), queue processing deferred", self.max_concurrent_tasks);
            return Ok(());
        }

        let next_task_id = {
            let mut queue = self.task_queue.lock().await;
            queue.pop_front()
        };

        if let Some(task_id) = next_task_id {
            self.start_task(task_id).await?;
            // Recursively process more tasks if capacity allows
            self.process_queue().await?;
        }

        Ok(())
    }

    /// Start executing a specific task
    async fn start_task(&self, task_id: Uuid) -> Result<()> {
        let task = {
            let mut tasks = self.tasks.write().await;
            let task = tasks.get_mut(&task_id)
                .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;

            task.status = TaskStatus::Running {
                progress: 0.0,
                current_step: "Initializing...".to_string(),
            };
            task.started_at = Some(chrono::Utc::now());
            task.clone()
        };

        info!("Starting background task: {} - {}", task_id, task.title);

        // Create progress channel
        let (progress_tx, progress_rx) = mpsc::unbounded_channel();

        // Clone necessary data for the task
        let task_clone = task.clone();
        let tasks_arc = self.tasks.clone();
        let notification_sender = self.notification_sender.clone();
        let approval_sender = self.approval_sender.clone();

        // Spawn the task
        let abort_handle = tokio::spawn(async move {
            let result = Self::execute_task(
                task_clone,
                progress_tx,
                notification_sender.clone(),
                approval_sender,
            ).await;

            // Update task status based on result
            let mut tasks = tasks_arc.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                match result {
                    Ok(task_result) => {
                        task.status = TaskStatus::Completed {
                            result: task_result,
                            duration: chrono::Utc::now()
                                .signed_duration_since(task.started_at.unwrap_or(task.created_at))
                                .to_std()
                                .unwrap_or_default(),
                        };
                        task.completed_at = Some(chrono::Utc::now());

                        // Send completion notification
                        let _ = notification_sender.send(TaskNotification {
                            task_id,
                            notification_type: NotificationType::TaskCompleted,
                            title: format!("Task Completed: {}", task.title),
                            message: format!("Background task '{}' has completed successfully", task.title),
                            priority: NotificationPriority::Normal,
                            created_at: chrono::Utc::now(),
                            actions: vec![
                                NotificationAction {
                                    id: "view_result".to_string(),
                                    label: "View Results".to_string(),
                                    action_type: ActionType::ViewDetails,
                                },
                            ],
                        });
                    }
                    Err(error) => {
                        task.status = TaskStatus::Failed {
                            error: error.to_string(),
                            retry_count: 0,
                        };

                        // Send failure notification
                        let _ = notification_sender.send(TaskNotification {
                            task_id,
                            notification_type: NotificationType::TaskFailed,
                            title: format!("Task Failed: {}", task.title),
                            message: format!("Background task '{}' failed: {}", task.title, error),
                            priority: NotificationPriority::High,
                            created_at: chrono::Utc::now(),
                            actions: vec![
                                NotificationAction {
                                    id: "retry".to_string(),
                                    label: "Retry".to_string(),
                                    action_type: ActionType::Retry,
                                },
                                NotificationAction {
                                    id: "view_error".to_string(),
                                    label: "View Error".to_string(),
                                    action_type: ActionType::ViewDetails,
                                },
                            ],
                        });
                    }
                }
            }
        }).abort_handle();

        // Store the task handle
        {
            let mut running = self.running_tasks.lock().await;
            running.insert(task_id, TaskHandle {
                abort_handle,
                progress_receiver: progress_rx,
            });
        }

        Ok(())
    }

    /// Execute a single task (runs in background)
    async fn execute_task(
        task: BackgroundTask,
        progress_tx: mpsc::UnboundedSender<TaskProgress>,
        _notification_sender: mpsc::UnboundedSender<TaskNotification>,
        _approval_sender: mpsc::UnboundedSender<ApprovalRequest>,
    ) -> Result<TaskResult> {
        debug!("Executing task: {} - {}", task.id, task.title);

        // Send initial progress
        let _ = progress_tx.send(TaskProgress {
            task_id: task.id,
            progress: 0.0,
            current_step: "Starting execution...".to_string(),
            logs: vec!["Task execution started".to_string()],
        });

        // Simulate work based on task type
        let result = match task.task_type {
            BackgroundTaskType::Analysis { target, analysis_type } => {
                Self::execute_analysis(task.id, target, analysis_type, progress_tx).await
            }
            BackgroundTaskType::FileOperation { operation, files } => {
                Self::execute_file_operation(task.id, operation, files, progress_tx).await
            }
            BackgroundTaskType::BuildTest { commands, workspace } => {
                Self::execute_build_test(task.id, commands, workspace, progress_tx).await
            }
            BackgroundTaskType::Refactoring { scope, strategy } => {
                Self::execute_refactoring(task.id, scope, strategy, progress_tx).await
            }
            BackgroundTaskType::Documentation { target, format } => {
                Self::execute_documentation(task.id, target, format, progress_tx).await
            }
            BackgroundTaskType::ExternalTool { tool_name, arguments } => {
                Self::execute_external_tool(task.id, tool_name, arguments, progress_tx).await
            }
            BackgroundTaskType::AgentWorkflow { workflow_steps, context } => {
                Self::execute_agent_workflow(task.id, workflow_steps, context, progress_tx).await
            }
        };

        // Send completion progress
        let _ = progress_tx.send(TaskProgress {
            task_id: task.id,
            progress: 1.0,
            current_step: "Completed".to_string(),
            logs: vec!["Task execution completed".to_string()],
        });

        result
    }

    async fn execute_analysis(
        task_id: Uuid,
        _target: AnalysisTarget,
        _analysis_type: String,
        progress_tx: mpsc::UnboundedSender<TaskProgress>,
    ) -> Result<TaskResult> {
        // Placeholder implementation
        let _ = progress_tx.send(TaskProgress {
            task_id,
            progress: 0.5,
            current_step: "Analyzing codebase...".to_string(),
            logs: vec!["Analysis in progress".to_string()],
        });

        // Simulate work
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        Ok(TaskResult {
            success: true,
            output: "Analysis completed successfully".to_string(),
            files_created: vec![],
            files_modified: vec![],
            warnings: vec![],
            metrics: HashMap::new(),
        })
    }

    async fn execute_file_operation(
        task_id: Uuid,
        _operation: FileOperationType,
        _files: Vec<std::path::PathBuf>,
        progress_tx: mpsc::UnboundedSender<TaskProgress>,
    ) -> Result<TaskResult> {
        // Placeholder implementation
        let _ = progress_tx.send(TaskProgress {
            task_id,
            progress: 0.5,
            current_step: "Processing files...".to_string(),
            logs: vec!["File operation in progress".to_string()],
        });

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        Ok(TaskResult {
            success: true,
            output: "File operation completed".to_string(),
            files_created: vec![],
            files_modified: vec![],
            warnings: vec![],
            metrics: HashMap::new(),
        })
    }

    async fn execute_build_test(
        task_id: Uuid,
        _commands: Vec<String>,
        _workspace: std::path::PathBuf,
        progress_tx: mpsc::UnboundedSender<TaskProgress>,
    ) -> Result<TaskResult> {
        let _ = progress_tx.send(TaskProgress {
            task_id,
            progress: 0.7,
            current_step: "Running tests...".to_string(),
            logs: vec!["Build and test in progress".to_string()],
        });

        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        Ok(TaskResult {
            success: true,
            output: "Build and tests passed".to_string(),
            files_created: vec![],
            files_modified: vec![],
            warnings: vec![],
            metrics: HashMap::new(),
        })
    }

    async fn execute_refactoring(
        _task_id: Uuid,
        _scope: RefactoringScope,
        _strategy: String,
        _progress_tx: mpsc::UnboundedSender<TaskProgress>,
    ) -> Result<TaskResult> {
        Ok(TaskResult {
            success: true,
            output: "Refactoring completed".to_string(),
            files_created: vec![],
            files_modified: vec![],
            warnings: vec![],
            metrics: HashMap::new(),
        })
    }

    async fn execute_documentation(
        _task_id: Uuid,
        _target: DocumentationTarget,
        _format: String,
        _progress_tx: mpsc::UnboundedSender<TaskProgress>,
    ) -> Result<TaskResult> {
        Ok(TaskResult {
            success: true,
            output: "Documentation generated".to_string(),
            files_created: vec![],
            files_modified: vec![],
            warnings: vec![],
            metrics: HashMap::new(),
        })
    }

    async fn execute_external_tool(
        _task_id: Uuid,
        _tool_name: String,
        _arguments: Vec<String>,
        _progress_tx: mpsc::UnboundedSender<TaskProgress>,
    ) -> Result<TaskResult> {
        Ok(TaskResult {
            success: true,
            output: "External tool completed".to_string(),
            files_created: vec![],
            files_modified: vec![],
            warnings: vec![],
            metrics: HashMap::new(),
        })
    }

    async fn execute_agent_workflow(
        _task_id: Uuid,
        _workflow_steps: Vec<WorkflowStep>,
        _context: HashMap<String, String>,
        _progress_tx: mpsc::UnboundedSender<TaskProgress>,
    ) -> Result<TaskResult> {
        Ok(TaskResult {
            success: true,
            output: "Agent workflow completed".to_string(),
            files_created: vec![],
            files_modified: vec![],
            warnings: vec![],
            metrics: HashMap::new(),
        })
    }

    /// Get all tasks with optional filtering
    pub async fn get_tasks(&self, status_filter: Option<TaskStatus>) -> Vec<BackgroundTask> {
        let tasks = self.tasks.read().await;
        tasks.values()
            .filter(|task| {
                status_filter.as_ref().map_or(true, |filter| {
                    std::mem::discriminant(&task.status) == std::mem::discriminant(filter)
                })
            })
            .cloned()
            .collect()
    }

    /// Cancel a specific task
    pub async fn cancel_task(&self, task_id: Uuid) -> Result<()> {
        // Remove from queue if not started
        {
            let mut queue = self.task_queue.lock().await;
            queue.retain(|&id| id != task_id);
        }

        // Abort if running
        {
            let mut running = self.running_tasks.lock().await;
            if let Some(handle) = running.remove(&task_id) {
                handle.abort_handle.abort();
            }
        }

        // Update status
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.status = TaskStatus::Cancelled;
            }
        }

        info!("Background task cancelled: {}", task_id);
        Ok(())
    }

    /// Send a notification
    async fn send_notification(&self, notification: TaskNotification) -> Result<()> {
        self.notification_sender.send(notification)
            .map_err(|e| anyhow::anyhow!("Failed to send notification: {}", e))?;
        Ok(())
    }

    /// Shutdown the task manager gracefully
    pub async fn shutdown(&self) {
        info!("Shutting down background task manager");

        // Signal shutdown
        self.shutdown_signal.notify_waiters();

        // Cancel all running tasks
        let running_task_ids: Vec<Uuid> = {
            let running = self.running_tasks.lock().await;
            running.keys().cloned().collect()
        };

        for task_id in running_task_ids {
            let _ = self.cancel_task(task_id).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_queuing() {
        let (notification_tx, _notification_rx) = mpsc::unbounded_channel();
        let (approval_tx, _approval_rx) = mpsc::unbounded_channel();

        let manager = BackgroundTaskManager::new(notification_tx, approval_tx);

        let task = BackgroundTask {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            description: "A test task".to_string(),
            task_type: BackgroundTaskType::Analysis {
                target: AnalysisTarget::EntireCodebase,
                analysis_type: "test".to_string(),
            },
            priority: TaskPriority::Normal,
            status: TaskStatus::Queued,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            dependencies: vec![],
            tags: vec![],
            estimated_duration: None,
            max_retries: 3,
        };

        let task_id = manager.queue_task(task).await.unwrap();
        assert!(!task_id.is_nil());

        // Give tasks time to start
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let tasks = manager.get_tasks(None).await;
        assert_eq!(tasks.len(), 1);
    }
}