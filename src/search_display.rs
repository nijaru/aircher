use std::path::Path;
use crate::semantic_search::SearchResult;
use crate::vector_search::ChunkType;

// ANSI color codes
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const ITALIC: &str = "\x1b[3m";
const BLUE: &str = "\x1b[34m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const BRIGHT_BLACK: &str = "\x1b[90m";
const WHITE: &str = "\x1b[37m";

/// Enhanced display formatting for search results
pub struct SearchResultDisplay;

impl SearchResultDisplay {
    /// Format and display a search result with syntax highlighting and context
    pub fn format_result(result: &SearchResult, index: usize, show_context: bool) -> String {
        let mut output = String::new();
        
        // Header with file path and similarity score
        output.push_str(&format!(
            "{} {} {}\n",
            format!("{}{}{}.{}", BLUE, BOLD, index + 1, RESET),
            format!("{}{}{}", GREEN, result.file_path.display(), RESET),
            format!("{}(similarity: {:.2}){}", BRIGHT_BLACK, result.similarity_score, RESET)
        ));
        
        // Chunk type and location
        let chunk_type_str = match &result.chunk.chunk_type {
            ChunkType::Function => format!("{}Function{}", CYAN, RESET),
            ChunkType::Class => format!("{}Class{}", MAGENTA, RESET),
            ChunkType::Module => format!("{}Module{}", YELLOW, RESET),
            ChunkType::Comment => format!("{}Comment{}", BRIGHT_BLACK, RESET),
            ChunkType::Generic => "Code".to_string(),
        };
        
        output.push_str(&format!(
            "   {} • Lines {}-{}\n",
            chunk_type_str,
            format!("{}{}{}", BRIGHT_BLACK, result.chunk.start_line, RESET),
            format!("{}{}{}", BRIGHT_BLACK, result.chunk.end_line, RESET)
        ));
        
        // Code snippet with syntax highlighting
        output.push_str("   ╭─────\n");
        
        let lines: Vec<&str> = result.chunk.content.lines().collect();
        let display_lines = if show_context { 10 } else { 5 };
        let total_lines = lines.len();
        
        for (i, line) in lines.iter().take(display_lines).enumerate() {
            let line_num = result.chunk.start_line + i;
            output.push_str(&format!(
                "   │ {} {}\n",
                format!("{}{:4}{}", BRIGHT_BLACK, line_num, RESET),
                Self::highlight_line(line, &result.file_path)
            ));
        }
        
        if total_lines > display_lines {
            output.push_str(&format!(
                "   │ {} {}\n",
                format!("{}    {}", BRIGHT_BLACK, RESET),
                format!("{}{}... {} more lines ...{}", BRIGHT_BLACK, ITALIC, total_lines - display_lines, RESET)
            ));
        }
        
        output.push_str("   ╰─────\n");
        
        // Context preview if available
        if show_context && !result.context_lines.is_empty() {
            output.push_str(&format!(
                "   {} {}\n",
                format!("{}Context:{}", BRIGHT_BLACK, RESET),
                format!("{}{}{}", BRIGHT_BLACK, result.context_lines.join(" ").chars().take(80).collect::<String>(), RESET)
            ));
        }
        
        output.push('\n');
        output
    }
    
    /// Basic syntax highlighting based on file extension and content
    fn highlight_line(line: &str, file_path: &Path) -> String {
        let ext = file_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        // Skip empty lines
        if line.trim().is_empty() {
            return line.to_string();
        }
        
        // Comments (most languages)
        if line.trim().starts_with("//") || line.trim().starts_with("#") {
            return format!("{}{}{}{}", BRIGHT_BLACK, ITALIC, line, RESET);
        }
        
        // Multi-line comment markers
        if line.contains("/*") || line.contains("*/") || line.trim().starts_with("*") {
            return format!("{}{}{}{}", BRIGHT_BLACK, ITALIC, line, RESET);
        }
        
        // Simple keyword highlighting
        let highlighted = match ext {
            "rs" => Self::highlight_rust(line),
            "js" | "ts" | "jsx" | "tsx" => Self::highlight_javascript(line),
            "py" => Self::highlight_python(line),
            "go" => Self::highlight_go(line),
            _ => line.to_string(),
        };
        
        highlighted
    }
    
    fn highlight_rust(line: &str) -> String {
        let keywords = ["fn", "let", "mut", "const", "struct", "enum", "impl", "trait", 
                        "pub", "use", "mod", "async", "await", "match", "if", "else", 
                        "for", "while", "loop", "return", "self", "Self"];
        
        let mut result = line.to_string();
        for keyword in &keywords {
            if result.contains(keyword) {
                result = result.replace(keyword, &format!("{}{}{}", BLUE, keyword, RESET));
            }
        }
        
        // Highlight strings
        if let Some(start) = result.find('"') {
            if let Some(end) = result[start+1..].find('"') {
                let before = &result[..start];
                let string_content = &result[start+1..start+1+end];
                let after = &result[start+1+end+1..];
                result = format!("{}{}\"{}\"{}{}", before, GREEN, string_content, RESET, after);
            }
        }
        
        result
    }
    
    fn highlight_javascript(line: &str) -> String {
        let keywords = ["function", "const", "let", "var", "class", "async", "await",
                        "if", "else", "for", "while", "return", "import", "export",
                        "from", "new", "this", "super", "extends"];
        
        let mut result = line.to_string();
        for keyword in &keywords {
            if result.contains(keyword) {
                result = result.replace(keyword, &format!("{}{}{}", BLUE, keyword, RESET));
            }
        }
        
        result
    }
    
    fn highlight_python(line: &str) -> String {
        let keywords = ["def", "class", "import", "from", "if", "else", "elif",
                        "for", "while", "return", "async", "await", "with",
                        "try", "except", "finally", "pass", "break", "continue"];
        
        let mut result = line.to_string();
        for keyword in &keywords {
            if result.contains(keyword) {
                result = result.replace(keyword, &format!("{}{}{}", BLUE, keyword, RESET));
            }
        }
        
        result
    }
    
    fn highlight_go(line: &str) -> String {
        let keywords = ["func", "var", "const", "type", "struct", "interface",
                        "if", "else", "for", "range", "return", "package",
                        "import", "go", "defer", "select", "case", "switch"];
        
        let mut result = line.to_string();
        for keyword in &keywords {
            if result.contains(keyword) {
                result = result.replace(keyword, &format!("{}{}{}", BLUE, keyword, RESET));
            }
        }
        
        result
    }
    
    /// Format the search summary with enhanced statistics
    pub fn format_summary(query: &str, result_count: usize, metrics: &str) -> String {
        if result_count == 0 {
            format!(
                "{}{} No results found for '{}'{}\n{}{} Try different search terms or check if the directory is indexed{}\n",
                YELLOW, "🔍", format!("{}{}", WHITE, query), RESET,
                BRIGHT_BLACK, "💡", RESET
            )
        } else {
            format!(
                "{}{} Found {}{}{} results for '{}{}{}' {}({}){}\n",
                GREEN, "🔍", 
                CYAN, BOLD, result_count, 
                WHITE, query, RESET,
                BRIGHT_BLACK, metrics, RESET
            )
        }
    }
    
    /// Format footer with helpful tips
    pub fn format_footer(semantic_search: bool) -> String {
        if semantic_search {
            format!(
                "{}{} {}Semantic search found contextually similar code{}\n   {}This goes beyond text matching to understand meaning{}\n",
                BLUE, "💡", 
                WHITE, RESET,
                BRIGHT_BLACK, RESET
            )
        } else {
            format!(
                "{}{} {}Text-based search (semantic search unavailable){}\n",
                YELLOW, "📝",
                BRIGHT_BLACK, RESET
            )
        }
    }
}