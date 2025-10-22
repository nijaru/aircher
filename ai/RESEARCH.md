# RESEARCH

*Index of external research findings and their application to Aircher*

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
