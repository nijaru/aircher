# Aircher CLI Command Specifications

## Overview

The Aircher CLI provides a comprehensive interface for AI-powered development assistance. The tool supports multiple LLM providers, project analysis, context management, and extensible tool integration through the Model Context Protocol (MCP).

## Command Structure

```
aircher [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGS]
```

## Global Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--config` | `-c` | Path to configuration file | `~/.config/aircher/config.toml` |
| `--verbose` | `-v` | Enable verbose logging | `false` |
| `--quiet` | `-q` | Suppress non-essential output | `false` |
| `--no-color` | | Disable colored output | `false` |
| `--help` | `-h` | Show help information | |
| `--version` | `-V` | Show version information | |

## Core Commands

### 1. Interactive Chat (`chat`)

Start an interactive conversation session with an AI assistant.

```bash
aircher chat [OPTIONS]
```

#### Options
- `--provider, -p <PROVIDER>` - Select LLM provider (openai, claude, gemini, ollama)
- `--model, -m <MODEL>` - Specify model to use
- `--system <MESSAGE>` - Set system message
- `--context <PATH>` - Include file or directory in context
- `--max-tokens <NUM>` - Maximum tokens for response
- `--temperature <NUM>` - Response creativity (0.0-2.0)
- `--stream` - Enable response streaming (default: true)
- `--no-stream` - Disable response streaming

#### Examples
```bash
# Start interactive chat with default provider
aircher chat

# Use specific provider and model
aircher chat -p claude -m claude-3-5-sonnet-20241022

# Include project context
aircher chat --context ./src --context ./docs

# Set system message
aircher chat --system "You are a Rust expert helping with code review"
```

### 2. Authentication (`auth`)

Manage API keys and authentication for LLM providers.

```bash
aircher auth <SUBCOMMAND>
```

#### Subcommands

##### `login`
Add or update provider credentials.

```bash
aircher auth login <PROVIDER> [OPTIONS]
```

**Options:**
- `--api-key <KEY>` - API key for the provider
- `--base-url <URL>` - Custom base URL (for OpenAI-compatible APIs)
- `--organization <ORG>` - Organization ID (OpenAI)

**Examples:**
```bash
# Add OpenAI credentials
aircher auth login openai --api-key sk-...

# Add Claude credentials
aircher auth login claude --api-key sk-ant-...

# Add custom OpenAI-compatible endpoint
aircher auth login openrouter --api-key sk-or-... --base-url https://openrouter.ai/api/v1
```

##### `logout`
Remove provider credentials.

```bash
aircher auth logout <PROVIDER>
```

##### `list`
Show configured providers.

```bash
aircher auth list [OPTIONS]
```

**Options:**
- `--show-keys` - Show masked API keys

##### `test`
Test connection to providers.

```bash
aircher auth test [PROVIDER]
```

### 3. Project Analysis (`analyze`)

Analyze project structure and generate documentation.

```bash
aircher analyze [OPTIONS] [PATH]
```

#### Options
- `--output, -o <PATH>` - Output file path (default: `.aircher/project_analysis.md`)
- `--format <FORMAT>` - Output format (markdown, json, yaml)
- `--include <PATTERN>` - Include files matching pattern
- `--exclude <PATTERN>` - Exclude files matching pattern
- `--depth <NUM>` - Maximum directory depth to analyze
- `--update` - Update existing analysis file
- `--interactive` - Interactive analysis with AI insights

#### Examples
```bash
# Analyze current directory
aircher analyze

# Analyze specific directory with custom output
aircher analyze ./src -o ./docs/analysis.md

# Generate JSON analysis
aircher analyze --format json

# Interactive analysis with AI insights
aircher analyze --interactive
```

### 4. Configuration (`config`)

Manage Aircher configuration settings.

```bash
aircher config <SUBCOMMAND>
```

#### Subcommands

##### `get`
Retrieve configuration values.

```bash
aircher config get [KEY]
```

##### `set`
Set configuration values.

```bash
aircher config set <KEY> <VALUE>
```

##### `list`
List all configuration settings.

```bash
aircher config list [OPTIONS]
```

**Options:**
- `--defaults` - Show default values
- `--sources` - Show configuration sources

##### `edit`
Open configuration file in editor.

```bash
aircher config edit
```

##### `validate`
Validate configuration file.

```bash
aircher config validate [PATH]
```

#### Configuration Keys

| Key | Description | Type | Default |
|-----|-------------|------|---------|
| `default_provider` | Default LLM provider | String | `openai` |
| `default_model` | Default model for provider | String | Provider-specific |
| `max_context_files` | Maximum files in context | Integer | `50` |
| `auto_save_conversations` | Auto-save chat sessions | Boolean | `true` |
| `stream_responses` | Enable response streaming | Boolean | `true` |
| `color_theme` | TUI color theme | String | `default` |

### 5. Context Management (`context`)

Manage conversation context and file relevance.

```bash
aircher context <SUBCOMMAND>
```

#### Subcommands

##### `add`
Add files or directories to context.

```bash
aircher context add <PATH> [OPTIONS]
```

**Options:**
- `--weight <NUM>` - Context weight (1-10)
- `--type <TYPE>` - Context type (code, docs, config, data)

##### `remove`
Remove items from context.

```bash
aircher context remove <PATH>
```

##### `list`
Show current context items.

```bash
aircher context list [OPTIONS]
```

**Options:**
- `--verbose` - Show detailed information
- `--sort <FIELD>` - Sort by field (relevance, size, date)

##### `clear`
Clear all context items.

```bash
aircher context clear
```

##### `optimize`
Optimize context based on relevance scoring.

```bash
aircher context optimize [OPTIONS]
```

**Options:**
- `--max-files <NUM>` - Maximum files to keep
- `--min-relevance <NUM>` - Minimum relevance score (0.0-1.0)

### 6. Conversation History (`history`)

Manage conversation sessions and history.

```bash
aircher history <SUBCOMMAND>
```

#### Subcommands

##### `list`
List conversation sessions.

```bash
aircher history list [OPTIONS]
```

**Options:**
- `--limit <NUM>` - Number of sessions to show
- `--format <FORMAT>` - Output format (table, json, yaml)

##### `show`
Show conversation details.

```bash
aircher history show <SESSION_ID>
```

##### `delete`
Delete conversation sessions.

```bash
aircher history delete <SESSION_ID>
```

##### `export`
Export conversation history.

```bash
aircher history export [OPTIONS] <PATH>
```

**Options:**
- `--format <FORMAT>` - Export format (json, markdown, txt)
- `--sessions <IDS>` - Specific session IDs to export

### 7. MCP Tools (`mcp`)

Manage Model Context Protocol tools and servers.

```bash
aircher mcp <SUBCOMMAND>
```

#### Subcommands

##### `list`
List available MCP tools and servers.

```bash
aircher mcp list [OPTIONS]
```

**Options:**
- `--servers` - Show only servers
- `--tools` - Show only tools

##### `install`
Install MCP server or tool.

```bash
aircher mcp install <NAME> [OPTIONS]
```

**Options:**
- `--source <URL>` - Installation source
- `--version <VERSION>` - Specific version

##### `remove`
Remove MCP server or tool.

```bash
aircher mcp remove <NAME>
```

##### `test`
Test MCP tool functionality.

```bash
aircher mcp test <TOOL_NAME>
```

### 8. Database Management (`db`)

Manage Aircher databases and storage.

```bash
aircher db <SUBCOMMAND>
```

#### Subcommands

##### `migrate`
Run database migrations.

```bash
aircher db migrate [OPTIONS]
```

**Options:**
- `--target <VERSION>` - Target migration version
- `--dry-run` - Show migrations without applying

##### `backup`
Create database backup.

```bash
aircher db backup <PATH>
```

##### `restore`
Restore database from backup.

```bash
aircher db restore <PATH>
```

##### `status`
Show database status and statistics.

```bash
aircher db status
```

##### `cleanup`
Clean up old data and optimize databases.

```bash
aircher db cleanup [OPTIONS]
```

**Options:**
- `--days <NUM>` - Keep data newer than N days
- `--vacuum` - Run VACUUM to reclaim space

## Command Chaining and Pipes

Aircher supports command chaining and Unix-style pipes for complex workflows:

```bash
# Analyze project and start chat with context
aircher analyze --interactive | aircher chat --context -

# Export conversation and analyze with external tools
aircher history export --format json | jq '.messages[] | select(.role == "assistant")'
```

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Authentication error |
| 4 | Network error |
| 5 | API error |
| 6 | File system error |
| 130 | Interrupted by user (Ctrl+C) |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `AIRCHER_CONFIG` | Configuration file path | `~/.config/aircher/config.toml` |
| `AIRCHER_DATA_DIR` | Data directory path | `~/.local/share/aircher` |
| `AIRCHER_LOG_LEVEL` | Log level (error, warn, info, debug, trace) | `info` |
| `AIRCHER_NO_COLOR` | Disable colored output | `false` |
| `OPENAI_API_KEY` | OpenAI API key | |
| `ANTHROPIC_API_KEY` | Claude API key | |
| `GOOGLE_API_KEY` | Gemini API key | |

## Shell Completion

Generate shell completion scripts:

```bash
# Bash
aircher completion bash > /etc/bash_completion.d/aircher

# Zsh
aircher completion zsh > ~/.zsh/completions/_aircher

# Fish
aircher completion fish > ~/.config/fish/completions/aircher.fish

# PowerShell
aircher completion powershell > $PROFILE
```

## Interactive Mode Features

When running `aircher chat`, the interactive mode provides:

- **Vim-style navigation**: `j/k` for scrolling, `/` for search
- **Command palette**: `:` to access commands
- **Context management**: Tab to manage conversation context
- **Model switching**: `Ctrl+M` to switch models mid-conversation
- **Export options**: `Ctrl+E` to export conversation
- **Help system**: `?` or `F1` for help

## Security Considerations

- API keys are stored encrypted in the system keyring
- Configuration files use secure permissions (0600)
- Network requests use TLS with certificate validation
- MCP tools run in sandboxed environments
- File system access respects `.gitignore` and `.aicignore` patterns