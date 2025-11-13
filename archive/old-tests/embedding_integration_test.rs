use aircher::cost::{EmbeddingManager, EmbeddingConfig};
use anyhow::Result;

#[tokio::test]
async fn test_ollama_embedding_integration() -> Result<()> {
    // Test default configuration
    let config = EmbeddingConfig::default();
    assert_eq!(config.preferred_model, "embeddinggemma");
    assert!(config.auto_download);
    assert!(config.use_ollama_if_available);
    assert_eq!(config.fallback_model, Some("nomic-embed-text".to_string()));

    let mut manager = EmbeddingManager::new(config);

    // Check Ollama availability
    let available = manager.check_ollama_availability().await;
    println!("Ollama available: {}", available);

    if available {
        // Test embedding generation
        let test_texts = vec![
            "async function handleRequest() { return await fetch(url); }",
            "def handle_request(): return requests.get(url)",
            "fn handle_request() -> Result<Response> { reqwest::get(url).await }",
        ];

        for text in test_texts {
            match manager.generate_embeddings(text).await {
                Ok(embeddings) => {
                    println!("✅ Generated {} dimensional embeddings for: {}...",
                        embeddings.len(),
                        &text[..30.min(text.len())]);
                    assert!(!embeddings.is_empty(), "Embeddings should not be empty");

                    // Check embedding quality - all values should be normalized
                    let magnitude: f32 = embeddings.iter().map(|x| x * x).sum::<f32>().sqrt();
                    assert!(magnitude > 0.9 && magnitude < 1.1, "Embeddings should be normalized");
                },
                Err(e) => {
                    println!("⚠️ Failed to generate embeddings: {}", e);
                    // Don't fail test if Ollama model not installed
                    if !e.to_string().contains("not found") {
                        return Err(e);
                    }
                }
            }
        }

        // Test batch embeddings
        let batch_texts = vec![
            "function add(a, b) { return a + b; }".to_string(),
            "class User { constructor(name) { this.name = name; } }".to_string(),
        ];

        match manager.generate_batch_embeddings(&batch_texts).await {
            Ok(batch_embeddings) => {
                println!("✅ Generated batch embeddings for {} texts", batch_embeddings.len());
                assert_eq!(batch_embeddings.len(), 2);
            },
            Err(e) => {
                println!("⚠️ Batch embedding generation failed: {}", e);
            }
        }
    } else {
        println!("ℹ️ Skipping embedding tests - Ollama not available");
    }

    Ok(())
}

#[tokio::test]
async fn test_embedding_model_selection() -> Result<()> {
    let mut config = EmbeddingConfig::default();

    // Test with different model preferences
    config.preferred_model = "nomic-embed-text".to_string();
    let mut manager = EmbeddingManager::new(config);

    // Get model selection prompt
    let prompt = manager.generate_selection_prompt().await;
    println!("Model selection prompt generated: {} chars", prompt.len());
    assert!(prompt.contains("embedding model") || prompt.contains("Embedding"));

    Ok(())
}

#[tokio::test]
async fn test_embedding_fallback_chain() -> Result<()> {
    let config = EmbeddingConfig {
        preferred_model: "non-existent-model".to_string(),
        fallback_model: Some("nomic-embed-text".to_string()),
        auto_download: false,
        use_ollama_if_available: true,
        max_model_size_mb: 1000,
    };

    let mut manager = EmbeddingManager::new(config);

    // Should fall back gracefully
    let status = manager.get_status_summary().await;
    println!("Status with non-existent model: {}", status);
    assert!(status.contains("Embedding Models Status"));

    Ok(())
}
