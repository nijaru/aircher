use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tracing::{debug, info};

use crate::semantic_search::{SemanticCodeSearch};

#[derive(Debug, Args)]
pub struct SearchArgs {
    #[command(subcommand)]
    pub command: SearchCommand,
}

#[derive(Debug, Subcommand)]
pub enum SearchCommand {
    /// Index a directory for semantic search
    Index {
        /// Directory path to index
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Force re-indexing
        #[arg(long)]
        force: bool,
    },
    /// Perform semantic code search with advanced filtering
    Query {
        /// Search query (e.g., "error handling patterns", "database connection")
        query: String,
        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Directory to search in
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        /// Filter by file types (e.g., "rs,py,js" or "rust,python,javascript")
        #[arg(long, value_delimiter = ',')]
        file_types: Option<Vec<String>>,
        /// Filter by programming languages (e.g., "rust,python")
        #[arg(long, value_delimiter = ',')]
        languages: Option<Vec<String>>,
        /// Filter by code scope (e.g., "functions,classes,modules")
        #[arg(long, value_delimiter = ',')]
        scope: Option<Vec<String>>,
        /// Filter by chunk types (e.g., "function,class,module,comment")
        #[arg(long, value_delimiter = ',')]
        chunk_types: Option<Vec<String>>,
        /// Minimum similarity threshold (0.0-1.0)
        #[arg(long)]
        min_similarity: Option<f32>,
        /// Maximum similarity threshold (0.0-1.0)
        #[arg(long)]
        max_similarity: Option<f32>,
        /// Exclude patterns (e.g., "test,bench,example")
        #[arg(long, value_delimiter = ',')]
        exclude: Option<Vec<String>>,
        /// Include only patterns (e.g., "src,lib")
        #[arg(long, value_delimiter = ',')]
        include: Option<Vec<String>>,
        /// Show debug information about filtering
        #[arg(long)]
        debug_filters: bool,
    },
    /// Show search index statistics
    Stats {
        /// Directory path
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

pub async fn handle_search_command(args: SearchArgs) -> Result<()> {
    match args.command {
        SearchCommand::Index { path, force: _ } => {
            println!("🧠 Building semantic search index...");
            
            let mut search = SemanticCodeSearch::new();
            
            // Ensure embedding model is available before indexing
            search.ensure_model_available().await?;
            
            match search.index_directory(&path).await {
                Ok(_) => {
                    let stats = search.get_stats();
                    println!("✅ Indexing complete!");
                    println!("   Files indexed: {}", stats.total_files);
                    println!("   Code chunks: {}", stats.total_chunks);
                    println!("   Embedded chunks: {}", stats.embedded_chunks);
                    println!("   Coverage: {:.1}%", stats.embedding_coverage * 100.0);
                    
                    if stats.embedding_coverage < 0.8 {
                        println!("⚠️  Low embedding coverage - ensure Ollama is running with nomic-embed-text");
                        println!("   Run: ollama pull nomic-embed-text");
                    }
                }
                Err(e) => {
                    println!("❌ Indexing failed: {}", e);
                    println!("💡 Try: aircher embedding setup --interactive");
                }
            }
        }
        
        SearchCommand::Query { 
            query, 
            limit, 
            path, 
            file_types,
            languages,
            scope,
            chunk_types,
            min_similarity,
            max_similarity,
            exclude,
            include,
            debug_filters
        } => {
            println!("🔍 Searching for: '{}'", query);
            
            if debug_filters {
                println!("🐛 Debug: Active filters:");
                if let Some(ref types) = file_types {
                    println!("   File types: {}", types.join(", "));
                }
                if let Some(ref langs) = languages {
                    println!("   Languages: {}", langs.join(", "));
                }
                if let Some(ref scopes) = scope {
                    println!("   Scope: {}", scopes.join(", "));
                }
                if let Some(ref chunks) = chunk_types {
                    println!("   Chunk types: {}", chunks.join(", "));
                }
                if let Some(min_sim) = min_similarity {
                    println!("   Min similarity: {:.2}", min_sim);
                }
                if let Some(max_sim) = max_similarity {
                    println!("   Max similarity: {:.2}", max_sim);
                }
                if let Some(ref excl) = exclude {
                    println!("   Exclude: {}", excl.join(", "));
                }
                if let Some(ref incl) = include {
                    println!("   Include: {}", incl.join(", "));
                }
                println!();
            }
            
            let mut search = SemanticCodeSearch::new();
            
            // Ensure embedding model is available before search
            search.ensure_model_available().await?;
            
            // Try to load existing index
            if let Err(e) = search.load_persisted_index().await {
                debug!("No existing index found: {}", e);
            }
            
            // Check if we have any indexed content first
            let stats = search.get_stats();
            if stats.total_files == 0 {
                println!("⚠️  No indexed files found. Building index first...");
                println!("💡 Tip: Run 'aircher search index' once to build a persistent index");
                info!("Indexing directory for search: {:?}", path);
                search.index_directory(&path).await?;
            } else {
                info!("Using existing index with {} files", stats.total_files);
            }
            
            match search.search(&query, limit * 3).await { // Get more results to filter
                Ok(mut results) => {
                    let original_count = results.len();
                    
                    // Apply advanced filters
                    results = apply_search_filters(
                        results,
                        &file_types,
                        &languages,
                        &scope,
                        &chunk_types,
                        min_similarity,
                        max_similarity,
                        &exclude,
                        &include,
                        debug_filters
                    );
                    
                    // Limit results after filtering
                    results.truncate(limit);
                    
                    if debug_filters && original_count != results.len() {
                        println!("🐛 Filtered {} → {} results", original_count, results.len());
                    }
                    
                    if results.is_empty() {
                        println!("No results found for '{}'", query);
                        if original_count > 0 {
                            println!("💡 {} results were filtered out - try adjusting filters", original_count);
                        } else {
                            println!("💡 Try broader terms or check if directory is indexed");
                        }
                    } else {
                        println!("Found {} results:\n", results.len());
                        
                        for (i, result) in results.iter().enumerate() {
                            println!("{}. {} (similarity: {:.2})", 
                                   i + 1, 
                                   result.file_path.display(), 
                                   result.similarity_score);
                            
                            println!("   Lines {}-{}", result.chunk.start_line, result.chunk.end_line);
                            
                            // Show code snippet
                            let preview = result.chunk.content
                                .lines()
                                .take(3)
                                .collect::<Vec<_>>()
                                .join("\n");
                            println!("   ```");
                            println!("   {}", preview);
                            if result.chunk.content.lines().count() > 3 {
                                println!("   ...");
                            }
                            println!("   ```\n");
                        }
                        
                        println!("💡 Semantic search found contextually similar code");
                        println!("   This goes beyond text matching to understand meaning");
                    }
                }
                Err(e) => {
                    println!("❌ Search failed: {}", e);
                    println!("💡 Ensure embedding models are available: aircher embedding status");
                }
            }
        }
        
        SearchCommand::Stats { path } => {
            println!("📊 Search Index Statistics for {:?}", path);
            
            let mut search = SemanticCodeSearch::new();
            
            // Ensure embedding model is available before generating stats
            search.ensure_model_available().await?;
            
            search.index_directory(&path).await?;
            
            let stats = search.get_stats();
            
            println!("Files indexed: {}", stats.total_files);
            println!("Code chunks: {}", stats.total_chunks);
            println!("Embedded chunks: {}", stats.embedded_chunks);
            println!("Embedding coverage: {:.1}%", stats.embedding_coverage * 100.0);
            
            if stats.total_files == 0 {
                println!("⚠️  No code files found in directory");
            } else if stats.embedding_coverage < 0.5 {
                println!("⚠️  Low embedding coverage - semantic search will be limited");
                println!("   Check: aircher embedding status");
            } else {
                println!("✅ Good semantic search coverage");
            }
        }
    }
    
    Ok(())
}

/// Apply advanced search filters to search results
fn apply_search_filters(
    mut results: Vec<crate::semantic_search::SearchResult>,
    file_types: &Option<Vec<String>>,
    languages: &Option<Vec<String>>,
    scope: &Option<Vec<String>>,
    chunk_types: &Option<Vec<String>>,
    min_similarity: Option<f32>,
    max_similarity: Option<f32>,
    exclude: &Option<Vec<String>>,
    include: &Option<Vec<String>>,
    debug_filters: bool,
) -> Vec<crate::semantic_search::SearchResult> {
    let original_count = results.len();
    
    // Filter by similarity thresholds
    if let Some(min_sim) = min_similarity {
        results.retain(|r| r.similarity_score >= min_sim);
        if debug_filters {
            debug!("After min similarity filter: {} results", results.len());
        }
    }
    
    if let Some(max_sim) = max_similarity {
        results.retain(|r| r.similarity_score <= max_sim);
        if debug_filters {
            debug!("After max similarity filter: {} results", results.len());
        }
    }
    
    // Filter by file types/extensions
    if let Some(ref types) = file_types {
        let normalized_types: Vec<String> = types.iter()
            .map(|t| normalize_file_type(t))
            .collect();
        
        results.retain(|r| {
            if let Some(ext) = r.file_path.extension().and_then(|e| e.to_str()) {
                normalized_types.contains(&ext.to_lowercase()) ||
                normalized_types.contains(&language_from_extension(ext))
            } else {
                false
            }
        });
        
        if debug_filters {
            debug!("After file type filter: {} results", results.len());
        }
    }
    
    // Filter by languages
    if let Some(ref langs) = languages {
        let normalized_langs: Vec<String> = langs.iter()
            .map(|l| l.to_lowercase())
            .collect();
        
        results.retain(|r| {
            if let Some(ext) = r.file_path.extension().and_then(|e| e.to_str()) {
                let lang = language_from_extension(ext);
                normalized_langs.contains(&lang)
            } else {
                false
            }
        });
        
        if debug_filters {
            debug!("After language filter: {} results", results.len());
        }
    }
    
    // Filter by chunk types
    if let Some(ref chunks) = chunk_types {
        let normalized_chunks: Vec<String> = chunks.iter()
            .map(|c| c.to_lowercase())
            .collect();
        
        results.retain(|r| {
            let chunk_type_str = match r.chunk.chunk_type {
                crate::vector_search::ChunkType::Function => "function",
                crate::vector_search::ChunkType::Class => "class",
                crate::vector_search::ChunkType::Module => "module",
                crate::vector_search::ChunkType::Comment => "comment",
                crate::vector_search::ChunkType::Generic => "generic",
            }.to_string();
            
            normalized_chunks.contains(&chunk_type_str)
        });
        
        if debug_filters {
            debug!("After chunk type filter: {} results", results.len());
        }
    }
    
    // Filter by scope (functions, classes, modules, etc.)
    if let Some(ref scopes) = scope {
        let normalized_scopes: Vec<String> = scopes.iter()
            .map(|s| s.to_lowercase())
            .collect();
        
        results.retain(|r| {
            let chunk_type_str = match r.chunk.chunk_type {
                crate::vector_search::ChunkType::Function => "function",
                crate::vector_search::ChunkType::Class => "class",
                crate::vector_search::ChunkType::Module => "module",
                crate::vector_search::ChunkType::Comment => "comment",
                crate::vector_search::ChunkType::Generic => "generic",
            }.to_string();
            
            // Check if scope matches chunk type or if "functions" matches "function"
            normalized_scopes.contains(&chunk_type_str) ||
            (chunk_type_str == "function" && normalized_scopes.contains(&"functions".to_string())) ||
            (chunk_type_str == "class" && normalized_scopes.contains(&"classes".to_string())) ||
            (chunk_type_str == "module" && normalized_scopes.contains(&"modules".to_string()))
        });
        
        if debug_filters {
            debug!("After scope filter: {} results", results.len());
        }
    }
    
    // Apply exclude patterns
    if let Some(ref excl_patterns) = exclude {
        results.retain(|r| {
            let path_str = r.file_path.to_string_lossy().to_lowercase();
            !excl_patterns.iter().any(|pattern| {
                let pattern_lower = pattern.to_lowercase();
                path_str.contains(&pattern_lower) ||
                r.chunk.content.to_lowercase().contains(&pattern_lower)
            })
        });
        
        if debug_filters {
            debug!("After exclude filter: {} results", results.len());
        }
    }
    
    // Apply include patterns
    if let Some(ref incl_patterns) = include {
        results.retain(|r| {
            let path_str = r.file_path.to_string_lossy().to_lowercase();
            incl_patterns.iter().any(|pattern| {
                let pattern_lower = pattern.to_lowercase();
                path_str.contains(&pattern_lower)
            })
        });
        
        if debug_filters {
            debug!("After include filter: {} results", results.len());
        }
    }
    
    if debug_filters && results.len() != original_count {
        info!("🔍 Filtered search results: {} → {}", original_count, results.len());
    }
    
    results
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

    #[tokio::test]
    async fn test_search_stats() {
        let args = SearchArgs {
            command: SearchCommand::Stats {
                path: PathBuf::from("."),
            },
        };
        
        // Should not panic
        assert!(handle_search_command(args).await.is_ok());
    }
}