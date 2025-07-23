use super::{AgentTool, ToolError, ToolOutput};
use crate::semantic_search::SemanticCodeSearch;
use crate::intelligence::IntelligenceEngine;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

pub struct SearchCodeTool {
    semantic_search: Option<Box<SemanticCodeSearch>>,
    intelligence: Option<IntelligenceEngine>,
}

#[derive(Debug, Deserialize)]
struct SearchCodeParams {
    query: String,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    file_types: Option<Vec<String>>,
    #[serde(default)]
    chunk_types: Option<Vec<String>>,
}

fn default_limit() -> usize {
    10
}

impl SearchCodeTool {
    pub fn new() -> Self {
        // These will be injected when the tool is registered with the controller
        Self {
            semantic_search: None,
            intelligence: None,
        }
    }
    
    pub fn with_semantic_search(semantic_search: SemanticCodeSearch, intelligence: IntelligenceEngine) -> Self {
        Self {
            semantic_search: Some(Box::new(semantic_search)),
            intelligence: Some(intelligence),
        }
    }
}

#[async_trait]
impl AgentTool for SearchCodeTool {
    fn name(&self) -> &str {
        "search_code"
    }
    
    fn description(&self) -> &str {
        "Search for code semantically across the codebase using natural language queries"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Natural language query to search for"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results to return",
                    "default": 10,
                    "minimum": 1,
                    "maximum": 50
                },
                "file_types": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Filter by file extensions (e.g., ['rs', 'py'])"
                },
                "chunk_types": {
                    "type": "array",
                    "items": { 
                        "type": "string",
                        "enum": ["function", "class", "module", "comment", "generic"]
                    },
                    "description": "Filter by code chunk types"
                }
            },
            "required": ["query"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: SearchCodeParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;
        
        // For now, return a simple indication that search is available but needs integration
        Ok(ToolOutput {
            success: true,
            result: json!({
                "query": params.query,
                "results": [],
                "count": 0,
                "message": "Search tool ready - integration with TUI in progress"
            }),
            error: None,
            usage: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FindDefinitionTool {
    workspace_root: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct FindDefinitionParams {
    symbol: String,
    #[serde(default)]
    file_types: Option<Vec<String>>,
}

impl FindDefinitionTool {
    pub fn new() -> Self {
        Self {
            workspace_root: std::env::current_dir().ok(),
        }
    }
}

#[async_trait]
impl AgentTool for FindDefinitionTool {
    fn name(&self) -> &str {
        "find_definition"
    }
    
    fn description(&self) -> &str {
        "Find the definition of a function, class, or variable"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "symbol": {
                    "type": "string",
                    "description": "Name of the symbol to find"
                },
                "file_types": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Filter by file extensions"
                }
            },
            "required": ["symbol"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: FindDefinitionParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;
        
        // Use ripgrep to find definitions
        use std::process::Command;
        
        let mut cmd = Command::new("rg");
        cmd.arg("--json")
            .arg("--type-add")
            .arg("code:*.{rs,py,js,ts,go,java,cpp,c,rb}")
            .arg("--type")
            .arg("code");
        
        // Add pattern for common definition patterns
        let patterns = vec![
            format!(r"^[[:space:]]*(pub[[:space:]]+)?fn[[:space:]]+{}", params.symbol),
            format!(r"^[[:space:]]*(pub[[:space:]]+)?struct[[:space:]]+{}", params.symbol),
            format!(r"^[[:space:]]*(pub[[:space:]]+)?enum[[:space:]]+{}", params.symbol),
            format!(r"^[[:space:]]*(pub[[:space:]]+)?trait[[:space:]]+{}", params.symbol),
            format!(r"^[[:space:]]*class[[:space:]]+{}", params.symbol),
            format!(r"^[[:space:]]*def[[:space:]]+{}", params.symbol),
            format!(r"^[[:space:]]*function[[:space:]]+{}", params.symbol),
            format!(r"^[[:space:]]*(const|let|var)[[:space:]]+{}", params.symbol),
        ];
        
        let pattern = patterns.join("|");
        cmd.arg(&pattern);
        
        if let Some(root) = &self.workspace_root {
            cmd.current_dir(root);
        }
        
        let output = cmd.output()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to run ripgrep: {}", e)))?;
        
        if !output.status.success() {
            return Ok(ToolOutput {
                success: true,
                result: json!({
                    "symbol": params.symbol,
                    "definitions": [],
                    "message": "No definitions found"
                }),
                error: None,
                usage: None,
            });
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut definitions = Vec::new();
        
        for line in stdout.lines() {
            if let Ok(entry) = serde_json::from_str::<Value>(line) {
                if let Some(data) = entry.get("data") {
                    if let (Some(path), Some(line_number), Some(lines)) = (
                        data.get("path").and_then(|p| p.get("text")).and_then(|t| t.as_str()),
                        data.get("line_number").and_then(|n| n.as_u64()),
                        data.get("lines").and_then(|l| l.get("text")).and_then(|t| t.as_str())
                    ) {
                        definitions.push(json!({
                            "file": path,
                            "line": line_number,
                            "text": lines.trim(),
                            "type": guess_definition_type(lines)
                        }));
                    }
                }
            }
        }
        
        Ok(ToolOutput {
            success: true,
            result: json!({
                "symbol": params.symbol,
                "definitions": definitions,
                "count": definitions.len()
            }),
            error: None,
            usage: None,
        })
    }
}

fn guess_definition_type(line: &str) -> &'static str {
    let trimmed = line.trim();
    if trimmed.contains("fn ") { "function" }
    else if trimmed.contains("struct ") { "struct" }
    else if trimmed.contains("enum ") { "enum" }
    else if trimmed.contains("trait ") { "trait" }
    else if trimmed.contains("class ") { "class" }
    else if trimmed.contains("def ") { "function" }
    else if trimmed.contains("const ") { "constant" }
    else if trimmed.contains("let ") || trimmed.contains("var ") { "variable" }
    else { "unknown" }
}