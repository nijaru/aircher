# Auto-Compaction Design

## Overview

This document outlines the design for automatic conversation compaction in Aircher to prevent context window overflow and optimize token usage.

## Current State

- Manual compaction exists via `/compact` command
- Compaction uses LLM to summarize conversation history
- Keeps last 3 messages for immediate context
- Updates session tokens after compaction
- No automatic triggers - users must manually compact

## Design Goals

1. **Prevent Context Overflow**: Auto-compact before hitting context limits
2. **Preserve Important Context**: Smart selection of what to keep vs summarize
3. **Minimize Disruption**: Compact at natural conversation boundaries
4. **User Control**: Configurable thresholds and behaviors
5. **Cost Efficiency**: Balance between compaction frequency and API costs

## Proposed Architecture

### 1. Context Monitoring

```rust
struct ContextMonitor {
    current_usage: u32,
    context_window: u32,
    warning_threshold: f32,  // e.g., 0.75 (75%)
    critical_threshold: f32, // e.g., 0.90 (90%)
    last_check: Instant,
}

impl ContextMonitor {
    fn usage_percentage(&self) -> f32 {
        self.current_usage as f32 / self.context_window as f32
    }

    fn should_compact(&self) -> CompactionTrigger {
        let usage = self.usage_percentage();
        if usage >= self.critical_threshold {
            CompactionTrigger::Critical
        } else if usage >= self.warning_threshold {
            CompactionTrigger::Warning
        } else {
            CompactionTrigger::None
        }
    }
}
```

### 2. Compaction Triggers

```rust
enum CompactionTrigger {
    None,
    Warning,     // Show warning, let user decide
    Critical,    // Auto-compact to prevent overflow
    UserForced,  // User initiated via /compact
    Periodic,    // Time-based compaction
}
```

### 3. Compaction Strategy

```rust
struct CompactionStrategy {
    // What to keep in full
    keep_recent_messages: usize,      // e.g., 5
    keep_system_messages: bool,       // Keep important system context
    keep_tool_results: bool,          // Keep code/file operations
    
    // How to summarize
    summarization_depth: SummaryDepth,
    preserve_code_blocks: bool,
    preserve_file_paths: bool,
    
    // When to compact
    auto_compact_enabled: bool,
    warning_threshold: f32,           // 75%
    critical_threshold: f32,          // 90%
    min_messages_before_compact: u32, // Don't compact tiny conversations
}

enum SummaryDepth {
    Brief,      // Key points only
    Standard,   // Balanced summary
    Detailed,   // Comprehensive summary
}
```

### 4. Implementation Plan

#### Phase 1: Context Monitoring
- Add real-time token counting for all messages
- Track context usage percentage
- Display warnings in status bar

#### Phase 2: Smart Compaction
- Implement message importance scoring
- Preserve critical information (code, files, decisions)
- Optimize summary prompts for different scenarios

#### Phase 3: Auto-Compaction
- Add configuration options
- Implement trigger conditions
- Add pre-compaction warnings
- Handle compaction during active conversations

## Configuration

Add to `config.toml`:

```toml
[compaction]
# Enable automatic compaction
auto_enabled = true

# Thresholds (as percentage of context window)
warning_threshold = 0.75
critical_threshold = 0.90

# Minimum messages before allowing compaction
min_messages = 10

# What to preserve
keep_recent_messages = 5
keep_system_messages = true
keep_tool_results = true

# Summarization settings
summary_depth = "standard"  # brief, standard, detailed
preserve_code_blocks = true
preserve_file_paths = true

# User notifications
show_warnings = true
require_confirmation = true  # For non-critical compactions
```

## User Experience

### Warning State (75-90% usage)
```
⚠️ Context usage high (78%) - Consider compacting conversation
```

### Critical State (>90% usage)
```
🚨 Context nearly full (92%) - Auto-compaction recommended
Would you like to compact now? [Y/n]
```

### Auto-Compaction Flow
1. Monitor context usage after each message
2. At warning threshold: Show notification
3. At critical threshold: 
   - If `require_confirmation = true`: Prompt user
   - If `require_confirmation = false`: Auto-compact with notification
4. Perform compaction preserving important context
5. Show summary of what was compacted

## Smart Preservation

### Message Importance Scoring
```rust
fn calculate_message_importance(msg: &Message) -> f32 {
    let mut score = 0.0;
    
    // Recency bonus
    score += recency_score(msg.timestamp);
    
    // Content type bonus
    if contains_code_blocks(&msg.content) { score += 0.3; }
    if contains_file_paths(&msg.content) { score += 0.2; }
    if contains_decisions(&msg.content) { score += 0.2; }
    
    // Role bonus
    match msg.role {
        MessageRole::Tool => score += 0.4,
        MessageRole::System => score += 0.3,
        _ => {}
    }
    
    // Length penalty (very long messages might be summarizable)
    if msg.content.len() > 2000 { score -= 0.1; }
    
    score
}
```

### Selective Summarization
- Keep high-importance messages in full
- Summarize low-importance messages
- Group related messages for better summaries

## API Optimization

### Efficient Summarization Prompts
```rust
fn build_compaction_prompt(messages: &[Message], strategy: &CompactionStrategy) -> String {
    let mut prompt = String::new();
    
    // Different prompts for different depths
    match strategy.summarization_depth {
        SummaryDepth::Brief => {
            prompt.push_str("Create a brief summary focusing only on key decisions and outcomes.");
        }
        SummaryDepth::Standard => {
            prompt.push_str("Summarize the conversation preserving important context and decisions.");
        }
        SummaryDepth::Detailed => {
            prompt.push_str("Create a comprehensive summary preserving all significant details.");
        }
    }
    
    // Special instructions for preservation
    if strategy.preserve_code_blocks {
        prompt.push_str("\nPreserve all code blocks and technical details exactly.");
    }
    
    prompt
}
```

## Testing Strategy

1. **Unit Tests**: Context monitoring calculations
2. **Integration Tests**: Compaction triggers and flows
3. **Manual Testing**: Different conversation types and edge cases
4. **Performance Testing**: Token counting accuracy

## Future Enhancements

1. **Semantic Chunking**: Group related messages before summarizing
2. **Incremental Compaction**: Compact older parts while keeping recent context
3. **Custom Compaction Rules**: User-defined preservation rules
4. **Compaction History**: Track what was summarized for reference
5. **Multi-Model Support**: Use cheaper models for summarization

## Migration Path

1. Add monitoring without auto-compaction
2. Collect usage data and refine thresholds
3. Enable auto-compaction with confirmation
4. Gradually reduce confirmation requirements based on success