# Turbo Mode Architecture

## Overview

Turbo mode transforms Aircher from a single-conversation assistant into an intelligent task orchestrator that can break down complex requests into parallel subtasks, each handled by specialized AI agents with fresh context windows.

## Core Concept

When turbo mode is active, the main LLM can spawn subtask conversations with other LLMs to gather information, perform research, or handle specific tasks. Results are intelligently summarized and returned to the main conversation.

## Architecture Design

### Two-Tier Model Configuration

To keep configuration simple while providing flexibility:

- **High Model**: Primary orchestrator (Claude Opus 4.1, Sonnet 4, GPT-4, GLM 4.5)
  - Handles main conversation
  - Breaks down complex tasks
  - Synthesizes results from subtasks
  - Makes key decisions

- **Low Model**: Subtask executor (Claude Haiku, GPT-4o-mini, GLM 4.5 Air)
  - Handles research tasks
  - Performs data gathering
  - Executes defined subtasks
  - Provides cost-effective processing

### Task Orchestration Flow

```
User Request → Main LLM (High Model) → Task Analysis
                    ↓
            [Break down into subtasks]
                    ↓
    ┌──────────────┴──────────────┐
    ↓              ↓              ↓
Subtask 1      Subtask 2      Subtask 3
(Low Model)    (Low Model)    (High Model for complex)
    ↓              ↓              ↓
    └──────────────┴──────────────┘
                    ↓
            [Summarize Results]
                    ↓
            Return to Main Conversation
```

## Implementation Strategy

### Phase 1: Foundation (Quick Win)
- Extend current turbo_mode to bypass approval prompts
- Enable parallel tool execution
- Add simple task patterns (research, analysis, code review)
- Estimated effort: 1-2 days

### Phase 2: Task Orchestration
- Implement TaskOrchestrator for subtask management
- Add progress tracking UI (tree view of subtasks)
- Enable high/low model configuration
- Add result summarization pipeline
- Estimated effort: 3-5 days

### Phase 3: Advanced Features
- Hierarchical task trees (subtasks spawning sub-subtasks)
- Smart result caching and reuse
- Adaptive model selection based on task complexity
- Cost optimization strategies
- Estimated effort: 1-2 weeks

## Technical Implementation

### Core Components

```rust
// src/agent/orchestrator.rs
pub struct TaskOrchestrator {
    main_controller: AgentController,
    subtask_pool: Vec<SubTask>,
    high_model: String,
    low_model: String,
    max_parallel: usize,
}

pub struct SubTask {
    id: String,
    prompt: String,
    context_budget: usize,  // Dedicated context for this subtask
    model: String,          // High or low tier
    status: TaskStatus,
    result: Option<TaskResult>,
}

pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}
```

### Task Patterns

Common patterns that can be automatically handled:

1. **Research Pattern**
   ```rust
   ResearchTask {
       query: String,
       sources: Vec<Source>,
       depth: ResearchDepth,
   }
   ```

2. **Analysis Pattern**
   ```rust
   AnalysisTask {
       target: String,
       aspects: Vec<AnalysisAspect>,
       detail_level: DetailLevel,
   }
   ```

3. **Code Review Pattern**
   ```rust
   CodeReviewTask {
       files: Vec<PathBuf>,
       checks: Vec<ReviewCheck>,
       severity: SeverityLevel,
   }
   ```

## UI/UX Design

### Progress Display

```
User: "Research similar projects and explain how we can apply their patterns"

┌─────────────────────────────────────┐
│ 🚀 TURBO MODE: Breaking down task... │
│                                      │
│ ├─ 📊 Analyzing competitors          │
│ │  └─ Found 5 similar projects...   │
│ ├─ 🔍 Extracting patterns           │
│ │  └─ Identified 3 key patterns...  │
│ └─ 💡 Generating recommendations    │
│     └─ Created implementation plan...│
└─────────────────────────────────────┘
```

### Status Indicators

- `⏳` Pending
- `🔄` Running
- `✅` Completed
- `❌` Failed
- `⚡` Using high model
- `💰` Using low model (cost-optimized)

## Example Use Cases

### Research Tasks
**User**: "Find and analyze similar open source projects"

**Turbo Mode Process**:
1. Main LLM creates research plan
2. Spawns 3 parallel searches:
   - GitHub trending projects (low model)
   - Technical blog posts (low model)
   - Academic papers (low model)
3. Main LLM analyzes and synthesizes findings
4. Returns comprehensive analysis

### Code Review
**User**: "Review this codebase for security issues"

**Turbo Mode Process**:
1. Main LLM creates security checklist
2. Spawns parallel checks:
   - SQL injection patterns (low model)
   - Authentication issues (low model)
   - Input validation (low model)
   - Complex vulnerability analysis (high model)
3. Main LLM prioritizes and explains findings
4. Returns actionable security report

### Documentation Generation
**User**: "Create comprehensive docs for this project"

**Turbo Mode Process**:
1. Main LLM analyzes codebase structure
2. Spawns documentation tasks:
   - API documentation (low model)
   - Setup guide (low model)
   - Architecture overview (high model)
   - Usage examples (low model)
3. Main LLM ensures consistency
4. Returns complete documentation set

## Cost Optimization

### Model Selection Strategy

```rust
fn select_model_for_task(task: &SubTask) -> ModelTier {
    match task.complexity {
        Complexity::Low => ModelTier::Low,    // Simple data gathering
        Complexity::Medium => {
            if task.requires_reasoning {
                ModelTier::High
            } else {
                ModelTier::Low
            }
        },
        Complexity::High => ModelTier::High,   // Complex analysis
    }
}
```

### Cost Tracking

- Display estimated cost before execution
- Track actual costs per subtask
- Provide cost breakdown in results
- Allow cost limits per conversation

## Error Handling

### Subtask Failure Strategy

1. **Retry with backoff**: Transient failures
2. **Escalate to high model**: If low model fails
3. **Graceful degradation**: Continue with partial results
4. **User notification**: Clear explanation of what failed

### Context Management

- Each subtask gets isolated context
- No cross-contamination between subtasks
- Main conversation context preserved
- Smart summarization to fit results back

## Comparison with Other Tools

| Feature | Claude Code | Cursor | Continue | Aircher Turbo |
|---------|------------|---------|----------|---------------|
| Task orchestration | Beta "agents" | No | No | Core feature |
| Parallel execution | Limited | No | No | Yes |
| Context isolation | No | No | No | Yes |
| Model mixing | No | Limited | No | Yes |
| Progress visibility | Minimal | No | No | Full tree |
| Cost optimization | No | No | No | Yes |

## Security Considerations

- Subtask isolation prevents prompt injection spreading
- Result validation before integration
- Rate limiting on subtask spawning
- Audit trail of all subtask operations

## Future Enhancements

1. **Learning System**: Track successful patterns for reuse
2. **Custom Task Templates**: User-defined orchestration patterns
3. **Model Marketplace**: Support for specialized models per task type
4. **Collaborative Mode**: Multiple users contributing to subtasks
5. **Export/Import**: Save and share orchestration patterns

## Configuration

```toml
[turbo_mode]
enabled = true
high_model = "claude-opus-4.1"
low_model = "claude-haiku"
max_parallel_tasks = 5
max_subtask_depth = 3
cost_limit_per_session = 5.00
show_progress_tree = true
auto_summarize = true
```

## Metrics and Monitoring

Track and optimize:
- Task completion rates
- Average subtasks per request
- Cost per task type
- Time to completion
- Model selection accuracy
- User satisfaction scores

This architecture positions Aircher as a next-generation AI assistant that treats complex task orchestration as a first-class citizen, providing users with unprecedented power while maintaining simplicity through the two-tier model configuration.