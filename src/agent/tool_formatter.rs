use serde_json::Value;
use crate::agent::tools::ToolCall;

/// Format tool calls into user-friendly status messages
pub fn format_tool_status(tool_name: &str, params: &Value, executing: bool) -> String {
    let icon = if executing { "🔧" } else { "✓" };
    
    match tool_name {
        "read_file" => {
            if let Some(path) = params.get("path").and_then(|p| p.as_str()) {
                format!("{} Reading file: {}", icon, path)
            } else {
                format!("{} Reading file", icon)
            }
        }
        "write_file" => {
            if let Some(path) = params.get("path").and_then(|p| p.as_str()) {
                format!("{} Writing file: {}", icon, path)
            } else {
                format!("{} Writing file", icon)
            }
        }
        "edit_file" => {
            if let Some(path) = params.get("path").and_then(|p| p.as_str()) {
                format!("{} Editing file: {}", icon, path)
            } else {
                format!("{} Editing file", icon)
            }
        }
        "list_files" => {
            if let Some(path) = params.get("path").and_then(|p| p.as_str()) {
                let recursive = params.get("recursive").and_then(|r| r.as_bool()).unwrap_or(false);
                if recursive {
                    format!("{} Listing directory recursively: {}", icon, path)
                } else {
                    format!("{} Listing directory: {}", icon, path)
                }
            } else {
                format!("{} Listing directory", icon)
            }
        }
        "search_code" => {
            if let Some(query) = params.get("query").and_then(|q| q.as_str()) {
                format!("{} Searching for: {}", icon, query)
            } else {
                format!("{} Searching code", icon)
            }
        }
        "run_command" => {
            if let Some(cmd) = params.get("command").and_then(|c| c.as_str()) {
                let args = params.get("args")
                    .and_then(|a| a.as_array())
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(" "))
                    .unwrap_or_default();
                
                if args.is_empty() {
                    format!("{} Running command: {}", icon, cmd)
                } else {
                    format!("{} Running command: {} {}", icon, cmd, args)
                }
            } else {
                format!("{} Running command", icon)
            }
        }
        _ => format!("{} Using tool: {}", icon, tool_name),
    }
}

/// Format tool results into concise summaries
pub fn format_tool_result(tool_name: &str, result: &Result<Value, String>) -> String {
    match result {
        Ok(value) => format_tool_success(tool_name, value),
        Err(error) => format!("✗ {}: {}", tool_name, error),
    }
}

fn format_tool_success(tool_name: &str, value: &Value) -> String {
    match tool_name {
        "read_file" => {
            if let Some(content) = value.get("content").and_then(|c| c.as_str()) {
                let lines = content.lines().count();
                format!("✓ Read {} lines", lines)
            } else {
                "✓ File read successfully".to_string()
            }
        }
        "write_file" => {
            if let Some(bytes) = value.get("bytes_written").and_then(|b| b.as_u64()) {
                format!("✓ Wrote {} bytes", bytes)
            } else {
                "✓ File written successfully".to_string()
            }
        }
        "edit_file" => {
            if let Some(changes) = value.get("changes_made").and_then(|c| c.as_u64()) {
                format!("✓ Made {} changes", changes)
            } else {
                "✓ File edited successfully".to_string()
            }
        }
        "list_files" => {
            let file_count = value.get("files")
                .and_then(|f| f.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            let dir_count = value.get("directories")
                .and_then(|d| d.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            
            format!("✓ Found {} files, {} directories", file_count, dir_count)
        }
        "search_code" => {
            if let Some(results) = value.get("results").and_then(|r| r.as_array()) {
                format!("✓ Found {} matches", results.len())
            } else {
                "✓ Search completed".to_string()
            }
        }
        "run_command" => {
            let exit_code = value.get("exit_code").and_then(|c| c.as_i64());
            let success = value.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
            
            if success {
                "✓ Command executed successfully".to_string()
            } else if let Some(code) = exit_code {
                format!("✗ Command failed with exit code {}", code)
            } else {
                "✗ Command failed".to_string()
            }
        }
        _ => "✓ Tool executed successfully".to_string(),
    }
}

/// Format multiple tool calls into a summary message
pub fn format_tool_batch(tool_calls: &[ToolCall]) -> String {
    if tool_calls.is_empty() {
        return String::new();
    }
    
    if tool_calls.len() == 1 {
        format_tool_status(&tool_calls[0].name, &tool_calls[0].parameters, true)
    } else {
        format!("🔧 Executing {} tools...", tool_calls.len())
    }
}