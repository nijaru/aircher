// SkillTool: Integrates skills into Aircher's tool registry
//
// Skills become invokable tools that guide the agent through
// skill-specific workflows using the skill's documentation.

use crate::agent::skills::metadata::SkillMetadata;
use crate::agent::tools::{AgentTool, ToolOutput, ToolError};
use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, info};

/// A tool that executes a skill
///
/// Skills guide the agent by providing instructions in their documentation.
/// The agent uses its existing tools to fulfill the skill's steps.
pub struct SkillTool {
    /// Skill metadata (name, description, parameters, etc.)
    metadata: SkillMetadata,
}

impl SkillTool {
    /// Create a new SkillTool from metadata
    pub fn new(metadata: SkillMetadata) -> Self {
        Self { metadata }
    }

    /// Validate parameters against skill schema
    fn validate_parameters(&self, params: &Value) -> Result<(), ToolError> {
        let params_obj = params
            .as_object()
            .ok_or_else(|| ToolError::InvalidParameters("Parameters must be an object".to_string()))?;

        // Check required parameters
        for param in &self.metadata.parameters {
            if param.required && !params_obj.contains_key(&param.name) {
                return Err(ToolError::InvalidParameters(format!("Missing required parameter: {}", param.name)));
            }
        }

        // TODO: Validate parameter types
        // This would check that:
        // - String params are strings
        // - Number params are numbers
        // - Enum params are valid enum values
        // For now, we trust the model to provide correct types

        Ok(())
    }

    /// Check if required capabilities are available
    ///
    /// This ensures the agent has access to the tools needed by the skill.
    /// For now, this is a placeholder - full implementation would query
    /// the tool registry to verify capabilities.
    fn check_capabilities(&self) -> Result<(), ToolError> {
        // TODO: Query tool registry to verify capabilities
        // For now, just log the required capabilities
        debug!(
            "Skill '{}' requires capabilities: {:?}",
            self.metadata.name, self.metadata.capabilities
        );

        // Placeholder: assume all capabilities are available
        // Real implementation would check tool registry

        Ok(())
    }

    /// Build enhanced prompt for skill execution
    ///
    /// This creates a prompt that includes:
    /// - Skill description
    /// - Parameters provided
    /// - Full skill documentation (instructions)
    async fn build_skill_prompt(&self, params: &Value) -> Result<String, ToolError> {
        // Load full documentation
        let docs = self
            .metadata
            .load_full_documentation()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to load documentation: {}", e)))?;

        // Build prompt that guides agent through skill execution
        let params_json = serde_json::to_string_pretty(params)
            .map_err(|e| ToolError::InvalidParameters(format!("Failed to serialize parameters: {}", e)))?;

        let prompt = format!(
            "Execute Skill: {}\n\
             \n\
             Description: {}\n\
             \n\
             Parameters:\n\
             ```json\n\
             {}\n\
             ```\n\
             \n\
             Instructions:\n\
             {}\n\
             \n\
             Please execute this skill step-by-step, using the available tools as needed. \
             Provide clear reasoning for each step and report the final result.",
            self.metadata.name,
            self.metadata.description,
            params_json,
            docs
        );

        Ok(prompt)
    }
}

#[async_trait]
impl AgentTool for SkillTool {
    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        info!("Executing skill: {}", self.metadata.name);
        if let Ok(params_str) = serde_json::to_string_pretty(&params) {
            debug!("Skill parameters: {}", params_str);
        }

        // 1. Validate parameters
        self.validate_parameters(&params)?;

        // 2. Check required capabilities
        self.check_capabilities()?;

        // 3. Build skill execution prompt
        let prompt = self.build_skill_prompt(&params).await?;

        // 4. Return skill prompt as output
        // The actual execution happens when the agent processes this prompt
        // and uses its tools to fulfill the skill's steps.
        //
        // For now, we return the prompt so it can be integrated into the
        // agent's execution flow. In Phase 2, we'll implement the full
        // SkillExecutor that handles the execution loop.

        let content = format!(
            "Skill '{}' loaded successfully. Ready for execution.\n\nPrompt:\n{}",
            self.metadata.name, prompt
        );

        Ok(ToolOutput {
            success: true,
            result: serde_json::json!({
                "skill": self.metadata.name,
                "prompt": prompt,
            }),
            error: None,
            usage: None,
        })
    }

    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn description(&self) -> &str {
        &self.metadata.description
    }

    fn parameters_schema(&self) -> Value {
        self.metadata.to_json_schema()
    }
}

/// Extension trait for registering skills as tools
pub trait SkillRegistryExt {
    /// Register all discovered skills as tools
    fn register_skills(&mut self, skills: Vec<SkillMetadata>) -> Result<(), ToolError>;

    /// Register a single skill as a tool
    fn register_skill(&mut self, skill: SkillMetadata) -> Result<(), ToolError>;
}

// Note: Actual implementation of SkillRegistryExt would be done
// in src/agent/tools/mod.rs by implementing the trait for ToolRegistry.
// This requires access to the ToolRegistry type which isn't available here.

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    async fn create_test_skill() -> SkillMetadata {
        let content = r#"---
name: test-skill
description: A test skill for unit testing
version: 1.0.0
parameters:
  - name: query
    type: string
    description: Search query
    required: true
  - name: limit
    type: number
    description: Result limit
    required: false
    default: 10
capabilities:
  - read_files
  - search_code
tags:
  - testing
---
# Test Skill

This is a test skill for validation.

## Steps

1. Parse the query parameter
2. Execute search with limit
3. Return results

## Expected Output

Returns search results matching the query.
"#;

        SkillMetadata::parse(content, PathBuf::from("test.md"))
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_skill_tool_name_and_description() {
        let metadata = create_test_skill().await;
        let skill_tool = SkillTool::new(metadata);

        assert_eq!(skill_tool.name(), "test-skill");
        assert_eq!(skill_tool.description(), "A test skill for unit testing");
    }

    #[tokio::test]
    async fn test_skill_tool_parameters_schema() {
        let metadata = create_test_skill().await;
        let skill_tool = SkillTool::new(metadata);

        let schema = skill_tool.parameters_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["query"].is_object());
        assert!(schema["properties"]["limit"].is_object());
        assert_eq!(schema["required"], serde_json::json!(["query"]));
    }

    #[tokio::test]
    async fn test_validate_parameters_valid() {
        let metadata = create_test_skill().await;
        let skill_tool = SkillTool::new(metadata);

        let params = serde_json::json!({
            "query": "test search",
            "limit": 5
        });

        let result = skill_tool.validate_parameters(&params);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_parameters_missing_required() {
        let metadata = create_test_skill().await;
        let skill_tool = SkillTool::new(metadata);

        // Missing required 'query' parameter
        let params = serde_json::json!({
            "limit": 5
        });

        let result = skill_tool.validate_parameters(&params);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing required parameter"));
    }

    #[tokio::test]
    async fn test_validate_parameters_not_object() {
        let metadata = create_test_skill().await;
        let skill_tool = SkillTool::new(metadata);

        // Parameters must be an object
        let params = serde_json::json!("not an object");

        let result = skill_tool.validate_parameters(&params);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be an object"));
    }

    #[tokio::test]
    async fn test_build_skill_prompt() {
        let metadata = create_test_skill().await;
        let skill_tool = SkillTool::new(metadata);

        let params = serde_json::json!({
            "query": "authentication",
            "limit": 10
        });

        let prompt = skill_tool.build_skill_prompt(&params).await.unwrap();

        // Verify prompt contains key elements
        assert!(prompt.contains("Execute Skill: test-skill"));
        assert!(prompt.contains("A test skill for unit testing"));
        assert!(prompt.contains("authentication"));
        assert!(prompt.contains("Test Skill")); // From documentation
        assert!(prompt.contains("## Steps")); // From documentation
    }

    #[tokio::test]
    async fn test_execute_skill_tool() {
        let metadata = create_test_skill().await;
        let skill_tool = SkillTool::new(metadata);

        let params = serde_json::json!({
            "query": "test",
            "limit": 5
        });

        let result = skill_tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success, "Expected successful tool execution");

        let content = output.result.as_str().expect("Result should be a string");
        assert!(content.contains("test-skill"));
        assert!(content.contains("loaded successfully"));
        assert!(content.contains("Prompt:"));
    }
}
