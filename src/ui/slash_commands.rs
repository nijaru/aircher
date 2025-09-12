#[derive(Debug, Clone)]
pub struct SlashCommand {
    pub command: &'static str,
    pub description: &'static str,
    pub aliases: &'static [&'static str],
}

pub const SLASH_COMMANDS: &[SlashCommand] = &[
    SlashCommand {
        command: "/init",
        description: "Initialize project with AGENT.md configuration",
        aliases: &[],
    },
    SlashCommand {
        command: "/model",
        description: "Select model/provider (Tab to switch modes)",
        aliases: &["/m"],
    },
    SlashCommand {
        command: "/provider",
        description: "Quick provider selection",
        aliases: &["/p"],
    },
    SlashCommand {
        command: "/search",
        description: "Semantic code search (works without LLM!)",
        aliases: &["/s"],
    },
    SlashCommand {
        command: "/help",
        description: "Show available commands",
        aliases: &["/h", "/?"],
    },
    SlashCommand {
        command: "/clear",
        description: "Clear conversation history",
        aliases: &["/c"],
    },
    SlashCommand {
        command: "/config",
        description: "Open settings (API keys, preferences)",
        aliases: &["/settings"],
    },
    SlashCommand {
        command: "/auth",
        description: "Setup API keys with guided wizard",
        aliases: &["/login"],
    },
    SlashCommand {
        command: "/sessions",
        description: "Browse previous sessions",
        aliases: &[],
    },
    SlashCommand {
        command: "/save",
        description: "Save current conversation with optional name",
        aliases: &[],
    },
    SlashCommand {
        command: "/load",
        description: "Load a saved conversation",
        aliases: &["/open"],
    },
    SlashCommand {
        command: "/share",
        description: "Share current conversation (create shareable link)",
        aliases: &[],
    },
    SlashCommand {
        command: "/export",
        description: "Export conversation to JSON file",
        aliases: &[],
    },
    SlashCommand {
        command: "/import",
        description: "Import conversation from JSON file",
        aliases: &[],
    },
    SlashCommand {
        command: "/compact",
        description: "Smart conversation compaction (auto-analyzes context)",
        aliases: &[],
    },
    SlashCommand {
        command: "/turbo",
        description: "Toggle autonomous turbo mode",
        aliases: &["/t"],
    },
    SlashCommand {
        command: "/quit",
        description: "Exit application",
        aliases: &["/exit", "/q"],
    },
];

/// Parse a slash command from user input
pub fn parse_slash_command(input: &str) -> Option<(&'static str, &str)> {
    if !input.starts_with('/') {
        return None;
    }
    
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let command_part = parts[0];
    let args = parts.get(1).unwrap_or(&"");
    
    // Find matching command or alias
    for cmd in SLASH_COMMANDS {
        if cmd.command == command_part || cmd.aliases.contains(&command_part) {
            return Some((cmd.command, args));
        }
    }
    
    None
}

/// Get slash command suggestions based on partial input
pub fn get_command_suggestions(partial: &str) -> Vec<&'static SlashCommand> {
    if !partial.starts_with('/') {
        return Vec::new();
    }
    
    SLASH_COMMANDS
        .iter()
        .filter(|cmd| {
            cmd.command.starts_with(partial) || 
            cmd.aliases.iter().any(|alias| alias.starts_with(partial))
        })
        .collect()
}

/// Format help text for all commands
pub fn format_help() -> Vec<String> {
    let mut lines = Vec::new();
    
    lines.push("Available commands:".to_string());
    lines.push(String::new());
    
    // Find the longest command for alignment
    let max_len = SLASH_COMMANDS.iter()
        .map(|cmd| cmd.command.len())
        .max()
        .unwrap_or(0);
    
    for cmd in SLASH_COMMANDS {
        let padding = " ".repeat(max_len - cmd.command.len());
        lines.push(format!("  {}{}  {}", cmd.command, padding, cmd.description));
        
        if !cmd.aliases.is_empty() {
            let alias_str = cmd.aliases.join(", ");
            lines.push(format!("  {}  Aliases: {}", " ".repeat(max_len), alias_str));
        }
    }
    
    lines.push(String::new());
    lines.push("Keyboard Shortcuts:".to_string());
    lines.push("  • Type / to see command suggestions".to_string());
    lines.push("  • Alt+Enter (or Shift+Enter) for newlines".to_string());
    lines.push("  • Ctrl+C to clear input, Ctrl+C again to quit (requires confirmation during message processing)".to_string());
    lines.push("  • Ctrl+W to delete last word".to_string());
    lines.push("  • Ctrl+M to open model selection".to_string());
    lines.push("  • Tab to autocomplete commands".to_string());
    lines.push("  • Shift+Tab to cycle modes (auto-accept, plan mode)".to_string());
    lines.push("  • F1 for help, F2 for settings".to_string());
    lines.push("  • Up/Down arrows to navigate message history".to_string());
    lines.push(String::new());
    lines.push("Scrolling:".to_string());
    lines.push("  • PageUp/PageDown to scroll chat (with overlap)".to_string());
    lines.push("  • Mouse wheel to scroll".to_string());
    lines.push("  • Ctrl+L or End key to jump to bottom".to_string());
    
    lines.push(String::new());
    lines.push("Modes:".to_string());
    lines.push("  • Default: Prompt before making file changes".to_string());
    lines.push("  • Auto-accept: Apply file changes automatically".to_string());
    lines.push("  • Plan mode: Build execution plans before changes".to_string());
    
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slash_command_parsing() {
        // Valid commands
        assert_eq!(parse_slash_command("/help"), Some(("/help", "")));
        assert_eq!(parse_slash_command("/model gpt-4"), Some(("/model", "gpt-4")));
        assert_eq!(parse_slash_command("/search test query"), Some(("/search", "test query")));
        
        // Aliases  
        assert_eq!(parse_slash_command("/h"), Some(("/help", "")));
        assert_eq!(parse_slash_command("/m claude"), Some(("/model", "claude")));
        assert_eq!(parse_slash_command("/s find this"), Some(("/search", "find this")));
        
        // Invalid commands
        assert_eq!(parse_slash_command("/unknown"), None);
        assert_eq!(parse_slash_command("not a command"), None);
        assert_eq!(parse_slash_command(""), None);
    }

    #[test]
    fn test_command_suggestions() {
        let suggestions = get_command_suggestions("/h");
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.command == "/help"));
        
        let suggestions = get_command_suggestions("/m");
        assert!(suggestions.iter().any(|s| s.command == "/model"));
        
        let suggestions = get_command_suggestions("/se");
        assert!(suggestions.iter().any(|s| s.command == "/search"));
        
        // No suggestions for non-slash input
        let suggestions = get_command_suggestions("test");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_help_formatting() {
        let help_lines = format_help();
        assert!(!help_lines.is_empty());
        
        // Should contain available commands header
        assert!(help_lines.iter().any(|line| line.contains("Available commands")));
        
        // Should contain some known commands
        assert!(help_lines.iter().any(|line| line.contains("/help")));
        assert!(help_lines.iter().any(|line| line.contains("/model")));
        assert!(help_lines.iter().any(|line| line.contains("/search")));
        
        // Should contain keyboard shortcuts section
        assert!(help_lines.iter().any(|line| line.contains("Keyboard Shortcuts")));
        assert!(help_lines.iter().any(|line| line.contains("Shift+Tab")));
        
        // Should contain modes section
        assert!(help_lines.iter().any(|line| line.contains("Modes:")));
        assert!(help_lines.iter().any(|line| line.contains("Auto-accept")));
        assert!(help_lines.iter().any(|line| line.contains("Plan mode")));
    }
}