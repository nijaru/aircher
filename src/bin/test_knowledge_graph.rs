// Test program to build knowledge graph from Aircher codebase
//
// Purpose: Validate knowledge graph implementation by:
// - Building graph from Aircher Rust code
// - Measuring build time
// - Counting nodes and edges
// - Testing query performance
// - Saving/loading graph
//
// Target: Reproduce POC numbers (3,942 nodes, 5,217 edges)

use aircher::intelligence::{GraphBuilder, KnowledgeGraph};
use anyhow::Result;
use std::time::Instant;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,aircher=debug")
        .init();

    println!("=== Aircher Knowledge Graph Test ===\n");

    // Get project root
    let project_root = std::env::current_dir()?;
    println!("Project root: {:?}\n", project_root);

    // Build knowledge graph
    println!("Building knowledge graph...");
    let start = Instant::now();

    let mut builder = GraphBuilder::new(project_root.clone())?;
    let graph = builder.build_graph()?;

    let build_time = start.elapsed();
    println!("Build time: {:?}\n", build_time);

    // Display statistics
    let stats = graph.stats();
    println!("Graph Statistics:");
    println!("  Nodes: {}", stats.node_count);
    println!("  Edges: {}", stats.edge_count);
    println!("  Files: {}", stats.file_count);
    println!("  Symbols: {}", stats.symbol_count);
    println!();

    // Compare to POC target
    println!("POC Target:");
    println!("  Nodes: 3,942");
    println!("  Edges: 5,217");
    println!();

    let node_ratio = stats.node_count as f64 / 3942.0 * 100.0;
    let edge_ratio = stats.edge_count as f64 / 5217.0 * 100.0;
    println!("Achieved:");
    println!("  Nodes: {:.1}% of POC target", node_ratio);
    println!("  Edges: {:.1}% of POC target", edge_ratio);
    println!();

    // Test some queries
    println!("Testing queries...");

    // Find files
    let all_files = graph.get_all_files();
    println!("  Total files indexed: {}", all_files.len());

    if let Some(first_file) = all_files.first() {
        println!("  Sample file: {:?}", first_file);

        // Query file contents
        let start = Instant::now();
        let contents = graph.get_file_contents(first_file)?;
        let query_time = start.elapsed();

        println!("    Functions/structs: {}", contents.len());
        println!("    Query time: {:?}", query_time);

        // Sample some symbols
        for (i, node) in contents.iter().take(3).enumerate() {
            match node {
                aircher::intelligence::NodeType::Function { name, line, .. } => {
                    println!("    - Function '{}' at line {}", name, line);
                }
                aircher::intelligence::NodeType::Class { name, line, .. } => {
                    println!("    - Struct '{}' at line {}", name, line);
                }
                _ => {}
            }
        }
    }
    println!();

    // Test save/load
    let graph_file = project_root.join("target").join("knowledge_graph.bin");
    println!("Testing persistence...");
    println!("  Saving to: {:?}", graph_file);

    let start = Instant::now();
    graph.save(&graph_file)?;
    let save_time = start.elapsed();
    println!("  Save time: {:?}", save_time);

    let file_size = std::fs::metadata(&graph_file)?.len();
    println!("  File size: {} MB", file_size / 1_000_000);

    let start = Instant::now();
    let loaded_graph = KnowledgeGraph::load(&graph_file)?;
    let load_time = start.elapsed();
    println!("  Load time: {:?}", load_time);

    let loaded_stats = loaded_graph.stats();
    println!("  Loaded nodes: {}", loaded_stats.node_count);
    println!("  Loaded edges: {}", loaded_stats.edge_count);

    assert_eq!(stats.node_count, loaded_stats.node_count, "Node count mismatch after load");
    assert_eq!(stats.edge_count, loaded_stats.edge_count, "Edge count mismatch after load");
    println!("  ✓ Save/load verified");
    println!();

    // Test incremental update
    println!("Testing incremental update...");
    if let Some(test_file) = all_files.iter().find(|f| {
        f.file_name().and_then(|n| n.to_str()).map_or(false, |n| n == "mod.rs")
    }) {
        println!("  Test file: {:?}", test_file);

        let before_nodes = loaded_stats.node_count;
        let before_edges = loaded_stats.edge_count;

        let mut graph_mut = loaded_graph;
        let start = Instant::now();
        builder.update_file(test_file, &mut graph_mut)?;
        let update_time = start.elapsed();

        let after_stats = graph_mut.stats();
        println!("  Update time: {:?}", update_time);
        println!("  Nodes: {} -> {}", before_nodes, after_stats.node_count);
        println!("  Edges: {} -> {}", before_edges, after_stats.edge_count);
        println!("  ✓ Incremental update complete");
    }
    println!();

    println!("=== Test Complete ===");
    println!();
    println!("Summary:");
    println!("  Build: {:?}", build_time);
    println!("  Nodes: {} ({:.1}% of POC)", stats.node_count, node_ratio);
    println!("  Edges: {} ({:.1}% of POC)", stats.edge_count, edge_ratio);
    println!("  Files: {}", stats.file_count);
    println!("  Graph size: {} MB", file_size / 1_000_000);

    Ok(())
}
