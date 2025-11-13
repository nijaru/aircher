# Agent Intelligence Roadmap

**Last Updated**: 2025-10-27
**Focus**: Intelligent ACP-compatible agent backend - NO UI development

## 🎯 Mission

Build a research-grade intelligent agent backend that works via Agent Client Protocol (ACP) in any compatible frontend (Zed, JetBrains IDEs, Neovim, Emacs, VSCode).

**What we're building:**
- ✅ Novel agent intelligence architecture
- ✅ Intent classification and dynamic context management
- ✅ Pattern-aware code generation
- ✅ Empirical validation vs competitors
- ✅ Publication-worthy research contributions

**What we're NOT building:**
- ❌ Custom TUI, IDE, or editor
- ❌ UI themes, keyboard shortcuts, terminal rendering
- ❌ Enterprise features (SSO, audit trails, team collaboration)

## 📊 Current Reality (October 2025)

### Major Strategic Pivot: SOTA Architecture Redesign ✨

**Research Findings** (Oct 27, 2025):
- **Factory Droid**: 58.8% Terminal-Bench using specialized agents
- **OpenCode**: Plan/Build separation + LSP integration + Git snapshots
- **Claude Code**: Sub-agents work for research (90% improvement) but terrible for coding (15x waste)
- **Amp**: Multi-model routing for cost optimization

**New Direction**: Hybrid architecture combining best patterns from all tools

### What Actually Works ✅
- **Week 6 Complete**: ACP protocol with session management, streaming, error handling (+635 lines)
- **Memory Systems**: ALL 3 complete (3,725 lines) - Episodic, Knowledge Graph, Working
- **Semantic Search**: Production-ready (6,468 vectors, 19+ languages, sub-second)
- **Multi-Provider**: OpenAI, Anthropic, Gemini, Ollama authentication
- **6 Production Tools**: read_file, write_file, edit_file, list_files, analyze_errors, analyze_code (2,300+ lines)

### What's Being Redesigned 🔄
- **Agent Architecture**: Single agent → Hybrid (Plan/Build modes + research sub-agents)
- **LSP Integration**: Adding real-time diagnostics feedback (from OpenCode pattern)
- **Git Snapshots**: Safe experimentation with auto-rollback (from OpenCode)
- **Model Routing**: Multi-model selection based on task complexity (from Amp)
- **Specialized Agents**: Explorer, Builder, Debugger, Refactorer configs (from Factory Droid)

### Competitive Position (Updated)
- **30-33% feature parity** with Claude Code (infrastructure complete, capabilities growing)
- **Unique advantages**: Memory systems (nobody has), ACP multi-frontend, intent-driven architecture
- **Research validated**: Patterns from 4 SOTA tools combined
- **Timeline**: Weeks 7-10 implement new architecture + benchmarks

## 🚀 10-Week Development Plan

### Phase 1: Core Agent + ACP (Weeks 1-4)

#### **Week 1-2: Real Tool Implementation**
**Goal**: Replace stubs with production-quality tools

**File Operations (Week 1):**
- [ ] Real `read_file` - syntax highlighting, context extraction, smart truncation
- [ ] Real `write_file` - backups, safety checks, atomic writes, protected files
- [ ] Real `edit_file` - line-based editing, change validation, diff preview
- [ ] Real `list_files` - gitignore respect, metadata, smart filtering

**Code Understanding (Week 2):**
- [ ] Real `search_code` - leverage semantic search, query expansion, context
- [ ] Real `analyze_code` - AST-based analysis, complexity metrics, patterns
- [ ] Real `find_references` - cross-file symbol tracking, imports/exports
- [ ] Real `get_definition` - symbol lookup with context, type information

**Success Criteria**:
- 8/10 tools real and functional (vs 1/10 currently)
- Can complete basic file/code tasks with real value
- Tools work reliably in test harness

#### **Week 3: ACP Protocol Implementation**
**Goal**: Agent works via ACP in Zed

**Tasks:**
- [ ] stdio transport implementation (JSON-RPC over stdin/stdout)
- [ ] ACP Agent trait full compliance
- [ ] Session management (create, resume, end sessions)
- [ ] Tool execution via ACP protocol
- [ ] Streaming response support
- [ ] Error handling and recovery

**Success Criteria**:
- Aircher agent launchable from Zed
- Basic tool execution works via ACP
- Can maintain conversation state
- Graceful error handling

#### **Week 4: Integration & Testing**
**Goal**: Polished Zed integration

**Tasks:**
- [ ] Test all 8 tools via Zed frontend
- [ ] Fix integration bugs
- [ ] Performance tuning
- [ ] Documentation for Zed users

**Success Criteria**:
- Smooth experience in Zed
- No crashes or data loss
- Clear installation instructions
- Demo-ready quality

### Phase 2: SOTA Architecture Implementation (Weeks 5-8) ✨ UPDATED

#### **Week 5: Working Memory** ✅ COMPLETE
- Dynamic context pruning with relevance scoring
- Integration of all 3 memory systems
- +820 lines production code, +620 lines tests

#### **Week 6: ACP Protocol Enhancements** ✅ COMPLETE
- Session management with 30-minute timeout
- Streaming notifications (5 types)
- Comprehensive error handling with retry logic
- +635 lines production code, +470 lines tests

#### **Week 7: Core Architecture Patterns** (NEW)
**Goal**: Implement OpenCode's proven patterns

**Day 1-2: Event Bus + LSP Integration**
- [ ] Implement tokio broadcast event bus
- [ ] LSP manager with global diagnostics map
- [ ] Real-time feedback loop (edit → LSP → diagnostics → agent)
- [ ] Support Rust, Python, TypeScript, Go

**Day 3-4: Plan/Build Mode Separation**
- [ ] `AgentMode` enum with tool restrictions
- [ ] Plan mode: read-only tools (grep, read, glob, LSP)
- [ ] Build mode: all tools + approval workflow
- [ ] Mode transition logic based on intent

**Day 5: Git Snapshots**
- [ ] SnapshotManager with temporary commits
- [ ] Auto-snapshot before risky operations
- [ ] Rollback on errors or permission rejection
- [ ] Clean history (snapshots don't pollute git log)

**Day 6-7: Model Router**
- [ ] Multi-model selection logic
- [ ] Cost-aware routing (Haiku for simple, Opus for complex)
- [ ] Performance testing across models

**Success Criteria**:
- LSP provides real-time diagnostics after file edits
- Plan mode safely explores without modification risk
- Git snapshots enable safe experimentation
- Model router reduces API costs by 40%

#### **Week 8: Specialized Agents + Sub-Agents** (NEW)
**Goal**: Factory Droid's specialized agent pattern + Claude Code's research sub-agents

**Day 1-2: Agent Configurations**
- [ ] Explorer agent (CodeReading tasks)
- [ ] Builder agent (CodeWriting tasks)
- [ ] Debugger agent (ProjectFixing tasks)
- [ ] Refactorer agent (code improvements)
- [ ] Specialized system prompts per agent

**Day 3-4: Research Sub-Agents**
- [ ] Parallel research spawning (max 10 concurrent)
- [ ] Task decomposition for research queries
- [ ] Result aggregation in main agent
- [ ] Memory integration (prevent duplicate research)

**Day 5-7: Integration Testing**
- [ ] Test Plan mode with sub-agents for research
- [ ] Test Build mode NEVER uses sub-agents (prevent 15x waste)
- [ ] Validate 90% improvement for research tasks
- [ ] Measure token usage: sub-agents vs main agent

**Success Criteria**:
- Specialized agents have focused, effective system prompts
- Research tasks 90% faster with parallel sub-agents
- Coding tasks NEVER spawn sub-agents (avoid 15x overhead)
- Memory prevents duplicate research across sessions

### Phase 3: Validation & Benchmarking (Weeks 9-10) - UPDATED TIMELINE

#### **Week 9: Empirical Comparison vs Claude Code**
**Goal**: Validate improvements with real benchmarks

**Benchmark Tasks** (from research):
1. Multi-file refactoring (measure: tool calls, context efficiency)
2. Bug fixing workflow (measure: time to resolution, relevant files)
3. New feature implementation (measure: code consistency, iterations)
4. Codebase exploration (measure: irrelevant files examined, research deduplication)

**Metrics to Track**:
- Tool calls needed (target: 60% reduction via memory)
- Files examined (target: fewer irrelevant files)
- Context efficiency (dynamic pruning effectiveness)
- Research deduplication (cache hit rate from episodic memory)
- LSP self-correction rate (fixes via diagnostics feedback)
- Sub-agent speedup (research tasks only)

**Success Criteria**:
- 60% reduction in tool calls (memory advantage)
- 90% improvement in research tasks (sub-agents)
- 0% sub-agent usage for coding (avoid 15x waste)
- Measurable LSP self-correction (fewer hallucinations)

#### **Week 10: Research Paper + Release**
**Goal**: Publication-ready research + open source launch

**Paper Sections** (updated):
1. Introduction: Problem with current agent architectures
2. Related Work: Factory Droid, OpenCode, Claude Code, Amp analysis
3. Architecture: Hybrid design combining best patterns
   - Plan/Build separation (OpenCode)
   - Specialized agents (Factory Droid)
   - Smart sub-agents (research only, not coding)
   - LSP integration (real-time feedback)
   - Memory systems (our innovation)
4. Evaluation: Benchmarks vs Claude Code
5. Results: 60% tool reduction, 90% research speedup, LSP self-correction
6. Discussion: When to use sub-agents, when not to
7. Conclusion: Intent-driven hybrid architecture is superior

**Release Checklist**:
- [ ] Research paper draft (arXiv submission)
- [ ] Blog post series (4-5 posts explaining architecture)
- [ ] Documentation (architecture guide, user manual)
- [ ] Video demos (30s teaser, 5min walkthrough)
- [ ] GitHub release (v0.1.0 with release notes)
- [ ] Community announcement (Reddit, HN, Twitter)

**Success Criteria**:
- Paper demonstrates clear advantages over existing tools
- Empirical evidence supports all claims
- Community understands our unique approach
- Open source release is production-ready


## 📈 Success Metrics (Updated)

### Phase 1 Success (Weeks 1-6) ✅ ACHIEVED
- ✅ 6 production tools (2,300+ lines)
- ✅ Memory systems complete (3,725 lines)
- ✅ ACP protocol enhanced (session management, streaming, error handling)
- ✅ 30-33% competitive parity
- ✅ Infrastructure solid

### Phase 2 Success (Weeks 7-8) - NEW GOALS
- ✅ LSP integration provides real-time diagnostics
- ✅ Plan/Build mode separation prevents accidental modifications
- ✅ Git snapshots enable safe experimentation
- ✅ Specialized agents have focused, effective prompts
- ✅ Research sub-agents show 90% improvement
- ✅ Coding tasks NEVER use sub-agents (0% usage)
- ✅ Model router reduces API costs by 40%

### Phase 3 Success (Weeks 9-10) - VALIDATION
- ✅ 60% reduction in tool calls vs Claude Code (memory advantage)
- ✅ 90% improvement in research tasks (parallel sub-agents)
- ✅ 0% sub-agent usage for coding (avoid 15x waste)
- ✅ Measurable LSP self-correction rate
- ✅ Research paper demonstrates unique hybrid architecture
- ✅ Empirical evidence supports all claims
- ✅ Open source release is production-ready

## 🔬 Research Contributions (Updated)

### 1. **Hybrid Agent Architecture** ✨ NEW
**Hypothesis**: Combining patterns from 4 SOTA tools creates superior agent
- **Components**: Plan/Build separation + specialized agents + smart sub-agents + LSP feedback + memory systems
- **Baseline**: Single-strategy agents (Claude Code, Amp, etc.)
- **Measurement**: Tool calls, research speed, code quality, cost
- **Expected**: 60% tool reduction (memory), 90% research speedup (sub-agents), 40% cost reduction (model routing)

### 2. **Intent-Driven Strategy Selection**
**Hypothesis**: User intent determines optimal agent strategy
- **Plan Mode**: Read-only exploration, can spawn research sub-agents
- **Build Mode**: Modification capable, NEVER uses sub-agents
- **Baseline**: One-size-fits-all or naive sub-agent usage
- **Measurement**: Safety (accidental modifications), efficiency (sub-agent usage)
- **Expected**: 0% sub-agents for coding (avoid 15x waste), 90% improvement for research

### 3. **LSP-Augmented Feedback Loop**
**Hypothesis**: Real-time diagnostics prevent hallucination
- **Pattern**: Edit → LSP diagnostic → agent sees errors → self-correct
- **Baseline**: No LSP feedback
- **Measurement**: Self-correction rate, errors caught before execution
- **Expected**: 50% fewer runtime errors via LSP self-correction

### 4. **Memory-Augmented Agent Intelligence**
**Hypothesis**: Three-layer memory enables continuous work
- **Episodic**: Prevent duplicate research
- **Knowledge Graph**: Instant codebase queries
- **Working Memory**: Dynamic context pruning
- **Baseline**: Stateless agent (Claude Code)
- **Measurement**: Research deduplication, tool call reduction, context efficiency
- **Expected**: 60% fewer tool calls, 80% cache hit rate for research

### 5. **Git-Based Safe Experimentation**
**Hypothesis**: Snapshots enable fearless exploration
- **Pattern**: Snapshot → risky operation → rollback on failure
- **Baseline**: No snapshots (permanent changes)
- **Measurement**: Rollback frequency, user confidence
- **Expected**: 100% recovery from failed operations

## 🚫 Anti-Goals (Explicit Non-Priorities)

### Not Building (UI/Frontend)
- ❌ Custom TUI application
- ❌ Terminal rendering or themes
- ❌ Keyboard shortcut systems
- ❌ Custom editor or IDE

**Why**: Zed/JetBrains/Neovim handle this better. We focus on agent intelligence.

### Not Building (Enterprise)
- ❌ Team collaboration features
- ❌ Enterprise SSO/authentication
- ❌ Audit trails and compliance
- ❌ Multi-user permissions
- ❌ Usage analytics dashboards

**Why**: Individual developer productivity, not enterprise platform.

### Not Building (Scope Creep)
- ❌ IDE integrations beyond ACP
- ❌ Browser-based UIs
- ❌ Mobile applications
- ❌ Hosted SaaS offering
- ❌ Non-agent features

**Why**: Stay focused on research contributions in agent intelligence.

## 📋 Weekly Checklist

### Every Week
- [ ] Update NOW.md with current sprint status
- [ ] Document decisions in DECISIONS.md
- [ ] Keep PROJECT_REALITY.md honest (parity estimates)
- [ ] Add discoveries to DISCOVERIES.md
- [ ] Zero compiler warnings maintained

### When Milestones Hit
- [ ] Update competitive parity estimate in PROJECT_REALITY.md
- [ ] Document architectural decisions in DECISIONS.md
- [ ] Add learnings to KNOWLEDGE.md
- [ ] Update this roadmap if priorities change

## 🎯 End Goal (Week 10)

**Deliverables:**
1. **Working Agent**: Usable in Zed, 8+ real tools, intent classification, dynamic context
2. **Research Paper**: "Intent-Driven Agent Architecture for Code Assistants"
3. **Empirical Evidence**: 15-30% improvements in key metrics vs Claude Code
4. **Open Source**: Full implementation on GitHub with documentation
5. **Community**: Installation guide, examples, contributor docs

**Positioning:**
- "Aircher: The smartest ACP-compatible agent backend"
- "Novel intent classification and dynamic context management"
- "Empirically validated improvements over leading competitors"
- "Use in Zed, JetBrains, Neovim, Emacs, or any ACP-compatible frontend"

**Not:**
- "Another terminal UI for coding agents"
- "Yet another editor with AI features"
- "A complete development environment"

## 🔄 Continuous Priorities

Throughout all 10 weeks:

1. **Agent Intelligence First**
   - Every hour on: intent classification, context management, tool quality
   - Zero hours on: UI design, themes, keyboard shortcuts

2. **Research Quality**
   - Rigorous benchmarking vs competitors
   - Ablation studies to prove feature value
   - Publication-grade documentation

3. **ACP Compliance**
   - Follow protocol spec precisely
   - Work in any ACP-compatible frontend
   - No custom extensions

4. **Empirical Validation**
   - Measure everything
   - Honest about what works/doesn't
   - Document failures as well as successes

5. **Open Source Excellence**
   - Zero warnings policy
   - Clear documentation
   - Community-friendly contribution

---

**Next Actions**: See NOW.md for current sprint tasks (Week 1: Real tool implementation)

**Success Metric**: In 10 weeks, working agent in Zed with empirical proof it's smarter than Claude Code's agent.
