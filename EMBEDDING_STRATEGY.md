# Aircher Embedding Strategy

## 🎯 Strategy: Auto-install embeddinggemma

### Decision: Use Google's embeddinggemma as Default
- **Model**: `embeddinggemma:latest` (621MB)
- **Provider**: Ollama (seamless integration)
- **Auto-download**: Yes, on first search use
- **Fallback**: `nomic-embed-text` (274MB)

### Why embeddinggemma?
1. **Google Quality**: From Google's Gemma family, high-quality embeddings
2. **Code-optimized**: Designed for code understanding tasks
3. **Reasonable Size**: 621MB (acceptable for development tool)
4. **Ollama Native**: No complex API integrations

### Implementation Strategy

#### Phase 1: Seamless Auto-install ✅
```rust
// Default config now uses embeddinggemma with auto-download
EmbeddingConfig {
    preferred_model: "embeddinggemma",
    auto_download: true,
    use_ollama_if_available: true,
    fallback_model: Some("nomic-embed-text"),
}
```

#### Phase 2: Smart Persistence (Current)
- Models persist across Aircher versions (Ollama manages lifecycle)
- Global Ollama model cache shared across projects
- `ollama pull embeddinggemma` once, use everywhere

#### Phase 3: User Choice (Future)
```bash
aircher config set embedding.model nomic-embed-text  # Smaller/faster
aircher config set embedding.model mxbai-embed-large # Higher quality
```

## 🎯 Embedding Use Cases

### 1. Primary: Semantic Code Search
```bash
aircher search "authentication middleware patterns"
aircher search "error handling with Result<>"
aircher search "async function patterns"
```

### 2. Intelligence Context Injection
- Auto-include relevant code when user asks questions
- Find similar patterns across codebase for suggestions
- Conversation context enhancement

### 3. Multi-turn Context Memory
- Remember related code from previous conversation
- Build context across multiple interactions
- Project-wide understanding

### 4. Future: Advanced Features
- Architectural analysis and refactoring suggestions
- Similar component detection across large codebases
- Documentation and code alignment detection

## 📊 Performance Characteristics

### embeddinggemma Expected Performance
- **Quality**: High (Google Gemma family)
- **Speed**: Good (optimized for code)
- **Size**: 621MB (one-time download)
- **Memory**: ~1-2GB during embedding generation

### Fallback Chain
1. `embeddinggemma` (preferred, auto-download)
2. `nomic-embed-text` (fallback, smaller)
3. `swerank-embed-small` (hash-based placeholder)
4. Text search (no embeddings)

## 🚀 Auto-install UX Flow

### First Use Experience
```
$ aircher search "async functions"

🔍 Setting up semantic search...
📦 Downloading embeddinggemma (621MB)... ██████░░░░ 60%
✅ Ready! Searching for "async functions"...

Found 15 results in 0.3s:
  src/async_handler.rs:42  async fn handle_request()
  ...
```

### Subsequent Uses
```
$ aircher search "error patterns"
✅ Found 23 results in 0.1s  # Fast, using cached embeddings
```

## 🔧 Technical Implementation

### Model Persistence
- Ollama handles model lifecycle (survives Aircher updates)
- Models stored in Ollama's global model directory
- Shared across all Aircher projects
- Manual cleanup: `ollama rm embeddinggemma`

### Integration Points
1. **SemanticCodeSearch**: Primary embedding consumer
2. **IntelligenceEngine**: Context enhancement
3. **ConversationManager**: Multi-turn context
4. **Agent Tools**: Smart tool suggestions

### Error Handling
```rust
// Graceful degradation when embeddings fail
SearchResult::EmbeddingsFailed => {
    warn!("Embeddings unavailable, falling back to text search");
    self.text_search(query)
}
```

## 📈 Success Metrics

### Performance Targets
- **First search**: <30s (includes download)
- **Subsequent searches**: <0.5s
- **Index rebuild**: <2 minutes for large projects
- **Memory usage**: <2GB during indexing

### Quality Metrics
- Better semantic understanding than pure text search
- Finds conceptually similar code, not just keyword matches
- Handles synonyms and code patterns effectively

---

## Next Steps

1. ✅ Configure embeddinggemma as default
2. ✅ Enable auto-download in default config
3. 🔄 Test integration with actual semantic search
4. 📋 Add progress indicators for model download
5. 📋 Implement smart fallback chain