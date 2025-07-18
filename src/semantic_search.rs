use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};
use dirs::cache_dir;
// use rayon::prelude::*; // TODO: Re-enable when parallel processing is needed

use crate::cost::{EmbeddingManager, EmbeddingConfig};
use crate::vector_search::{VectorSearchEngine, ChunkMetadata, ChunkType as VectorChunkType};
use crate::code_chunking::{CodeChunker, ChunkType as CodeChunkType};

/// Semantic code search using embeddings and instant-distance HNSW
pub struct SemanticCodeSearch {
    embedding_manager: EmbeddingManager,
    vector_search: VectorSearchEngine,
    code_chunker: CodeChunker,
    indexed_files: Vec<IndexedFile>,
}

#[derive(Debug, Clone)]
pub struct IndexedFile {
    pub path: PathBuf,
    pub content: String,
    pub chunks: Vec<CodeChunk>,
}

#[derive(Debug, Clone)]
pub struct CodeChunk {
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub chunk_type: VectorChunkType,
    pub embedding: Option<Vec<f32>>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: PathBuf,
    pub chunk: CodeChunk,
    pub similarity_score: f32,
    pub context_lines: Vec<String>,
}

impl SemanticCodeSearch {
    pub fn new() -> Self {
        let config = EmbeddingConfig::default();
        let embedding_manager = EmbeddingManager::new(config);
        
        // Create vector search engine with typical embedding dimension
        let cache_dir = cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("aircher")
            .join("search_index");
        
        let vector_search = VectorSearchEngine::new(cache_dir, 384) // Common embedding dimension
            .unwrap_or_else(|e| {
                warn!("Failed to create vector search engine: {}", e);
                // Create a fallback with default settings
                VectorSearchEngine::new(PathBuf::from("./search_index"), 384).unwrap()
            });
        
        let code_chunker = CodeChunker::new()
            .unwrap_or_else(|e| {
                warn!("Failed to create code chunker: {}", e);
                CodeChunker::default()
            });
        
        Self {
            embedding_manager,
            vector_search,
            code_chunker,
            indexed_files: Vec::new(),
        }
    }

    /// Ensure embedding model is available from bundled resources
    pub async fn ensure_model_available(&mut self) -> Result<()> {
        let cache_dir = cache_dir()
            .ok_or_else(|| anyhow::anyhow!("Unable to determine cache directory"))?
            .join("aircher")
            .join("models");

        // Create cache directory if it doesn't exist
        tokio::fs::create_dir_all(&cache_dir).await?;
        
        let model_path = cache_dir.join("swerank-embed-small.bin");
        
        // Extract bundled model if not already present
        if !model_path.exists() {
            info!("Extracting bundled SweRankEmbed-Small model");
            let model_data = include_bytes!("../models/swerank-embed-small.bin");
            tokio::fs::write(&model_path, model_data).await?;
            info!("Model extracted to: {}", model_path.display());
        }
        
        Ok(())
    }

    /// Index a directory of code files
    pub async fn index_directory(&mut self, dir_path: &Path) -> Result<()> {
        info!("Starting semantic indexing of directory: {:?}", dir_path);
        
        let mut file_count = 0;
        let files = self.find_code_files(dir_path).await?;
        
        for file_path in files {
            match self.index_file(&file_path).await {
                Ok(_) => {
                    file_count += 1;
                    if file_count % 10 == 0 {
                        info!("Indexed {} files", file_count);
                    }
                }
                Err(e) => {
                    warn!("Failed to index {:?}: {}", file_path, e);
                }
            }
        }
        
        info!("Completed indexing {} files", file_count);
        Ok(())
    }

    /// Index a single file using improved chunking and vector search
    pub async fn index_file(&mut self, file_path: &Path) -> Result<()> {
        let content = fs::read_to_string(file_path).await?;
        
        // Use tree-sitter based chunking for better semantic boundaries
        let chunks = self.code_chunker.chunk_file(file_path, &content)?;
        
        // Generate embeddings for chunks (sequential for now due to borrowing constraints)
        let mut chunk_embeddings = Vec::new();
        for chunk in &chunks {
            let embedding_result = self.embedding_manager.generate_embeddings(&chunk.content).await;
            chunk_embeddings.push((chunk, embedding_result));
        }
        
        // Add successful embeddings to vector search
        let mut embedded_chunks = Vec::new();
        for (chunk, embedding_result) in chunk_embeddings {
            match embedding_result {
                Ok(embedding) => {
                    // Convert code chunk to vector search metadata
                    let metadata = ChunkMetadata {
                        file_path: file_path.to_path_buf(),
                        start_line: chunk.start_line,
                        end_line: chunk.end_line,
                        chunk_type: convert_chunk_type(&chunk.chunk_type),
                        content: chunk.content.clone(),
                    };
                    
                    // Add to vector search index
                    self.vector_search.add_embedding(embedding.clone(), metadata)?;
                    
                    // Keep for compatibility with existing code
                    let code_chunk = CodeChunk {
                        content: chunk.content.clone(),
                        start_line: chunk.start_line,
                        end_line: chunk.end_line,
                        chunk_type: convert_chunk_type(&chunk.chunk_type),
                        embedding: Some(embedding),
                    };
                    embedded_chunks.push(code_chunk);
                }
                Err(e) => {
                    debug!("Failed to generate embedding for chunk: {}", e);
                    // Keep chunk without embedding for fallback text search
                    let code_chunk = CodeChunk {
                        content: chunk.content.clone(),
                        start_line: chunk.start_line,
                        end_line: chunk.end_line,
                        chunk_type: convert_chunk_type(&chunk.chunk_type),
                        embedding: None,
                    };
                    embedded_chunks.push(code_chunk);
                }
            }
        }
        
        let indexed_file = IndexedFile {
            path: file_path.to_path_buf(),
            content,
            chunks: embedded_chunks,
        };
        
        self.indexed_files.push(indexed_file);
        Ok(())
    }

    /// Semantic search for code matching a query using FAISS
    pub async fn search(&mut self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        info!("Performing semantic search for: '{}'", query);
        
        // Generate query embedding
        let query_embedding = match self.embedding_manager.generate_embeddings(query).await {
            Ok(embedding) => embedding,
            Err(e) => {
                warn!("Failed to generate query embedding: {}", e);
                return self.fallback_text_search(query, limit);
            }
        };
        
        // Build HNSW index if not already built
        if self.vector_search.get_stats().total_vectors > 0 && !self.vector_search.get_stats().index_built {
            info!("Building HNSW index...");
            self.vector_search.build_index()?;
        }
        
        // Search using instant-distance HNSW
        let vector_results = match self.vector_search.search(&query_embedding, limit) {
            Ok(results) => results,
            Err(e) => {
                warn!("HNSW search failed: {}", e);
                return self.fallback_text_search(query, limit);
            }
        };
        
        // Convert vector search results to semantic search results
        let mut results = Vec::new();
        for vector_result in vector_results {
            let context_lines = self.get_context_lines(&vector_result.content, vector_result.start_line, vector_result.end_line);
            
            let chunk = CodeChunk {
                content: vector_result.content,
                start_line: vector_result.start_line,
                end_line: vector_result.end_line,
                chunk_type: vector_result.chunk_type,
                embedding: None, // Not needed for results
            };
            
            results.push(SearchResult {
                file_path: vector_result.file_path,
                chunk,
                similarity_score: vector_result.similarity_score,
                context_lines,
            });
        }
        
        info!("Found {} semantic matches", results.len());
        Ok(results)
    }

    /// Fallback text search when embeddings fail
    fn fallback_text_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        warn!("Using fallback text search");
        
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        
        for file in &self.indexed_files {
            for chunk in &file.chunks {
                if chunk.content.to_lowercase().contains(&query_lower) {
                    let context_lines = self.get_context_lines(&file.content, chunk.start_line, chunk.end_line);
                    
                    results.push(SearchResult {
                        file_path: file.path.clone(),
                        chunk: chunk.clone(),
                        similarity_score: 0.5, // Default similarity for text match
                        context_lines,
                    });
                }
            }
        }
        
        results.truncate(limit);
        Ok(results)
    }

    /// Find code files in directory
    fn find_code_files<'a>(&'a self, dir_path: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<PathBuf>>> + 'a>> {
        Box::pin(async move {
            let mut files = Vec::new();
            let mut entries = fs::read_dir(dir_path).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                
                if path.is_dir() {
                    // Skip common non-code directories
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if matches!(name, "target" | "node_modules" | ".git" | "dist" | "build") {
                            continue;
                        }
                    }
                    
                    // Recursively search subdirectories
                    let sub_files = self.find_code_files(&path).await?;
                    files.extend(sub_files);
                } else if self.is_code_file(&path) {
                    files.push(path);
                }
            }
            
            Ok(files)
        })
    }

    /// Check if file is a code file we should index
    fn is_code_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            matches!(extension, 
                "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | 
                "cpp" | "c" | "h" | "hpp" | "cs" | "rb" | "php" | "swift" |
                "kt" | "scala" | "clj" | "hs" | "ml" | "fs" | "elm" | "ex" |
                "exs" | "cr" | "nim" | "zig" | "d" | "dart" | "r" | "jl"
            )
        } else {
            false
        }
    }

    /// Extract meaningful code chunks from file content
    fn extract_code_chunks(&self, content: &str) -> Result<Vec<CodeChunk>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();
        
        // Simple chunking strategy: split by functions and classes
        let mut current_chunk = String::new();
        let mut start_line = 0;
        let mut in_function = false;
        let mut brace_count = 0;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Detect function/class definitions
            if self.is_function_start(trimmed) || self.is_class_start(trimmed) {
                // Save previous chunk if it exists
                if !current_chunk.trim().is_empty() {
                    chunks.push(CodeChunk {
                        content: current_chunk.clone(),
                        start_line,
                        end_line: line_num,
                        chunk_type: VectorChunkType::Generic,
                        embedding: None,
                    });
                }
                
                // Start new chunk
                current_chunk = line.to_string() + "\n";
                start_line = line_num;
                in_function = true;
                brace_count = trimmed.matches('{').count() as i32 - trimmed.matches('}').count() as i32;
            } else if in_function {
                current_chunk.push_str(line);
                current_chunk.push('\n');
                
                // Track braces to find end of function
                brace_count += trimmed.matches('{').count() as i32 - trimmed.matches('}').count() as i32;
                
                if brace_count <= 0 {
                    // End of function
                    chunks.push(CodeChunk {
                        content: current_chunk.clone(),
                        start_line,
                        end_line: line_num + 1,
                        chunk_type: if self.is_function_start(&lines[start_line].trim()) {
                            VectorChunkType::Function
                        } else {
                            VectorChunkType::Class
                        },
                        embedding: None,
                    });
                    
                    current_chunk.clear();
                    in_function = false;
                }
            } else {
                // Build generic chunks for non-function code
                current_chunk.push_str(line);
                current_chunk.push('\n');
                
                // Create chunks every 10 lines for generic code
                if line_num - start_line >= 10 {
                    if !current_chunk.trim().is_empty() {
                        chunks.push(CodeChunk {
                            content: current_chunk.clone(),
                            start_line,
                            end_line: line_num + 1,
                            chunk_type: VectorChunkType::Generic,
                            embedding: None,
                        });
                    }
                    
                    current_chunk.clear();
                    start_line = line_num + 1;
                }
            }
        }
        
        // Save final chunk
        if !current_chunk.trim().is_empty() {
            chunks.push(CodeChunk {
                content: current_chunk,
                start_line,
                end_line: lines.len(),
                chunk_type: VectorChunkType::Generic,
                embedding: None,
            });
        }
        
        Ok(chunks)
    }

    /// Check if line starts a function
    fn is_function_start(&self, line: &str) -> bool {
        line.contains("fn ") || 
        line.contains("def ") || 
        line.contains("function ") ||
        line.contains("func ") ||
        (line.contains("(") && line.contains(")") && line.contains("{"))
    }

    /// Check if line starts a class
    fn is_class_start(&self, line: &str) -> bool {
        line.starts_with("class ") || 
        line.starts_with("struct ") ||
        line.starts_with("impl ") ||
        line.starts_with("trait ")
    }

    /// Get context lines around a chunk
    fn get_context_lines(&self, content: &str, start_line: usize, end_line: usize) -> Vec<String> {
        let lines: Vec<&str> = content.lines().collect();
        let context_start = start_line.saturating_sub(3);
        let context_end = (end_line + 2).min(lines.len());
        
        lines[context_start..context_end]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get statistics about indexed content
    pub fn get_stats(&self) -> IndexStats {
        let total_chunks: usize = self.indexed_files.iter()
            .map(|f| f.chunks.len())
            .sum();
            
        let embedded_chunks: usize = self.indexed_files.iter()
            .flat_map(|f| &f.chunks)
            .filter(|c| c.embedding.is_some())
            .count();

        IndexStats {
            total_files: self.indexed_files.len(),
            total_chunks,
            embedded_chunks,
            embedding_coverage: if total_chunks > 0 {
                embedded_chunks as f32 / total_chunks as f32
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug)]
pub struct IndexStats {
    pub total_files: usize,
    pub total_chunks: usize,
    pub embedded_chunks: usize,
    pub embedding_coverage: f32,
}

/// Convert code chunking types to vector search types
fn convert_chunk_type(chunk_type: &CodeChunkType) -> VectorChunkType {
    match chunk_type {
        CodeChunkType::Function => VectorChunkType::Function,
        CodeChunkType::Method => VectorChunkType::Function,
        CodeChunkType::Class => VectorChunkType::Class,
        CodeChunkType::Struct => VectorChunkType::Class,
        CodeChunkType::Interface => VectorChunkType::Class,
        CodeChunkType::Module => VectorChunkType::Module,
        CodeChunkType::Import => VectorChunkType::Module,
        CodeChunkType::Comment => VectorChunkType::Comment,
        CodeChunkType::Generic => VectorChunkType::Generic,
    }
}

impl Default for SemanticCodeSearch {
    fn default() -> Self {
        Self::new()
    }
}