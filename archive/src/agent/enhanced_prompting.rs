/// Enhanced prompting system based on research findings
///
/// Incorporates ReAct, Reflexion, and Tree-of-Thoughts patterns directly into prompts
/// instead of complex external orchestration.

use std::collections::HashMap;
use crate::intelligence::working_memory::ContextWindowStats;

/// Enhanced prompting system that leverages model's internal reasoning
pub struct EnhancedPromptingSystem {
    memory: HashMap<String, Vec<String>>, // Store past learnings
}

impl EnhancedPromptingSystem {
    pub fn new() -> Self {
        Self {
            memory: HashMap::new(),
        }
    }

    /// Create enhanced system prompt based on task type and research patterns
    pub fn create_enhanced_prompt(&self, user_message: &str, context_stats: Option<&ContextWindowStats>) -> String {
        let message_lower = user_message.to_lowercase();

        // Get base prompt based on task type
        let base_prompt = if self.is_debug_task(&message_lower) {
            self.create_reflexion_enhanced_prompt(user_message)
        } else if self.is_complex_analysis_task(&message_lower) {
            self.create_tree_of_thoughts_prompt(user_message)
        } else if self.is_multi_step_task(&message_lower) {
            self.create_react_enhanced_prompt(user_message)
        } else if self.is_exploration_task(&message_lower) {
            self.create_systematic_exploration_prompt(user_message)
        } else {
            self.create_standard_enhanced_prompt(user_message)
        };

        // Inject context stats
        self.inject_context_stats(&base_prompt, context_stats)
    }

    /// ReAct-enhanced prompt for multi-step tasks
    /// Based on "Synergizing Reasoning and Acting" (Google, 2022)
    fn create_react_enhanced_prompt(&self, user_message: &str) -> String {
        let past_learnings = self.get_relevant_learnings(user_message);
        let learnings_context = if !past_learnings.is_empty() {
            format!("\\n\\nPrevious learnings from similar tasks:\\n{}\\n",
                    past_learnings.join("\\n"))
        } else {
            String::new()
        };

        format!(
            "You are Aircher, an AI coding assistant. Use the ReAct approach for this task: {}{}

**Instructions:**
1. **Think** step-by-step about what you need to do
2. **Act** by using the most appropriate tool for the current step
3. **Observe** the results carefully and understand what you learned
4. **Repeat** until the task is complete

**Critical Guidelines:**
- Think through each step before acting
- Use only ONE tool at a time and wait for results
- After each tool result, reflect on what you discovered
- Adapt your approach based on actual results, not assumptions
- If something fails, think about why and try a different approach

Available tools: {{tools}}

Begin by thinking through your approach, then take the first action.",
            user_message, learnings_context
        )
    }

    /// Reflexion-enhanced prompt for debugging tasks
    /// Based on "Reflexion: Language Agents with Verbal Reinforcement Learning" (Shinn et al, 2023)
    fn create_reflexion_enhanced_prompt(&self, user_message: &str) -> String {
        let past_failures = self.get_past_failures_for_task(user_message);
        let failure_context = if !past_failures.is_empty() {
            format!("\\n\\nPast debugging attempts to learn from:\\n{}\\n",
                    past_failures.join("\\n"))
        } else {
            String::new()
        };

        format!(
            "You are Aircher, an AI debugging assistant. Use systematic reflection for: {}{}

**Reflexion Approach:**
1. **Reproduce**: First understand and reproduce the issue
2. **Analyze**: Identify the root cause systematically
3. **Reflect**: If your first approach doesn't work, reflect on why:
   - What assumptions were wrong?
   - What did you miss in your analysis?
   - What patterns from past successes apply here?
4. **Improve**: Generate a better approach based on reflection
5. **Validate**: Test your fix thoroughly

**Self-Reflection Questions:**
- Why might this error be occurring?
- What would I do differently if I were debugging this again?
- What patterns from successful debugging sessions apply here?
- Am I making assumptions instead of gathering facts?

Be systematic, reflect on failures, and iterate until you find the solution.",
            user_message, failure_context
        )
    }

    /// Tree-of-Thoughts enhanced prompt for complex analysis
    /// Based on "Tree of Thoughts: Deliberate Problem Solving" (Princeton, 2023)
    fn create_tree_of_thoughts_prompt(&self, user_message: &str) -> String {
        format!(
            "You are Aircher, an AI coding assistant. Use multi-path reasoning for: {}

**Tree-of-Thoughts Approach:**
1. **Generate Multiple Approaches**: Consider 2-3 different solution strategies
   - What are the different ways to approach this problem?
   - Which approach might be most effective and why?

2. **Evaluate Each Approach**:
   - What are the pros and cons of each approach?
   - What risks or challenges might each approach face?
   - Which approach has the highest chance of success?

3. **Select and Execute**: Choose the most promising approach
   - Explain why you selected this approach over alternatives
   - Execute step-by-step with tools
   - Monitor progress and backtrack if needed

4. **Backtrack if Needed**: If your chosen path hits obstacles:
   - Reflect on what you learned
   - Switch to your second-best approach
   - Apply learnings from the failed path

Begin by outlining 2-3 different approaches to solve this problem.",
            user_message
        )
    }

    /// Systematic exploration prompt for codebase analysis
    fn create_systematic_exploration_prompt(&self, user_message: &str) -> String {
        format!(
            "You are Aircher, an AI coding assistant. Use systematic exploration for: {}

**Systematic Exploration Strategy:**
1. **Start High-Level**: Get an overview of the relevant codebase areas
   - Use `list_files` to understand directory structure
   - Look for key files, tests, documentation

2. **Dive Deep Systematically**:
   - Read files in order of importance (main → tests → utils)
   - Build understanding incrementally
   - Take notes on key patterns, interfaces, conventions

3. **Trace Execution Paths**:
   - Follow code flows from entry points
   - Understand data transformations and dependencies
   - Map out the relationships between components

4. **Validate Understanding**:
   - Test your understanding by predicting behavior
   - Look for examples and test cases that confirm your model

Work systematically through the codebase to build complete understanding.",
            user_message
        )
    }

    /// Enhanced standard prompt with basic reasoning guidance
    fn create_standard_enhanced_prompt(&self, user_message: &str) -> String {
        format!(
            "You are Aircher, an AI coding assistant. Task: {}

**Approach:**
1. Think through this problem step-by-step
2. Use tools when you need to examine or modify existing code
3. Generate high-quality code that follows project patterns
4. Validate your solution works as expected

**Guidelines:**
- Be systematic and methodical in your approach
- Use tools to gather facts, not make assumptions
- Follow established patterns in the codebase
- Test your solutions when possible

Available tools: {{tools}}",
            user_message
        )
    }

    // Helper methods for task classification
    fn is_debug_task(&self, message_lower: &str) -> bool {
        message_lower.contains("fix") || message_lower.contains("debug") ||
        message_lower.contains("error") || message_lower.contains("failing") ||
        message_lower.contains("broken") || message_lower.contains("issue") ||
        message_lower.contains("problem") || message_lower.contains("not working")
    }

    fn is_complex_analysis_task(&self, message_lower: &str) -> bool {
        message_lower.contains("refactor") || message_lower.contains("optimize") ||
        message_lower.contains("improve") || message_lower.contains("architect") ||
        message_lower.contains("design") || message_lower.contains("restructure")
    }

    fn is_multi_step_task(&self, message_lower: &str) -> bool {
        message_lower.contains("then") || message_lower.contains("and then") ||
        message_lower.contains(", then") || message_lower.contains("after that") ||
        message_lower.contains("first") && message_lower.contains("second")
    }

    fn is_exploration_task(&self, message_lower: &str) -> bool {
        message_lower.contains("understand") || message_lower.contains("analyze") ||
        message_lower.contains("explore") || message_lower.contains("investigate") ||
        message_lower.contains("find") || message_lower.contains("locate") ||
        message_lower.contains("search for")
    }

    /// Add context awareness section to any prompt
    pub fn inject_context_stats(&self, base_prompt: &str, stats: Option<&ContextWindowStats>) -> String {
        let Some(stats) = stats else {
            return base_prompt.to_string();
        };

        let context_section = format!(
            "\n\n**Context Status**:\n\
            - Tokens: {}/{} used ({}%)\n\
            - Items: {} context items\n\
            - Capacity: {} tokens remaining\n\
            - Pruning operations: {}\n\
            \n\
            Use this information to:\n\
            - Decide whether to continue current approach\n\
            - Adapt verbosity based on remaining space\n\
            - Summarize completed work if approaching limit (>80%)\n\
            - Focus on essential information\n\
            \n\
            If approaching limit (>80%), consider:\n\
            1. Being more concise in responses\n\
            2. Summarizing completed tasks\n\
            3. Focusing on current task only",
            stats.total_tokens,
            stats.max_tokens,
            (stats.utilization * 100.0) as u32,
            stats.total_items,
            stats.max_tokens.saturating_sub(stats.total_tokens),
            stats.pruning_count
        );

        format!("{}{}", base_prompt, context_section)
    }

    // Memory management methods
    fn get_relevant_learnings(&self, _user_message: &str) -> Vec<String> {
        // TODO: Implement intelligent retrieval of relevant past learnings
        vec![]
    }

    fn get_past_failures_for_task(&self, _user_message: &str) -> Vec<String> {
        // TODO: Implement retrieval of past failures for reflection
        vec![]
    }

    pub fn record_success_pattern(&mut self, task: &str, learnings: Vec<String>) {
        // TODO: Store successful patterns for future use
        self.memory.insert(task.to_string(), learnings);
    }

    pub fn record_failure_pattern(&mut self, task: &str, failure_insights: Vec<String>) {
        // TODO: Store failure patterns for reflection in future similar tasks
        let key = format!("failures_{}", task);
        self.memory.insert(key, failure_insights);
    }
}

impl Default for EnhancedPromptingSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_stats_injection() {
        let system = EnhancedPromptingSystem::new();
        let base_prompt = "You are Aircher.";

        let stats = ContextWindowStats {
            total_items: 47,
            total_tokens: 97_234,
            max_tokens: 200_000,
            utilization: 0.4861,
            pruning_count: 0,
            sticky_items: 3,
        };

        let result = system.inject_context_stats(base_prompt, Some(&stats));

        assert!(result.contains("97234"));
        assert!(result.contains("200000"));
        assert!(result.contains("48%"));
        assert!(result.contains("102766 tokens remaining"));
        assert!(result.contains("47 context items"));
        assert!(result.contains("You are Aircher."));
    }

    #[test]
    fn test_context_stats_none() {
        let system = EnhancedPromptingSystem::new();
        let base_prompt = "You are Aircher.";

        let result = system.inject_context_stats(base_prompt, None);

        // Should return unchanged when no stats
        assert_eq!(result, base_prompt);
    }

    #[test]
    fn test_create_enhanced_prompt_with_stats() {
        let system = EnhancedPromptingSystem::new();
        let user_message = "Fix the authentication bug";

        let stats = ContextWindowStats {
            total_items: 20,
            total_tokens: 50_000,
            max_tokens: 200_000,
            utilization: 0.25,
            pruning_count: 0,
            sticky_items: 2,
        };

        let result = system.create_enhanced_prompt(user_message, Some(&stats));

        // Should contain both the Reflexion prompt (debug task) and context stats
        assert!(result.contains("Reflexion"));
        assert!(result.contains("50000"));
        assert!(result.contains("25%"));
    }

    #[test]
    fn test_create_enhanced_prompt_without_stats() {
        let system = EnhancedPromptingSystem::new();
        let user_message = "Fix the authentication bug";

        let result = system.create_enhanced_prompt(user_message, None);

        // Should contain the Reflexion prompt but no context stats
        assert!(result.contains("Reflexion"));
        assert!(!result.contains("Context Status"));
    }
}
