# Zed-Style Edit Predictions for TUI Chat Autocomplete

## Overview

Implement intelligent autocomplete functionality in the TUI chat interface similar to Zed editor's edit predictions. This will provide contextual, AI-powered suggestions as users type their messages, significantly improving the user experience and message quality.

## Features

### Core Functionality
- **Real-time suggestions**: As-you-type predictions that appear inline or in a popup
- **Tab completion**: Accept suggestions with Tab key (similar to ChatGPT app)
- **Context-aware**: Suggestions based on:
  - Current project context (files, recent changes)
  - Conversation history
  - Common coding patterns and queries
  - User's typical message style

### Suggestion Types
1. **Code-related queries**: "How do I...", "Explain this...", "Fix the error in..."
2. **Project-specific**: References to files, functions, and classes in the current project
3. **Command completions**: "/help", "/quit", "/search", etc.
4. **Common patterns**: "Generate tests for...", "Refactor this...", "Document this..."

## Technical Implementation

### Architecture
```
TUI Input Field
├── InputPredictor (new component)
│   ├── LocalPatternMatcher (fast, instant suggestions)
│   ├── ProjectContextProvider (file/code aware suggestions)
│   └── AIPredictor (intelligent, context-aware suggestions)
└── UI Components
    ├── InlineSuggestion (ghosted text after cursor)
    ├── SuggestionPopup (dropdown list of options)
    └── KeyboardHandlers (Tab, Esc, arrows)
```

### Core Components

#### 1. InputPredictor
```rust
pub struct InputPredictor {
    local_patterns: PatternMatcher,
    project_context: ProjectContextProvider,
    ai_predictor: Option<AIPredictor>,
    current_suggestions: Vec<Suggestion>,
}

pub struct Suggestion {
    pub text: String,
    pub completion: String,
    pub score: f32,
    pub source: SuggestionSource,
}

pub enum SuggestionSource {
    LocalPattern,
    ProjectContext,
    AIGenerated,
    CommandCompletion,
}
```

#### 2. Local Pattern Matching (Instant)
- Pre-built patterns for common queries
- Trigram/n-gram matching for message prefixes
- Command completions ("/help", "/search", etc.)
- File path completions

#### 3. Project Context Provider
- Recent file modifications
- Function/class names in current project
- Error messages from recent builds
- Git status and branch information

#### 4. AI Predictor (Optional/Premium)
- Uses a lightweight model (like Zed's edit model)
- Sends partial message + project context to prediction service
- Falls back gracefully when unavailable

### UI/UX Design

#### Inline Suggestions (Primary)
```
User types: "How do I fix the er"
Display:    "How do I fix the er|ror in authentication.rs?" [ghosted]
```

#### Popup Suggestions (Secondary)
```
User types: "explain"
┌─ Suggestions ──────────────────┐
│ → explain this function        │
│   explain the error in         │
│   explain authentication.rs    │
│   explain the test failure     │
└────────────────────────────────┘
```

#### Keyboard Interactions
- **Tab**: Accept current inline suggestion
- **Ctrl+Space**: Show/hide suggestion popup
- **Up/Down**: Navigate popup suggestions
- **Esc**: Dismiss suggestions
- **Right arrow**: Accept word-by-word

### Integration Points

#### With Existing Systems
1. **Intelligence Engine**: Use project analysis for context-aware suggestions
2. **Semantic Search**: Suggest searches based on recent code changes
3. **File Monitor**: Incorporate recent file changes into suggestions
4. **Session History**: Learn from user's previous successful queries

#### Configuration Options
```toml
[ui.autocomplete]
enabled = true
inline_suggestions = true
popup_suggestions = true
ai_predictions = true  # Requires model/API
max_suggestions = 5
debounce_ms = 150
```

## Implementation Phases

### Phase 1: Foundation (Week 1)
- Basic InputPredictor structure
- Local pattern matching for common queries
- Simple inline suggestions
- Tab completion for basic patterns

### Phase 2: Project Integration (Week 2) 
- Project context provider
- File/function name completions
- Recent changes integration
- Command completions ("/", "aircher")

### Phase 3: AI Enhancement (Week 3)
- Integration with lightweight prediction model
- Context-aware intelligent suggestions
- Fallback mechanisms
- Performance optimization

### Phase 4: Polish (Week 4)
- Advanced keyboard navigation
- Visual improvements
- User preference settings
- Analytics and learning

## Model Options for AI Predictions

### Option 1: Local Model (Recommended)
- **Model**: CodeT5+ or similar lightweight model
- **Size**: ~200MB (reasonable for desktop app)
- **Latency**: <100ms
- **Privacy**: Fully local, no data sent externally

### Option 2: API-based (Premium/Optional)
- **Service**: OpenAI GPT-3.5-turbo or Claude Haiku
- **Prompt**: Optimized for completion prediction
- **Fallback**: Always available local patterns
- **Cost**: Low (completion requests are small)

### Option 3: Hybrid Approach
- **Local**: Instant patterns and project context
- **API**: Enhanced suggestions when available
- **Caching**: Store successful predictions locally

## Success Metrics

### User Experience
- Reduced typing time for common queries
- Higher message quality (fewer typos, better phrasing)
- Increased feature discovery (users find new commands)
- Positive user feedback on autocomplete usefulness

### Technical Performance
- Suggestion latency <50ms for local patterns
- <200ms for AI-enhanced suggestions
- Memory usage <100MB for suggestion cache
- No impact on TUI responsiveness

## Example User Flows

### Flow 1: File-based Query
```
User types: "exp"
Suggestion: "explain authentication.rs" (recent file)
User presses Tab → completes to full suggestion
User continues: "explain authentication.rs and why it's failing"
```

### Flow 2: Error Investigation
```
User types: "why is"
Suggestions based on recent errors:
- "why is my build failing?"
- "why is authentication not working?"
- "why is the test in user_service.rs failing?"
```

### Flow 3: Code Generation
```
User types: "gen"
Suggestions:
- "generate tests for the UserService class"
- "generate documentation for"
- "generate a README for this project"
```

## Dependencies

### New Dependencies
```toml
# For local AI model (if implemented)
candle = "0.3"  # Or tch for PyTorch models
tokenizers = "0.14"

# For improved text processing
fuzzy-matcher = "0.3"
regex = "1.0"

# For debouncing user input
tokio-util = "0.7"
```

### Integration with Existing Code
- Extend `TuiManager` with autocomplete functionality
- Modify input handling in TUI chat interface
- Integrate with `IntelligenceEngine` for project context
- Use existing configuration system for settings

## Conclusion

This feature will significantly enhance the user experience by making the TUI chat interface more intelligent and responsive. By implementing it in phases, we can deliver value incrementally while maintaining system stability and performance.

The hybrid approach (local patterns + optional AI) ensures the feature works reliably for all users while providing enhanced capabilities for those who opt in to AI-powered suggestions.