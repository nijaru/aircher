# Compaction Improvements

## Issue: Manual /compact vs Auto-compaction Quality

**Problem Discovered**: Claude Code achieves better compaction results when the agent performs automatic compaction at max context length compared to when users trigger it manually with `/compact`.

**Root Cause**: Manual compaction uses a generic prompt that doesn't consider:
- Current conversation context
- What the user is working on
- What information would be most valuable to preserve
- The specific task or project state

## Current Implementation Analysis

### Manual /compact Command (Basic Prompt)
```
Please create a concise summary of this conversation that captures:
1. The main topics discussed
2. Key decisions or conclusions reached
3. Important context that should be preserved
4. Current project state or progress

Keep the summary detailed enough to maintain continuity but concise enough to save context.
```

### Auto-compaction (Same Basic Prompt)
Currently uses the same generic prompt with empty custom instructions.

## Proposed Enhancement: Intelligent Compaction Prompts

### Smart Prompt Generation for /compact
When user triggers `/compact`, automatically analyze:
1. **Recent Context**: What was discussed in last 3-5 messages
2. **Active Task**: What is the user currently working on
3. **Project State**: Files mentioned, tools used, commands run
4. **Critical Information**: Error states, solutions found, decisions made

### Enhanced Prompt Template
```
Please create a targeted summary of this conversation optimized for continuing work on: {CURRENT_TASK}

Focus especially on preserving:
- {TASK_SPECIFIC_CONTEXT}
- Recent progress on: {RECENT_WORK}
- Important findings: {KEY_DISCOVERIES}
- Current project state: {PROJECT_STATE}
- Any unresolved issues or next steps

Maintain enough detail for seamless continuation of the current work while compacting efficiently.

[Special focus areas: {DOMAIN_SPECIFIC_PRIORITIES}]
```

### Implementation Strategy

#### Phase 1: Context Analysis
1. **Task Detection**: Analyze recent messages for:
   - File paths being worked on
   - Commands being run
   - Error messages and solutions
   - Feature requests or bug fixes

2. **Domain Detection**: Identify:
   - Programming language (from file extensions, commands)
   - Framework (from package.json, Cargo.toml, etc.)
   - Tool usage patterns (git, test, build commands)

3. **Progress Analysis**: Track:
   - What was attempted
   - What succeeded/failed
   - Current state of work

#### Phase 2: Smart Prompt Assembly
```rust
struct CompactionContext {
    current_task: String,
    recent_files: Vec<String>,
    active_tools: Vec<String>,
    key_decisions: Vec<String>,
    unresolved_issues: Vec<String>,
    project_type: Option<String>, // rust, node, python, etc.
}

impl CompactionContext {
    fn generate_smart_prompt(&self, conversation: &str) -> String {
        let mut prompt = String::new();

        // Task-specific focus
        if !self.current_task.is_empty() {
            prompt.push_str(&format!(
                "Focus on preserving context for: {}\n\n",
                self.current_task
            ));
        }

        // Domain-specific priorities
        match self.project_type.as_deref() {
            Some("rust") => prompt.push_str("Prioritize: compilation errors, dependency issues, test results, performance insights\n"),
            Some("node") => prompt.push_str("Prioritize: npm commands, test results, build issues, package updates\n"),
            Some("python") => prompt.push_str("Prioritize: import errors, virtual env setup, test results, dependencies\n"),
            _ => {}
        }

        // Add base compaction instructions...
    }
}
```

#### Phase 3: Integration Points
1. **Manual /compact**: Generate smart prompt based on conversation analysis
2. **Auto-compaction**: Same smart analysis when hitting context limits
3. **Custom /compact args**: Allow user to specify focus areas
   - `/compact focus:authentication`
   - `/compact preserve:error-solutions`
   - `/compact for:next-session`

## Expected Benefits

1. **Better Context Preservation**: Keep information actually needed for continuation
2. **Task-Aware Summaries**: Focus on current work rather than generic overview
3. **Reduced Re-explanation**: User doesn't need to re-explain context after compaction
4. **Improved Continuity**: Agent maintains better understanding of project state

## Implementation Priority

**High Priority**: This directly impacts user experience and agent effectiveness. Poor compaction forces users to re-explain context, breaking flow.

**Complexity**: Medium - requires conversation analysis and smart prompt generation

**Impact**: High - significantly improves agent utility and user satisfaction

## Usage Examples

### Before (Generic)
```
User: /compact
Agent: [Generic summary that loses important debugging context]
User: [Has to re-explain the authentication bug they were fixing]
```

### After (Smart)
```
User: /compact
Agent: [Analyzes: user was fixing auth bug in login.rs, tried 3 solutions, error was JWT parsing]
Agent: [Generates targeted summary preserving: the specific error, solutions attempted, current state]
User: [Can continue immediately without re-explanation]
```

### Custom Focus
```
User: /compact focus:database-migration
Agent: [Summary optimized for database work, preserving schema changes, migration errors, etc.]
```

## Technical Notes

- Should work with any LLM provider
- Needs conversation message analysis (lightweight)
- Could leverage existing semantic search for file/project understanding
- Should be configurable (users can disable smart analysis if preferred)

---

**Status**: Documented for implementation
**Priority**: High (UX impact)
**Effort**: Medium (analysis + prompt generation)
