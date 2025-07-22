use anyhow::Result;
use instant_distance::{Builder, Point, Search};
use std::path::PathBuf;
use std::time::Instant;
use tokio::fs;
use tracing::{info, warn};

use super::backend::{VectorSearchBackend, IndexStats};
use super::{EmbeddingVector, ChunkMetadata, SearchResult, SearchFilter, ChunkType};

/// Original instant-distance backend implementation
pub struct InstantDistanceBackend {
    index: Option<instant_distance::HnswMap<EmbeddingVector, usize>>,
    dimension: usize,
    storage_path: PathBuf,
    embeddings: Vec<EmbeddingVector>,
    metadata: Vec<ChunkMetadata>,
}

impl VectorSearchBackend for InstantDistanceBackend {
    fn new(storage_path: PathBuf, dimension: usize) -> Result<Self> {
        Ok(Self {
            index: None,
            dimension,
            storage_path,
            embeddings: Vec::new(),
            metadata: Vec::new(),
        })
    }
    
    fn add_embedding(&mut self, embedding: Vec<f32>, metadata: ChunkMetadata) -> Result<()> {
        if embedding.len() != self.dimension {
            return Err(anyhow::anyhow!("Embedding dimension mismatch: expected {}, got {}", 
                self.dimension, embedding.len()));
        }
        
        self.embeddings.push(EmbeddingVector(embedding));
        self.metadata.push(metadata);
        
        Ok(())
    }
    
    fn build_index(&mut self) -> Result<()> {
        if self.embeddings.is_empty() {
            return Err(anyhow::anyhow!("No embeddings to index"));
        }

        let start = Instant::now();
        
        // TEMPORARY: Limit vectors to make search usable until we migrate to hnswlib-rs
        const MAX_VECTORS_FOR_INDEX: usize = 1000;
        let total_embeddings = self.embeddings.len();
        let limited = total_embeddings > MAX_VECTORS_FOR_INDEX;
        
        let embeddings_to_index = if limited {
            MAX_VECTORS_FOR_INDEX
        } else {
            total_embeddings
        };
        
        info!("Building HNSW index with {} embeddings (total: {}) using instant-distance", embeddings_to_index, total_embeddings);
        
        // Show progress for large indexes
        if limited {
            println!("⚠️  Large codebase detected ({} vectors)", total_embeddings);
            println!("   Building limited index with first {} vectors for faster search.", MAX_VECTORS_FOR_INDEX);
            println!("   Full index support available with hnswlib-rs backend.");
        } else if total_embeddings > 500 {
            println!("⏳ Building search index for {} vectors...", total_embeddings);
            println!("   This is a one-time operation using instant-distance.");
        }
        
        // Create points for instant-distance - limit if needed
        let points: Vec<EmbeddingVector> = if limited {
            self.embeddings.iter().take(MAX_VECTORS_FOR_INDEX).cloned().collect()
        } else {
            self.embeddings.clone()
        };
        
        // Create values (indices) for mapping back to metadata
        let values: Vec<usize> = (0..embeddings_to_index).collect();

        // Build HNSW index
        let index = Builder::default().build(points, values);
        
        self.index = Some(index);
        
        let elapsed = start.elapsed();
        info!("HNSW index built successfully in {:.2}s using instant-distance", elapsed.as_secs_f64());
        
        if limited {
            println!("✅ Limited index built successfully in {:.1}s", elapsed.as_secs_f64());
        } else if total_embeddings > 500 {
            println!("✅ Index built successfully in {:.1}s", elapsed.as_secs_f64());
        }
        
        Ok(())
    }
    
    fn search(&self, query_embedding: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        let index = self.index.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Index not built. Call build_index() first"))?;

        if query_embedding.len() != self.dimension {
            return Err(anyhow::anyhow!("Query embedding dimension mismatch"));
        }

        // Search using instant-distance
        let mut search = Search::default();
        let query_vector = EmbeddingVector(query_embedding.to_vec());
        let search_results = index.search(&query_vector, &mut search);
        
        let mut results = Vec::new();
        for result in search_results.take(k) {
            let idx = *result.value;
            
            if idx >= self.metadata.len() {
                warn!("Index out of bounds from HNSW: {}", idx);
                continue;
            }
            
            let metadata = &self.metadata[idx];
            // Convert distance to similarity (lower distance = higher similarity)
            let similarity = 1.0 / (1.0 + result.distance);
            
            results.push(SearchResult {
                file_path: metadata.file_path.clone(),
                content: metadata.content.clone(),
                start_line: metadata.start_line,
                end_line: metadata.end_line,
                similarity_score: similarity,
                chunk_type: metadata.chunk_type.clone(),
            });
        }

        // Sort by similarity (higher is better)
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score)
            .unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }
    
    fn search_with_filter(&self, query_embedding: &[f32], k: usize, filter: &SearchFilter) -> Result<Vec<SearchResult>> {
        let index = self.index.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Index not built. Call build_index() first"))?;

        if query_embedding.len() != self.dimension {
            return Err(anyhow::anyhow!("Query embedding dimension mismatch"));
        }

        // Search using instant-distance (get more results to account for filtering)
        let mut search = Search::default();
        let query_vector = EmbeddingVector(query_embedding.to_vec());
        let search_results = index.search(&query_vector, &mut search);
        
        let mut results = Vec::new();
        let mut count = 0;
        
        // Apply filter during result processing to avoid examining filtered-out items
        for result in search_results {
            if count >= k {
                break;
            }
            
            let idx = *result.value;
            
            if idx >= self.metadata.len() {
                warn!("Index out of bounds from HNSW: {}", idx);
                continue;
            }
            
            let metadata = &self.metadata[idx];
            
            // Apply filter - skip if doesn't match
            if !filter.matches(metadata) {
                continue;
            }
            
            // Convert distance to similarity (lower distance = higher similarity)
            let similarity = 1.0 / (1.0 + result.distance);
            
            results.push(SearchResult {
                file_path: metadata.file_path.clone(),
                content: metadata.content.clone(),
                start_line: metadata.start_line,
                end_line: metadata.end_line,
                similarity_score: similarity,
                chunk_type: metadata.chunk_type.clone(),
            });
            
            count += 1;
        }

        // Sort by similarity (higher is better)
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score)
            .unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }
    
    async fn save_index(&self) -> Result<()> {
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // Save metadata
        let metadata_path = self.storage_path.with_extension("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&self.metadata)?;
        fs::write(&metadata_path, metadata_json).await?;
        
        // Save embeddings (enables index reconstruction)
        let embeddings_path = self.storage_path.with_extension("embeddings.json");
        let embeddings_data: Vec<Vec<f32>> = self.embeddings.iter().map(|e| e.0.clone()).collect();
        let embeddings_json = serde_json::to_string_pretty(&embeddings_data)?;
        fs::write(&embeddings_path, embeddings_json).await?;
        
        // Save index state
        let state_path = self.storage_path.with_extension("state.json");
        let state = serde_json::json!({
            "backend": "instant-distance",
            "index_built": self.index.is_some(),
            "vector_count": self.embeddings.len(),
            "dimension": self.dimension,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state)?).await?;
        
        info!("instant-distance index saved: metadata={}, embeddings={}, state={}", 
              metadata_path.display(), embeddings_path.display(), state_path.display());
        Ok(())
    }
    
    async fn load_index(&mut self) -> Result<()> {
        let metadata_path = self.storage_path.with_extension("metadata.json");
        let embeddings_path = self.storage_path.with_extension("embeddings.json");

        if !metadata_path.exists() || !embeddings_path.exists() {
            return Err(anyhow::anyhow!("instant-distance index files not found"));
        }

        // Load metadata
        let metadata_content = fs::read_to_string(&metadata_path).await?;
        self.metadata = serde_json::from_str(&metadata_content)?;
        
        // Load embeddings
        let embeddings_content = fs::read_to_string(&embeddings_path).await?;
        let embeddings_data: Vec<Vec<f32>> = serde_json::from_str(&embeddings_content)?;
        self.embeddings = embeddings_data.into_iter().map(EmbeddingVector).collect();
        
        // Check if index was previously built
        let state_path = self.storage_path.with_extension("state.json");
        if state_path.exists() {
            let state_content = fs::read_to_string(&state_path).await?;
            let state: serde_json::Value = serde_json::from_str(&state_content)?;
            let was_built = state.get("index_built").and_then(|v| v.as_bool()).unwrap_or(false);
            if was_built {
                info!("Loaded {} embeddings. Index will be built on-demand during search.", self.embeddings.len());
                // Note: We don't rebuild the index here because instant-distance 
                // can't serialize the HNSW graph. The index will be built lazily
                // during the first search operation.
            }
        } else {
            info!("instant-distance index data loaded: {} vectors (not built)", self.embeddings.len());
        }
        
        Ok(())
    }
    
    fn get_stats(&self) -> IndexStats {
        IndexStats {
            total_vectors: self.metadata.len(),
            dimension: self.dimension,
            index_built: self.index.is_some(),
        }
    }
    
    fn needs_index_build(&self) -> bool {
        // If no embeddings, no need to build
        if self.embeddings.is_empty() {
            return false;
        }
        
        // If index already exists in memory, it's valid
        if self.index.is_some() {
            return false;
        }
        
        // Otherwise, we need to build
        true
    }
    
    fn clear(&mut self) {
        self.index = None;
        self.embeddings.clear();
        self.metadata.clear();
    }
    
    fn backend_name(&self) -> &'static str {
        "instant-distance"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_instant_distance_backend_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("test_index");
        
        let backend = InstantDistanceBackend::new(storage_path, 768).unwrap();
        assert_eq!(backend.dimension, 768);
        assert!(backend.index.is_none());
        assert_eq!(backend.backend_name(), "instant-distance");
    }

    #[test]
    fn test_add_embedding() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("test_index");
        let mut backend = InstantDistanceBackend::new(storage_path, 3).unwrap();
        
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