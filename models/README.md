# Aircher Models Directory

This directory contains embedding models for semantic code search.

## SweRankEmbed Model

The `swerank-embed-small.safetensors` model (260.81 MB) is required for semantic search functionality.

### Download Instructions

#### Option 1: Automatic Download (Recommended)
```bash
# From the project root:
./scripts/download-models.sh
```

#### Option 2: Manual Download
1. Download the model from: [URL placeholder - update with actual model location]
2. Place it in this directory as `swerank-embed-small.safetensors`

#### Option 3: Git LFS (For Contributors)
If you have Git LFS installed:
```bash
git lfs pull
```

### Model Details
- **Name**: SweRankEmbed-Small
- **Size**: 260.81 MB
- **Format**: SafeTensors
- **Purpose**: Code embedding for semantic search
- **Performance**: 74.45% on SWE-Bench-Lite

### Why Not in Git?
Files larger than 100 MB require Git LFS. To keep the repository accessible to all users, we provide the model through alternative download methods.