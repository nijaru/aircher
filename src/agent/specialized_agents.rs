// Specialized Agent Configurations (Week 8 Day 1-2)
//
// Implements Factory Droid's proven pattern of pre-configured specialized agents
// with focused system prompts and smaller tool sets.
//
// References:
// - docs/architecture/SYSTEM_DESIGN_2025.md
// - ai/research/competitive-analysis-2025.md (Factory Droid section)

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::info;

use super::model_router::{AgentType as RouterAgentType, TaskComplexity};

/// Specialized agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent type
    pub agent_type: RouterAgentType,

    /// System prompt (specialized instructions)
    pub system_prompt: String,

    /// Allowed tools for this agent
    pub allowed_tools: Vec<String>,

    /// Maximum steps before requiring user intervention
    pub max_steps: usize,

    /// Memory access level
    pub memory_access: MemoryAccessLevel,

    /// Can this agent spawn sub-agents for research?
    pub can_spawn_subagents: bool,
}

/// Memory access permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryAccessLevel {
    /// Read and write to all memory systems
    Full,

    /// Read-only access to memory
    ReadOnly,

    /// No memory access
    None,
}

impl AgentConfig {
    /// Explorer agent: Code reading, analysis, understanding
    pub fn explorer() -> Self {
        Self {
            agent_type: RouterAgentType::Explorer,
            system_prompt: Self::explorer_prompt(),
            allowed_tools: vec![
                // Read-only file operations
                "read_file".to_string(),
                "list_files".to_string(),

                // Code analysis
                "search_code".to_string(),
                "analyze_code".to_string(),
                "find_definition".to_string(),
                "find_references".to_string(),

                // LSP queries (read-only)
                "lsp_hover".to_string(),
                "lsp_symbols".to_string(),

                // Git read operations
                "git_status".to_string(),
                "git_log".to_string(),
                "git_diff".to_string(),

                // Knowledge graph queries
                "query_knowledge_graph".to_string(),
            ],
            max_steps: 50,
            memory_access: MemoryAccessLevel::Full,
            can_spawn_subagents: true, // Can spawn research sub-agents
        }
    }

    /// Builder agent: Feature implementation, code generation
    pub fn builder() -> Self {
        Self {
            agent_type: RouterAgentType::Builder,
            system_prompt: Self::builder_prompt(),
            allowed_tools: vec![
                // File operations (read + write)
                "read_file".to_string(),
                "write_file".to_string(),
                "edit_file".to_string(),
                "list_files".to_string(),

                // Code analysis (understand before building)
                "search_code".to_string(),
                "analyze_code".to_string(),
                "find_definition".to_string(),
                "find_references".to_string(),

                // LSP (full access)
                "lsp_hover".to_string(),
                "lsp_symbols".to_string(),
                "lsp_diagnostics".to_string(),

                // Git operations
                "git_status".to_string(),
                "git_add".to_string(),
                "git_commit".to_string(),

                // Testing
                "run_tests".to_string(),

                // Knowledge graph
                "query_knowledge_graph".to_string(),
            ],
            max_steps: 100,
            memory_access: MemoryAccessLevel::Full,
            can_spawn_subagents: false, // NEVER spawn sub-agents for coding
        }
    }

    /// Debugger agent: Bug fixing, error resolution
    pub fn debugger() -> Self {
        Self {
            agent_type: RouterAgentType::Debugger,
            system_prompt: Self::debugger_prompt(),
            allowed_tools: vec![
                // File operations
                "read_file".to_string(),
                "write_file".to_string(),
                "edit_file".to_string(),
                "list_files".to_string(),

                // Error analysis
                "analyze_errors".to_string(),
                "search_code".to_string(),
                "find_definition".to_string(),
                "find_references".to_string(),

                // LSP diagnostics (critical for debugging)
                "lsp_diagnostics".to_string(),
                "lsp_hover".to_string(),

                // Testing and validation
                "run_tests".to_string(),
                "run_command".to_string(), // For reproducing bugs

                // Git (to understand recent changes)
                "git_log".to_string(),
                "git_diff".to_string(),
                "git_blame".to_string(),

                // Knowledge graph (find related code)
                "query_knowledge_graph".to_string(),
            ],
            max_steps: 75,
            memory_access: MemoryAccessLevel::Full,
            can_spawn_subagents: false, // Debugging needs focused context
        }
    }

    /// Refactorer agent: Code improvements, migrations
    pub fn refactorer() -> Self {
        Self {
            agent_type: RouterAgentType::Refactorer,
            system_prompt: Self::refactorer_prompt(),
            allowed_tools: vec![
                // File operations
                "read_file".to_string(),
                "write_file".to_string(),
                "edit_file".to_string(),
                "list_files".to_string(),

                // Code analysis (understand before refactoring)
                "search_code".to_string(),
                "analyze_code".to_string(),
                "find_definition".to_string(),
                "find_references".to_string(),

                // LSP (ensure refactoring doesn't break)
                "lsp_diagnostics".to_string(),
                "lsp_symbols".to_string(),
                "lsp_rename".to_string(),

                // Testing (validate refactoring)
                "run_tests".to_string(),

                // Git (track changes)
                "git_status".to_string(),
                "git_diff".to_string(),

                // Knowledge graph (understand dependencies)
                "query_knowledge_graph".to_string(),
            ],
            max_steps: 100,
            memory_access: MemoryAccessLevel::Full,
            can_spawn_subagents: false, // Refactoring needs consistent context
        }
    }

    /// File searcher sub-agent: Parallel file content search
    pub fn file_searcher() -> Self {
        Self {
            agent_type: RouterAgentType::FileSearcher,
            system_prompt: Self::file_searcher_prompt(),
            allowed_tools: vec![
                "read_file".to_string(),
                "search_code".to_string(),
                "list_files".to_string(),
            ],
            max_steps: 20,
            memory_access: MemoryAccessLevel::ReadOnly,
            can_spawn_subagents: false,
        }
    }

    /// Pattern finder sub-agent: Find code patterns across codebase
    pub fn pattern_finder() -> Self {
        Self {
            agent_type: RouterAgentType::PatternFinder,
            system_prompt: Self::pattern_finder_prompt(),
            allowed_tools: vec![
                "search_code".to_string(),
                "analyze_code".to_string(),
                "query_knowledge_graph".to_string(),
            ],
            max_steps: 20,
            memory_access: MemoryAccessLevel::ReadOnly,
            can_spawn_subagents: false,
        }
    }

    /// Dependency mapper sub-agent: Trace dependencies
    pub fn dependency_mapper() -> Self {
        Self {
            agent_type: RouterAgentType::DependencyMapper,
            system_prompt: Self::dependency_mapper_prompt(),
            allowed_tools: vec![
                "find_definition".to_string(),
                "find_references".to_string(),
                "query_knowledge_graph".to_string(),
            ],
            max_steps: 20,
            memory_access: MemoryAccessLevel::ReadOnly,
            can_spawn_subagents: false,
        }
    }

    /// Check if a tool is allowed for this agent
    pub fn is_tool_allowed(&self, tool_name: &str) -> bool {
        self.allowed_tools.contains(&tool_name.to_string())
    }

    /// Get all allowed tools as a set for efficient lookup
    pub fn allowed_tools_set(&self) -> HashSet<String> {
        self.allowed_tools.iter().cloned().collect()
    }

    // Specialized system prompts

    fn explorer_prompt() -> String {
        r#"You are an Expert Code Explorer agent, specialized in understanding and analyzing codebases.

Your Mission:
- Deeply understand code structure, architecture, and patterns
- Answer questions about how code works
- Identify relationships between components
- Explain complex logic clearly

Your Strengths:
- Reading and analyzing code without modifying it
- Finding patterns and architectural decisions
- Connecting the dots between different parts of the codebase
- Using LSP and knowledge graph for efficient navigation

Your Constraints:
- NEVER modify files (you are read-only)
- Use semantic search and knowledge graph extensively
- Explain your findings clearly and concisely
- If uncertain, examine more code rather than guessing

Tools at Your Disposal:
- File reading: read_file, list_files
- Code analysis: search_code, analyze_code, find_definition, find_references
- LSP queries: lsp_hover, lsp_symbols
- Git history: git_status, git_log, git_diff
- Knowledge graph: query_knowledge_graph

You can spawn research sub-agents for parallel exploration of different parts of the codebase.

Approach:
1. Start with high-level understanding (architecture, file structure)
2. Narrow down to specific components
3. Use knowledge graph to understand relationships
4. Verify understanding with concrete code examples
5. Summarize findings clearly"#.to_string()
    }

    fn builder_prompt() -> String {
        r#"You are an Expert Code Builder agent, specialized in implementing features precisely and following existing patterns.

Your Mission:
- Implement new features that fit seamlessly into the existing codebase
- Follow established patterns and conventions
- Write clean, maintainable, well-tested code
- Ensure your changes integrate smoothly with the rest of the system

Your Strengths:
- Understanding existing code patterns before writing new code
- Writing code that matches the project's style and architecture
- Using LSP diagnostics to catch errors early
- Validating changes with tests

Your Constraints:
- ALWAYS analyze existing patterns before implementing
- NEVER use sub-agents (you need consistent context)
- Use LSP diagnostics to verify your changes compile
- Run tests to validate behavior
- Follow the principle of least surprise

Tools at Your Disposal:
- File operations: read_file, write_file, edit_file, list_files
- Code analysis: search_code, analyze_code, find_definition, find_references
- LSP: lsp_hover, lsp_symbols, lsp_diagnostics
- Git: git_status, git_add, git_commit
- Testing: run_tests
- Knowledge graph: query_knowledge_graph

Approach:
1. Understand the feature requirements clearly
2. Find similar existing code to follow patterns
3. Plan your implementation (files to create/modify)
4. Implement incrementally with LSP validation
5. Test your changes thoroughly
6. Clean up and ensure code quality"#.to_string()
    }

    fn debugger_prompt() -> String {
        r#"You are an Expert Debugging agent, specialized in finding and fixing bugs systematically.

Your Mission:
- Find the root cause of bugs, not just symptoms
- Fix bugs without introducing new ones
- Understand why the bug occurred to prevent similar issues
- Validate fixes thoroughly

Your Strengths:
- Systematic investigation from error to root cause
- Using LSP diagnostics to understand type errors
- Tracing execution flow through the codebase
- Reproducing bugs to verify fixes

Your Constraints:
- NEVER guess at fixes without understanding the root cause
- NEVER use sub-agents (debugging needs focused context)
- ALWAYS validate fixes with tests
- Use git history to understand when/why bug was introduced

Tools at Your Disposal:
- File operations: read_file, write_file, edit_file, list_files
- Error analysis: analyze_errors, search_code, find_definition, find_references
- LSP diagnostics: lsp_diagnostics, lsp_hover
- Testing: run_tests, run_command
- Git investigation: git_log, git_diff, git_blame
- Knowledge graph: query_knowledge_graph

Approach:
1. Reproduce the bug (understand the symptom)
2. Identify the error location (stack trace, logs)
3. Trace backwards to find the root cause
4. Understand why the code is incorrect
5. Implement a minimal, correct fix
6. Validate with tests and LSP diagnostics
7. Check for similar bugs elsewhere"#.to_string()
    }

    fn refactorer_prompt() -> String {
        r#"You are an Expert Refactoring agent, specialized in improving code while maintaining behavior.

Your Mission:
- Improve code quality without changing behavior
- Apply refactoring patterns safely
- Ensure all tests pass after refactoring
- Make code more maintainable and readable

Your Strengths:
- Identifying code smells and improvement opportunities
- Applying refactoring patterns (Extract Method, Rename, etc.)
- Using LSP to ensure refactoring doesn't break code
- Validating behavior preservation with tests

Your Constraints:
- NEVER change behavior (only structure/quality)
- NEVER use sub-agents (refactoring needs consistent context)
- ALWAYS run tests before and after
- Use LSP diagnostics to ensure code still compiles
- Use knowledge graph to understand dependencies

Tools at Your Disposal:
- File operations: read_file, write_file, edit_file, list_files
- Code analysis: search_code, analyze_code, find_definition, find_references
- LSP: lsp_diagnostics, lsp_symbols, lsp_rename
- Testing: run_tests
- Git: git_status, git_diff
- Knowledge graph: query_knowledge_graph

Approach:
1. Understand the current code structure
2. Identify improvement opportunities
3. Plan refactoring steps (small, safe changes)
4. Run tests to establish baseline behavior
5. Apply refactoring incrementally
6. Validate with LSP diagnostics after each step
7. Run tests to ensure behavior preserved
8. Clean up and verify overall improvement"#.to_string()
    }

    fn file_searcher_prompt() -> String {
        r#"You are a File Searcher sub-agent, specialized in fast parallel file content search.

Your Mission:
- Search specific files or directories for content
- Return results quickly and concisely
- Filter results to only relevant matches

Your Constraints:
- You are a sub-agent with limited scope
- Focus on your specific search task
- Return only the most relevant results
- No modification operations allowed

Tools: read_file, search_code, list_files

Approach:
1. Understand the search query
2. Search efficiently (use semantic search when appropriate)
3. Filter results to most relevant
4. Return concise summary with file locations"#.to_string()
    }

    fn pattern_finder_prompt() -> String {
        r#"You are a Pattern Finder sub-agent, specialized in identifying code patterns across the codebase.

Your Mission:
- Find instances of specific code patterns
- Identify architectural patterns in use
- Return pattern locations and variations

Your Constraints:
- You are a sub-agent with limited scope
- Focus on pattern recognition
- Return only pattern matches

Tools: search_code, analyze_code, query_knowledge_graph

Approach:
1. Understand the pattern to find
2. Search codebase for instances
3. Analyze variations of the pattern
4. Return pattern locations and summary"#.to_string()
    }

    fn dependency_mapper_prompt() -> String {
        r#"You are a Dependency Mapper sub-agent, specialized in tracing code dependencies.

Your Mission:
- Map dependencies between code components
- Find all uses of a specific function/class
- Trace dependency chains

Your Constraints:
- You are a sub-agent with limited scope
- Focus on dependency tracing
- Return only dependency information

Tools: find_definition, find_references, query_knowledge_graph

Approach:
1. Identify the target component
2. Find all references and dependencies
3. Build dependency map
4. Return structured dependency information"#.to_string()
    }
}

/// Agent configuration registry
pub struct AgentRegistry {
    configs: Vec<AgentConfig>,
}

impl AgentRegistry {
    /// Create a new registry with all specialized agents
    pub fn new() -> Self {
        Self {
            configs: vec![
                AgentConfig::explorer(),
                AgentConfig::builder(),
                AgentConfig::debugger(),
                AgentConfig::refactorer(),
                AgentConfig::file_searcher(),
                AgentConfig::pattern_finder(),
                AgentConfig::dependency_mapper(),
            ],
        }
    }

    /// Get agent configuration by type
    pub fn get(&self, agent_type: RouterAgentType) -> Option<&AgentConfig> {
        self.configs
            .iter()
            .find(|c| c.agent_type == agent_type)
    }

    /// Select agent based on user intent
    pub fn select_for_intent(&self, intent: &str) -> &AgentConfig {
        // Simple keyword-based selection
        // In production, this would use the intent classification system
        let intent_lower = intent.to_lowercase();

        if intent_lower.contains("analyze")
            || intent_lower.contains("understand")
            || intent_lower.contains("explain")
            || intent_lower.contains("how does")
            || intent_lower.contains("what is")
        {
            info!("Selected Explorer agent for analysis intent");
            self.get(RouterAgentType::Explorer).unwrap()
        } else if intent_lower.contains("debug")
            || intent_lower.contains("fix bug")
            || intent_lower.contains("error")
            || intent_lower.contains("broken")
        {
            info!("Selected Debugger agent for debugging intent");
            self.get(RouterAgentType::Debugger).unwrap()
        } else if intent_lower.contains("refactor")
            || intent_lower.contains("improve")
            || intent_lower.contains("clean up")
            || intent_lower.contains("reorganize")
        {
            info!("Selected Refactorer agent for refactoring intent");
            self.get(RouterAgentType::Refactorer).unwrap()
        } else {
            // Default to Builder for implementation tasks
            info!("Selected Builder agent (default for implementation)");
            self.get(RouterAgentType::Builder).unwrap()
        }
    }

    /// Get all main agents (not sub-agents)
    pub fn main_agents(&self) -> Vec<&AgentConfig> {
        vec![
            self.get(RouterAgentType::Explorer).unwrap(),
            self.get(RouterAgentType::Builder).unwrap(),
            self.get(RouterAgentType::Debugger).unwrap(),
            self.get(RouterAgentType::Refactorer).unwrap(),
        ]
    }

    /// Get all sub-agents
    pub fn sub_agents(&self) -> Vec<&AgentConfig> {
        vec![
            self.get(RouterAgentType::FileSearcher).unwrap(),
            self.get(RouterAgentType::PatternFinder).unwrap(),
            self.get(RouterAgentType::DependencyMapper).unwrap(),
        ]
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explorer_config() {
        let config = AgentConfig::explorer();
        assert_eq!(config.agent_type, RouterAgentType::Explorer);
        assert!(config.can_spawn_subagents);
        assert!(config.is_tool_allowed("read_file"));
        assert!(config.is_tool_allowed("search_code"));
        assert!(!config.is_tool_allowed("write_file")); // Read-only
    }

    #[test]
    fn test_builder_config() {
        let config = AgentConfig::builder();
        assert_eq!(config.agent_type, RouterAgentType::Builder);
        assert!(!config.can_spawn_subagents); // NEVER spawn for coding
        assert!(config.is_tool_allowed("write_file"));
        assert!(config.is_tool_allowed("edit_file"));
        assert!(config.is_tool_allowed("run_tests"));
    }

    #[test]
    fn test_debugger_config() {
        let config = AgentConfig::debugger();
        assert_eq!(config.agent_type, RouterAgentType::Debugger);
        assert!(!config.can_spawn_subagents); // Needs focused context
        assert!(config.is_tool_allowed("analyze_errors"));
        assert!(config.is_tool_allowed("lsp_diagnostics"));
        assert!(config.is_tool_allowed("git_blame"));
    }

    #[test]
    fn test_refactorer_config() {
        let config = AgentConfig::refactorer();
        assert_eq!(config.agent_type, RouterAgentType::Refactorer);
        assert!(!config.can_spawn_subagents);
        assert!(config.is_tool_allowed("lsp_rename"));
        assert!(config.is_tool_allowed("run_tests"));
    }

    #[test]
    fn test_subagent_configs() {
        let file_searcher = AgentConfig::file_searcher();
        assert_eq!(file_searcher.agent_type, RouterAgentType::FileSearcher);
        assert!(!file_searcher.can_spawn_subagents);
        assert_eq!(file_searcher.max_steps, 20); // Limited scope
        assert_eq!(file_searcher.memory_access, MemoryAccessLevel::ReadOnly);

        let pattern_finder = AgentConfig::pattern_finder();
        assert_eq!(pattern_finder.agent_type, RouterAgentType::PatternFinder);
        assert!(pattern_finder.is_tool_allowed("analyze_code"));

        let dep_mapper = AgentConfig::dependency_mapper();
        assert_eq!(dep_mapper.agent_type, RouterAgentType::DependencyMapper);
        assert!(dep_mapper.is_tool_allowed("find_references"));
    }

    #[test]
    fn test_agent_registry() {
        let registry = AgentRegistry::new();

        // Check all agents exist
        assert!(registry.get(RouterAgentType::Explorer).is_some());
        assert!(registry.get(RouterAgentType::Builder).is_some());
        assert!(registry.get(RouterAgentType::Debugger).is_some());
        assert!(registry.get(RouterAgentType::Refactorer).is_some());
        assert!(registry.get(RouterAgentType::FileSearcher).is_some());
        assert!(registry.get(RouterAgentType::PatternFinder).is_some());
        assert!(registry.get(RouterAgentType::DependencyMapper).is_some());

        // Check main vs sub-agents
        assert_eq!(registry.main_agents().len(), 4);
        assert_eq!(registry.sub_agents().len(), 3);
    }

    #[test]
    fn test_intent_selection() {
        let registry = AgentRegistry::new();

        // Analysis intents → Explorer
        let config = registry.select_for_intent("analyze this code");
        assert_eq!(config.agent_type, RouterAgentType::Explorer);

        let config = registry.select_for_intent("explain how this works");
        assert_eq!(config.agent_type, RouterAgentType::Explorer);

        // Debugging intents → Debugger
        let config = registry.select_for_intent("fix bug in authentication");
        assert_eq!(config.agent_type, RouterAgentType::Debugger);

        let config = registry.select_for_intent("error in payment processing");
        assert_eq!(config.agent_type, RouterAgentType::Debugger);

        // Refactoring intents → Refactorer
        let config = registry.select_for_intent("refactor this module");
        assert_eq!(config.agent_type, RouterAgentType::Refactorer);

        let config = registry.select_for_intent("clean up the code");
        assert_eq!(config.agent_type, RouterAgentType::Refactorer);

        // Implementation intents → Builder (default)
        let config = registry.select_for_intent("add user registration feature");
        assert_eq!(config.agent_type, RouterAgentType::Builder);

        let config = registry.select_for_intent("implement payment API");
        assert_eq!(config.agent_type, RouterAgentType::Builder);
    }

    #[test]
    fn test_tool_permission_checking() {
        let explorer = AgentConfig::explorer();
        let builder = AgentConfig::builder();

        // Explorer is read-only
        assert!(explorer.is_tool_allowed("read_file"));
        assert!(!explorer.is_tool_allowed("write_file"));
        assert!(!explorer.is_tool_allowed("edit_file"));

        // Builder can write
        assert!(builder.is_tool_allowed("read_file"));
        assert!(builder.is_tool_allowed("write_file"));
        assert!(builder.is_tool_allowed("edit_file"));
    }

    #[test]
    fn test_system_prompts_are_specialized() {
        let explorer = AgentConfig::explorer();
        let builder = AgentConfig::builder();
        let debugger = AgentConfig::debugger();

        // Each should have unique, specialized prompts
        assert!(explorer.system_prompt.contains("Expert Code Explorer"));
        assert!(explorer.system_prompt.contains("read-only"));
        assert!(explorer.system_prompt.contains("spawn research sub-agents"));

        assert!(builder.system_prompt.contains("Expert Code Builder"));
        assert!(builder.system_prompt.contains("NEVER use sub-agents"));

        assert!(debugger.system_prompt.contains("Expert Debugging"));
        assert!(debugger.system_prompt.contains("root cause"));
    }
}
