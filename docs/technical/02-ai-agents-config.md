# AI Agents Configuration

This file defines the supported AI agents and their configuration files for the aircher tool. When running `/init` or similar commands, aircher will check for these files and create/update them as needed.

## Supported AI Agents

### GitHub Copilot
- **Primary File**: `.github/copilot-instructions.md`
- **Type**: Markdown redirect
- **Strategy**: Redirect to AGENTS.md
- **Detection**: Presence of `.github/` directory or GitHub repository

### Claude Code
- **Primary File**: `CLAUDE.md`
- **Type**: Markdown redirect
- **Strategy**: Redirect to AGENTS.md
- **Detection**: Manual creation or explicit Claude usage

### OpenAI Codex CLI
- **Primary File**: `AGENTS.md`
- **Type**: Master navigation hub
- **Strategy**: Primary source of truth
- **Detection**: Always present (master file)

### Cursor
- **Primary File**: `.cursor/rules/ruler_cursor_instructions.md`
- **Type**: Markdown redirect
- **Strategy**: Redirect to AGENTS.md
- **Detection**: `.cursor/` directory or Cursor editor usage

### Windsurf
- **Primary File**: `.windsurf/rules/ruler_windsurf_instructions.md`
- **Type**: Markdown redirect
- **Strategy**: Redirect to AGENTS.md
- **Detection**: `.windsurf/` directory or Windsurf editor usage

### Cline
- **Primary File**: `.clinerules`
- **Type**: Plain text redirect
- **Strategy**: Plain text redirect to AGENTS.md
- **Detection**: Cline extension or explicit usage

### Aider
- **Primary File**: `ruler_aider_instructions.md`
- **Secondary File**: `.aider.conf.yml`
- **Type**: Markdown + YAML config
- **Strategy**: Redirect + configuration
- **Detection**: `.aider/` directory or aider command usage

### Firebase Studio
- **Primary File**: `.idx/airules.md`
- **Type**: Markdown redirect
- **Strategy**: Redirect to AGENTS.md
- **Detection**: `.idx/` directory or Firebase Studio usage

### Open Hands
- **Primary File**: `.openhands/microagents/repo.md`
- **Secondary File**: `.openhands/config.toml`
- **Type**: Markdown + TOML config
- **Strategy**: Redirect + configuration
- **Detection**: `.openhands/` directory

## Configuration Templates

### Redirect Template (Markdown)
```markdown
# {AGENT_NAME} AI Development Guide

This project uses `AGENTS.md` as the primary AI navigation hub for all AI assistants, including {AGENT_NAME}.

**Please refer to [`AGENTS.md`](RELATIVE_PATH_TO_AGENTS.md) for:**
- Project overview and context
- Essential documentation files
- Task-to-documentation mapping
- AI workflow patterns
- Development guidelines

The `AGENTS.md` file is designed to work with multiple AI agents and contains all the guidance you need for effective development collaboration.

---

**Start here:** [`AGENTS.md`](RELATIVE_PATH_TO_AGENTS.md)
```

### Redirect Template (Plain Text)
```
# {AGENT_NAME} AI Development Guide

This project uses AGENTS.md as the primary AI navigation hub for all AI assistants, including {AGENT_NAME}.

Please refer to AGENTS.md for:
- Project overview and context
- Essential documentation files
- Task-to-documentation mapping
- AI workflow patterns
- Development guidelines

The AGENTS.md file is designed to work with multiple AI agents and contains all the guidance you need for effective development collaboration.

---

Start here: AGENTS.md
```

## Aircher Tool Integration

### Initialization Behavior
When aircher runs `/init` or detects a new project:

1. **Check for existing AGENTS.md**: If not present, create from template
2. **Scan for agent directories**: Check for `.github/`, `.cursor/`, `.windsurf/`, etc.
3. **Create redirect files**: Generate appropriate redirect files for detected agents
4. **Update .gitignore**: Ensure agent files are appropriately tracked
5. **Validate structure**: Ensure all redirects point to valid AGENTS.md

### Detection Logic
```yaml
detection_patterns:
  github_copilot:
    - ".github/"
    - "copilot-workspace.yml"
    - "copilot-instructions.md"
  
  cursor:
    - ".cursor/"
    - ".cursorrules"
    - "cursor.yml"
  
  windsurf:
    - ".windsurf/"
    - "windsurf.yml"
  
  cline:
    - ".clinerules"
    - ".cline/"
  
  aider:
    - ".aider/"
    - ".aider.conf.yml"
    - "aider-instructions.md"
  
  firebase_studio:
    - ".idx/"
    - "idx.yml"
  
  openhands:
    - ".openhands/"
    - "openhands.yml"
```

### File Creation Priority
1. **AGENTS.md** - Always created/updated first (master file)
2. **Agent-specific files** - Created based on detection or explicit request
3. **Configuration files** - Created for agents that require them (Aider, Open Hands)

### Maintenance Strategy
- **Single Source of Truth**: AGENTS.md contains all actual content
- **Redirect Pattern**: All other agent files redirect to AGENTS.md
- **Consistency**: Updates to AGENTS.md automatically benefit all agents
- **Minimal Duplication**: Avoid copying content, use redirects instead

## Implementation Notes

### Path Resolution
- Use relative paths in redirects to ensure they work in different environments
- Account for different directory depths (`.github/` vs `.cursor/rules/`)
- Test path resolution in various project structures

### Content Synchronization
- Consider implementing a sync command to update all redirect files
- Monitor AGENTS.md for changes and offer to update redirects
- Validate that all redirects point to existing AGENTS.md

### Agent-Specific Customization
- Some agents may require specific formatting or additional content
- Consider allowing agent-specific overrides while maintaining redirect pattern
- Support for agent-specific configuration files (YAML, TOML, etc.)

## Usage Examples

### Initialize all detected agents
```bash
aircher init --agents=auto
```

### Initialize specific agents
```bash
aircher init --agents=cursor,claude,github
```

### Update all agent files
```bash
aircher sync-agents
```

### Validate agent configuration
```bash
aircher validate-agents
```

## Benefits

1. **Consistency**: All agents get the same high-quality development guidance
2. **Maintenance**: Single file to update (AGENTS.md) instead of multiple duplicates
3. **Flexibility**: Easy to add new agents by creating redirect files
4. **Compatibility**: Works with existing agent file conventions
5. **Discovery**: Automatic detection of agent usage in projects
6. **Standards**: Enforces consistent AI development patterns across tools

---

This configuration enables aircher to provide unified AI agent support while respecting each tool's specific file conventions and requirements.