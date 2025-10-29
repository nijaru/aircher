// Agent Mode System - Plan/Build Separation (Week 7 Day 3-4)
//
// Implements OpenCode's proven pattern of separating read-only exploration (Plan)
// from modification operations (Build) for safety and clarity.
//
// References:
// - docs/architecture/SYSTEM_DESIGN_2025.md
// - Research: OpenCode uses Plan/Build separation in production

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, info};

use super::events::{AgentEvent, AgentMode};
use crate::intelligence::unified_intelligence::UserIntent;

// AgentMode is defined in events.rs to avoid circular dependency
// (since AgentEvent::ModeChanged uses AgentMode)

impl AgentMode {
    /// Get allowed tools for this mode
    pub fn allowed_tools(&self) -> HashSet<&'static str> {
        match self {
            AgentMode::Plan => {
                // Read-only tools for exploration
                vec![
                    "read_file",
                    "list_files",
                    "search_code",
                    "find_definition",
                    "find_references",
                    "analyze_code",
                    "analyze_errors",
                    // LSP queries (read-only)
                    // Knowledge graph queries (read-only)
                ]
                .into_iter()
                .collect()
            }
            AgentMode::Build => {
                // All tools available
                vec![
                    // File operations
                    "read_file",
                    "write_file",
                    "edit_file",
                    "list_files",
                    // Code understanding
                    "search_code",
                    "find_definition",
                    "find_references",
                    "analyze_code",
                    "analyze_errors",
                    // System operations (with approval)
                    "run_command",
                    "git_status",
                    // Future: test runner, debugger, etc.
                ]
                .into_iter()
                .collect()
            }
        }
    }

    /// Check if a tool is allowed in this mode
    pub fn is_tool_allowed(&self, tool_name: &str) -> bool {
        self.allowed_tools().contains(tool_name)
    }

    /// Get system prompt for this mode
    pub fn system_prompt(&self) -> &'static str {
        match self {
            AgentMode::Plan => {
                "You are in PLAN mode - a read-only exploration mode. \
                 Your goal is to understand the codebase, analyze patterns, and research solutions. \
                 You can read files, search code, and analyze structure, but you CANNOT modify files or run commands. \
                 Focus on gathering information and planning your approach. \
                 When ready to implement changes, explicitly transition to BUILD mode."
            }
            AgentMode::Build => {
                "You are in BUILD mode - full modification capabilities. \
                 You can read, write, edit files, and run commands to implement solutions. \
                 Always verify your changes and consider their impact. \
                 Use LSP diagnostics to self-correct errors before execution. \
                 Focus on precise, well-tested implementations following existing patterns."
            }
        }
    }

    // Note: can_spawn_subagents() and description() are now in events.rs
}

/// Mode classifier that determines appropriate mode from user intent
pub struct ModeClassifier;

impl ModeClassifier {
    /// Classify user intent into appropriate agent mode
    pub fn classify(intent: &UserIntent) -> AgentMode {
        match intent {
            UserIntent::CodeReading { .. } => AgentMode::Plan,
            UserIntent::ProjectExploration { .. } => AgentMode::Plan,
            UserIntent::CodeWriting { .. } => AgentMode::Build,
            UserIntent::ProjectFixing { .. } => AgentMode::Build,
            UserIntent::Mixed { primary_intent, .. } => {
                // Use primary intent for classification
                Self::classify(primary_intent)
            }
        }
    }

    /// Determine if mode transition is needed based on request
    pub fn should_transition(current_mode: AgentMode, request: &str) -> Option<AgentMode> {
        // Simple keyword-based transition detection
        // In production, this would use LLM classification

        let request_lower = request.to_lowercase();

        // Transition to Build mode if user wants to modify
        if current_mode == AgentMode::Plan {
            if request_lower.contains("implement")
                || request_lower.contains("write")
                || request_lower.contains("edit")
                || request_lower.contains("fix")
                || request_lower.contains("change")
                || request_lower.contains("modify")
                || request_lower.contains("create file")
            {
                return Some(AgentMode::Build);
            }
        }

        // Transition to Plan mode if user wants to explore
        if current_mode == AgentMode::Build {
            if request_lower.contains("analyze")
                || request_lower.contains("understand")
                || request_lower.contains("explain")
                || request_lower.contains("research")
                || request_lower.contains("explore")
                || request_lower.contains("what does")
                || request_lower.contains("how does")
            {
                return Some(AgentMode::Plan);
            }
        }

        None // No transition needed
    }
}

/// Mode transition event for logging and analytics
#[derive(Debug, Clone)]
pub struct ModeTransition {
    pub from: AgentMode,
    pub to: AgentMode,
    pub reason: String,
    pub timestamp: std::time::SystemTime,
}

impl ModeTransition {
    pub fn new(from: AgentMode, to: AgentMode, reason: String) -> Self {
        Self {
            from,
            to,
            reason,
            timestamp: std::time::SystemTime::now(),
        }
    }

    /// Create event for event bus
    pub fn to_event(&self) -> AgentEvent {
        AgentEvent::ModeChanged {
            old_mode: self.from,
            new_mode: self.to,
            reason: self.reason.clone(),
            timestamp: self.timestamp,
        }
    }

    /// Log the transition
    pub fn log(&self) {
        info!(
            "Agent mode transition: {:?} → {:?} (reason: {})",
            self.from, self.to, self.reason
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_mode_tools() {
        let mode = AgentMode::Plan;
        assert!(mode.is_tool_allowed("read_file"));
        assert!(mode.is_tool_allowed("search_code"));
        assert!(!mode.is_tool_allowed("write_file"));
        assert!(!mode.is_tool_allowed("edit_file"));
        assert!(!mode.is_tool_allowed("run_command"));
    }

    #[test]
    fn test_build_mode_tools() {
        let mode = AgentMode::Build;
        assert!(mode.is_tool_allowed("read_file"));
        assert!(mode.is_tool_allowed("write_file"));
        assert!(mode.is_tool_allowed("edit_file"));
        assert!(mode.is_tool_allowed("run_command"));
    }

    #[test]
    fn test_subagent_spawning() {
        assert!(AgentMode::Plan.can_spawn_subagents());
        assert!(!AgentMode::Build.can_spawn_subagents());
    }

    #[test]
    fn test_mode_classification() {
        use crate::intelligence::unified_intelligence::{AnalysisDepth, CodeType, UrgencyLevel, ExplorationScope};

        assert_eq!(
            ModeClassifier::classify(&UserIntent::CodeReading {
                files_mentioned: vec![],
                analysis_depth: AnalysisDepth::Surface
            }),
            AgentMode::Plan
        );

        assert_eq!(
            ModeClassifier::classify(&UserIntent::CodeWriting {
                target_files: vec![],
                code_type: CodeType::NewFeature
            }),
            AgentMode::Build
        );

        assert_eq!(
            ModeClassifier::classify(&UserIntent::ProjectExploration {
                scope: ExplorationScope::Module
            }),
            AgentMode::Plan
        );

        assert_eq!(
            ModeClassifier::classify(&UserIntent::ProjectFixing {
                error_indicators: vec![],
                urgency_level: UrgencyLevel::Medium
            }),
            AgentMode::Build
        );
    }

    #[test]
    fn test_mode_transitions() {
        // Plan → Build when user wants to modify
        assert_eq!(
            ModeClassifier::should_transition(AgentMode::Plan, "implement this feature"),
            Some(AgentMode::Build)
        );

        assert_eq!(
            ModeClassifier::should_transition(AgentMode::Plan, "fix this bug"),
            Some(AgentMode::Build)
        );

        // Build → Plan when user wants to explore
        assert_eq!(
            ModeClassifier::should_transition(AgentMode::Build, "analyze this code"),
            Some(AgentMode::Plan)
        );

        assert_eq!(
            ModeClassifier::should_transition(AgentMode::Build, "explain how this works"),
            Some(AgentMode::Plan)
        );

        // No transition needed
        assert_eq!(
            ModeClassifier::should_transition(AgentMode::Plan, "search for auth code"),
            None
        );

        assert_eq!(
            ModeClassifier::should_transition(AgentMode::Build, "write the tests"),
            None
        );
    }

    #[test]
    fn test_mode_descriptions() {
        assert!(!AgentMode::Plan.description().is_empty());
        assert!(!AgentMode::Build.description().is_empty());
        assert!(!AgentMode::Plan.system_prompt().is_empty());
        assert!(!AgentMode::Build.system_prompt().is_empty());
    }
}
