// Week 4: Knowledge Graph Builder - Extract code structure with tree-sitter
//
// Purpose: Build knowledge graph by parsing codebase with tree-sitter
// Supports: Rust (initial), expandable to 19+ languages
//
// Process:
// 1. Scan repository for source files
// 2. Parse each file with tree-sitter
// 3. Extract nodes (functions, classes, variables, imports)
// 4. Extract edges (calls, contains, imports, uses)
// 5. Build petgraph structure

use super::knowledge_graph::{EdgeType, KnowledgeGraph, NodeType};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use streaming_iterator::StreamingIterator;
use tracing::{debug, info, warn};
use tree_sitter::{Parser, Query, QueryCursor, Tree};
use walkdir::WalkDir;

/// Language configuration for tree-sitter parsing
pub struct LanguageConfig {
    pub name: String,
    pub extensions: Vec<String>,
    pub parser: Parser,
    pub queries: LanguageQueries,
}

/// Tree-sitter queries for extracting code structure
pub struct LanguageQueries {
    /// Query for finding function definitions
    pub functions: Option<Query>,
    /// Query for finding class/struct definitions
    pub classes: Option<Query>,
    /// Query for finding import statements
    pub imports: Option<Query>,
    /// Query for finding function calls
    pub calls: Option<Query>,
}

/// Main builder for constructing knowledge graphs
pub struct GraphBuilder {
    root_path: PathBuf,
    languages: HashMap<String, LanguageConfig>,
}

impl GraphBuilder {
    /// Create a new graph builder for a repository
    pub fn new(root_path: PathBuf) -> Result<Self> {
        info!("Creating graph builder for {:?}", root_path);

        let mut builder = Self {
            root_path,
            languages: HashMap::new(),
        };

        // Initialize Rust language support
        builder.add_rust_support()?;

        Ok(builder)
    }

    /// Add Rust language parsing support
    fn add_rust_support(&mut self) -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .context("Failed to set Rust language")?;

        // Tree-sitter queries for Rust code structure
        let function_query = Query::new(
            &tree_sitter_rust::LANGUAGE.into(),
            r#"
            (function_item
                name: (identifier) @name
                parameters: (parameters) @params) @function
            "#,
        )
        .ok();

        let struct_query = Query::new(
            &tree_sitter_rust::LANGUAGE.into(),
            r#"
            (struct_item
                name: (type_identifier) @name) @struct
            "#,
        )
        .ok();

        let impl_query = Query::new(
            &tree_sitter_rust::LANGUAGE.into(),
            r#"
            (impl_item
                type: (type_identifier) @type
                body: (declaration_list) @body) @impl
            "#,
        )
        .ok();

        let use_query = Query::new(
            &tree_sitter_rust::LANGUAGE.into(),
            r#"
            (use_declaration
                argument: (_) @path) @use
            "#,
        )
        .ok();

        let call_query = Query::new(
            &tree_sitter_rust::LANGUAGE.into(),
            r#"
            (call_expression
                function: (_) @function) @call
            "#,
        )
        .ok();

        let config = LanguageConfig {
            name: "rust".to_string(),
            extensions: vec!["rs".to_string()],
            parser,
            queries: LanguageQueries {
                functions: function_query,
                classes: struct_query,
                imports: use_query,
                calls: call_query,
            },
        };

        self.languages.insert("rust".to_string(), config);
        info!("Added Rust language support");

        Ok(())
    }

    /// Build knowledge graph by scanning repository
    pub fn build_graph(&mut self) -> Result<KnowledgeGraph> {
        info!("Building knowledge graph for {:?}", self.root_path);

        let mut graph = KnowledgeGraph::new(self.root_path.clone());

        // Scan repository for source files
        let files = self.scan_repository()?;
        info!("Found {} source files", files.len());

        // Parse each file and add to graph
        for file_path in files {
            if let Err(e) = self.process_file(&file_path, &mut graph) {
                warn!("Failed to process {:?}: {}", file_path, e);
                continue;
            }
        }

        let stats = graph.stats();
        info!("Built knowledge graph: {}", stats);

        Ok(graph)
    }

    /// Scan repository for source files
    fn scan_repository(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(&self.root_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // Skip hidden directories and common ignore patterns
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.')
                    && name != "target"
                    && name != "node_modules"
                    && name != "dist"
            })
        {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            // Check if file extension matches any supported language
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                for lang in self.languages.values() {
                    if lang.extensions.contains(&ext.to_string()) {
                        files.push(path.to_path_buf());
                        break;
                    }
                }
            }
        }

        Ok(files)
    }

    /// Process a single file and add nodes/edges to graph
    fn process_file(&mut self, file_path: &Path, graph: &mut KnowledgeGraph) -> Result<()> {
        debug!("Processing file: {:?}", file_path);

        // Determine language from extension
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .context("No file extension")?;

        let lang_name = self
            .languages
            .values()
            .find(|l| l.extensions.contains(&ext.to_string()))
            .map(|l| l.name.clone())
            .context("Unsupported language")?;

        // Read file content
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {:?}", file_path))?;

        let line_count = content.lines().count();

        // Add file node to graph
        let file_node = NodeType::File {
            path: file_path.to_path_buf(),
            language: lang_name.clone(),
            line_count,
        };
        let file_idx = graph.add_node(file_node);

        // Parse file with tree-sitter
        let lang_config = self
            .languages
            .get_mut(&lang_name)
            .context("Language config not found")?;

        let tree = lang_config
            .parser
            .parse(&content, None)
            .context("Failed to parse file")?;

        // Extract nodes and edges based on language
        match lang_name.as_str() {
            "rust" => self.extract_rust_structure(&content, &tree, file_path, file_idx, graph)?,
            _ => {
                warn!("Language not yet implemented: {}", lang_name);
            }
        }

        Ok(())
    }

    /// Extract Rust code structure from tree-sitter parse tree
    fn extract_rust_structure(
        &self,
        content: &str,
        tree: &Tree,
        file_path: &Path,
        file_idx: petgraph::graph::NodeIndex,
        graph: &mut KnowledgeGraph,
    ) -> Result<()> {
        let root_node = tree.root_node();
        let lang_config = self
            .languages
            .get("rust")
            .context("Rust language config not found")?;

        // Extract functions
        if let Some(function_query) = &lang_config.queries.functions {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(function_query, root_node, content.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures.iter() {
                    let capture_name = &function_query.capture_names()[capture.index as usize];

                    if capture_name == "name" {
                        let name = capture
                            .node
                            .utf8_text(content.as_bytes())
                            .unwrap_or("")
                            .to_string();

                        // Get parent function node to extract full signature
                        if let Some(parent) = capture.node.parent() {
                            let signature = parent
                                .utf8_text(content.as_bytes())
                                .unwrap_or("")
                                .lines()
                                .next()
                                .unwrap_or("")
                                .to_string();

                            let line = parent.start_position().row + 1;

                            let func_node = NodeType::Function {
                                name,
                                signature,
                                line,
                                file_path: file_path.to_path_buf(),
                            };

                            let func_idx = graph.add_node(func_node);
                            graph.add_edge(file_idx, func_idx, EdgeType::Contains);
                        }
                    }
                }
            }
        }

        // Extract structs/classes
        if let Some(class_query) = &lang_config.queries.classes {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(class_query, root_node, content.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures.iter() {
                    let capture_name = &class_query.capture_names()[capture.index as usize];

                    if capture_name == "name" {
                        let name = capture
                            .node
                            .utf8_text(content.as_bytes())
                            .unwrap_or("")
                            .to_string();

                        let line = capture.node.start_position().row + 1;

                        let class_node = NodeType::Class {
                            name,
                            line,
                            file_path: file_path.to_path_buf(),
                        };

                        let class_idx = graph.add_node(class_node);
                        graph.add_edge(file_idx, class_idx, EdgeType::Contains);
                    }
                }
            }
        }

        // Extract imports/use statements
        if let Some(import_query) = &lang_config.queries.imports {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(import_query, root_node, content.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures.iter() {
                    let capture_name = &import_query.capture_names()[capture.index as usize];

                    if capture_name == "path" {
                        let module = capture
                            .node
                            .utf8_text(content.as_bytes())
                            .unwrap_or("")
                            .to_string();

                        let import_node = NodeType::Import {
                            module,
                            items: vec![], // TODO: Extract specific items
                            file_path: file_path.to_path_buf(),
                        };

                        let import_idx = graph.add_node(import_node);
                        graph.add_edge(file_idx, import_idx, EdgeType::Contains);
                    }
                }
            }
        }

        Ok(())
    }

    /// Update graph for a single changed file (incremental update)
    pub fn update_file(&mut self, file_path: &Path, graph: &mut KnowledgeGraph) -> Result<()> {
        info!("Updating graph for changed file: {:?}", file_path);

        // Remove old nodes for this file
        if let Some(file_idx) = graph.get_file_index(file_path) {
            // TODO: Remove edges and contained nodes
            warn!("Incremental update not fully implemented yet");
        }

        // Re-process file
        self.process_file(file_path, graph)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_builder_creation() {
        let temp_dir = TempDir::new().unwrap();
        let builder = GraphBuilder::new(temp_dir.path().to_path_buf());
        assert!(builder.is_ok());
    }

    #[test]
    fn test_rust_language_support() {
        let temp_dir = TempDir::new().unwrap();
        let builder = GraphBuilder::new(temp_dir.path().to_path_buf()).unwrap();
        assert!(builder.languages.contains_key("rust"));
    }

    #[test]
    fn test_scan_empty_repository() {
        let temp_dir = TempDir::new().unwrap();
        let builder = GraphBuilder::new(temp_dir.path().to_path_buf()).unwrap();
        let files = builder.scan_repository().unwrap();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_scan_with_rust_file() {
        let temp_dir = TempDir::new().unwrap();

        // Create a test Rust file
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let builder = GraphBuilder::new(temp_dir.path().to_path_buf()).unwrap();
        let files = builder.scan_repository().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], test_file);
    }

    #[test]
    fn test_build_simple_graph() {
        let temp_dir = TempDir::new().unwrap();

        // Create a simple Rust file
        let test_file = temp_dir.path().join("lib.rs");
        fs::write(
            &test_file,
            r#"
fn foo() {
    println!("Hello");
}

struct Bar {
    value: i32,
}

fn main() {
    foo();
}
"#,
        )
        .unwrap();

        let mut builder = GraphBuilder::new(temp_dir.path().to_path_buf()).unwrap();
        let graph = builder.build_graph().unwrap();

        let stats = graph.stats();
        assert!(stats.node_count > 0, "Expected nodes in graph");
        assert!(stats.file_count > 0, "Expected files in graph");

        // Check that we can query the graph
        let file_contents = graph.get_file_contents(&test_file);
        assert!(file_contents.is_ok(), "Should be able to query file contents");
    }
}
