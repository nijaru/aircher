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

### User Frustrations (HN Research - Sept 2025)

#### **Rate Limits & API Dependencies**
- **Major Pain Point**: Both Claude Code and Cursor hit rate limits during serious development
- **User Quote**: "Rate limits impact serious development workflows"
- **Our Advantage**: Local models (Ollama) = unlimited usage
- **Impact**: Users paying $100+/month for adequate rate limits

#### **Trust vs Control Dilemma**
- **Claude Code**: "Flying blind" - hides decision process, asks for trust
- **Cursor**: "Decision fatigue" - too many choices, complex UI (4 Accept buttons)
- **User Need**: Want autonomy BUT with visibility into what's happening
- **Our Solution**: Autonomous execution with transparent step-by-step display

#### **Inconsistent Performance**
- **Claude Code**: "50 incidents in July, 40 in August, 21 in September"
- **Quality drops**: Users notice degradation "around 12 ET (9AM pacific)"
- **Our Advantage**: Local models = consistent performance, no infrastructure issues

### Competitive Positioning (Updated Sept 2025)

#### **vs Claude Code**
- **Their strength**: Sophisticated autonomy, large context windows
- **Their weakness**: No transparency, rate limits, flying blind experience
- **Our advantage**: Autonomous + transparent, unlimited local usage, better safety
- **Target**: Users who want Claude Code's power with Cursor's visibility

#### **vs Cursor**
- **Their strength**: Model flexibility, step-by-step control, IDE integration
- **Their weakness**: Rate limits, complex UI, decision fatigue
- **Our advantage**: Same model flexibility without rate limits, cleaner UX
- **Target**: Users who want Cursor's control without the complexity

#### **vs Both (Unique Positioning)**
- **Shared weakness**: Neither handles Jupyter notebooks well
- **Our opportunity**: Add Jupyter support as differentiator
- **Market gap**: "Autonomous coding with complete visibility"
- **Value prop**: Unlimited usage + trust through transparency

### User Workflow Insights

#### **What Users Actually Want**
1. **Multi-step autonomous execution**: "Tell the AI what to accomplish rather than what changes to make"
2. **Large codebase analysis**: "Analyze entire codebase before generating anything"
3. **Execution transparency**: Want to see progress without managing every step
4. **Cost predictability**: Frustrated by surprise API bills
5. **Reliable performance**: Consistent quality without degradation

#### **Usage Patterns**
- **Deep coding sessions**: Cursor's autocomplete for quick fixes
- **Architecture work**: Claude Code for complex thinking/planning
- **Many use both**: Cursor for typing, Claude Code for thinking
- **Switching triggers**: Rate limits, quality drops, specific task types

### Technical Lessons

#### **Model Performance Insights**
- **Token sampling matters**: "top-k" and temperature significantly impact quality
- **Infrastructure complexity**: Multiple hardware platforms create bugs
- **Evaluation needs**: More sensitive continuous evaluation on production
- **User perception**: Quality inconsistency more noticeable than absolute performance

#### **UX Design Patterns**
- **Transparency wins trust**: Users prefer seeing steps over blind execution
- **Single decision points**: Multiple "Accept" buttons create confusion
- **Cost visibility**: Hidden usage costs create user anxiety
- **Consistent interfaces**: Complexity should be optional, not required

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
