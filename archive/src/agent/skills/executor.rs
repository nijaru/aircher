// Skill execution engine
//
// Handles skill execution with approval workflow, capability checking,
// and integration with the Agent's execution system.

use crate::agent::skills::metadata::SkillMetadata;
use crate::agent::tools::{ToolOutput, ToolError};
use crate::agent::approval_modes::PendingChange;
use anyhow::Result;
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, info, warn};

/// Context for skill execution
#[derive(Debug, Clone)]
pub struct SkillContext {
    /// Skill being executed
    pub skill_name: String,

    /// Parameters provided for execution
    pub parameters: Value,

    /// Available tools for this skill
    pub available_tools: Vec<String>,
}

/// Skill executor that handles skill invocation with approval workflow
///
/// Phase 2: Simplified implementation that creates execution prompt
/// and integrates with existing approval system.
///
/// Future: Full execution loop with agent feedback (when Agent.execute_with_instructions exists)
pub struct SkillExecutor {
    /// Channel to send approval requests
    approval_tx: Option<UnboundedSender<PendingChange>>,
}

impl SkillExecutor {
    /// Create a new SkillExecutor without approval workflow
    pub fn new() -> Self {
        Self {
            approval_tx: None,
        }
    }

    /// Create a SkillExecutor with approval workflow
    pub fn with_approval(approval_tx: UnboundedSender<PendingChange>) -> Self {
        Self {
            approval_tx: Some(approval_tx),
        }
    }

    /// Execute a skill with the given parameters
    ///
    /// Returns a ToolOutput containing the skill execution prompt.
    /// The actual execution happens when the agent processes this prompt
    /// using its existing tools.
    pub async fn execute_skill(
        &self,
        metadata: &SkillMetadata,
        params: Value,
        instructions: &str,
    ) -> Result<ToolOutput, ToolError> {
        info!("Executing skill: {}", metadata.name);

        // 1. Create skill execution context
        let context = SkillContext {
            skill_name: metadata.name.clone(),
            parameters: params.clone(),
            available_tools: self.get_available_tools(&metadata.capabilities),
        };

        debug!(
            "Skill context: {} tools available for {} capabilities",
            context.available_tools.len(),
            metadata.capabilities.len()
        );

        // 2. Request approval if skill requires dangerous operations
        if self.requires_approval(metadata) {
            self.request_skill_approval(metadata, &params).await?;
        }

        // 3. Build execution response
        // For now, return the prompt so the agent can execute the skill
        // In the future, this would invoke Agent.execute_with_instructions()
        let result = serde_json::json!({
            "skill": metadata.name,
            "status": "ready_for_execution",
            "instructions": instructions,
            "context": {
                "parameters": params,
                "available_tools": context.available_tools,
            }
        });

        Ok(ToolOutput {
            success: true,
            result,
            error: None,
            usage: None,
        })
    }

    /// Check if skill requires approval based on its capabilities
    fn requires_approval(&self, metadata: &SkillMetadata) -> bool {
        // No approval channel = no approval needed
        if self.approval_tx.is_none() {
            return false;
        }

        // Check if skill capabilities include dangerous operations
        metadata.capabilities.iter().any(|cap| {
            matches!(
                cap.as_str(),
                "run_commands"
                    | "write_files"
                    | "edit_files"
                    | "delete_files"
                    | "network_access"
                    | "file_system_access"
            )
        })
    }

    /// Request approval for skill execution
    async fn request_skill_approval(
        &self,
        metadata: &SkillMetadata,
        params: &Value,
    ) -> Result<(), ToolError> {
        let Some(approval_tx) = &self.approval_tx else {
            // No approval channel, skip approval
            return Ok(());
        };

        let params_str = serde_json::to_string_pretty(params)
            .unwrap_or_else(|_| params.to_string());

        let description = format!(
            "Execute skill '{}' (v{}) with parameters:\n{}",
            metadata.name, metadata.version, params_str
        );

        // Create a pending change for skill execution
        // Using RunCommand as a placeholder since skills invoke multiple tools
        let change = PendingChange::new(
            crate::agent::approval_modes::ChangeType::RunCommand {
                command: format!("skill:{}", metadata.name),
                cwd: None,
            },
            format!("skill:{}", metadata.name),
            description,
        );

        // Send approval request
        approval_tx
            .send(change)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to request approval: {}", e)))?;

        info!("Approval requested for skill: {}", metadata.name);

        Ok(())
    }

    /// Get available tools based on capabilities
    ///
    /// Maps skill capabilities to actual tool names.
    /// This enables capability-based access control.
    fn get_available_tools(&self, capabilities: &[String]) -> Vec<String> {
        let mut tools = Vec::new();

        for capability in capabilities {
            match capability.as_str() {
                // File operations
                "read_files" => {
                    tools.push("read_file".to_string());
                    tools.push("list_files".to_string());
                }
                "write_files" => {
                    tools.push("write_file".to_string());
                }
                "edit_files" => {
                    tools.push("edit_file".to_string());
                }
                "delete_files" => {
                    tools.push("delete_file".to_string());
                }

                // Code operations
                "search_code" => {
                    tools.push("search_code".to_string());
                }
                "analyze_code" => {
                    tools.push("analyze_code".to_string());
                }
                "find_definition" => {
                    tools.push("find_definition".to_string());
                }
                "find_references" => {
                    tools.push("find_references".to_string());
                }

                // System operations
                "run_commands" => {
                    tools.push("run_command".to_string());
                }
                "git_operations" => {
                    tools.push("git_status".to_string());
                    tools.push("git_diff".to_string());
                    tools.push("git_log".to_string());
                }

                // Semantic search
                "semantic_search" => {
                    tools.push("semantic_search".to_string());
                }

                // Network operations (future)
                "network_access" => {
                    warn!("Network access capability requested but not yet implemented");
                }

                // File system (comprehensive access)
                "file_system_access" => {
                    tools.push("read_file".to_string());
                    tools.push("write_file".to_string());
                    tools.push("edit_file".to_string());
                    tools.push("list_files".to_string());
                }

                _ => {
                    warn!("Unknown capability requested: {}", capability);
                }
            }
        }

        // Deduplicate tools
        tools.sort();
        tools.dedup();

        tools
    }
}

impl Default for SkillExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    async fn create_test_skill() -> SkillMetadata {
        let content = r#"---
name: test-skill
description: A test skill
version: 1.0.0
capabilities:
  - read_files
  - write_files
parameters:
  - name: target
    type: string
    description: Target file
    required: true
---
# Test Skill
Instructions for test skill.
"#;

        SkillMetadata::parse(content, PathBuf::from("test.md"))
            .await
            .unwrap()
    }

    async fn create_safe_skill() -> SkillMetadata {
        let content = r#"---
name: safe-skill
description: A safe skill (read-only)
version: 1.0.0
capabilities:
  - read_files
  - search_code
parameters:
  - name: query
    type: string
    description: Search query
    required: true
---
# Safe Skill
Read-only operations.
"#;

        SkillMetadata::parse(content, PathBuf::from("safe.md"))
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_skill_executor_creation() {
        let executor = SkillExecutor::new();
        assert!(executor.approval_tx.is_none());

        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let executor = SkillExecutor::with_approval(tx);
        assert!(executor.approval_tx.is_some());
    }

    #[tokio::test]
    async fn test_requires_approval_dangerous_skill() {
        let executor = SkillExecutor::new();
        let skill = create_test_skill().await;

        // No approval channel = no approval required
        assert!(!executor.requires_approval(&skill));

        // With approval channel = approval required for dangerous capabilities
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let executor = SkillExecutor::with_approval(tx);
        assert!(executor.requires_approval(&skill)); // Has write_files capability
    }

    #[tokio::test]
    async fn test_requires_approval_safe_skill() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let executor = SkillExecutor::with_approval(tx);
        let skill = create_safe_skill().await;

        // Read-only capabilities don't require approval
        assert!(!executor.requires_approval(&skill));
    }

    #[tokio::test]
    async fn test_get_available_tools() {
        let executor = SkillExecutor::new();

        // Test read capabilities
        let caps = vec!["read_files".to_string()];
        let tools = executor.get_available_tools(&caps);
        assert!(tools.contains(&"read_file".to_string()));
        assert!(tools.contains(&"list_files".to_string()));

        // Test write capabilities
        let caps = vec!["write_files".to_string()];
        let tools = executor.get_available_tools(&caps);
        assert!(tools.contains(&"write_file".to_string()));

        // Test code capabilities
        let caps = vec!["search_code".to_string(), "analyze_code".to_string()];
        let tools = executor.get_available_tools(&caps);
        assert!(tools.contains(&"search_code".to_string()));
        assert!(tools.contains(&"analyze_code".to_string()));

        // Test deduplication
        let caps = vec![
            "read_files".to_string(),
            "file_system_access".to_string(),
        ];
        let tools = executor.get_available_tools(&caps);
        // Should not have duplicates
        let unique_count = tools.len();
        let mut sorted = tools.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(unique_count, sorted.len());
    }

    #[tokio::test]
    async fn test_execute_skill_without_approval() {
        let executor = SkillExecutor::new();
        let skill = create_test_skill().await;
        let params = serde_json::json!({
            "target": "test.rs"
        });
        let instructions = "Execute test skill";

        let result = executor
            .execute_skill(&skill, params.clone(), instructions)
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.result["skill"], "test-skill");
        assert_eq!(output.result["status"], "ready_for_execution");
        assert_eq!(output.result["instructions"], instructions);
        assert_eq!(output.result["context"]["parameters"], params);
    }

    #[tokio::test]
    async fn test_skill_context_creation() {
        let context = SkillContext {
            skill_name: "test-skill".to_string(),
            parameters: serde_json::json!({"foo": "bar"}),
            available_tools: vec!["read_file".to_string(), "write_file".to_string()],
        };

        assert_eq!(context.skill_name, "test-skill");
        assert_eq!(context.available_tools.len(), 2);
        assert!(context.available_tools.contains(&"read_file".to_string()));
    }
}
