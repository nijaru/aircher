# Aircher Documentation

**Intelligent ACP-compatible coding agent backend**

## Quick Start

```bash
# Install dependencies
uv sync --dev

# Test installation
uv run aircher hello

# Run tests
uv run pytest tests/

# Start development
uv run aircher --help
```

## Architecture

### Core Components
- **Agent**: LangGraph-based intelligence engine
- **Memory**: 3-layer system (SQLite + DuckDB + ChromaDB)
- **Protocol**: ACP-compatible communication
- **Tools**: File operations, code analysis, system integration
- **Modes**: READ/EDIT/TURBO safety levels

### Memory System
1. **SQLite**: Session management, conversation history
2. **DuckDB**: Analytics, episodic memory, performance data
3. **ChromaDB**: Vector search, semantic similarity

## Development

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

### Commands
- `uv sync --dev` - Install dependencies
- `uv run ruff check . --fix` - Lint and format
- `uv run mypy src/` - Type checking
- `uv run pytest tests/` - Run tests

## API Reference

### Agent Modes
- **READ**: Safe file reading and analysis
- **EDIT**: File modification with confirmation
- **TURBO**: Full access with --bypass flag

### ACP Protocol
JSON-RPC over stdio for universal frontend compatibility.

## Configuration

Environment variables (prefix `AIRCHER_`):
- `AGENT_MODE` - Default agent mode (read/edit/turbo)
- `DEFAULT_MODEL` - LLM model to use
- `OPENAI_API_KEY` - OpenAI API key
- `ANTHROPIC_API_KEY` - Anthropic API key
- `DATA_DIR` - Data directory path

## Performance

Targets:
- **Response Time**: <100ms p95 for simple queries
- **Memory Usage**: <1GB for typical workloads
- **Context Window**: 10K+ tokens with smart pruning

## Contributing

1. Follow code style (ruff, mypy)
2. Add tests for new features
3. Update documentation
4. Use conventional commits

## License

MIT License - see LICENSE file
