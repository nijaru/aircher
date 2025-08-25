# AGENT.md Workflow and Configuration

## Overview

Aircher implements a comprehensive AGENT.md workflow that follows industry best practices from Claude Code, Cursor, and other AI development tools. This document describes our implementation and how it compares to other tools.

## How It Works

### Automatic Loading
- AGENT.md is **automatically loaded** from `.aircher/AGENT.md` (or `./AGENT.md` fallback)
- Content is injected into **every conversation** as project instructions in the system prompt
- No manual intervention required - works seamlessly in the background

### File Hierarchy
Our system respects the standard AI agent configuration hierarchy:

1. **Global instructions**: `~/.agent/AGENT.md` (planned)
2. **Project instructions**: `.aircher/AGENT.md` (current implementation)
3. **Legacy formats**: `CLAUDE.md`, `.cursorrules`, `.copilot` (auto-detected)

## The `/init` Command

### What It Does
Similar to Claude Code's `/init`, our command:
- Analyzes your codebase structure and language
- Detects build tools and common commands
- Creates a comprehensive AGENT.md file
- Provides project-specific context for future AI sessions

### Example Output
```markdown
# MyProject Agent Configuration

This file provides context and instructions for AI agents working with this project.

## Project Overview

MyProject is a Rust project.

## Build Commands

- `cargo build`
- `cargo test`
- `cargo check`

## Development Guidelines

### Code Quality
- Write clean, readable code with proper error handling
- Follow existing code style and conventions
- Add tests for new functionality

### Architecture
- Maintain separation of concerns
- Use appropriate design patterns
- Document complex logic and decisions

## Notes for AI Agents

- Prioritize code quality and maintainability
- Consider performance implications of changes
- Preserve existing functionality unless explicitly changing it
- Ask for clarification when requirements are unclear
```

## Supported Configuration Files

Our system automatically discovers and loads:

### Primary Format
- **AGENT.md**: Our primary format (follows Claude Code conventions)

### Legacy/Alternative Formats
- **.cursorrules**: Cursor IDE agent instructions
- **.copilot**: GitHub Copilot instructions  
- **CLAUDE.md**: Legacy Claude-specific instructions
- **.gemini**: Gemini-specific instructions (if found)

## Integration Points

### System Prompt Injection
```rust
// In create_enhanced_system_prompt()
if let Some(project_instructions) = &ai_config.project_instructions {
    prompt.push_str("# Project Instructions\n\n");
    prompt.push_str(project_instructions);
    prompt.push_str("\n\n");
}
```

### Loading Process
```rust
// Automatic loading in intelligence/tui_tools.rs
if let Ok(agent_path) = self.project_manager.get_agent_config_path() {
    if agent_path.exists() {
        if let Ok(content) = self.read_file_safe(&agent_path.to_string_lossy()).await {
            config.project_instructions = Some(content);
        }
    }
}
```

## Comparison with Other Tools

### Claude Code
- ✅ **Similar**: `/init` command for project analysis
- ✅ **Similar**: Automatic loading in conversations
- ✅ **Similar**: Project-specific configuration
- ✅ **Better**: Multi-format support (not just CLAUDE.md)

### Cursor
- ✅ **Compatible**: Reads `.cursorrules` files
- ✅ **Similar**: Project-level configuration
- ✅ **Better**: Unified system (works with all formats)

### GitHub Copilot
- ✅ **Compatible**: Reads `.copilot` instructions
- ✅ **Better**: More comprehensive project analysis

## Best Practices

### For Users
1. Run `/init` in new projects to generate initial AGENT.md
2. Customize the generated file for your specific needs
3. Include project-specific conventions and requirements
4. Keep instructions concise but comprehensive

### For AI Agents
1. Always reference AGENT.md content when available
2. Follow project-specific guidelines over general practices
3. Respect build commands and testing procedures
4. Maintain consistency with project conventions

## Implementation Details

### File Locations
- **Primary**: `.aircher/AGENT.md` (created by `/init`)
- **Fallback**: `./AGENT.md` (if .aircher doesn't exist)
- **Legacy**: Various formats auto-detected in project root

### Language Detection
Supports automatic detection for:
- **Rust**: Cargo.toml → cargo build/test/check
- **Node.js**: package.json → npm install/build/test  
- **Python**: requirements.txt/pyproject.toml → pip/pytest
- **Go**: go.mod → go build/test

### Content Structure
Generated AGENT.md includes:
- Project overview and language
- Build and test commands
- Development guidelines
- Code quality standards
- Architecture principles
- AI-specific instructions

## Usage Examples

### Initialize New Project
```bash
# In your project directory
aircher
> /init
🎯 Analyzing your codebase to create AGENT.md configuration...
✅ Created AGENT.md at: /path/to/project/.aircher/AGENT.md
```

### Check Existing Configuration
```bash
# AGENT.md is automatically loaded - no action needed
# Content appears in all AI conversations
```

### Migrate from Other Tools
```bash
# Aircher automatically detects and loads:
# - .cursorrules (Cursor)
# - .copilot (GitHub Copilot)  
# - CLAUDE.md (legacy Claude)
# No migration needed!
```

## Future Enhancements

### Planned Features
- **Global AGENT.md**: `~/.agent/AGENT.md` for user-wide defaults
- **Template system**: Pre-built templates for common project types
- **Team sharing**: Shared configurations for team consistency
- **Validation**: Lint and validate AGENT.md content

### Integration Opportunities
- **IDE plugins**: Editor support for AGENT.md editing
- **CI/CD integration**: Validate AGENT.md in build pipelines
- **Team workflows**: Shared configurations via git

## Conclusion

Aircher's AGENT.md implementation provides a comprehensive, industry-standard approach to AI agent configuration. By supporting multiple formats and providing intelligent defaults, we make it easy for developers to get consistent AI assistance while respecting their existing workflows and preferences.

The automatic loading and `/init` command ensure that projects get proper AI context without manual setup, while the multi-format support means teams can adopt Aircher without losing their existing AI configurations.