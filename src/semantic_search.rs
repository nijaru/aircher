use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::fs;
use tracing::{debug, info, warn};
// use rayon::prelude::*; // Reserved for future parallel processing

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

/// Performance metrics for search operations
#[derive(Debug, Clone)]
pub struct SearchMetrics {
    pub total_duration: Duration,
    pub embedding_duration: Duration,
    pub vector_search_duration: Duration,
    pub result_processing_duration: Duration,
    pub total_results_found: usize,
    pub filtered_results_count: Option<usize>,
}

impl SearchMetrics {
    pub fn new() -> Self {
        Self {
            total_duration: Duration::from_secs(0),
            embedding_duration: Duration::from_secs(0),
            vector_search_duration: Duration::from_secs(0),
            result_processing_duration: Duration::from_secs(0),
            total_results_found: 0,
            filtered_results_count: None,
        }
    }
    
    pub fn format_summary(&self) -> String {
        if let Some(filtered_count) = self.filtered_results_count {
            format!("{:.2}s, filtered {}→{} results", 
                self.total_duration.as_secs_f64(), 
                self.total_results_found, 
                filtered_count)
        } else {
            format!("{:.2}s, {} results", 
                self.total_duration.as_secs_f64(), 
                self.total_results_found)
        }
    }
    
    pub fn format_detailed(&self) -> String {
        format!(
            "Total: {:.2}s (embedding: {:.2}s, search: {:.2}s, processing: {:.2}s)",
            self.total_duration.as_secs_f64(),
            self.embedding_duration.as_secs_f64(),
            self.vector_search_duration.as_secs_f64(),
            self.result_processing_duration.as_secs_f64()
        )
    }
}

impl SemanticCodeSearch {
    pub fn new() -> Self {
        let search = Self::with_config_sync(Self::default_config());
        // Try to load existing index in the background
        let _ = tokio::spawn(async move {
            // This will be handled by explicit load calls
        });
        search
    }

    /// Default configuration: bundled model only, no external dependencies
    fn default_config() -> EmbeddingConfig {
        EmbeddingConfig {
            preferred_model: "swerank-embed-small".to_string(), // Always use bundled model
            fallback_model: None, // No automatic fallbacks
            auto_download: false, // Never auto-download
            use_ollama_if_available: false, // Only use if explicitly configured
            max_model_size_mb: 1000,
        }
    }

    /// Create with explicit configuration (synchronous version)
    fn with_config_sync(config: EmbeddingConfig) -> Self {
        let embedding_manager = EmbeddingManager::new(config);
        
        // Create vector search engine with typical embedding dimension
        let cache_dir = crate::config::ArcherConfig::cache_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("search_index");
        
        let vector_search = VectorSearchEngine::new(cache_dir, 768) // SweRankEmbed dimension
            .unwrap_or_else(|e| {
                warn!("Failed to create vector search engine: {}", e);
                // Create a fallback with default settings
                VectorSearchEngine::new(PathBuf::from("./search_index"), 768).unwrap()
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
    
    /// Create with explicit configuration (async version that loads persisted data)
    pub async fn with_config(config: EmbeddingConfig) -> Result<Self> {
        let mut search = Self::with_config_sync(config);
        
        // Try to load existing index
        if let Err(e) = search.load_persisted_index().await {
            debug!("No existing index found or load failed: {}", e);
        }
        
        Ok(search)
    }
    
    /// Load persisted index if available
    pub async fn load_persisted_index(&mut self) -> Result<()> {
        // Load vector search metadata
        self.vector_search.load_index().await?;
        
        // Rebuild indexed_files from metadata
        let stats = self.vector_search.get_stats();
        if stats.total_vectors > 0 {
            info!("Loaded existing index with {} vectors", stats.total_vectors);
            // Note: We'll rebuild indexed_files on-demand during search operations
            // since we don't persist the full file content, only metadata
        }
        
        Ok(())
    }

    /// Ensure embedding model is available - now uses model selection approach
    pub async fn ensure_model_available(&mut self) -> Result<()> {
        // Check if user has already selected a model
        // For now, we'll use fallback mode and let the first-run selection handle model setup
        debug!("Model availability will be handled by first-run model selection");
        info!("Semantic search ready - model selection handled by user choice system");
        Ok(())
    }

    /// Index a directory of code files with timeout protection
    pub async fn index_directory(&mut self, dir_path: &Path) -> Result<()> {
        use std::time::{Duration, Instant};
        
        info!("Indexing directory: {:?}", dir_path);
        let start_time = Instant::now();
        
        let files = self.find_code_files(dir_path).await?;
        let total_files = files.len();
        
        if total_files == 0 {
            warn!("No code files found");
            return Ok(());
        }
        
        // Protect against huge directories - limit to 500 files max
        const MAX_FILES: usize = 500;
        let files_to_process = if total_files > MAX_FILES {
            warn!("Large directory detected ({} files). Processing first {} files only", total_files, MAX_FILES);
            files.into_iter().take(MAX_FILES).collect::<Vec<_>>()
        } else {
            files
        };
        
        let processing_count = files_to_process.len();
        let mut indexed_count = 0;
        let mut failed_count = 0;
        let mut consecutive_failures = 0;
        
        // Overall timeout: 2 minutes max for directory indexing
        const OVERALL_TIMEOUT: Duration = Duration::from_secs(120);
        
        for (i, file_path) in files_to_process.iter().enumerate() {
            // Check overall timeout
            if start_time.elapsed() > OVERALL_TIMEOUT {
                warn!("Directory indexing timeout after {:?}. Processed {}/{} files", 
                      start_time.elapsed(), i, processing_count);
                break;
            }
            
            match self.index_file(file_path).await {
                Ok(_) => {
                    indexed_count += 1;
                    consecutive_failures = 0; // Reset failure counter
                    if indexed_count % 25 == 0 {
                        let elapsed = start_time.elapsed();
                        let rate = indexed_count as f64 / elapsed.as_secs_f64();
                        info!("Progress: {}/{} files ({:.1} files/sec)", indexed_count, processing_count, rate);
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    consecutive_failures += 1;
                    debug!("Skipped {:?}: {}", file_path, e);
                    
                    // Early termination on too many consecutive failures (likely system issue)
                    if consecutive_failures >= 10 {
                        warn!("Too many consecutive failures ({}). Stopping indexing to prevent timeouts", consecutive_failures);
                        break;
                    }
                }
            }
            
            // Yield CPU every 10 files to reduce system load and improve responsiveness
            if (i + 1) % 10 == 0 {
                tokio::task::yield_now().await;
            }
        }
        
        let elapsed = start_time.elapsed();
        if failed_count > 0 {
            info!("Indexed {}/{} files in {:?} ({} skipped)", indexed_count, processing_count, elapsed, failed_count);
        } else {
            info!("Indexed {} files in {:?}", indexed_count, elapsed);
        }
        
        // Save index after directory indexing is complete
        if let Err(e) = self.vector_search.save_index().await {
            warn!("Failed to save index after directory indexing: {}", e);
        }
        
        Ok(())
    }

    /// Index a single file using improved chunking and vector search
    pub async fn index_file(&mut self, file_path: &Path) -> Result<()> {
        let content = fs::read_to_string(file_path).await
            .context(format!("Failed to read file: {:?}", file_path))?;
        
        // Skip empty or very small files
        if content.trim().len() < 50 {
            return Ok(());
        }
        
        // Use tree-sitter based chunking for better semantic boundaries
        let chunks = self.code_chunker.chunk_file(file_path, &content)
            .context(format!("Failed to chunk file: {:?}", file_path))?;
        
        if chunks.is_empty() {
            return Ok(()); // No meaningful content to index
        }
        
        // Process chunks and generate embeddings in batches for better performance
        let mut embedded_chunks = Vec::with_capacity(chunks.len());
        
        // Batch processing: collect chunk texts for batch embedding generation
        let chunk_texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        
        // Generate embeddings in batch (much more efficient)
        match self.embedding_manager.generate_batch_embeddings(&chunk_texts).await {
            Ok(embeddings_batch) => {
                // Process successful batch embeddings
                for (chunk, embedding) in chunks.into_iter().zip(embeddings_batch.into_iter()) {
                    let metadata = ChunkMetadata {
                        file_path: file_path.to_path_buf(),
                        start_line: chunk.start_line,
                        end_line: chunk.end_line,
                        chunk_type: convert_chunk_type(&chunk.chunk_type),
                        content: chunk.content.clone(),
                    };
                    
                    // Add to vector search index
                    self.vector_search.add_embedding(embedding.clone(), metadata)?;
                    
                    embedded_chunks.push(CodeChunk {
                        content: chunk.content,
                        start_line: chunk.start_line,
                        end_line: chunk.end_line,
                        chunk_type: convert_chunk_type(&chunk.chunk_type),
                        embedding: Some(embedding),
                    });
                }
            }
            Err(_) => {
                // Fall back to individual processing if batch fails
                for chunk in chunks {
                    let code_chunk = match self.embedding_manager.generate_embeddings(&chunk.content).await {
                        Ok(embedding) => {
                            let metadata = ChunkMetadata {
                                file_path: file_path.to_path_buf(),
                                start_line: chunk.start_line,
                                end_line: chunk.end_line,
                                chunk_type: convert_chunk_type(&chunk.chunk_type),
                                content: chunk.content.clone(),
                            };
                            
                            self.vector_search.add_embedding(embedding.clone(), metadata)?;
                            
                            CodeChunk {
                                content: chunk.content,
                                start_line: chunk.start_line,
                                end_line: chunk.end_line,
                                chunk_type: convert_chunk_type(&chunk.chunk_type),
                                embedding: Some(embedding),
                            }
                        }
                        Err(_) => {
                            CodeChunk {
                                content: chunk.content,
                                start_line: chunk.start_line,
                                end_line: chunk.end_line,
                                chunk_type: convert_chunk_type(&chunk.chunk_type),
                                embedding: None,
                            }
                        }
                    };
                    embedded_chunks.push(code_chunk);
                }
            }
        }
        
        self.indexed_files.push(IndexedFile {
            path: file_path.to_path_buf(),
            content,
            chunks: embedded_chunks,
        });
        
        Ok(())
    }

    /// Semantic search for code matching a query
    pub async fn search(&mut self, query: &str, limit: usize) -> Result<(Vec<SearchResult>, SearchMetrics)> {
        let total_start = Instant::now();
        let mut metrics = SearchMetrics::new();
        
        info!("Searching for: '{}'", query);
        
        // Generate query embedding
        info!("Starting embedding generation...");
        let embedding_start = Instant::now();
        let query_embedding = match self.embedding_manager.generate_embeddings(query).await {
            Ok(embedding) => {
                info!("Embedding generated in {:?}", embedding_start.elapsed());
                embedding
            },
            Err(e) => {
                warn!("Embedding failed, using text search: {}", e);
                let (results, mut fallback_metrics) = self.fallback_text_search_with_metrics(query, limit)?;
                fallback_metrics.total_duration = total_start.elapsed();
                return Ok((results, fallback_metrics));
            }
        };
        metrics.embedding_duration = embedding_start.elapsed();
        
        // Build HNSW index if needed
        let stats = self.vector_search.get_stats();
        if stats.total_vectors > 0 && !stats.index_built {
            info!("Building search index");
            self.vector_search.build_index()?;
            // Save index after building
            if let Err(e) = self.vector_search.save_index().await {
                warn!("Failed to save index: {}", e);
            }
        }
        
        // Search using vector similarity
        let search_start = Instant::now();
        let vector_results = match self.vector_search.search(&query_embedding, limit) {
            Ok(results) => results,
            Err(e) => {
                warn!("Vector search failed: {}", e);
                let (results, mut fallback_metrics) = self.fallback_text_search_with_metrics(query, limit)?;
                fallback_metrics.total_duration = total_start.elapsed();
                return Ok((results, fallback_metrics));
            }
        };
        metrics.vector_search_duration = search_start.elapsed();
        
        // Convert vector search results to semantic search results
        let processing_start = Instant::now();
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
        metrics.result_processing_duration = processing_start.elapsed();
        
        // Finalize metrics
        metrics.total_duration = total_start.elapsed();
        metrics.total_results_found = results.len();
        
        info!("Found {} matches in {:.2}s", results.len(), metrics.total_duration.as_secs_f64());
        Ok((results, metrics))
    }

    /// Optimized semantic search with query-time filtering
    pub async fn search_with_filters(
        &mut self, 
        query: &str, 
        limit: usize,
        file_types: &Option<Vec<String>>,
        chunk_types: &Option<Vec<String>>,
        exclude_patterns: &Option<Vec<String>>,
        include_patterns: &Option<Vec<String>>,
    ) -> Result<(Vec<SearchResult>, SearchMetrics)> {
        let total_start = Instant::now();
        let mut metrics = SearchMetrics::new();
        
        info!("Searching with filters for: '{}'", query);
        
        // Generate query embedding
        let embedding_start = Instant::now();
        let query_embedding = match self.embedding_manager.generate_embeddings(query).await {
            Ok(embedding) => embedding,
            Err(e) => {
                warn!("Embedding failed, using text search: {}", e);
                let (results, mut fallback_metrics) = self.fallback_text_search_with_metrics(query, limit)?;
                fallback_metrics.total_duration = total_start.elapsed();
                return Ok((results, fallback_metrics));
            }
        };
        metrics.embedding_duration = embedding_start.elapsed();
        
        // Build HNSW index if needed
        let stats = self.vector_search.get_stats();
        if stats.total_vectors > 0 && !stats.index_built {
            info!("Building search index");
            self.vector_search.build_index()?;
            // Save index after building
            if let Err(e) = self.vector_search.save_index().await {
                warn!("Failed to save index: {}", e);
            }
        }
        
        // Create vector search filter
        let mut vector_filter = crate::vector_search::SearchFilter::default();
        vector_filter.file_types = file_types.clone();
        vector_filter.exclude_patterns = exclude_patterns.clone();
        vector_filter.include_patterns = include_patterns.clone();
        
        // Convert chunk types from CLI format to vector search format
        if let Some(ref chunk_type_strings) = chunk_types {
            let vector_chunk_types: Vec<crate::vector_search::ChunkType> = chunk_type_strings.iter()
                .filter_map(|s| match s.to_lowercase().as_str() {
                    "function" => Some(crate::vector_search::ChunkType::Function),
                    "class" => Some(crate::vector_search::ChunkType::Class),
                    "module" => Some(crate::vector_search::ChunkType::Module),
                    "comment" => Some(crate::vector_search::ChunkType::Comment),
                    "generic" => Some(crate::vector_search::ChunkType::Generic),
                    _ => None,
                })
                .collect();
            vector_filter.chunk_types = Some(vector_chunk_types);
        }
        
        // Search using optimized vector similarity with filtering
        let search_start = Instant::now();
        let vector_results = match self.vector_search.search_with_filter(&query_embedding, limit, &vector_filter) {
            Ok(results) => results,
            Err(e) => {
                warn!("Vector search failed: {}", e);
                let (results, mut fallback_metrics) = self.fallback_text_search_with_metrics(query, limit)?;
                fallback_metrics.total_duration = total_start.elapsed();
                return Ok((results, fallback_metrics));
            }
        };
        metrics.vector_search_duration = search_start.elapsed();
        
        // Convert vector search results to semantic search results
        let processing_start = Instant::now();
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
        metrics.result_processing_duration = processing_start.elapsed();
        
        // Finalize metrics
        metrics.total_duration = total_start.elapsed();
        metrics.total_results_found = results.len();
        
        info!("Found {} filtered matches in {:.2}s", results.len(), metrics.total_duration.as_secs_f64());
        Ok((results, metrics))
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

    /// Fallback text search with metrics when embeddings fail
    fn fallback_text_search_with_metrics(&self, query: &str, limit: usize) -> Result<(Vec<SearchResult>, SearchMetrics)> {
        let start = Instant::now();
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
        
        let mut metrics = SearchMetrics::new();
        metrics.total_duration = start.elapsed();
        metrics.result_processing_duration = start.elapsed(); // All time spent in processing for text search
        metrics.total_results_found = results.len();
        
        Ok((results, metrics))
    }

    /// Find code files in directory with depth and size limits
    fn find_code_files<'a>(&'a self, dir_path: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<PathBuf>>> + 'a>> {
        Box::pin(async move {
            self.find_code_files_recursive(dir_path, 0, &mut 0).await
        })
    }
    
    /// Recursive helper with depth and count limits
    fn find_code_files_recursive<'a>(&'a self, dir_path: &'a Path, depth: usize, file_count: &'a mut usize) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<PathBuf>>> + 'a>> {
        Box::pin(async move {
            let mut files = Vec::new();
            
            // Depth limit to prevent infinite recursion in complex directory structures
            const MAX_DEPTH: usize = 8;
            const MAX_FILES_SCAN: usize = 2000; // Stop scanning if we find too many files
            
            if depth > MAX_DEPTH {
                debug!("Skipping directory due to depth limit: {:?}", dir_path);
                return Ok(files);
            }
            
            if *file_count > MAX_FILES_SCAN {
                debug!("Stopping scan - found {} files (limit reached)", *file_count);
                return Ok(files);
            }
            
            let mut entries = match fs::read_dir(dir_path).await {
                Ok(entries) => entries,
                Err(e) => {
                    debug!("Cannot read directory {:?}: {}", dir_path, e);
                    return Ok(files);
                }
            };
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                
                if path.is_dir() {
                    // Skip common non-code directories and problematic paths
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if matches!(name, 
                            "target" | "node_modules" | ".git" | "dist" | "build" | "external" |
                            "vendor" | "deps" | "packages" | ".cargo" | ".npm" | "__pycache__" |
                            ".pytest_cache" | "venv" | "env" | ".venv" | ".env" | "tmp" | "temp" |
                            ".next" | ".nuxt" | "coverage" | ".coverage" | "test-results" |
                            ".idea" | ".vscode" | "logs" | "log" | ".cache"
                        ) {
                            debug!("Skipping directory: {:?}", path);
                            continue;
                        }
                        
                        // Skip hidden directories that start with . (except .config, .src, etc.)
                        if name.starts_with('.') && !matches!(name, ".config" | ".src" | ".github") {
                            debug!("Skipping hidden directory: {:?}", path);
                            continue;
                        }
                    }
                    
                    // Recursively search subdirectories
                    let sub_files = self.find_code_files_recursive(&path, depth + 1, file_count).await?;
                    files.extend(sub_files);
                } else if self.is_code_file(&path) {
                    files.push(path);
                    *file_count += 1;
                    
                    // Early exit if we're finding too many files
                    if *file_count > MAX_FILES_SCAN {
                        break;
                    }
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
        
        // Ensure start_line and end_line are valid and in the right order
        let actual_start = start_line.min(end_line);
        let actual_end = start_line.max(end_line);
        
        let context_start = actual_start.saturating_sub(3);
        let context_end = (actual_end + 2).min(lines.len());
        
        // Ensure context_start <= context_end
        let safe_start = context_start.min(context_end);
        let safe_end = context_end.max(safe_start);
        
        lines[safe_start..safe_end]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Remove a file from the index
    pub fn remove_file(&mut self, file_path: &Path) -> Result<bool> {
        let path_buf = file_path.to_path_buf();
        
        // Find the file in our index
        let file_index = self.indexed_files.iter().position(|f| f.path == path_buf);
        
        if let Some(index) = file_index {
            let _removed_file = self.indexed_files.remove(index);
            
            // Remove embeddings from vector search
            // Note: Current instant-distance implementation doesn't support removal
            // For now we'll rebuild the index when needed
            info!("Removed file from index: {:?}", file_path);
            Ok(true)
        } else {
            debug!("File not found in index: {:?}", file_path);
            Ok(false)
        }
    }
    
    /// Update a single file in the index (re-index if exists, add if new)
    pub async fn update_file(&mut self, file_path: &Path) -> Result<()> {
        // Remove existing file from index if present
        self.remove_file(file_path)?;
        
        // Re-index the file
        self.index_file(file_path).await?;
        
        info!("Updated file in index: {:?}", file_path);
        Ok(())
    }
    
    /// Check if a file is already indexed
    pub fn is_file_indexed(&self, file_path: &Path) -> bool {
        let path_buf = file_path.to_path_buf();
        self.indexed_files.iter().any(|f| f.path == path_buf)
    }
    
    /// Rebuild the vector search index (needed after file removals)
    pub fn rebuild_vector_index(&mut self) -> Result<()> {
        // Clear the existing vector index
        let cache_dir = crate::config::ArcherConfig::cache_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("search_index");
        
        // Create new vector search engine
        self.vector_search = VectorSearchEngine::new(cache_dir, 768)?;
        
        // Re-add all embeddings
        let mut total_added = 0;
        for file in &self.indexed_files {
            for chunk in &file.chunks {
                if let Some(embedding) = &chunk.embedding {
                    let metadata = ChunkMetadata {
                        file_path: file.path.clone(),
                        start_line: chunk.start_line,
                        end_line: chunk.end_line,
                        chunk_type: chunk.chunk_type.clone(),
                        content: chunk.content.clone(),
                    };
                    
                    self.vector_search.add_embedding(embedding.clone(), metadata)?;
                    total_added += 1;
                }
            }
        }
        
        // Build the index
        if total_added > 0 {
            self.vector_search.build_index()?;
            info!("Rebuilt vector index with {} embeddings", total_added);
        }
        
        Ok(())
    }

    /// Get statistics about indexed content
    pub fn get_stats(&self) -> IndexStats {
        // If we have in-memory indexed files, use those stats
        if !self.indexed_files.is_empty() {
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
        } else {
            // Check if we have persisted data in vector search
            let vector_stats = self.vector_search.get_stats();
            IndexStats {
                total_files: if vector_stats.total_vectors > 0 { 1 } else { 0 }, // Approximation
                total_chunks: vector_stats.total_vectors,
                embedded_chunks: vector_stats.total_vectors, // All vectors are embedded
                embedding_coverage: if vector_stats.total_vectors > 0 { 1.0 } else { 0.0 },
            }
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