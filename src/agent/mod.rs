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
pub mod strategies; // NEW: Research-based reasoning strategies (ReAct, Reflexion, ToT, etc.)
pub mod intelligent_strategy_selection; // NEW: Intelligence-enhanced strategy selection and adaptation
// pub mod background_tasks; // NEW: Background task execution system (disabled due to compilation errors)
// pub mod runtime_validation; // NEW: Runtime validation and testing system (disabled due to compilation errors)
// pub mod orchestrator; // DEPRECATED - used sub-agents, replaced by task_orchestrator
// pub mod sub_agents; // DEPRECATED - using dynamic context instead

pub use controller::AgentController;
pub use conversation::{CodingConversation, ProjectContext};
pub use tools::{AgentTool, ToolOutput, ToolError};
pub use core::Agent;