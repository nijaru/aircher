# Aircher Agent Configuration

This file provides context and instructions for AI agents working with the Aircher project.

## Project Overview

Aircher is a production-ready AI-powered terminal assistant built with Rust. It features semantic code search, TUI interface with demo mode, and multi-provider AI chat capabilities.

## Current Status

- **Production ready** semantic search system with 99.9% speed improvement
- **TUI with graceful auth flow** - full interface available without API keys
- **Clean codebase** - reduced from ~190 compiler warnings to just 1
- **Comprehensive test coverage** with integration testing
- **Multi-provider support** for Claude, Gemini, OpenAI, OpenRouter, Ollama

## Key Features Completed

### 1. TUI Demo Mode (Recently Completed)
- Users can launch `aircher` immediately without any setup
- Interactive auth setup screen guides users through configuration
- Demo mode provides full semantic search and file monitoring
- Clear visual indicators distinguish demo vs. full functionality
- Graceful fallback when API keys are unavailable

### 2. Semantic Code Search
- Production-grade performance with index persistence
- 19+ language support via tree-sitter parsing
- Advanced filtering and query expansion
- Typo correction and synonym matching

### 3. Intelligence Engine
- Context-aware development assistance
- Background file monitoring
- Project structure analysis
- Smart context injection

## Development Guidelines

### Code Quality Standards
- Zero tolerance for compiler warnings (currently down to 1)
- Comprehensive error handling with graceful fallbacks
- Clear separation between core functionality and UI layers
- Proper resource management and cleanup

### Testing Standards
- Integration tests for all TUI workflows
- Mock implementations for external dependencies
- Performance testing for search operations
- Error condition testing

### Architecture Principles
- Pure Rust implementation, no system dependencies
- Graceful degradation when services unavailable
- User choice over prescriptive defaults
- Configuration hierarchy: hardcoded → global → local → environment

## Recent Major Changes

### TUI Auth Flow Implementation
- Modified `TuiManager::new_with_auth_state()` for graceful auth handling
- Added `SelectionModal::from_config()` for demo mode initialization
- Implemented auth setup screens with clear user guidance
- Enhanced error handling to prevent crashes on missing API keys

### Performance Optimizations
- Index persistence for 99.9% faster subsequent searches
- Batch embedding generation for 80% faster indexing
- Cooperative multitasking to prevent CPU spikes
- Proper resource cleanup and memory management

## Working With This Codebase

### Key Directories
- `src/ui/` - TUI implementation with auth flow
- `src/semantic_search.rs` - Core search functionality
- `src/intelligence/` - Context-aware assistance engine
- `src/providers/` - Multi-provider AI chat interface
- `tests/` - Comprehensive test suite

### Development Flow
1. All changes should maintain or improve the user experience
2. Demo mode must continue working without API keys
3. Performance regressions are not acceptable
4. New compiler warnings should be fixed immediately

### Testing Requirements
- Run `cargo test` for unit and integration tests
- Test TUI functionality in both demo and full modes
- Verify semantic search performance hasn't regressed
- Check that all providers still work correctly

## Current Development Focus

1. **Maintaining code quality** - Keep warnings at zero
2. **User experience polish** - Improve error messages and help text
3. **Performance monitoring** - Ensure search performance remains optimal
4. **Documentation** - Keep all docs current with features

## Notes for AI Agents

- This project prioritizes user experience and immediate usability
- Demo mode is a key differentiator - preserve its functionality
- Performance is critical - users expect sub-second search responses
- Code quality matters - clean, readable, well-tested code is required
- The TUI is the primary interface - command-line is secondary

## Commands for Quick Testing

```bash
# Test demo mode (should work without API keys)
aircher

# Test semantic search
aircher search query "error handling"

# Test with API keys (if available)
ANTHROPIC_API_KEY=your_key aircher

# Run full test suite
cargo test

# Check for compiler warnings
cargo check
```