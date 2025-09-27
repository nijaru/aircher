# Autonomous Agent Intelligence Design

*TUI-focused intelligent agent architecture for Aircher*

## Core Philosophy

**Intelligence, Not Complexity**: Build autonomous intelligence by enhancing existing capabilities, not replacing them.

**TUI-First**: Fast, offline, keyboard-first intelligence that complements the terminal workflow.

**Incremental Learning**: Agent gets smarter through interaction without requiring complex training.

## Architecture Overview

```rust
pub struct AgentIntelligence {
    // Leverage existing semantic search capabilities
    semantic_search: Arc<SemanticSearchEngine>,
    
    // Minimal new intelligence components
    project_memory: ProjectMemory,
    intent_analyzer: IntentAnalyzer,
    prompt_enhancer: PromptEnhancer,
}
```

## Core Components

### 1. Project Memory (Simple Learning)

**Purpose**: Remember what works for this project and user.

```rust
pub struct ProjectMemory {
    store: redb::Database,    // Rust-native embedded DB
    cache_dir: PathBuf,       // JSON exports for debugging
}

// Simple, effective storage
impl ProjectMemory {
    // Remember successful patterns
    pub async fn remember_success(&self, context: &str, action: &str, outcome: Quality);
    
    // Recall similar past actions
    pub async fn recall_similar(&self, context: &str) -> Vec<PastAction>;
    
    // Learn user preferences
    pub async fn learn_preference(&self, pref_type: PrefType, value: String);
}
```

**Storage Schema**:
```rust
#[derive(Serialize, Deserialize)]
pub struct PastAction {
    context: String,           // "fix rust compilation error"
    action: String,            // "check Cargo.toml dependencies"
    outcome: Quality,          // Success/Partial/Failed
    timestamp: DateTime<Utc>,
    effectiveness_score: f32,  // 0.0-1.0
}
```

### 2. Intent Analyzer (Context Understanding)

**Purpose**: Understand what the user wants to accomplish.

```rust
pub struct IntentAnalyzer {
    patterns: HashMap<TaskType, Vec<Pattern>>,
}

#[derive(Debug, Clone)]
pub enum TaskType {
    Debug,           // "fix this error"
    Implement,       // "add feature X"
    Explain,         // "how does this work"
    Refactor,        // "improve this code"
    Search,          // "find where X is defined"
}
```

**Simple Pattern Matching**:
```rust
impl IntentAnalyzer {
    pub fn analyze(&self, message: &str) -> Intent {
        // Simple keyword-based classification
        let words = message.to_lowercase();
        
        if words.contains("error") || words.contains("fix") {
            Intent::Debug
        } else if words.contains("add") || words.contains("implement") {
            Intent::Implement
        } // ... etc
    }
}
```

### 3. Prompt Enhancer (Contextual Intelligence)

**Purpose**: Enhance prompts with project context and past learnings.

```rust
pub struct PromptEnhancer {
    project_context: ProjectContext,
    memory: Arc<ProjectMemory>,
}

impl PromptEnhancer {
    pub async fn enhance_prompt(
        &self, 
        user_message: &str,
        semantic_context: &[SearchResult],
        intent: &Intent
    ) -> Result<String> {
        let base_prompt = self.get_base_prompt_for_intent(intent);
        let project_info = self.get_project_context_summary();
        let relevant_memory = self.memory.recall_similar(&user_message).await?;
        let semantic_summary = self.summarize_semantic_context(semantic_context);
        
        format!(
            "{}\n\nProject Context:\n{}\n\nRelevant Code:\n{}\n\nPast Success Patterns:\n{}\n\nUser Query: {}",
            base_prompt, project_info, semantic_summary, 
            self.format_memory(&relevant_memory), user_message
        )
    }
}
```

## Integration with Existing Systems

### Enhanced Agent Controller

```rust
impl AgentController {
    pub async fn process_message_intelligently(&mut self, message: &str) -> Result<String> {
        // 1. Understand what user wants
        let intent = self.intelligence.analyze_intent(message).await?;
        
        // 2. Gather context from multiple sources
        let semantic_results = self.semantic_search.search(message, 5).await?;
        let memory_context = self.intelligence.recall_relevant(&intent).await?;
        
        // 3. Create enhanced prompt
        let enhanced_prompt = self.intelligence.enhance_prompt(
            message, &semantic_results, &intent
        ).await?;
        
        // 4. Execute with better context
        let response = self.execute_with_enhanced_context(enhanced_prompt).await?;
        
        // 5. Learn from the interaction
        self.intelligence.learn_from_interaction(&intent, &response).await?;
        
        Ok(response)
    }
    
    // Keep existing functionality as fallback
    pub async fn process_message(&mut self, message: &str) -> Result<String> {
        if self.intelligence_enabled() {
            self.process_message_intelligently(message).await
        } else {
            self.process_message_legacy(message).await
        }
    }
}
```

### Intelligent Tool Enhancement

```rust
// Enhance existing tools without rewriting them
pub struct IntelligentToolWrapper<T: AgentTool> {
    base_tool: T,
    intelligence: Arc<AgentIntelligence>,
}

impl<T: AgentTool> AgentTool for IntelligentToolWrapper<T> {
    async fn execute(&self, params: ToolParams) -> Result<ToolOutput> {
        // 1. Execute base tool functionality
        let base_result = self.base_tool.execute(params).await?;
        
        // 2. Add intelligence enhancements
        let suggestions = self.intelligence.suggest_next_actions(&base_result).await?;
        let context = self.intelligence.explain_result_context(&base_result).await?;
        
        // 3. Return enhanced output
        ToolOutput::Enhanced {
            base_result,
            suggestions,
            context,
            related_patterns: self.intelligence.find_related_patterns(&base_result).await?,
        }
    }
}
```

## Data Storage Strategy

### Hybrid Approach

1. **redb Database**: Fast, embedded, zero-config
   - Structured data: preferences, action outcomes, patterns
   - Key-value storage optimized for agent workloads

2. **Leverage Semantic Search**: Don't duplicate vector storage
   - Use existing semantic search for code similarity
   - Build intelligence on top of semantic results

3. **JSON Cache**: Human-readable debugging
   - Export important data as JSON for inspection
   - Easy to understand what agent has learned

### Storage Location
```
~/.local/share/aircher/intelligence/
├── memory.redb              # Main intelligence database
├── cache/                   # JSON exports for debugging
│   ├── preferences.json
│   ├── patterns.json
│   └── successful_actions.json
└── logs/                    # Intelligence decision logs
```

## Performance Considerations

**Fast by Design**:
- Embedded database (no network calls)
- Simple pattern matching (no ML inference)
- Cache frequently used data in memory
- Async/await for non-blocking operations

**Memory Efficient**:
- redb uses memory mapping
- JSON cache only for debugging (optional)
- Leverage existing semantic search vectors

**TUI Responsive**:
- Intelligence enhancement happens in background
- Fallback to basic mode if intelligence is slow
- Progressive enhancement of user experience

## Privacy & Security

**Local-First**:
- All learning data stays on user's machine
- No telemetry or data uploading
- User controls their own intelligence data

**Transparent**:
- JSON cache files show what agent learned
- Clear logging of intelligence decisions
- User can inspect and modify learned patterns

## Implementation Phases

### Phase 1: Foundation (Week 1-2)
- Add redb dependency
- Create basic ProjectMemory
- Implement simple IntentAnalyzer
- Basic PromptEnhancer with semantic search integration

### Phase 2: Integration (Week 3)
- Connect to existing AgentController
- Wrap existing tools with intelligence
- Add learning from successful interactions

### Phase 3: Polish (Week 4)
- JSON cache for debugging
- Performance optimization
- User controls for intelligence features

## Success Metrics

**Autonomous Intelligence**:
- Agent suggests relevant next actions
- Learns user preferences over time
- Provides better context for queries
- Improves tool effectiveness

**TUI Performance**:
- No degradation in startup time
- Response time under 100ms for intelligence features
- Memory usage increase < 50MB

**User Experience**:
- More relevant responses over time
- Reduced need for follow-up questions
- Better understanding of project context