# Aircher SOTA System Design (Updated Oct 2025)

**Status**: Research-driven redesign based on Factory Droid, OpenCode, Claude Code, and Amp analysis

## Executive Summary

Aircher will adopt a **hybrid agent architecture** combining the best patterns from SOTA tools:
- **OpenCode's** plan/build separation + LSP integration + event bus
- **Factory Droid's** specialized agents for different tasks
- **Claude Code's** research sub-agents (but NOT for coding)
- **Amp's** multi-model flexibility
- **Our innovation**: Memory systems + intent classification drive strategy selection

## Core Architecture Principles

### 1. **Plan Mode vs Build Mode** (from OpenCode)

```
┌─────────────────────────────────────────┐
│           User Request                   │
└─────────────────┬───────────────────────┘
                  │
                  ▼
         ┌────────────────┐
         │ Intent Classify │
         └────────┬───────┘
                  │
        ┌─────────┴─────────┐
        ▼                   ▼
   PLAN MODE           BUILD MODE
  (Read-Only)        (Can Modify)
        │                   │
        ▼                   ▼
  • Explore code       • Edit files
  • Analyze patterns   • Run commands
  • Research           • Write code
  • Report findings    • Execute tools
        │                   │
        └─────────┬─────────┘
                  ▼
         Exit with results
```

**Why This Matters**:
- **Safety**: Read-only mode can't accidentally break things
- **Performance**: Different tool permissions = smaller context
- **UX**: Clear distinction between analysis and modification
- **OpenCode validation**: This pattern works in production

**Implementation**:
```rust
pub enum AgentMode {
    Plan {
        tools: Vec<Tool>,  // grep, read, glob, LSP (read-only)
        can_spawn_subagents: true,
    },
    Build {
        tools: Vec<Tool>,  // All tools including write, edit, bash
        requires_approval: Vec<String>,  // Dangerous operations
    },
}

impl Agent {
    pub async fn execute(&self, request: UserRequest) -> Result<Response> {
        // Classify intent
        let intent = self.intelligence.classify_intent(&request.content).await?;

        // Select mode based on intent
        let mode = match intent {
            UserIntent::CodeReading { .. } => AgentMode::Plan,
            UserIntent::ProjectExploration { .. } => AgentMode::Plan,
            UserIntent::CodeWriting { .. } => AgentMode::Build,
            UserIntent::ProjectFixing { .. } => AgentMode::Build,
            UserIntent::Mixed { .. } => {
                // Start in Plan, explicit transition to Build
                AgentMode::Plan
            }
        };

        self.execute_with_mode(request, mode).await
    }
}
```

### 2. **Specialized Agents** (from Factory Droid)

Instead of one monolithic agent, we have **specialized configurations**:

```rust
pub enum AgentType {
    // Primary agents
    Explorer,      // CodeReading: Research, analyze, understand
    Builder,       // CodeWriting: Implement features
    Debugger,      // ProjectFixing: Fix bugs, errors
    Refactorer,    // Code improvements, migrations

    // Specialized sub-agents (spawned as needed)
    FileSearcher,  // Parallel file content search
    PatternFinder, // Find code patterns across codebase
    DependencyMapper, // Trace dependencies
    TestRunner,    // Run tests, analyze failures
    DocumentationGenerator, // Generate docs
}

pub struct AgentConfig {
    pub agent_type: AgentType,
    pub system_prompt: String,
    pub allowed_tools: Vec<String>,
    pub max_steps: usize,
    pub memory_access: MemoryAccessLevel,
}
```

**Key Insight from Factory Droid**: Pre-configured agents with specialized prompts outperform generic agents because:
- Focused system prompts (no "you can do everything" dilution)
- Smaller tool sets = less decision paralysis
- Optimized for specific tasks

### 3. **Hybrid Sub-Agent Strategy** (from Claude Code research)

**Decision Matrix**:

| Task Type | Strategy | Reason |
|-----------|----------|--------|
| **Coding** (write/edit) | ❌ **NO sub-agents** | 15x token waste, context isolation fatal |
| **Research** (search/analyze) | ✅ **Spawn sub-agents** | 90% improvement, parallel execution |
| **Testing** | ✅ **Sub-agent** | Isolated test runs, can fail safely |
| **Documentation** | ✅ **Sub-agent** | Can analyze without polluting main context |

**Implementation**:
```rust
impl Agent {
    pub async fn execute_research_task(&self, query: &str) -> Result<ResearchResults> {
        // Break into parallel subtasks
        let subtasks = self.decompose_research_query(query).await?;

        // Spawn sub-agents (max 10 concurrent)
        let handles: Vec<_> = subtasks.into_iter()
            .take(10)  // Claude Code limit
            .map(|task| {
                let config = AgentConfig {
                    agent_type: AgentType::FileSearcher,
                    system_prompt: format!("Find: {}", task.query),
                    allowed_tools: vec!["grep", "read", "glob"],
                    max_steps: 50,
                    memory_access: MemoryAccessLevel::ReadOnly,
                };

                tokio::spawn(async move {
                    let agent = SubAgent::new(config);
                    agent.research(task).await
                })
            })
            .collect();

        // Aggregate results
        let results = join_all(handles).await?;

        // Save to episodic memory (prevent duplicate research)
        self.memory.record_research_session(query, &results).await?;

        Ok(results)
    }

    pub async fn execute_coding_task(&self, task: &CodingTask) -> Result<CodeChanges> {
        // NEVER spawn sub-agents for coding
        // Use main agent with dynamic context management
        self.execute_with_main_agent(task).await
    }
}
```

### 4. **LSP Integration with Event Bus** (from OpenCode)

**Architecture**:
```
┌──────────────────────────────────────────────┐
│            Agent Core                         │
└────────────────┬─────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────────┐
│         Event Bus (tokio broadcast)           │
│  • FileChanged events                         │
│  • DiagnosticsReceived events                │
│  • TestResults events                         │
│  • ToolExecuted events                        │
└────────┬─────────────────────────────────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌─────────┐ ┌─────────────────┐
│   LSP   │ │  Tool Registry  │
│ Manager │ │                 │
└─────────┘ └─────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  Global Diagnostics Map              │
│  HashMap<FilePath, Vec<Diagnostic>>  │
└─────────────────────────────────────┘
```

**Why This Matters**:
- **Real-time feedback**: LSP diagnostics immediately after edits
- **Prevents hallucination**: Model sees type errors, can self-correct
- **Multi-language**: Works for any language with LSP server

**Implementation**:
```rust
pub struct LspManager {
    servers: HashMap<Language, LspServer>,
    diagnostics: Arc<RwLock<HashMap<PathBuf, Vec<Diagnostic>>>>,
    event_bus: broadcast::Sender<Event>,
}

impl LspManager {
    pub async fn on_file_changed(&self, path: &Path) -> Result<()> {
        // Notify LSP server
        if let Some(server) = self.get_server_for_file(path) {
            server.notify_did_change(path).await?;
        }

        // Wait for diagnostics (with timeout)
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if self.diagnostics.read().await.contains_key(path) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }).await?;

        // Broadcast diagnostics to agent
        let diags = self.diagnostics.read().await.get(path).cloned();
        if let Some(diags) = diags {
            self.event_bus.send(Event::DiagnosticsReceived {
                path: path.to_path_buf(),
                diagnostics: diags,
            })?;
        }

        Ok(())
    }
}
```

### 5. **Git-Based Snapshots** (from OpenCode)

**Pattern**: Before risky operations, create temporary Git commits (not in history)

```rust
pub struct SnapshotManager {
    repo: Repository,
}

impl SnapshotManager {
    pub async fn create_snapshot(&self, message: &str) -> Result<Oid> {
        // Create temporary commit (detached HEAD)
        let oid = self.repo.head()?.peel_to_commit()?.id();

        // Save current state
        let mut index = self.repo.index()?;
        index.add_all(["*"], git2::IndexAddOption::DEFAULT, None)?;
        let tree_id = index.write_tree()?;

        // Create temp commit
        let tree = self.repo.find_tree(tree_id)?;
        let parent = self.repo.head()?.peel_to_commit()?;
        let signature = Signature::now("aircher-snapshot", "noreply@aircher")?;

        let commit_id = self.repo.commit(
            None,  // Don't update HEAD
            &signature,
            &signature,
            message,
            &tree,
            &[&parent],
        )?;

        Ok(commit_id)
    }

    pub async fn rollback(&self, snapshot: Oid) -> Result<()> {
        // Restore to snapshot (hard reset)
        let commit = self.repo.find_commit(snapshot)?;
        self.repo.reset(commit.as_object(), ResetType::Hard, None)?;
        Ok(())
    }
}
```

**Use Cases**:
- Before executing bash commands
- Before bulk file edits
- Before running migrations
- After permission rejections (auto-rollback)

### 6. **Multi-Model Architecture** (from Amp)

**Pattern**: Different models for different tasks

```rust
pub struct ModelRouter {
    models: HashMap<AgentType, ModelConfig>,
}

impl ModelRouter {
    pub fn select_model(&self, agent_type: AgentType, task: &Task) -> ModelConfig {
        match (agent_type, task.complexity) {
            // Fast models for simple tasks
            (AgentType::Explorer, Complexity::Low) => ModelConfig {
                provider: "anthropic",
                model: "claude-sonnet-4",  // Fast, cheap
            },

            // Powerful models for complex reasoning
            (AgentType::Builder, Complexity::High) => ModelConfig {
                provider: "anthropic",
                model: "claude-opus-4.1",  // Best reasoning
            },

            // Parallel sub-agents use fast models
            (AgentType::FileSearcher, _) => ModelConfig {
                provider: "anthropic",
                model: "claude-haiku",  // Cheap parallelization
            },

            // User override
            _ if task.model_override.is_some() => task.model_override.clone().unwrap(),

            // Default
            _ => self.models.get(&agent_type).cloned().unwrap_or_default(),
        }
    }
}
```

### 7. **Memory Systems Integration** (Our Advantage)

**Key Innovation**: Use our 3-layer memory to prevent duplicate work

```rust
impl Agent {
    pub async fn should_research(&self, query: &str) -> Result<ResearchDecision> {
        // Check episodic memory
        let past_research = self.memory.episodic
            .find_similar_research(query, 0.85)  // 85% similarity
            .await?;

        if let Some(past) = past_research {
            // Check if still recent (within 1 hour)
            if past.timestamp.elapsed() < Duration::from_secs(3600) {
                return Ok(ResearchDecision::UseCache(past.results));
            }
        }

        // Check knowledge graph
        let graph_has_info = self.memory.knowledge_graph
            .can_answer_query(query)
            .await?;

        if graph_has_info {
            let answer = self.memory.knowledge_graph
                .query(query)
                .await?;
            return Ok(ResearchDecision::UseGraph(answer));
        }

        // Need to research
        Ok(ResearchDecision::Research)
    }
}
```

**Impact**: Sub-agents don't repeat research that we already did

## Updated Week 7-10 Plan

### Week 7: Architecture Refactor

**Day 1-2: Event Bus + LSP Integration**
- Implement tokio broadcast event bus
- LSP manager with diagnostics map
- Integration with edit_file tool

**Day 3-4: Plan/Build Mode Separation**
- AgentMode enum with tool restrictions
- Mode transition logic
- System prompt per mode

**Day 5: Git Snapshots**
- SnapshotManager implementation
- Auto-snapshot before risky operations
- Rollback on errors

**Day 6-7: Model Router**
- Multi-model selection logic
- Cost-aware routing
- Performance testing

### Week 8: Specialized Agents + Sub-Agents

**Day 1-2: Agent Configs**
- Explorer, Builder, Debugger, Refactorer configs
- Specialized system prompts
- Tool permission sets

**Day 3-4: Research Sub-Agents**
- Parallel research spawning
- Result aggregation
- Memory integration (prevent duplicates)

**Day 5-7: Testing**
- End-to-end testing
- Compare: main agent vs sub-agents for different tasks
- Validate 90% improvement claim for research

### Week 9: Benchmarks vs Claude Code

**Day 1-3: Benchmark Implementation**
- 4 benchmark tasks (research found)
- Metrics: tool calls, files examined, success rate, time
- Run against Claude Code

**Day 4-5: Analysis**
- Compare results
- Document improvements
- Identify weaknesses

**Day 6-7: Refinements**
- Fix issues found in benchmarks
- Optimize based on results

### Week 10: Research Paper + Release

**Day 1-3: Paper Writing**
- Sections: intro, related work, architecture, evaluation, results
- Graphs and tables
- Submission-ready draft

**Day 4-5: Documentation**
- User guides
- Architecture docs
- API reference

**Day 6-7: Release**
- GitHub release
- Blog posts
- Community announcement

## Key Metrics to Track

### Performance Metrics
- **Tool calls**: Target 60% reduction vs Claude Code
- **Context efficiency**: Dynamic pruning effectiveness
- **Research deduplication**: Cache hit rate from memory

### Quality Metrics
- **Code quality**: Match existing patterns (from pattern learning)
- **Error rate**: Self-correction via LSP feedback
- **Success rate**: Task completion without human intervention

### Cost Metrics
- **Token usage**: Multi-model routing savings
- **API costs**: Haiku for sub-agents, Opus for complex tasks
- **Time to completion**: Parallel sub-agents speed

## Competitive Advantages (Updated)

| Feature | Aircher | Claude Code | Factory Droid | OpenCode | Amp |
|---------|---------|-------------|---------------|----------|-----|
| **Plan/Build Separation** | ✅ | ❌ | ❌ | ✅ | ❌ |
| **Memory Systems** | ✅ (3-layer) | ❌ | ❌ | ❌ | ❌ |
| **Smart Sub-Agents** | ✅ (research only) | ⚠️ (naive) | ✅ | ⚠️ (Task tool) | ❌ |
| **LSP Integration** | ✅ (planned) | ❌ | ❌ | ✅ | ❌ |
| **Git Snapshots** | ✅ (planned) | ❌ | ❌ | ✅ | ❌ |
| **Multi-Model** | ✅ (planned) | ❌ | ❌ | ❌ | ✅ |
| **ACP Multi-Frontend** | ✅ | ❌ | ❌ | ❌ | ❌ |

**Our Unique Combination**:
- ✅ Memory prevents duplicate work (nobody else has this)
- ✅ Intent-driven strategy selection (plan vs build vs research)
- ✅ LSP feedback loop (prevents hallucination)
- ✅ Git snapshots (safe experimentation)
- ✅ Smart sub-agents (research only, not coding)

## Research Paper Angle (Updated)

**Title**: "Intent-Driven Hybrid Agent Architecture: Combining Memory, Mode Separation, and Selective Parallelization"

**Key Contributions**:
1. **Intent classification** drives architecture selection (plan/build/research)
2. **Memory systems** prevent duplicate work across sessions
3. **Hybrid sub-agent strategy**: Use for research (90% gain), avoid for coding (15x waste)
4. **LSP integration** provides real-time feedback loop
5. **Empirical validation**: 60% fewer tool calls vs Claude Code

**Novelty**: First agent to combine all these patterns in one system

## Implementation Priority

**Must Have (Week 7)**:
1. Event bus + LSP integration
2. Plan/Build mode separation
3. Git snapshots

**Should Have (Week 8)**:
1. Specialized agent configs
2. Research sub-agents
3. Model router

**Nice to Have (Week 9-10)**:
1. Advanced memory optimizations
2. Visual graph explorer
3. Team collaboration features

---

**Next Steps**: Implement Week 7 Day 1-2 (Event Bus + LSP) starting now.
