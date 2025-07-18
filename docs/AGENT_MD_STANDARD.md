# AGENT.md Standard Reference

## Overview
The AGENT.md standard is an emerging universal configuration file format for AI-powered coding tools. Multiple implementations are being developed to create a single, standardized approach that works across different AI development platforms.

## Purpose
- Create a universal configuration file that works across multiple AI coding tools
- Provide comprehensive guidance for how AI agents should interact with specific projects
- Solve the fragmentation problem of tool-specific configuration files

## Key Characteristics
- **Location**: Root directory of software projects
- **Format**: Markdown with structured sections
- **Scope**: Hierarchical configuration (supports subdirectory files)
- **Integration**: Supports @-mentions for referencing other files

## Recommended Sections
1. **Project Structure** - Organization and layout
2. **Build & Development** - Commands for building, testing, running
3. **Code Style** - Conventions and formatting rules
4. **Architecture** - Design patterns and architectural decisions
5. **Testing** - Testing strategies and guidelines
6. **Security** - Security considerations and best practices

## Implementation Status
- **Native Support**: Amp, Claude Code, Cursor, Gemini CLI, OpenAI Codex, Replit, Windsurf
- **Symbolic Links**: Backward compatibility with existing tool-specific configs
- **Migration Tools**: Commands available for converting legacy configuration files

## Key Benefits
- **Unified Voice**: Single configuration understood by all AI tools
- **Reduced Fragmentation**: Eliminates need for multiple tool-specific configs
- **Human-Readable**: Markdown format accessible to both humans and AI
- **Extensible**: Supports custom sections and project-specific requirements

## Aircher Implementation
Currently, Aircher uses `AIRCHER.md` (formerly `AGENT.md`) to avoid conflicts with the emerging standard while providing the same functionality. Users who prefer the universal standard can:

1. **Use AGENT.md**: Create `AGENT.md` following the standard format
2. **Symbolic Link**: Link `AGENT.md` to `AIRCHER.md` for compatibility
3. **Migration**: Use migration tools when available for seamless transition

## References
- Primary Standard: https://github.com/agentmd/agent.md
- Alternative Implementation: https://ampcode.com/AGENT.md
- Industry Adoption: Growing support across major AI development platforms

## Recommendation
Consider adopting the AGENT.md standard for maximum compatibility across AI development tools while maintaining Aircher-specific features through the `AIRCHER.md` approach.