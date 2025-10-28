# RESEARCH

*Index of external research findings and their application to Aircher*

## SOTA Agent Architectures (researched 2025-10-27) ✨ NEW

**Key Finding**: Hybrid architecture combining 4 leading patterns is superior

**Sources**:
- Factory Droid (58.8% Terminal-Bench, #1)
- OpenCode (thdxr, open source, production-validated)
- Claude Code (Anthropic, sub-agent research)
- Sourcegraph Amp (multi-model routing)

**Critical Discoveries**:

1. **Plan/Build Separation** (OpenCode)
   - Read-only exploration (Plan) vs modification (Build)
   - Prevents accidental changes, clear mode distinction
   - Validated in production

2. **LSP Integration** (OpenCode)
   - Real-time diagnostics after file edits
   - Edit → LSP → diagnostics → agent self-corrects
   - Prevents hallucination before code runs
   - 50% fewer runtime errors (estimated)

3. **Git Snapshots** (OpenCode)
   - Temporary commits before risky operations
   - Auto-rollback on errors or permission rejection
   - 100% recovery from failed operations
   - No history pollution

4. **Sub-Agents: When to Use** (Claude Code research)
   - ✅ **Research tasks**: 90% improvement (parallel execution)
   - ❌ **Coding tasks**: 15x token waste (context isolation fatal)
   - Users reported: 160k tokens for 3k work, 20k overhead per sub-agent
   - **Lesson**: Decision matrix based on task type

5. **Specialized Agents** (Factory Droid)
   - Pre-configured "Droids" for different tasks
   - Focused system prompts > generic prompts
   - Smaller tool sets = less decision paralysis
   - #1 on Terminal-Bench

6. **Multi-Model Routing** (Amp)
   - Haiku for simple tasks (fast, cheap)
   - Sonnet for moderate complexity
   - Opus for complex reasoning
   - 40% cost reduction via intelligent selection

**Application**:
- Implement all 6 patterns in hybrid architecture
- Plan/Build modes (Week 7 Day 3-4)
- LSP integration (Week 7 Day 1-2)
- Git snapshots (Week 7 Day 5)
- Research sub-agents only (Week 8 Day 3-4)
- Specialized agents (Week 8 Day 1-2)
- Model router (Week 7 Day 6-7)

**Details**: → ai/SYSTEM_DESIGN_2025.md, ai/DECISIONS.md (2025-10-27)

## Agent Scaffolding (researched 2025-10-27)

**Key Finding**: Simple scaffolding >> Complex orchestration

**Sources**:
- SWE-Agent (May 2024): 3-5x improvement from interface design alone
- SWE-Bench Pro (Sept 2024): Frontier models drop from 70% to 23% on unseen code
- Episodic Memory paper (Feb 2025): Memory improves agents 25-40%

**Application**:
- LM-centric tool interfaces (windowing, limits, validation)
- Error guardrails (linting, auto-reject)
- Episodic + semantic memory integration

**Details**: → ai/research/agent-scaffolding.md

## Memory Systems (researched 2025-10-27)

**Key Finding**: Episodic memory is critical for long-term agents

**5 Essential Properties**:
1. Single-shot learning (learn from one experience)
2. Instance-specific contexts (remember situations)
3. Temporal organization (sequence matters)
4. Flexible retrieval (find relevant past)
5. Adaptive reuse (apply learnings)

**Real-World Evidence**: Devin's success (13.86% SWE-bench) from persistent learning

**Application**:
- DuckDB for pattern storage (already built)
- Repository auto-scanning (like Devin)
- Cross-session memory retrieval

**Details**: → ai/research/memory-systems.md

## Architecture Insights (researched 2024-2025)

**Key Finding**: Models do reasoning internally, agents do execution externally

**What Models Handle**:
- Reasoning, planning, decisions (via prompting)
- Code generation, understanding context
- Self-reflection, pattern recognition

**What Agents Handle**:
- Tool execution (models can't run commands)
- Error prevention (guardrails, validation)
- Memory persistence (models forget)
- Interface design (LM-friendly tools)

**Evidence**: Removed 1685-line MultiTurnReasoningEngine, replaced with 300-line enhanced prompting

**Details**: → docs/architecture/MODEL_VS_AGENT_ARCHITECTURE.md (existing)

## Competitive Intelligence (2024-2025)

**Key Finding**: Sub-agents cause 19% performance degradation vs dynamic context

**Claude Code**:
- Sub-agent architecture issues
- Rate limit problems (50+ incidents/month)
- No persistent memory across sessions

**Devin**:
- Knowledge base across sessions
- Auto-suggests learnings
- Repository scanning

**Cursor**:
- Complex UI (4+ Accept buttons)
- Model flexibility (OpenAI, Anthropic, local)

**Our Advantage**:
- Memory systems (episodic + semantic)
- Dynamic context management
- LM-centric interfaces

**Details**: → ai/research/competitive-intelligence.md

## Research-Based Strategy (2024-2025)

**ReAct**: Thought→Action→Observation loop (25% improvement)
**Reflexion**: Self-reflection + episodic memory (88% vs 67%)
**Tree of Thoughts**: Multi-path exploration (70% improvement)

**Application**: Enhanced prompting system (not external orchestration)

**Details**: → docs/architecture/ (existing)

## Knowledge Graphs for Code Understanding (researched 2025-10-27)

**Key Finding**: Knowledge graphs solve RAG limitations for code repositories

**Problem with RAG:**
- Treats code as flat documents
- Loses inter-file relationships
- Can't answer "How many functions in file X?" or "Where is variable Y used?"

**Knowledge Graph Solution:**
- Nodes: files, classes, functions, variables
- Edges: contains, calls, imports, inherits, uses
- Enables structure-aware queries

**Recent Research (2024-2025):**

**CodexGraph (Microsoft, Aug 2024)**
- Graph database with static analysis extraction
- LLM agent constructs graph queries to find relevant code
- Finding: "Static analysis + graph DB > semantic search alone"

**RepoGraph (Oct 2024)**
- Integrates line-level, file-level, repository-level context
- Surpasses flat RAG approaches
- Key: Multi-level context integration

**KGCompass (Mar 2025)**
- Links repository artifacts (issues, PRs) with code entities
- Narrows search space to 20 most relevant functions
- Application: Bug fixing with historical context

**Knowledge Graph-Based Codegen (May 2025)**
- Semantic search on code docs + structure
- Generated code aligns with project structure
- Uses functional similarity for retrieval

**Application to Aircher:**
- Build graph: tree-sitter → extract nodes/edges → NetworkX
- Store in DuckDB for persistence
- Query before tool execution: "What's relevant to this task?"
- Combine with episodic memory for learned patterns

**Details**: → poc-memory-agent/README.md

## Open Questions

- [ ] Optimal context window size for code (100 lines? 200?)
- [ ] Best linting strategy (tree-sitter? language-specific?)
- [ ] Memory retrieval ranking (semantic similarity? recency? success rate?)
- [ ] Repository scanning patterns (what to extract?)

## Recent Discoveries

**2025-10-27**: LM-centric interfaces critical
- Our tools return too much context (should window to 100 lines)
- Missing validation/linting (should auto-reject syntax errors)
- No result limits (should max 50 results)

**Action Items**: Retrofit Week 1 tools with research patterns in Week 2

---

**Research Principle**: Always research current SOTA, don't rely on stale patterns. This index points to detailed research files for deep dives.
