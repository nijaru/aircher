#[derive(Debug, Clone)]
pub struct SlashCommand {
    pub command: &'static str,
    pub description: &'static str,
    pub aliases: &'static [&'static str],
}

pub const SLASH_COMMANDS: &[SlashCommand] = &[
    SlashCommand {
        command: "/model",
        description: "Select model/provider (Tab to switch modes)",
        aliases: &["/m"],
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
        command: "/sessions",
        description: "Browse previous sessions",
        aliases: &[],
    },
    SlashCommand {
        command: "/compact",
        description: "Summarize conversation to save context",
        aliases: &[],
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
pub fn format_help() -> String {
    let mut help = String::from("Available commands:\n\n");
    
    for cmd in SLASH_COMMANDS {
        help.push_str(&format!("  {} - {}\n", cmd.command, cmd.description));
        if !cmd.aliases.is_empty() {
            help.push_str(&format!("    Aliases: {}\n", cmd.aliases.join(", ")));
        }
    }
    
    help.push_str("\nTips:\n");
    help.push_str("  • Type / to see command suggestions\n");
    help.push_str("  • Use Tab to autocomplete commands\n");
    help.push_str("  • Press F2 for settings, F3 for model selection\n");
    
    help
}