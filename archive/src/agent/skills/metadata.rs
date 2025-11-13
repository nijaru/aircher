// Skill metadata parsing and progressive loading
//
// Skills are defined in SKILL.md files with YAML frontmatter.
// This module handles parsing the frontmatter and loading documentation on-demand.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::OnceCell;

/// Metadata for a skill, parsed from YAML frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    /// Unique skill identifier (kebab-case)
    pub name: String,

    /// One-line description for model selection
    pub description: String,

    /// Semantic version (e.g., "1.0.0")
    pub version: String,

    /// Optional skill creator
    #[serde(default)]
    pub author: Option<String>,

    /// Input parameters (JSON schema style)
    #[serde(default)]
    pub parameters: Vec<ParameterSchema>,

    /// Required Aircher capabilities
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Categorization tags for discovery
    #[serde(default)]
    pub tags: Vec<String>,

    /// Quick examples for model
    #[serde(default)]
    pub examples: Vec<SkillExample>,

    /// MCP tools this skill uses
    #[serde(default)]
    pub mcp_tools: Vec<McpToolRef>,

    /// Path to the SKILL.md file
    #[serde(skip)]
    pub file_path: PathBuf,

    /// Full documentation (loaded on-demand)
    #[serde(skip)]
    full_documentation: OnceCell<String>,
}

/// Parameter schema for skill inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    /// Parameter name
    pub name: String,

    /// Parameter type (string, number, boolean, array, object)
    #[serde(rename = "type")]
    pub param_type: ParameterType,

    /// Parameter description
    pub description: String,

    /// Whether parameter is required
    #[serde(default)]
    pub required: bool,

    /// Default value if not provided
    #[serde(default)]
    pub default: Option<serde_json::Value>,

    /// Valid values (for enums)
    #[serde(default)]
    pub r#enum: Option<Vec<String>>,
}

/// Parameter types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

/// Example skill invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExample {
    /// Example input parameters
    pub input: serde_json::Value,

    /// Expected output
    pub output: String,
}

/// Reference to an MCP tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolRef {
    /// Tool name
    pub name: String,

    /// MCP server providing the tool
    pub server: String,
}

impl SkillMetadata {
    /// Parse skill metadata from SKILL.md file
    ///
    /// This loads only the YAML frontmatter, not the full documentation.
    /// The documentation is loaded on-demand when needed.
    pub async fn from_file(path: PathBuf) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .await
            .with_context(|| format!("Failed to read skill file: {}", path.display()))?;

        Self::parse(&content, path).await
    }

    /// Parse skill metadata from string content
    pub async fn parse(content: &str, file_path: PathBuf) -> Result<Self> {
        // Extract YAML frontmatter
        let frontmatter = Self::extract_frontmatter(content)?;

        // Parse YAML
        let mut metadata: SkillMetadata = serde_yaml::from_str(&frontmatter)
            .with_context(|| format!("Failed to parse YAML frontmatter in {}", file_path.display()))?;

        metadata.file_path = file_path;

        // Validate required fields
        metadata.validate()?;

        Ok(metadata)
    }

    /// Extract YAML frontmatter from markdown content
    ///
    /// Frontmatter is delimited by `---` markers:
    /// ```markdown
    /// ---
    /// name: skill_name
    /// description: Description
    /// ---
    /// # Documentation
    /// ```
    fn extract_frontmatter(content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();

        // Check for opening delimiter
        if lines.is_empty() || !lines[0].trim().starts_with("---") {
            anyhow::bail!("Missing YAML frontmatter opening delimiter (---)")
        }

        // Find closing delimiter
        let closing_index = lines[1..]
            .iter()
            .position(|line| line.trim().starts_with("---"))
            .ok_or_else(|| anyhow::anyhow!("Missing YAML frontmatter closing delimiter (---)"))?
            + 1;

        // Extract frontmatter (excluding delimiters)
        let frontmatter = lines[1..closing_index].join("\n");

        Ok(frontmatter)
    }

    /// Validate required fields
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            anyhow::bail!("Skill name cannot be empty");
        }

        if self.description.is_empty() {
            anyhow::bail!("Skill description cannot be empty");
        }

        if self.version.is_empty() {
            anyhow::bail!("Skill version cannot be empty");
        }

        // Validate name is kebab-case (lowercase with hyphens)
        if !self.name.chars().all(|c| c.is_lowercase() || c == '-' || c.is_ascii_digit()) {
            anyhow::bail!("Skill name must be kebab-case (lowercase with hyphens)");
        }

        Ok(())
    }

    /// Load full documentation for the skill
    ///
    /// This is called on-demand when the skill is invoked.
    /// The documentation is cached after the first load.
    pub async fn load_full_documentation(&self) -> Result<&str> {
        self.full_documentation
            .get_or_try_init(|| async {
                let content = fs::read_to_string(&self.file_path)
                    .await
                    .with_context(|| format!("Failed to read skill file: {}", self.file_path.display()))?;

                // Extract markdown body (skip frontmatter)
                let body = Self::extract_markdown_body(&content)?;

                Ok(body)
            })
            .await
            .map(|s| s.as_str())
    }

    /// Extract markdown body from content (excluding YAML frontmatter)
    fn extract_markdown_body(content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();

        // Find closing frontmatter delimiter
        let closing_index = lines[1..]
            .iter()
            .position(|line| line.trim().starts_with("---"))
            .ok_or_else(|| anyhow::anyhow!("Missing YAML frontmatter closing delimiter"))?
            + 1;

        // Extract body (after closing delimiter)
        let body = lines[(closing_index + 1)..]
            .join("\n")
            .trim()
            .to_string();

        Ok(body)
    }

    /// Convert to JSON schema for tool registration
    pub fn to_json_schema(&self) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for param in &self.parameters {
            let mut param_schema = serde_json::Map::new();

            // Type
            param_schema.insert(
                "type".to_string(),
                serde_json::Value::String(match param.param_type {
                    ParameterType::String => "string",
                    ParameterType::Number => "number",
                    ParameterType::Boolean => "boolean",
                    ParameterType::Array => "array",
                    ParameterType::Object => "object",
                }.to_string()),
            );

            // Description
            param_schema.insert(
                "description".to_string(),
                serde_json::Value::String(param.description.clone()),
            );

            // Enum values
            if let Some(enum_values) = &param.r#enum {
                param_schema.insert(
                    "enum".to_string(),
                    serde_json::Value::Array(
                        enum_values
                            .iter()
                            .map(|v| serde_json::Value::String(v.clone()))
                            .collect(),
                    ),
                );
            }

            // Default value
            if let Some(default) = &param.default {
                param_schema.insert("default".to_string(), default.clone());
            }

            properties.insert(param.name.clone(), serde_json::Value::Object(param_schema));

            if param.required {
                required.push(param.name.clone());
            }
        }

        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_parse_valid_skill() {
        let content = r#"---
name: test_skill
description: A test skill for unit testing
version: 1.0.0
author: aircher
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
  - example
---
# Test Skill

This is the documentation for the test skill.

## Usage

Call this skill when you need to test something.
"#;

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let metadata = SkillMetadata::parse(content, path).await.unwrap();

        assert_eq!(metadata.name, "test_skill");
        assert_eq!(metadata.description, "A test skill for unit testing");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.author, Some("aircher".to_string()));
        assert_eq!(metadata.parameters.len(), 2);
        assert_eq!(metadata.capabilities.len(), 2);
        assert_eq!(metadata.tags.len(), 2);
    }

    #[tokio::test]
    async fn test_parse_missing_frontmatter() {
        let content = "# Test Skill\n\nNo frontmatter here.";
        let path = PathBuf::from("test.md");

        let result = SkillMetadata::parse(content, path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_name_format() {
        let content = r#"---
name: InvalidName
description: Bad name format
version: 1.0.0
---
# Test
"#;
        let path = PathBuf::from("test.md");

        let result = SkillMetadata::parse(content, path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("kebab-case"));
    }

    #[tokio::test]
    async fn test_progressive_loading() {
        let content = r#"---
name: test-skill
description: Test
version: 1.0.0
---
# Full Documentation

This is the full documentation that should be loaded on-demand.

## Section 1

Content here.
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file.path(), content).await.unwrap();

        let metadata = SkillMetadata::from_file(temp_file.path().to_path_buf())
            .await
            .unwrap();

        // Documentation not loaded yet
        assert!(metadata.full_documentation.get().is_none());

        // Load documentation
        let docs = metadata.load_full_documentation().await.unwrap();
        assert!(docs.contains("Full Documentation"));
        assert!(docs.contains("Section 1"));

        // Documentation now cached
        assert!(metadata.full_documentation.get().is_some());
    }

    #[tokio::test]
    async fn test_to_json_schema() {
        let content = r#"---
name: test-skill
description: Test
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
  - name: format
    type: string
    description: Output format
    required: false
    enum: [json, yaml, text]
---
# Test
"#;
        let path = PathBuf::from("test.md");
        let metadata = SkillMetadata::parse(content, path).await.unwrap();

        let schema = metadata.to_json_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["query"].is_object());
        assert!(schema["properties"]["limit"].is_object());
        assert!(schema["properties"]["format"].is_object());
        assert_eq!(schema["required"], serde_json::json!(["query"]));

        // Check enum
        assert!(schema["properties"]["format"]["enum"].is_array());
    }
}
