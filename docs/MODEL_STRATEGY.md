# Aircher Model Handling Strategy

## Overview

Aircher uses Elastic License 2.0, which allows free use but prevents companies from offering Aircher as a managed service. This document outlines our strategy for handling embedding models with various licenses.

## Model Licensing Compatibility

### Aircher's License: Elastic License 2.0
- ✅ **Allows**: Free use, modification, distribution
- ❌ **Prevents**: Offering as a managed service, removing license notices
- 🎯 **Goal**: Protect against cloud providers monetizing without contributing back

### Compatible Embedding Models
1. **Apache 2.0 Licensed** (Default)
   - `all-MiniLM-L6-v2` (90MB) - Fast, efficient
   - `all-mpnet-base-v2` (420MB) - Higher quality
   - `thenlper/gte-small` (130MB) - Good balance

2. **MIT Licensed**
   - `BAAI/bge-small-en-v1.5` (130MB) - Balanced performance

3. **CC BY-NC 4.0** (Optional, with warnings)
   - `SweRankEmbed-Small` (260MB) - Best for code, non-commercial only

## Implementation Strategy

### 1. First-Run Model Selection
```
aircher
├─ Detects no model configured
├─ Shows interactive selection:
│  ├─ Lists models with licenses
│  ├─ Recommends based on use case
│  └─ Downloads selected model
└─ Saves configuration
```

### 2. Model Management CLI
```bash
# Check current model
aircher model current

# List available models
aircher model list

# Switch models
aircher model install all-MiniLM-L6-v2
aircher model switch all-mpnet-base-v2

# Remove models
aircher model remove SweRankEmbed-Small
```

### 3. Configuration Storage
```toml
# ~/.config/aircher/config.toml
[embedding]
model = "all-MiniLM-L6-v2"
path = "~/.cache/aircher/models/all-MiniLM-L6-v2"

[models.all-MiniLM-L6-v2]
license = "Apache 2.0"
size_mb = 90
downloaded = "2024-01-20"
```

### 4. Download Sources
- **Primary**: HuggingFace Hub (official source)
- **Mirror**: GitHub Releases (for reliability)
- **Cache**: Local ~/.cache/aircher/models/

### 5. Commercial Use Safeguards
1. Default to Apache/MIT licensed models
2. Show clear warnings for CC BY-NC models
3. Track model usage in configuration
4. Provide license compliance check command

## User Experience Flow

### New User
1. Install Aircher
2. Run `aircher` for first time
3. See model selection prompt with licenses
4. Choose model (defaults to Apache 2.0)
5. Model downloads automatically
6. Ready to use!

### Existing User
1. Update Aircher
2. Existing model continues working
3. Can switch models anytime via CLI
4. Clear licensing information always visible

### Commercial User
1. Run `aircher model check-license`
2. Warns if non-commercial models present
3. Suggests commercial-friendly alternatives
4. One command to switch: `aircher model install-commercial`

## Technical Implementation

### Model Storage
```
~/.cache/aircher/
├── models/
│   ├── all-MiniLM-L6-v2/
│   │   ├── model.safetensors
│   │   └── config.json
│   └── checksums.json
└── config/
    └── models.toml
```

### Download Process
1. Check local cache first
2. Verify checksums if exists
3. Download from HuggingFace if needed
4. Show progress bar during download
5. Verify download integrity
6. Extract and configure

### Fallback Behavior
- No model? Use basic text search
- Model corrupted? Re-download automatically
- Network issues? Offer offline alternatives
- Ollama available? Suggest local models

## Benefits of This Approach

1. **User Freedom**: Choose model based on needs
2. **Legal Clarity**: Clear licensing information
3. **Zero Friction**: Works immediately after install
4. **Commercial Safe**: Defaults protect business users
5. **Future Proof**: Easy to add new models

## Implementation Timeline

1. **Phase 1**: Model selection on first run ✓
2. **Phase 2**: CLI model management
3. **Phase 3**: Automatic model updates
4. **Phase 4**: Model performance benchmarks

## FAQ

**Q: Why not bundle models?**
A: Keeps download small, respects licenses, allows user choice

**Q: What about offline users?**
A: Provide downloadable model packs, support Ollama integration

**Q: Can I use my own models?**
A: Yes! Point to any ONNX/SafeTensors model via config

**Q: How do updates work?**
A: Models update independently of Aircher, check with `aircher model update`