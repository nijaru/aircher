# XDG Base Directory Specification Compliance

Aircher follows the XDG Base Directory Specification for consistent, cross-platform file organization.

## Directory Structure

### Configuration Files
- **Location**: `$XDG_CONFIG_HOME/aircher/` or `~/.config/aircher/`
- **Contents**: 
  - `config.toml` - Main configuration
  - `auth.json` - API key storage (obfuscated)
  - `permissions.toml` - Command permissions

### Data Files  
- **Location**: `$XDG_DATA_HOME/aircher/` or `~/.local/share/aircher/`
- **Contents**:
  - `sessions.db` - Session history database
  - `knowledge.db` - Knowledge base
  - `file_index.db` - File indexing data
  - `model_registry.json` - Embedding model registry

### Cache Files
- **Location**: `$XDG_CACHE_HOME/aircher/` or `~/.cache/aircher/`
- **Contents**:
  - Temporary files
  - Performance caches
  - Downloadable content

## Project-Local Directories

Project-specific data remains in `.aircher/` within each project:
- `.aircher/config.toml` - Project-specific configuration
- `.aircher/AGENT.md` - AI assistant configuration
- `.aircher/sessions/` - Project session data
- `.aircher/intelligence/` - Cached analysis

## Environment Variables

You can override default locations:
```bash
export XDG_CONFIG_HOME=/custom/config
export XDG_DATA_HOME=/custom/data  
export XDG_CACHE_HOME=/custom/cache
```

## Migration from Legacy Paths

If you have data in old locations:
- `~/.aircher/` → `~/.local/share/aircher/`
- `~/Library/Application Support/aircher/` → `~/.config/aircher/`

Simply move the files to the new XDG-compliant locations.

## Benefits

1. **Consistency**: Same paths on Linux, macOS, and other Unix-like systems
2. **Backup-friendly**: Easy to backup all config files from `~/.config/`
3. **Clean home directory**: No more hidden directories in `~/`
4. **Standard compliance**: Works with XDG-aware tools and scripts

## Implementation

Aircher uses a custom `XdgDirs` utility that:
- Respects XDG environment variables when set
- Falls back to XDG defaults when not set
- Ensures directories exist before use
- Provides consistent API across the codebase