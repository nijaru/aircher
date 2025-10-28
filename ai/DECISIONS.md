# Technical Decisions Log

**Purpose**: Document WHY significant technical decisions were made.

## 2025-10-27: Architecture Pivot to Hybrid SOTA Design

### Decision
Redesign Aircher architecture combining best patterns from 4 leading agents:
1. **OpenCode**: Plan/Build separation + LSP integration + Git snapshots
2. **Factory Droid**: Specialized agents for different tasks
3. **Claude Code**: Research sub-agents (BUT learned when NOT to use them)
4. **Amp**: Multi-model routing for cost optimization

### Context
After Week 6 ACP completion, conducted comprehensive SOTA research to validate our architecture decisions from September 2024. Research revealed critical insights that warranted major redesign.

### Research Findings

**Factory Droid** (58.8% Terminal-Bench, #1):
- Uses specialized "Droids" with pre-configured prompts for specific tasks
- Pattern: Focused agents > generic multi-purpose agent
- Closed source, but concept is validated

**OpenCode** (thdxr, open source):
- **Plan/Build separation**: Read-only exploration vs modification-capable
- **LSP integration**: Real-time diagnostics after edits prevents hallucination
- **Git snapshots**: Temporary commits before risky ops, auto-rollback on failure
- **Event bus**: Global diagnostics map, event-driven architecture
- Validated in production, proven pattern

**Claude Code** (Anthropic):
- **Critical discovery**: Sub-agents have OPPOSITE effects for different tasks
  - ✅ Research: 90% improvement (parallel information gathering)
  - ❌ Coding: 15x token waste (context isolation is fatal)
- Users complaining: 160k tokens for 3k work, 20k overhead per sub-agent
- **Lesson**: NEVER use sub-agents for coding, ONLY for research

**Amp** (Sourcegraph):
- Multi-model routing: Haiku for simple, Opus for complex
- Cost-aware selection: 40% reduction via intelligent routing
- Model flexibility: User can override, per-task selection

### Rationale

**Our September 2024 Decision Was Partially Wrong**:
- ✅ **Correct**: Rejected sub-agents for coding (they're terrible)
- ❌ **Mistake**: Dismissed sub-agents entirely (they're great for research)
- ❌ **Missed**: Plan/Build separation, LSP integration, Git snapshots

**New Hybrid Architecture Advantages**:
1. **Plan/Build Separation** (from OpenCode)
   - Plan mode: Safe read-only exploration, can spawn research sub-agents
   - Build mode: Controlled modification, NEVER uses sub-agents
   - Prevents accidental modifications, clear mode distinction

2. **LSP Integration** (from OpenCode)
   - Edit → LSP diagnostics → Agent sees errors → Self-correct
   - Prevents hallucination before code runs
   - Real-time feedback loop

3. **Specialized Agents** (from Factory Droid)
   - Explorer (CodeReading), Builder (CodeWriting), Debugger (ProjectFixing), Refactorer
   - Focused system prompts per agent type
   - Smaller tool sets = less decision paralysis

4. **Smart Sub-Agents** (learned from Claude Code)
   - ✅ Spawn for research tasks: 90% improvement
   - ❌ NEVER for coding: Avoid 15x token waste
   - Decision matrix based on task type

5. **Git Snapshots** (from OpenCode)
   - Snapshot before risky operations
   - Auto-rollback on errors or rejections
   - 100% recovery from failures

6. **Model Router** (from Amp)
   - Haiku for simple tasks (fast, cheap)
   - Sonnet for moderate complexity
   - Opus for complex reasoning
   - 40% cost reduction target

7. **Memory Systems** (our unique advantage)
   - Episodic: Prevents duplicate research
   - Knowledge Graph: Instant codebase queries
   - Working Memory: Dynamic context pruning
   - **Nobody else has this**

### Implementation Timeline

**Week 7**: Event bus, LSP, Plan/Build modes, Git snapshots, model router
**Week 8**: Specialized agents, research sub-agents, integration testing
**Week 9**: Benchmarks vs Claude Code
**Week 10**: Research paper + release

### Expected Results

| Metric | Target | Source |
|--------|--------|--------|
| Tool call reduction | 60% | Memory systems |
| Research speedup | 90% | Parallel sub-agents |
| Coding sub-agent usage | 0% | Avoid 15x waste |
| LSP self-correction | 50% | Real-time diagnostics |
| Cost reduction | 40% | Model routing |
| Operation recovery | 100% | Git snapshots |

### Impact

**Research Contributions**:
1. First agent to combine all these patterns
2. Validated hybrid architecture (not single-strategy)
3. Decision matrix for sub-agent usage (when yes, when no)
4. LSP-augmented feedback loop
5. Memory-augmented intelligence

**Competitive Advantages**:
- Only agent with all patterns combined
- Memory systems (nobody has)
- Intent-driven strategy selection
- Empirically validated design

### Trade-offs

**Complexity**: More patterns = more code to maintain
- Mitigation: Each pattern is independently valuable
- Can implement incrementally, validate each

**Implementation Time**: 4 weeks vs 2 weeks for simple approach
- Mitigation: Research-validated patterns reduce risk
- Better to get it right than ship fast

**Learning Curve**: Users must understand modes
- Mitigation: Mode selection is automatic (intent-driven)
- Clear documentation for manual override

### Alternative Considered

**Keep September 2024 Architecture**: Single agent with dynamic context only
- **Rejected**: Leaves too much performance on the table
  - Missing 90% research speedup (sub-agents)
  - Missing 50% error reduction (LSP feedback)
  - Missing 100% recovery (Git snapshots)
  - Missing 40% cost savings (model routing)

**Reasoning**: SOTA research shows clear advantages. Would be irresponsible to ignore.

### Review Date

After Week 9 benchmarks - validate all improvement claims with empirical data.

---

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

## 2025-10-27: DuckDB + petgraph for Memory Systems

### Decision
Use **DuckDB for episodic memory** and **petgraph for knowledge graph**.

### Context
Memory system architecture requires two storage strategies:
1. **Episodic memory**: Track tool executions, file interactions, tasks, patterns
2. **Knowledge graph**: Codebase structure (files, functions, calls, imports)

**Requirements**:
- Episodic: Analytical queries ("files edited together?", "have I seen this before?")
- Graph: Fast traversals ("what calls this?", "what's in file X?")
- Both: Persist across sessions, incremental updates

### Rationale

**DuckDB for Episodic Memory**:
- ✅ Already have infrastructure (`src/intelligence/memory/duckdb_memory.rs`)
- ✅ Better than SQLite for analytical queries (vectorized execution)
- ✅ JSON columns for flexible schema (parameters, patterns, metadata)
- ✅ Embedded (no server, single file)
- ✅ Arrow integration (export to DataFrame for analysis)
- ✅ Size estimate: ~20MB for typical session (1000 tool calls, 10K interactions)

**petgraph for Knowledge Graph**:
- ✅ In-memory = microsecond graph traversals (vs milliseconds in database)
- ✅ Mature Rust crate (3.2M downloads, stable API)
- ✅ Supports directed graphs with typed nodes/edges
- ✅ Binary serialization (serde) for persistence
- ✅ POC validated: 3,942 nodes, 5,217 edges for Aircher codebase
- ✅ Incremental updates: Re-parse only changed files

**Alternatives Considered**:

1. **SQLite for episodic**: Good but DuckDB better for analytical queries
   - Rejected: DuckDB already integrated, better performance

2. **Graph database (Neo4j, etc.)**: Powerful but heavyweight
   - Rejected: Overkill for our use case, deployment complexity

3. **All-in-one solution (PostgreSQL + graph extension)**: Possible
   - Rejected: More complex, petgraph in-memory is faster

### Implementation

**DuckDB Schema (5 tables)**:
- tool_executions: Every tool call, success/failure, duration
- file_interactions: Every file operation, in what context
- task_history: User-level goals, status, outcome
- context_snapshots: Periodic state for debugging
- learned_patterns: Co-edit patterns, error fixes

**petgraph Structure**:
- Nodes: File, Function, Class, Import, Variable (enum)
- Edges: Contains, Calls, Imports, Uses, Inherits (enum)
- Queries: get_file_contents, get_callers, find_symbol
- Storage: Binary file (`knowledge_graph.bin`) loaded on startup

**Week-by-week**:
- Week 3: DuckDB episodic memory (schema + recording + queries)
- Week 4: petgraph knowledge graph (build + query + incremental)
- Week 5: Dynamic context management (integrate both)

### Impact
- **Continuous work**: Dynamic pruning removes low-value context, summarizes to episodic memory
- **60% fewer tool calls**: Knowledge graph answers "what calls this?" without file scanning
- **Pattern learning**: Co-edit detection, error-fix patterns improve over time
- **Cross-session memory**: Both systems persist, agent learns across conversations

### Trade-offs
- **Memory usage**: petgraph in-memory (~50-100MB for large codebases)
  - Mitigation: Serialize when not in use, lazy load on query
- **Build time**: Initial graph construction 10-30 seconds
  - Mitigation: Cache graph, incremental updates on file change
- **Complexity**: Two storage systems to maintain
  - Mitigation: Clear separation of concerns, well-tested

### Research Contribution
This architecture enables the **continuous work capability** that distinguishes Aircher from Claude Code:
- Claude Code: Restarts when context fills
- Aircher: Prunes intelligently, remembers via episodic memory, queries knowledge graph

**Paper title**: "Memory-Augmented Coding Agents: Empirical Validation"
**Key metric**: 60% reduction in tool calls (POC validated)

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