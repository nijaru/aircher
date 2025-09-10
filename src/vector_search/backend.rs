use anyhow::Result;
use std::path::PathBuf;

use super::{ChunkMetadata, SearchResult, SearchFilter};

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
    #[allow(async_fn_in_trait)]
    async fn save_index(&self) -> Result<()>;
    
    /// Load index from disk  
    #[allow(async_fn_in_trait)]
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

