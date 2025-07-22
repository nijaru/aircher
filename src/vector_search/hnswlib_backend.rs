#[cfg(feature = "hnswlib-rs")]
use anyhow::Result;
#[cfg(feature = "hnswlib-rs")]
use std::path::PathBuf;
#[cfg(feature = "hnswlib-rs")]
use std::time::Instant;
#[cfg(feature = "hnswlib-rs")]
use tokio::fs;
#[cfg(feature = "hnswlib-rs")]
use tracing::{info, warn};

#[cfg(feature = "hnswlib-rs")]
use super::backend::{VectorSearchBackend, IndexStats};
#[cfg(feature = "hnswlib-rs")]
use super::{EmbeddingVector, ChunkMetadata, SearchResult, SearchFilter, ChunkType};

// TODO: Proper hnswlib-rs integration - currently using placeholder implementation
#[cfg(feature = "hnswlib-rs")]
pub struct HnswlibBackend {
    dimension: usize,
    storage_path: PathBuf,
    embeddings: Vec<Vec<f32>>,
    metadata: Vec<ChunkMetadata>,
    index_built: bool,
}

#[cfg(feature = "hnswlib-rs")]
impl VectorSearchBackend for HnswlibBackend {
    fn new(storage_path: PathBuf, dimension: usize) -> Result<Self> {
        Ok(Self {
            dimension,
            storage_path,
            embeddings: Vec::new(),
            metadata: Vec::new(),
            index_built: false,
        })
    }
    
    fn add_embedding(&mut self, embedding: Vec<f32>, metadata: ChunkMetadata) -> Result<()> {
        if embedding.len() != self.dimension {
            return Err(anyhow::anyhow!("Embedding dimension mismatch: expected {}, got {}", 
                self.dimension, embedding.len()));
        }
        
        self.embeddings.push(embedding);
        self.metadata.push(metadata);
        Ok(())
    }
    
    fn build_index(&mut self) -> Result<()> {
        if self.embeddings.is_empty() {
            return Err(anyhow::anyhow!("No embeddings to index"));
        }

        let start = Instant::now();
        let total_embeddings = self.embeddings.len();
        
        info!("Building placeholder HNSW index with {} embeddings (hnswlib-rs prototype)", total_embeddings);
        
        // TODO: Replace with actual hnswlib-rs implementation
        // Currently using a placeholder that just marks the index as built
        println!("⚡ Building high-performance index prototype for {} vectors...", total_embeddings);
        
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        self.index_built = true;
        
        let elapsed = start.elapsed();
        info!("Placeholder HNSW index built in {:.2}s", elapsed.as_secs_f64());
        println!("✅ Prototype index built successfully in {:.1}s", elapsed.as_secs_f64());
        
        Ok(())
    }
    
    fn search(&self, query_embedding: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        if !self.index_built {
            return Err(anyhow::anyhow!("Index not built. Call build_index() first"));
        }

        if query_embedding.len() != self.dimension {
            return Err(anyhow::anyhow!("Query embedding dimension mismatch"));
        }

        // TODO: Replace with actual hnswlib-rs search
        // Currently using a simple linear search as placeholder
        let mut similarities: Vec<(usize, f32)> = Vec::new();
        
        for (idx, embedding) in self.embeddings.iter().enumerate() {
            let similarity = cosine_similarity(query_embedding, embedding);
            similarities.push((idx, similarity));
        }
        
        // Sort by similarity (higher is better)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let mut results = Vec::new();
        for (idx, similarity) in similarities.into_iter().take(k) {
            if idx >= self.metadata.len() {
                continue;
            }
            
            let metadata = &self.metadata[idx];
            results.push(SearchResult {
                file_path: metadata.file_path.clone(),
                content: metadata.content.clone(),
                start_line: metadata.start_line,
                end_line: metadata.end_line,
                similarity_score: similarity,
                chunk_type: metadata.chunk_type.clone(),
            });
        }

        Ok(results)
    }
    
    fn search_with_filter(&self, query_embedding: &[f32], k: usize, filter: &SearchFilter) -> Result<Vec<SearchResult>> {
        // TODO: Implement proper filtering with hnswlib-rs
        let results = self.search(query_embedding, k * 2)?; // Get more results for filtering
        
        let filtered_results: Vec<SearchResult> = results
            .into_iter()
            .filter(|result| {
                let metadata = ChunkMetadata {
                    file_path: result.file_path.clone(),
                    start_line: result.start_line,
                    end_line: result.end_line,
                    chunk_type: result.chunk_type.clone(),
                    content: result.content.clone(),
                };
                filter.matches(&metadata)
            })
            .take(k)
            .collect();

        Ok(filtered_results)
    }
    
    async fn save_index(&self) -> Result<()> {
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // Save metadata
        let metadata_path = self.storage_path.with_extension("hnswlib.metadata.json");
        let metadata_json = serde_json::to_string_pretty(&self.metadata)?;
        fs::write(&metadata_path, metadata_json).await?;
        
        // Save embeddings
        let embeddings_path = self.storage_path.with_extension("hnswlib.embeddings.json");
        let embeddings_json = serde_json::to_string_pretty(&self.embeddings)?;
        fs::write(&embeddings_path, embeddings_json).await?;
        
        // Save index state
        let state_path = self.storage_path.with_extension("hnswlib.state.json");
        let state = serde_json::json!({
            "backend": "hnswlib-rs-prototype",
            "index_built": self.index_built,
            "vector_count": self.embeddings.len(),
            "dimension": self.dimension,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state)?).await?;
        
        info!("hnswlib-rs prototype index saved: metadata={}, embeddings={}, state={}", 
              metadata_path.display(), embeddings_path.display(), state_path.display());
        Ok(())
    }
    
    async fn load_index(&mut self) -> Result<()> {
        let metadata_path = self.storage_path.with_extension("hnswlib.metadata.json");
        let embeddings_path = self.storage_path.with_extension("hnswlib.embeddings.json");

        if !metadata_path.exists() || !embeddings_path.exists() {
            return Err(anyhow::anyhow!("hnswlib-rs prototype index files not found"));
        }

        // Load metadata
        let metadata_content = fs::read_to_string(&metadata_path).await?;
        self.metadata = serde_json::from_str(&metadata_content)?;
        
        // Load embeddings
        let embeddings_content = fs::read_to_string(&embeddings_path).await?;
        self.embeddings = serde_json::from_str(&embeddings_content)?;
        
        // Check if index was previously built
        let state_path = self.storage_path.with_extension("hnswlib.state.json");
        if state_path.exists() {
            let state_content = fs::read_to_string(&state_path).await?;
            let state: serde_json::Value = serde_json::from_str(&state_content)?;
            self.index_built = state.get("index_built").and_then(|v| v.as_bool()).unwrap_or(false);
            if self.index_built && !self.embeddings.is_empty() {
                info!("Loaded hnswlib-rs prototype index for {} embeddings", self.embeddings.len());
            }
        } else {
            info!("hnswlib-rs prototype index data loaded: {} vectors (not built)", self.embeddings.len());
        }
        
        Ok(())
    }
    
    fn get_stats(&self) -> IndexStats {
        IndexStats {
            total_vectors: self.metadata.len(),
            dimension: self.dimension,
            index_built: self.index_built,
        }
    }
    
    fn needs_index_build(&self) -> bool {
        !self.embeddings.is_empty() && !self.index_built
    }
    
    fn clear(&mut self) {
        self.index_built = false;
        self.embeddings.clear();
        self.metadata.clear();
    }
    
    fn backend_name(&self) -> &'static str {
        "hnswlib-rs-prototype"
    }
}

#[cfg(feature = "hnswlib-rs")]
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|y| y * y).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "hnswlib-rs")]
    use super::*;
    #[cfg(feature = "hnswlib-rs")]
    use tempfile::TempDir;

    #[cfg(feature = "hnswlib-rs")]
    #[test]
    fn test_hnswlib_backend_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("test_index");
        
        let backend = HnswlibBackend::new(storage_path, 768).unwrap();
        assert_eq!(backend.dimension, 768);
        assert!(!backend.index_built);
        assert_eq!(backend.backend_name(), "hnswlib-rs-prototype");
    }

    #[cfg(feature = "hnswlib-rs")]
    #[test]
    fn test_add_embedding() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("test_index");
        let mut backend = HnswlibBackend::new(storage_path, 3).unwrap();
        
        let embedding = vec![1.0, 2.0, 3.0];
        let metadata = ChunkMetadata {
            file_path: PathBuf::from("test.rs"),
            start_line: 1,
            end_line: 10,
            chunk_type: ChunkType::Function,
            content: "fn test() {}".to_string(),
        };
        
        backend.add_embedding(embedding, metadata).unwrap();
        assert_eq!(backend.embeddings.len(), 1);
        assert_eq!(backend.metadata.len(), 1);
    }
}