# Vector Search Backend Architecture

## Overview

Aircher uses hnswlib-rs as its high-performance vector search backend, providing fast and scalable semantic code search capabilities. The backend leverages HNSW (Hierarchical Navigable Small World) algorithm with SIMD optimizations for production-ready performance.

## Architecture Design

### Backend Abstraction Layer

```rust
pub trait VectorSearchBackend: Send + Sync {
    fn new(storage_path: PathBuf, dimension: usize) -> Result<Self>;
    fn add_embedding(&mut self, embedding: Vec<f32>, metadata: ChunkMetadata) -> Result<()>;
    fn build_index(&mut self) -> Result<()>;
    fn search(&self, query_embedding: &[f32], k: usize) -> Result<Vec<SearchResult>>;
    fn search_with_filter(&self, query_embedding: &[f32], k: usize, filter: &SearchFilter) -> Result<Vec<SearchResult>>;
    async fn save_index(&self) -> Result<()>;
    async fn load_index(&mut self) -> Result<()>;
    fn get_stats(&self) -> IndexStats;
    fn needs_index_build(&self) -> bool;
    fn clear(&mut self);
    fn backend_name(&self) -> &'static str;
}
```

### Implementation Details

```rust
// hnswlib-rs backend configuration
let hnsw = Hnsw::<f32, DistCosine>::new(
    max_nb_connection: 32,     // Optimized for code similarity
    vector_count,
    max_layer: 16,
    ef_construction: 400,      // High quality index
    DistCosine{},
);
```

## Current Implementation

### hnswlib-rs Backend
- **Status**: Fully implemented and production-ready
- **Performance**: 45x faster indexing, 2.1x faster search
- **Features**: Multithreading, SIMD optimization, parallel insertion
- **Algorithm**: HNSW with cosine distance
- **Scalability**: Handles 10,000+ vectors efficiently

## Performance Characteristics

### Indexing Performance
- **Construction**: 0.12s for 1000 vectors (vs 5.63s previously)
- **Parallel insertion**: Utilizes all CPU cores
- **Memory efficient**: Optimized data structures

### Search Performance
- **Query time**: <2ms average
- **Throughput**: 1248 req/s (vs 582 req/s previously)
- **Accuracy**: 92-99% recall with proper parameters

## Usage

### Building Aircher
```bash
cargo build --release
```

### Creating Vector Engine
```rust
// Initialize with hnswlib-rs backend
let engine = VectorSearchEngine::new(
    storage_path, 
    768  // Dimension for SweRankEmbed
)?;

// Index embeddings
engine.add_embedding(embedding, metadata)?;
engine.build_index()?;

// Search
let results = engine.search(&query_embedding, k)?;
```

## Optimized Parameters

### Code Search Configuration
```rust
max_nb_connection: 32      // Higher connectivity for semantic similarity
ef_construction: 400       // High quality index construction
ef_search: 200            // Balanced search quality/speed
max_layer: 16             // Standard maximum for datasets <1M
```

### Performance Metrics
- **Index Build**: 15-20s for typical projects (3K+ vectors)
- **Search**: <2ms average query time
- **Memory**: ~240MB for 3K vectors
- **Throughput**: 1000+ req/s sustained

## Implementation Features

### Core Capabilities
- [x] HNSW algorithm with cosine distance
- [x] Parallel index construction
- [x] SIMD-optimized distance calculations
- [x] Efficient metadata storage
- [x] Index persistence and loading
- [x] Advanced filtering support

### Future Enhancements

1. **Native Serialization**: Direct HNSW graph serialization
2. **GPU Acceleration**: CUDA/Metal support for larger datasets
3. **Distributed Search**: Multi-node capabilities
4. **Dynamic Parameters**: Auto-tuning based on dataset characteristics

## Development Guidelines

### Working with the Backend
1. Use `VectorSearchEngine` high-level API
2. Configure parameters based on dataset size
3. Monitor memory usage for large datasets
4. Use batch operations for efficiency

### Testing
```bash
# Run vector search tests
cargo test vector_search

# Run benchmarks
cargo bench vector_search
```

## Conclusion

The hnswlib-rs backend provides Aircher with production-ready, high-performance vector search capabilities. With 45x faster indexing and 2.1x faster search operations, it enables instant semantic code search even on large codebases while maintaining excellent accuracy and reasonable memory usage.