use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::ListItem,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enhanced message structure that supports collapsible tool results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollapsibleToolResult {
    pub tool_name: String,
    pub parameters: String,
    pub result: String,
    pub success: bool,
    pub duration_ms: Option<u64>,
    pub suggestions: Vec<String>,
    pub collapsed: bool,
    pub id: String,
}

/// State manager for collapsible tool results
pub struct CollapsibleToolManager {
    pub collapsed_states: HashMap<String, bool>,
    pub selected_tool_id: Option<String>,
}

impl CollapsibleToolManager {
    pub fn new() -> Self {
        Self {
            collapsed_states: HashMap::new(),
            selected_tool_id: None,
        }
    }

    /// Toggle collapse state for a tool result
    pub fn toggle_collapse(&mut self, tool_id: &str) {
        let current_state = self.collapsed_states.get(tool_id).unwrap_or(&true);
        self.collapsed_states.insert(tool_id.to_string(), !current_state);
    }

    /// Check if a tool result is collapsed
    pub fn is_collapsed(&self, tool_id: &str) -> bool {
        self.collapsed_states.get(tool_id).unwrap_or(&true).clone()
    }

    /// Set selected tool for keyboard navigation
    pub fn set_selected(&mut self, tool_id: Option<String>) {
        self.selected_tool_id = tool_id;
    }

    /// Get currently selected tool ID
    pub fn get_selected(&self) -> Option<&String> {
        self.selected_tool_id.as_ref()
    }
}

impl CollapsibleToolResult {
    pub fn new(
        tool_name: String, 
        parameters: String, 
        result: String, 
        success: bool,
        duration_ms: Option<u64>
    ) -> Self {
        Self {
            id: format!("{}_{}", tool_name, chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)),
            tool_name,
            parameters,
            result,
            success,
            duration_ms,
            suggestions: Vec::new(),
            collapsed: true, // Start collapsed by default
        }
    }

    /// Generate a concise summary for collapsed state
    pub fn generate_summary(&self) -> String {
        let status_icon = if self.success { "✓" } else { "✗" };
        let duration_text = if let Some(duration) = self.duration_ms {
            if duration < 1000 {
                format!(" ({}ms)", duration)
            } else {
                format!(" ({:.1}s)", duration as f64 / 1000.0)
            }
        } else {
            String::new()
        };

        // Create intelligent summary based on tool type
        let summary = match self.tool_name.as_str() {
            "read_file" => {
                if self.success {
                    let lines = self.result.lines().count();
                    format!("{} lines", lines)
                } else {
                    "read failed".to_string()
                }
            },
            "write_file" => {
                if self.success {
                    "file updated"
                } else {
                    "write failed"
                }.to_string()
            },
            "run_command" => {
                if self.success {
                    "command completed"
                } else {
                    "command failed"
                }.to_string()
            },
            "search_code" => {
                if self.success {
                    // Try to count results from search output
                    let results = self.result.lines()
                        .filter(|line| line.contains(":"))
                        .count();
                    format!("{} results", results)
                } else {
                    "search failed".to_string()
                }
                },
            _ => {
                if self.success {
                    "completed"
                } else {
                    "failed"
                }.to_string()
            }
        };

        format!("{} {} — {}{}", status_icon, self.tool_name, summary, duration_text)
    }

    /// Render as collapsed list items
    pub fn render_collapsed(&self, is_selected: bool) -> Vec<ListItem<'static>> {
        let summary = self.generate_summary();
        
        // Style based on selection and success state
        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(if self.success { 
                Color::Rgb(144, 238, 144) // Light green for success
            } else { 
                Color::Rgb(255, 182, 193) // Light red for failure
            })
        };

        let expand_indicator = if is_selected { "▶" } else { "▼" };
        
        vec![ListItem::new(Line::from(vec![
            Span::styled("▐ ".to_string(), Style::default().fg(Color::Cyan)), // Cyan bar for tools
            Span::styled(expand_indicator.to_string(), Style::default().fg(Color::Yellow)),
            Span::styled(" ".to_string(), Style::default()),
            Span::styled(summary, style),
        ]))]
    }

    /// Render as expanded list items with syntax highlighting
    pub fn render_expanded(&self, is_selected: bool, _syntax_highlighter: &crate::ui::syntax_highlight::SyntaxHighlighter) -> Vec<ListItem<'static>> {
        let mut items = Vec::new();

        // Header with collapse indicator
        let header_style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(if self.success { 
                Color::Green
            } else { 
                Color::Red
            }).add_modifier(Modifier::BOLD)
        };

        let collapse_indicator = if is_selected { "▼" } else { "▲" };
        let summary = self.generate_summary();

        items.push(ListItem::new(Line::from(vec![
            Span::styled("▐ ".to_string(), Style::default().fg(Color::Cyan)),
            Span::styled(collapse_indicator.to_string(), Style::default().fg(Color::Yellow)),
            Span::styled(" ".to_string(), Style::default()),
            Span::styled(summary, header_style),
        ])));

        // Parameters section (if not empty)
        if !self.parameters.is_empty() && self.parameters != "{}" {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("▐   ".to_string(), Style::default().fg(Color::Cyan)),
                Span::styled("📝 Parameters: ".to_string(), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                Span::styled(self.parameters.clone(), Style::default().fg(Color::Rgb(200, 200, 200))),
            ])));
        }

        // Result content with simple text rendering (avoiding lifetime issues)
        if !self.result.is_empty() {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("▐   ".to_string(), Style::default().fg(Color::Cyan)),
                Span::styled("📤 Result:".to_string(), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            ])));

            // Use simple text rendering to avoid lifetime issues with syntax highlighter
            for line in self.result.lines() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("▐     ".to_string(), Style::default().fg(Color::Cyan)),
                    Span::styled(line.to_string(), Style::default().fg(Color::Rgb(240, 240, 235))),
                ])));
            }
        }

        // Suggestions section (if any)
        if !self.suggestions.is_empty() {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("▐   ".to_string(), Style::default().fg(Color::Cyan)),
                Span::styled("💡 Suggestions:".to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ])));

            for suggestion in &self.suggestions {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("▐     • ".to_string(), Style::default().fg(Color::Cyan)),
                    Span::styled(suggestion.clone(), Style::default().fg(Color::Rgb(255, 255, 150))),
                ])));
            }
        }

        // Add spacing after expanded tool result
        items.push(ListItem::new(Line::from("")));

        items
    }
}

/// Helper function to format tool duration
pub fn format_duration(duration_ms: u64) -> String {
    if duration_ms < 1000 {
        format!("{}ms", duration_ms)
    } else if duration_ms < 60000 {
        format!("{:.1}s", duration_ms as f64 / 1000.0)
    } else {
        let seconds = duration_ms / 1000;
        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        format!("{}m{}s", minutes, remaining_seconds)
    }
}

/// Extract tool results from a message content (if it contains tool calls)
pub fn extract_tool_results_from_content(content: &str) -> Vec<CollapsibleToolResult> {
    let mut results = Vec::new();
    
    // Parse tool execution patterns from agent responses
    // This is a simplified parser - in practice, you'd integrate with the actual tool execution system
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i];
        
        // Look for tool execution patterns
        if line.contains("🔧") && line.contains("—") {
            // Parse tool summary line: "🔧 read_file Cargo.toml — 120 lines (48ms)"
            let parts: Vec<&str> = line.split("—").collect();
            if parts.len() == 2 {
                let tool_part = parts[0].trim();
                let result_part = parts[1].trim();
                
                // Extract tool name
                let tool_name = if tool_part.contains("read_file") {
                    "read_file"
                } else if tool_part.contains("write_file") {
                    "write_file"
                } else if tool_part.contains("run_command") {
                    "run_command"
                } else {
                    "unknown_tool"
                }.to_string();
                
                // Determine success and extract duration
                let success = !result_part.contains("failed") && !result_part.contains("error");
                let duration_ms = extract_duration_from_text(result_part);
                
                // Collect following lines as result content until next tool or end
                let mut result_content = String::new();
                let mut j = i + 1;
                while j < lines.len() && !lines[j].contains("🔧") {
                    if !result_content.is_empty() {
                        result_content.push('\n');
                    }
                    result_content.push_str(lines[j]);
                    j += 1;
                }
                
                let tool_result = CollapsibleToolResult::new(
                    tool_name,
                    "{}".to_string(), // Parameters would come from actual tool execution
                    result_content.trim().to_string(),
                    success,
                    duration_ms,
                );
                
                results.push(tool_result);
                i = j;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    
    results
}

/// Extract duration from text like "(48ms)" or "(1.2s)"
fn extract_duration_from_text(text: &str) -> Option<u64> {
    use regex::Regex;
    
    // Try to find duration patterns
    if let Ok(re) = Regex::new(r"\((\d+(?:\.\d+)?)([ms]?)s?\)") {
        if let Some(captures) = re.captures(text) {
            if let Some(number_str) = captures.get(1) {
                if let Ok(number) = number_str.as_str().parse::<f64>() {
                    let unit = captures.get(2).map(|m| m.as_str()).unwrap_or("ms");
                    return Some(match unit {
                        "s" => (number * 1000.0) as u64,
                        _ => number as u64, // Default to ms
                    });
                }
            }
        }
    }
    
    None
}