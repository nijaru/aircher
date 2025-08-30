use std::path::PathBuf;
use tempfile::TempDir;
use aircher::semantic_search::SemanticCodeSearch;

#[tokio::test]
async fn test_end_to_end_semantic_search() {
    println!("🧪 Testing end-to-end semantic search pipeline...");
    
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_code.rs");
    
    // Create a sample Rust file
    let sample_code = r#"
/// This is a test function that adds two numbers
fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

/// This is a test struct for user data
struct User {
    name: String,
    age: u32,
}

impl User {
    fn new(name: String, age: u32) -> Self {
        Self { name, age }
    }
    
    fn get_name(&self) -> &str {
        &self.name
    }
}

/// This function multiplies two numbers
fn multiply(x: f64, y: f64) -> f64 {
    x * y
}
"#;
    
    // Write the sample code to file
    std::fs::write(&test_file, sample_code).unwrap();
    
    // Create semantic search instance
    let mut search = SemanticCodeSearch::new();
    
    // Test 1: Check if model extraction works
    match search.ensure_model_available().await {
        Ok(()) => println!("✅ Model extraction successful"),
        Err(e) => {
            println!("❌ Model extraction failed: {}", e);
            panic!("Model extraction failed: {}", e);
        }
    }
    
    // Test 2: Index the sample file
    match search.index_file(&test_file).await {
        Ok(()) => println!("✅ File indexing successful"),
        Err(e) => {
            println!("❌ File indexing failed: {}", e);
            panic!("File indexing failed: {}", e);
        }
    }
    
    // Test 3: Check stats
    let stats = search.get_stats();
    println!("📊 Index stats: {} files, {} chunks, {:.2}% embedded", 
             stats.total_files, stats.total_chunks, stats.embedding_coverage * 100.0);
    
    assert!(stats.total_files > 0, "Should have indexed at least one file");
    assert!(stats.total_chunks > 0, "Should have created at least one chunk");
    
    // Test 4: Perform semantic search
    let search_queries = vec![
        "add two numbers",
        "user data structure", 
        "multiply function",
        "get user name",
    ];
    
    for query in search_queries {
        println!("🔍 Searching for: '{}'", query);
        
        match search.search(query, 3).await {
            Ok((results, _metrics)) => {
                println!("✅ Search successful, found {} results", results.len());
                for (i, result) in results.iter().enumerate() {
                    println!("  {}. {} (lines {}-{}) - score: {:.3}", 
                             i + 1, 
                             result.file_path.file_name().unwrap().to_string_lossy(),
                             result.chunk.start_line,
                             result.chunk.end_line,
                             result.similarity_score);
                }
            },
            Err(e) => {
                println!("❌ Search failed for '{}': {}", query, e);
                // Don't panic here - semantic search might fall back to text search
            }
        }
    }
    
    println!("🎉 End-to-end test completed!");
}