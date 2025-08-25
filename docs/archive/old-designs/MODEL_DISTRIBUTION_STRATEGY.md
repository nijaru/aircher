# Model Distribution & Versioning Strategy

## Cache Structure
```
~/.cache/aircher/
├── models/
│   ├── all-MiniLM-L6-v2/
│   │   ├── model.safetensors
│   │   ├── config.json
│   │   └── .version
│   ├── gte-large/
│   │   ├── model.safetensors
│   │   ├── config.json
│   │   └── .version
│   └── checksums.json
├── indices/
│   └── project-specific-indices/
└── config/
    └── model-preferences.json
```

## Versioning System
- **Model versions**: Semantic versioning in `.version` files
- **Checksums**: SHA256 verification in `checksums.json`
- **Update detection**: Compare local vs remote versions
- **Backwards compatibility**: Keep old versions during transitions

## Distribution Options

### Option 1: GitHub Releases (Recommended)
**Pros:**
- ✅ Free bandwidth and storage
- ✅ Built-in versioning with release tags
- ✅ Global CDN via GitHub's infrastructure
- ✅ Easy CI/CD integration
- ✅ Transparent download metrics

**Cons:**
- ❌ 2GB file size limit per release asset
- ❌ Dependent on GitHub availability

**Implementation:**
```
github.com/nijaru/aircher/releases/download/models-v1.0.0/
├── all-MiniLM-L6-v2.tar.gz (90MB)
├── gte-large.tar.gz (670MB)  
├── checksums.sha256
└── manifest.json
```

### Option 2: HuggingFace Hub (Best for ML)
**Pros:**
- ✅ Designed specifically for ML models
- ✅ Built-in versioning and model cards
- ✅ Automatic chunking for large files
- ✅ Industry standard for AI models
- ✅ Git LFS handling built-in

**Cons:**
- ❌ Additional dependency
- ❌ Potential rate limiting

**Implementation:**
```rust
use hf_hub::api::tokio::Api;

async fn download_model(model_name: &str) -> Result<PathBuf> {
    let api = Api::new()?;
    let repo = api.model("aircher-ai/embedding-models".to_string());
    let filename = format!("{}.safetensors", model_name);
    let local_path = cache_dir().join("models").join(model_name).join("model.safetensors");
    repo.get(&filename).await?;
    Ok(local_path)
}
```

### Option 3: Hybrid Approach (Optimal)
- **Small models** (< 100MB): GitHub Releases
- **Large models** (> 100MB): HuggingFace Hub
- **Fallback mirrors**: Both sources for redundancy

## Download Strategy

### Progressive Enhancement
1. **Check cache first**: Use existing models
2. **Version check**: Compare with remote manifest  
3. **Smart downloading**: Only download if needed
4. **Progress indication**: Show download progress
5. **Verification**: Checksum validation
6. **Atomic updates**: Download to temp, then move

### Implementation
```rust
pub struct ModelManager {
    cache_dir: PathBuf,
    manifest: RemoteManifest,
}

impl ModelManager {
    pub async fn ensure_model(&self, model: &EmbeddingModel) -> Result<PathBuf> {
        let model_dir = self.cache_dir.join("models").join(&model.name);
        
        // Check if current version exists
        if self.is_model_current(&model_dir, &model.version).await? {
            return Ok(model_dir.join("model.safetensors"));
        }
        
        // Download new version
        self.download_model_with_progress(model).await
    }
    
    async fn download_model_with_progress(&self, model: &EmbeddingModel) -> Result<PathBuf> {
        let progress = ProgressBar::new(model.size_bytes);
        
        match model.size_mb {
            size if size < 100 => self.download_from_github(model, &progress).await,
            _ => self.download_from_huggingface(model, &progress).await,
        }
    }
}
```

## Security & Integrity

### Checksum Verification
```rust
pub async fn verify_model(path: &Path, expected_hash: &str) -> Result<bool> {
    let mut hasher = Sha256::new();
    let mut file = File::open(path).await?;
    let mut buffer = vec![0; 8192];
    
    while let Ok(n) = file.read(&mut buffer).await {
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    
    let hash = format!("{:x}", hasher.finalize());
    Ok(hash == expected_hash)
}
```

### Signature Verification (Future)
- GPG-signed manifest files
- Model authenticity verification
- Supply chain security

## Bandwidth Optimization

### Smart Caching
- **Global cache**: Share models between projects
- **Compression**: gzip/zstd for transport
- **Resumable downloads**: Handle network interruptions
- **Delta updates**: Only download changes (future)

### CDN Strategy
- **Primary**: GitHub Releases CDN
- **Secondary**: HuggingFace Hub
- **Mirror**: Self-hosted backup (if needed)

## Update Mechanism

### Automatic Updates
```bash
# Check for model updates
aircher model check-updates

# Update specific model
aircher model update all-MiniLM-L6-v2

# Update all models
aircher model update --all
```

### Version Management
```json
{
  "manifest_version": "1.0.0",
  "models": {
    "all-MiniLM-L6-v2": {
      "version": "1.2.0",
      "size_bytes": 94371840,
      "sha256": "abc123...",
      "download_urls": {
        "github": "https://github.com/nijaru/aircher/releases/download/...",
        "huggingface": "https://huggingface.co/aircher-ai/embedding-models/resolve/main/..."
      },
      "license": "Apache-2.0",
      "quality_score": 85
    }
  }
}
```

## Recommended Implementation

1. **Start simple**: GitHub Releases for initial models
2. **Add HuggingFace**: For larger models as needed
3. **Implement versioning**: Checksum-based integrity
4. **Add progress bars**: Better UX during downloads
5. **Build update system**: Check for newer models

## Benefits

- ✅ **User-friendly**: No permission issues
- ✅ **Reliable**: Multiple sources with fallbacks  
- ✅ **Secure**: Checksum verification
- ✅ **Efficient**: Smart caching and updates
- ✅ **Transparent**: Clear versioning and provenance
- ✅ **Cost-effective**: Leverages free infrastructure