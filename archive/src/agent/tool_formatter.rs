use serde_json::Value;
use crate::agent::tools::ToolCall;

// Helpers
fn shorten_path(path: &str, max: usize) -> String {
    if path.len() <= max {
        return path.to_string();
    }
    // Preserve the tail (filename) when possible
    let parts: Vec<&str> = path.split('/').collect();
    if let Some(last) = parts.last() {
        // Show leading ellipsis and last component
        let prefix = if parts.len() > 2 { "…/" } else { "" };
        let mut s = format!("{}{}", prefix, last);
        if s.len() > max {
            // Truncate filename too
            s.truncate(max.saturating_sub(1));
            s.push('…');
        }
        s
    } else {
        let mut s = path.to_string();
        s.truncate(max.saturating_sub(1));
        s.push('…');
        s
    }
}

fn join_args(args: &Value, max: usize) -> String {
    let joined = args
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(" "))
        .unwrap_or_default();
    if joined.len() <= max {
        joined
    } else {
        let mut s = joined;
        s.truncate(max.saturating_sub(1));
        s.push('…');
        s
    }
}

/// Format tool calls into user-friendly status messages
pub fn format_tool_status(tool_name: &str, params: &Value, executing: bool) -> String {
    // Consistent: symbol tool target — state
    // Running:  "🔧 read_file Cargo.toml — running…"
    // Finished: handled by format_tool_result
    let icon = if executing { "🔧" } else { "✓" };
    let target = match tool_name {
        "read_file" | "write_file" | "edit_file" | "list_files" => {
            params.get("path").and_then(|p| p.as_str()).map(|p| shorten_path(p, 48)).unwrap_or_default()
        }
        "search_code" => params.get("query").and_then(|q| q.as_str()).map(|q| {
            let mut s = q.to_string();
            if s.len() > 48 { s.truncate(47); s.push('…'); }
            s
        }).unwrap_or_default(),
        "run_command" => {
            let cmd = params.get("command").and_then(|c| c.as_str()).unwrap_or("");
            let args = join_args(params.get("args").unwrap_or(&Value::Null), 40);
            if args.is_empty() { cmd.to_string() } else { format!("{} {}", cmd, args) }
        }
        _ => String::new(),
    };
    let state = if executing { "running…" } else { "done" };
    if target.is_empty() {
        format!("{} {} — {}", icon, tool_name, state)
    } else {
        format!("{} {} {} — {}", icon, tool_name, target, state)
    }
}

/// Format tool results into concise summaries
pub fn format_tool_result(tool_name: &str, result: &Result<Value, String>) -> String {
    match result {
        Ok(value) => format_tool_success(tool_name, value),
        Err(error) => format!("✗ {} — {}", tool_name, error),
    }
}

fn format_tool_success(tool_name: &str, value: &Value) -> String {
    let dur = value.get("duration_ms").and_then(|d| d.as_u64());
    let with_dur = |base: String| -> String {
        if let Some(ms) = dur { format!("{} ({}ms)", base, ms) } else { base }
    };
    match tool_name {
        "read_file" => {
            if let Some(content) = value.get("content").and_then(|c| c.as_str()) {
                let lines = content.lines().count();
                with_dur(format!("✓ read_file — {} lines", lines))
            } else {
                with_dur("✓ read_file — ok".to_string())
            }
        }
        "write_file" => {
            if let Some(bytes) = value.get("bytes_written").and_then(|b| b.as_u64()) {
                with_dur(format!("✓ write_file — {} bytes", bytes))
            } else {
                with_dur("✓ write_file — ok".to_string())
            }
        }
        "edit_file" => {
            if let Some(changes) = value.get("changes_made").and_then(|c| c.as_u64()) {
                with_dur(format!("✓ edit_file — {} changes", changes))
            } else {
                with_dur("✓ edit_file — ok".to_string())
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
            with_dur(format!("✓ list_files — {} files, {} dirs", file_count, dir_count))
        }
        "search_code" => {
            if let Some(results) = value.get("results").and_then(|r| r.as_array()) {
                with_dur(format!("✓ search_code — {} matches", results.len()))
            } else {
                with_dur("✓ search_code — ok".to_string())
            }
        }
        "run_command" => {
            let exit_code = value.get("exit_code").and_then(|c| c.as_i64());
            let success = value.get("success").and_then(|s| s.as_bool()).unwrap_or(false);

            if success {
                with_dur("✓ run_command — ok".to_string())
            } else if let Some(code) = exit_code {
                format!("✗ run_command — exit {}", code)
            } else {
                "✗ run_command — failed".to_string()
            }
        }
        _ => "✓ tool — ok".to_string(),
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
