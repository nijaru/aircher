use anyhow::Result;
use std::path::PathBuf;

use super::{EmbeddingVector, ChunkMetadata, SearchResult, SearchFilter};

/// Abstract vector search backend interface
pub trait VectorSearchBackend: Send + Sync {
    /// Create a new backend instance
    fn new(storage_path: PathBuf, dimension: usize) -> Result<Self> where Self: Sized;
    
    /// Add embedding to the backend
    fn add_embedding(&mut self, embedding: Vec<f32>, metadata: ChunkMetadata) -> Result<()>;
    
    /// Build the search index
    fn build_index(&mut self) -> Result<()>;
    
    /// Search for similar vectors
    fn search(&self, query_embedding: &[f32], k: usize) -> Result<Vec<SearchResult>>;
    
    /// Search with filtering
    fn search_with_filter(&self, query_embedding: &[f32], k: usize, filter: &SearchFilter) -> Result<Vec<SearchResult>>;
    
    /// Save index to disk
    async fn save_index(&self) -> Result<()>;
    
    /// Load index from disk
    async fn load_index(&mut self) -> Result<()>;
    
    /// Get statistics about the index
    fn get_stats(&self) -> IndexStats;
    
    /// Check if index needs to be built
    fn needs_index_build(&self) -> bool;
    
    /// Clear the index and metadata
    fn clear(&mut self);
    
    /// Get backend name for identification
    fn backend_name(&self) -> &'static str;
}

#[derive(Debug)]
pub struct IndexStats {
    pub total_vectors: usize,
    pub dimension: usize,
    pub index_built: bool,
}

/// Vector search backend type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorBackend {
    InstantDistance,
    #[cfg(feature = "hnswlib-rs")]
    HnswRs,
}

impl VectorBackend {
    /// Get the default backend based on available features
    pub fn default() -> Self {
        #[cfg(feature = "hnswlib-rs")]
        return VectorBackend::HnswRs;
        
        #[cfg(not(feature = "hnswlib-rs"))]
        VectorBackend::InstantDistance
    }
    
    /// Get backend name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            VectorBackend::InstantDistance => "instant-distance",
            #[cfg(feature = "hnswlib-rs")]
            VectorBackend::HnswRs => "hnswlib-rs",
        }
    }
    
    /// Parse backend from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "instant-distance" => Some(VectorBackend::InstantDistance),
            #[cfg(feature = "hnswlib-rs")]
            "hnswlib-rs" | "hnsw-rs" => Some(VectorBackend::HnswRs),
            _ => None,
        }
    }
    
    /// Get all available backends
    pub fn available_backends() -> Vec<VectorBackend> {
        let mut backends = vec![VectorBackend::InstantDistance];
        
        #[cfg(feature = "hnswlib-rs")]
        backends.push(VectorBackend::HnswRs);
        
        backends
    }
}

impl std::fmt::Display for VectorBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}