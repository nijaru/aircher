# Vector Search Backend Architecture

## Overview

Aircher implements a flexible dual-backend architecture for vector search, supporting both instant-distance (current) and hnswlib-rs (prototype) backends. This design enables zero-risk migration and performance optimization while maintaining backward compatibility.

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

### Backend Selection

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VectorBackend {
    InstantDistance,
    #[cfg(feature = "hnswlib-rs")]
    HnswRs,
}
```

### Feature Flags

```toml
[features]
default = ["instant-distance"]
instant-distance = []
hnswlib-rs = ["hnsw_rs"]
```

## Current Implementation Status

### instant-distance Backend (Production)
- **Status**: Fully implemented and production-ready
- **Performance**: 0.02s search after index build
- **Limitations**: 1000 vector limit for usability
- **Algorithm**: HNSW with cosine distance

### hnswlib-rs Backend (Prototype)
- **Status**: Architecture complete, placeholder implementation
- **Expected Performance**: 100-1000x improvement
- **Features**: Multithreading, SIMD optimization
- **Next Steps**: Replace placeholder with hnsw_rs integration

## Migration Strategy

### Phase 1: Dual Backend Support ✅
- Backend trait abstraction
- Feature-flagged implementations
- Unified VectorSearchEngine interface
- Zero-risk migration path

### Phase 2: Performance Validation (Next)
- Complete hnsw_rs integration
- Comprehensive benchmarking
- Parameter optimization
- A/B testing with real data

### Phase 3: Production Rollout
- Gradual migration with monitoring
- Performance metrics collection
- Fallback mechanisms
- Documentation updates

## Usage Examples

### Default Backend (instant-distance)
```bash
cargo build --release
```

### Enable hnswlib-rs Backend
```bash
cargo build --release --features hnswlib-rs
```

### Runtime Selection
```rust
// Use instant-distance
let engine = VectorSearchEngine::new_with_backend(
    storage_path, 768, VectorBackend::InstantDistance
)?;

// Use hnswlib-rs (when available)
#[cfg(feature = "hnswlib-rs")]
let engine = VectorSearchEngine::new_with_backend(
    storage_path, 768, VectorBackend::HnswRs
)?;
```

## Performance Expectations

### Current (instant-distance)
- Index Build: 10-15 seconds (3K vectors)
- Search: ~0.02s (after build)
- Memory: ~200MB
- Throughput: ~50 req/s

### Target (hnswlib-rs)
- Index Build: 3-7 seconds (50-75% faster)
- Search: <0.0002s (100x faster)
- Memory: ~240MB (20% overhead)
- Throughput: 12k-62k req/s

## Implementation Checklist

- [x] Backend trait abstraction
- [x] Dual backend enum support
- [x] Feature flag configuration
- [x] Unified VectorSearchEngine
- [x] instant-distance backend
- [x] hnswlib-rs prototype structure
- [ ] Complete hnsw_rs integration
- [ ] Performance benchmarking
- [ ] Parameter optimization
- [ ] Production validation

## Future Considerations

1. **Additional Backends**: Architecture supports adding more backends (e.g., Faiss, Annoy)
2. **GPU Acceleration**: Potential for CUDA/Metal backends
3. **Distributed Search**: Multi-node search capabilities
4. **Dynamic Selection**: Auto-select backend based on dataset size

## Development Guidelines

### Adding a New Backend
1. Implement the `VectorSearchBackend` trait
2. Add to `VectorBackend` enum
3. Update `VectorSearchEngine::new_with_backend`
4. Add feature flag if needed
5. Write comprehensive tests
6. Update documentation

### Testing Backends
```bash
# Test instant-distance
cargo test --features instant-distance

# Test hnswlib-rs
cargo test --features hnswlib-rs

# Test both
cargo test --all-features
```

## Conclusion

The dual backend architecture provides a robust foundation for vector search optimization while maintaining stability and backward compatibility. The modular design enables incremental improvements and future enhancements without disrupting existing functionality.