# Aircher - Intelligent ACP-Compatible Coding Agent

**AI-powered coding agent backend with novel memory architecture and ACP protocol support**

## Quick Start

```bash
# Clone and setup
git clone https://github.com/nijaru/aircher.git
cd aircher

# Install dependencies
uv sync --dev

# Test installation
uv run aircher status

# Run tests
uv run pytest tests/
```

## What Is Aircher?

Aircher is an **intelligent coding agent** with novel memory architecture and ACP protocol support:

- **CLI interface** (development and testing)
- **ACP-compatible** (works with any ACP frontend)
- **Toad integration** (planned when released)
- **Bash/Code tools** (efficient, composable)
- **3-layer memory** (SQLite + DuckDB + ChromaDB)

**Architecture**: Python 3.13+ + LangGraph with modern tooling

## Features

### 🧠 **3-Layer Memory System**
- **SQLite**: Session management, conversation history
- **DuckDB**: Analytics, episodic memory, performance data
- **ChromaDB**: Vector search, semantic similarity

### 🔒 **READ/WRITE + --admin Safety System**
- **READ**: Safe mode, file reading only
- **WRITE**: File modification with confirmation
- **--admin**: Full access without confirmations

### 🛠️ **Smart Tool Management**
- **Bundled**: ripgrep, ast-grep (always available, latest versions)
- **Assumed**: fd, jq, sd, git (with fallbacks)
- **Python-based**: tree-sitter, PyYAML, toml
- **Zero Setup**: Works out of the box, no installation required

### 🔌 **Extensibility**
- **ACP Protocol**: Universal frontend compatibility
- **MCP Support**: Planned for extensibility
- **Hooks & Skills**: Custom commands like Claude Code
- **Mojo Ready**: Performance optimization path

### 📋 **Phased Development**
- **Phase 3-4**: Agent backend, CLI interface
- **Phase 5+**: Toad integration when released
- **Future**: Mojo for performance-critical components

## Development Status

**Phase 2 Complete**: Python project setup with modern tooling ✅

**Current Phase**: Core implementation (ACP protocol, LangGraph agent, memory systems)

**Progress**:
- ✅ Python project structure with uv, ruff, ty, vulture
- ✅ CI/CD pipeline with GitHub Actions
- ✅ Basic agent skeleton and mode system
- ✅ Modern tools integration strategy
- ✅ Bash/Code tools architecture
- ✅ Phased TUI approach (CLI now, Toad later)
- 🔄 ACP protocol implementation
- 🔄 LangGraph workflow creation
- 🔄 Memory system development

## Architecture

```
Frontend (any ACP-compatible)
        ↓ (Agent Client Protocol)
        ↓
    Aircher Backend (Python)
├── LangGraph Agent
├── 3-Layer Memory
├── Tool Framework
├── Mode Management
└── Configuration
```

## Development

### Commands
```bash
uv sync --dev              # Install dependencies
uv run aircher status      # Show system status
uv run pytest tests/       # Run tests
uv run ruff check . --fix  # Lint and format
uv run ty src/ --strict    # Type checking
uv run vulture src/        # Dead code detection
```

### Project Structure
```
src/aircher/
├── agent/         # LangGraph implementation
├── memory/        # Memory system interfaces
├── protocol/      # ACP protocol
├── tools/         # Tool framework
├── modes/         # Agent modes
└── config/        # Configuration
```

## AI Working Context

This project uses AI-optimized organization:

- **ai/STATUS.md** - Current implementation state
- **ai/TODO.md** - Active development tasks
- **ai/DECISIONS.md** - Architectural decisions
- **ai/RESEARCH.md** - Research findings

See [AGENTS.md](AGENTS.md) for complete AI context.

## Performance Targets

- **Response Time**: <100ms p95 for simple queries
- **Memory Usage**: <1GB for typical workloads
- **Context Window**: 10K+ tokens with smart pruning
- **Tool Execution**: <500ms for file operations

## Contributing

1. Follow code style (ruff, ty, vulture)
2. Add tests for new features
3. Update documentation
4. Use conventional commits

## License

MIT License - see [LICENSE](LICENSE) file

---

**Mission**: Build the smartest ACP-compatible agent backend through novel intelligence architecture and empirical validation.

**Current Status**: Phase 2 complete - Python project setup finished. Starting Phase 3 core implementation with READ/WRITE + --admin modes, modern tools integration, and phased TUI approach.
