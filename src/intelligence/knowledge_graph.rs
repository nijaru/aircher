// Week 4: Knowledge Graph - Semantic memory of codebase structure
//
// Purpose: Understand code relationships for instant queries like:
// - "What functions are in file X?"
// - "What calls function Y?"
// - "What files does module Z depend on?"
//
// Architecture: petgraph in-memory for microsecond traversals
// POC target: 3,942 nodes, 5,217 edges (Aircher codebase)

use anyhow::{Context, Result};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Node types in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    /// File node - represents a source code file
    File {
        path: PathBuf,
        language: String,
        line_count: usize,
    },
    /// Function node - represents a function/method
    Function {
        name: String,
        signature: String,
        line: usize,
        file_path: PathBuf,
    },
    /// Class node - represents a class/struct/interface
    Class {
        name: String,
        line: usize,
        file_path: PathBuf,
    },
    /// Import node - represents an import/require/use statement
    Import {
        module: String,
        items: Vec<String>,
        file_path: PathBuf,
    },
    /// Variable node - represents a global/module-level variable
    Variable {
        name: String,
        scope: String,
        file_path: PathBuf,
    },
}

/// Edge types representing relationships between nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeType {
    /// File contains function/class/variable
    Contains,
    /// Function calls another function
    Calls,
    /// File imports module
    Imports,
    /// Function uses variable
    Uses,
    /// Class inherits from another class
    Inherits,
    /// Function references class
    References,
}

/// Main knowledge graph structure
pub struct KnowledgeGraph {
    /// Directed graph: nodes are code entities, edges are relationships
    graph: DiGraph<NodeType, EdgeType>,

    /// Fast lookup: file path -> node index
    file_index: HashMap<PathBuf, NodeIndex>,

    /// Fast lookup: symbol name -> node indices (multiple files may define same name)
    symbol_index: HashMap<String, Vec<NodeIndex>>,

    /// Metadata
    root_path: PathBuf,
    node_count: usize,
    edge_count: usize,
}

impl KnowledgeGraph {
    /// Create a new empty knowledge graph
    pub fn new(root_path: PathBuf) -> Self {
        info!("Creating new knowledge graph for {:?}", root_path);
        Self {
            graph: DiGraph::new(),
            file_index: HashMap::new(),
            symbol_index: HashMap::new(),
            root_path,
            node_count: 0,
            edge_count: 0,
        }
    }

    /// Add a node to the graph, updating indices
    pub fn add_node(&mut self, node_type: NodeType) -> NodeIndex {
        let node_idx = self.graph.add_node(node_type.clone());
        self.node_count += 1;

        // Update file index
        match &node_type {
            NodeType::File { path, .. } => {
                self.file_index.insert(path.clone(), node_idx);
            }
            _ => {}
        }

        // Update symbol index
        match &node_type {
            NodeType::Function { name, .. }
            | NodeType::Class { name, .. }
            | NodeType::Variable { name, .. } => {
                self.symbol_index
                    .entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(node_idx);
            }
            NodeType::Import { module, .. } => {
                self.symbol_index
                    .entry(module.clone())
                    .or_insert_with(Vec::new)
                    .push(node_idx);
            }
            _ => {}
        }

        debug!("Added node: {:?}", node_type);
        node_idx
    }

    /// Add an edge between two nodes
    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, edge_type: EdgeType) {
        self.graph.add_edge(from, to, edge_type);
        self.edge_count += 1;
    }

    /// Get all nodes in a file (functions, classes, variables)
    pub fn get_file_contents(&self, file_path: &Path) -> Result<Vec<NodeType>> {
        let file_idx = self
            .file_index
            .get(file_path)
            .context(format!("File not found in graph: {:?}", file_path))?;

        // Find all nodes that this file contains
        let mut contents = Vec::new();
        for edge in self.graph.edges_directed(*file_idx, Direction::Outgoing) {
            if let EdgeType::Contains = edge.weight() {
                let target_idx = edge.target();
                let node = &self.graph[target_idx];
                contents.push(node.clone());
            }
        }

        Ok(contents)
    }

    /// Find all callers of a function
    pub fn get_callers(&self, function_name: &str) -> Result<Vec<NodeType>> {
        let function_indices = self
            .symbol_index
            .get(function_name)
            .context(format!("Function not found: {}", function_name))?;

        let mut callers = Vec::new();
        for &func_idx in function_indices {
            // Find all incoming "Calls" edges
            for edge in self.graph.edges_directed(func_idx, Direction::Incoming) {
                if let EdgeType::Calls = edge.weight() {
                    let caller = &self.graph[edge.source()];
                    callers.push(caller.clone());
                }
            }
        }

        Ok(callers)
    }

    /// Get dependencies of a file (what it imports)
    pub fn get_dependencies(&self, file_path: &Path) -> Result<Vec<NodeType>> {
        let file_idx = self
            .file_index
            .get(file_path)
            .context(format!("File not found in graph: {:?}", file_path))?;

        let mut dependencies = Vec::new();

        // Find all import nodes in this file
        for edge in self.graph.edges_directed(*file_idx, Direction::Outgoing) {
            if let EdgeType::Contains = edge.weight() {
                let node = &self.graph[edge.target()];
                if let NodeType::Import { module, .. } = node {
                    // Find the file that defines this module
                    if let Some(target_indices) = self.symbol_index.get(module) {
                        for &target_idx in target_indices {
                            let target_node = &self.graph[target_idx];
                            dependencies.push(target_node.clone());
                        }
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Find where a symbol is defined
    pub fn find_symbol(&self, symbol_name: &str) -> Result<Vec<NodeType>> {
        let indices = self
            .symbol_index
            .get(symbol_name)
            .context(format!("Symbol not found: {}", symbol_name))?;

        let symbols: Vec<NodeType> = indices
            .iter()
            .map(|&idx| self.graph[idx].clone())
            .collect();

        Ok(symbols)
    }

    /// Get statistics about the graph
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            node_count: self.node_count,
            edge_count: self.edge_count,
            file_count: self.file_index.len(),
            symbol_count: self.symbol_index.len(),
        }
    }

    /// Get all files in the graph
    pub fn get_all_files(&self) -> Vec<PathBuf> {
        self.file_index.keys().cloned().collect()
    }

    /// Get node by index (for advanced queries)
    pub fn get_node(&self, idx: NodeIndex) -> Option<&NodeType> {
        self.graph.node_weight(idx)
    }

    /// Get file index for a path
    pub fn get_file_index(&self, path: &Path) -> Option<NodeIndex> {
        self.file_index.get(path).copied()
    }

    /// Remove all nodes and edges for a file (for incremental updates)
    pub fn remove_file(&mut self, file_path: &Path) -> Result<()> {
        let file_idx = match self.file_index.get(file_path) {
            Some(idx) => *idx,
            None => return Ok(()), // File not in graph, nothing to remove
        };

        debug!("Removing file from graph: {:?}", file_path);

        // Collect all nodes contained by this file
        let mut nodes_to_remove = Vec::new();
        for edge in self.graph.edges_directed(file_idx, Direction::Outgoing) {
            if let EdgeType::Contains = edge.weight() {
                nodes_to_remove.push(edge.target());
            }
        }

        // Remove contained nodes (functions, classes, imports, variables)
        for node_idx in nodes_to_remove {
            // Remove from symbol index
            if let Some(node) = self.graph.node_weight(node_idx) {
                match node {
                    NodeType::Function { name, .. }
                    | NodeType::Class { name, .. }
                    | NodeType::Variable { name, .. } => {
                        if let Some(indices) = self.symbol_index.get_mut(name) {
                            indices.retain(|&idx| idx != node_idx);
                            if indices.is_empty() {
                                self.symbol_index.remove(name);
                            }
                        }
                    }
                    NodeType::Import { module, .. } => {
                        if let Some(indices) = self.symbol_index.get_mut(module) {
                            indices.retain(|&idx| idx != node_idx);
                            if indices.is_empty() {
                                self.symbol_index.remove(module);
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Remove node from graph
            self.graph.remove_node(node_idx);
            self.node_count -= 1;
        }

        // Remove file node itself
        self.file_index.remove(file_path);
        self.graph.remove_node(file_idx);
        self.node_count -= 1;

        // Update edge count (approximate - edges are automatically removed with nodes)
        self.edge_count = self.graph.edge_count();

        debug!("Removed file: {:?}, {} nodes remaining", file_path, self.node_count);

        Ok(())
    }

    /// Save knowledge graph to binary file
    pub fn save(&self, path: &Path) -> Result<()> {
        info!("Saving knowledge graph to {:?}", path);

        // Create serializable representation
        let serializable = SerializableGraph {
            nodes: self.graph.node_weights().cloned().collect(),
            edges: self
                .graph
                .edge_references()
                .map(|e| {
                    let source = e.source().index();
                    let target = e.target().index();
                    let weight = e.weight().clone();
                    (source, target, weight)
                })
                .collect(),
            root_path: self.root_path.clone(),
        };

        // Serialize to binary with bincode
        let encoded = bincode::serialize(&serializable)
            .context("Failed to serialize knowledge graph")?;

        // Write to file
        std::fs::write(path, encoded).context("Failed to write knowledge graph file")?;

        info!(
            "Saved knowledge graph: {} nodes, {} edges",
            self.node_count, self.edge_count
        );

        Ok(())
    }

    /// Load knowledge graph from binary file
    pub fn load(path: &Path) -> Result<Self> {
        info!("Loading knowledge graph from {:?}", path);

        // Read binary file
        let encoded = std::fs::read(path).context("Failed to read knowledge graph file")?;

        // Deserialize with bincode
        let serializable: SerializableGraph =
            bincode::deserialize(&encoded).context("Failed to deserialize knowledge graph")?;

        // Rebuild graph
        let mut graph = DiGraph::new();
        let mut node_map: HashMap<usize, NodeIndex> = HashMap::new();

        // Add all nodes
        for (idx, node) in serializable.nodes.into_iter().enumerate() {
            let new_idx = graph.add_node(node);
            node_map.insert(idx, new_idx);
        }

        // Add all edges
        for (source_idx, target_idx, edge_type) in serializable.edges {
            let source = node_map
                .get(&source_idx)
                .context("Invalid source node index")?;
            let target = node_map
                .get(&target_idx)
                .context("Invalid target node index")?;
            graph.add_edge(*source, *target, edge_type);
        }

        // Rebuild indices
        let mut file_index = HashMap::new();
        let mut symbol_index: HashMap<String, Vec<NodeIndex>> = HashMap::new();

        for (node_idx, node) in graph.node_weights().enumerate() {
            let idx = NodeIndex::new(node_idx);

            match node {
                NodeType::File { path, .. } => {
                    file_index.insert(path.clone(), idx);
                }
                NodeType::Function { name, .. }
                | NodeType::Class { name, .. }
                | NodeType::Variable { name, .. } => {
                    symbol_index
                        .entry(name.clone())
                        .or_insert_with(Vec::new)
                        .push(idx);
                }
                NodeType::Import { module, .. } => {
                    symbol_index
                        .entry(module.clone())
                        .or_insert_with(Vec::new)
                        .push(idx);
                }
            }
        }

        let node_count = graph.node_count();
        let edge_count = graph.edge_count();

        info!("Loaded knowledge graph: {} nodes, {} edges", node_count, edge_count);

        Ok(Self {
            graph,
            file_index,
            symbol_index,
            root_path: serializable.root_path,
            node_count,
            edge_count,
        })
    }
}

/// Serializable representation of knowledge graph for persistence
#[derive(Debug, Serialize, Deserialize)]
struct SerializableGraph {
    nodes: Vec<NodeType>,
    edges: Vec<(usize, usize, EdgeType)>,
    root_path: PathBuf,
}

/// Statistics about the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub file_count: usize,
    pub symbol_count: usize,
}

impl std::fmt::Display for GraphStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Knowledge Graph: {} nodes, {} edges ({} files, {} symbols)",
            self.node_count, self.edge_count, self.file_count, self.symbol_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let graph = KnowledgeGraph::new(PathBuf::from("/test"));
        let stats = graph.stats();
        assert_eq!(stats.node_count, 0);
        assert_eq!(stats.edge_count, 0);
    }

    #[test]
    fn test_add_file_node() {
        let mut graph = KnowledgeGraph::new(PathBuf::from("/test"));

        let file_node = NodeType::File {
            path: PathBuf::from("/test/main.rs"),
            language: "rust".to_string(),
            line_count: 100,
        };

        graph.add_node(file_node);

        let stats = graph.stats();
        assert_eq!(stats.node_count, 1);
        assert_eq!(stats.file_count, 1);
    }

    #[test]
    fn test_add_function_and_contains_edge() {
        let mut graph = KnowledgeGraph::new(PathBuf::from("/test"));

        let file_node = NodeType::File {
            path: PathBuf::from("/test/main.rs"),
            language: "rust".to_string(),
            line_count: 100,
        };
        let file_idx = graph.add_node(file_node);

        let func_node = NodeType::Function {
            name: "main".to_string(),
            signature: "fn main()".to_string(),
            line: 10,
            file_path: PathBuf::from("/test/main.rs"),
        };
        let func_idx = graph.add_node(func_node);

        graph.add_edge(file_idx, func_idx, EdgeType::Contains);

        let stats = graph.stats();
        assert_eq!(stats.node_count, 2);
        assert_eq!(stats.edge_count, 1);

        // Test get_file_contents
        let contents = graph.get_file_contents(&PathBuf::from("/test/main.rs")).unwrap();
        assert_eq!(contents.len(), 1);

        if let NodeType::Function { name, .. } = &contents[0] {
            assert_eq!(name, "main");
        } else {
            panic!("Expected Function node");
        }
    }

    #[test]
    fn test_find_symbol() {
        let mut graph = KnowledgeGraph::new(PathBuf::from("/test"));

        let func_node = NodeType::Function {
            name: "foo".to_string(),
            signature: "fn foo()".to_string(),
            line: 5,
            file_path: PathBuf::from("/test/lib.rs"),
        };
        graph.add_node(func_node);

        let symbols = graph.find_symbol("foo").unwrap();
        assert_eq!(symbols.len(), 1);

        if let NodeType::Function { name, .. } = &symbols[0] {
            assert_eq!(name, "foo");
        } else {
            panic!("Expected Function node");
        }
    }

    #[test]
    fn test_get_callers() {
        let mut graph = KnowledgeGraph::new(PathBuf::from("/test"));

        let caller = NodeType::Function {
            name: "caller".to_string(),
            signature: "fn caller()".to_string(),
            line: 10,
            file_path: PathBuf::from("/test/main.rs"),
        };
        let caller_idx = graph.add_node(caller);

        let callee = NodeType::Function {
            name: "callee".to_string(),
            signature: "fn callee()".to_string(),
            line: 20,
            file_path: PathBuf::from("/test/main.rs"),
        };
        let callee_idx = graph.add_node(callee);

        graph.add_edge(caller_idx, callee_idx, EdgeType::Calls);

        let callers = graph.get_callers("callee").unwrap();
        assert_eq!(callers.len(), 1);

        if let NodeType::Function { name, .. } = &callers[0] {
            assert_eq!(name, "caller");
        } else {
            panic!("Expected Function node");
        }
    }

    #[test]
    fn test_remove_file() {
        let mut graph = KnowledgeGraph::new(PathBuf::from("/test"));

        // Add file with some functions
        let file_path = PathBuf::from("/test/lib.rs");
        let file_node = NodeType::File {
            path: file_path.clone(),
            language: "rust".to_string(),
            line_count: 50,
        };
        let file_idx = graph.add_node(file_node);

        let func1 = NodeType::Function {
            name: "foo".to_string(),
            signature: "fn foo()".to_string(),
            line: 5,
            file_path: file_path.clone(),
        };
        let func1_idx = graph.add_node(func1);

        let func2 = NodeType::Function {
            name: "bar".to_string(),
            signature: "fn bar()".to_string(),
            line: 10,
            file_path: file_path.clone(),
        };
        let func2_idx = graph.add_node(func2);

        graph.add_edge(file_idx, func1_idx, EdgeType::Contains);
        graph.add_edge(file_idx, func2_idx, EdgeType::Contains);

        let stats_before = graph.stats();
        assert_eq!(stats_before.node_count, 3);
        assert_eq!(stats_before.edge_count, 2);

        // Remove file
        graph.remove_file(&file_path).unwrap();

        let stats_after = graph.stats();
        assert_eq!(stats_after.node_count, 0);
        assert_eq!(stats_after.edge_count, 0);
        assert_eq!(stats_after.file_count, 0);

        // Verify symbols removed from index
        assert!(graph.find_symbol("foo").is_err());
        assert!(graph.find_symbol("bar").is_err());
    }

    #[test]
    fn test_save_load() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let graph_file = temp_dir.path().join("test_graph.bin");

        // Create graph
        let mut graph = KnowledgeGraph::new(PathBuf::from("/test"));

        let file_node = NodeType::File {
            path: PathBuf::from("/test/main.rs"),
            language: "rust".to_string(),
            line_count: 100,
        };
        let file_idx = graph.add_node(file_node);

        let func_node = NodeType::Function {
            name: "main".to_string(),
            signature: "fn main()".to_string(),
            line: 10,
            file_path: PathBuf::from("/test/main.rs"),
        };
        let func_idx = graph.add_node(func_node);

        graph.add_edge(file_idx, func_idx, EdgeType::Contains);

        let stats_before = graph.stats();

        // Save
        graph.save(&graph_file).unwrap();
        assert!(graph_file.exists());

        // Load
        let loaded_graph = KnowledgeGraph::load(&graph_file).unwrap();
        let stats_after = loaded_graph.stats();

        // Verify same structure
        assert_eq!(stats_before.node_count, stats_after.node_count);
        assert_eq!(stats_before.edge_count, stats_after.edge_count);
        assert_eq!(stats_before.file_count, stats_after.file_count);
        assert_eq!(stats_before.symbol_count, stats_after.symbol_count);

        // Verify can query loaded graph
        let contents = loaded_graph
            .get_file_contents(&PathBuf::from("/test/main.rs"))
            .unwrap();
        assert_eq!(contents.len(), 1);
    }

    #[test]
    fn test_multiple_files_same_symbol() {
        let mut graph = KnowledgeGraph::new(PathBuf::from("/test"));

        // Add two files with same function name
        let file1 = PathBuf::from("/test/a.rs");
        let file1_node = NodeType::File {
            path: file1.clone(),
            language: "rust".to_string(),
            line_count: 10,
        };
        graph.add_node(file1_node);

        let func1 = NodeType::Function {
            name: "test".to_string(),
            signature: "fn test()".to_string(),
            line: 5,
            file_path: file1.clone(),
        };
        graph.add_node(func1);

        let file2 = PathBuf::from("/test/b.rs");
        let file2_node = NodeType::File {
            path: file2.clone(),
            language: "rust".to_string(),
            line_count: 10,
        };
        graph.add_node(file2_node);

        let func2 = NodeType::Function {
            name: "test".to_string(),
            signature: "fn test()".to_string(),
            line: 5,
            file_path: file2.clone(),
        };
        graph.add_node(func2);

        // Should find both symbols
        let symbols = graph.find_symbol("test").unwrap();
        assert_eq!(symbols.len(), 2);
    }
}
