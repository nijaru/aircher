use std::path::PathBuf;
use tempfile::TempDir;
use aircher::semantic_search::SemanticCodeSearch;

#[tokio::test]
async fn test_simple_search() {
    println!("🧪 Testing simple search flow...");

    // Create a temporary directory with test code
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("sample.rs");

    let sample_code = r#"
/// This function handles user authentication
pub fn authenticate_user(username: &str, password: &str) -> bool {
    // Validate credentials
    if username.is_empty() || password.is_empty() {
        return false;
    }

    // Check against database
    verify_credentials(username, password)
}
"#;

    std::fs::write(&test_file, sample_code).unwrap();

    // Test the search functionality
    let mut search = SemanticCodeSearch::new();

    // Ensure model is available
    match search.ensure_model_available().await {
        Ok(()) => println!("✅ Model available"),
        Err(e) => println!("❌ Model not available: {}", e),
    }

    // Index the directory
    match search.index_directory(&temp_dir.path()).await {
        Ok(()) => {
            println!("✅ Directory indexed successfully");

            // Get stats
            let stats = search.get_stats();
            println!("📊 Stats: {} files, {} chunks", stats.total_files, stats.total_chunks);

            // Test search
            match search.search("authentication function", 5).await {
                Ok((results, _metrics)) => {
                    println!("✅ Search successful: {} results", results.len());
                    for result in results {
                        println!("  - {} (score: {:.3})", result.file_path.display(), result.similarity_score);
                    }
                },
                Err(e) => {
                    println!("❌ Search failed: {}", e);
                }
            }
        },
        Err(e) => {
            println!("❌ Indexing failed: {}", e);
        }
    }

    println!("🎉 Simple search test completed!");
}
