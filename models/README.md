# Aircher Models Cache Directory

This directory caches downloaded embedding models for semantic code search.

## Model Selection Strategy

Aircher uses a **choose-your-model** approach:
- 🎯 **First run**: Interactive model selection with license information
- 🔄 **Anytime**: Switch models via `aircher model install <name>`
- ⚖️ **Commercial safe**: Defaults to Apache 2.0/MIT licensed models

## Available Models

### Commercial-Compatible Options
- **all-MiniLM-L6-v2** (90MB) - Apache 2.0 - Default, fast startup (~200ms)
- **gte-large** (670MB) - Apache 2.0 - Premium quality (~800ms)

### Research/Personal Use
- **SweRankEmbed-Small** (260MB) - CC BY-NC 4.0 - Best for code, non-commercial only

## How Models Are Managed

### Automatic Download
```bash
aircher                    # First run shows model selection
aircher model install <name>  # Install specific model
```

### Manual Management
```bash
aircher model list         # Show available models
aircher model current      # Show active model
aircher model remove <name> # Remove cached model
```

### Storage Location
Models are cached in:
- **Linux/macOS**: `~/.cache/aircher/models/`
- **Windows**: `%LOCALAPPDATA%/aircher/models/`

## No Model? No Problem!

Aircher works without any models using basic text search. Embedding models enhance search with semantic understanding but aren't required for core functionality.

## License Compliance

Run `aircher model check-license` to verify your configuration meets your licensing requirements.