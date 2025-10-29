pub mod tools;
pub mod controller;
pub mod conversation;
pub mod parser;
pub mod tool_formatter;
pub mod streaming;
pub mod core;
pub mod reasoning;
pub mod dynamic_context;
pub mod context_engine;
pub mod task_orchestrator; // NEW: Context-engineered orchestrator
pub mod enhanced_context_analyzer; // NEW: Enhanced semantic context analysis
pub mod approval_modes; // NEW: Approval modes and change management system
pub mod plan_mode; // NEW: Plan Mode for safe code exploration
pub mod multi_turn_reasoning; // NEW: Real multi-turn reasoning engine for systematic problem solving
pub mod enhanced_prompting; // NEW: Research-based enhanced prompting (replaces complex orchestration)
pub mod strategies; // NEW: Research-based reasoning strategies (ReAct, Reflexion, ToT, etc.)
pub mod intelligent_strategy_selection; // NEW: Intelligence-enhanced strategy selection and adaptation
pub mod events; // NEW (Week 7): Event bus system for agent-wide communication
pub mod lsp_manager; // NEW (Week 7): LSP manager with global diagnostics + event integration
pub mod agent_mode; // NEW (Week 7 Day 3-4): Plan/Build mode separation
// pub mod background_tasks; // NEW: Background task execution system (disabled due to compilation errors)
// pub mod runtime_validation; // NEW: Runtime validation and testing system (disabled due to compilation errors)
// pub mod orchestrator; // DEPRECATED - used sub-agents, replaced by task_orchestrator
// pub mod sub_agents; // DEPRECATED - using dynamic context instead

pub use controller::AgentController;
pub use conversation::{CodingConversation, ProjectContext};
pub use tools::{AgentTool, ToolOutput, ToolError};
pub use core::Agent;
pub use events::{EventBus, EventListener, AgentEvent, AgentMode, SharedEventBus, create_event_bus};
pub use lsp_manager::LspManager;
pub use agent_mode::{ModeClassifier, ModeTransition};