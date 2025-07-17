use anyhow::Result;
use std::collections::HashMap;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Zed-style predictive autocomplete for TUI chat interface
pub struct AutocompleteEngine {
    suggestions: Vec<Suggestion>,
    selected_index: usize,
    is_visible: bool,
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
        
        // Commands
        common_patterns.insert("/".to_string(), vec![
            "/search".to_string(),
            "/help".to_string(),
            "/clear".to_string(),
            "/history".to_string(),
        ]);
        
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
        
        if input.len() < 2 {
            self.is_visible = false;
            return Ok(());
        }
        
        let current_word = self.get_current_word(input, cursor_position);
        let prefix = &input[..cursor_position];
        
        // Generate different types of suggestions
        self.add_command_suggestions(&current_word);
        self.add_pattern_suggestions(prefix, &current_word);
        self.add_recent_command_suggestions(&current_word);
        self.add_contextual_suggestions(input);
        
        // Sort suggestions by confidence
        self.suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        // Limit to top 8 suggestions for UI
        self.suggestions.truncate(8);
        
        self.is_visible = !self.suggestions.is_empty();
        
        Ok(())
    }
    
    fn get_current_word(&self, input: &str, cursor_position: usize) -> String {
        let before_cursor = &input[..cursor_position];
        
        // Find the start of the current word
        let word_start = before_cursor.rfind(|c: char| c.is_whitespace() || c == '/')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        
        before_cursor[word_start..].to_string()
    }
    
    fn add_command_suggestions(&mut self, current_word: &str) {
        if current_word.starts_with('/') {
            let commands = vec![
                ("search", "Search through code files"),
                ("help", "Show help information"),
                ("clear", "Clear chat history"),
                ("history", "Show command history"),
                ("config", "Show configuration"),
                ("cost", "Show session cost"),
            ];
            
            for (cmd, desc) in commands {
                let full_cmd = format!("/{}", cmd);
                if full_cmd.starts_with(current_word) {
                    self.suggestions.push(Suggestion {
                        text: current_word.to_string(),
                        completion: full_cmd,
                        description: desc.to_string(),
                        suggestion_type: SuggestionType::Command,
                        confidence: 0.9,
                    });
                }
            }
        }
    }
    
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
        
        // Calculate popup area (above the input box)
        let popup_height = (self.suggestions.len() as u16 + 2).min(10);
        let popup_area = Rect {
            x: area.x,
            y: area.y.saturating_sub(popup_height),
            width: area.width.min(80),
            height: popup_height,
        };
        
        // Create suggestion items
        let items: Vec<ListItem> = self.suggestions
            .iter()
            .enumerate()
            .map(|(i, suggestion)| {
                let icon = match suggestion.suggestion_type {
                    SuggestionType::Command => "⚡",
                    SuggestionType::CodeContext => "🧠",
                    SuggestionType::Question => "❓",
                    SuggestionType::FileReference => "📁",
                    SuggestionType::CommonPhrase => "💬",
                };
                
                let style = if i == self.selected_index {
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Gray)
                };
                
                let confidence_bar = "█".repeat((suggestion.confidence * 5.0) as usize);
                
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", icon), style),
                    Span::styled(&suggestion.completion, style),
                    Span::styled(format!(" [{}]", confidence_bar), Style::default().fg(Color::DarkGray)),
                ]))
            })
            .collect();
        
        let title = format!("Suggestions ({}/{})", self.selected_index + 1, self.suggestions.len());
        let suggestions_list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)));
        
        // Clear the background and render
        f.render_widget(ratatui::widgets::Clear, popup_area);
        f.render_widget(suggestions_list, popup_area);
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