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
// pub mod orchestrator; // DEPRECATED - used sub-agents, replaced by task_orchestrator
// pub mod sub_agents; // DEPRECATED - using dynamic context instead

pub use controller::AgentController;
pub use conversation::{CodingConversation, ProjectContext};
pub use tools::{AgentTool, ToolOutput, ToolError};
pub use core::Agent;