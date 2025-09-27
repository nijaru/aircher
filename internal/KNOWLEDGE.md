# Aircher Knowledge Base

*Patterns, learnings, and insights from development*

## Research-Based Agent Strategy Insights (2024-2025)

### Industry State of the Art
- **ReAct (Google, 2022)**: Interleaving reasoning and acting beats pure CoT by 25% on complex tasks
- **Reflexion (Shinn et al, 2023)**: Self-reflection achieves 88% pass@1 on HumanEval (vs 67% GPT-4)
- **Tree of Thoughts (Princeton, 2023)**: Multi-path exploration improves reasoning by 70%
- **Devin (Cognition, 2024)**: Interactive planning + parallel execution achieved 13.86% SWE-bench
- **Claude Opus 4 (2025)**: Reached 72.5% on SWE-bench with systematic exploration

### Key Success Patterns from Research
1. **Simplicity Over Complexity**: Anthropic found simple, composable patterns beat complex frameworks
2. **Tool Design Matters More Than Prompts**: SWE-bench winners spent more time on tools than prompts
3. **Dynamic Strategy Selection**: Different task types need fundamentally different approaches
4. **Self-Reflection Critical**: Reflexion's verbal feedback loop enables rapid improvement
5. **Context Management**: All successful agents have sophisticated context pruning/management

### What Actually Works
- **For Bug Fixing**: Systematic exploration → reproduction → analysis → fix → validation (SWE-bench pattern)
- **For Exploration**: ReAct's thought-action-observation loop with tool integration
- **For Complex Problems**: Tree of Thoughts with backtracking and multi-path evaluation
- **For Iterative Tasks**: Reflexion's self-reflection and episodic memory

## Architectural Insights

### Models are Reasoning Engines, Agents are Execution Engines (Sep 19, 2025)
- **Critical Discovery**: We over-engineered 1685-line MultiTurnReasoningEngine
- **Models already do internally**: Chain-of-thought, reflection, multi-path reasoning
- **Research validated**: ReAct/Reflexion improvements from prompts, not orchestration
- **Correct approach**: Enhanced prompting patterns, not external reasoning management
- **Agent focus**: Tool execution, validation, persistence, safety
- **Key learning**: Don't externalize what models optimize for internally

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

### Competitive Positioning (Updated Jan 2025)

#### CLI/TUI Agent Market Landscape

##### **Claude Code (Anthropic)**
- **Type**: Official Anthropic terminal client, closed-source
- **Strengths**:
  - Deep integration with Claude models (Opus 4.1, Sonnet 4, Haiku)
  - Sophisticated multi-step autonomous execution
  - 200K+ context windows with intelligent compaction
  - Strong adoption among enterprise users
  - Excellent at architectural planning and complex refactoring
- **Weaknesses**:
  - "Flying blind" experience - hides reasoning/decision process
  - Aggressive rate limits (50+ incidents/month reported)
  - No local model support
  - Limited model flexibility (Anthropic models only)
  - Performance degradation during peak hours (9AM PST)
- **Market Position**: Premium tool for enterprise teams
- **Pricing**: $25-100+/month depending on usage

##### **Cursor (Anysphere)**
- **Type**: IDE with integrated AI (not pure CLI)
- **Strengths**:
  - Deep IDE integration (VSCode fork)
  - Model flexibility (OpenAI, Anthropic, local)
  - Inline autocomplete + chat modes
  - Composer mode for multi-file editing
  - Step-by-step transparency
- **Weaknesses**:
  - Complex UI (4+ different Accept buttons)
  - Decision fatigue from too many options
  - Rate limits still apply for API models
  - Heavy resource usage (Electron-based)
  - Not terminal-native
- **Market Position**: IDE-first for developers who want control
- **Pricing**: $20/month + API costs

##### **FactoryAI Droid**
- **Type**: Benchmark-focused CLI tool, closed-source
- **Strengths**:
  - Best SWE-bench scores (35%+ on verified)
  - Optimized for autonomous task completion
  - Strong test generation capabilities
  - Minimal setup required
- **Weaknesses**:
  - Black box approach - no visibility into operations
  - Limited customization options
  - Closed ecosystem, no extensibility
  - Single provider lock-in
  - No local model support
- **Market Position**: Performance-focused for benchmark tasks
- **Pricing**: Not publicly disclosed

##### **Sourcegraph Cody CLI**
- **Type**: Enterprise codebase intelligence tool
- **Strengths**:
  - Excellent large codebase navigation
  - Enterprise SSO and compliance features
  - Code search across entire organizations
  - Context from multiple repositories
  - Strong security/audit features
- **Weaknesses**:
  - Heavy enterprise focus (overkill for individuals)
  - Requires Sourcegraph server deployment
  - Limited autonomous capabilities
  - Complex setup and configuration
- **Market Position**: Enterprise code intelligence platform
- **Pricing**: Enterprise contracts ($$$)

##### **Aider (Open Source)**
- **Type**: Terminal-based pair programming tool
- **Strengths**:
  - True open source (Apache 2.0)
  - Git-aware with automatic commits
  - Multiple model support (OpenAI, Anthropic, local)
  - Map of whole repository context
  - Strong community (15K+ stars)
- **Weaknesses**:
  - Limited autonomous capabilities
  - Basic UI (pure terminal output)
  - No advanced features (no semantic search)
  - Requires manual file selection
- **Market Position**: Open source alternative for individuals
- **Pricing**: Free (bring your own API keys)

##### **Continue.dev (Open Source)**
- **Type**: IDE extension with CLI capabilities
- **Strengths**:
  - Open source with active development
  - Supports multiple IDEs (VSCode, JetBrains)
  - Local model support via Ollama
  - Customizable with TypeScript
  - Tab autocomplete + chat modes
- **Weaknesses**:
  - IDE-dependent (not standalone CLI)
  - Limited autonomous execution
  - Setup complexity for advanced features
  - Performance varies by configuration
- **Market Position**: Open source IDE assistant
- **Pricing**: Free (self-hosted) or $10/month (cloud)

##### **Mentat (Open Source)**
- **Type**: Terminal-based AI coding assistant
- **Strengths**:
  - Clean terminal interface
  - Automatic context detection
  - Interactive editing mode
  - Git integration
- **Weaknesses**:
  - Limited model support (OpenAI primarily)
  - No semantic search capabilities
  - Basic tool set
  - Small community
- **Market Position**: Lightweight CLI alternative
- **Pricing**: Free (bring your own API keys)

#### Browser-Based Autonomous Agent Platforms

A new category of fully automated, browser-based coding agents has emerged, focusing on complete task automation rather than developer assistance:

##### **Google Jules (Labs)**
- **Type**: Browser-based, fully autonomous coding agent
- **Strengths**:
  - Complete task automation with minimal human intervention
  - Creates comprehensive plans before execution
  - Handles entire GitHub workflows (issues → PRs)
  - Multi-file modifications with context awareness
  - Integrated with Google's AI infrastructure
- **Weaknesses**:
  - Limited to browser environment
  - No local development environment access
  - Black box execution model
  - Limited customization/control
  - GitHub-centric workflow
- **Market Position**: Experimental autonomous agent for GitHub
- **Access**: Limited preview (Labs product)

##### **Replit Agent**
- **Type**: Cloud IDE with autonomous agent capabilities
- **Strengths**:
  - Full application generation from prompts
  - Integrated deployment pipeline
  - Real-time collaboration features
  - Handles frontend + backend + database
  - One-click deployment
- **Weaknesses**:
  - Locked to Replit ecosystem
  - Limited to web applications
  - No local development option
  - Subscription required for advanced features
- **Market Position**: Full-stack app generator for rapid prototyping
- **Pricing**: $20-25/month for agent features

##### **GitHub Copilot Workspace**
- **Type**: Cloud-based task planning and execution system
- **Strengths**:
  - Deep GitHub integration
  - Task decomposition and planning
  - Multi-file coordinated changes
  - Natural language task descriptions
  - Preview changes before applying
- **Weaknesses**:
  - GitHub-only workflow
  - Limited to web interface
  - No terminal/CLI access
  - Requires Copilot subscription
- **Market Position**: GitHub-native development environment
- **Pricing**: Part of GitHub Copilot ($10-19/month)

##### **Vercel v0**
- **Type**: AI-powered UI generation platform
- **Strengths**:
  - Generates complete React components from descriptions
  - Instant deployment to Vercel
  - Shadcn/UI integration
  - Iterative refinement through chat
- **Weaknesses**:
  - Frontend-only focus
  - Limited to React/Next.js ecosystem
  - No backend logic generation
  - Requires manual integration
- **Market Position**: UI/UX rapid prototyping tool
- **Pricing**: Credits-based system

#### Terminal/CLI vs Browser-Based Agents

| Aspect | Terminal/CLI Agents | Browser-Based Agents |
|--------|-------------------|---------------------|
| **Environment** | Local development | Cloud/browser only |
| **Control** | Fine-grained, step-by-step | Fully automated, less control |
| **Transparency** | Can show exact operations | Often black-box execution |
| **Integration** | Works with any local tools | Limited to platform ecosystem |
| **Offline Work** | Fully supported (with local models) | Requires internet connection |
| **Data Privacy** | Code stays local | Code processed in cloud |
| **Customization** | Highly configurable | Platform-defined workflows |
| **Target Users** | Professional developers | Rapid prototypers, beginners |

#### Competitive Analysis Matrix

| Feature | Claude Code | Cursor | Jules | Replit Agent | Copilot WS | Aider | Aircher (Us) |
|---------|------------|--------|-------|--------------|------------|-------|--------------|
| **Agent Autonomy** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Task Planning** | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐ | ⭐⭐⭐⭐ |
| **Multi-Step Execution** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| **Transparency** | ⭐ | ⭐⭐⭐⭐ | ⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Local Development** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ❌ | ❌ | ❌ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Customization** | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐ | ⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Local Models** | ❌ | ⭐⭐⭐ | ❌ | ❌ | ❌ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Rate Limits** | ⭐ | ⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐ | N/A | ⭐⭐⭐⭐⭐ |
| **Open Source** | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |

#### Our Unique Positioning: Agent-First Terminal Development

**Core Value Proposition: "Professional agent autonomy with complete transparency and control"**

We occupy a unique position in the market by combining:
- **Agent-level autonomy** (matching Jules/Replit Agent capabilities)
- **Terminal-native performance** (instant startup, low resources)
- **Complete transparency** (see every decision and action)
- **Local-first architecture** (privacy, no rate limits with Ollama)

##### Agent Capability Advantages

**What Sets Our Agent Apart:**

1. **Hybrid Autonomy Model**
   - Can run fully autonomous (like Jules) OR step-by-step (like Cursor)
   - User chooses level of control per task
   - Smart approval workflows for safety

2. **Terminal-Native Agent Power**
   - Direct access to ALL local development tools
   - No browser limitations or sandbox restrictions
   - Works with existing developer workflows

3. **Transparent Agent Reasoning**
   - Shows planning phase (unlike Claude Code's black box)
   - Displays tool selection rationale
   - Explains multi-step execution strategy
   - Learn from agent's decision-making

4. **Unlimited Local Agent Execution**
   - Ollama integration = infinite agent runs
   - No rate limits on complex multi-step tasks
   - Consistent performance (no cloud variability)

5. **Agent Memory & Learning**
   - Remembers project patterns and preferences
   - Learns from successful task completions
   - Improves suggestions over time (unlike stateless browser agents)

##### Competitive Agent Positioning

**vs Browser-Based Agents (Jules, Replit, Copilot Workspace):**
- ✅ **We win on**: Local control, privacy, customization, professional tools
- ❌ **They win on**: Zero setup, beginner-friendly, instant deployment
- **Our target**: Professional developers who need control + autonomy

**vs Terminal/CLI Tools (Claude Code, Aider):**
- ✅ **We win on**: Transparency, local models, agent learning, safety
- ❌ **They win on**: Market maturity, enterprise features
- **Our target**: Developers frustrated with rate limits and black-box execution

**vs IDE Extensions (Cursor, Continue):**
- ✅ **We win on**: Terminal performance, agent autonomy, resource efficiency
- ❌ **They win on**: IDE integration, inline editing
- **Our target**: Terminal-first developers who want agent capabilities

##### Strategic Agent Features (Competitive Differentiation)

1. **Task Orchestration Engine**
   - Decomposes complex tasks like Jules
   - But shows the plan and allows modification
   - Can pause/resume multi-step executions

2. **Intelligent Tool Selection**
   - Agent chooses optimal tools for each step
   - Explains why each tool was selected
   - User can override or suggest alternatives

3. **Adaptive Execution Strategies**
   - Multiple approaches for same task
   - Agent selects based on project context
   - Falls back gracefully on failures

4. **Project-Aware Intelligence**
   - Understands your codebase structure
   - Respects existing patterns and conventions
   - Suggests improvements based on project history

##### Market Opportunity

The market is splitting into two segments:
1. **Fully Automated** (Jules, Replit) - for rapid prototyping
2. **Developer Assistance** (Cursor, Copilot) - for professional coding

**We target the gap**: Professional developers who want agent-level automation but need transparency, control, and local execution. This is especially valuable for:
- Security-conscious organizations (code stays local)
- Developers hitting API rate limits
- Teams needing customizable agent behavior
- Projects requiring specific tool integrations

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

*Updated: 2025-01-27 - Added browser-based agents (Jules, Replit) and agent-first positioning*
*Previous: 2025-01-27 - Enhanced competitive intelligence with detailed competitor analysis*
*Previous: 2025-09-10 - Initial knowledge base from competitive analysis*
