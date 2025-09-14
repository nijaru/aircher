use super::{AgentTool, ToolError, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::process::Command as TokioCommand;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct BuildSystemTool {
    workspace_root: PathBuf,
}

#[derive(Debug, Deserialize)]
struct BuildParams {
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    release: bool,
    #[serde(default)]
    clean: bool,
    #[serde(default)]
    verbose: bool,
    #[serde(default)]
    framework: Option<String>,
}

impl BuildSystemTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Detect the project's build system
    async fn detect_build_system(&self) -> Result<String> {
        // Check for Cargo.toml (Rust)
        if self.workspace_root.join("Cargo.toml").exists() {
            return Ok("cargo".to_string());
        }

        // Check for package.json (Node.js)
        if self.workspace_root.join("package.json").exists() {
            // Read package.json to check for build scripts
            if let Ok(content) = tokio::fs::read_to_string(self.workspace_root.join("package.json")).await {
                if content.contains("\"build\"") {
                    return Ok("npm".to_string());
                }
            }
            return Ok("npm".to_string());
        }

        // Check for go.mod (Go)
        if self.workspace_root.join("go.mod").exists() {
            return Ok("go".to_string());
        }

        // Check for Makefile
        if self.workspace_root.join("Makefile").exists() || self.workspace_root.join("makefile").exists() {
            return Ok("make".to_string());
        }

        // Check for CMakeLists.txt (CMake)
        if self.workspace_root.join("CMakeLists.txt").exists() {
            return Ok("cmake".to_string());
        }

        // Check for build.gradle (Gradle)
        if self.workspace_root.join("build.gradle").exists() || self.workspace_root.join("build.gradle.kts").exists() {
            return Ok("gradle".to_string());
        }

        // Check for pom.xml (Maven)
        if self.workspace_root.join("pom.xml").exists() {
            return Ok("maven".to_string());
        }

        // Check for pyproject.toml or setup.py (Python)
        if self.workspace_root.join("pyproject.toml").exists() || self.workspace_root.join("setup.py").exists() {
            return Ok("python".to_string());
        }

        // Check for Dockerfile
        if self.workspace_root.join("Dockerfile").exists() {
            return Ok("docker".to_string());
        }

        Err(anyhow::anyhow!("No recognized build system found"))
    }

    /// Get available build targets for the detected system
    async fn get_build_targets(&self, build_system: &str) -> Vec<String> {
        match build_system {
            "cargo" => {
                // Parse Cargo.toml for targets
                if let Ok(content) = tokio::fs::read_to_string(self.workspace_root.join("Cargo.toml")).await {
                    let mut targets = vec!["default".to_string()];
                    if content.contains("[[bin]]") {
                        targets.push("bin".to_string());
                    }
                    if content.contains("[lib]") {
                        targets.push("lib".to_string());
                    }
                    if content.contains("[[example]]") {
                        targets.push("examples".to_string());
                    }
                    targets
                } else {
                    vec!["default".to_string()]
                }
            }
            "npm" => {
                // Parse package.json for scripts
                if let Ok(content) = tokio::fs::read_to_string(self.workspace_root.join("package.json")).await {
                    let mut targets = vec!["build".to_string()];
                    if content.contains("\"dev\"") { targets.push("dev".to_string()); }
                    if content.contains("\"start\"") { targets.push("start".to_string()); }
                    if content.contains("\"prod\"") { targets.push("prod".to_string()); }
                    if content.contains("\"dist\"") { targets.push("dist".to_string()); }
                    targets
                } else {
                    vec!["build".to_string()]
                }
            }
            "go" => vec!["build".to_string(), "install".to_string()],
            "make" => vec!["all".to_string(), "build".to_string(), "install".to_string(), "clean".to_string()],
            "cmake" => vec!["all".to_string(), "clean".to_string()],
            "gradle" => vec!["build".to_string(), "assemble".to_string(), "check".to_string(), "clean".to_string()],
            "maven" => vec!["compile".to_string(), "package".to_string(), "install".to_string(), "clean".to_string()],
            "python" => vec!["build".to_string(), "install".to_string(), "wheel".to_string()],
            "docker" => vec!["build".to_string()],
            _ => vec!["build".to_string()],
        }
    }
}

#[async_trait]
impl AgentTool for BuildSystemTool {
    fn name(&self) -> &str {
        "build_project"
    }

    fn description(&self) -> &str {
        "Build the project using the detected build system (cargo, npm, make, cmake, gradle, maven, etc.)"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Build target (e.g., 'release', 'debug', 'production')"
                },
                "release": {
                    "type": "boolean",
                    "description": "Build in release/production mode",
                    "default": false
                },
                "clean": {
                    "type": "boolean",
                    "description": "Clean before building",
                    "default": false
                },
                "verbose": {
                    "type": "boolean",
                    "description": "Enable verbose output",
                    "default": false
                },
                "framework": {
                    "type": "string",
                    "description": "Override build system detection"
                }
            },
            "required": []
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: BuildParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        // Detect or use specified build system
        let build_system = if let Some(framework) = params.framework {
            framework
        } else {
            self.detect_build_system().await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to detect build system: {}", e)))?
        };

        info!("Using build system: {}", build_system);

        // Handle clean first if requested
        if params.clean {
            let clean_result = self.run_clean_command(&build_system).await;
            if let Err(e) = clean_result {
                warn!("Clean command failed: {}", e);
            }
        }

        // Build the main command based on build system
        let mut cmd = match build_system.as_str() {
            "cargo" => {
                let mut c = TokioCommand::new("cargo");
                c.arg("build");

                if params.release {
                    c.arg("--release");
                }
                if params.verbose {
                    c.arg("--verbose");
                }
                if let Some(target) = &params.target {
                    if target != "default" {
                        c.arg("--").arg(target);
                    }
                }
                c
            }
            "npm" => {
                let mut c = TokioCommand::new("npm");
                c.arg("run");

                let target = params.target.as_deref().unwrap_or("build");
                c.arg(target);

                if params.verbose {
                    c.arg("--verbose");
                }
                c
            }
            "go" => {
                let mut c = TokioCommand::new("go");
                let target = params.target.as_deref().unwrap_or("build");
                c.arg(target);

                if params.verbose {
                    c.arg("-v");
                }
                if target == "build" && params.release {
                    c.arg("-ldflags").arg("-s -w"); // Strip debug info for release
                }
                c.arg("./...");
                c
            }
            "make" => {
                let mut c = TokioCommand::new("make");
                let target = params.target.as_deref().unwrap_or("all");
                c.arg(target);

                if params.verbose {
                    c.arg("V=1");
                }
                if params.release {
                    c.env("BUILD_TYPE", "release");
                }
                c
            }
            "cmake" => {
                // CMake typically needs a build directory
                let build_dir = self.workspace_root.join("build");
                tokio::fs::create_dir_all(&build_dir).await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create build directory: {}", e)))?;

                let mut c = TokioCommand::new("cmake");
                c.arg("--build").arg(&build_dir);

                if let Some(target) = &params.target {
                    c.arg("--target").arg(target);
                }
                if params.verbose {
                    c.arg("--verbose");
                }
                c
            }
            "gradle" => {
                let mut c = TokioCommand::new("./gradlew");
                // Fall back to system gradle if gradlew doesn't exist
                if !self.workspace_root.join("gradlew").exists() {
                    c = TokioCommand::new("gradle");
                }

                let target = params.target.as_deref().unwrap_or("build");
                c.arg(target);

                if params.verbose {
                    c.arg("--info");
                }
                c
            }
            "maven" => {
                let mut c = TokioCommand::new("mvn");
                let target = params.target.as_deref().unwrap_or("compile");
                c.arg(target);

                if params.verbose {
                    c.arg("-X");
                }
                if params.release {
                    c.arg("-Dspring.profiles.active=prod");
                }
                c
            }
            "python" => {
                let mut c = TokioCommand::new("python");
                c.arg("-m").arg("build");

                if params.verbose {
                    c.arg("--verbose");
                }
                c
            }
            "docker" => {
                let mut c = TokioCommand::new("docker");
                c.arg("build");

                if let Some(tag) = &params.target {
                    c.arg("-t").arg(tag);
                } else {
                    c.arg("-t").arg("app:latest");
                }
                c.arg(".");
                c
            }
            _ => {
                return Err(ToolError::ExecutionFailed(format!("Unsupported build system: {}", build_system)));
            }
        };

        // Set working directory
        cmd.current_dir(&self.workspace_root);

        debug!("Running build command: {:?}", cmd);

        // Execute the build command
        let output = cmd
            .output()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to execute build command: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let success = output.status.success();

        let result = json!({
            "build_system": build_system,
            "target": params.target,
            "release": params.release,
            "success": success,
            "exit_code": output.status.code(),
            "stdout": stdout.to_string(),
            "stderr": stderr.to_string(),
            "available_targets": self.get_build_targets(&build_system).await
        });

        if !success {
            return Ok(ToolOutput {
                success: false,
                result,
                error: Some(format!("Build failed with exit code {}: {}",
                    output.status.code().unwrap_or(-1), stderr)),
                usage: None,
            });
        }

        Ok(ToolOutput {
            success: true,
            result,
            error: None,
            usage: None,
        })
    }
}

impl BuildSystemTool {
    async fn run_clean_command(&self, build_system: &str) -> Result<()> {
        let mut cmd = match build_system {
            "cargo" => {
                let mut c = TokioCommand::new("cargo");
                c.arg("clean");
                c
            }
            "npm" => {
                // Try npm run clean, fallback to removing node_modules
                let mut c = TokioCommand::new("npm");
                c.arg("run").arg("clean");
                c
            }
            "go" => {
                let mut c = TokioCommand::new("go");
                c.arg("clean");
                c
            }
            "make" => {
                let mut c = TokioCommand::new("make");
                c.arg("clean");
                c
            }
            "cmake" => {
                let mut c = TokioCommand::new("cmake");
                c.arg("--build").arg(self.workspace_root.join("build"));
                c.arg("--target").arg("clean");
                c
            }
            "gradle" => {
                let mut c = TokioCommand::new("gradle");
                c.arg("clean");
                c
            }
            "maven" => {
                let mut c = TokioCommand::new("mvn");
                c.arg("clean");
                c
            }
            _ => return Ok(()), // No clean command for this build system
        };

        cmd.current_dir(&self.workspace_root);
        let _ = cmd.output().await?; // Ignore clean failures
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_detect_build_system() {
        let temp = TempDir::new().unwrap();

        // Test Rust detection
        std::fs::write(temp.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        let tool = BuildSystemTool::new(temp.path().to_path_buf());
        let build_system = tool.detect_build_system().await.unwrap();
        assert_eq!(build_system, "cargo");

        // Test Node.js detection
        std::fs::remove_file(temp.path().join("Cargo.toml")).unwrap();
        std::fs::write(temp.path().join("package.json"), "{}").unwrap();
        let build_system = tool.detect_build_system().await.unwrap();
        assert_eq!(build_system, "npm");
    }

    #[tokio::test]
    async fn test_get_build_targets() {
        let temp = TempDir::new().unwrap();
        let tool = BuildSystemTool::new(temp.path().to_path_buf());

        let targets = tool.get_build_targets("cargo").await;
        assert!(targets.contains(&"default".to_string()));

        let targets = tool.get_build_targets("npm").await;
        assert!(targets.contains(&"build".to_string()));
    }
}