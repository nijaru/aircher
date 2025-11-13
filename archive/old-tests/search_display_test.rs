use aircher::search_display::SearchResultDisplay;
use aircher::semantic_search::{SearchResult, CodeChunk};
use aircher::vector_search::ChunkType;
use std::path::PathBuf;


#[test]
fn test_search_result_display_formatting() {
    let chunk = CodeChunk {
        content: "fn test() {\n    println!(\"test\");\n}".to_string(),
        start_line: 10,
        end_line: 12,
        chunk_type: ChunkType::Function,
        embedding: None,
    };

    let result = SearchResult {
        file_path: PathBuf::from("test.rs"),
        chunk,
        similarity_score: 0.95,
        context_lines: vec![],
    };

    let formatted = SearchResultDisplay::format_result(&result, 0, false);

    // Check basic formatting
    assert!(formatted.contains("test.rs"));
    assert!(formatted.contains("similarity: 0.95"));
    assert!(formatted.contains("Function"));
    // The line numbers have ANSI codes between them
    assert!(formatted.contains("Lines"));
    assert!(formatted.contains("10"));
    assert!(formatted.contains("12"));

    // Check syntax highlighting is applied
    assert!(formatted.contains("\x1b[")); // ANSI codes
    assert!(formatted.contains("fn"));
    assert!(formatted.contains("test"));
}

#[test]
fn test_search_result_display_with_context() {
    let chunk = CodeChunk {
        content: "impl Display for Error {\n    fn fmt(&self, f: &mut Formatter) -> Result {\n        write!(f, \"{}\")\n    }\n}".to_string(),
        start_line: 20,
        end_line: 24,
        chunk_type: ChunkType::Function,
        embedding: None,
    };

    let result = SearchResult {
        file_path: PathBuf::from("src/error.rs"),
        chunk,
        similarity_score: 0.88,
        context_lines: vec!["Error handling implementation".to_string()],
    };

    let formatted = SearchResultDisplay::format_result(&result, 1, true);

    // Check enhanced display with context
    assert!(formatted.contains("src/error.rs"));
    assert!(formatted.contains("2.")); // Index 1 + 1
    assert!(formatted.contains("similarity: 0.88"));

    // Should use advanced highlighter for multi-line chunks
    assert!(formatted.contains("impl"));
    assert!(formatted.contains("Display"));
    assert!(formatted.contains("\x1b[")); // ANSI codes present
}
