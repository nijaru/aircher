use anyhow::Result;
use instant_distance::{Builder, Point, Search};
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn};

// Wrapper type to implement Point trait (orphan rule workaround)
#[derive(Debug, Clone)]
pub struct EmbeddingVector(pub Vec<f32>);

impl Point for EmbeddingVector {
    fn distance(&self, other: &Self) -> f32 {
        // Cosine distance calculation
        let dot_product: f32 = self.0.iter().zip(other.0.iter()).map(|(a, b)| a * b).sum();
        let norm_a: f32 = self.0.iter().map(|a| a * a).sum::<f32>().sqrt();
        let norm_b: f32 = other.0.iter().map(|b| b * b).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            1.0 // Maximum distance for zero vectors
        } else {
            1.0 - (dot_product / (norm_a * norm_b))
        }
    }
}

/// High-performance vector search using instant-distance
pub struct VectorSearchEngine {
    index: Option<instant_distance::HnswMap<EmbeddingVector, usize>>,
    dimension: usize,
    storage_path: PathBuf,
    embeddings: Vec<EmbeddingVector>,
    metadata: Vec<ChunkMetadata>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChunkMetadata {
    pub file_path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub chunk_type: ChunkType,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ChunkType {
    Function,
    Class,
    Module,
    Comment,
    Generic,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: PathBuf,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub similarity_score: f32,
    pub chunk_type: ChunkType,
}

impl VectorSearchEngine {
    /// Create new vector search engine
    pub fn new(storage_path: PathBuf, dimension: usize) -> Result<Self> {
        Ok(Self {
            index: None,
            dimension,
            storage_path,
            embeddings: Vec::new(),
            metadata: Vec::new(),
        })
    }

    /// Add embedding to the index
    pub fn add_embedding(&mut self, embedding: Vec<f32>, metadata: ChunkMetadata) -> Result<()> {
        if embedding.len() != self.dimension {
            return Err(anyhow::anyhow!("Embedding dimension mismatch: expected {}, got {}", 
                self.dimension, embedding.len()));
        }
        
        self.embeddings.push(EmbeddingVector(embedding));
        self.metadata.push(metadata);
        
        Ok(())
    }

    /// Build the HNSW index from accumulated embeddings
    pub fn build_index(&mut self) -> Result<()> {
        if self.embeddings.is_empty() {
            return Err(anyhow::anyhow!("No embeddings to index"));
        }

        info!("Building HNSW index with {} embeddings", self.embeddings.len());
        
        // Create points for instant-distance - just the embeddings
        let points: Vec<EmbeddingVector> = self.embeddings.clone();
        
        // Create values (indices) for mapping back to metadata
        let values: Vec<usize> = (0..self.embeddings.len()).collect();

        // Build HNSW index
        let index = Builder::default().build(points, values);
        
        self.index = Some(index);
        
        info!("HNSW index built successfully");
        Ok(())
    }

    /// Search for similar vectors
    pub fn search(&self, query_embedding: &[f32], k: usize) -> Result<Vec<SearchResult>> {
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

    /// Save index to disk
    pub async fn save_index(&self) -> Result<()> {
        // TODO: Implement HNSW index saving (instant-distance doesn't have built-in serialization yet)
        warn!("HNSW index saving not yet implemented");
        
        // For now, just save metadata
        let metadata_path = self.storage_path.with_extension("metadata.json");
        if let Some(parent) = metadata_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        let metadata_json = serde_json::to_string_pretty(&self.metadata)?;
        fs::write(&metadata_path, metadata_json).await?;

        info!("Metadata saved to: {}", metadata_path.display());
        Ok(())
    }

    /// Load index from disk
    pub async fn load_index(&mut self) -> Result<()> {
        let metadata_path = self.storage_path.with_extension("metadata.json");

        if !metadata_path.exists() {
            return Err(anyhow::anyhow!("Metadata file not found"));
        }

        // Load metadata
        let metadata_content = fs::read_to_string(&metadata_path).await?;
        self.metadata = serde_json::from_str(&metadata_content)?;

        // TODO: Implement HNSW index loading (instant-distance doesn't have built-in serialization yet)
        warn!("HNSW index loading not yet implemented - index will need to be rebuilt");
        
        info!("Metadata loaded from: {}", metadata_path.display());
        Ok(())
    }

    /// Get statistics about the index
    pub fn get_stats(&self) -> IndexStats {
        IndexStats {
            total_vectors: self.metadata.len(),
            dimension: self.dimension,
            index_built: self.index.is_some(),
        }
    }

    /// Clear the index and metadata
    pub fn clear(&mut self) {
        self.index = None;
        self.embeddings.clear();
        self.metadata.clear();
    }
}

#[derive(Debug)]
pub struct IndexStats {
    pub total_vectors: usize,
    pub dimension: usize,
    pub index_built: bool,
}

// Make metadata serializable
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadataSerializable {
    pub file_path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub chunk_type: ChunkTypeSerializable,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkTypeSerializable {
    Function,
    Class,
    Module,
    Comment,
    Generic,
}

impl From<ChunkMetadata> for ChunkMetadataSerializable {
    fn from(metadata: ChunkMetadata) -> Self {
        Self {
            file_path: metadata.file_path,
            start_line: metadata.start_line,
            end_line: metadata.end_line,
            chunk_type: match metadata.chunk_type {
                ChunkType::Function => ChunkTypeSerializable::Function,
                ChunkType::Class => ChunkTypeSerializable::Class,
                ChunkType::Module => ChunkTypeSerializable::Module,
                ChunkType::Comment => ChunkTypeSerializable::Comment,
                ChunkType::Generic => ChunkTypeSerializable::Generic,
            },
            content: metadata.content,
        }
    }
}

impl From<ChunkMetadataSerializable> for ChunkMetadata {
    fn from(metadata: ChunkMetadataSerializable) -> Self {
        Self {
            file_path: metadata.file_path,
            start_line: metadata.start_line,
            end_line: metadata.end_line,
            chunk_type: match metadata.chunk_type {
                ChunkTypeSerializable::Function => ChunkType::Function,
                ChunkTypeSerializable::Class => ChunkType::Class,
                ChunkTypeSerializable::Module => ChunkType::Module,
                ChunkTypeSerializable::Comment => ChunkType::Comment,
                ChunkTypeSerializable::Generic => ChunkType::Generic,
            },
            content: metadata.content,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_vector_search_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("test_index");
        
        let engine = VectorSearchEngine::new(storage_path, 384).unwrap();
        assert_eq!(engine.dimension, 384);
        assert!(engine.index.is_none());
    }

    #[test]
    fn test_add_embedding() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("test_index");
        let mut engine = VectorSearchEngine::new(storage_path, 3).unwrap();
        
        let embedding = vec![1.0, 2.0, 3.0];
        let metadata = ChunkMetadata {
            file_path: PathBuf::from("test.rs"),
            start_line: 1,
            end_line: 10,
            chunk_type: ChunkType::Function,
            content: "fn test() {}".to_string(),
        };
        
        engine.add_embedding(embedding, metadata).unwrap();
        assert_eq!(engine.embeddings.len(), 1);
        assert_eq!(engine.metadata.len(), 1);
    }
}