# Aircher Knowledge Base

*Patterns, learnings, and insights from development*

## Architectural Insights

### Dynamic Context Management > Sub-Agents (Sep 14, 2025)
- **Sub-agents cause 19% performance degradation** in experienced developers
- **Tunnel vision problem**: Sub-agents get stuck in single reasoning paths
- **Context pollution**: Multiple agents actually worsen context management
- **Our innovation**: DynamicContextManager that actively manages context
- **Key insight**: Single agent with smart context beats multiple autonomous agents
- **Implementation**: Context pruning, prefetching, and relevance scoring
- **Competitive advantage**: Better than Claude Code's sub-agents without the overhead

### Critical Issues with Current Context Manager (Sep 14, 2025)
**Design Issues:**
- **SemanticCodeSearch fails without index**: Search returns "Index not built" error on empty index
- **No actual indexing**: We create SemanticCodeSearch but never index the codebase
- **Unused components**: prefetch_queue, relationship_graph, learned_patterns never used
- **No learning mechanism**: ContextPredictor has structures but no actual learning logic
- **Simplistic analysis**: analyze_context_needs just extracts keywords, no deep understanding

**Functional Gaps:**
- **Search will always fail**: Without indexed codebase, semantic search throws errors
- **No relationship tracking**: Context items have relationship fields but never populated
- **Basic token limits**: Just removes low-importance items, no smart compaction
- **No predictive loading**: Config flag exists but no actual implementation
- **Missing embeddings**: Can't do semantic search without embedding model loaded

**What Actually Works:**
- ✅ File content loading from disk when requested
- ✅ Token counting and basic limit enforcement
- ✅ Context item creation and storage
- ✅ Integration with IntelligenceEngine
- ✅ Cache eviction mechanism
- ✅ Relevance scoring updates
- ✅ File access tracking from tools

**Required Fixes:**
1. Either index codebase on startup OR handle empty index gracefully
2. Remove or implement unused components (predictor, relationships)
3. Implement actual learning from patterns
4. Make search optional/fallback to file-based context
5. Add proper error handling for search failures

### TUI Performance Optimization
- **ratatui** is stable and proven vs React-based terminals
- Rust startup time (<100ms) is key competitive advantage  
- Collapsible tool outputs prevent conversation bloat
- Vertical layout (conversation → todos → input → status) optimal for terminal

### Model Selection as Differentiation
- Users want transparency vs "always best models" approach
- Rich metadata display (context, cost, capabilities) builds trust
- Real-time model availability checks improve UX
- Multi-provider fallbacks reduce reliability concerns

### Tool Integration Lessons
- Shell-first approach simpler than native integrations
- JSON output parsing more reliable than text scraping
- Agent-tool communication needs bulletproof error handling
- Progressive enhancement: basic features must work without tools

## Code Organization Insights

### Rust Project Structure
- `src/ui/` - TUI implementation (ratatui-based)
- `src/semantic_search.rs` - Core search engine (production-ready)
- `src/agent/` - Tool system (needs integration work)
- `src/providers/` - Multi-provider API handling
- Zero warnings policy maintained for competitive quality

### Development Workflow
- Document-first approach prevents feature creep
- Regular commits with descriptive messages
- TODO tracking essential for multi-step implementations
- External pattern references improve code consistency

## Competitive Intelligence  

### vs Amp
- **Their strength**: Curated "always best" models
- **Our advantage**: Model choice transparency + multi-provider
- **Must match**: TODO panel, conversation UX, tool reliability

### vs Claude Code/Electron TUIs
- **Their weakness**: Performance with long conversations
- **Our advantage**: Rust native performance
- **Key differentiator**: Fast startup + efficient rendering

### vs Cursor/IDE Integration
- **Their strength**: Editor integration
- **Our advantage**: Terminal-first workflow efficiency
- **Target users**: CLI-heavy developers, remote work

## Technical Discoveries

### Tool Calling Architecture
- Agent system IS connected but needs reliability polish
- Tool parsing supports both XML and JSON formats
- Ollama provider support fixed (was hardcoded false)
- End-to-end testing critical for tool workflows

### Provider Management
- Dynamic model fetching improves accuracy
- Graceful fallbacks prevent user-facing errors
- Loading states better than error messages
- Cost tracking adds transparency value

## Development Patterns

### Documentation Standards
- Follow @external/agent-contexts/standards/DOC_PATTERNS.md
- AGENTS.md as entry point with @references
- internal/ for tracking, root for strategy
- Decision logs append-only for historical context

### Code Standards  
- Apply @external/agent-contexts/standards/AI_CODE_PATTERNS.md
- Zero warnings policy enforced
- Descriptive naming over generic (no "data", "info")
- TODO comments with specific actions needed

## Future Insights

### Phase 1 Priorities (Current)
1. Tool calling reliability (critical path)
2. Model selection polish (differentiation)  
3. TODO panel implementation (parity)
4. Conversation UX improvements (user retention)

### Long-term Vision
- Become the "model selection expert" in coding agent space
- Superior multi-provider experience
- Best-in-class local model (Ollama) integration
- Terminal performance advantage over Electron competitors

---

*Updated: 2025-09-10 - Initial knowledge base from competitive analysis*
