use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tokio::process::Command as TokioCommand;
use tracing::{debug, info, warn};

use crate::agent::tools::{AgentTool, ToolError, ToolOutput};
use crate::intelligence::IntelligenceEngine;

/// Smart Git commit tool that generates intelligent commit messages
pub struct SmartCommitTool {
    workspace_root: PathBuf,
    intelligence: Option<Arc<IntelligenceEngine>>,
}

impl SmartCommitTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            intelligence: None,
        }
    }
    
    pub fn with_intelligence(mut self, intelligence: Arc<IntelligenceEngine>) -> Self {
        self.intelligence = Some(intelligence);
        self
    }
    
    /// Analyze changes and generate commit message
    async fn analyze_changes(&self) -> Result<(String, Vec<String>)> {
        // Get git diff
        let diff_output = TokioCommand::new("git")
            .arg("diff")
            .arg("--cached")
            .arg("--stat")
            .current_dir(&self.workspace_root)
            .output()
            .await?;
        
        let stats = String::from_utf8_lossy(&diff_output.stdout);
        
        // Get detailed changes
        let changes_output = TokioCommand::new("git")
            .arg("diff")
            .arg("--cached")
            .arg("--name-status")
            .current_dir(&self.workspace_root)
            .output()
            .await?;
        
        let changed_files: Vec<String> = String::from_utf8_lossy(&changes_output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();
        
        // Analyze the type of changes
        let mut commit_type = "chore";
        let mut scope = "";
        
        for file in &changed_files {
            if file.contains("test") {
                commit_type = "test";
                scope = "tests";
            } else if file.contains("src/") {
                if file.contains("agent/") {
                    commit_type = "feat";
                    scope = "agent";
                } else if file.contains("ui/") {
                    commit_type = "feat";
                    scope = "ui";
                } else if file.contains("providers/") {
                    commit_type = "feat";
                    scope = "providers";
                }
            } else if file.contains("docs/") {
                commit_type = "docs";
                scope = "documentation";
            } else if file.contains("Cargo.toml") {
                commit_type = "build";
                scope = "deps";
            }
        }
        
        // Generate intelligent commit message
        let message = if changed_files.len() == 1 {
            format!("{}: {}", commit_type, changed_files[0].split('\t').last().unwrap_or("update"))
        } else {
            format!("{}{}",
                commit_type,
                if !scope.is_empty() { format!("({}): update {} files", scope, changed_files.len()) } else { format!(": update {} files", changed_files.len()) }
            )
        };
        
        Ok((message, changed_files))
    }
}

#[async_trait]
impl AgentTool for SmartCommitTool {
    fn name(&self) -> &str {
        "smart_commit"
    }
    
    fn description(&self) -> &str {
        "Create intelligent Git commits with auto-generated messages"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Optional custom commit message (will auto-generate if not provided)"
                },
                "files": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Specific files to commit (all staged if not provided)"
                }
            },
            "required": []
        })
    }
    
    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let custom_message = params["message"].as_str();
        let files = params["files"].as_array();
        
        // Stage files if specified
        if let Some(files) = files {
            for file in files {
                if let Some(path) = file.as_str() {
                    TokioCommand::new("git")
                        .arg("add")
                        .arg(path)
                        .current_dir(&self.workspace_root)
                        .output()
                        .await
                        .map_err(|e| ToolError::ExecutionFailed(format!("Failed to stage file: {}", e)))?;
                }
            }
        }
        
        // Generate or use provided message
        let message = if let Some(msg) = custom_message {
            msg.to_string()
        } else {
            let (generated_msg, _) = self.analyze_changes().await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to analyze changes: {}", e)))?;
            generated_msg
        };
        
        // Create commit
        let output = TokioCommand::new("git")
            .arg("commit")
            .arg("-m")
            .arg(&message)
            .current_dir(&self.workspace_root)
            .output()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create commit: {}", e)))?;
        
        if output.status.success() {
            Ok(ToolOutput {
                success: true,
                result: json!({
                    "message": message,
                    "output": String::from_utf8_lossy(&output.stdout)
                }),
                error: None,
                usage: None,
            })
        } else {
            Ok(ToolOutput {
                success: false,
                result: Value::Null,
                error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
                usage: None,
            })
        }
    }
}

/// Create pull request automation tool
pub struct CreatePRTool {
    workspace_root: PathBuf,
}

impl CreatePRTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
    
    /// Get current branch name
    async fn get_current_branch(&self) -> Result<String> {
        let output = TokioCommand::new("git")
            .arg("branch")
            .arg("--show-current")
            .current_dir(&self.workspace_root)
            .output()
            .await?;
        
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    
    /// Generate PR description from commits
    async fn generate_pr_description(&self, base: &str) -> Result<String> {
        let output = TokioCommand::new("git")
            .arg("log")
            .arg(format!("{}..HEAD", base))
            .arg("--pretty=format:- %s")
            .current_dir(&self.workspace_root)
            .output()
            .await?;
        
        let commits = String::from_utf8_lossy(&output.stdout);
        
        Ok(format!(
            "## Summary\n\n{}\n\n## Changes\n\n{}\n\n## Testing\n\n- [ ] Tests pass\n- [ ] Manual testing completed",
            "This PR implements improvements to the codebase.",
            commits
        ))
    }
}

#[async_trait]
impl AgentTool for CreatePRTool {
    fn name(&self) -> &str {
        "create_pr"
    }
    
    fn description(&self) -> &str {
        "Create a pull request with intelligent description"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "PR title"
                },
                "base": {
                    "type": "string",
                    "description": "Base branch (default: main)"
                },
                "description": {
                    "type": "string",
                    "description": "Optional PR description (will auto-generate if not provided)"
                },
                "draft": {
                    "type": "boolean",
                    "description": "Create as draft PR"
                }
            },
            "required": ["title"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let title = params["title"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters("Title is required".to_string()))?;
        let base = params["base"].as_str().unwrap_or("main");
        let custom_description = params["description"].as_str();
        let draft = params["draft"].as_bool().unwrap_or(false);
        
        // Push current branch
        let current_branch = self.get_current_branch().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get branch: {}", e)))?;
        
        let push_output = TokioCommand::new("git")
            .arg("push")
            .arg("-u")
            .arg("origin")
            .arg(&current_branch)
            .current_dir(&self.workspace_root)
            .output()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to push: {}", e)))?;
        
        if !push_output.status.success() {
            return Ok(ToolOutput {
                success: false,
                result: Value::Null,
                error: Some(format!("Failed to push branch: {}", 
                    String::from_utf8_lossy(&push_output.stderr))),
                usage: None,
            });
        }
        
        // Generate or use provided description
        let description = if let Some(desc) = custom_description {
            desc.to_string()
        } else {
            self.generate_pr_description(base).await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to generate description: {}", e)))?
        };
        
        // Create PR using gh CLI
        let mut pr_cmd = TokioCommand::new("gh");
        pr_cmd.arg("pr")
            .arg("create")
            .arg("--title")
            .arg(title)
            .arg("--body")
            .arg(&description)
            .arg("--base")
            .arg(base);
        
        if draft {
            pr_cmd.arg("--draft");
        }
        
        let output = pr_cmd
            .current_dir(&self.workspace_root)
            .output()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create PR: {}", e)))?;
        
        if output.status.success() {
            let pr_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(ToolOutput {
                success: true,
                result: json!({
                    "pr_url": pr_url,
                    "branch": current_branch,
                    "base": base
                }),
                error: None,
                usage: None,
            })
        } else {
            Ok(ToolOutput {
                success: false,
                result: Value::Null,
                error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
                usage: None,
            })
        }
    }
}

/// Branch management automation tool
pub struct BranchManagementTool {
    workspace_root: PathBuf,
}

impl BranchManagementTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
    
    /// Generate branch name from description
    fn generate_branch_name(&self, description: &str) -> String {
        let clean = description
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>();
        
        // Remove multiple dashes and trim
        let mut result = String::new();
        let mut prev_dash = false;
        for c in clean.chars() {
            if c == '-' {
                if !prev_dash && !result.is_empty() {
                    result.push(c);
                    prev_dash = true;
                }
            } else {
                result.push(c);
                prev_dash = false;
            }
        }
        
        result.trim_matches('-').to_string()
    }
}

#[async_trait]
impl AgentTool for BranchManagementTool {
    fn name(&self) -> &str {
        "branch_management"
    }
    
    fn description(&self) -> &str {
        "Manage Git branches intelligently"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["create", "switch", "delete", "list", "merge"],
                    "description": "Branch operation to perform"
                },
                "name": {
                    "type": "string",
                    "description": "Branch name (auto-generated for create if not provided)"
                },
                "description": {
                    "type": "string",
                    "description": "Description for branch creation"
                },
                "from": {
                    "type": "string",
                    "description": "Base branch for creation (default: current branch)"
                }
            },
            "required": ["action"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let action = params["action"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters("Action is required".to_string()))?;
        
        match action {
            "create" => {
                let name = if let Some(n) = params["name"].as_str() {
                    n.to_string()
                } else if let Some(desc) = params["description"].as_str() {
                    self.generate_branch_name(desc)
                } else {
                    return Ok(ToolOutput {
                        success: false,
                        result: Value::Null,
                        error: Some("Branch name or description required".to_string()),
                        usage: None,
                    });
                };
                
                let mut cmd = TokioCommand::new("git");
                cmd.arg("checkout")
                    .arg("-b")
                    .arg(&name);
                
                if let Some(from) = params["from"].as_str() {
                    cmd.arg(from);
                }
                
                let output = cmd
                    .current_dir(&self.workspace_root)
                    .output()
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create branch: {}", e)))?;
                
                if output.status.success() {
                    Ok(ToolOutput {
                        success: true,
                        result: json!({
                            "branch": name,
                            "message": format!("Created and switched to branch: {}", name)
                        }),
                        error: None,
                        usage: None,
                    })
                } else {
                    Ok(ToolOutput {
                        success: false,
                        result: Value::Null,
                        error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
                        usage: None,
                    })
                }
            },
            
            "list" => {
                let output = TokioCommand::new("git")
                    .arg("branch")
                    .arg("-a")
                    .current_dir(&self.workspace_root)
                    .output()
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to list branches: {}", e)))?;
                
                let branches = String::from_utf8_lossy(&output.stdout);
                Ok(ToolOutput {
                    success: true,
                    result: json!({
                        "branches": branches.lines().map(|s| s.trim()).collect::<Vec<_>>()
                    }),
                    error: None,
                    usage: None,
                })
            },
            
            _ => Ok(ToolOutput {
                success: false,
                result: Value::Null,
                error: Some(format!("Unsupported action: {}", action)),
                usage: None,
            })
        }
    }
}

/// Test runner tool with intelligent test detection
pub struct TestRunnerTool {
    workspace_root: PathBuf,
}

impl TestRunnerTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
    
    /// Detect the project's test framework
    async fn detect_test_framework(&self) -> Result<String> {
        // Check for Cargo.toml (Rust)
        if self.workspace_root.join("Cargo.toml").exists() {
            return Ok("cargo".to_string());
        }
        
        // Check for package.json (Node.js)
        if self.workspace_root.join("package.json").exists() {
            // Read package.json to detect test runner
            let content = tokio::fs::read_to_string(self.workspace_root.join("package.json")).await?;
            if content.contains("\"jest\"") {
                return Ok("jest".to_string());
            } else if content.contains("\"vitest\"") {
                return Ok("vitest".to_string());
            } else if content.contains("\"mocha\"") {
                return Ok("mocha".to_string());
            }
            return Ok("npm".to_string());
        }
        
        // Check for go.mod (Go)
        if self.workspace_root.join("go.mod").exists() {
            return Ok("go".to_string());
        }
        
        // Check for requirements.txt or pyproject.toml (Python)
        if self.workspace_root.join("pyproject.toml").exists() || 
           self.workspace_root.join("requirements.txt").exists() {
            return Ok("pytest".to_string());
        }
        
        Err(anyhow::anyhow!("Could not detect test framework"))
    }
}

#[async_trait]
impl AgentTool for TestRunnerTool {
    fn name(&self) -> &str {
        "run_tests"
    }
    
    fn description(&self) -> &str {
        "Run tests with intelligent framework detection"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "filter": {
                    "type": "string",
                    "description": "Test filter pattern"
                },
                "framework": {
                    "type": "string",
                    "description": "Override test framework detection"
                },
                "watch": {
                    "type": "boolean",
                    "description": "Run in watch mode"
                },
                "coverage": {
                    "type": "boolean",
                    "description": "Generate coverage report"
                }
            },
            "required": []
        })
    }
    
    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let filter = params["filter"].as_str();
        let framework_override = params["framework"].as_str();
        let watch = params["watch"].as_bool().unwrap_or(false);
        let coverage = params["coverage"].as_bool().unwrap_or(false);
        
        // Detect or use specified framework
        let framework = if let Some(f) = framework_override {
            f.to_string()
        } else {
            self.detect_test_framework().await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to detect framework: {}", e)))?
        };
        
        // Build test command based on framework
        let mut cmd = match framework.as_str() {
            "cargo" => {
                let mut c = TokioCommand::new("cargo");
                c.arg("test");
                if let Some(f) = filter {
                    c.arg(f);
                }
                if coverage {
                    c.arg("--").arg("--nocapture");
                }
                c
            },
            "jest" => {
                let mut c = TokioCommand::new("npx");
                c.arg("jest");
                if let Some(f) = filter {
                    c.arg(f);
                }
                if watch {
                    c.arg("--watch");
                }
                if coverage {
                    c.arg("--coverage");
                }
                c
            },
            "go" => {
                let mut c = TokioCommand::new("go");
                c.arg("test");
                if let Some(f) = filter {
                    c.arg("-run").arg(f);
                }
                if coverage {
                    c.arg("-cover");
                }
                c.arg("./...");
                c
            },
            "pytest" => {
                let mut c = TokioCommand::new("pytest");
                if let Some(f) = filter {
                    c.arg("-k").arg(f);
                }
                if coverage {
                    c.arg("--cov");
                }
                c
            },
            _ => {
                let mut c = TokioCommand::new("npm");
                c.arg("test");
                if let Some(f) = filter {
                    c.arg("--").arg(f);
                }
                c
            }
        };
        
        let output = cmd
            .current_dir(&self.workspace_root)
            .output()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to run tests: {}", e)))?;
        
        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Parse test results if possible
        let mut passed = 0;
        let mut failed = 0;
        
        // Simple parsing for common patterns
        for line in stdout.lines().chain(stderr.lines()) {
            if line.contains("passed") || line.contains("PASS") {
                if let Some(num) = line.split_whitespace()
                    .find_map(|s| s.parse::<i32>().ok()) {
                    passed = num;
                }
            }
            if line.contains("failed") || line.contains("FAIL") {
                if let Some(num) = line.split_whitespace()
                    .find_map(|s| s.parse::<i32>().ok()) {
                    failed = num;
                }
            }
        }
        
        Ok(ToolOutput {
            success,
            result: json!({
                "framework": framework,
                "passed": passed,
                "failed": failed,
                "output": stdout.to_string(),
                "errors": if !success { stderr.to_string() } else { String::new() }
            }),
            error: if !success { Some(format!("Tests failed: {} failures", failed)) } else { None },
            usage: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_smart_commit_message_generation() {
        let temp = TempDir::new().unwrap();
        let tool = SmartCommitTool::new(temp.path().to_path_buf());
        
        // Initialize git repo
        Command::new("git")
            .arg("init")
            .current_dir(temp.path())
            .output()
            .unwrap();
        
        // Create a test file
        std::fs::write(temp.path().join("test.rs"), "fn test() {}").unwrap();
        
        Command::new("git")
            .arg("add")
            .arg("test.rs")
            .current_dir(temp.path())
            .output()
            .unwrap();
        
        // Test analyze_changes
        let (message, files) = tool.analyze_changes().await.unwrap();
        assert!(message.contains("test"));
        assert_eq!(files.len(), 1);
    }
    
    #[test]
    fn test_branch_name_generation() {
        let temp = TempDir::new().unwrap();
        let tool = BranchManagementTool::new(temp.path().to_path_buf());
        
        let name = tool.generate_branch_name("Fix authentication bug in login flow");
        assert_eq!(name, "fix-authentication-bug-in-login-flow");
        
        let name2 = tool.generate_branch_name("Add new feature: user profiles!!!");
        assert_eq!(name2, "add-new-feature-user-profiles");
    }
    
    #[tokio::test]
    async fn test_framework_detection() {
        let temp = TempDir::new().unwrap();
        
        // Test Rust detection
        std::fs::write(temp.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        let tool = TestRunnerTool::new(temp.path().to_path_buf());
        let framework = tool.detect_test_framework().await.unwrap();
        assert_eq!(framework, "cargo");
        
        // Test Node.js detection
        std::fs::remove_file(temp.path().join("Cargo.toml")).unwrap();
        std::fs::write(temp.path().join("package.json"), r#"{"devDependencies": {"jest": "^27.0.0"}}"#).unwrap();
        let framework = tool.detect_test_framework().await.unwrap();
        assert_eq!(framework, "jest");
    }
}