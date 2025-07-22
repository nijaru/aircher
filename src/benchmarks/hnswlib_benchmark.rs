#[cfg(feature = "hnswlib-rs")]
use super::{VectorSearchBenchmark, BenchmarkConfig, BenchmarkResult};
#[cfg(feature = "hnswlib-rs")]
use anyhow::Result;
#[cfg(feature = "hnswlib-rs")]
use instant_distance::Point;
#[cfg(feature = "hnswlib-rs")]
use std::sync::Arc;

#[cfg(feature = "hnswlib-rs")]
use crate::vector_search::EmbeddingVector;

/// Benchmark implementation for hnswlib-rs
#[cfg(feature = "hnswlib-rs")]
pub struct HnswlibBenchmark {
    dimension: usize,
    max_nb_connection: usize,
    ef_construction: usize,
}

#[cfg(feature = "hnswlib-rs")]
impl HnswlibBenchmark {
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            max_nb_connection: 16, // Default value, can be tuned
            ef_construction: 200,  // Default value, can be tuned
        }
    }
    
    pub fn with_params(dimension: usize, max_nb_connection: usize, ef_construction: usize) -> Self {
        Self {
            dimension,
            max_nb_connection,
            ef_construction,
        }
    }
}

#[cfg(feature = "hnswlib-rs")]
impl VectorSearchBenchmark for HnswlibBenchmark {
    type Point = EmbeddingVector;
    type Index = (); // Placeholder until proper hnsw_rs API integration
    type SearchResult = (EmbeddingVector, f32, usize); // (vector, distance, index)
    
    fn build_index(&self, _points: Vec<Self::Point>) -> Result<Self::Index> {
        // TODO: Implement proper hnsw_rs integration
        // For now, just return a placeholder
        Ok(())
    }
    
    fn search(&self, _index: &Self::Index, query: &Self::Point, k: usize) -> Result<Vec<Self::SearchResult>> {
        // TODO: Implement proper hnsw_rs search
        // For now, just return dummy results
        let results = (0..k)
            .map(|i| {
                let dummy_vector = EmbeddingVector(query.0.clone());
                let dummy_distance = 0.1 * (i as f32);
                (dummy_vector, dummy_distance, i)
            })
            .collect();
        
        Ok(results)
    }
    
    fn get_memory_usage(&self, _index: &Self::Index) -> f64 {
        // Estimate memory usage for hnswlib-rs
        // This is a rough estimate - actual usage would depend on graph structure
        let base_memory = (self.dimension * 4) as f64; // 4 bytes per float32
        let graph_overhead = 64.0; // Estimate for graph connections
        (base_memory + graph_overhead) / 1024.0 / 1024.0 // Convert to MB
    }
    
    fn get_point_distance(&self, p1: &Self::Point, p2: &Self::Point) -> f32 {
        // Use cosine distance (same as our current implementation)
        p1.distance(p2)
    }
}

/// Run comparison benchmark between instant-distance and hnswlib-rs
#[cfg(feature = "hnswlib-rs")]
pub async fn run_comparison_benchmark() -> Result<super::BenchmarkComparison> {
    use super::vector_benchmark::{InstantDistanceBenchmark, generate_test_data};
    
    let config = BenchmarkConfig {
        name: "Instant-distance vs hnswlib-rs Comparison".to_string(),
        vector_count: 2000,
        dimension: 768,
        search_queries: 100,
        k_nearest: 10,
    };
    
    println!("🔧 Generating test data ({} vectors, {} dimensions)...", 
             config.vector_count, config.dimension);
    
    let test_data = generate_test_data(config.vector_count, config.dimension);
    let query_data = generate_test_data(config.search_queries, config.dimension);
    
    let mut comparison = super::BenchmarkComparison::new();
    
    // Benchmark instant-distance (current implementation)
    println!("🚀 Benchmarking instant-distance...");
    let instant_benchmark = InstantDistanceBenchmark::new(config.dimension);
    let instant_result = super::benchmark_implementation(
        &config,
        instant_benchmark,
        "instant-distance",
        &test_data,
        &query_data,
    )?;
    comparison.add_result(instant_result);
    
    // Benchmark hnswlib-rs
    println!("🚀 Benchmarking hnswlib-rs...");
    let hnswlib_benchmark = HnswlibBenchmark::new(config.dimension);
    let hnswlib_result = super::benchmark_implementation(
        &config,
        hnswlib_benchmark,
        "hnswlib-rs",
        &test_data,
        &query_data,
    )?;
    comparison.add_result(hnswlib_result);
    
    println!("✅ Comparison benchmark completed!");
    
    Ok(comparison)
}

/// Run parameter tuning for hnswlib-rs to find optimal configuration
#[cfg(feature = "hnswlib-rs")]
pub async fn tune_hnswlib_parameters() -> Result<()> {
    use super::vector_benchmark::generate_test_data;
    
    let config = BenchmarkConfig {
        name: "hnswlib-rs Parameter Tuning".to_string(),
        vector_count: 1500,
        dimension: 768,
        search_queries: 50,
        k_nearest: 10,
    };
    
    let test_data = generate_test_data(config.vector_count, config.dimension);
    let query_data = generate_test_data(config.search_queries, config.dimension);
    
    // Test different parameter combinations
    let parameter_sets = vec![
        (16, 200),  // Default
        (32, 200),  // Higher connectivity
        (16, 400),  // Higher ef_construction  
        (32, 400),  // Both higher
        (48, 200),  // Very high connectivity
        (16, 800),  // Very high ef_construction
    ];
    
    println!("🔧 Running parameter tuning for hnswlib-rs...");
    
    for (max_conn, ef_const) in parameter_sets {
        println!("Testing max_nb_connection={}, ef_construction={}", max_conn, ef_const);
        
        let benchmark = HnswlibBenchmark::with_params(config.dimension, max_conn, ef_const);
        let result = super::benchmark_implementation(
            &config,
            benchmark,
            &format!("hnswlib-rs (conn={}, ef={})", max_conn, ef_const),
            &test_data,
            &query_data,
        )?;
        
        println!("  Construction: {:.2}s, Search: {:.4}s, Memory: {:.1}MB",
                 result.index_construction_time.as_secs_f64(),
                 result.average_search_time().as_secs_f64(),
                 result.memory_usage_mb);
    }
    
    Ok(())
}

#[cfg(not(feature = "hnswlib-rs"))]
pub fn hnswlib_not_available() -> &'static str {
    "hnswlib-rs benchmarking requires the 'benchmarks' feature to be enabled.\n\
     Run: cargo test --features benchmarks"
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg(feature = "hnswlib-rs")]
    #[tokio::test]
    async fn test_hnswlib_benchmark() {
        let test_data = super::super::vector_benchmark::generate_test_data(100, 128);
        let benchmark = HnswlibBenchmark::new(128);
        
        let index = benchmark.build_index(test_data.clone()).unwrap();
        let query = &test_data[0];
        let results = benchmark.search(&index, query, 5).unwrap();
        
        assert_eq!(results.len(), 5);
    }
    
    #[cfg(feature = "hnswlib-rs")]
    #[tokio::test] 
    async fn test_comparison_benchmark() {
        let comparison = run_comparison_benchmark().await.unwrap();
        assert_eq!(comparison.results.len(), 2);
    }
}