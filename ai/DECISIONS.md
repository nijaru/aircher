# Technical Decisions Log

**Purpose**: Document WHY significant technical decisions were made.

## 2025-10-27: Toad as Primary Frontend + Stick with Rust

### Decision
Use Toad (universal terminal UI) as primary frontend, keep Rust for agent backend.

### Context
After POC validation (60% improvement), needed to decide:
1. Frontend: Build custom TUI vs use existing ACP-compatible frontend
2. Backend: Continue Rust vs rewrite in Python

**Research findings**:
- Toad: Universal terminal UI for agentic coding (Python/Textual by Will McGugan)
- ACP support announced July 2025
- OpenCode uses TypeScript + Go (split responsibilities)
- Factory Droid scores highest (58.8% Terminal-Bench) but closed source
- Research shows Rust > Python for agent scalability (GIL limitations)

### Rationale
**Frontend: Toad over Custom TUI**:
- Toad provides universal terminal UI (we don't build/maintain)
- ACP protocol means language-agnostic communication (JSON-RPC over stdio)
- Saves 4-6 weeks vs custom Ratatui TUI development
- Works in 5+ frontends: Toad, Zed, Neovim, Emacs, JetBrains

**Backend: Rust over Python**:
- Keep 86K lines investment (semantic search, tools, providers)
- Performance critical for benchmarks (true parallelism, no GIL)
- hnswlib-rs 45x faster than Python alternatives
- Single binary deployment (easy to reproduce benchmarks)
- Research: "Developers moving from Python to Rust for agentic AI"

**Python POC served its purpose**: Validated memory approach in 1-2 weeks

### Implementation
- Week 2: Code understanding tools (search, analyze, references, definitions)
- Week 3-4: ACP protocol (stdio, JSON-RPC) + memory port to Rust
- Week 5-6: Test with Toad (when stable), wire intelligence
- Week 7-8: Benchmarks vs Claude Code

### Impact
- **Time saved**: 4-6 weeks (custom TUI avoided)
- **Reach**: 5+ frontends vs 1 custom TUI
- **Performance**: Rust maintains benchmark advantage
- **Maintenance**: Toad team handles UI, we focus on intelligence

### Alternative Considered
**Rewrite in Python + custom TUI**: 14-16 weeks total, slower benchmarks, harder deployment
**Rejected**: POC already validated approach, Rust infrastructure is competitive advantage

---

## 2025-10-27: Make Repository Public

### Decision
Made Aircher repository public at https://github.com/nijaru/aircher during Week 1 of development.

### Context
- Pivoted from TUI-focused project to ACP agent backend research project
- 10-week timeline targeting publication with empirical validation
- Repository contained no sensitive information (API keys, credentials)
- README rewritten to clearly position as research project (Week 1 of 10, 17-21% parity)
- First production-quality tool (read_file) implemented and committed

### Rationale
**For Research**:
- Open source from day 1 enables community contributions
- Transparent development builds trust and credibility
- Early feedback helps validate approach
- Publication requires open source implementation

**For Development**:
- No reason to stay private - all infrastructure is standard
- Clear honest positioning (Week 1 of 10) sets expectations
- Public commits create accountability and momentum

**Risk Mitigation**:
- Comprehensive .gitignore (target/, private/, .env files)
- No sensitive data in repository
- Honest README about current status (not overpromising)
- Clear it's research project, not production tool

### Impact
- **Positive**: Community can contribute, track progress, validate claims
- **Accountability**: Public commits create pressure to deliver on 10-week plan
- **Positioning**: Early research project, not failed product
- **Collaboration**: Easier to work with others on tool implementations

### Alternative Considered
- Keep private until Week 10 (research paper ready)
- Rejected: No benefit to privacy, misses collaboration opportunities

---

## 2025-09-14: Dynamic Context Management over Sub-Agents

### Decision
Implement Dynamic Context Management instead of autonomous sub-agents for handling complex tasks.

### Context
- Research shows sub-agents cause 19% performance degradation for experienced developers
- Sub-agents suffer from tunnel vision and context pollution issues
- Coordination overhead between multiple agents is problematic
- Industry moving toward structured context engineering in 2025

### Implementation
- DynamicContextManager actively manages context during execution
- Intelligent pruning and fetching of relevant context
- Predictive context loading based on task analysis
- Context importance scoring and token limit enforcement
- No autonomous agents - single agent with smart context

### Impact
- Better performance without multi-agent overhead
- Cleaner context management without pollution
- More predictable behavior than autonomous agents
- Maintains benefits of specialization through context templates

### Alternative Considered
- Sub-agents (Claude Code style): Too much overhead, tunnel vision issues
- Static context windows: Not flexible enough for complex tasks
- Multiple autonomous agents: Coordination problems, context pollution

---

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

## 2025-09-17: Competitive Positioning Strategy Based on User Feedback

### Context
Analysis of HN discussions and user feedback revealed key frustrations with existing AI coding agents:
- Rate limits causing workflow interruptions (Claude Code/Cursor both affected)
- Lack of transparency in execution steps (Claude Code "flying blind")
- Unpredictable costs for heavy usage ($100+/month required)
- Complex UI with multiple "Accept" buttons (Cursor pain point)
- Neither tool handles Jupyter notebooks well

### Options Considered
1. **Copy Market Leaders**
   - Pros: Proven features, easier to build
   - Cons: Commodity product, no differentiation

2. **Focus on Single Workflow**
   - Pros: Deep optimization, clear positioning
   - Cons: Limited market, locks out use cases

3. **Hybrid Approach - Best of Both Worlds**
   - Pros: Unique positioning, addresses real pain points
   - Cons: More complex to build, requires execution excellence

### Decision
**Chosen: Hybrid Approach - Autonomous Transparency**

### Rationale
- Users want Claude Code's autonomy BUT Cursor's visibility
- Local models eliminate rate limit frustrations (major differentiator)
- Our architecture already supports both modes (approval workflows)
- Safety improvements (SafeWriteFileTool) exceed both competitors
- Semantic search advantage provides better codebase understanding

### Implementation
1. **Multi-step autonomous execution** with visible progress
2. **Transparent step-by-step display** of what agent is doing
3. **Local model optimization** for rate-limit-free usage
4. **Model switching mid-conversation** for flexibility
5. **Predictable cost tracking** with local model fallbacks
6. **Jupyter notebook support** as differentiator

### Impact
- **vs Cursor**: Better autonomy, no rate limits, lower cost
- **vs Claude Code**: Better transparency, model flexibility, safety
- **Market positioning**: "Autonomous coding with complete visibility"
- **User value**: Unlimited usage + trust through transparency

### Consequences
- Need to deliver on both autonomy AND transparency (execution risk)
- Must optimize local models to compete with API speed
- Documentation/UX becomes critical differentiator
- Success depends on execution quality over feature count

### Review Date
After 6 weeks of implementation - measure user adoption patterns

---

## 2025-09-19: Enhanced Prompting over Complex Orchestration

### Decision
Replace 1685-line MultiTurnReasoningEngine with 300-line enhanced prompting system.

### Context
- Discovered we were externalizing reasoning that models do internally
- Research shows 25-70% improvements come from better prompts, not orchestration
- MultiTurnReasoningEngine tried to manage external reasoning phases
- Models already optimize for chain-of-thought, reflection, multi-path reasoning

### Implementation
- Created EnhancedPromptingSystem with research-based patterns
- ReAct prompts for multi-step tasks (Think→Act→Observe)
- Reflexion prompts for debugging (systematic reflection)
- Tree-of-Thoughts prompts for complex analysis (multi-path)
- Direct prompting leverages model's internal reasoning

### Impact
- **-1685 lines** of complex orchestration code
- **Faster execution** without plan generation overhead
- **Better reasoning** by leveraging model optimization
- **Simpler architecture** with clear separation of concerns

### Alternative Considered
- Keep MultiTurnReasoningEngine and improve it
- Rejected: Fundamentally wrong approach, models already do this better

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