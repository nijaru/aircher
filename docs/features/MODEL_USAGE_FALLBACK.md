# Model Usage Fallback Feature

## Overview
Implement a "default" model option that automatically switches between high-tier and lower-tier models based on usage limits, similar to Claude Code's Pro/Max subscription feature. Enhanced with task-aware model selection for optimal cost/performance balance.

## Current Behavior (Claude Code)
- **Default Model**: Uses Opus 4 for up to 50% of usage limit
- **Automatic Fallback**: Switches to Sonnet 4 after 50% usage
- **No Auto-Reset**: Doesn't detect subscription window reset

## Proposed Enhancement for Aircher

### Task-Aware Three-Tier Model Selection
```rust
pub enum ModelSelection {
    Specific(String),
    Default {
        planning: String,     // e.g., "claude-opus-4" for complex reasoning
        coding: String,       // e.g., "claude-sonnet-4" for implementation
        summary: String,      // e.g., "claude-haiku-3" for simple tasks
        threshold: f32,       // e.g., 0.5 (50%) for usage-based fallback
    }
}

pub enum TaskType {
    Planning,    // Architecture, design, complex problem solving
    Coding,      // Implementation, debugging, refactoring
    Summary,     // Documentation, simple explanations, formatting
}
```

### Usage Tracking
```toml
[models.usage]
window_start = "2024-01-15T00:00:00Z"
window_duration = "30d"
opus_4_used = 15000     # tokens
opus_4_limit = 30000    # tokens per window
auto_reset = true       # Aircher enhancement
```

### Configuration Example
```toml
[providers.anthropic]
models = [
    { name = "default", type = "smart_fallback" },
    { name = "claude-opus-4", aliases = ["opus", "o4"] },
    { name = "claude-sonnet-4", aliases = ["sonnet", "s4"] },
    { name = "claude-haiku-3", aliases = ["haiku", "h3"] }
]

[models.smart_fallback]
# Task-specific model assignments
planning = "claude-opus-4"      # Best reasoning for architecture/design
coding = "claude-sonnet-4"      # Balanced for implementation
summary = "claude-haiku-3"      # Efficient for simple tasks

# Usage-based fallback (when approaching limits)
threshold = 0.5                 # Switch at 50% usage
fallback_chain = [              # Graceful degradation
    "claude-opus-4",
    "claude-sonnet-4", 
    "claude-haiku-3"
]
track_window = true             # Track subscription windows

# Optional: Task detection patterns
[models.task_detection]
planning = ["design", "architecture", "plan", "strategy"]
coding = ["implement", "fix", "debug", "refactor", "code"]
summary = ["summarize", "explain", "document", "format"]
```

### Status Bar Display
```
# Under 50% usage
Model: Default (Opus 4) - 35% used

# Over 50% usage  
Model: Default (Sonnet 4) - 65% used

# Window reset detected
Model: Default (Opus 4) - 0% used [Reset]
```

### Implementation Steps
1. Track model-specific usage per provider
2. Store usage window start/end dates
3. Calculate usage percentage in real-time
4. Automatic model switching at threshold
5. Detect subscription window reset
6. Optional notification on model switch

### Enhanced Features (Beyond Claude Code)
1. **Auto-Reset Detection**: Check if usage window has reset
2. **Multiple Tiers**: Support 3+ model fallback chain
3. **Custom Thresholds**: User-configurable percentages
4. **Usage Predictions**: Warn before hitting limits
5. **Cost Optimization**: Suggest optimal model based on task

### API Integration
```rust
impl ProviderManager {
    pub async fn get_effective_model(&self, requested: &str) -> String {
        if requested == "default" {
            let usage = self.get_usage_percentage().await?;
            if self.should_reset_window() {
                self.reset_usage_tracking().await?;
            }
            
            match usage {
                0.0..=0.5 => "claude-opus-4",
                _ => "claude-sonnet-4"
            }
        } else {
            requested
        }
    }
}
```

### Benefits
- Maximizes high-tier model usage
- Automatic fallback prevents hitting limits
- No manual model switching needed
- Cost-effective for power users
- Better than Claude Code with auto-reset

### Future Enhancements
- Per-project model preferences
- Time-based switching (peak hours)
- Task-aware model selection
- Usage analytics and reporting