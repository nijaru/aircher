use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tracing::info;

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
    /// Perform semantic code search
    Query {
        /// Search query (e.g., "error handling patterns", "database connection")
        query: String,
        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Directory to search in
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
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
        
        SearchCommand::Query { query, limit, path } => {
            println!("🔍 Searching for: '{}'", query);
            
            let mut search = SemanticCodeSearch::new();
            
            // Quick index if not already done
            info!("Indexing directory for search: {:?}", path);
            search.index_directory(&path).await?;
            
            match search.search(&query, limit).await {
                Ok(results) => {
                    if results.is_empty() {
                        println!("No results found for '{}'", query);
                        println!("💡 Try broader terms or check if directory is indexed");
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