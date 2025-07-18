use std::path::PathBuf;
use aircher::vector_search::{VectorSearchEngine, ChunkMetadata, ChunkType};

#[test]
fn test_instant_distance_integration() {
    // Create temporary storage path
    let storage_path = PathBuf::from("/tmp/test_instant_distance");
    
    // Create vector search engine
    let mut engine = VectorSearchEngine::new(storage_path, 3).unwrap();
    
    // Add some test embeddings
    let embedding1 = vec![1.0, 0.0, 0.0];
    let metadata1 = ChunkMetadata {
        file_path: PathBuf::from("test1.rs"),
        start_line: 1,
        end_line: 10,
        chunk_type: ChunkType::Function,
        content: "fn test_function() {}".to_string(),
    };
    engine.add_embedding(embedding1, metadata1).unwrap();
    
    let embedding2 = vec![0.0, 1.0, 0.0];
    let metadata2 = ChunkMetadata {
        file_path: PathBuf::from("test2.rs"),
        start_line: 1,
        end_line: 10,
        chunk_type: ChunkType::Class,
        content: "struct TestStruct {}".to_string(),
    };
    engine.add_embedding(embedding2, metadata2).unwrap();
    
    let embedding3 = vec![0.9, 0.1, 0.0]; // Similar to embedding1
    let metadata3 = ChunkMetadata {
        file_path: PathBuf::from("test3.rs"),
        start_line: 1,
        end_line: 10,
        chunk_type: ChunkType::Function,
        content: "fn another_function() {}".to_string(),
    };
    engine.add_embedding(embedding3, metadata3).unwrap();
    
    // Build the index
    engine.build_index().unwrap();
    
    // Test search with query similar to embedding1
    let query = vec![1.0, 0.0, 0.0];
    let results = engine.search(&query, 2).unwrap();
    
    // Should find 2 results, with the most similar first
    assert_eq!(results.len(), 2);
    assert!(results[0].content.contains("test_function"));
    assert!(results[1].content.contains("another_function"));
    
    // Test stats
    let stats = engine.get_stats();
    assert_eq!(stats.total_vectors, 3);
    assert!(stats.index_built);
    assert_eq!(stats.dimension, 3);
    
    println!("✅ instant-distance integration test passed!");
}