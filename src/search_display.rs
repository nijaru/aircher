use std::path::Path;
use anyhow::Result;
use tree_sitter::{Parser, Tree, Node};
use crate::semantic_search::SearchResult;
use crate::vector_search::ChunkType;

// ANSI color codes
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const _DIM: &str = "\x1b[2m";
const ITALIC: &str = "\x1b[3m";
const BLUE: &str = "\x1b[34m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const BRIGHT_BLACK: &str = "\x1b[90m";
const WHITE: &str = "\x1b[37m";

/// AST-based syntax highlighter using tree-sitter
pub struct SyntaxHighlighter {
    parser: Parser,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
        }
    }

    /// Highlight code using AST analysis with tree-sitter (optimized for performance)
    pub fn highlight_code(&mut self, content: &str, language: &str) -> String {
        // For single lines or small content, use basic highlighting for performance
        if content.lines().count() <= 1 || content.len() < 100 {
            return BasicHighlighter::highlight_line(content, language);
        }
        
        // Only use tree-sitter for supported languages and larger content
        match self.parse_and_highlight(content, language) {
            Ok(highlighted) => highlighted,
            Err(_) => {
                // Fallback to basic highlighting if tree-sitter fails
                BasicHighlighter::highlight_line(content, language)
            }
        }
    }

    fn parse_and_highlight(&mut self, content: &str, language: &str) -> Result<String> {
        // Set the appropriate language for tree-sitter (only stable languages for now)
        let tree_language = match language {
            "rust" | "rs" => tree_sitter_rust::LANGUAGE.into(),
            "javascript" | "js" => tree_sitter_javascript::LANGUAGE.into(),
            "typescript" | "ts" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            "python" | "py" => tree_sitter_python::LANGUAGE.into(),
            "go" => tree_sitter_go::LANGUAGE.into(),
            "c" => tree_sitter_c::LANGUAGE.into(),
            "cpp" | "cc" | "cxx" => tree_sitter_cpp::LANGUAGE.into(),
            "java" => tree_sitter_java::LANGUAGE.into(),
            "json" => tree_sitter_json::LANGUAGE.into(),
            "bash" | "sh" => tree_sitter_bash::LANGUAGE.into(),
            // Other languages fall back to basic highlighting
            _ => return Err(anyhow::anyhow!("Using fallback highlighting for language: {}", language)),
        };

        self.parser.set_language(&tree_language)?;
        let tree = self.parser.parse(content, None);

        match tree {
            Some(tree) => Ok(self.traverse_and_highlight(&tree, content)),
            None => Err(anyhow::anyhow!("Failed to parse content")),
        }
    }

    fn traverse_and_highlight(&self, tree: &Tree, source: &str) -> String {
        let root_node = tree.root_node();
        let mut result = String::new();
        let mut last_pos = 0;

        self.highlight_node(&root_node, source, &mut result, &mut last_pos);
        
        // Add any remaining content
        if last_pos < source.len() {
            result.push_str(&source[last_pos..]);
        }

        result
    }

    fn highlight_node(&self, node: &Node, source: &str, result: &mut String, last_pos: &mut usize) {
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();

        // Add any text between the last position and this node
        if *last_pos < start_byte {
            result.push_str(&source[*last_pos..start_byte]);
        }

        // Get node type and apply appropriate styling
        let node_kind = node.kind();
        let node_text = &source[start_byte..end_byte];

        let styled_text = match node_kind {
            // Keywords
            "fn" | "let" | "mut" | "const" | "struct" | "enum" | "impl" | "trait" |
            "pub" | "use" | "mod" | "async" | "await" | "match" | "if" | "else" |
            "for" | "while" | "loop" | "return" | "function" | "class" | "def" |
            "import" | "export" | "from" | "var" | "type" | "interface" => {
                format!("{}{}{}{}", BLUE, BOLD, node_text, RESET)
            }
            
            // Strings
            "string_literal" | "raw_string_literal" | "string" => {
                format!("{}{}{}", GREEN, node_text, RESET)
            }
            
            // Numbers
            "integer_literal" | "float_literal" | "number" => {
                format!("{}{}{}", CYAN, node_text, RESET)
            }
            
            // Comments
            "comment" | "line_comment" | "block_comment" => {
                format!("{}{}{}{}", BRIGHT_BLACK, ITALIC, node_text, RESET)
            }
            
            // Function/method names
            "function_item" | "method_definition" | "function_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name_text = &source[name_node.start_byte()..name_node.end_byte()];
                    format!("{}{}{}{}", YELLOW, BOLD, name_text, RESET)
                } else {
                    node_text.to_string()
                }
            }
            
            // Types
            "type_identifier" | "primitive_type" => {
                format!("{}{}{}", MAGENTA, node_text, RESET)
            }
            
            _ => {
                // For compound nodes, recursively process children
                if node.child_count() > 0 {
                    let mut child_result = String::new();
                    let mut child_pos = start_byte;
                    
                    let mut cursor = node.walk();
                    cursor.goto_first_child();
                    
                    loop {
                        let child = cursor.node();
                        self.highlight_node(&child, source, &mut child_result, &mut child_pos);
                        
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    
                    // Add any remaining content within this node
                    if child_pos < end_byte {
                        child_result.push_str(&source[child_pos..end_byte]);
                    }
                    
                    child_result
                } else {
                    node_text.to_string()
                }
            }
        };

        result.push_str(&styled_text);
        *last_pos = end_byte;
    }
}

/// Basic fallback highlighter for unsupported languages
struct BasicHighlighter;

impl BasicHighlighter {
    fn highlight_line(line: &str, language: &str) -> String {
        // Use basic highlighting for all languages as fallback
        match language {
            "rust" | "rs" => Self::highlight_rust(line),
            "javascript" | "js" | "typescript" | "ts" | "jsx" | "tsx" => Self::highlight_javascript(line),
            "python" | "py" => Self::highlight_python(line),
            "go" => Self::highlight_go(line),
            "php" => Self::highlight_php(line),
            "ruby" | "rb" => Self::highlight_ruby(line),
            "swift" => Self::highlight_swift(line),
            "kotlin" | "kt" => Self::highlight_kotlin(line),
            "csharp" | "cs" => Self::highlight_csharp(line),
            "css" => Self::highlight_css(line),
            "html" | "htm" => Self::highlight_html(line),
            "yaml" | "yml" => Self::highlight_yaml(line),
            "sql" => Self::highlight_sql(line),
            _ => Self::highlight_generic(line),
        }
    }

    fn highlight_rust(line: &str) -> String {
        // Optimized highlighting with word boundary checking to avoid issues like "function" matching "fn"
        let mut result = line.to_string();
        
        // Only highlight if the line is not too long (performance protection)
        if result.len() > 500 {
            return result;
        }
        
        // Simple keyword highlighting with word boundaries
        if result.contains(" fn ") || result.starts_with("fn ") {
            result = result.replace(" fn ", &format!(" {}{}{} ", BLUE, "fn", RESET));
            if result.starts_with("fn ") {
                result = result.replacen("fn ", &format!("{}{}{} ", BLUE, "fn", RESET), 1);
            }
        }
        
        if result.contains(" let ") {
            result = result.replace(" let ", &format!(" {}{}{} ", BLUE, "let", RESET));
        }
        
        if result.contains(" pub ") || result.starts_with("pub ") {
            result = result.replace(" pub ", &format!(" {}{}{} ", BLUE, "pub", RESET));
            if result.starts_with("pub ") {
                result = result.replacen("pub ", &format!("{}{}{} ", BLUE, "pub", RESET), 1);
            }
        }
        
        result
    }
    
    fn highlight_javascript(line: &str) -> String {
        let mut result = line.to_string();
        
        // Performance protection
        if result.len() > 500 {
            return result;
        }
        
        // Simple highlighting for common JS keywords
        if result.contains(" function ") || result.starts_with("function ") {
            result = result.replace(" function ", &format!(" {}{}{} ", BLUE, "function", RESET));
        }
        
        if result.contains(" const ") {
            result = result.replace(" const ", &format!(" {}{}{} ", BLUE, "const", RESET));
        }
        
        result
    }
    
    fn highlight_python(line: &str) -> String {
        let mut result = line.to_string();
        
        // Performance protection
        if result.len() > 500 {
            return result;
        }
        
        // Simple highlighting for common Python keywords
        if result.contains(" def ") || result.starts_with("def ") {
            result = result.replace(" def ", &format!(" {}{}{} ", BLUE, "def", RESET));
            if result.starts_with("def ") {
                result = result.replacen("def ", &format!("{}{}{} ", BLUE, "def", RESET), 1);
            }
        }
        
        if result.contains(" class ") || result.starts_with("class ") {
            result = result.replace(" class ", &format!(" {}{}{} ", BLUE, "class", RESET));
            if result.starts_with("class ") {
                result = result.replacen("class ", &format!("{}{}{} ", BLUE, "class", RESET), 1);
            }
        }
        
        result
    }
    
    fn highlight_go(line: &str) -> String {
        let mut result = line.to_string();
        
        // Performance protection
        if result.len() > 500 {
            return result;
        }
        
        // Simple highlighting for Go
        if result.contains(" func ") || result.starts_with("func ") {
            result = result.replace(" func ", &format!(" {}{}{} ", BLUE, "func", RESET));
            if result.starts_with("func ") {
                result = result.replacen("func ", &format!("{}{}{} ", BLUE, "func", RESET), 1);
            }
        }
        
        result
    }

    // Simplified, fast highlighting methods for all other languages
    fn highlight_php(line: &str) -> String {
        Self::simple_highlight(line, "<?php", "function", "class")
    }

    fn highlight_ruby(line: &str) -> String {
        Self::simple_highlight(line, "def", "class", "module")
    }

    fn highlight_swift(line: &str) -> String {
        Self::simple_highlight(line, "func", "var", "let")
    }

    fn highlight_kotlin(line: &str) -> String {
        Self::simple_highlight(line, "fun", "val", "var")
    }

    fn highlight_csharp(line: &str) -> String {
        Self::simple_highlight(line, "class", "interface", "struct")
    }

    fn highlight_css(line: &str) -> String {
        if line.len() > 200 { return line.to_string(); }
        if line.trim().starts_with('.') || line.trim().starts_with('#') {
            format!("{}{}{}", GREEN, line, RESET)
        } else {
            line.to_string()
        }
    }

    fn highlight_html(line: &str) -> String {
        if line.len() > 200 { return line.to_string(); }
        if line.contains('<') && line.contains('>') {
            // Very simple tag highlighting - just color the whole line if it has tags
            format!("{}{}{}", BLUE, line, RESET)
        } else {
            line.to_string()
        }
    }

    fn highlight_yaml(line: &str) -> String {
        if line.len() > 200 { return line.to_string(); }
        if line.contains(':') && !line.trim().starts_with('#') {
            format!("{}{}{}", GREEN, line, RESET)
        } else {
            line.to_string()
        }
    }

    fn highlight_sql(line: &str) -> String {
        Self::simple_highlight(line, "SELECT", "FROM", "WHERE")
    }

    fn highlight_generic(line: &str) -> String {
        // Just return the line as-is for generic highlighting
        line.to_string()
    }

    // Helper method for safe, simple highlighting
    fn simple_highlight(line: &str, keyword1: &str, keyword2: &str, keyword3: &str) -> String {
        if line.len() > 200 {
            return line.to_string();
        }
        
        let mut result = line.to_string();
        
        // Simple, safe keyword highlighting
        let check_and_highlight = |text: &str, kw: &str| -> String {
            if text.contains(&format!(" {} ", kw)) {
                text.replace(&format!(" {} ", kw), &format!(" {}{}{} ", BLUE, kw, RESET))
            } else if text.starts_with(&format!("{} ", kw)) {
                text.replacen(&format!("{} ", kw), &format!("{}{}{} ", BLUE, kw, RESET), 1)
            } else {
                text.to_string()
            }
        };
        
        result = check_and_highlight(&result, keyword1);
        result = check_and_highlight(&result, keyword2);
        result = check_and_highlight(&result, keyword3);
        
        result
    }
}

/// Enhanced display formatting for search results
pub struct SearchResultDisplay;

impl SearchResultDisplay {
    /// Get a thread-local syntax highlighter instance
    fn with_highlighter<F, R>(f: F) -> R
    where
        F: FnOnce(&mut SyntaxHighlighter) -> R,
    {
        thread_local! {
            static HIGHLIGHTER: std::cell::RefCell<SyntaxHighlighter> = std::cell::RefCell::new(SyntaxHighlighter::new());
        }
        
        HIGHLIGHTER.with(|h| f(&mut h.borrow_mut()))
    }

    /// Format and display a search result with enhanced syntax highlighting and context
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
        
        // Determine language from file extension for better syntax highlighting
        let language = Self::get_language_from_path(&result.file_path);
        
        // For multi-line chunks, use advanced syntax highlighter
        let use_advanced = lines.len() > 1 && !language.is_empty();
        
        if use_advanced {
            // Get the full chunk content for AST-based highlighting
            let chunk_to_highlight: String = lines.iter()
                .take(display_lines)
                .map(|&line| line)
                .collect::<Vec<_>>()
                .join("\n");
            
            let highlighted_chunk = Self::with_highlighter(|highlighter| {
                highlighter.highlight_code(&chunk_to_highlight, &language)
            });
            
            // Split highlighted chunk back into lines and display
            for (i, highlighted_line) in highlighted_chunk.lines().enumerate() {
                let line_num = result.chunk.start_line + i;
                output.push_str(&format!(
                    "   │ {} {}\n",
                    format!("{}{:4}{}", BRIGHT_BLACK, line_num, RESET),
                    highlighted_line
                ));
            }
        } else {
            // For single lines or unsupported languages, use basic highlighting
            for (i, line) in lines.iter().take(display_lines).enumerate() {
                let line_num = result.chunk.start_line + i;
                let highlighted_line = if line.trim().is_empty() {
                    line.to_string()
                } else {
                    BasicHighlighter::highlight_line(line, &language)
                };
                output.push_str(&format!(
                    "   │ {} {}\n",
                    format!("{}{:4}{}", BRIGHT_BLACK, line_num, RESET),
                    highlighted_line
                ));
            }
        }
        
        if total_lines > display_lines {
            output.push_str(&format!(
                "   │ {} {}\n",
                format!("{}    {}", BRIGHT_BLACK, RESET),
                format!("{}{}... {} more lines ...{}", BRIGHT_BLACK, ITALIC, total_lines - display_lines, RESET)
            ));
        }
        
        output.push_str("   ╰─────\n");
        
        // Enhanced context display
        if show_context {
            output.push_str(&Self::format_enhanced_context(result));
        }
        
        output.push('\n');
        output
    }
    
    /// Extract language identifier from file path
    fn get_language_from_path(file_path: &Path) -> String {
        let extension = file_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        match extension {
            "rs" => "rust".to_string(),
            "js" | "jsx" => "javascript".to_string(),
            "ts" | "tsx" => "typescript".to_string(),
            "py" => "python".to_string(),
            "go" => "go".to_string(),
            "c" => "c".to_string(),
            "cpp" | "cc" | "cxx" | "c++" => "cpp".to_string(),
            "java" => "java".to_string(),
            "cs" => "csharp".to_string(),
            "php" => "php".to_string(),
            "rb" => "ruby".to_string(),
            "swift" => "swift".to_string(),
            "kt" => "kotlin".to_string(),
            _ => extension.to_string(), // Fallback to extension
        }
    }
    
    /// Enhanced context display with better structure understanding
    fn format_enhanced_context(result: &SearchResult) -> String {
        let mut context = String::new();
        
        if !result.context_lines.is_empty() {
            context.push_str(&format!(
                "   {} {}\n",
                format!("{}📍 Context:{}", BRIGHT_BLACK, RESET),
                format!("{}{}{}", 
                    BRIGHT_BLACK,
                    result.context_lines.join(" • ").chars().take(100).collect::<String>(),
                    RESET
                )
            ));
        }
        
        // Add file structure hint if available
        if let Some(parent) = result.file_path.parent() {
            if let Some(parent_name) = parent.file_name().and_then(|n| n.to_str()) {
                context.push_str(&format!(
                    "   {} {}\n",
                    format!("{}📂 Directory:{}", BRIGHT_BLACK, RESET),
                    format!("{}{}{}", BRIGHT_BLACK, parent_name, RESET)
                ));
            }
        }
        
        context
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