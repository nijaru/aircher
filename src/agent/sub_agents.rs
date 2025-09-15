use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::agent::conversation::{CodingConversation, Message, MessageRole};
use crate::agent::tools::ToolRegistry;
use crate::intelligence::IntelligenceEngine;
use crate::providers::LLMProvider;

/// Specialization types for sub-agents
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentSpecialization {
    /// Frontend development (React, Vue, HTML, CSS)
    Frontend,
    /// Backend development (APIs, databases, servers)
    Backend,
    /// DevOps and infrastructure (CI/CD, deployment, Docker)
    DevOps,
    /// Testing and QA (unit tests, integration tests, debugging)
    Testing,
    /// Research and documentation (analysis, planning, docs)
    Research,
    /// Code review and refactoring
    CodeReview,
    /// Security analysis and fixes
    Security,
    /// Performance optimization
    Performance,
    /// Custom specialization with user-defined prompt
    Custom(String),
}

impl AgentSpecialization {
    /// Get the system prompt for this specialization
    pub fn system_prompt(&self) -> String {
        match self {
            Self::Frontend => {
                "You are a frontend development specialist. Focus on user interfaces, \
                 React/Vue/Angular components, HTML/CSS, responsive design, and user experience. \
                 Prioritize clean component architecture, accessibility, and performance."
            }
            Self::Backend => {
                "You are a backend development specialist. Focus on APIs, databases, \
                 server architecture, data modeling, and system design. Prioritize \
                 scalability, security, and maintainability."
            }
            Self::DevOps => {
                "You are a DevOps specialist. Focus on CI/CD pipelines, deployment, \
                 containerization, infrastructure as code, and monitoring. Prioritize \
                 reliability, automation, and operational excellence."
            }
            Self::Testing => {
                "You are a testing and QA specialist. Focus on test coverage, \
                 test-driven development, debugging, and quality assurance. Write \
                 comprehensive tests and identify edge cases."
            }
            Self::Research => {
                "You are a research and analysis specialist. Focus on understanding \
                 requirements, analyzing codebases, planning implementations, and \
                 creating documentation. Provide thorough analysis and recommendations."
            }
            Self::CodeReview => {
                "You are a code review specialist. Focus on code quality, best practices, \
                 potential bugs, performance issues, and architectural improvements. \
                 Provide constructive feedback and suggestions."
            }
            Self::Security => {
                "You are a security specialist. Focus on identifying vulnerabilities, \
                 implementing secure coding practices, authentication, authorization, \
                 and data protection. Prioritize security without compromising usability."
            }
            Self::Performance => {
                "You are a performance optimization specialist. Focus on profiling, \
                 benchmarking, algorithmic improvements, caching strategies, and \
                 resource optimization. Provide data-driven performance improvements."
            }
            Self::Custom(prompt) => prompt,
        }.to_string()
    }

    /// Get the default allowed tools for this specialization
    pub fn default_tools(&self) -> HashSet<String> {
        let mut tools = HashSet::new();

        // Common tools for all agents
        tools.insert("read_file".to_string());
        tools.insert("search_code".to_string());
        tools.insert("list_files".to_string());

        match self {
            Self::Frontend => {
                tools.insert("write_file".to_string());
                tools.insert("edit_file".to_string());
                tools.insert("run_command".to_string());
                tools.insert("web_browsing".to_string());
            }
            Self::Backend => {
                tools.insert("write_file".to_string());
                tools.insert("edit_file".to_string());
                tools.insert("run_command".to_string());
                tools.insert("smart_commit".to_string());
            }
            Self::DevOps => {
                tools.insert("run_command".to_string());
                tools.insert("write_file".to_string());
                tools.insert("edit_file".to_string());
                tools.insert("smart_commit".to_string());
                tools.insert("branch_management".to_string());
            }
            Self::Testing => {
                tools.insert("write_file".to_string());
                tools.insert("edit_file".to_string());
                tools.insert("run_tests".to_string());
                tools.insert("run_command".to_string());
            }
            Self::Research => {
                // Read-only for research
                tools.insert("web_search".to_string());
                tools.insert("web_browsing".to_string());
                tools.insert("find_definition".to_string());
            }
            Self::CodeReview => {
                // Mostly read-only for review
                tools.insert("diagnostics".to_string());
                tools.insert("find_references".to_string());
                tools.insert("hover".to_string());
            }
            Self::Security => {
                tools.insert("diagnostics".to_string());
                tools.insert("edit_file".to_string());
                tools.insert("run_command".to_string());
            }
            Self::Performance => {
                tools.insert("run_command".to_string());
                tools.insert("edit_file".to_string());
                tools.insert("diagnostics".to_string());
            }
            Self::Custom(_) => {
                // Custom agents get minimal tools by default
                // User can override with tool permissions
            }
        }

        tools
    }
}

/// Configuration for a sub-agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentConfig {
    /// Unique name for this sub-agent
    pub name: String,
    /// Specialization type
    pub specialization: AgentSpecialization,
    /// Custom system prompt (overrides specialization default)
    pub custom_prompt: Option<String>,
    /// Allowed tool names
    pub allowed_tools: HashSet<String>,
    /// Maximum context window size (messages)
    pub max_context_size: usize,
    /// Whether this agent can delegate to other agents
    pub can_delegate: bool,
    /// Preferred model (optional override)
    pub preferred_model: Option<String>,
}

impl SubAgentConfig {
    /// Create a new sub-agent configuration
    pub fn new(name: String, specialization: AgentSpecialization) -> Self {
        let allowed_tools = specialization.default_tools();
        Self {
            name,
            specialization,
            custom_prompt: None,
            allowed_tools,
            max_context_size: 50, // Default context size
            can_delegate: false,
            preferred_model: None,
        }
    }

    /// Get the effective system prompt
    pub fn system_prompt(&self) -> String {
        self.custom_prompt
            .clone()
            .unwrap_or_else(|| self.specialization.system_prompt())
    }
}

/// A sub-agent instance with its own context and capabilities
pub struct SubAgent {
    /// Configuration for this agent
    pub config: SubAgentConfig,
    /// Dedicated conversation context
    pub conversation: Arc<RwLock<CodingConversation>>,
    /// Reference to shared tool registry
    pub tools: Arc<ToolRegistry>,
    /// Reference to shared intelligence engine
    pub intelligence: Arc<IntelligenceEngine>,
    /// Task delegation tracking
    pub delegated_tasks: RwLock<Vec<DelegatedTask>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegatedTask {
    pub task_id: String,
    pub description: String,
    pub delegated_to: String,
    pub status: TaskStatus,
    pub result: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

impl SubAgent {
    /// Create a new sub-agent
    pub fn new(
        config: SubAgentConfig,
        tools: Arc<ToolRegistry>,
        intelligence: Arc<IntelligenceEngine>,
    ) -> Self {
        let conversation = Arc::new(RwLock::new(CodingConversation::new()));

        Self {
            config,
            conversation,
            tools,
            intelligence,
            delegated_tasks: RwLock::new(Vec::new()),
        }
    }

    /// Check if a tool is allowed for this agent
    pub fn is_tool_allowed(&self, tool_name: &str) -> bool {
        self.config.allowed_tools.contains(tool_name)
    }

    /// Get filtered tool registry for this agent
    pub fn get_allowed_tools(&self) -> Vec<String> {
        self.tools
            .list_tools()
            .into_iter()
            .filter(|tool| self.is_tool_allowed(&tool.name))
            .map(|tool| tool.name)
            .collect()
    }

    /// Add a message to this agent's context
    pub async fn add_message(&self, message: Message) {
        let mut conv = self.conversation.write().await;
        conv.messages.push(message);

        // Trim context if needed
        while conv.messages.len() > self.config.max_context_size {
            conv.messages.remove(0);
        }
    }

    /// Process a task with this sub-agent
    pub async fn process_task(
        &self,
        task: &str,
        provider: &dyn LLMProvider,
        model: &str,
    ) -> Result<String> {
        info!(
            "Sub-agent {} ({:?}) processing task: {}",
            self.config.name, self.config.specialization, task
        );

        // Add system message with specialization
        self.add_message(Message {
            role: MessageRole::System,
            content: self.config.system_prompt(),
            tool_calls: None,
            timestamp: chrono::Utc::now(),
        }).await;

        // Add user task
        self.add_message(Message {
            role: MessageRole::User,
            content: task.to_string(),
            tool_calls: None,
            timestamp: chrono::Utc::now(),
        }).await;

        // TODO: Integrate with actual agent processing
        // This is a placeholder for the actual implementation
        Ok(format!(
            "{} agent processed task: {}",
            self.config.name, task
        ))
    }

    /// Delegate a task to another sub-agent
    pub async fn delegate_task(
        &self,
        task: &str,
        target_agent: &str,
    ) -> Result<String> {
        if !self.config.can_delegate {
            return Err(anyhow::anyhow!(
                "Agent {} is not allowed to delegate tasks",
                self.config.name
            ));
        }

        let task_id = uuid::Uuid::new_v4().to_string();
        let delegated_task = DelegatedTask {
            task_id: task_id.clone(),
            description: task.to_string(),
            delegated_to: target_agent.to_string(),
            status: TaskStatus::Pending,
            result: None,
        };

        let mut tasks = self.delegated_tasks.write().await;
        tasks.push(delegated_task);

        Ok(task_id)
    }
}

/// Manager for multiple sub-agents
pub struct SubAgentManager {
    /// Collection of sub-agents
    agents: RwLock<HashMap<String, Arc<SubAgent>>>,
    /// Shared tool registry
    tools: Arc<ToolRegistry>,
    /// Shared intelligence engine
    intelligence: Arc<IntelligenceEngine>,
    /// Currently active agent
    active_agent: RwLock<Option<String>>,
}

impl SubAgentManager {
    /// Create a new sub-agent manager
    pub fn new(tools: Arc<ToolRegistry>, intelligence: Arc<IntelligenceEngine>) -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
            tools,
            intelligence,
            active_agent: RwLock::new(None),
        }
    }

    /// Register a new sub-agent
    pub async fn register_agent(&self, config: SubAgentConfig) -> Result<()> {
        let name = config.name.clone();
        let agent = Arc::new(SubAgent::new(config, self.tools.clone(), self.intelligence.clone()));

        let mut agents = self.agents.write().await;
        if agents.contains_key(&name) {
            return Err(anyhow::anyhow!("Agent {} already exists", name));
        }

        agents.insert(name.clone(), agent);
        info!("Registered sub-agent: {}", name);
        Ok(())
    }

    /// Get a sub-agent by name
    pub async fn get_agent(&self, name: &str) -> Option<Arc<SubAgent>> {
        let agents = self.agents.read().await;
        agents.get(name).cloned()
    }

    /// List all registered agents
    pub async fn list_agents(&self) -> Vec<String> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
    }

    /// Switch the active agent
    pub async fn switch_agent(&self, name: &str) -> Result<()> {
        let agents = self.agents.read().await;
        if !agents.contains_key(name) {
            return Err(anyhow::anyhow!("Agent {} not found", name));
        }

        let mut active = self.active_agent.write().await;
        *active = Some(name.to_string());
        info!("Switched to agent: {}", name);
        Ok(())
    }

    /// Get the currently active agent
    pub async fn get_active_agent(&self) -> Option<Arc<SubAgent>> {
        let active = self.active_agent.read().await;
        if let Some(name) = active.as_ref() {
            self.get_agent(name).await
        } else {
            None
        }
    }

    /// Initialize default sub-agents
    pub async fn init_default_agents(&self) -> Result<()> {
        // Create default specialized agents
        let default_agents = vec![
            SubAgentConfig::new("frontend".to_string(), AgentSpecialization::Frontend),
            SubAgentConfig::new("backend".to_string(), AgentSpecialization::Backend),
            SubAgentConfig::new("devops".to_string(), AgentSpecialization::DevOps),
            SubAgentConfig::new("testing".to_string(), AgentSpecialization::Testing),
            SubAgentConfig::new("research".to_string(), AgentSpecialization::Research),
            SubAgentConfig::new("review".to_string(), AgentSpecialization::CodeReview),
            SubAgentConfig::new("security".to_string(), AgentSpecialization::Security),
            SubAgentConfig::new("performance".to_string(), AgentSpecialization::Performance),
        ];

        for config in default_agents {
            self.register_agent(config).await?;
        }

        info!("Initialized {} default sub-agents", 8);
        Ok(())
    }

    /// Route a task to the most appropriate agent
    pub async fn route_task(&self, task: &str) -> Result<String> {
        // Use intelligence engine to determine best agent
        let task_lower = task.to_lowercase();

        let agent_name = if task_lower.contains("ui") || task_lower.contains("frontend") || task_lower.contains("react") {
            "frontend"
        } else if task_lower.contains("api") || task_lower.contains("database") || task_lower.contains("backend") {
            "backend"
        } else if task_lower.contains("deploy") || task_lower.contains("docker") || task_lower.contains("ci") {
            "devops"
        } else if task_lower.contains("test") || task_lower.contains("debug") {
            "testing"
        } else if task_lower.contains("security") || task_lower.contains("vulnerability") {
            "security"
        } else if task_lower.contains("performance") || task_lower.contains("optimize") {
            "performance"
        } else if task_lower.contains("review") || task_lower.contains("refactor") {
            "review"
        } else {
            "research" // Default to research for analysis
        };

        debug!("Routing task to {} agent based on content", agent_name);
        Ok(agent_name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specialization_prompts() {
        assert!(AgentSpecialization::Frontend.system_prompt().contains("frontend"));
        assert!(AgentSpecialization::Backend.system_prompt().contains("backend"));
        assert!(AgentSpecialization::Security.system_prompt().contains("security"));
    }

    #[test]
    fn test_default_tools() {
        let frontend_tools = AgentSpecialization::Frontend.default_tools();
        assert!(frontend_tools.contains("write_file"));
        assert!(frontend_tools.contains("web_browsing"));

        let research_tools = AgentSpecialization::Research.default_tools();
        assert!(research_tools.contains("read_file"));
        assert!(!research_tools.contains("write_file")); // Read-only
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let tools = Arc::new(ToolRegistry::default());
        let intelligence = Arc::new(IntelligenceEngine::mock()); // Would need mock
        let manager = SubAgentManager::new(tools, intelligence);

        let config = SubAgentConfig::new("test".to_string(), AgentSpecialization::Frontend);
        manager.register_agent(config).await.unwrap();

        let agents = manager.list_agents().await;
        assert_eq!(agents.len(), 1);
        assert!(agents.contains(&"test".to_string()));
    }
}