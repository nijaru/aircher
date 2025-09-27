/// Real implementation of error analysis tool that provides actual value
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;

use super::{AgentTool, ToolOutput, ToolError};

/// Real error analysis tool that actually parses and analyzes errors
pub struct RealAnalyzeErrorsTool {
    /// Known error patterns and their solutions
    error_patterns: HashMap<String, ErrorPattern>,
    /// Workspace root for context
    workspace_root: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct ErrorPattern {
    pattern: Regex,
    category: ErrorCategory,
    common_causes: Vec<String>,
    suggested_fixes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ErrorCategory {
    CompileError,
    RuntimeError,
    TypeMismatch,
    BorrowChecker,
    NullReference,
    ImportError,
    SyntaxError,
    PermissionError,
    NetworkError,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorAnalysis {
    error_type: String,
    severity: String,
    location: Option<ErrorLocation>,
    category: ErrorCategory,
    root_cause: String,
    common_causes: Vec<String>,
    suggested_fixes: Vec<String>,
    related_files: Vec<String>,
    confidence: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorLocation {
    file: String,
    line: Option<u32>,
    column: Option<u32>,
}

impl RealAnalyzeErrorsTool {
    pub fn new(workspace_root: Option<PathBuf>) -> Self {
        let mut error_patterns = HashMap::new();

        // Rust-specific error patterns
        error_patterns.insert(
            "borrow_checker".to_string(),
            ErrorPattern {
                pattern: Regex::new(r"cannot borrow .* as mutable").unwrap(),
                category: ErrorCategory::BorrowChecker,
                common_causes: vec![
                    "Attempting to mutably borrow a value that's already borrowed".to_string(),
                    "Multiple mutable references to the same data".to_string(),
                ],
                suggested_fixes: vec![
                    "Use RefCell or Mutex for interior mutability".to_string(),
                    "Clone the data if ownership isn't critical".to_string(),
                    "Restructure code to avoid simultaneous borrows".to_string(),
                ],
            }
        );

        error_patterns.insert(
            "moved_value".to_string(),
            ErrorPattern {
                pattern: Regex::new(r"value moved here|use of moved value").unwrap(),
                category: ErrorCategory::BorrowChecker,
                common_causes: vec![
                    "Value ownership was transferred in a previous operation".to_string(),
                    "Attempting to use a value after it was moved".to_string(),
                ],
                suggested_fixes: vec![
                    "Clone the value before moving if you need to use it again".to_string(),
                    "Use borrowing (&) instead of moving ownership".to_string(),
                    "Implement Copy trait if the type is simple".to_string(),
                ],
            }
        );

        // Type errors
        error_patterns.insert(
            "type_mismatch".to_string(),
            ErrorPattern {
                pattern: Regex::new(r"expected .*, found .*|mismatched types").unwrap(),
                category: ErrorCategory::TypeMismatch,
                common_causes: vec![
                    "Function expects different type than provided".to_string(),
                    "Implicit type conversion not supported".to_string(),
                ],
                suggested_fixes: vec![
                    "Check function signature and adjust argument types".to_string(),
                    "Use type conversion methods (as, into, from)".to_string(),
                    "Verify generic type parameters are correct".to_string(),
                ],
            }
        );

        // Import/Module errors
        error_patterns.insert(
            "unresolved_import".to_string(),
            ErrorPattern {
                pattern: Regex::new(r"unresolved import|cannot find .* in this scope").unwrap(),
                category: ErrorCategory::ImportError,
                common_causes: vec![
                    "Missing dependency in Cargo.toml".to_string(),
                    "Incorrect module path or name".to_string(),
                    "Private module/function not exposed".to_string(),
                ],
                suggested_fixes: vec![
                    "Add the crate to dependencies in Cargo.toml".to_string(),
                    "Check the correct module path with 'use' statement".to_string(),
                    "Ensure the item is marked as 'pub' if importing from another module".to_string(),
                    "Run 'cargo add <crate_name>' to add missing dependency".to_string(),
                ],
            }
        );

        // Runtime errors
        error_patterns.insert(
            "null_reference".to_string(),
            ErrorPattern {
                pattern: Regex::new(r"null pointer|undefined|None\.unwrap\(\)|panic.*Option::None").unwrap(),
                category: ErrorCategory::NullReference,
                common_causes: vec![
                    "Unwrapping an Option that contains None".to_string(),
                    "Dereferencing a null or invalid pointer".to_string(),
                    "Accessing uninitialized data".to_string(),
                ],
                suggested_fixes: vec![
                    "Use pattern matching or if let instead of unwrap()".to_string(),
                    "Use unwrap_or() or unwrap_or_else() for defaults".to_string(),
                    "Check with is_some() before unwrapping".to_string(),
                    "Use the ? operator for error propagation".to_string(),
                ],
            }
        );

        // Network/IO errors
        error_patterns.insert(
            "connection_error".to_string(),
            ErrorPattern {
                pattern: Regex::new(r"connection refused|timeout|network unreachable").unwrap(),
                category: ErrorCategory::NetworkError,
                common_causes: vec![
                    "Service not running on specified port".to_string(),
                    "Firewall blocking the connection".to_string(),
                    "Incorrect host/port configuration".to_string(),
                ],
                suggested_fixes: vec![
                    "Verify the service is running: 'netstat -an | grep <port>'".to_string(),
                    "Check firewall rules and network connectivity".to_string(),
                    "Verify URL/host configuration is correct".to_string(),
                    "Add retry logic with exponential backoff".to_string(),
                ],
            }
        );

        Self {
            error_patterns,
            workspace_root,
        }
    }

    /// Parse error message to extract structured information
    fn parse_error_message(&self, error_message: &str) -> ErrorAnalysis {
        // Extract file location if present
        let location = self.extract_location(error_message);

        // Determine error category and suggestions
        let (category, causes, fixes, confidence) = self.analyze_error_pattern(error_message);

        // Extract error type from message
        let error_type = self.extract_error_type(error_message);

        // Determine severity
        let severity = self.determine_severity(&category, error_message);

        // Find related files
        let related_files = self.find_related_files(error_message, &location);

        // Identify root cause
        let root_cause = self.identify_root_cause(error_message, &category);

        ErrorAnalysis {
            error_type,
            severity,
            location,
            category,
            root_cause,
            common_causes: causes,
            suggested_fixes: fixes,
            related_files,
            confidence,
        }
    }

    /// Extract file location from error message
    fn extract_location(&self, error_message: &str) -> Option<ErrorLocation> {
        // Try Rust error format: src/main.rs:10:5
        let rust_pattern = Regex::new(r"(\S+\.rs):(\d+):(\d+)").unwrap();
        if let Some(captures) = rust_pattern.captures(error_message) {
            return Some(ErrorLocation {
                file: captures[1].to_string(),
                line: captures[2].parse().ok(),
                column: captures[3].parse().ok(),
            });
        }

        // Try generic format: file.ext:line
        let generic_pattern = Regex::new(r"(\S+\.\w+):(\d+)").unwrap();
        if let Some(captures) = generic_pattern.captures(error_message) {
            return Some(ErrorLocation {
                file: captures[1].to_string(),
                line: captures[2].parse().ok(),
                column: None,
            });
        }

        None
    }

    /// Analyze error against known patterns
    fn analyze_error_pattern(&self, error_message: &str) -> (ErrorCategory, Vec<String>, Vec<String>, f32) {
        let lower_msg = error_message.to_lowercase();

        // Check against known patterns
        for pattern in self.error_patterns.values() {
            if pattern.pattern.is_match(&lower_msg) {
                return (
                    pattern.category.clone(),
                    pattern.common_causes.clone(),
                    pattern.suggested_fixes.clone(),
                    0.9, // High confidence for pattern match
                );
            }
        }

        // Fallback heuristics
        if lower_msg.contains("type") || lower_msg.contains("expected") {
            return (
                ErrorCategory::TypeMismatch,
                vec!["Type mismatch between expected and actual values".to_string()],
                vec!["Check type annotations and function signatures".to_string()],
                0.7,
            );
        }

        if lower_msg.contains("syntax") || lower_msg.contains("unexpected") {
            return (
                ErrorCategory::SyntaxError,
                vec!["Invalid syntax or unexpected token".to_string()],
                vec!["Check for missing semicolons, brackets, or quotes".to_string()],
                0.7,
            );
        }

        if lower_msg.contains("permission") || lower_msg.contains("denied") {
            return (
                ErrorCategory::PermissionError,
                vec!["Insufficient permissions for the operation".to_string()],
                vec!["Check file permissions or run with appropriate privileges".to_string()],
                0.8,
            );
        }

        // Unknown error
        (
            ErrorCategory::Unknown,
            vec!["Error cause could not be automatically determined".to_string()],
            vec![
                "Review the full error message and stack trace".to_string(),
                "Search for similar errors in documentation or forums".to_string(),
                "Use a debugger to step through the code".to_string(),
            ],
            0.3,
        )
    }

    /// Extract error type from message
    fn extract_error_type(&self, error_message: &str) -> String {
        // Look for explicit error types (E0308, etc.)
        let error_code_pattern = Regex::new(r"E\d{4}").unwrap();
        if let Some(captures) = error_code_pattern.find(error_message) {
            return format!("Rust Error {}", captures.as_str());
        }

        // Extract first line as error type
        error_message.lines()
            .next()
            .unwrap_or("Unknown Error")
            .chars()
            .take(100)
            .collect()
    }

    /// Determine error severity
    fn determine_severity(&self, category: &ErrorCategory, error_message: &str) -> String {
        let lower_msg = error_message.to_lowercase();

        if lower_msg.contains("panic") || lower_msg.contains("fatal") {
            return "critical".to_string();
        }

        match category {
            ErrorCategory::CompileError | ErrorCategory::SyntaxError => "high".to_string(),
            ErrorCategory::TypeMismatch | ErrorCategory::BorrowChecker => "medium".to_string(),
            ErrorCategory::ImportError => "medium".to_string(),
            ErrorCategory::RuntimeError | ErrorCategory::NullReference => "high".to_string(),
            ErrorCategory::PermissionError | ErrorCategory::NetworkError => "medium".to_string(),
            ErrorCategory::Unknown => "low".to_string(),
        }
    }

    /// Find files related to the error
    fn find_related_files(&self, error_message: &str, location: &Option<ErrorLocation>) -> Vec<String> {
        let mut files = Vec::new();

        // Add main error file
        if let Some(loc) = location {
            files.push(loc.file.clone());
        }

        // Extract other file references from error message
        let file_pattern = Regex::new(r"(\S+\.(?:rs|toml|json|js|py|go|cpp|c|java))\b").unwrap();
        for captures in file_pattern.captures_iter(error_message) {
            let file = captures[1].to_string();
            if !files.contains(&file) {
                files.push(file);
            }
        }

        files
    }

    /// Identify the root cause of the error
    fn identify_root_cause(&self, error_message: &str, category: &ErrorCategory) -> String {
        // Extract the most relevant line from the error
        let lines: Vec<&str> = error_message.lines().collect();

        // Look for lines with "error:", "error[E", or similar
        for line in &lines {
            if line.contains("error:") || line.contains("error[E") {
                return line.trim_start_matches("error:").trim().to_string();
            }
        }

        // Fallback to category-based description
        match category {
            ErrorCategory::BorrowChecker => "Rust ownership or borrowing rules violation".to_string(),
            ErrorCategory::TypeMismatch => "Type incompatibility between expected and actual values".to_string(),
            ErrorCategory::ImportError => "Module or dependency resolution failure".to_string(),
            ErrorCategory::NullReference => "Attempted to use a null or None value".to_string(),
            ErrorCategory::SyntaxError => "Invalid syntax preventing parsing".to_string(),
            _ => lines.first().unwrap_or(&"Unable to determine root cause").to_string(),
        }
    }
}

#[async_trait]
impl AgentTool for RealAnalyzeErrorsTool {
    fn name(&self) -> &str {
        "analyze_errors"
    }

    fn description(&self) -> &str {
        "Analyze error messages to identify patterns, root causes, and provide actionable fixes"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "error_message": {
                    "type": "string",
                    "description": "The error message to analyze"
                },
                "context": {
                    "type": "object",
                    "description": "Additional context about the error",
                    "properties": {
                        "language": {
                            "type": "string",
                            "description": "Programming language (rust, python, javascript, etc.)"
                        },
                        "file_path": {
                            "type": "string",
                            "description": "Path to the file with the error"
                        }
                    }
                }
            },
            "required": ["error_message"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let error_message = params.get("error_message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing error_message".to_string()))?;

        // Perform real analysis
        let analysis = self.parse_error_message(error_message);

        // Build detailed output
        let output = json!({
            "error_type": analysis.error_type,
            "severity": analysis.severity,
            "category": format!("{:?}", analysis.category),
            "location": analysis.location.as_ref().map(|loc| json!({
                "file": loc.file,
                "line": loc.line,
                "column": loc.column,
            })),
            "root_cause": analysis.root_cause,
            "common_causes": analysis.common_causes,
            "suggested_fixes": analysis.suggested_fixes,
            "related_files": analysis.related_files,
            "confidence": analysis.confidence,
            "analysis_complete": true,
        });

        Ok(ToolOutput {
            success: true,
            result: output,
            error: None,
            usage: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rust_borrow_checker_error() {
        let tool = RealAnalyzeErrorsTool::new(None);

        let params = json!({
            "error_message": "error[E0502]: cannot borrow `data` as mutable because it is also borrowed as immutable\n  --> src/main.rs:10:5\n   |\n9  |     let reference = &data;\n   |                     ----- immutable borrow occurs here\n10 |     modify(&mut data);\n   |            ^^^^^^^^^ mutable borrow occurs here"
        });

        let result = tool.execute(params).await.unwrap();
        let output = result.result;

        assert!(result.success);
        assert_eq!(output["severity"], "medium");
        assert!(output["suggested_fixes"].as_array().unwrap().len() > 0);
        assert_eq!(output["location"]["file"], "src/main.rs");
        assert_eq!(output["location"]["line"], 10);
    }

    #[tokio::test]
    async fn test_type_mismatch_error() {
        let tool = RealAnalyzeErrorsTool::new(None);

        let params = json!({
            "error_message": "error[E0308]: mismatched types\n  --> src/lib.rs:42:18\n   |\n42 |     let count: u32 = \"not a number\";\n   |                ^^^   ^^^^^^^^^^^^^^ expected `u32`, found `&str`"
        });

        let result = tool.execute(params).await.unwrap();
        let output = result.result;

        assert!(result.success);
        assert!(output["root_cause"].as_str().unwrap().contains("mismatch"));
        assert!(output["suggested_fixes"].as_array().unwrap()
            .iter()
            .any(|fix| fix.as_str().unwrap().contains("type")));
    }

    #[tokio::test]
    async fn test_import_error() {
        let tool = RealAnalyzeErrorsTool::new(None);

        let params = json!({
            "error_message": "error[E0432]: unresolved import `tokio::runtime`\n --> src/main.rs:1:5\n  |\n1 | use tokio::runtime::Runtime;\n  |     ^^^^^^^^^^^^^^ could not find `runtime` in `tokio`"
        });

        let result = tool.execute(params).await.unwrap();
        let output = result.result;

        assert!(result.success);
        assert_eq!(output["category"], "ImportError");
        assert!(output["suggested_fixes"].as_array().unwrap()
            .iter()
            .any(|fix| fix.as_str().unwrap().contains("Cargo.toml")));
    }
}