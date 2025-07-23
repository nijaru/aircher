# Model Usage Fallback Feature

## Overview
Implement a "default" model option that automatically switches between high-tier and lower-tier models based on usage limits, similar to Claude Code's Pro/Max subscription feature.

## Current Behavior (Claude Code)
- **Default Model**: Uses Opus 4 for up to 50% of usage limit
- **Automatic Fallback**: Switches to Sonnet 4 after 50% usage
- **No Auto-Reset**: Doesn't detect subscription window reset

## Proposed Enhancement for Aircher

### Basic Implementation
```rust
pub enum ModelSelection {
    Specific(String),
    Default {
        primary: String,      // e.g., "claude-opus-4"
        fallback: String,     // e.g., "claude-sonnet-4"
        threshold: f32,       // e.g., 0.5 (50%)
    }
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
    { name = "claude-opus-4", aliases = ["opus"] },
    { name = "claude-sonnet-4", aliases = ["sonnet"] }
]

[models.smart_fallback]
primary = "claude-opus-4"
fallback = "claude-sonnet-4" 
threshold = 0.5  # Switch at 50% usage
track_window = true  # Track subscription windows
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