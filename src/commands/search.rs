use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tracing::{debug, info};

use crate::semantic_search::{SemanticCodeSearch};
use crate::search_presets::{PresetManager, SearchPreset, SearchFilters};

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
        /// Use a saved search preset
        #[arg(long)]
        preset: Option<String>,
        /// Save current search as a preset
        #[arg(long)]
        save_preset: Option<String>,
    },
    /// Show search index statistics
    Stats {
        /// Directory path
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Manage search presets
    Preset {
        #[command(subcommand)]
        preset_command: PresetCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum PresetCommand {
    /// List all available presets
    List {
        /// Show detailed preset information
        #[arg(long)]
        verbose: bool,
    },
    /// Show details of a specific preset
    Show {
        /// Preset name
        name: String,
    },
    /// Save current search filters as a preset
    Save {
        /// Preset name
        name: String,
        /// Preset description
        #[arg(short, long)]
        description: Option<String>,
        /// Save as global preset (default: project-local)
        #[arg(long)]
        global: bool,
        /// File types filter
        #[arg(long, value_delimiter = ',')]
        file_types: Option<Vec<String>>,
        /// Languages filter
        #[arg(long, value_delimiter = ',')]
        languages: Option<Vec<String>>,
        /// Scope filter
        #[arg(long, value_delimiter = ',')]
        scope: Option<Vec<String>>,
        /// Chunk types filter
        #[arg(long, value_delimiter = ',')]
        chunk_types: Option<Vec<String>>,
        /// Minimum similarity threshold
        #[arg(long)]
        min_similarity: Option<f32>,
        /// Maximum similarity threshold
        #[arg(long)]
        max_similarity: Option<f32>,
        /// Exclude patterns
        #[arg(long, value_delimiter = ',')]
        exclude: Option<Vec<String>>,
        /// Include patterns
        #[arg(long, value_delimiter = ',')]
        include: Option<Vec<String>>,
        /// Default limit
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Delete a preset
    Delete {
        /// Preset name
        name: String,
        /// Delete global preset (default: project-local)
        #[arg(long)]
        global: bool,
    },
    /// Create built-in presets
    Init {
        /// Force overwrite existing presets
        #[arg(long)]
        force: bool,
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
            debug_filters,
            preset,
            save_preset
        } => {
            // Initialize preset manager
            let mut preset_manager = PresetManager::new()?;
            
            // Handle preset loading and filter merging
            let mut effective_file_types = file_types;
            let mut effective_languages = languages;
            let mut effective_scope = scope;
            let mut effective_chunk_types = chunk_types;
            let mut effective_min_similarity = min_similarity;
            let mut effective_max_similarity = max_similarity;
            let mut effective_exclude = exclude;
            let mut effective_include = include;
            let mut effective_limit = limit;
            
            if let Some(preset_name) = &preset {
                match preset_manager.get_preset(preset_name).await? {
                    Some(preset_data) => {
                        println!("📋 Using preset: {} - {}", preset_data.name, preset_data.description);
                        
                        // Merge preset filters with command-line filters (CLI takes precedence)
                        if effective_file_types.is_none() {
                            effective_file_types = preset_data.filters.file_types;
                        }
                        if effective_languages.is_none() {
                            effective_languages = preset_data.filters.languages;
                        }
                        if effective_scope.is_none() {
                            effective_scope = preset_data.filters.scope;
                        }
                        if effective_chunk_types.is_none() {
                            effective_chunk_types = preset_data.filters.chunk_types;
                        }
                        if effective_min_similarity.is_none() {
                            effective_min_similarity = preset_data.filters.min_similarity;
                        }
                        if effective_max_similarity.is_none() {
                            effective_max_similarity = preset_data.filters.max_similarity;
                        }
                        if effective_exclude.is_none() {
                            effective_exclude = preset_data.filters.exclude;
                        }
                        if effective_include.is_none() {
                            effective_include = preset_data.filters.include;
                        }
                        if let Some(preset_limit) = preset_data.filters.limit {
                            if effective_limit == 10 { // Default limit
                                effective_limit = preset_limit;
                            }
                        }
                        
                        // Increment usage count
                        let _ = preset_manager.increment_usage(preset_name).await;
                    }
                    None => {
                        println!("⚠️  Preset '{}' not found", preset_name);
                        println!("💡 List available presets: aircher search preset list");
                        return Ok(());
                    }
                }
            }
            
            println!("🔍 Searching for: '{}'", query);
            
            if debug_filters {
                println!("🐛 Debug: Active filters:");
                if let Some(ref types) = effective_file_types {
                    println!("   File types: {}", types.join(", "));
                }
                if let Some(ref langs) = effective_languages {
                    println!("   Languages: {}", langs.join(", "));
                }
                if let Some(ref scopes) = effective_scope {
                    println!("   Scope: {}", scopes.join(", "));
                }
                if let Some(ref chunks) = effective_chunk_types {
                    println!("   Chunk types: {}", chunks.join(", "));
                }
                if let Some(min_sim) = effective_min_similarity {
                    println!("   Min similarity: {:.2}", min_sim);
                }
                if let Some(max_sim) = effective_max_similarity {
                    println!("   Max similarity: {:.2}", max_sim);
                }
                if let Some(ref excl) = effective_exclude {
                    println!("   Exclude: {}", excl.join(", "));
                }
                if let Some(ref incl) = effective_include {
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
            
            match search.search(&query, effective_limit * 3).await { // Get more results to filter
                Ok((mut results, mut metrics)) => {
                    let original_count = results.len();
                    
                    // Apply advanced filters
                    results = apply_search_filters(
                        results,
                        &effective_file_types,
                        &effective_languages,
                        &effective_scope,
                        &effective_chunk_types,
                        effective_min_similarity,
                        effective_max_similarity,
                        &effective_exclude,
                        &effective_include,
                        debug_filters
                    );
                    
                    // Limit results after filtering
                    results.truncate(effective_limit);
                    
                    // Update metrics with filter effectiveness
                    if original_count != results.len() {
                        metrics.filtered_results_count = Some(results.len());
                    }
                    
                    if debug_filters && original_count != results.len() {
                        println!("🐛 Filtered {} → {} results", original_count, results.len());
                    }
                    
                    if results.is_empty() {
                        println!("🔍 No results found for '{}' ({})", query, metrics.format_summary());
                        if original_count > 0 {
                            println!("💡 {} results were filtered out - try adjusting filters", original_count);
                        } else {
                            println!("💡 Try broader terms or check if directory is indexed");
                        }
                    } else {
                        println!("🔍 Found {} results ({}):\n", results.len(), metrics.format_summary());
                        
                        if debug_filters {
                            println!("⏱️  {}\n", metrics.format_detailed());
                        }
                        
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
                    
                    // Handle save_preset if specified
                    if let Some(preset_name) = save_preset {
                        let filters = SearchFilters::from_cli_args(
                            &effective_file_types,
                            &effective_languages,
                            &effective_scope,
                            &effective_chunk_types,
                            effective_min_similarity,
                            effective_max_similarity,
                            &effective_exclude,
                            &effective_include,
                            Some(effective_limit),
                        );
                        
                        let new_preset = SearchPreset {
                            name: preset_name.clone(),
                            description: format!("Search preset created from query: '{}'", query),
                            filters,
                            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                            usage_count: 0,
                        };
                        
                        match preset_manager.save_preset(&new_preset, false).await {
                            Ok(()) => {
                                println!("\n💾 Saved search as preset '{}'", preset_name);
                                println!("   Use with: aircher search query \"<query>\" --preset {}", preset_name);
                            }
                            Err(e) => {
                                println!("\n⚠️  Failed to save preset: {}", e);
                            }
                        }
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
        
        SearchCommand::Preset { preset_command } => {
            let mut preset_manager = PresetManager::new()?;
            
            match preset_command {
                PresetCommand::List { verbose } => {
                    let presets = preset_manager.list_presets().await?;
                    
                    if presets.is_empty() {
                        println!("📋 No search presets found");
                        println!("💡 Create presets with: aircher search preset save <name> [filters...]");
                        println!("💡 Or initialize built-ins: aircher search preset init");
                    } else {
                        println!("📋 Available search presets:\n");
                        
                        for preset in presets {
                            if verbose {
                                println!("🔖 {}", preset.name);
                                println!("   Description: {}", preset.description);
                                println!("   Filters: {}", preset.filters.format_summary());
                                println!("   Created: {} | Used: {} times\n", preset.created_at, preset.usage_count);
                            } else {
                                println!("🔖 {} - {} ({})", 
                                    preset.name, 
                                    preset.description, 
                                    preset.filters.format_summary()
                                );
                            }
                        }
                        
                        if !verbose {
                            println!("\n💡 Use --verbose for detailed information");
                        }
                        println!("💡 Use preset: aircher search query \"<query>\" --preset <name>");
                    }
                }
                
                PresetCommand::Show { name } => {
                    match preset_manager.get_preset(&name).await? {
                        Some(preset) => {
                            println!("🔖 Preset: {}", preset.name);
                            println!("📝 Description: {}", preset.description);
                            println!("📅 Created: {}", preset.created_at);
                            println!("📊 Usage count: {}", preset.usage_count);
                            println!("\n🔍 Filters:");
                            
                            if let Some(file_types) = &preset.filters.file_types {
                                println!("   File types: {}", file_types.join(", "));
                            }
                            if let Some(languages) = &preset.filters.languages {
                                println!("   Languages: {}", languages.join(", "));
                            }
                            if let Some(scope) = &preset.filters.scope {
                                println!("   Scope: {}", scope.join(", "));
                            }
                            if let Some(chunk_types) = &preset.filters.chunk_types {
                                println!("   Chunk types: {}", chunk_types.join(", "));
                            }
                            if let Some(min_sim) = preset.filters.min_similarity {
                                println!("   Min similarity: {:.2}", min_sim);
                            }
                            if let Some(max_sim) = preset.filters.max_similarity {
                                println!("   Max similarity: {:.2}", max_sim);
                            }
                            if let Some(exclude) = &preset.filters.exclude {
                                println!("   Exclude: {}", exclude.join(", "));
                            }
                            if let Some(include) = &preset.filters.include {
                                println!("   Include: {}", include.join(", "));
                            }
                            if let Some(limit) = preset.filters.limit {
                                println!("   Limit: {}", limit);
                            }
                            
                            println!("\n💡 CLI equivalent:");
                            let cli_args = preset.filters.to_cli_args();
                            if cli_args.is_empty() {
                                println!("   aircher search query \"<query>\"");
                            } else {
                                println!("   aircher search query \"<query>\" {}", cli_args.join(" "));
                            }
                        }
                        None => {
                            println!("⚠️  Preset '{}' not found", name);
                            println!("💡 List available presets: aircher search preset list");
                        }
                    }
                }
                
                PresetCommand::Save { 
                    name, 
                    description, 
                    global, 
                    file_types,
                    languages,
                    scope,
                    chunk_types,
                    min_similarity,
                    max_similarity,
                    exclude,
                    include,
                    limit
                } => {
                    let filters = SearchFilters::from_cli_args(
                        &file_types,
                        &languages,
                        &scope,
                        &chunk_types,
                        min_similarity,
                        max_similarity,
                        &exclude,
                        &include,
                        limit,
                    );
                    
                    let preset = SearchPreset {
                        name: name.clone(),
                        description: description.clone().unwrap_or_else(|| format!("Custom search preset: {}", name)),
                        filters,
                        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                        usage_count: 0,
                    };
                    
                    match preset_manager.save_preset(&preset, global).await {
                        Ok(()) => {
                            let scope_str = if global { "globally" } else { "locally" };
                            println!("💾 Saved preset '{}' {}", name, scope_str);
                            println!("   Description: {}", preset.description);
                            println!("   Filters: {}", preset.filters.format_summary());
                            println!("💡 Use with: aircher search query \"<query>\" --preset {}", name);
                        }
                        Err(e) => {
                            println!("❌ Failed to save preset: {}", e);
                        }
                    }
                }
                
                PresetCommand::Delete { name, global } => {
                    match preset_manager.delete_preset(&name, global).await {
                        Ok(true) => {
                            let scope_str = if global { "global" } else { "local" };
                            println!("🗑️  Deleted {} preset '{}'", scope_str, name);
                        }
                        Ok(false) => {
                            let scope_str = if global { "global" } else { "local" };
                            println!("⚠️  {} preset '{}' not found", scope_str, name);
                        }
                        Err(e) => {
                            println!("❌ Failed to delete preset: {}", e);
                        }
                    }
                }
                
                PresetCommand::Init { force } => {
                    if force {
                        println!("🔄 Initializing built-in presets (force mode)...");
                    } else {
                        println!("🔄 Initializing built-in presets...");
                    }
                    
                    match preset_manager.create_builtin_presets().await {
                        Ok(()) => {
                            println!("✅ Built-in presets created:");
                            println!("   🔖 rust-functions - Rust functions and methods");
                            println!("   🔖 auth-security - Authentication and security patterns");
                            println!("   🔖 error-handling - Error handling and exception patterns");
                            println!("   🔖 config-patterns - Configuration and settings code");
                            println!("\n💡 List all presets: aircher search preset list");
                            println!("💡 Use preset: aircher search query \"<query>\" --preset <name>");
                        }
                        Err(e) => {
                            println!("❌ Failed to create built-in presets: {}", e);
                        }
                    }
                }
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