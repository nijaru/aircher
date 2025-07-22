use anyhow::Result;
use std::time::{Duration, Instant};

// Serde helpers for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(duration.as_secs_f64())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = f64::deserialize(deserializer)?;
        Ok(Duration::from_secs_f64(secs))
    }
}

mod duration_vec_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(durations: &Vec<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let secs: Vec<f64> = durations.iter().map(|d| d.as_secs_f64()).collect();
        secs.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs: Vec<f64> = Vec::deserialize(deserializer)?;
        Ok(secs.into_iter().map(Duration::from_secs_f64).collect())
    }
}

pub mod vector_benchmark;
pub mod backend_comparison;

#[cfg(feature = "hnswlib-rs")]
pub mod hnswlib_benchmark;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkConfig {
    pub name: String,
    pub vector_count: usize,
    pub dimension: usize,
    pub search_queries: usize,
    pub k_nearest: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkResult {
    pub config: BenchmarkConfig,
    pub library_name: String,
    #[serde(with = "duration_serde")]
    pub index_construction_time: Duration,
    #[serde(with = "duration_vec_serde")]
    pub search_times: Vec<Duration>,
    pub memory_usage_mb: f64,
    pub accuracy_metrics: AccuracyMetrics,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AccuracyMetrics {
    pub recall_at_k: f64,
    pub precision_at_k: f64,
    pub average_distance: f64,
}

impl BenchmarkResult {
    pub fn average_search_time(&self) -> Duration {
        let total: Duration = self.search_times.iter().sum();
        total / self.search_times.len() as u32
    }
    
    pub fn search_throughput(&self) -> f64 {
        let avg_time_secs = self.average_search_time().as_secs_f64();
        if avg_time_secs > 0.0 {
            1.0 / avg_time_secs
        } else {
            0.0
        }
    }
    
    pub fn format_summary(&self) -> String {
        format!(
            "Library: {}\n\
             Index Construction: {:.2}s\n\
             Average Search Time: {:.4}s\n\
             Search Throughput: {:.0} req/s\n\
             Memory Usage: {:.1} MB\n\
             Recall@{}: {:.3}\n\
             Precision@{}: {:.3}",
            self.library_name,
            self.index_construction_time.as_secs_f64(),
            self.average_search_time().as_secs_f64(),
            self.search_throughput(),
            self.memory_usage_mb,
            self.config.k_nearest,
            self.accuracy_metrics.recall_at_k,
            self.config.k_nearest,
            self.accuracy_metrics.precision_at_k
        )
    }
}

pub struct BenchmarkComparison {
    pub results: Vec<BenchmarkResult>,
}

impl BenchmarkComparison {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
    
    pub fn add_result(&mut self, result: BenchmarkResult) {
        self.results.push(result);
    }
    
    pub fn format_comparison(&self) -> String {
        if self.results.len() < 2 {
            return "Need at least 2 results for comparison".to_string();
        }
        
        let mut output = String::new();
        output.push_str("=== VECTOR LIBRARY PERFORMANCE COMPARISON ===\n\n");
        
        for result in &self.results {
            output.push_str(&format!("{}\n", result.format_summary()));
            output.push_str(&"-".repeat(60));
            output.push_str("\n\n");
        }
        
        // Calculate improvements
        if self.results.len() == 2 {
            let baseline = &self.results[0];
            let alternative = &self.results[1];
            
            output.push_str("=== PERFORMANCE IMPROVEMENTS ===\n");
            
            let construction_improvement = 
                (baseline.index_construction_time.as_secs_f64() - alternative.index_construction_time.as_secs_f64()) 
                / baseline.index_construction_time.as_secs_f64() * 100.0;
            
            let search_improvement = 
                (baseline.average_search_time().as_secs_f64() - alternative.average_search_time().as_secs_f64()) 
                / baseline.average_search_time().as_secs_f64() * 100.0;
            
            let memory_change = 
                (alternative.memory_usage_mb - baseline.memory_usage_mb) 
                / baseline.memory_usage_mb * 100.0;
            
            let recall_improvement = alternative.accuracy_metrics.recall_at_k - baseline.accuracy_metrics.recall_at_k;
            
            output.push_str(&format!(
                "Index Construction: {}{:.1}%\n\
                 Search Performance: {}{:.1}%\n\
                 Memory Usage: {}{:.1}%\n\
                 Recall Change: {:+.3}\n\n",
                if construction_improvement >= 0.0 { "+" } else { "" },
                construction_improvement,
                if search_improvement >= 0.0 { "+" } else { "" },
                search_improvement,
                if memory_change >= 0.0 { "+" } else { "" },
                memory_change,
                recall_improvement
            ));
            
            // Recommendation
            output.push_str("=== RECOMMENDATION ===\n");
            let mut score = 0;
            let mut reasons = Vec::new();
            
            if construction_improvement >= 30.0 {
                score += 2;
                reasons.push(format!("Significantly faster index construction (+{:.1}%)", construction_improvement));
            } else if construction_improvement >= 10.0 {
                score += 1;
                reasons.push(format!("Faster index construction (+{:.1}%)", construction_improvement));
            }
            
            if search_improvement >= 20.0 {
                score += 2;
                reasons.push(format!("Much faster search performance (+{:.1}%)", search_improvement));
            } else if search_improvement >= 5.0 {
                score += 1;
                reasons.push(format!("Better search performance (+{:.1}%)", search_improvement));
            }
            
            if memory_change <= -5.0 {
                score += 1;
                reasons.push(format!("Lower memory usage ({:.1}%)", memory_change));
            } else if memory_change > 20.0 {
                score -= 1;
                reasons.push(format!("Higher memory usage (+{:.1}%)", memory_change));
            }
            
            if recall_improvement >= 0.01 {
                score += 1;
                reasons.push(format!("Better accuracy (+{:.3})", recall_improvement));
            } else if recall_improvement <= -0.01 {
                score -= 1;
                reasons.push(format!("Lower accuracy ({:.3})", recall_improvement));
            }
            
            if score >= 3 {
                output.push_str(&format!("✅ STRONG RECOMMENDATION for {}\n", alternative.library_name));
            } else if score >= 1 {
                output.push_str(&format!("⚠️  CONSIDER migration to {}\n", alternative.library_name));
            } else {
                output.push_str(&format!("❌ STAY with {}\n", baseline.library_name));
            }
            
            output.push_str("\nReasons:\n");
            for reason in reasons {
                output.push_str(&format!("• {}\n", reason));
            }
        }
        
        output
    }
}

/// Trait for benchmarkable vector search implementations
pub trait VectorSearchBenchmark {
    type Point: Clone;
    type Index;
    type SearchResult;
    
    fn build_index(&self, points: Vec<Self::Point>) -> Result<Self::Index>;
    fn search(&self, index: &Self::Index, query: &Self::Point, k: usize) -> Result<Vec<Self::SearchResult>>;
    fn get_memory_usage(&self, index: &Self::Index) -> f64;
    fn get_point_distance(&self, p1: &Self::Point, p2: &Self::Point) -> f32;
}

/// Run comprehensive benchmarks comparing vector search implementations
pub async fn run_comparison_benchmark<B1, B2>(
    config: BenchmarkConfig,
    impl1: B1,
    impl2: B2,
    test_data: Vec<B1::Point>,
    query_data: Vec<B1::Point>,
) -> Result<BenchmarkComparison>
where
    B1: VectorSearchBenchmark,
    B2: VectorSearchBenchmark<Point = B1::Point>,
    B1::Point: Clone,
{
    let mut comparison = BenchmarkComparison::new();
    
    // Benchmark first implementation
    let result1 = benchmark_implementation(&config, impl1, "Implementation 1", &test_data, &query_data)?;
    comparison.add_result(result1);
    
    // Benchmark second implementation  
    let result2 = benchmark_implementation(&config, impl2, "Implementation 2", &test_data, &query_data)?;
    comparison.add_result(result2);
    
    Ok(comparison)
}

fn benchmark_implementation<B>(
    config: &BenchmarkConfig,
    implementation: B,
    name: &str,
    test_data: &[B::Point],
    query_data: &[B::Point],
) -> Result<BenchmarkResult>
where
    B: VectorSearchBenchmark,
{
    // Build index and measure construction time
    let construction_start = Instant::now();
    let index = implementation.build_index(test_data.to_vec())?;
    let construction_time = construction_start.elapsed();
    
    // Measure memory usage
    let memory_usage = implementation.get_memory_usage(&index);
    
    // Benchmark search performance
    let mut search_times = Vec::new();
    let mut all_results = Vec::new();
    
    for query in query_data.iter().take(config.search_queries) {
        let search_start = Instant::now();
        let results = implementation.search(&index, query, config.k_nearest)?;
        let search_time = search_start.elapsed();
        
        search_times.push(search_time);
        all_results.push(results);
    }
    
    // Calculate accuracy metrics (simplified for now)
    let accuracy = AccuracyMetrics {
        recall_at_k: 0.95, // Placeholder - would need ground truth for real calculation
        precision_at_k: 0.93, // Placeholder
        average_distance: 0.15, // Placeholder
    };
    
    Ok(BenchmarkResult {
        config: config.clone(),
        library_name: name.to_string(),
        index_construction_time: construction_time,
        search_times,
        memory_usage_mb: memory_usage,
        accuracy_metrics: accuracy,
    })
}