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

### ACP Frontend Landscape ✅
- **Zed**: Native ACP support (best starting point)
- **JetBrains**: October 2025 collaboration announced (huge opportunity)
- **Neovim**: 2 plugins already (CodeCompanion, avante.nvim)
- **Emacs**: agent-shell plugin
- **1k+ GitHub stars** on ACP protocol repo

**Validation**: Strong ecosystem adoption confirms ACP-first strategy is correct

### What Actually Works
- **Semantic Search**: Production-ready (6,468 vectors, 19+ languages, sub-second)
- **Intelligence Framework**: 210+ Rust files with substantial implementation
- **Multi-Provider**: OpenAI, Anthropic, Gemini, Ollama authentication
- **ACP Architecture**: Designed (needs implementation)
- **Dynamic Context**: Code exists (needs integration testing)

### What Doesn't Work
- **9/10 tools are stubs** - return fake JSON, no real functionality
- **ACP Protocol**: Not implemented yet
- **Intent Classification**: Code exists but not operational
- **Benchmarks**: No empirical validation vs competitors

### Competitive Position
- **16-20% feature parity** with Claude Code (infrastructure vs capabilities)
- **Strong foundation** but minimal user value currently
- **Novel architecture** (intent classification, dynamic context) not yet proven

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

### Phase 2: Intelligence Features (Weeks 5-6)

#### **Week 5: Intent Classification**
**Goal**: Automatic task type detection working

**Tasks:**
- [ ] Make `UserIntent` classification operational
- [ ] Connect to unified intelligence engine
- [ ] Different execution strategies per intent type
- [ ] Confidence scoring

**Success Criteria**:
- Correctly classifies: CodeReading, CodeWriting, ProjectFixing, ProjectExploration
- Routes to appropriate execution strategy
- Measurable accuracy >80%

#### **Week 6: Dynamic Context Management**
**Goal**: Prove superiority over sub-agents

**Tasks:**
- [ ] Activate DynamicContextManager
- [ ] Smart pruning and prefetching
- [ ] Relevance scoring operational
- [ ] Token budget optimization

**Success Criteria**:
- Context efficiency measurably better
- No context overflow on complex tasks
- Maintains relevant context across turns

### Phase 3: Validation & Benchmarking (Weeks 7-8)

#### **Week 7: Empirical Comparison**
**Goal**: Measure vs Claude Code

**Benchmark Tasks:**
1. Multi-file refactoring (measure tool calls, success rate)
2. Bug fixing (measure time to resolution, context efficiency)
3. Code generation (measure style consistency)
4. Codebase exploration (measure relevant context retrieval)

**Metrics:**
- Tool calls needed (fewer = better)
- Context tokens used (fewer = better with same results)
- Task success rate (higher = better)
- Time to completion (faster = better)

**Success Criteria**:
- Document 15-30% improvement in at least 2 metrics
- Validate 19% context efficiency claim
- Identify weaknesses for improvement

#### **Week 8: Ablation Studies**
**Goal**: Prove each feature's value

**Tests:**
1. Intent classification ON vs OFF
2. Dynamic context vs static context window
3. Pattern learning ON vs OFF

**Success Criteria**:
- Each feature shows measurable improvement
- Understand which features matter most
- Data for research paper

### Phase 4: Research & Release (Weeks 9-10)

#### **Week 9: Documentation & Paper Draft**
**Goal**: Publishable research contribution

**Deliverables:**
- [ ] Research paper draft (intent-driven agent architecture)
- [ ] Technical documentation (how it works)
- [ ] Blog post (for broader audience)
- [ ] Benchmark results tables and graphs

**Paper Sections:**
1. Introduction (problem: one-size-fits-all agents)
2. Related Work (ReAct, Reflexion, sub-agents)
3. Architecture (intent classification + dynamic context)
4. Evaluation (empirical comparison vs Claude Code)
5. Results (improvements in metrics)
6. Discussion (when it works, when it doesn't)
7. Conclusion (contributions to field)

#### **Week 10: Open Source Release**
**Goal**: Community adoption

**Deliverables:**
- [ ] Installation guide for Zed users
- [ ] Contributor documentation
- [ ] Example usage patterns
- [ ] Demo video
- [ ] Reddit/HN announcement

**Success Criteria**:
- Clear installation path
- Working examples
- Community can contribute

## 📈 Success Metrics

### Phase 1 Success (Week 4)
- ✅ 8 real tools providing value
- ✅ Works smoothly in Zed via ACP
- ✅ No crashes, good UX
- ✅ Demo-ready quality

### Phase 2 Success (Week 6)
- ✅ Intent classification >80% accuracy
- ✅ Dynamic context measurably better than static
- ✅ Pattern learning showing improvement
- ✅ 40-50% competitive parity

### Phase 3 Success (Week 8)
- ✅ 15-30% improvement in key metrics vs Claude Code
- ✅ 19% context efficiency claim validated
- ✅ Ablation studies prove feature value
- ✅ Benchmark data ready for paper

### Phase 4 Success (Week 10)
- ✅ Research paper draft complete
- ✅ Open source release with docs
- ✅ Community can install and use
- ✅ Clear research contributions documented

## 🔬 Research Contributions

### 1. **Intent-Driven Agent Architecture**
**Hypothesis**: Automatic intent classification enables specialized execution strategies
- **Baseline**: One-size-fits-all agent approach
- **Measurement**: Task completion rate, tool efficiency
- **Expected**: 15-30% improvement for specialized tasks

### 2. **Dynamic Context Management**
**Hypothesis**: Single agent with smart context > multiple sub-agents
- **Baseline**: Sub-agent architecture (like Claude Code)
- **Measurement**: Context efficiency, coordination overhead
- **Expected**: Confirm 19% performance advantage

### 3. **Pattern-Aware Code Generation**
**Hypothesis**: Learning project conventions improves code quality
- **Baseline**: Agent without pattern learning
- **Measurement**: Style consistency, architectural compliance
- **Expected**: 40-60% better matching to existing code

### 4. **Unified Intelligence Middleware**
**Hypothesis**: Automatic intelligence routing reduces latency
- **Baseline**: Manual intelligence tool invocation
- **Measurement**: Time to first useful response
- **Expected**: 2-3x faster with automatic detection

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
