use anyhow::Result;
use regex::Regex;
use serde_json::Value;
use crate::agent::tools::ToolCall;

/// Parse tool calls from assistant responses
/// Supports multiple formats for compatibility with different models
pub struct ToolCallParser {
    xml_regex: Regex,
    json_regex: Regex,
    openai_json_regex: Regex,
    function_regex: Regex,
}

impl ToolCallParser {
    pub fn new() -> Result<Self> {
        Ok(Self {
            // XML-style: <tool>read_file</tool><params>{"path": "src/main.rs"}</params>
            xml_regex: Regex::new(r"<tool>([^<]+)</tool>\s*<params>([^<]+)</params>")?,

            // JSON-style: {"tool": "read_file", "params": {"path": "src/main.rs"}}
            json_regex: Regex::new(r#"\{[^}]*"tool"\s*:\s*"([^"]+)"[^}]*"params"\s*:\s*(\{[^}]+\})[^}]*\}"#)?,

            // OpenAI-style JSON: {"name": "tool_name", "parameters": {...}} or {"name": "tool_name", "arguments": {...}}
            // Simple pattern to catch JSON anywhere in content
            openai_json_regex: Regex::new(r#"\{\s*"name"\s*:\s*"([^"]+)"\s*,\s*"(?:parameters|arguments)"\s*:\s*(\{[^}]*\})\s*\}"#)?,

            // Function-style: read_file({"path": "src/main.rs"})
            function_regex: Regex::new(r"(\w+)\s*\((\{[^}]+\})\)")?,
        })
    }

    pub fn parse(&self, content: &str) -> Vec<ToolCall> {
        let mut tool_calls = Vec::new();

        // Try XML-style first (most explicit)
        for cap in self.xml_regex.captures_iter(content) {
            if let (Some(name), Some(params)) = (cap.get(1), cap.get(2)) {
                if let Ok(params_json) = serde_json::from_str::<Value>(params.as_str()) {
                    tool_calls.push(ToolCall {
                        name: name.as_str().to_string(),
                        parameters: params_json,
                    });
                }
            }
        }

        // If no XML-style found, try OpenAI-style JSON (most common from models)
        if tool_calls.is_empty() {
            for cap in self.openai_json_regex.captures_iter(content) {
                if let (Some(name), Some(params)) = (cap.get(1), cap.get(2)) {
                    if let Ok(params_json) = serde_json::from_str::<Value>(params.as_str()) {
                        tool_calls.push(ToolCall {
                            name: name.as_str().to_string(),
                            parameters: params_json,
                        });
                    }
                }
            }
        }

        // If no OpenAI-style found, try our legacy JSON-style
        if tool_calls.is_empty() {
            for cap in self.json_regex.captures_iter(content) {
                if let (Some(name), Some(params)) = (cap.get(1), cap.get(2)) {
                    if let Ok(params_json) = serde_json::from_str::<Value>(params.as_str()) {
                        tool_calls.push(ToolCall {
                            name: name.as_str().to_string(),
                            parameters: params_json,
                        });
                    }
                }
            }
        }

        // If still no calls found, try function-style
        if tool_calls.is_empty() {
            for cap in self.function_regex.captures_iter(content) {
                if let (Some(name), Some(params)) = (cap.get(1), cap.get(2)) {
                    if let Ok(params_json) = serde_json::from_str::<Value>(params.as_str()) {
                        tool_calls.push(ToolCall {
                            name: name.as_str().to_string(),
                            parameters: params_json,
                        });
                    }
                }
            }
        }

        tool_calls
    }

    /// Parse a more structured format with explicit tool use blocks
    pub fn parse_structured(&self, content: &str) -> Result<(String, Vec<ToolCall>)> {
        let mut tool_calls = Vec::new();
        let mut text_parts = Vec::new();
        let mut last_end = 0;

        // Look for tool use blocks first
        let tool_block_regex = Regex::new(r"(?s)<tool_use>\s*(.+?)\s*</tool_use>")?;

        for mat in tool_block_regex.find_iter(content) {
            // Add text before this tool block
            text_parts.push(&content[last_end..mat.start()]);
            last_end = mat.end();

            // Parse the tool block
            let block_content = &content[mat.start()+10..mat.end()-11]; // Skip tags
            let calls = self.parse(block_content);
            tool_calls.extend(calls);
        }

        // If no tool use blocks found, try parsing the entire content for tool calls
        if tool_calls.is_empty() {
            tool_calls = self.parse(content);
        }

        // Add remaining text
        text_parts.push(&content[last_end..]);

        let clean_text = text_parts.join("").trim().to_string();
        Ok((clean_text, tool_calls))
    }
}

/// Format tool results for inclusion in conversation
pub fn format_tool_results(results: &[(String, Result<Value, String>)]) -> String {
    let mut formatted = String::new();

    for (tool_name, result) in results {
        formatted.push_str(&format!("\n<tool_result name=\"{}\">\n", tool_name));

        match result {
            Ok(value) => {
                let s = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
                // Truncate overly long outputs to keep conversation reliable
                const MAX_OUTPUT_CHARS: usize = 6000;
                if s.len() > MAX_OUTPUT_CHARS {
                    let mut truncated = s[..MAX_OUTPUT_CHARS].to_string();
                    truncated.push_str("\n… (truncated)\n");
                    formatted.push_str(&truncated);
                } else {
                    formatted.push_str(&s);
                }
            }
            Err(error) => {
                formatted.push_str(&format!("Error: {}", error));
            }
        }

        formatted.push_str("\n</tool_result>\n");
    }

    formatted
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_xml_style_parsing() {
        let parser = ToolCallParser::new().unwrap();
        let content = r#"I'll read the file for you.

<tool>read_file</tool><params>{"path": "src/main.rs", "start_line": 1}</params>

Now let me check another file:

<tool>list_files</tool><params>{"path": "src", "recursive": false}</params>"#;

        let calls = parser.parse(content);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "read_file");
        assert_eq!(calls[0].parameters["path"], "src/main.rs");
        assert_eq!(calls[1].name, "list_files");
    }

    #[test]
    fn test_json_style_parsing() {
        let parser = ToolCallParser::new().unwrap();
        let content = r#"I'll help you with that. Let me read the file:

{"tool": "read_file", "params": {"path": "config.toml"}}

And then list the directory:

{"tool": "list_files", "params": {"path": ".", "include_hidden": true}}"#;

        let calls = parser.parse(content);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "read_file");
        assert_eq!(calls[1].parameters["include_hidden"], true);
    }

    #[test]
    fn test_function_style_parsing() {
        let parser = ToolCallParser::new().unwrap();
        let content = r#"Let me check what's in the directory:

list_files({"path": "/home/user/project"})

And read the README:

read_file({"path": "README.md"})"#;

        let calls = parser.parse(content);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "list_files");
        assert_eq!(calls[1].name, "read_file");
    }

    #[test]
    fn test_openai_style_parsing() {
        let parser = ToolCallParser::new().unwrap();

        // Test with "parameters" field (llama3.1 style)
        let content1 = r#"{"name": "list_files", "parameters": {"path": "src/providers/", "recursive": true}}"#;
        let calls1 = parser.parse(content1);
        assert_eq!(calls1.len(), 1);
        assert_eq!(calls1[0].name, "list_files");
        assert_eq!(calls1[0].parameters["path"], "src/providers/");
        assert_eq!(calls1[0].parameters["recursive"], true);

        // Test with "arguments" field (qwen2.5-coder style)
        let content2 = r#"{"name": "read_file", "arguments": {"path": "src/main.rs", "start_line": 1}}"#;
        let calls2 = parser.parse(content2);
        assert_eq!(calls2.len(), 1);
        assert_eq!(calls2[0].name, "read_file");
        assert_eq!(calls2[0].parameters["path"], "src/main.rs");
        assert_eq!(calls2[0].parameters["start_line"], 1);
    }

    #[test]
    fn test_structured_parsing() {
        let parser = ToolCallParser::new().unwrap();
        let content = r#"I'll help you fix that bug. Let me first look at the code.

<tool_use>
<tool>read_file</tool><params>{"path": "src/bug.rs"}</params>
</tool_use>

Now I understand the issue. Let me fix it:

<tool_use>
<tool>edit_file</tool><params>{"path": "src/bug.rs", "search": "old_code", "replace": "new_code"}</params>
</tool_use>

The bug should now be fixed!"#;

        let (text, calls) = parser.parse_structured(content).unwrap();
        assert_eq!(calls.len(), 2);
        assert!(text.contains("I'll help you fix that bug"));
        assert!(text.contains("The bug should now be fixed!"));
        assert!(!text.contains("<tool_use>"));
    }

    #[test]
    fn test_format_tool_results() {
        let results = vec![
            ("read_file".to_string(), Ok(json!({
                "content": "fn main() {}\n",
                "lines": 1
            }))),
            ("list_files".to_string(), Err("Permission denied".to_string())),
        ];

        let formatted = format_tool_results(&results);
        assert!(formatted.contains("tool_result name=\"read_file\""));
        assert!(formatted.contains("fn main()"));
        assert!(formatted.contains("Error: Permission denied"));
    }
}
