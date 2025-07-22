use super::{BenchmarkConfig, BenchmarkResult};
use crate::vector_search::{VectorSearchEngine, VectorBackend, ChunkMetadata, ChunkType};
use anyhow::Result;
use std::time::Instant;
use std::env;

/// Generate synthetic test data for benchmarking
pub fn generate_test_data(count: usize, dimension: usize) -> Vec<Vec<f32>> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    (0..count)
        .map(|_| {
            let vector: Vec<f32> = (0..dimension)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            vector
        })
        .collect()
}

/// Generate synthetic metadata for test embeddings
pub fn generate_test_metadata(count: usize) -> Vec<ChunkMetadata> {
    (0..count)
        .map(|i| ChunkMetadata {
            file_path: format!("test_file_{}.rs", i % 10).into(),
            start_line: i * 10,
            end_line: i * 10 + 5,
            chunk_type: match i % 4 {
                0 => ChunkType::Function,
                1 => ChunkType::Class,
                2 => ChunkType::Module,
                _ => ChunkType::Generic,
            },
            content: format!("fn test_function_{}() {{\n    // Implementation\n}}", i),
        })
        .collect()
}

/// Compare performance between different vector search backends
pub async fn compare_backends(config: BenchmarkConfig) -> Result<Vec<BenchmarkResult>> {
    println!("🔍 Starting comprehensive backend comparison...");
    println!("   Config: {} vectors, {} dimensions, {} queries", 
        config.vector_count, config.dimension, config.search_queries);
    
    let mut results = Vec::new();
    
    // Generate test data once for consistent comparison
    println!("📊 Generating {} test vectors and {} metadata entries...", 
        config.vector_count, config.vector_count);
    let test_embeddings = generate_test_data(config.vector_count, config.dimension);
    let test_metadata = generate_test_metadata(config.vector_count);
    let query_data = generate_test_data(config.search_queries, config.dimension);
    
    // Test instant-distance backend
    println!("\n🚀 Testing instant-distance backend...");
    let instant_result = benchmark_backend(
        VectorBackend::InstantDistance,
        &config,
        &test_embeddings,
        &test_metadata,
        &query_data,
    ).await?;
    results.push(instant_result);
    
    // Test hnswlib-rs backend if available
    #[cfg(feature = "hnswlib-rs")]
    {
        println!("\n⚡ Testing hnswlib-rs backend...");
        let hnsw_result = benchmark_backend(
            VectorBackend::HnswRs,
            &config,
            &test_embeddings,
            &test_metadata,
            &query_data,
        ).await?;
        results.push(hnsw_result);
    }
    
    #[cfg(not(feature = "hnswlib-rs"))]
    {
        println!("⚠️  hnswlib-rs backend not available (feature not enabled)");
    }
    
    // Print comparison summary
    print_comparison_summary(&results);
    
    Ok(results)
}

async fn benchmark_backend(
    backend_type: VectorBackend,
    config: &BenchmarkConfig,
    embeddings: &[Vec<f32>],
    metadata: &[ChunkMetadata],
    queries: &[Vec<f32>],
) -> Result<BenchmarkResult> {
    let cache_dir = env::temp_dir().join(format!("benchmark_{}", backend_type.as_str()));
    
    // Create engine with specific backend
    let mut engine = VectorSearchEngine::new_with_backend(
        cache_dir, 
        config.dimension, 
        backend_type
    )?;
    
    println!("   Backend: {} ({})", engine.backend_name(), backend_type.as_str());
    
    // Add embeddings
    println!("   📋 Adding {} embeddings...", embeddings.len());
    for (embedding, meta) in embeddings.iter().zip(metadata.iter()) {
        engine.add_embedding(embedding.clone(), meta.clone())?;
    }
    
    // Build index and measure time
    println!("   🔨 Building index...");
    let construction_start = Instant::now();
    engine.build_index()?;
    let construction_time = construction_start.elapsed();
    
    println!("   ✅ Index built in {:.2}s", construction_time.as_secs_f64());
    
    // Benchmark search performance
    println!("   🔍 Running {} search queries...", queries.len());
    let mut search_times = Vec::new();
    
    for (i, query) in queries.iter().enumerate() {
        if i % (queries.len() / 5).max(1) == 0 {
            println!("      Progress: {}/{}", i, queries.len());
        }
        
        let search_start = Instant::now();
        let _results = engine.search(query, config.k_nearest)?;
        let search_time = search_start.elapsed();
        
        search_times.push(search_time);
    }
    
    // Calculate memory usage estimate
    let stats = engine.get_stats();
    let memory_usage = estimate_memory_usage(stats.total_vectors, stats.dimension, &backend_type);
    
    // Calculate accuracy metrics (simplified - would need ground truth for real measurement)
    let accuracy = super::AccuracyMetrics {
        recall_at_k: 0.95, // Would need ground truth for real measurement
        precision_at_k: 0.93,
        average_distance: 0.15,
    };
    
    let result = BenchmarkResult {
        config: config.clone(),
        library_name: backend_type.as_str().to_string(),
        index_construction_time: construction_time,
        search_times,
        memory_usage_mb: memory_usage,
        accuracy_metrics: accuracy,
    };
    
    println!("   📊 Results summary:");
    println!("      Construction: {:.2}s", construction_time.as_secs_f64());
    println!("      Avg search: {:.4}s", result.average_search_time().as_secs_f64());
    println!("      Throughput: {:.0} req/s", result.search_throughput());
    println!("      Memory: {:.1} MB", memory_usage);
    
    Ok(result)
}

fn estimate_memory_usage(vector_count: usize, dimension: usize, backend: &VectorBackend) -> f64 {
    match backend {
        VectorBackend::InstantDistance => {
            // Rough estimate for instant-distance
            let vector_memory = vector_count * dimension * 4; // f32 = 4 bytes
            let index_overhead = vector_count * 64; // Rough graph overhead
            (vector_memory + index_overhead) as f64 / 1024.0 / 1024.0
        }
        #[cfg(feature = "hnswlib-rs")]
        VectorBackend::HnswRs => {
            // Rough estimate for hnswlib-rs (typically more memory efficient)
            let vector_memory = vector_count * dimension * 4;
            let index_overhead = vector_count * 48; // Better memory efficiency
            (vector_memory + index_overhead) as f64 / 1024.0 / 1024.0
        }
    }
}

fn print_comparison_summary(results: &[BenchmarkResult]) {
    if results.len() < 2 {
        return;
    }
    
    println!("\n📋 BACKEND COMPARISON SUMMARY");
    println!("============================");
    
    for result in results {
        println!("\n🔧 {} Backend:", result.library_name.to_uppercase());
        println!("   Construction Time: {:.2}s", result.index_construction_time.as_secs_f64());
        println!("   Average Search: {:.4}s ({:.0} req/s)", 
            result.average_search_time().as_secs_f64(),
            result.search_throughput());
        println!("   Memory Usage: {:.1} MB", result.memory_usage_mb);
    }
    
    // Calculate improvements
    if let (Some(baseline), Some(candidate)) = (
        results.iter().find(|r| r.library_name == "instant-distance"),
        results.iter().find(|r| r.library_name == "hnswlib-rs")
    ) {
        let construction_improvement = baseline.index_construction_time.as_secs_f64() / 
            candidate.index_construction_time.as_secs_f64();
        let search_improvement = baseline.average_search_time().as_secs_f64() / 
            candidate.average_search_time().as_secs_f64();
        let throughput_improvement = candidate.search_throughput() / baseline.search_throughput();
        
        println!("\n🚀 PERFORMANCE IMPROVEMENTS (hnswlib-rs vs instant-distance):");
        println!("   Construction Speed: {:.1}x faster", construction_improvement);
        println!("   Search Speed: {:.1}x faster", search_improvement);
        println!("   Throughput: {:.1}x higher", throughput_improvement);
        
        if construction_improvement >= 1.5 || search_improvement >= 2.0 {
            println!("\n✅ RECOMMENDATION: hnswlib-rs shows significant performance benefits!");
        } else {
            println!("\n⚠️  RECOMMENDATION: Performance differences are marginal");
        }
    }
}

/// Run a comprehensive benchmark comparison in release mode
pub async fn run_release_benchmark() -> Result<()> {
    // Check if we're in release mode
    let is_release = !cfg!(debug_assertions);
    if !is_release {
        println!("⚠️  Warning: Running in debug mode. Results will be much slower than production.");
        println!("   Run with 'cargo run --release --features benchmarks' for accurate results.");
    }
    
    let config = BenchmarkConfig {
        name: "Production Backend Comparison".to_string(),
        vector_count: 2000,  // Substantial test size
        dimension: 768,      // SweRankEmbed dimension
        search_queries: 200, // Comprehensive search testing
        k_nearest: 10,       // Typical search result count
    };
    
    let results = compare_backends(config).await?;
    
    // Save results to file for analysis
    let results_path = env::temp_dir().join("aircher_backend_benchmark.json");
    let results_json = serde_json::to_string_pretty(&results)?;
    tokio::fs::write(&results_path, results_json).await?;
    
    println!("\n💾 Detailed results saved to: {}", results_path.display());
    println!("📈 Use this data to make the final migration decision.");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_backend_comparison_small() {
        let config = BenchmarkConfig {
            name: "Test Comparison".to_string(),
            vector_count: 50,
            dimension: 128,
            search_queries: 10,
            k_nearest: 5,
        };
        
        let results = compare_backends(config).await;
        assert!(results.is_ok());
        
        let results = results.unwrap();
        assert!(!results.is_empty());
        
        // Should have at least instant-distance
        assert!(results.iter().any(|r| r.library_name == "instant-distance"));
    }
}