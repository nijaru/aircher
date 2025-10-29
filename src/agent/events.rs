// Event Bus System for Agent Communication
//
// Implements tokio::sync::broadcast-based event bus for real-time communication
// between agent subsystems (LSP, file operations, tests, tools).
//
// Week 7 Day 1-2: Event Bus + LSP Integration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, warn};

/// Global event channel capacity
const EVENT_CHANNEL_CAPACITY: usize = 1000;

/// Events that flow through the agent system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    /// File was modified (triggers LSP notification)
    FileChanged {
        path: PathBuf,
        operation: FileOperation,
        timestamp: std::time::SystemTime,
    },

    /// LSP diagnostics received for a file
    DiagnosticsReceived {
        path: PathBuf,
        diagnostics: Vec<Diagnostic>,
        timestamp: std::time::SystemTime,
    },

    /// Test execution completed
    TestResults {
        test_suite: String,
        passed: usize,
        failed: usize,
        errors: Vec<TestError>,
        timestamp: std::time::SystemTime,
    },

    /// Tool execution event
    ToolExecuted {
        tool_name: String,
        success: bool,
        duration_ms: u64,
        context: Option<String>,
        timestamp: std::time::SystemTime,
    },

    /// Agent mode changed (Plan/Build)
    ModeChanged {
        old_mode: AgentMode,
        new_mode: AgentMode,
        reason: String,
        timestamp: std::time::SystemTime,
    },

    /// Git snapshot created
    SnapshotCreated {
        snapshot_id: String,
        message: String,
        files_changed: usize,
        timestamp: std::time::SystemTime,
    },

    /// Git snapshot rolled back
    SnapshotRolledBack {
        snapshot_id: String,
        reason: String,
        timestamp: std::time::SystemTime,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileOperation {
    Read,
    Write,
    Edit,
    Delete,
    Create,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: DiagnosticRange,
    pub severity: DiagnosticSeverity,
    pub code: Option<String>,
    pub message: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRange {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

impl DiagnosticSeverity {
    pub fn from_lsp(severity: u32) -> Self {
        match severity {
            1 => DiagnosticSeverity::Error,
            2 => DiagnosticSeverity::Warning,
            3 => DiagnosticSeverity::Information,
            4 => DiagnosticSeverity::Hint,
            _ => DiagnosticSeverity::Information,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestError {
    pub test_name: String,
    pub error_message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum AgentMode {
    /// Read-only exploration mode (can spawn research sub-agents)
    Plan,
    /// Modification mode (all tools, never uses sub-agents)
    Build,
}

impl AgentMode {
    /// Can this mode spawn sub-agents for parallel research?
    pub fn can_spawn_subagents(&self) -> bool {
        match self {
            AgentMode::Plan => true,  // Can spawn research sub-agents
            AgentMode::Build => false, // NEVER spawn sub-agents (15x waste)
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            AgentMode::Plan => "Read-only exploration and analysis",
            AgentMode::Build => "Full modification and implementation",
        }
    }
}

/// Event bus for agent-wide communication
pub struct EventBus {
    sender: broadcast::Sender<AgentEvent>,
}

impl EventBus {
    /// Create new event bus
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self { sender }
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: AgentEvent) {
        match self.sender.send(event.clone()) {
            Ok(receivers) => {
                debug!("Published event to {} receivers: {:?}", receivers, event);
            }
            Err(e) => {
                // This happens when no receivers are active (not an error)
                debug!("No receivers for event: {:?} (error: {})", event, e);
            }
        }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.sender.subscribe()
    }

    /// Get the sender for direct use
    pub fn sender(&self) -> broadcast::Sender<AgentEvent> {
        self.sender.clone()
    }

    /// Get number of active receivers
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

/// Event listener that can filter and handle specific events
pub struct EventListener {
    receiver: broadcast::Receiver<AgentEvent>,
}

impl EventListener {
    /// Create new listener from event bus
    pub fn new(event_bus: &EventBus) -> Self {
        Self {
            receiver: event_bus.subscribe(),
        }
    }

    /// Wait for next event
    pub async fn next(&mut self) -> Option<AgentEvent> {
        match self.receiver.recv().await {
            Ok(event) => Some(event),
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                warn!("Event listener lagged, skipped {} events", skipped);
                // Try to get the next event after lag
                self.receiver.recv().await.ok()
            }
            Err(broadcast::error::RecvError::Closed) => None,
        }
    }

    /// Wait for specific event type with timeout
    pub async fn wait_for<F>(&mut self, predicate: F, timeout: std::time::Duration) -> Option<AgentEvent>
    where
        F: Fn(&AgentEvent) -> bool,
    {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if let Some(event) = self.next().await {
                if predicate(&event) {
                    return Some(event);
                }
            }
        }

        None
    }

    /// Filter events by predicate
    pub async fn filter<F>(&mut self, predicate: F) -> Vec<AgentEvent>
    where
        F: Fn(&AgentEvent) -> bool,
    {
        let mut matching_events = Vec::new();

        // Non-blocking drain of current events
        while let Ok(event) = self.receiver.try_recv() {
            if predicate(&event) {
                matching_events.push(event);
            }
        }

        matching_events
    }
}

/// Shared event bus instance for the agent
pub type SharedEventBus = Arc<EventBus>;

/// Create a shared event bus
pub fn create_event_bus() -> SharedEventBus {
    Arc::new(EventBus::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::new();
        let mut listener = EventListener::new(&bus);

        let event = AgentEvent::ToolExecuted {
            tool_name: "test_tool".to_string(),
            success: true,
            duration_ms: 100,
            context: None,
            timestamp: std::time::SystemTime::now(),
        };

        bus.publish(event.clone());

        let received = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            listener.next()
        ).await;

        assert!(received.is_ok());
        let received_event = received.unwrap();
        assert!(received_event.is_some());
    }

    #[tokio::test]
    async fn test_event_listener_wait_for() {
        let bus = EventBus::new();
        let mut listener = EventListener::new(&bus);

        // Spawn task to publish event after delay
        let bus_clone = bus.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            bus_clone.publish(AgentEvent::FileChanged {
                path: PathBuf::from("/test.rs"),
                operation: FileOperation::Edit,
                timestamp: std::time::SystemTime::now(),
            });
        });

        let result = listener.wait_for(
            |event| matches!(event, AgentEvent::FileChanged { .. }),
            std::time::Duration::from_millis(200),
        ).await;

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_event_listener_filter() {
        let bus = EventBus::new();
        let mut listener = EventListener::new(&bus);

        // Publish multiple events
        bus.publish(AgentEvent::ToolExecuted {
            tool_name: "tool1".to_string(),
            success: true,
            duration_ms: 100,
            context: None,
            timestamp: std::time::SystemTime::now(),
        });

        bus.publish(AgentEvent::FileChanged {
            path: PathBuf::from("/test.rs"),
            operation: FileOperation::Edit,
            timestamp: std::time::SystemTime::now(),
        });

        bus.publish(AgentEvent::ToolExecuted {
            tool_name: "tool2".to_string(),
            success: false,
            duration_ms: 50,
            context: None,
            timestamp: std::time::SystemTime::now(),
        });

        // Small delay to ensure events are received
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let tool_events = listener.filter(|event| {
            matches!(event, AgentEvent::ToolExecuted { .. })
        }).await;

        assert_eq!(tool_events.len(), 2);
    }

    #[tokio::test]
    async fn test_diagnostics_severity() {
        assert_eq!(DiagnosticSeverity::from_lsp(1), DiagnosticSeverity::Error);
        assert_eq!(DiagnosticSeverity::from_lsp(2), DiagnosticSeverity::Warning);
        assert_eq!(DiagnosticSeverity::from_lsp(3), DiagnosticSeverity::Information);
        assert_eq!(DiagnosticSeverity::from_lsp(4), DiagnosticSeverity::Hint);
        assert_eq!(DiagnosticSeverity::from_lsp(999), DiagnosticSeverity::Information);
    }

    #[test]
    fn test_event_bus_clone() {
        let bus1 = EventBus::new();
        let bus2 = bus1.clone();

        assert_eq!(bus1.receiver_count(), bus2.receiver_count());
    }

    #[test]
    fn test_create_shared_event_bus() {
        let bus = create_event_bus();
        assert_eq!(bus.receiver_count(), 0);
    }
}
