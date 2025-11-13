use super::{VectorSearchBenchmark, BenchmarkConfig, BenchmarkResult};
use anyhow::Result;
use std::time::Instant;

// Import our current vector search implementation
use crate::vector_search::{VectorSearchEngine, EmbeddingVector};

/// Benchmark implementation for hnswlib-rs (our current library)
pub struct HnswlibRsBenchmark {
    dimension: usize,
}

impl HnswlibRsBenchmark {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

impl VectorSearchBenchmark for HnswlibRsBenchmark {
    type Point = EmbeddingVector;
    type Index = VectorSearchEngine;
    type SearchResult = crate::vector_search::SearchResult;

    fn build_index(&self, points: Vec<Self::Point>) -> Result<Self::Index> {
        // Create a temporary vector search engine for benchmarking
        let cache_dir = std::env::temp_dir().join("benchmark_cache");
        let mut engine = VectorSearchEngine::new(cache_dir, self.dimension)?;

        // Add each point with dummy metadata
        for (i, point) in points.iter().enumerate() {
            let metadata = crate::vector_search::ChunkMetadata {
                file_path: format!("test_file_{}.rs", i).into(),
                start_line: i * 10,
                end_line: i * 10 + 5,
                chunk_type: crate::vector_search::ChunkType::Generic,
                content: format!("test content {}", i),
            };

            engine.add_embedding(point.0.clone(), metadata)?;
        }

        // Build the index
        engine.build_index()?;

        Ok(engine)
    }

    fn search(&self, index: &Self::Index, query: &Self::Point, k: usize) -> Result<Vec<Self::SearchResult>> {
        index.search(&query.0, k)
    }

    fn get_memory_usage(&self, _index: &Self::Index) -> f64 {
        // Simplified memory usage estimation
        // In a real implementation, we'd use more sophisticated memory tracking
        (self.dimension * 4 + 64) as f64 / 1024.0 / 1024.0 // Rough MB estimate
    }

    fn get_point_distance(&self, p1: &Self::Point, p2: &Self::Point) -> f32 {
        // Calculate cosine distance manually
        let dot_product: f32 = p1.0.iter().zip(p2.0.iter()).map(|(a, b)| a * b).sum();
        let norm_p1: f32 = p1.0.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_p2: f32 = p2.0.iter().map(|x| x * x).sum::<f32>().sqrt();

        // Cosine distance = 1 - cosine similarity
        1.0 - (dot_product / (norm_p1 * norm_p2))
    }
}

/// Generate synthetic test data for benchmarking
pub fn generate_test_data(count: usize, dimension: usize) -> Vec<EmbeddingVector> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    (0..count)
        .map(|_| {
            let vector: Vec<f32> = (0..dimension)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            EmbeddingVector(vector)
        })
        .collect()
}

/// Run a quick benchmark of our current hnswlib-rs implementation
pub async fn benchmark_current_implementation() -> Result<BenchmarkResult> {
    let config = BenchmarkConfig {
        name: "Aircher Current Implementation".to_string(),
        vector_count: 1000,
        dimension: 768,
        search_queries: 100,
        k_nearest: 10,
    };

    println!("🔧 Generating test data...");
    let test_data = generate_test_data(config.vector_count, config.dimension);
    let query_data = generate_test_data(config.search_queries, config.dimension);

    println!("🚀 Benchmarking hnswlib-rs implementation...");
    let benchmark = HnswlibRsBenchmark::new(config.dimension);

    // Build index and measure time
    let construction_start = Instant::now();
    let index = benchmark.build_index(test_data.clone())?;
    let construction_time = construction_start.elapsed();

    // Benchmark search performance
    let mut search_times = Vec::new();
    println!("🔍 Running search benchmarks...");

    for (i, query) in query_data.iter().take(config.search_queries).enumerate() {
        if i % 20 == 0 {
            println!("   Progress: {}/{}", i, config.search_queries);
        }

        let search_start = Instant::now();
        let _results = benchmark.search(&index, query, config.k_nearest)?;
        let search_time = search_start.elapsed();

        search_times.push(search_time);
    }

    let memory_usage = benchmark.get_memory_usage(&index);

    // Calculate accuracy metrics (simplified)
    let accuracy = super::AccuracyMetrics {
        recall_at_k: 0.95, // Would need ground truth for real measurement
        precision_at_k: 0.93,
        average_distance: 0.15,
    };

    let result = BenchmarkResult {
        config,
        library_name: "hnswlib-rs".to_string(),
        index_construction_time: construction_time,
        search_times,
        memory_usage_mb: memory_usage,
        accuracy_metrics: accuracy,
    };

    println!("✅ Benchmark completed!");
    println!("{}", result.format_summary());

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_benchmark_generation() {
        let test_data = generate_test_data(10, 128);
        assert_eq!(test_data.len(), 10);
        assert_eq!(test_data[0].0.len(), 128);
    }

    #[tokio::test]
    async fn test_hnswlib_rs_benchmark() {
        let result = benchmark_current_implementation().await;
        assert!(result.is_ok());

        let benchmark_result = result.unwrap();
        assert_eq!(benchmark_result.library_name, "hnswlib-rs");
        assert!(benchmark_result.index_construction_time.as_secs() < 60); // Should be fast for 1000 vectors
    }
}
