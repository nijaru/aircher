# Aircher Intelligence Engine

**Context-Aware Development Assistant for AI Agents**

## Executive Summary

The Intelligence Engine transforms raw codebase data into **contextual insights** that AI agents can use to make informed decisions about what code, files, and context to include in conversations. Unlike traditional code analysis tools, it maintains **development narrative** and **conversational memory** to provide truly relevant context.

## Core Philosophy

### From Search Engine to Research Assistant
- **Not Just**: "What files changed recently?"
- **But Rather**: "What context would help an AI agent understand the current development story?"

### Multi-Dimensional Relevance
Context isn't binary - it's layered, temporal, and purpose-driven.

## Architecture Overview

### Tool-Based Interface
The Intelligence Engine exposes itself as **tool calls** that AI agents can invoke:

```rust
// Core tool interface
pub trait IntelligenceTools {
    async fn get_development_context(&self, query: &str) -> ContextualInsight;
    async fn analyze_change_impact(&self, files: &[String]) -> ImpactAnalysis;
    async fn suggest_missing_context(&self, current_files: &[String]) -> ContextSuggestions;
    async fn track_conversation_outcome(&self, files: &[String], outcome: Outcome) -> ();
    async fn get_project_momentum(&self) -> ProjectMomentum;
}
```

## Core Components

### 1. Contextual Relevance Engine

**Multi-layered scoring system** that understands different types of relevance:

```rust
pub struct ContextualRelevance {
    immediate: f64,     // Files being actively edited/discussed
    sequential: f64,    // Files likely needed next in workflow
    dependent: f64,     // Files that depend on current work
    reference: f64,     // Files that provide conceptual context
    historical: f64,    // Files that show how we got here
}

pub struct ContextualInsight {
    development_phase: String,
    active_story: String,
    key_files: Vec<FileWithContext>,
    architectural_context: Vec<Decision>,
    recent_patterns: Vec<Pattern>,
    suggested_next_actions: Vec<Action>,
    confidence: f64,
}

pub struct FileWithContext {
    path: String,
    relevance: ContextualRelevance,
    purpose: String,           // Why this file matters now
    last_significant_change: DateTime,
    relationship_to_current_work: String,
}
```

### 2. Development Narrative Tracker

**Maintains the story of the codebase**, not just its current state:

```rust
pub struct DevelopmentNarrative {
    current_epic: String,      // What major feature/refactor is happening
    recent_decisions: Vec<ArchitecturalDecision>,
    development_momentum: Vec<MomentumIndicator>,
    knowledge_gaps: Vec<Gap>,
    success_patterns: Vec<Pattern>,
}

pub struct ArchitecturalDecision {
    decision: String,
    rationale: String,
    affected_files: Vec<String>,
    implications: Vec<String>,
    timestamp: DateTime,
}
```

### 3. Conversational Memory System

**Learns from AI conversations** to improve context assembly:

```rust
pub struct ConversationMemory {
    successful_contexts: Vec<ContextOutcome>,
    insufficient_contexts: Vec<ContextGap>,
    user_intent_patterns: Vec<IntentPattern>,
    agent_effectiveness_metrics: AgentMetrics,
}

pub struct ContextOutcome {
    files_included: Vec<String>,
    user_goal: String,
    outcome_quality: f64,
    missing_context_identified: Vec<String>,
    timestamp: DateTime,
}
```

### 4. Predictive Context Assembly

**Anticipates what context will be needed** based on development patterns:

```rust
pub struct ContextPrediction {
    likely_next_files: Vec<String>,
    potential_dependencies: Vec<String>,
    architectural_considerations: Vec<String>,
    historical_context_needs: Vec<String>,
    confidence_scores: HashMap<String, f64>,
}
```

## Tool Call Interface

### Primary Tools for AI Agents

#### 1. `get_development_context(query: &str) -> ContextualInsight`
**Purpose**: Get comprehensive context for current development work
**Input**: Natural language query about what the user is trying to do
**Output**: Structured insight with relevant files, decisions, and narrative

```rust
// Example usage by AI agent
let context = intelligence.get_development_context(
    "User wants to add model selection to the TUI interface"
).await;

// Returns files like:
// - src/ui/mod.rs (immediate - current TUI implementation)
// - src/ui/components/modal.rs (sequential - model selection modal exists)
// - src/providers/mod.rs (dependent - need to understand available models)
// - src/config/mod.rs (reference - model configuration patterns)
```

#### 2. `analyze_change_impact(files: &[String]) -> ImpactAnalysis`
**Purpose**: Understand ripple effects of potential changes
**Input**: Files being considered for modification
**Output**: Analysis of what else might be affected

#### 3. `suggest_missing_context(current_files: &[String]) -> ContextSuggestions`
**Purpose**: Identify gaps in current context
**Input**: Files currently in conversation
**Output**: Suggestions for additional context that might be helpful

#### 4. `track_conversation_outcome(files: &[String], outcome: Outcome)`
**Purpose**: Learn from conversation results to improve future context assembly
**Input**: Files that were included and how successful the outcome was
**Output**: None (updates internal learning models)

#### 5. `get_project_momentum() -> ProjectMomentum`
**Purpose**: Understand overall project direction and recent patterns
**Input**: None
**Output**: High-level project state and momentum indicators

### System Prompt Integration

The Intelligence Engine provides a **dynamic system prompt component**:

```text
## Current Development Context

**Project Phase**: {{development_phase}}
**Active Story**: {{active_story}}

**Key Architecture Decisions**:
{{#each recent_decisions}}
- {{decision}}: {{rationale}}
{{/each}}

**Development Momentum**:
- Recent focus: {{momentum.recent_focus}}
- Next priorities: {{momentum.next_priorities}}
- Architectural direction: {{momentum.architectural_direction}}

**Code Context Guidelines**:
- Primary working files: {{key_files.immediate}}
- Supporting context: {{key_files.reference}}
- Dependencies to consider: {{key_files.dependent}}

Use the Intelligence Engine tools to get specific context for user requests.
```

## Implementation Architecture

### Data Sources
1. **Git History**: Commit patterns, file changes, branch activity
2. **AST Analysis**: Code structure, dependencies, complexity metrics
3. **Conversation Logs**: Previous AI interactions and their outcomes
4. **Project Metadata**: Task status, documentation, architectural decisions

### Storage Strategy
```rust
pub struct IntelligenceStorage {
    // Fast lookup for active context
    context_cache: LruCache<String, ContextualInsight>,

    // Learning models
    pattern_database: SqliteConnection,
    conversation_memory: SqliteConnection,

    // Real-time analysis
    file_watcher: FileWatcher,
    git_monitor: GitMonitor,
}
```

### Performance Characteristics
- **< 50ms** for cached context queries
- **< 200ms** for fresh analysis of medium projects
- **Incremental updates** on file changes
- **Background learning** from conversation outcomes

## Cross-Project Intelligence

### Multi-Project Session Support
Following the Claude Code pattern, Aircher supports adding external directories to intelligence sessions:

```bash
# Add external project directory
aircher --add-project /path/to/other/project

# Analyze patterns across projects
aircher --cross-project-analysis "authentication patterns"
```

### Global Configuration System
Universal AI configuration files for consistent behavior across tools:

```
~/.agent/AGENT.md          # Global AI instructions (like ~/.agents/AGENTS.md)
project-root/AGENT.md      # Project-specific AI instructions
project-root/.cursorrules  # Cursor IDE rules
project-root/.copilot      # GitHub Copilot instructions
```

**Intelligence Engine Discovery Order:**
1. Global `~/.agent/AGENT.md` (user-wide context)
2. Project `AGENT.md` (project-specific instructions)
3. Legacy files: `AGENTS.md`, `.cursorrules`, `.copilot-instructions`
4. Standard patterns: `AI_INSTRUCTIONS.md`, `.ai-instructions`

### Cross-Project Learning
```rust
pub struct CrossProjectIntelligence {
    // Learn patterns that apply across codebases
    universal_patterns: Vec<UniversalPattern>,
    // Architecture decisions that work across projects
    architectural_knowledge: Vec<ArchitecturalKnowledge>,
    // User preferences and coding styles
    user_patterns: Vec<UserPattern>,
}

// Example cross-project insights
let insight = intelligence.analyze_cross_project(
    "How do other projects handle error types?"
).await;
```

## Integration with Aircher

### CLI Integration
```bash
# AI agent tool calls
aircher --intelligence="get context for adding authentication"
aircher --intelligence="analyze impact of changing provider interface"

# Cross-project capabilities
aircher --add-project ~/other-rust-project
aircher --cross-project="error handling patterns"
```

### TUI Integration
- **Context sidebar** showing current development narrative
- **Smart file suggestions** based on current conversation
- **Progress indicators** for project momentum
- **Cross-project panel** showing insights from other projects

### Agent Integration
```rust
// In AI agent conversation loop
let context = intelligence_engine
    .get_development_context(&user_request)
    .await?;

// Use context to inform file selection and conversation framing
let relevant_files = context.key_files
    .iter()
    .filter(|f| f.relevance.immediate > 0.7)
    .collect();
```

## Success Metrics

### Effectiveness Measures
- **Context Relevance**: How often suggested files are actually useful
- **Conversation Success**: Improved task completion rates
- **Development Velocity**: Faster problem resolution
- **Learning Quality**: Improvement in context suggestions over time

### Performance Targets
- **Accuracy**: >85% of suggested files are relevant to user goals
- **Coverage**: <5% of conversations need additional context requests
- **Efficiency**: Average context assembly in <100ms
- **Learning**: 10% improvement in relevance scores per month

## Future Enhancements

### Advanced Capabilities
- **Cross-project learning**: Patterns that apply across codebases
- **Team intelligence**: Learning from multiple developers' patterns
- **Architectural guidance**: Suggesting refactoring opportunities
- **Documentation generation**: Auto-updating architectural decisions

### Integration Opportunities
- **IDE plugins**: Real-time context in development environment
- **CI/CD integration**: Context-aware code review suggestions
- **Documentation tools**: Auto-generating context-aware docs

## Implementation Phases

### Phase 1: Core Intelligence (SPRINT-004)
- Basic contextual relevance engine
- Simple development narrative tracking
- Tool call interface implementation

### Phase 2: Learning System (SPRINT-010)
- Conversational memory implementation
- Pattern recognition and learning
- Predictive context assembly

### Phase 3: Advanced Features (SPRINT-015)
- Cross-project intelligence
- Advanced architectural guidance
- Real-time development assistance

---

**The Intelligence Engine transforms Aircher from a chat tool into a development partner that truly understands your codebase and development goals.**
