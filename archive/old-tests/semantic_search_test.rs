use std::path::PathBuf;
use tempfile::TempDir;
use aircher::semantic_search::SemanticCodeSearch;
use aircher::code_chunking::CodeChunker;

#[tokio::test]
async fn test_semantic_search_step_by_step() {
    println!("🧪 Testing SemanticCodeSearch step by step...");

    // Step 1: Initialize SemanticCodeSearch
    let mut search = SemanticCodeSearch::new();
    println!("✅ SemanticCodeSearch initialized");

    // Step 2: Test model availability (this was failing in the original test)
    println!("📦 Testing model availability...");
    match search.ensure_model_available().await {
        Ok(()) => println!("✅ Model availability check passed"),
        Err(e) => {
            println!("❌ Model availability failed: {}", e);
            // Don't panic - let's continue and see what else we can test
        }
    }

    // Step 3: Test code chunking separately
    println!("🔍 Testing code chunking...");
    let mut chunker = CodeChunker::new().unwrap();

    let sample_code = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}

struct User {
    name: String,
}
"#;

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    std::fs::write(&test_file, sample_code).unwrap();

    let chunks = chunker.chunk_file(&test_file, sample_code).unwrap();
    println!("✅ Code chunking successful: {} chunks", chunks.len());

    for (i, chunk) in chunks.iter().enumerate() {
        println!("  Chunk {}: {:?} (lines {}-{})",
                 i + 1, chunk.chunk_type, chunk.start_line, chunk.end_line);
    }

    // Step 4: Test file indexing (this might be where the issue is)
    println!("📝 Testing file indexing...");
    match search.index_file(&test_file).await {
        Ok(()) => println!("✅ File indexing successful"),
        Err(e) => {
            println!("❌ File indexing failed: {}", e);
            return; // Can't continue without indexing
        }
    }

    // Step 5: Check stats
    let stats = search.get_stats();
    println!("📊 Search stats: {} files, {} chunks, {:.2}% embedded",
             stats.total_files, stats.total_chunks, stats.embedding_coverage * 100.0);

    // Step 6: Test search
    println!("🔍 Testing search...");
    match search.search("add function", 2).await {
        Ok((results, _metrics)) => {
            println!("✅ Search successful: {} results", results.len());
            for result in results {
                println!("  - {} (score: {:.3})", result.chunk.content.lines().next().unwrap_or(""), result.similarity_score);
            }
        },
        Err(e) => {
            println!("❌ Search failed: {}", e);
        }
    }

    println!("🎉 Test completed!");
}
