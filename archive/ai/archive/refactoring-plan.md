# Aircher Intelligence Refactoring Plan

*Structured approach to implementing autonomous agent intelligence*

## Overview

**Goal**: Transform Aircher from a tool-executing agent into an autonomously intelligent coding companion.

**Strategy**: Incremental enhancement of existing systems, not replacement.

**Timeline**: 3-4 weeks for core intelligence, ongoing refinement.

## Refactoring Phases

### Phase 2.1: Intelligence Foundation (Week 1-2)

**Objective**: Add core intelligence components without breaking existing functionality.

**Dependencies**:
```toml
[dependencies]
redb = "2.1"              # Embedded database
serde_cbor = "0.11"       # Compact serialization
chrono = { version = "0.4", features = ["serde"] }
```

**File Structure**:
```
src/intelligence/
├── mod.rs                # Public API and integration
├── memory/
│   ├── mod.rs           # Memory system interface
│   ├── storage.rs       # redb database operations
│   └── cache.rs         # JSON export/debugging
├── analysis/
│   ├── mod.rs           # Analysis components
│   ├── intent.rs        # Intent classification
│   └── context.rs       # Context extraction
└── enhancement/
    ├── mod.rs           # Enhancement systems
    ├── prompts.rs       # Dynamic prompt composition
    └── suggestions.rs   # Next action suggestions
```

**Key Implementation Tasks**:

1. **Create Intelligence Module** (`src/intelligence/mod.rs`):
```rust
pub struct AgentIntelligence {
    memory: ProjectMemory,
    intent_analyzer: IntentAnalyzer,
    prompt_enhancer: PromptEnhancer,
    semantic_search: Arc<SemanticSearchEngine>,
}

impl AgentIntelligence {
    pub async fn new(
        project_root: PathBuf,
        semantic_search: Arc<SemanticSearchEngine>
    ) -> Result<Self>;

    pub async fn process_intelligently(
        &self,
        message: &str
    ) -> Result<IntelligentResponse>;
}
```

2. **Implement Project Memory** (`src/intelligence/memory/storage.rs`):
```rust
pub struct ProjectMemory {
    db: redb::Database,
    cache_dir: PathBuf,
}

// Simple learning schema
struct ActionRecord {
    context: String,
    action: String,
    outcome: Quality,
    timestamp: DateTime<Utc>,
    effectiveness: f32,
}
```

3. **Add Intent Analysis** (`src/intelligence/analysis/intent.rs`):
```rust
pub enum IntentType {
    Debug,      // Fix errors, troubleshoot issues
    Implement,  // Add features, write code
    Explain,    // Understand existing code
    Refactor,   // Improve code structure
    Search,     // Find specific code or patterns
    Learn,      // General learning questions
}

pub struct IntentAnalyzer {
    patterns: HashMap<IntentType, Vec<&'static str>>,
}
```

4. **Create Prompt Enhancement** (`src/intelligence/enhancement/prompts.rs`):
```rust
pub struct PromptEnhancer {
    base_prompts: HashMap<IntentType, String>,
    memory: Arc<ProjectMemory>,
}

impl PromptEnhancer {
    pub async fn enhance_prompt(
        &self,
        message: &str,
        intent: &IntentType,
        semantic_context: &[SearchResult]
    ) -> Result<String>;
}
```

### Phase 2.2: Agent Controller Integration (Week 2-3)

**Objective**: Connect intelligence to existing agent system.

**Key Changes**:

1. **Activate IntelligenceEngine** (`src/agent/controller.rs`):
```rust
impl AgentController {
    pub fn new(
        intelligence: IntelligenceEngine,
        auth_manager: Arc<AuthManager>,
        project_context: ProjectContext,
    ) -> Result<Self> {
        Ok(Self {
            tools: ToolRegistry::default(),
            intelligence,  // Remove underscore, make active
            // ... rest unchanged
        })
    }

    pub async fn process_message_intelligently(
        &mut self,
        message: &str,
        provider: &dyn LLMProvider,
        model: &str
    ) -> Result<(String, Vec<String>)> {
        // Use intelligence to enhance processing
        let enhanced_prompt = self.intelligence
            .enhance_prompt(message, &self.conversation.project_context)
            .await?;

        self.process_with_enhanced_context(enhanced_prompt, provider, model).await
    }
}
```

2. **Feature Flag Integration**:
```rust
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub intelligence_enabled: bool,
    pub learning_enabled: bool,
    pub memory_retention_days: u32,
}

impl AgentController {
    pub async fn process_message(&mut self, message: &str) -> Result<String> {
        if self.config.intelligence_enabled {
            self.process_message_intelligently(message).await
        } else {
            self.process_message_legacy(message).await
        }
    }
}
```

### Phase 2.3: Tool Intelligence Enhancement (Week 3)

**Objective**: Add project awareness and learning to existing tools.

**Implementation Strategy**: Wrap existing tools vs rewrite them.

1. **Create Intelligent Tool Wrapper**:
```rust
pub struct IntelligentToolWrapper<T: AgentTool> {
    base_tool: T,
    intelligence: Arc<AgentIntelligence>,
    tool_name: String,
}

impl<T: AgentTool> AgentTool for IntelligentToolWrapper<T> {
    async fn execute(&self, params: ToolParams) -> Result<ToolOutput> {
        // 1. Execute base functionality
        let base_result = self.base_tool.execute(params).await?;

        // 2. Add intelligence enhancements
        let context = self.intelligence.analyze_tool_context(&base_result).await?;
        let suggestions = self.intelligence.suggest_next_actions(&base_result).await?;

        // 3. Learn from tool usage
        self.intelligence.record_tool_usage(&self.tool_name, &base_result).await?;

        // 4. Return enhanced output
        Ok(ToolOutput::Enhanced {
            base_result,
            context,
            suggestions,
        })
    }
}
```

2. **Enhance File Operations**:
```rust
impl IntelligentFileTool {
    async fn read_file_intelligently(&self, path: &Path) -> Result<ToolOutput> {
        let content = self.read_file_basic(path).await?;

        // Add intelligence
        let related_files = self.semantic_search.find_related_files(path, 3).await?;
        let patterns = self.intelligence.memory.get_file_patterns(path).await?;
        let suggestions = self.generate_file_suggestions(&content, &patterns).await?;

        Ok(ToolOutput::IntelligentFileRead {
            content,
            related_files,
            common_patterns: patterns,
            suggested_actions: suggestions,
        })
    }
}
```

### Phase 2.4: TUI Integration (Week 3-4)

**Objective**: Expose intelligence features in the user interface.

**UI Enhancements**:

1. **Intelligence Status Display**:
```rust
// Status bar additions
pub struct IntelligenceStatus {
    pub learning_enabled: bool,
    pub patterns_learned: usize,
    pub context_quality: f32,  // 0.0-1.0
}
```

2. **Enhanced Tool Output Display**:
```rust
// Tool result with intelligence
pub enum ToolResultUI {
    Enhanced {
        base_content: String,
        suggestions: Vec<NextAction>,
        context_notes: Vec<String>,
        collapsible: bool,
    }
}
```

3. **Keyboard Shortcuts for Intelligence**:
```rust
// New shortcuts
// Ctrl+I - Toggle intelligence features
// Ctrl+? - Show intelligence insights
// Ctrl+H - Show learning history
```

## Implementation Guidelines

### Error Handling Strategy

```rust
#[derive(Debug, thiserror::Error)]
pub enum IntelligenceError {
    #[error("Memory storage error: {0}")]
    Storage(#[from] redb::Error),

    #[error("Context analysis failed: {0}")]
    Analysis(String),

    #[error("Intelligence features disabled")]
    Disabled,
}

// Graceful degradation
impl AgentController {
    async fn process_with_fallback(&mut self, message: &str) -> Result<String> {
        match self.process_message_intelligently(message).await {
            Ok(response) => Ok(response),
            Err(IntelligenceError::Disabled) => self.process_message_legacy(message).await,
            Err(e) => {
                warn!("Intelligence error: {}, falling back", e);
                self.process_message_legacy(message).await
            }
        }
    }
}
```

### Performance Requirements

**Response Time Targets**:
- Intent analysis: < 10ms
- Memory lookup: < 50ms
- Prompt enhancement: < 100ms
- Total intelligence overhead: < 200ms

**Memory Usage**:
- Intelligence system: < 50MB additional RAM
- Database size: < 100MB per project
- Cache files: < 10MB per project

### Testing Strategy

**Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_intent_analysis() {
        let analyzer = IntentAnalyzer::new();
        assert_eq!(
            analyzer.analyze("fix this error"),
            IntentType::Debug
        );
    }

    #[tokio::test]
    async fn test_memory_persistence() {
        let memory = ProjectMemory::new_temp().await?;
        memory.remember_success("test", "action", Quality::High).await?;

        let similar = memory.recall_similar("test").await?;
        assert!(!similar.is_empty());
    }
}
```

**Integration Tests**:
```rust
#[tokio::test]
async fn test_intelligent_agent_flow() {
    let mut agent = create_test_agent_with_intelligence().await?;

    let response = agent.process_message("help me debug this rust error").await?;

    // Should show enhanced understanding
    assert!(response.contains("Based on similar issues in this project"));
}
```

## Configuration Management

### Intelligence Settings

```toml
# ~/.config/aircher/config.toml
[intelligence]
enabled = true
learning = true
memory_retention_days = 90
max_patterns_per_file = 100

[intelligence.privacy]
store_conversations = false  # Don't store full conversations
anonymize_paths = true      # Hash file paths in learning data
local_only = true          # Never upload intelligence data
```

### Feature Flags

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct IntelligenceConfig {
    pub enabled: bool,
    pub learning: bool,
    pub suggestions: bool,
    pub context_enhancement: bool,
    pub tool_intelligence: bool,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            learning: true,
            suggestions: true,
            context_enhancement: true,
            tool_intelligence: false,  // Opt-in for now
        }
    }
}
```

## Migration Strategy

### Existing User Experience

**Zero Breaking Changes**:
- All existing functionality works unchanged
- Intelligence features are additive enhancements
- Configuration is optional with sensible defaults
- Fallback behavior maintains current experience

**Gradual Enablement**:
1. Week 1: Core intelligence (background learning)
2. Week 2: Enhanced prompts (better responses)
3. Week 3: Tool intelligence (smarter suggestions)
4. Week 4: Full UI integration (visual indicators)

### Data Migration

**No Migration Required**:
- New intelligence data stored separately
- Existing configuration unchanged
- Existing semantic search indices preserved

## Success Metrics

### Intelligence Effectiveness

**Quantitative**:
- Response relevance improved by 30%
- Tool suggestion accuracy > 80%
- User follow-up questions reduced by 20%
- Context understanding score > 0.8

**Qualitative**:
- Agent "remembers" previous solutions
- Suggests context-appropriate next actions
- Learns user preferences over time
- Provides project-specific insights

### Performance Impact

**System Performance**:
- TUI startup time unchanged (< 100ms)
- Response time increase < 200ms
- Memory usage increase < 50MB
- Storage growth < 100MB per project

**User Experience**:
- No degradation in basic functionality
- Enhanced features feel responsive
- Learning happens transparently
- Intelligence insights are helpful, not distracting

## Risk Mitigation

### Technical Risks

**Database Corruption**:
- Regular backup to JSON cache
- Database integrity checks on startup
- Graceful fallback to non-intelligent mode

**Performance Degradation**:
- Background intelligence processing
- Configurable timeout limits
- Performance monitoring and alerts

**Memory Leaks**:
- Regular cleanup of old patterns
- Memory usage monitoring
- Automatic cache size limits

### User Experience Risks

**Feature Confusion**:
- Clear documentation of intelligence features
- Progressive disclosure of advanced capabilities
- Option to disable intelligence features

**Privacy Concerns**:
- All data stored locally
- No telemetry or data uploading
- Transparent data usage policies
- User control over learning data

## Timeline Summary

**Week 1**: Core intelligence foundation
**Week 2**: Agent integration and testing
**Week 3**: Tool enhancement and UI updates
**Week 4**: Polish, optimization, documentation

**Total Effort**: ~3-4 weeks for MVP intelligence system
**Follow-up**: Ongoing refinement based on usage patterns
