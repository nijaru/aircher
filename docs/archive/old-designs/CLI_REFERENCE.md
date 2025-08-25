# Aircher CLI Reference

Complete command reference for Aircher AI terminal assistant.

## Global Options

```bash
aircher [OPTIONS] [COMMAND|MESSAGE]

OPTIONS:
    -h, --help              Show help information
    -V, --version           Show version information
    -v, --verbose           Enable verbose output
    -q, --quiet             Suppress non-error output
    --config <PATH>         Use custom config file
    --provider <NAME>       Override default provider (claude, openai, gemini, ollama, openrouter)
    --model <NAME>          Override default model
    --json                  Output in JSON format (for scripting)
```

## Primary Modes

### Interactive TUI (Default)
```bash
aircher                     # Launch rich terminal UI
```

### One-Shot Chat
```bash
aircher "your message here" # Direct chat response
aircher "find error handling in this codebase"  # Semantic search + chat
```

## Configuration Management

### aircher config
```bash
aircher config <SUBCOMMAND>

SUBCOMMANDS:
    show                    Show all configuration
    get <KEY>              Get specific configuration value
    set <KEY> <VALUE>      Set configuration value
    unset <KEY>            Remove configuration value
    edit                   Open config file in $EDITOR
    reset                  Reset to default configuration
    validate               Check configuration validity
    path                   Show config file path

EXAMPLES:
    aircher config show
    aircher config get ui.theme
    aircher config set ui.theme dark
    aircher config set providers.claude.api_key sk-xxx
    aircher config edit
```

### Configuration Keys
```
providers.claude.api_key           # Claude API key
providers.claude.default_model     # Default Claude model
providers.openai.api_key           # OpenAI API key
providers.openai.default_model     # Default OpenAI model
providers.gemini.api_key           # Gemini API key
providers.ollama.base_url          # Ollama server URL
ui.theme                           # UI theme (dark/light)
ui.default_interface               # Default interface (tui/cli)
embedding.provider                 # Embedding provider (ollama/embedded/api)
embedding.model                    # Embedding model name
search.max_results                 # Max search results
search.auto_index                  # Auto-index on startup
```

## Model Management

### aircher model
```bash
aircher model <SUBCOMMAND>

SUBCOMMANDS:
    current                 Show currently configured models
    list                   List all available models
    select                 Interactive model selection (opens TUI modal)
    test                   Test model connections
    providers              Show available providers and their models
    info <MODEL>           Show detailed model information
    benchmark              Compare model performance

OPTIONS:
    --provider <NAME>      Filter by provider
    --type <TYPE>          Filter by type (chat/embedding)
    --json                 Output in JSON format

EXAMPLES:
    aircher model current
    aircher model list --provider ollama
    aircher model test
    aircher model info claude-3-5-sonnet
```

## Semantic Search

### aircher search
```bash
aircher search <SUBCOMMAND>

SUBCOMMANDS:
    index [PATH]           Index directory for semantic search
    query <QUERY>          Perform semantic code search
    stats [PATH]           Show search index statistics
    clear [PATH]           Clear search index

OPTIONS:
    --path <PATH>          Directory to search (default: .)
    --limit <NUM>          Maximum results (default: 10)
    --threshold <FLOAT>    Similarity threshold (default: 0.3)
    --json                 Output in JSON format
    --force                Force re-indexing

EXAMPLES:
    aircher search index
    aircher search query "error handling patterns"
    aircher search query "database connection" --limit 20
    aircher search stats
```

## Session Management

### aircher session
```bash
aircher session <SUBCOMMAND>

SUBCOMMANDS:
    list                   List all sessions
    new <TITLE>            Create new session
    load <ID>              Load/resume existing session
    export <ID>            Export session to file
    delete <ID>            Delete session
    cleanup                Clean up old sessions
    stats                  Show session statistics

OPTIONS:
    --provider <NAME>      Filter by provider
    --format <FORMAT>      Export format (json/markdown/csv/plain)
    --output <FILE>        Output file for export
    --days <NUM>           Age filter for cleanup

EXAMPLES:
    aircher session list
    aircher session new "Feature development"
    aircher session export 123 --format markdown
    aircher session cleanup --days 30
```

## Embedding Management

### aircher embedding
```bash
aircher embedding <SUBCOMMAND>

SUBCOMMANDS:
    (default)              List available models with current marked
    list                   List available embedding models with current marked
    set <MODEL>            Set embedding model ('auto' for intelligent selection)
    verify [TEXT]          Verify current embedding model is working
    update                 Update embedding models to latest versions
    clean                  Clean up unused models and stale indices
    status                 Show storage usage and cleanup recommendations

OPTIONS:
    --check-only           Check for updates without installing (update)
    --models               Remove unused model versions (clean)
    --indices              Remove stale search indices (clean)
    --all                  Remove everything (clean)

EXAMPLES:
    aircher embedding                    # Show list with current marked
    aircher embedding set auto           # Intelligent auto-selection
    aircher embedding set swerank-embed-small
    aircher embedding verify "sample code"
    aircher embedding update --check-only
    aircher embedding clean --models --indices
```

## Help System

### Built-in Help
```bash
aircher --help                     # Global help
aircher config --help              # Config subcommand help
aircher search query --help        # Specific command help

# Each subcommand provides detailed help
aircher <command> --help
```

### Help Topics
```bash
aircher help                       # Show general help
aircher help config                # Configuration guide
aircher help models                # Model selection guide
aircher help search                # Semantic search guide
aircher help providers             # Provider setup guide
aircher help examples              # Usage examples
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0    | Success |
| 1    | General error |
| 2    | Configuration error |
| 3    | API/Network error |
| 4    | User cancelled |
| 5    | File/Permission error |

## Environment Variables

```bash
# API Keys (alternative to config file)
ANTHROPIC_API_KEY=sk-xxx           # Claude API key
OPENAI_API_KEY=sk-xxx              # OpenAI API key  
GOOGLE_API_KEY=xxx                 # Gemini API key
OPENROUTER_API_KEY=sk-xxx          # OpenRouter API key

# Behavior
AIRCHER_CONFIG_DIR=/custom/path    # Custom config directory
AIRCHER_DEFAULT_PROVIDER=ollama    # Default provider
AIRCHER_LOG_LEVEL=debug            # Logging level
EDITOR=vim                         # Editor for 'config edit'
```

## Configuration File Locations

### Standard Locations (Cross-platform)
```bash
# Linux/macOS
~/.aircher/config.toml             # Main configuration
~/.aircher/config/models.toml      # Model specifications
~/.aircher/cache/models/           # Downloaded models
~/.aircher/data/sessions.db        # Session database

# Windows  
%APPDATA%\aircher\config.toml
%LOCALAPPDATA%\aircher\cache\models\
%APPDATA%\aircher\sessions\
```

## Scripting Examples

### Automation Scripts
```bash
#!/bin/bash
# Automated setup script

# Configure API keys
aircher config set providers.claude.api_key "$CLAUDE_KEY"
aircher config set providers.openai.api_key "$OPENAI_KEY"

# Setup embedding models
aircher model select --non-interactive

# Index codebase
aircher search index --force

# Test configuration
aircher model test --json > model_status.json
```

### CI/CD Integration
```bash
# In CI pipeline
aircher search query "TODO" --json | jq '.results | length'
aircher session export latest --format json > session_report.json
```

## Keyboard Shortcuts (TUI Mode)

| Key | Action |
|-----|--------|
| Ctrl+C | Exit application |
| Tab | Open model selection |
| F1 | Help panel |
| F2 | Settings panel |
| Enter | Send message |
| Up/Down | Scroll chat history |
| Esc | Close modals |

## Troubleshooting

### Common Issues
```bash
# Configuration problems
aircher config validate             # Check config validity
aircher config path                # Show config file location

# API connection issues  
aircher model test                 # Test all providers
aircher model test --provider claude

# Search not working
aircher search stats               # Check index status
aircher search index --force       # Rebuild index

# Model not found
aircher model list                 # Show available models
aircher model providers            # Show provider status
```

### Debug Mode
```bash
# Enable verbose logging
aircher --verbose config show
AIRCHER_LOG_LEVEL=debug aircher search query "patterns"
```

This comprehensive reference covers all current functionality for the Aircher CLI interface. All documented commands are now implemented and functional.

## Implementation Status
✅ **Complete**: All subcommands (config, search, model, embedding, session) are fully implemented and integrated into the main CLI parser. TUI-first behavior is now default.