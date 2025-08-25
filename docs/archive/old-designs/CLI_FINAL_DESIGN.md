# CLI Final Design - Aircher v0.1

**Status**: ✅ **Complete** - Enhanced model selection with final CLI refinements  
**Date**: 2025-01-18  
**Context**: Final CLI design after extensive user experience optimization and implementation

## 🎯 Core Design Principles

### 1. **Invisible Embedding Models**
- **90% of users** never touch embedding commands
- Auto-download SweRankEmbed-Small on first search
- Background indexing and lifecycle management
- Only power users need `aircher embedding` commands

### 2. **Chat Models = User Choice** 
- Users have strong preferences (Claude vs GPT personality)
- Enhanced TUI for Provider → Model → Host → Auth flow
- CLI commands for scriptability

### 3. **Standard CLI Patterns**
- Default commands show lists with current selection marked
- `set` commands change configuration
- `status` commands show detailed information
- Auto-setup with manual override options

## 🛠️ Final CLI Structure

### Chat Models (User-Facing)
```bash
# Primary workflows
aircher model                 # Enhanced TUI selection modal
aircher model set claude-3-5-sonnet  # Direct setting
aircher model list            # All available models
aircher model current         # Current configuration
aircher model test            # Test current model

# Advanced
aircher model select          # Same as default (TUI)
aircher model providers       # Provider status
```

### Search (Main User Workflow) 
```bash
# Core workflow - embedding setup is invisible
aircher search "query"        # Auto-index on first use, then search
aircher search stats          # Index status

# Manual controls (power users)
aircher search index          # Force re-index
aircher search index --watch  # Auto re-index on changes
```

### Embedding Models (Power Users Only)
```bash
# Most users never use these commands
aircher embedding            # List all with current marked ⭐
aircher embedding set auto   # Intelligent auto-selection
aircher embedding set swerank-embed-small  # Specific model
aircher embedding verify     # Verify current model works
aircher embedding status     # Storage usage/cleanup

# Lifecycle management
aircher embedding update     # Update to latest versions
aircher embedding clean      # Cleanup unused/stale files
```

### Configuration (Standard)
```bash
aircher config              # Show all configuration
aircher config get <key>    # Get specific value  
aircher config set <key> <value>  # Set specific value
aircher config edit         # Open in $EDITOR
```

## 🚀 User Experience Flow

### New User (Zero Config)
```bash
# First time using Aircher
cd my-project
aircher search "authentication"

# Behind the scenes:
# 1. Detects no embedding model
# 2. Downloads SweRankEmbed-Small (137MB, ~30s)
# 3. Auto-indexes project files  
# 4. Returns search results
# 5. All future searches are instant
```

### Chat Model Selection
```bash
# Interactive TUI with multi-level navigation
aircher model

# Flow: Provider → Model → Host → Auth
# ←/Esc (back), →/Enter (advance), ↑↓ (navigate)
# Fuzzy filtering for OpenRouter
# Auto-highlight defaults for quick selection
```

### Power User Embedding Management
```bash
# Check what's installed
aircher embedding
# Output:
# Available embedding models:
#   swerank-embed-small v1.0 (137MB, embedded) ← current
#   nomic-embed-text v2.1 (400MB, Ollama)

# Switch models
aircher embedding set nomic-embed-text
# Auto re-indexes with new model

# Cleanup old versions
aircher embedding clean --models --indices
# Freed: 520MB (2 unused models, 3 stale indices)
```

## 🔄 Lifecycle Management

### Model Updates
```bash
# Automatic update checking
aircher embedding update --check-only
# Output:
# 📦 Found 1 available update:
#   swerank-embed-small v1.0 → v1.1
#     📈 15% better code search accuracy, reduced size
#     📥 Download size: 120MB

# Apply updates
aircher embedding update
# Background download, seamless transition, auto re-index
```

### Storage Cleanup
```bash
# Storage overview
aircher embedding status
# Output:
# 📊 Embedding Storage Status:
# Current model: swerank-embed-small v1.1 (120MB)
# Storage usage: 2.1GB total
#   Models: 3 (1 unused)  
#   Indices: 8 (2 stale)
# 🧹 Cleanup recommendations:
#   1 unused model version can be removed (400MB)
#   2 stale indices can be removed (80MB)
```

## 🎯 Implementation Status

### ✅ Completed
- **Enhanced Model Selection TUI** - Multi-level navigation with fuzzy filtering
- **Embedding Lifecycle Manager** - Version tracking, updates, cleanup
- **CLI Integration** - All commands wired into main parser
- **Auto-Selection Engine** - Intelligent embedding model selection
- **Configuration Management** - TOML config with save/load
- **Final CLI Refinements** - Default list view, test→verify rename, standard patterns

### 🔄 Active Development  
- **Auto-Download System** - SweRankEmbed-Small download on first use
- **Background Indexing** - File watcher and incremental updates
- **Model Update Mechanism** - Remote version checking and seamless updates

### 🚀 Future Integration (Planned)
- **OmenDB Vector Database** - Replace file-based storage with high-performance embedded vector DB
  - 79% memory reduction vs traditional approaches
  - 0.37ms query latency (enterprise-grade performance)
  - RoarGraph algorithm for perfect accuracy
  - Clean Python API integration via PyO3
  - Zero external dependencies (embedded like SQLite)

### 📋 Next Steps
- Implement auto-download with progress bars
- Add background file watching for incremental indexing  
- Create model update registry and remote version checking
- Add download resume capability for interrupted downloads

## 🧠 Key Design Decisions

### Why Not Include SweRankEmbed-Small in Binary?
- **137MB** would make CLI massive for users who only want chat
- **Download UX** can be seamless with good progress indication
- **Updates** are easier with separate model files
- **Storage flexibility** for multiple model versions

### Why Separate Chat vs Embedding Commands?
- **Different user mental models**: personality choice vs performance optimization
- **Different use cases**: active selection vs background operation  
- **Different complexity**: simple choice vs system optimization

### Why Default Embedding Command Shows List?
- **Follows standard CLI patterns** (git branch, docker context)
- **Most common need**: "What are my options and what's current?"
- **Efficient workflow**: see options → set → verify

This CLI design balances simplicity for most users with power for advanced users, following standard CLI conventions while making embedding models as invisible as possible.