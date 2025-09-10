use anyhow::Result;
use std::collections::HashMap;
use tracing::debug;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Zed-style predictive autocomplete for TUI chat interface
pub struct AutocompleteEngine {
    pub suggestions: Vec<Suggestion>,
    selected_index: usize,
    is_visible: bool,
    #[allow(dead_code)]
    common_patterns: HashMap<String, Vec<String>>,
    recent_commands: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub text: String,
    pub completion: String,
    pub description: String,
    pub suggestion_type: SuggestionType,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionType {
    Command,         // e.g., "/search", "/help"
    CodeContext,     // e.g., "fix the bug in src/main.rs"
    Question,        // e.g., "How do I implement..."
    FileReference,   // e.g., "Look at config.toml"
    CommonPhrase,    // e.g., "explain", "refactor", "optimize"
}

impl AutocompleteEngine {
    pub fn new() -> Self {
        let mut common_patterns = HashMap::new();
        
        // Programming commands
        common_patterns.insert("fix".to_string(), vec![
            "fix the bug in".to_string(),
            "fix the error".to_string(),
            "fix the test".to_string(),
            "fix the compilation error".to_string(),
        ]);
        
        common_patterns.insert("explain".to_string(), vec![
            "explain this code".to_string(),
            "explain how to".to_string(),
            "explain the error".to_string(),
            "explain why".to_string(),
        ]);
        
        common_patterns.insert("how".to_string(), vec![
            "how do I".to_string(),
            "how to implement".to_string(),
            "how to fix".to_string(),
            "how does this work".to_string(),
        ]);
        
        common_patterns.insert("refactor".to_string(), vec![
            "refactor this code".to_string(),
            "refactor the function".to_string(),
            "refactor to use".to_string(),
        ]);
        
        common_patterns.insert("optimize".to_string(), vec![
            "optimize this code".to_string(),
            "optimize performance".to_string(),
            "optimize memory usage".to_string(),
        ]);
        
        common_patterns.insert("add".to_string(), vec![
            "add a function".to_string(),
            "add error handling".to_string(),
            "add tests for".to_string(),
            "add documentation".to_string(),
        ]);
        
        common_patterns.insert("create".to_string(), vec![
            "create a new".to_string(),
            "create a function".to_string(),
            "create a module".to_string(),
            "create tests".to_string(),
        ]);
        
        common_patterns.insert("debug".to_string(), vec![
            "debug this issue".to_string(),
            "debug the error".to_string(),
            "debug why".to_string(),
        ]);
        
        // File operations
        common_patterns.insert("look".to_string(), vec![
            "look at".to_string(),
            "look in".to_string(),
        ]);
        
        common_patterns.insert("check".to_string(), vec![
            "check the".to_string(),
            "check if".to_string(),
            "check for errors".to_string(),
        ]);
        
        // Commands - removed since we'll get these from SLASH_COMMANDS
        
        Self {
            suggestions: Vec::new(),
            selected_index: 0,
            is_visible: false,
            common_patterns,
            recent_commands: Vec::new(),
        }
    }
    
    /// Generate suggestions based on current input
    pub fn generate_suggestions(&mut self, input: &str, cursor_position: usize) -> Result<()> {
        self.suggestions.clear();
        self.selected_index = 0;
        
        // Bounds check cursor position
        let safe_cursor_position = cursor_position.min(input.len());
        
        // Debug logging
        debug!("=== Generating suggestions ===");
        debug!("Input: '{}', cursor: {} (safe: {})", input, cursor_position, safe_cursor_position);
        
        // Special case: show slash commands immediately when typing /
        if input == "/" || input.starts_with('/') {
            debug!("Input starts with slash, processing slash commands");
            let current_word = self.get_current_word(input, safe_cursor_position);
            debug!("Current word extracted: '{}'", current_word);
            self.add_command_suggestions(&current_word);
            
            // Sort slash command suggestions by confidence too!
            self.suggestions.sort_by(|a, b| {
                // First compare by confidence (b.confidence vs a.confidence for descending order)
                let conf_cmp = b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal);
                if conf_cmp != std::cmp::Ordering::Equal {
                    return conf_cmp;
                }
                
                // If confidence is equal, prefer shorter commands (they're usually more common)
                let len_cmp = a.completion.len().cmp(&b.completion.len());
                if len_cmp != std::cmp::Ordering::Equal {
                    return len_cmp;
                }
                
                // If length is equal, sort alphabetically
                a.completion.cmp(&b.completion)
            });
            
            // Limit to top 8 suggestions
            self.suggestions.truncate(8);
            
            self.is_visible = !self.suggestions.is_empty();
            debug!("Generated {} slash command suggestions, visible: {}", self.suggestions.len(), self.is_visible);
            debug!("Suggestions: {:?}", self.suggestions.iter().map(|s| &s.completion).collect::<Vec<_>>());
            return Ok(());
        }
        
        // Only show autocomplete for slash commands, not regular text
        self.is_visible = false;
        Ok(())
    }
    
    fn get_current_word(&self, input: &str, cursor_position: usize) -> String {
        // Ensure cursor position is within bounds
        let safe_cursor_position = cursor_position.min(input.len());
        
        if safe_cursor_position == 0 {
            return String::new();
        }
        
        let before_cursor = &input[..safe_cursor_position];
        
        // Find the start of the current word
        // Special handling for slash commands - we want to include the /
        if before_cursor.starts_with('/') {
            // For slash commands, find the start from the beginning or last whitespace
            let word_start = before_cursor.rfind(|c: char| c.is_whitespace())
                .map(|pos| pos + 1)
                .unwrap_or(0);
            return before_cursor[word_start..].to_string();
        }
        
        // For regular words, use the original logic
        let word_start = before_cursor.rfind(|c: char| c.is_whitespace() || c == '/')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        
        before_cursor[word_start..].to_string()
    }
    
    fn add_command_suggestions(&mut self, current_word: &str) {
        if current_word.starts_with('/') {
            // Import SLASH_COMMANDS from slash_commands module
            use crate::ui::slash_commands::SLASH_COMMANDS;
            
            debug!("Adding command suggestions for: '{}'", current_word);
            debug!("Total SLASH_COMMANDS available: {}", SLASH_COMMANDS.len());
            
            // Convert to lowercase for case-insensitive matching
            let search_term = current_word.to_lowercase();
            
            for cmd in SLASH_COMMANDS {
                // Check multiple matching strategies
                let exact_match = cmd.command.starts_with(current_word);
                let case_insensitive_match = cmd.command.to_lowercase().starts_with(&search_term);
                let alias_match = cmd.aliases.iter().any(|alias| {
                    let exact_alias = alias == &current_word;
                    let starts_alias = alias.starts_with(current_word);
                    let case_alias = alias.to_lowercase().starts_with(&search_term);
                    exact_alias || starts_alias || case_alias
                });
                
                // For fuzzy matching, don't match if we already have exact/case-insensitive/alias matches
                let should_check_fuzzy = !exact_match && !case_insensitive_match && !alias_match;
                
                // Fuzzy matching: check if all chars in search term appear in order in command
                let fuzzy_match = if should_check_fuzzy && search_term.len() > 1 {
                    let search_chars_vec: Vec<char> = search_term.chars().skip(1).collect(); // Skip the '/'
                    if search_chars_vec.is_empty() {
                        false
                    } else {
                        let cmd_chars: Vec<char> = cmd.command.to_lowercase().chars().skip(1).collect(); // Skip the '/'
                        let mut search_idx = 0;
                        
                        for cmd_char in cmd_chars {
                            if search_idx < search_chars_vec.len() && cmd_char == search_chars_vec[search_idx] {
                                search_idx += 1;
                                if search_idx == search_chars_vec.len() {
                                    break; // Found all characters
                                }
                            }
                        }
                        search_idx == search_chars_vec.len() // All characters were found
                    }
                } else {
                    false
                };
                
                let matches = exact_match || case_insensitive_match || alias_match || fuzzy_match;
                
                debug!("Checking '{}' against '{}': exact={}, case_insensitive={}, alias={}, fuzzy={}, matches={}", 
                    cmd.command, current_word, exact_match, case_insensitive_match, alias_match, fuzzy_match, matches);
                
                if matches {
                    // Calculate confidence based on match type and position
                    let confidence = if exact_match {
                        1.0
                    } else if case_insensitive_match {
                        // Prioritize commands that start with the search term
                        if cmd.command.to_lowercase().starts_with(&search_term) {
                            0.95
                        } else {
                            0.7 // Lower confidence for non-prefix matches
                        }
                    } else if alias_match {
                        // Exact alias match should have very high confidence
                        if cmd.aliases.iter().any(|alias| alias == &current_word) {
                            1.0  // Exact alias match
                        } else {
                            0.9  // Prefix alias match
                        }
                    } else {
                        // Fuzzy match - give lower confidence
                        // But boost if it's a prefix match
                        if cmd.command.to_lowercase().starts_with(&search_term) {
                            0.85
                        } else {
                            // Even lower for fuzzy matches that aren't prefix matches
                            0.5
                        }
                    };
                    
                    self.suggestions.push(Suggestion {
                        text: current_word.to_string(),
                        completion: cmd.command.to_string(),
                        description: cmd.description.to_string(),
                        suggestion_type: SuggestionType::Command,
                        confidence,
                    });
                    debug!("Added suggestion: {} with confidence {} (total suggestions: {})", 
                        cmd.command, confidence, self.suggestions.len());
                }
            }
            debug!("Finished adding command suggestions. Total: {}", self.suggestions.len());
        } else {
            debug!("Current word '{}' doesn't start with /, skipping command suggestions", current_word);
        }
    }
    
    #[allow(dead_code)] // TODO: Implement pattern suggestions
    fn add_pattern_suggestions(&mut self, prefix: &str, current_word: &str) {
        if current_word.len() < 2 {
            return;
        }
        
        for (pattern, completions) in &self.common_patterns {
            if pattern.starts_with(&current_word.to_lowercase()) {
                for completion in completions {
                    // Check if this would make sense in context
                    let confidence = self.calculate_pattern_confidence(prefix, completion);
                    
                    if confidence > 0.3 {
                        self.suggestions.push(Suggestion {
                            text: current_word.to_string(),
                            completion: completion.clone(),
                            description: format!("Common phrase: {}", completion),
                            suggestion_type: SuggestionType::CommonPhrase,
                            confidence,
                        });
                    }
                }
            }
        }
    }
    
    #[allow(dead_code)] // TODO: Implement recent command suggestions
    fn add_recent_command_suggestions(&mut self, current_word: &str) {
        for recent in &self.recent_commands {
            if recent.to_lowercase().starts_with(&current_word.to_lowercase()) && recent != current_word {
                self.suggestions.push(Suggestion {
                    text: current_word.to_string(),
                    completion: recent.clone(),
                    description: "Recent command".to_string(),
                    suggestion_type: SuggestionType::Question,
                    confidence: 0.7,
                });
            }
        }
    }
    
    #[allow(dead_code)] // TODO: Implement contextual suggestions
    fn add_contextual_suggestions(&mut self, input: &str) {
        let lowercase_input = input.to_lowercase();
        
        // File references
        if lowercase_input.contains("src/") || lowercase_input.contains("config") || lowercase_input.contains("cargo") {
            if !input.ends_with(".rs") && !input.ends_with(".toml") {
                self.suggestions.push(Suggestion {
                    text: input.to_string(),
                    completion: format!("{} file", input),
                    description: "File reference".to_string(),
                    suggestion_type: SuggestionType::FileReference,
                    confidence: 0.6,
                });
            }
        }
        
        // Code context suggestions
        if lowercase_input.contains("error") || lowercase_input.contains("bug") {
            let suggestions = vec![
                "What's the error message?",
                "Can you show the stack trace?",
                "Which file has the error?",
            ];
            
            for suggestion in suggestions {
                self.suggestions.push(Suggestion {
                    text: input.to_string(),
                    completion: suggestion.to_string(),
                    description: "Debug question".to_string(),
                    suggestion_type: SuggestionType::Question,
                    confidence: 0.5,
                });
            }
        }
        
        // Implementation suggestions
        if lowercase_input.contains("implement") || lowercase_input.contains("create") {
            let suggestions = vec![
                "What functionality do you need?",
                "Should I add error handling?",
                "Do you need tests for this?",
            ];
            
            for suggestion in suggestions {
                self.suggestions.push(Suggestion {
                    text: input.to_string(),
                    completion: suggestion.to_string(),
                    description: "Implementation guidance".to_string(),
                    suggestion_type: SuggestionType::Question,
                    confidence: 0.5,
                });
            }
        }
    }
    
    #[allow(dead_code)] // TODO: Implement pattern confidence calculation
    fn calculate_pattern_confidence(&self, prefix: &str, completion: &str) -> f32 {
        let mut confidence: f32 = 0.5;
        
        // Boost confidence if the completion makes sense in context
        let prefix_lower = prefix.to_lowercase();
        let completion_lower = completion.to_lowercase();
        
        // Check for related keywords
        if prefix_lower.contains("error") && completion_lower.contains("fix") {
            confidence += 0.3;
        }
        
        if prefix_lower.contains("function") && completion_lower.contains("create") {
            confidence += 0.2;
        }
        
        if prefix_lower.contains("test") && completion_lower.contains("add") {
            confidence += 0.2;
        }
        
        // Reduce confidence for repetitive suggestions
        if prefix.contains(&completion[..completion.len().min(10)]) {
            confidence -= 0.3;
        }
        
        confidence.max(0.0).min(1.0)
    }
    
    /// Navigate through suggestions
    pub fn move_selection_up(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_index = if self.selected_index > 0 {
                self.selected_index - 1
            } else {
                self.suggestions.len() - 1
            };
        }
    }
    
    pub fn move_selection_down(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.suggestions.len();
        }
    }
    
    /// Get the currently selected suggestion
    pub fn get_selected_suggestion(&self) -> Option<&Suggestion> {
        self.suggestions.get(self.selected_index)
    }
    
    /// Accept the selected suggestion
    pub fn accept_suggestion(&mut self) -> Option<String> {
        if let Some(suggestion) = self.get_selected_suggestion() {
            let completion = suggestion.completion.clone();
            
            // Add to recent commands
            self.recent_commands.push(completion.clone());
            if self.recent_commands.len() > 50 {
                self.recent_commands.remove(0);
            }
            
            self.hide();
            Some(completion)
        } else {
            None
        }
    }
    
    /// Show/hide suggestions
    pub fn show(&mut self) {
        self.is_visible = true;
    }
    
    pub fn hide(&mut self) {
        self.is_visible = false;
        self.suggestions.clear();
        self.selected_index = 0;
    }
    
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
    
    pub fn has_suggestions(&self) -> bool {
        !self.suggestions.is_empty()
    }
    
    /// Render autocomplete suggestions
    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.is_visible || self.suggestions.is_empty() {
            return;
        }
        
        // For slash commands, render in a more compact style (prefer above input)
        let first_suggestion = &self.suggestions[0];
        if first_suggestion.suggestion_type == SuggestionType::Command {
            // Calculate popup area (prefer above input, fallback to below)
            let max_visible_items = 8;
            
            // Use fixed height for consistent positioning, regardless of filtered items
            let fixed_popup_height = max_visible_items + 2; // +2 for borders (top and bottom)
            let actual_popup_height = (self.suggestions.len() as u16).min(max_visible_items) + 2; // +2 for borders
            
            // Position above the input box using FIXED height, but fallback to below if not enough space
            let available_space_above = area.y;
            let (popup_y, available_space) = if available_space_above >= fixed_popup_height {
                // Enough space above - position above input using fixed height for consistency
                (area.y.saturating_sub(fixed_popup_height), available_space_above)
            } else {
                // Not enough space above - position below input
                let popup_y_below = area.y + area.height;
                let screen_height = f.area().height;
                let available_space_below = screen_height.saturating_sub(popup_y_below);
                (popup_y_below, available_space_below)
            };
            
            let popup_area = Rect {
                x: area.x,
                y: popup_y,
                width: area.width.min(80), // Increased width for full descriptions
                height: actual_popup_height.min(available_space), // Use actual height for rendering
            };
            
            // Don't render if no space available
            if popup_area.height == 0 {
                return;
            }
            
            // Calculate visible window for scrolling
            let visible_start = if self.selected_index >= max_visible_items as usize {
                self.selected_index - max_visible_items as usize + 1
            } else {
                0
            };
            let visible_end = (visible_start + max_visible_items as usize).min(self.suggestions.len());
            
            // Debug logging for rendering issue
            use std::fs::OpenOptions;
            use std::io::Write;
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("/tmp/autocomplete_debug.log") {
                writeln!(file, "\nRENDER: visible_start={}, visible_end={}, suggestions.len={}", 
                    visible_start, visible_end, self.suggestions.len()).unwrap();
                writeln!(file, "RENDER: area={{x={}, y={}, width={}, height={}}}", 
                    popup_area.x, popup_area.y, popup_area.width, popup_area.height).unwrap();
                writeln!(file, "RENDER: Selected index: {}", self.selected_index).unwrap();
            }
            
            // Create suggestion items in a compact format
            let items: Vec<ListItem> = self.suggestions[visible_start..visible_end]
                .iter()
                .enumerate()
                .map(|(i, suggestion)| {
                    let actual_index = visible_start + i;
                    let is_selected = actual_index == self.selected_index;
                    let style = if is_selected {
                        Style::default().fg(Color::Cyan).bg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::White)  // Changed from Gray to White for better visibility
                    };
                    
                    // Format: command with description - use less padding
                    let text = format!("{:<10} {}", suggestion.completion, suggestion.description);
                    
                    // Debug log the actual item content
                    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("/tmp/autocomplete_debug.log") {
                        writeln!(file, "RENDER: Item #{}: text='{}', completion='{}', desc='{}', selected={}", 
                            i, text, suggestion.completion, suggestion.description, actual_index == self.selected_index).unwrap();
                    }
                    
                    // Create line with proper selection indicator
                    let prefix = if is_selected { "▶ " } else { "  " };
                    ListItem::new(Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(text, style),
                    ]))
                })
                .collect();
            
            // Add scroll indicator in the title
            let title = if self.suggestions.len() > max_visible_items as usize {
                format!(" Commands ({}/{}) ", self.selected_index + 1, self.suggestions.len())
            } else {
                " Commands ".to_string()
            };
            
            // More debug logging
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("/tmp/autocomplete_debug.log") {
                writeln!(file, "RENDER: Created {} items, is_empty={}", items.len(), items.is_empty()).unwrap();
                writeln!(file, "RENDER: Block title: '{}'", title).unwrap();
            }
            
            let suggestions_list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Yellow))  // More visible border
                    .style(Style::default().bg(Color::Black)));  // Dark background for contrast
            
            // Clear the background and render
            f.render_widget(ratatui::widgets::Clear, popup_area);
            f.render_widget(suggestions_list, popup_area);
        } else {
            // Regular suggestions (keep existing style for non-commands)
            let popup_height = (self.suggestions.len() as u16 + 2).min(10);
            let popup_area = Rect {
                x: area.x,
                y: area.y.saturating_sub(popup_height),
                width: area.width.min(80),
                height: popup_height,
            };
            
            let items: Vec<ListItem> = self.suggestions
                .iter()
                .enumerate()
                .map(|(i, suggestion)| {
                    let style = if i == self.selected_index {
                        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    
                    ListItem::new(Line::from(vec![
                        Span::styled(&suggestion.completion, style),
                        Span::styled(format!(" - {}", suggestion.description), Style::default().fg(Color::DarkGray)),
                    ]))
                })
                .collect();
            
            let suggestions_list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray)));
            
            f.render_widget(ratatui::widgets::Clear, popup_area);
            f.render_widget(suggestions_list, popup_area);
        }
    }
    
    /// Get suggestion preview text for inline display
    pub fn get_inline_preview(&self) -> Option<String> {
        if let Some(suggestion) = self.get_selected_suggestion() {
            Some(format!(" → {}", suggestion.completion))
        } else {
            None
        }
    }
}

impl Default for AutocompleteEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autocomplete_engine_creation() {
        let engine = AutocompleteEngine::new();
        assert!(!engine.is_visible);
        assert_eq!(engine.selected_index, 0);
        assert!(engine.suggestions.is_empty());
        assert!(!engine.common_patterns.is_empty());
    }

    #[test]
    fn test_slash_command_suggestions() {
        let mut engine = AutocompleteEngine::new();
        
        // Test slash command detection
        let _ = engine.generate_suggestions("/h", 2);
        assert!(engine.is_visible);
        assert!(!engine.suggestions.is_empty());
        
        // Should contain help command
        assert!(engine.suggestions.iter().any(|s| 
            s.completion.contains("/help") && s.suggestion_type == SuggestionType::Command
        ));
    }

    #[test]
    fn test_common_phrase_suggestions() {
        let mut engine = AutocompleteEngine::new();
        
        // Test partial word matching
        let _ = engine.generate_suggestions("fi", 2);
        
        // Should have suggestions for "fix"
        assert!(engine.suggestions.iter().any(|s| 
            s.text.starts_with("fix") && s.suggestion_type == SuggestionType::CommonPhrase
        ));
    }

    #[test]
    fn test_suggestion_selection() {
        let mut engine = AutocompleteEngine::new();
        let _ = engine.generate_suggestions("/h", 2);
        
        if !engine.suggestions.is_empty() {
            // Test navigation
            let initial_index = engine.selected_index;
            engine.move_selection_down();
            assert_eq!(engine.selected_index, (initial_index + 1) % engine.suggestions.len());
            
            engine.move_selection_up();
            assert_eq!(engine.selected_index, initial_index);
            
            // Test accepting suggestion
            let completion = engine.accept_suggestion();
            assert!(completion.is_some());
            assert!(!engine.is_visible);
        }
    }

    #[test]
    fn test_visibility_control() {
        let mut engine = AutocompleteEngine::new();
        
        assert!(!engine.is_visible);
        
        let _ = engine.generate_suggestions("/test", 2);
        // Should be visible if suggestions were generated
        
        engine.hide();
        assert!(!engine.is_visible);
    }
    
    #[test]
    fn test_fuzzy_matching_prioritization() {
        let mut engine = AutocompleteEngine::new();
        
        // Test /m - should show /model first
        engine.generate_suggestions("/m", 2).unwrap();
        assert!(!engine.suggestions.is_empty());
        println!("/m suggestions: {:?}", engine.suggestions.iter().map(|s| &s.completion).collect::<Vec<_>>());
        
        // Test /mo - should show /model
        engine.generate_suggestions("/mo", 3).unwrap();
        assert!(!engine.suggestions.is_empty());
        assert!(engine.suggestions.iter().any(|s| s.completion == "/model"));
        println!("/mo suggestions: {:?}", engine.suggestions.iter().map(|s| &s.completion).collect::<Vec<_>>());
        
        // Test /c - should show /clear and /config before /search
        engine.generate_suggestions("/c", 2).unwrap();
        assert!(!engine.suggestions.is_empty());
        
        // Debug: show order before collecting
        println!("Raw suggestions order:");
        for (i, sug) in engine.suggestions.iter().enumerate() {
            println!("  {}: {} (conf: {:.2})", i, sug.completion, sug.confidence);
        }
        
        let c_suggestions: Vec<_> = engine.suggestions.iter()
            .map(|s| format!("{} (conf: {:.2})", s.completion, s.confidence))
            .collect();
        println!("/c suggestions with confidence: {:?}", c_suggestions);
        
        let c_completions: Vec<_> = engine.suggestions.iter().map(|s| s.completion.clone()).collect();
        
        // /clear and /config should come before /search
        let clear_pos = c_completions.iter().position(|s| s == "/clear");
        let config_pos = c_completions.iter().position(|s| s == "/config");
        let search_pos = c_completions.iter().position(|s| s == "/search");
        
        if let (Some(search), Some(clear)) = (search_pos, clear_pos) {
            assert!(clear < search, "/clear should come before /search");
        }
        if let (Some(search), Some(config)) = (search_pos, config_pos) {
            assert!(config < search, "/config should come before /search");
        }
        
        // Test /cl - should show /clear
        engine.generate_suggestions("/cl", 3).unwrap();
        assert!(engine.suggestions.iter().any(|s| s.completion == "/clear"));
        println!("/cl suggestions: {:?}", engine.suggestions.iter().map(|s| &s.completion).collect::<Vec<_>>());
        
        // Test /co - should show /config
        engine.generate_suggestions("/co", 3).unwrap();
        assert!(engine.suggestions.iter().any(|s| s.completion == "/config"));
        println!("/co suggestions: {:?}", engine.suggestions.iter().map(|s| &s.completion).collect::<Vec<_>>());
    }
}
