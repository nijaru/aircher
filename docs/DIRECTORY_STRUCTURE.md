# Aircher Directory Structure

Aircher uses a simple, conventional directory structure similar to other developer tools.

## Main Directory: `~/.aircher`

All global Aircher data is stored under `~/.aircher` in your home directory. This approach provides:
- Simplicity and clarity
- Easy access and backup
- Consistency with tools like `~/.ssh`, `~/.aws`, `~/.cargo`

### Directory Layout

```
~/.aircher/
├── config.toml      # Main configuration
├── auth.json        # API keys (obfuscated)
├── config/          # Additional configuration files
│   ├── permissions.toml  # Command permissions
│   └── presets/     # Model presets (future)
├── data/            # Application data
│   ├── sessions.db  # Session history
│   ├── knowledge.db # Knowledge base
│   └── model_registry.json # Model registry
├── cache/           # Temporary/cache files
│   └── indices/     # Search indices
└── logs/            # Application logs
    └── aircher.log  # Main log file
```

## Project-Local Directories

Project-specific data remains in `.aircher/` within each project:
```
.aircher/
├── config.toml      # Project-specific configuration
├── AGENT.md         # AI assistant configuration
├── sessions/        # Project session data
└── intelligence/    # Cached analysis
```

## Environment Variables

You can override the base directory if needed:
```bash
export AIRCHER_HOME=/custom/path/aircher
```

## Benefits of Simple Structure

1. **Discoverability**: Easy to find and understand
2. **Backup-friendly**: Single directory to backup
3. **Portability**: Easy to move between systems
4. **Convention**: Follows common developer tool patterns

## Implementation

Aircher uses the `AircherDirs` utility that:
- Provides consistent paths across the codebase
- Ensures directories exist before use
- Supports environment variable overrides