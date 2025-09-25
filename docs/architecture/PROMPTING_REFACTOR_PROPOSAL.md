# Prompting System Refactor Proposal

## Current Problem

Our agent has two conflicting systems:
1. **Simple prompting system**: Works well, task-based heuristics (~200 lines)
2. **MultiTurnReasoningEngine**: Over-engineered orchestration (1685 lines)

Tasks like "debug error", "fix bug", "refactor code" unnecessarily trigger the complex system.

## Proposed Solution

**Replace** MultiTurnReasoningEngine with **enhanced prompting** based on research patterns.

### Current Flow (Over-engineered)
```rust
if needs_multi_turn_reasoning(message) {
    // Trigger 1685-line complex orchestration system
    let plan = reasoning_engine.create_reasoning_plan(message);
    for phase in plan.phases {
        for action in phase.actions {
            execute_complex_orchestration(action);
        }
    }
}
```

### Proposed Flow (Research-based)
```rust
// Enhanced prompting system handles all cases
let enhanced_prompt = prompt_system.create_enhanced_prompt(message);
let response = llm.chat(enhanced_prompt).await?;

// Agent focuses on execution and validation
if let Some(tool_calls) = response.tool_calls {
    let results = execute_tools(tool_calls).await?;
    validate_and_persist(results).await?;
}
```

## Implementation Plan

### Phase 1: Integrate Enhanced Prompting
1. Add `enhanced_prompting.rs` to agent module
2. Update `agent/core.rs` to use enhanced prompts for "complex" tasks
3. Keep existing simple prompting for basic tasks

### Phase 2: A/B Test Performance
1. Add feature flag to compare systems
2. Test both approaches on same tasks
3. Measure: response quality, execution time, success rate

### Phase 3: Remove Complex System (if validated)
1. Remove `needs_multi_turn_reasoning()` logic
2. Delete `MultiTurnReasoningEngine` (1685 lines)
3. Simplify agent architecture

## Expected Benefits

### Performance Improvements
- **Faster response time**: No complex plan generation
- **Better reasoning**: Leverages model's internal optimization
- **Reduced complexity**: Single execution path

### Code Simplification
- **-1685 lines**: Remove MultiTurnReasoningEngine
- **-500 lines**: Remove strategy orchestration
- **+300 lines**: Add enhanced prompting system
- **Net: -1885 lines of complex code**

### Research Alignment
- **ReAct patterns**: Think → Act → Observe prompting
- **Reflexion patterns**: Systematic debugging with reflection
- **Tree-of-Thoughts**: Multi-path analysis for complex problems

## Integration Code Changes

### Update agent/mod.rs
```rust
pub mod enhanced_prompting;  // Add new module
// pub mod multi_turn_reasoning;  // Remove eventually
```

### Update agent/core.rs
```rust
use crate::agent::enhanced_prompting::EnhancedPromptingSystem;

struct UnifiedAgent {
    // ... existing fields
    enhanced_prompting: EnhancedPromptingSystem,  // Add this
    // multi_turn_reasoning: Arc<Mutex<MultiTurnReasoningEngine>>,  // Remove eventually
}

impl UnifiedAgent {
    // Replace complex logic:
    pub async fn process_message(&self, message: &str) -> Result<String> {
        // OLD: Check if needs complex reasoning
        // if self.needs_multi_turn_reasoning(message).await {
        //     return self.process_with_multi_turn_reasoning(message).await;
        // }

        // NEW: Always use enhanced prompting
        let enhanced_prompt = self.enhanced_prompting.create_enhanced_prompt(message);

        // Rest of the logic remains the same (tool execution, validation)
        // ...
    }
}
```

## Risk Assessment

### Low Risk
- Enhanced prompting is additive (doesn't break existing functionality)
- Can be feature-flagged for safe rollout
- Based on proven research patterns

### Medium Risk
- Need to validate that enhanced prompts perform as well as complex orchestration
- Learning/memory systems need to be preserved

### Mitigation
- Implement A/B testing to validate performance
- Keep complex system during transition period
- Gradual rollout with monitoring

## Success Metrics

### Quality Metrics
- Task completion rate (should be same or better)
- User satisfaction with responses
- Accuracy of problem-solving

### Performance Metrics
- Response latency (should improve significantly)
- Memory usage (should decrease)
- Code maintainability (fewer bugs, easier debugging)

### Research Validation
- Compare against ReAct, Reflexion benchmarks
- Measure improvement in systematic reasoning
- Validate learning and memory retention

## Timeline

- **Week 1**: Implement enhanced prompting system
- **Week 2**: Integrate with existing agent, feature flag
- **Week 3**: A/B test and performance validation
- **Week 4**: Remove complex system if validated

## Next Steps

1. **Implement enhanced prompting system** (already started)
2. **Add integration code** in agent/core.rs
3. **Create A/B test framework** for validation
4. **Test with real tasks** that currently trigger complex reasoning

This refactor aligns with our architectural insight: **models are reasoning engines, agents are execution engines**.