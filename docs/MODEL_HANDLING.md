# Model Handling Strategy

## Overview

Aircher uses a 260MB SweRankEmbed model for semantic code search. Due to GitHub's 100MB file size limit, we use a hybrid approach that maintains the "seamless bundled install" experience while working within platform constraints.

## Strategy

### For Users (Seamless Experience)
1. **Release Binaries**: Include embedded model (increases binary size by ~260MB)
2. **Fallback Mode**: Hash-based embeddings work without the model file
3. **Zero Configuration**: Works out of the box, better with model

### For Developers (Flexible Development)
1. **Git LFS**: Track model file for easy collaboration
2. **Local File**: Use models/swerank-embed-small.safetensors during development
3. **Download Script**: Automated model acquisition

## Implementation

### Build Configuration
```rust
// build.rs detects model availability
if model_exists {
    cfg("has_bundled_model")
    if release_build {
        cfg("embed_model")  // Embed in binary for releases
    }
}
```

### Model Loading Priority
1. Check cache directory (~/.cache/aircher/models/)
2. If release build: Extract embedded model
3. If dev build: Copy from local models/ directory
4. Fallback: Use hash-based embeddings with warning

## Setup Instructions

### For Users
```bash
# Just works - model embedded in release builds
aircher search query "error handling"
```

### For Contributors
```bash
# Option 1: Git LFS (recommended)
brew install git-lfs
git lfs install
git lfs pull

# Option 2: Download manually
./scripts/download-models.sh

# Option 3: Place manually
# Download to: models/swerank-embed-small.safetensors
```

## File Locations

- **Development**: `models/swerank-embed-small.safetensors`
- **Runtime Cache**: `~/.cache/aircher/models/swerank-embed-small.safetensors`
- **Release Binary**: Embedded via `include_bytes!`

## Benefits

1. **Users**: Truly bundled, zero-config experience
2. **Developers**: Flexible model management
3. **CI/CD**: Can build without model (uses fallback)
4. **Releases**: Self-contained binaries

## Trade-offs

- **Binary Size**: Release binaries ~260MB larger
- **Build Time**: Slightly longer for release builds
- **Memory**: Model loaded once and cached

## Future Improvements

1. **Compression**: Use zstd to reduce model size
2. **Lazy Loading**: Load model only when needed
3. **Model Selection**: Support multiple model sizes
4. **Download on Demand**: First-use download option