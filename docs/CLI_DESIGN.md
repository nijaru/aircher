# Aircher CLI Design

Comprehensive command-line interface design following modern CLI best practices and user expectations.

## Design Philosophy

**TUI-First with CLI Scripting Support**
- Default: Interactive TUI (like successful tools: Gemini CLI, OpenCode)
- One-shot: Natural language input for quick queries
- Subcommands: Domain-specific automation and configuration
- Config-driven: All settings stored in TOML files

## Command Structure

### Primary Interface (95% of usage)
```bash
aircher                          # → TUI (default interface)
```

### One-Shot Mode (Quick tasks)
```bash
aircher "hello world"            # → Direct chat response
aircher "find error handling"    # → Semantic search + chat
aircher --model gpt-4 "explain rust ownership"
aircher --provider ollama "local model query"
```

### Domain Subcommands (Automation/Scripting)

#### Configuration Management
```bash
aircher config show             # Show current configuration
aircher config get ui.theme     # Get specific value
aircher config set ui.theme dark
aircher config set providers.claude.api_key sk-xxx
aircher config edit             # Open config.toml in $EDITOR
aircher config reset            # Reset to defaults
```

#### Model Management  
```bash
aircher model current           # Show active chat + embedding models
aircher model select            # Interactive model selection (TUI modal)
aircher model test              # Test all configured models
aircher model providers         # List available providers + models
```

#### Semantic Search
```bash
aircher search index            # Index current directory
aircher search query "patterns" # Semantic code search
aircher search stats            # Show search index statistics
```

#### Session Management
```bash
aircher session list           # List all sessions
aircher session new "title"    # Create new session
aircher session export 123     # Export session
```

## Configuration System

### File Locations (Cross-platform)
```bash
# Linux/macOS (XDG compliant)
~/.config/aircher/config.toml      # Main configuration
~/.config/aircher/models.toml      # Model specifications
~/.cache/aircher/models/           # Downloaded embedding models
~/.local/share/aircher/sessions/   # Session database

# Windows
%APPDATA%\aircher\config.toml
%LOCALAPPDATA%\aircher\cache\models\
%APPDATA%\aircher\sessions\
```

### Main Configuration (config.toml)
```toml
[providers.claude]
api_key = "sk-..."
default_model = "claude-3-5-sonnet-20241022"
enabled = true

[providers.ollama]
base_url = "http://localhost:11434"
default_model = "llama3.3"
enabled = true

[embedding]
provider = "ollama"              # ollama, embedded, or api
model = "nomic-embed-text"
auto_setup = true
similarity_threshold = 0.3

[search]
max_results = 10
auto_index = true
index_on_startup = false

[ui]
default_interface = "tui"        # tui, cli, or auto
theme = "dark"
auto_save_sessions = true

[session]
max_history_size = 1000
cleanup_old_sessions = true
max_session_age_days = 30
```

## UX Principles

### Natural Language Detection
```bash
# These are detected as chat messages (contain spaces/questions)
aircher "how do I..."
aircher "explain this code"
aircher "find authentication logic"

# These are detected as subcommands (single words)
aircher config
aircher search 
aircher session
```

### Flag Consistency
```bash
# Global flags (apply to any command)
--provider <name>         # Override default provider
--model <name>           # Override default model  
--config <path>          # Use custom config file
--verbose/-v             # Verbose output
--json                   # JSON output for scripting

# Command-specific flags
aircher search query "..." --limit 20 --json
aircher session export 123 --format markdown
```

### Help System
```bash
aircher --help              # Global help
aircher config --help       # Subcommand help
aircher search query --help # Sub-subcommand help
```

## Error Handling

### Graceful Degradation
- Missing API keys → Prompt for configuration with helpful guidance
- Embedding models unavailable → Fall back to text search
- Network issues → Show clear error messages with recovery suggestions

### User-Friendly Messages
```bash
❌ Claude API key not found
💡 Set it with: aircher config set providers.claude.api_key YOUR_KEY
   or set environment variable: export ANTHROPIC_API_KEY=YOUR_KEY

✅ Configuration updated successfully
🔍 Indexing 247 files for semantic search...
⚡ Found 8 results for "error handling patterns"
```

## Scripting Support

### Exit Codes
- 0: Success
- 1: General error
- 2: Configuration error
- 3: API/Network error
- 4: User cancelled

### JSON Output
```bash
aircher config get ui.theme --json
aircher search query "patterns" --json --limit 5
aircher session list --json
```

### Environment Variable Support
```bash
export ANTHROPIC_API_KEY=sk-xxx
export AIRCHER_CONFIG_DIR=/custom/path
export AIRCHER_DEFAULT_PROVIDER=ollama
```

## Comparison with Other Tools

### Follows Successful Patterns
- **Gemini CLI**: `gemini` → TUI default ✅
- **Claude Code**: `claude` → Interactive session ✅  
- **OpenCode**: `opencode` → TUI, `opencode run "..."` for one-shot ✅
- **Git**: Rich subcommands with config management ✅

### Improves Upon Limitations
- **GitHub Copilot**: Always requires subcommands (clunky) ❌
- **Complex CLIs**: Too many required flags ❌

## Implementation Priorities

1. **TUI as default** - Update CLI parser to launch TUI by default
2. **Natural language detection** - Smart parsing of "message" vs subcommand
3. **Config subcommand** - TOML file management
4. **Model subcommand** - Model selection and testing
5. **Search integration** - Wire existing search commands
6. **Documentation** - Update README and help text

This design provides a natural, discoverable interface that scales from casual use to power user scripting while following modern CLI conventions.