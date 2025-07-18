use std::path::PathBuf;
use tempfile::TempDir;
use aircher::cost::{EmbeddingManager, EmbeddingConfig};
use aircher::cost::swerank_integration::SweRankEmbedModel;

#[tokio::test]
async fn test_swerank_model_direct() {
    println!("🧪 Testing SweRankEmbed model directly...");
    
    // Test 1: Initialize SweRankEmbed model directly
    match SweRankEmbedModel::new().await {
        Ok(model) => {
            println!("✅ SweRankEmbed model initialized successfully");
            
            // Test 2: Generate embeddings
            let test_text = "fn add(a: i32, b: i32) -> i32 { a + b }";
            match model.generate_embeddings(test_text).await {
                Ok(embeddings) => {
                    println!("✅ Generated embeddings: {} dimensions", embeddings.len());
                    assert_eq!(embeddings.len(), 768);
                },
                Err(e) => {
                    println!("❌ Embedding generation failed: {}", e);
                    panic!("Embedding generation failed: {}", e);
                }
            }
        },
        Err(e) => {
            println!("❌ SweRankEmbed initialization failed: {}", e);
            panic!("SweRankEmbed initialization failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_embedding_manager() {
    println!("🧪 Testing EmbeddingManager...");
    
    let config = EmbeddingConfig::default();
    let mut manager = EmbeddingManager::new(config);
    
    let test_text = "fn multiply(x: f64, y: f64) -> f64 { x * y }";
    
    match manager.generate_embeddings(test_text).await {
        Ok(embeddings) => {
            println!("✅ EmbeddingManager generated embeddings: {} dimensions", embeddings.len());
        },
        Err(e) => {
            println!("❌ EmbeddingManager failed: {}", e);
            // This might fail if Ollama is not available, which is expected
        }
    }
}

#[tokio::test]
async fn test_swerank_specific() {
    println!("🧪 Testing SweRankEmbed specifically...");
    
    let config = EmbeddingConfig::default();
    let mut manager = EmbeddingManager::new(config);
    
    let test_text = "struct User { name: String }";
    
    // Force it to use the swerank model
    match manager.generate_embeddings_with_model(test_text, "swerank-embed-small").await {
        Ok(embeddings) => {
            println!("✅ SweRankEmbed via EmbeddingManager: {} dimensions", embeddings.len());
        },
        Err(e) => {
            println!("❌ SweRankEmbed via EmbeddingManager failed: {}", e);
            panic!("SweRankEmbed via EmbeddingManager failed: {}", e);
        }
    }
}