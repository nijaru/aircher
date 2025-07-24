use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use std::collections::HashMap;

/// Simple syntax highlighter for common languages
pub struct SyntaxHighlighter {
    keywords: HashMap<String, Vec<&'static str>>,
    colors: SyntaxColors,
}

#[derive(Clone)]
pub struct SyntaxColors {
    pub keyword: Color,
    pub string: Color,
    pub comment: Color,
    pub number: Color,
    pub function: Color,
    pub type_name: Color,
    pub operator: Color,
    pub normal: Color,
    pub bracket: Color,
}

impl Default for SyntaxColors {
    fn default() -> Self {
        Self {
            keyword: Color::Rgb(139, 92, 246),    // Purple - keywords
            string: Color::Rgb(34, 197, 94),      // Green - strings
            comment: Color::Rgb(115, 115, 115),   // Gray - comments
            number: Color::Rgb(251, 146, 60),     // Orange - numbers
            function: Color::Rgb(59, 130, 246),   // Blue - functions
            type_name: Color::Rgb(168, 85, 247),  // Bright purple - types
            operator: Color::Rgb(244, 63, 94),    // Pink - operators
            normal: Color::Rgb(163, 136, 186),    // Default text color
            bracket: Color::Rgb(156, 163, 175),   // Gray - brackets
        }
    }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let mut keywords = HashMap::new();
        
        // Rust keywords
        keywords.insert("rust".to_string(), vec![
            "fn", "let", "mut", "pub", "struct", "enum", "impl", "trait", "use", "mod",
            "if", "else", "match", "loop", "while", "for", "in", "break", "continue",
            "return", "async", "await", "move", "static", "const", "type", "where",
            "unsafe", "extern", "crate", "super", "self", "Self",
        ]);
        
        // JavaScript/TypeScript keywords
        keywords.insert("javascript".to_string(), vec![
            "function", "const", "let", "var", "if", "else", "for", "while", "do",
            "switch", "case", "default", "break", "continue", "return", "try", "catch",
            "finally", "throw", "new", "class", "extends", "import", "export", "from",
            "async", "await", "yield", "typeof", "instanceof",
        ]);
        keywords.insert("typescript".to_string(), keywords.get("javascript").unwrap().clone());
        keywords.insert("js".to_string(), keywords.get("javascript").unwrap().clone());
        keywords.insert("ts".to_string(), keywords.get("javascript").unwrap().clone());
        
        // Python keywords
        keywords.insert("python".to_string(), vec![
            "def", "class", "if", "elif", "else", "for", "while", "try", "except",
            "finally", "with", "as", "import", "from", "return", "yield", "lambda",
            "and", "or", "not", "in", "is", "True", "False", "None", "async", "await",
        ]);
        keywords.insert("py".to_string(), keywords.get("python").unwrap().clone());
        
        // Go keywords
        keywords.insert("go".to_string(), vec![
            "func", "var", "const", "type", "struct", "interface", "if", "else",
            "for", "range", "switch", "case", "default", "select", "go", "chan",
            "defer", "return", "break", "continue", "package", "import",
        ]);
        
        // JSON (special case)
        keywords.insert("json".to_string(), vec![
            "true", "false", "null",
        ]);
        
        Self {
            keywords,
            colors: SyntaxColors::default(),
        }
    }
    
    /// Parse message content and return highlighted lines
    pub fn highlight_message(&self, content: &str) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let mut in_code_block = false;
        let mut current_language = None;
        
        for line_text in content.lines() {
            if line_text.trim_start().starts_with("```") {
                if in_code_block {
                    // End of code block
                    in_code_block = false;
                    current_language = None;
                    lines.push(Line::from(Span::styled(
                        line_text.to_string(),
                        Style::default().fg(self.colors.comment)
                    )));
                } else {
                    // Start of code block
                    in_code_block = true;
                    let lang = line_text.trim_start().strip_prefix("```").unwrap_or("").trim();
                    current_language = if lang.is_empty() { None } else { Some(lang.to_lowercase()) };
                    lines.push(Line::from(Span::styled(
                        line_text.to_string(),
                        Style::default().fg(self.colors.comment)
                    )));
                }
            } else if in_code_block {
                // Inside code block - apply syntax highlighting
                if let Some(ref lang) = current_language {
                    lines.push(self.highlight_code_line(line_text, lang));
                } else {
                    lines.push(self.highlight_code_line(line_text, ""));
                }
            } else {
                // Regular text - check for inline code
                lines.push(self.highlight_inline_code(line_text));
            }
        }
        
        lines
    }
    
    /// Highlight a single line of code
    fn highlight_code_line(&self, line: &str, language: &str) -> Line<'static> {
        if line.trim().is_empty() {
            return Line::from(Span::raw(line.to_string()));
        }
        
        // Simple token-based highlighting
        let mut spans = Vec::new();
        let mut current_word = String::new();
        let mut chars = line.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                // String literals - simple case
                '"' => {
                    if !current_word.is_empty() {
                        spans.extend(self.highlight_token(&current_word, language));
                        current_word.clear();
                    }
                    
                    let mut string_content = String::from("\"");
                    let mut escaped = false;
                    
                    while let Some(next_ch) = chars.next() {
                        string_content.push(next_ch);
                        if next_ch == '"' && !escaped {
                            break;
                        }
                        escaped = next_ch == '\\' && !escaped;
                    }
                    
                    spans.push(Span::styled(string_content, Style::default().fg(self.colors.string)));
                }
                
                // Single quotes
                '\'' => {
                    if !current_word.is_empty() {
                        spans.extend(self.highlight_token(&current_word, language));
                        current_word.clear();
                    }
                    
                    let mut string_content = String::from("'");
                    let mut escaped = false;
                    
                    while let Some(next_ch) = chars.next() {
                        string_content.push(next_ch);
                        if next_ch == '\'' && !escaped {
                            break;
                        }
                        escaped = next_ch == '\\' && !escaped;
                    }
                    
                    spans.push(Span::styled(string_content, Style::default().fg(self.colors.string)));
                }
                
                // Comments - simple cases
                '/' if language.contains("rust") || language.contains("javascript") || language.contains("go") => {
                    if chars.peek() == Some(&'/') {
                        // Rest of line is comment
                        if !current_word.is_empty() {
                            spans.extend(self.highlight_token(&current_word, language));
                            current_word.clear();
                        }
                        let comment = format!("/{}", chars.collect::<String>());
                        spans.push(Span::styled(comment, Style::default().fg(self.colors.comment)));
                        break;
                    } else {
                        spans.push(Span::styled("/".to_string(), Style::default().fg(self.colors.operator)));
                    }
                }
                
                '#' if language.contains("python") => {
                    // Rest of line is comment
                    if !current_word.is_empty() {
                        spans.extend(self.highlight_token(&current_word, language));
                        current_word.clear();
                    }
                    let comment = format!("#{}", chars.collect::<String>());
                    spans.push(Span::styled(comment, Style::default().fg(self.colors.comment)));
                    break;
                }
                
                // Operators
                '+' | '-' | '*' | '=' | '<' | '>' | '!' | '&' | '|' | '^' | '%' => {
                    if !current_word.is_empty() {
                        spans.extend(self.highlight_token(&current_word, language));
                        current_word.clear();
                    }
                    spans.push(Span::styled(ch.to_string(), Style::default().fg(self.colors.operator)));
                }
                
                // Brackets
                '(' | ')' | '[' | ']' | '{' | '}' => {
                    if !current_word.is_empty() {
                        spans.extend(self.highlight_token(&current_word, language));
                        current_word.clear();
                    }
                    spans.push(Span::styled(ch.to_string(), Style::default().fg(self.colors.bracket)));
                }
                
                // Whitespace and punctuation
                ' ' | '\t' | ',' | ';' | ':' | '.' => {
                    if !current_word.is_empty() {
                        spans.extend(self.highlight_token(&current_word, language));
                        current_word.clear();
                    }
                    spans.push(Span::raw(ch.to_string()));
                }
                
                _ => {
                    current_word.push(ch);
                }
            }
        }
        
        // Handle remaining word
        if !current_word.is_empty() {
            spans.extend(self.highlight_token(&current_word, language));
        }
        
        Line::from(spans)
    }
    
    /// Highlight a token (word) based on its type
    fn highlight_token(&self, token: &str, language: &str) -> Vec<Span<'static>> {
        // Check if it's a keyword
        if let Some(keywords) = self.keywords.get(language) {
            if keywords.contains(&token) {
                return vec![Span::styled(token.to_string(), Style::default().fg(self.colors.keyword))];
            }
        }
        
        // Check if it's a number
        if token.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '_') && !token.is_empty() {
            return vec![Span::styled(token.to_string(), Style::default().fg(self.colors.number))];
        }
        
        // Check if it looks like a function call or variable
        if token.chars().all(|c| c.is_alphanumeric() || c == '_') && !token.is_empty() {
            if token.chars().next().map_or(false, |c| c.is_lowercase()) || token.contains('_') {
                return vec![Span::styled(token.to_string(), Style::default().fg(self.colors.function))];
            }
        }
        
        // Check if it looks like a type (starts with uppercase)
        if token.chars().next().map_or(false, |c| c.is_uppercase()) {
            return vec![Span::styled(token.to_string(), Style::default().fg(self.colors.type_name))];
        }
        
        // Default color
        vec![Span::styled(token.to_string(), Style::default().fg(self.colors.normal))]
    }
    
    /// Highlight inline code (backticks)
    fn highlight_inline_code(&self, line: &str) -> Line<'static> {
        let mut spans = Vec::new();
        let mut chars = line.chars();
        let mut current = String::new();
        let mut in_inline_code = false;
        
        while let Some(ch) = chars.next() {
            if ch == '`' {
                if in_inline_code {
                    // End of inline code
                    spans.push(Span::styled(
                        format!("`{}`", current),
                        Style::default().fg(self.colors.string)
                    ));
                    current.clear();
                    in_inline_code = false;
                } else {
                    // Start of inline code or regular text to add
                    if !current.is_empty() {
                        spans.push(Span::styled(current.clone(), Style::default().fg(self.colors.normal)));
                        current.clear();
                    }
                    in_inline_code = true;
                }
            } else {
                current.push(ch);
            }
        }
        
        // Handle remaining content
        if !current.is_empty() {
            if in_inline_code {
                spans.push(Span::styled(
                    format!("`{}", current),
                    Style::default().fg(self.colors.string)
                ));
            } else {
                spans.push(Span::styled(current, Style::default().fg(self.colors.normal)));
            }
        }
        
        if spans.is_empty() {
            Line::from(Span::raw(line.to_string()))
        } else {
            Line::from(spans)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rust_highlighting() {
        let highlighter = SyntaxHighlighter::new();
        let code = "fn main() {\n    let x = 42;\n    println!(\"Hello\");\n}";
        let lines = highlighter.highlight_message(&[
            "```rust",
            code,
            "```"
        ].join("\n"));
        
        assert_eq!(lines.len(), 5); // ``` + 3 code lines + ```
    }
    
    #[test]
    fn test_inline_code() {
        let highlighter = SyntaxHighlighter::new();
        let text = "Use `cargo run` to start the application.";
        let lines = highlighter.highlight_message(text);
        
        assert_eq!(lines.len(), 1);
    }
}