use anyhow::Result;
use std::path::PathBuf;

// Backend abstraction
pub mod backend;
pub use backend::{VectorSearchBackend, IndexStats};

// Backend implementations
pub mod hnswlib_backend;
pub use hnswlib_backend::HnswlibBackend;

// Core data types
#[derive(Debug, Clone)]
pub struct EmbeddingVector(pub Vec<f32>);


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChunkMetadata {
    pub file_path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub chunk_type: ChunkType,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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

/// Filter criteria for vector search optimization
#[derive(Debug, Clone, Default)]
pub struct SearchFilter {
    pub file_types: Option<Vec<String>>,
    pub chunk_types: Option<Vec<ChunkType>>,
    pub exclude_patterns: Option<Vec<String>>,
    pub include_patterns: Option<Vec<String>>,
}

impl SearchFilter {
    pub fn matches(&self, metadata: &ChunkMetadata) -> bool {
        // File type filtering
        if let Some(ref file_types) = self.file_types {
            if let Some(ext) = metadata.file_path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                let lang = language_from_extension(&ext_lower);
                let normalized_types: Vec<String> = file_types.iter()
                    .map(|t| normalize_file_type(t))
                    .collect();
                
                if !normalized_types.contains(&ext_lower) && !normalized_types.contains(&lang) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Chunk type filtering
        if let Some(ref chunk_types) = self.chunk_types {
            if !chunk_types.contains(&metadata.chunk_type) {
                return false;
            }
        }
        
        // Exclude pattern filtering
        if let Some(ref exclude_patterns) = self.exclude_patterns {
            let path_str = metadata.file_path.to_string_lossy().to_lowercase();
            for pattern in exclude_patterns {
                let pattern_lower = pattern.to_lowercase();
                if path_str.contains(&pattern_lower) || 
                   metadata.content.to_lowercase().contains(&pattern_lower) {
                    return false;
                }
            }
        }
        
        // Include pattern filtering
        if let Some(ref include_patterns) = self.include_patterns {
            let path_str = metadata.file_path.to_string_lossy().to_lowercase();
            let matches_include = include_patterns.iter().any(|pattern| {
                let pattern_lower = pattern.to_lowercase();
                path_str.contains(&pattern_lower)
            });
            if !matches_include {
                return false;
            }
        }
        
        true
    }
}


/// Unified vector search engine using HnswlibBackend
pub struct VectorSearchEngine {
    backend: HnswlibBackend,
}

impl VectorSearchEngine {
    /// Create new vector search engine
    pub fn new(storage_path: PathBuf, dimension: usize) -> Result<Self> {
        let backend = HnswlibBackend::new(storage_path, dimension)?;
        Ok(Self { backend })
    }
    
    /// Get backend name
    pub fn backend_name(&self) -> &str {
        self.backend.backend_name()
    }
    
    /// Add embedding to the index
    pub fn add_embedding(&mut self, embedding: Vec<f32>, metadata: ChunkMetadata) -> Result<()> {
        self.backend.add_embedding(embedding, metadata)
    }
    
    /// Build the HNSW index from accumulated embeddings
    pub fn build_index(&mut self) -> Result<()> {
        self.backend.build_index()
    }
    
    /// Search for similar vectors
    pub fn search(&self, query_embedding: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        self.backend.search(query_embedding, k)
    }
    
    /// Search with pre-filtering for better performance
    pub fn search_with_filter(&self, query_embedding: &[f32], k: usize, filter: &SearchFilter) -> Result<Vec<SearchResult>> {
        self.backend.search_with_filter(query_embedding, k, filter)
    }
    
    /// Save index to disk
    pub async fn save_index(&self) -> Result<()> {
        self.backend.save_index().await
    }
    
    /// Load index from disk
    pub async fn load_index(&mut self) -> Result<()> {
        self.backend.load_index().await
    }
    
    /// Get statistics about the index
    pub fn get_stats(&self) -> IndexStats {
        self.backend.get_stats()
    }
    
    /// Check if we need to build the index
    pub fn needs_index_build(&self) -> bool {
        self.backend.needs_index_build()
    }
    
    /// Clear the index and metadata
    pub fn clear(&mut self) {
        self.backend.clear()
    }
}

/// Normalize file type input (e.g., "rs" -> "rs", "rust" -> "rs")
fn normalize_file_type(file_type: &str) -> String {
    match file_type.to_lowercase().as_str() {
        "rust" => "rs".to_string(),
        "python" => "py".to_string(),
        "javascript" => "js".to_string(),
        "typescript" => "ts".to_string(),
        "c++" | "cpp" => "cpp".to_string(),
        "c#" | "csharp" => "cs".to_string(),
        "golang" | "go" => "go".to_string(),
        other => other.to_string(),
    }
}

/// Get language name from file extension
fn language_from_extension(ext: &str) -> String {
    match ext.to_lowercase().as_str() {
        "rs" => "rust".to_string(),
        "py" => "python".to_string(),
        "js" => "javascript".to_string(),
        "jsx" => "javascript".to_string(),
        "ts" => "typescript".to_string(),
        "tsx" => "typescript".to_string(),
        "cpp" | "cc" | "cxx" => "cpp".to_string(),
        "c" => "c".to_string(),
        "h" | "hpp" => "c".to_string(),
        "cs" => "csharp".to_string(),
        "go" => "go".to_string(),
        "java" => "java".to_string(),
        "rb" => "ruby".to_string(),
        "php" => "php".to_string(),
        "swift" => "swift".to_string(),
        "kt" => "kotlin".to_string(),
        other => other.to_string(),
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
        
        // Test engine creation
        let engine = VectorSearchEngine::new(storage_path.clone(), 768).unwrap();
        assert_eq!(engine.backend_name(), "hnswlib-rs");
    }

    #[test]
    fn test_search_filter() {
        let filter = SearchFilter {
            file_types: Some(vec!["rs".to_string()]),
            chunk_types: Some(vec![ChunkType::Function]),
            exclude_patterns: None,
            include_patterns: None,
        };
        
        let metadata = ChunkMetadata {
            file_path: PathBuf::from("test.rs"),
            start_line: 1,
            end_line: 10,
            chunk_type: ChunkType::Function,
            content: "fn test() {}".to_string(),
        };
        
        assert!(filter.matches(&metadata));
        
        // Test non-matching filter
        let metadata2 = ChunkMetadata {
            file_path: PathBuf::from("test.py"),
            start_line: 1,
            end_line: 10,
            chunk_type: ChunkType::Function,
            content: "def test(): pass".to_string(),
        };
        
        assert!(!filter.matches(&metadata2));
    }
}