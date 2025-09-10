# Technical Decisions Log

**Purpose**: Document WHY significant technical decisions were made.

## 2025-08-25: Fix Ollama Provider Tool Support

### Decision
Fixed hardcoded `false` return value in Ollama provider's `supports_tools()` method.

### Context
- Testing revealed gpt-oss model sends proper OpenAI-style tool calls
- Provider was ignoring `tool_calls` field in responses
- Documentation claimed agent system was disconnected (incorrect)

### Implementation
1. Updated `OllamaMessage` struct to include `tool_calls` and `thinking` fields
2. Modified `chat()` method to parse and convert tool calls to standard format
3. Changed `supports_tools()` to return `true` for modern models

### Impact
- Enables local testing without API keys using Ollama
- Tool calling now works with gpt-oss, qwen2.5-coder, deepseek-r1
- Validates agent system is more functional than documented

### Alternative Considered
- Keep tool support disabled and require API providers
- Rejected: Local testing critical for development velocity

---

## 2025-08-09: Adopt hnswlib-rs Backend

### Decision
Replace custom vector search with hnswlib-rs for 45x performance improvement.

### Context
- Index building took 2+ minutes for medium codebases
- Search performance degraded with >1000 vectors
- Users complained about slow first search

### Implementation
- Integrated hnswlib-rs with SIMD optimizations
- Added proper index serialization/deserialization
- Maintained compatibility with existing embeddings

### Impact
- Index building: 2+ minutes → 15-20 seconds
- Search latency: 200ms → 2ms
- Handles 10,000+ vectors efficiently

### Alternative Considered
- GPU acceleration: Too complex for CLI tool
- Custom optimization: Would take months

---

## 2025-08-08: Shell-First Agent Architecture

### Decision
Use shell commands for language tooling instead of native integrations.

### Context
- Need to support multiple languages and tools
- Native integrations would require language-specific dependencies
- Users want transparency in what agent does

### Implementation
- Agent executes shell commands through `RunCommandTool`
- Structured output parsed with JSON when available
- Language servers accessed via stdio

### Impact
- No complex integrations to maintain
- Works with any CLI tool immediately
- Users can reproduce agent actions manually

### Alternative Considered
- Native language bindings: Too much maintenance
- LSP client libraries: Complex and heavy

---

## 2025-08-01: User-Choice Embedding Model Strategy

### Decision
Offer multiple embedding models with clear licensing.

### Context
- SweRank (best quality) has restrictive license
- Users need commercial-safe options
- Different use cases need different quality/size tradeoffs

### Implementation
- MiniLM-L6-v2: Default, Apache 2.0, 90MB
- GTE-Large: Premium, Apache 2.0, 670MB  
- SweRankEmbed: Best, non-commercial, 260MB

### Impact
- Commercial users have safe defaults
- Power users can opt into best models
- Clear licensing prevents legal issues

### Alternative Considered
- Single model only: Too limiting
- Auto-selection: Legal risk

---

## 2025-07-15: Rust + Ratatui for TUI

### Decision
Build TUI in Rust with Ratatui instead of Electron or web UI.

### Context
- Need fast, responsive interface
- Target audience uses terminal extensively
- Electron alternatives are resource-heavy

### Implementation
- Pure Rust TUI with Ratatui
- Crossterm for terminal handling
- Custom components for chat interface

### Impact
- Instant startup (<100ms)
- Low memory usage (<200MB)
- Native terminal integration

### Alternative Considered
- Electron: 500MB+ memory, slow startup
- Web UI: Requires browser, breaks terminal flow
- Blessed.js: Node dependency

---

## 2025-06-20: Tree-sitter for Code Parsing

### Decision
Use tree-sitter for syntax highlighting and AST analysis.

### Context
- Need to parse 19+ languages
- Syntax highlighting essential for search results
- Future AST-based intelligence features

### Implementation
- Tree-sitter with language-specific parsers
- Lazy loading of grammars
- Cached parse trees

### Impact
- Accurate syntax highlighting
- Fast incremental parsing
- Foundation for code intelligence

### Alternative Considered
- Regex-based: Too limited
- Language-specific parsers: Too many dependencies
- TextMate grammars: Less accurate

---

## Template for Future Decisions

## YYYY-MM-DD: [Decision Title]

### Decision
[What was decided]

### Context
[Why this decision was needed]

### Implementation
[How it was/will be implemented]

### Impact
[What changed as a result]

### Alternative Considered
[What else was considered and why rejected]