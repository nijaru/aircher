//! MCP Integration for Intelligence Engine
//!
//! This module integrates Model Context Protocol (MCP) servers with Aircher's
//! Intelligence Engine, providing enhanced development tools and resources
//! from external MCP-compatible services.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, info};

#[cfg(feature = "mcp")]
use crate::mcp::{McpClientManager, ToolInfo, ResourceInfo, initialize_mcp};

// Placeholder types when MCP feature is not enabled
#[cfg(not(feature = "mcp"))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: Option<String>,
    pub server: String,
}

#[cfg(not(feature = "mcp"))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceInfo {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub server: String,
}

use super::{
    IntelligenceTools, ContextualInsight, ImpactAnalysis, ContextSuggestions, 
    Outcome, ProjectMomentum, CrossProjectInsight, AiConfiguration,
    Action, Pattern, ArchitecturalLesson, ImplementationExample, CodeSearchResult
};

/// MCP-enhanced Intelligence Engine that augments core intelligence
/// with tools and resources from MCP servers
pub struct McpEnhancedIntelligenceEngine<T> {
    /// Core intelligence engine
    base_intelligence: T,
    
    /// MCP client manager for external tool access
    #[cfg(feature = "mcp")]
    mcp_manager: Option<McpClientManager>,
    
    #[cfg(not(feature = "mcp"))]
    _mcp_placeholder: std::marker::PhantomData<()>,
}

impl<T> McpEnhancedIntelligenceEngine<T> 
where
    T: IntelligenceTools
{
    /// Create a new MCP-enhanced intelligence engine
    pub async fn new(base_intelligence: T) -> Result<Self> {
        #[cfg(feature = "mcp")]
        {
            info!("Initializing MCP-enhanced Intelligence Engine");
            
            // Initialize MCP subsystem
            let mcp_manager = match initialize_mcp().await {
                Ok(manager) => {
                    info!("Successfully initialized MCP client manager");
                    Some(manager)
                }
                Err(e) => {
                    warn!("Failed to initialize MCP: {}. Continuing without MCP support.", e);
                    None
                }
            };
            
            Ok(Self {
                base_intelligence,
                mcp_manager,
            })
        }
        
        #[cfg(not(feature = "mcp"))]
        {
            info!("MCP feature disabled - using base intelligence engine only");
            Ok(Self {
                base_intelligence,
                _mcp_placeholder: std::marker::PhantomData,
            })
        }
    }
    
    /// Discover and integrate available MCP tools into development context
    #[cfg(feature = "mcp")]
    async fn discover_mcp_tools(&self) -> HashMap<String, Vec<ToolInfo>> {
        if let Some(ref manager) = self.mcp_manager {
            match manager.list_all_tools().await {
                Ok(tools) => {
                    debug!("Discovered MCP tools from {} servers", tools.len());
                    tools
                }
                Err(e) => {
                    warn!("Failed to list MCP tools: {}", e);
                    HashMap::new()
                }
            }
        } else {
            HashMap::new()
        }
    }
    
    #[cfg(not(feature = "mcp"))]
    async fn discover_mcp_tools(&self) -> HashMap<String, Vec<ToolInfo>> {
        HashMap::new()
    }
    
    /// Get available MCP resources for project analysis
    #[cfg(feature = "mcp")]
    async fn get_mcp_resources(&self) -> HashMap<String, Vec<ResourceInfo>> {
        if let Some(ref manager) = self.mcp_manager {
            match manager.list_all_resources().await {
                Ok(resources) => {
                    debug!("Discovered MCP resources from {} servers", resources.len());
                    resources
                }
                Err(e) => {
                    warn!("Failed to list MCP resources: {}", e);
                    HashMap::new()
                }
            }
        } else {
            HashMap::new()
        }
    }
    
    #[cfg(not(feature = "mcp"))]
    async fn get_mcp_resources(&self) -> HashMap<String, Vec<ResourceInfo>> {
        HashMap::new()
    }
    
    /// Execute an MCP tool to enhance development context
    #[cfg(feature = "mcp")]
    async fn execute_mcp_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        parameters: Value,
    ) -> Result<Value> {
        if let Some(ref manager) = self.mcp_manager {
            let result = manager.call_tool(server_name, tool_name, parameters).await?;
            
            if let Ok(response) = result.result {
                debug!("Successfully executed MCP tool '{}' on server '{}'", tool_name, server_name);
                Ok(response)
            } else {
                Err(anyhow::anyhow!("MCP tool '{}' failed: {:?}", tool_name, result.result))
            }
        } else {
            Err(anyhow::anyhow!("MCP not available"))
        }
    }
    
    #[cfg(not(feature = "mcp"))]
    async fn execute_mcp_tool(
        &self,
        _server_name: &str,
        _tool_name: &str,
        _parameters: Value,
    ) -> Result<Value> {
        Err(anyhow::anyhow!("MCP feature not enabled"))
    }
    
    /// Analyze project using available MCP filesystem tools
    async fn enhance_project_analysis(&self, query: &str) -> Vec<Action> {
        let mut enhanced_actions = Vec::new();
        
        // Get available MCP tools
        let mcp_tools = self.discover_mcp_tools().await;
        
        // Look for filesystem and analysis tools
        for (server_name, tools) in mcp_tools {
            for tool in tools {
                match tool.name.as_str() {
                    "read_file" | "list_files" | "search_files" => {
                        enhanced_actions.push(Action {
                            action_type: "mcp_filesystem_analysis".to_string(),
                            description: format!(
                                "Use {} from {} server to analyze project files related to: {}",
                                tool.name, server_name, query
                            ),
                            confidence: 0.8,
                        });
                    }
                    "analyze_code" | "get_dependencies" | "check_syntax" => {
                        enhanced_actions.push(Action {
                            action_type: "mcp_code_analysis".to_string(),
                            description: format!(
                                "Use {} from {} server for code analysis: {}",
                                tool.name, server_name, tool.description.unwrap_or_default()
                            ),
                            confidence: 0.9,
                        });
                    }
                    "git_log" | "git_status" | "git_diff" => {
                        enhanced_actions.push(Action {
                            action_type: "mcp_version_control".to_string(),
                            description: format!(
                                "Use {} from {} server to understand recent changes",
                                tool.name, server_name
                            ),
                            confidence: 0.7,
                        });
                    }
                    _ => {
                        // Generic tool suggestion
                        enhanced_actions.push(Action {
                            action_type: "mcp_external_tool".to_string(),
                            description: format!(
                                "Consider using {} tool: {}",
                                tool.name, tool.description.unwrap_or("No description".to_string())
                            ),
                            confidence: 0.6,
                        });
                    }
                }
            }
        }
        
        enhanced_actions
    }
    
    /// Discover architectural patterns using MCP database or knowledge tools
    async fn discover_architectural_patterns(&self, _query: &str) -> Vec<ArchitecturalLesson> {
        let mut patterns = Vec::new();
        let mcp_tools = self.discover_mcp_tools().await;
        
        // Look for database or knowledge base tools
        for (server_name, tools) in mcp_tools {
            for tool in tools {
                if tool.name.contains("postgres") || tool.name.contains("database") 
                    || tool.name.contains("query") || tool.name.contains("search") {
                    
                    patterns.push(ArchitecturalLesson {
                        pattern_name: format!("MCP Database Analysis via {}", server_name),
                        description: format!(
                            "Use {} to query architectural patterns and examples from database",
                            tool.name
                        ),
                        projects_using: vec![server_name.clone()],
                        success_rate: 0.8,
                        best_practices: vec![
                            "Query for similar architectural decisions".to_string(),
                            "Analyze successful implementation patterns".to_string(),
                            format!("Leverage {} for insights", tool.description.unwrap_or_default()),
                        ],
                    });
                }
            }
        }
        
        patterns
    }
    
    /// Get enhanced implementation examples using MCP resources
    async fn get_implementation_examples(&self, query: &str) -> Vec<ImplementationExample> {
        let mut examples = Vec::new();
        let mcp_resources = self.get_mcp_resources().await;
        
        // Look through MCP resources for relevant code examples
        for (server_name, resources) in mcp_resources {
            for resource in resources {
                if resource.uri.contains(".rs") || resource.uri.contains(".js") 
                    || resource.uri.contains(".py") || resource.uri.contains(".go") {
                    
                    examples.push(ImplementationExample {
                        project_path: server_name.clone(),
                        file_path: resource.uri.clone(),
                        description: format!(
                            "Code example from {} server: {}",
                            server_name,
                            resource.description.unwrap_or("Code resource".to_string())
                        ),
                        relevance_score: if resource.name.to_lowercase().contains(&query.to_lowercase()) {
                            0.9
                        } else {
                            0.6
                        },
                    });
                }
            }
        }
        
        examples
    }
}

/// Implement IntelligenceTools for the MCP-enhanced engine
#[async_trait]
impl<T> IntelligenceTools for McpEnhancedIntelligenceEngine<T>
where
    T: IntelligenceTools + Send + Sync
{
    /// Enhanced development context with MCP capabilities
    async fn get_development_context(&self, query: &str) -> ContextualInsight {
        // Start with base intelligence
        let mut context = self.base_intelligence.get_development_context(query).await;
        
        // Enhance with MCP-powered actions
        let mcp_actions = self.enhance_project_analysis(query).await;
        context.suggested_next_actions.extend(mcp_actions);
        
        // Add MCP-discovered patterns
        let mcp_patterns = self.discover_mcp_tools().await;
        for (server_name, tools) in mcp_patterns {
            for tool in tools {
                context.recent_patterns.push(Pattern {
                    pattern_type: "mcp_tool_availability".to_string(),
                    description: format!(
                        "MCP tool available: {} from {} - {}",
                        tool.name,
                        server_name,
                        tool.description.unwrap_or_default()
                    ),
                    confidence: 0.8,
                    occurrences: 1,
                });
            }
        }
        
        debug!("Enhanced development context with MCP capabilities");
        context
    }
    
    /// Enhanced impact analysis considering MCP tool capabilities
    async fn analyze_change_impact(&self, files: &[String]) -> ImpactAnalysis {
        let mut impact = self.base_intelligence.analyze_change_impact(files).await;
        
        // Add MCP-specific impact considerations
        let mcp_tools = self.discover_mcp_tools().await;
        
        for (server_name, tools) in mcp_tools {
            for tool in tools {
                if tool.name.contains("test") || tool.name.contains("validate") {
                    impact.suggested_tests.push(format!(
                        "Use MCP tool '{}' from {} server to validate changes",
                        tool.name, server_name
                    ));
                }
                
                if tool.name.contains("lint") || tool.name.contains("check") {
                    impact.risk_areas.push(format!(
                        "Run MCP tool '{}' to check for issues",
                        tool.name
                    ));
                }
            }
        }
        
        impact
    }
    
    /// Enhanced context suggestions with MCP resources
    async fn suggest_missing_context(&self, current_files: &[String]) -> ContextSuggestions {
        let mut suggestions = self.base_intelligence.suggest_missing_context(current_files).await;
        
        // Add MCP resource suggestions
        let mcp_resources = self.get_mcp_resources().await;
        
        for (server_name, resources) in mcp_resources {
            for resource in resources {
                // Check if resource might be relevant to current files
                let is_relevant = current_files.iter().any(|file| {
                    resource.uri.contains(&std::path::Path::new(file)
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string())
                });
                
                if is_relevant {
                    suggestions.architectural_context.push(format!(
                        "Consider MCP resource: {} from {} - {}",
                        resource.name,
                        server_name,
                        resource.description.unwrap_or_default()
                    ));
                }
            }
        }
        
        suggestions
    }
    
    /// Pass through to base intelligence
    async fn track_conversation_outcome(&self, files: &[String], outcome: Outcome) {
        self.base_intelligence.track_conversation_outcome(files, outcome).await;
    }
    
    /// Enhanced project momentum with MCP tool insights
    async fn get_project_momentum(&self) -> ProjectMomentum {
        let mut momentum = self.base_intelligence.get_project_momentum().await;
        
        // Add MCP tool availability as momentum indicators
        let mcp_tools = self.discover_mcp_tools().await;
        let tool_count = mcp_tools.values().map(|tools| tools.len()).sum::<usize>();
        
        if tool_count > 0 {
            momentum.velocity_indicators.push(format!(
                "MCP integration active: {} tools available across {} servers",
                tool_count,
                mcp_tools.len()
            ));
        }
        
        momentum
    }
    
    /// Pass through to base intelligence  
    async fn add_project_directory(&self, path: &str) -> Result<(), String> {
        self.base_intelligence.add_project_directory(path).await
    }
    
    /// Enhanced cross-project analysis with MCP capabilities
    async fn analyze_cross_project_patterns(&self, query: &str) -> CrossProjectInsight {
        let mut insight = self.base_intelligence.analyze_cross_project_patterns(query).await;
        
        // Enhance with MCP-discovered patterns
        let mcp_patterns = self.discover_architectural_patterns(query).await;
        insight.architectural_lessons.extend(mcp_patterns);
        
        // Add MCP implementation examples
        let mcp_examples = self.get_implementation_examples(query).await;
        insight.implementation_examples.extend(mcp_examples);
        
        insight
    }
    
    /// Pass through to base intelligence
    async fn load_ai_configuration(&self) -> AiConfiguration {
        self.base_intelligence.load_ai_configuration().await
    }
    
    async fn search_code_semantically(&self, query: &str, limit: usize) -> Result<Vec<CodeSearchResult>, String> {
        // Delegate to base intelligence engine
        self.base_intelligence.search_code_semantically(query, limit).await
    }
}

/// MCP-specific intelligence extensions
#[async_trait]
pub trait McpIntelligenceExtensions {
    /// Get available MCP tools for the current project context
    async fn get_available_mcp_tools(&self) -> HashMap<String, Vec<ToolInfo>>;
    
    /// Get available MCP resources for the current project
    async fn get_available_mcp_resources(&self) -> HashMap<String, Vec<ResourceInfo>>;
    
    /// Execute a specific MCP tool with context awareness
    async fn execute_contextual_mcp_tool(
        &self,
        tool_identifier: &str,
        context_query: &str,
        parameters: Value,
    ) -> Result<Value>;
    
    /// Discover MCP servers relevant to current development context
    async fn discover_relevant_mcp_servers(&self, context: &str) -> Vec<String>;
}

#[async_trait]
impl<T> McpIntelligenceExtensions for McpEnhancedIntelligenceEngine<T>
where
    T: IntelligenceTools + Send + Sync
{
    async fn get_available_mcp_tools(&self) -> HashMap<String, Vec<ToolInfo>> {
        self.discover_mcp_tools().await
    }
    
    async fn get_available_mcp_resources(&self) -> HashMap<String, Vec<ResourceInfo>> {
        self.get_mcp_resources().await
    }
    
    async fn execute_contextual_mcp_tool(
        &self,
        tool_identifier: &str,
        context_query: &str,
        parameters: Value,
    ) -> Result<Value> {
        // Parse tool identifier (format: "server_name.tool_name" or just "tool_name")
        let (server_name, tool_name) = if tool_identifier.contains('.') {
            let parts: Vec<&str> = tool_identifier.splitn(2, '.').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            // Find the tool in any available server
            let tools = self.discover_mcp_tools().await;
            let mut found_server = None;
            
            for (server, server_tools) in tools {
                if server_tools.iter().any(|t| t.name == tool_identifier) {
                    found_server = Some(server);
                    break;
                }
            }
            
            match found_server {
                Some(server) => (server, tool_identifier.to_string()),
                None => return Err(anyhow::anyhow!("Tool '{}' not found in any MCP server", tool_identifier)),
            }
        };
        
        info!("Executing MCP tool '{}' on server '{}' with context '{}'", tool_name, server_name, context_query);
        self.execute_mcp_tool(&server_name, &tool_name, parameters).await
    }
    
    async fn discover_relevant_mcp_servers(&self, context: &str) -> Vec<String> {
        let tools = self.discover_mcp_tools().await;
        let mut relevant_servers = Vec::new();
        
        let context_lower = context.to_lowercase();
        
        for (server_name, server_tools) in tools {
            let is_relevant = server_tools.iter().any(|tool| {
                tool.description
                    .as_ref()
                    .map(|desc| desc.to_lowercase().contains(&context_lower))
                    .unwrap_or(false)
                || tool.name.to_lowercase().contains(&context_lower)
            });
            
            if is_relevant {
                relevant_servers.push(server_name);
            }
        }
        
        relevant_servers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intelligence::*;
    
    // Mock implementation for testing
    struct MockIntelligenceEngine;
    
    #[async_trait]
    impl IntelligenceTools for MockIntelligenceEngine {
        async fn get_development_context(&self, _query: &str) -> ContextualInsight {
            ContextualInsight {
                development_phase: "testing".to_string(),
                active_story: "mcp integration".to_string(),
                key_files: vec![],
                architectural_context: vec![],
                recent_patterns: vec![],
                suggested_next_actions: vec![],
                confidence: 0.8,
            }
        }
        
        async fn analyze_change_impact(&self, _files: &[String]) -> ImpactAnalysis {
            ImpactAnalysis {
                direct_impacts: vec![],
                indirect_impacts: vec![],
                risk_areas: vec![],
                suggested_tests: vec![],
            }
        }
        
        async fn suggest_missing_context(&self, _current_files: &[String]) -> ContextSuggestions {
            ContextSuggestions {
                missing_dependencies: vec![],
                architectural_context: vec![],
                historical_context: vec![],
                confidence: 0.7,
            }
        }
        
        async fn track_conversation_outcome(&self, _files: &[String], _outcome: Outcome) {}
        
        async fn get_project_momentum(&self) -> ProjectMomentum {
            ProjectMomentum {
                recent_focus: "testing".to_string(),
                velocity_indicators: vec![],
                architectural_direction: "forward".to_string(),
                next_priorities: vec![],
                knowledge_gaps: vec![],
            }
        }
        
        async fn add_project_directory(&self, _path: &str) -> Result<(), String> {
            Ok(())
        }
        
        async fn analyze_cross_project_patterns(&self, _query: &str) -> CrossProjectInsight {
            CrossProjectInsight {
                similar_patterns: vec![],
                architectural_lessons: vec![],
                user_preferences: vec![],
                implementation_examples: vec![],
            }
        }
        
        async fn load_ai_configuration(&self) -> AiConfiguration {
            AiConfiguration {
                global_instructions: None,
                project_instructions: None,
                cursor_rules: None,
                copilot_instructions: None,
                legacy_claude: None,
                custom_instructions: vec![],
            }
        }
    }
    
    #[tokio::test]
    async fn test_mcp_enhanced_engine_creation() {
        let base_engine = MockIntelligenceEngine;
        let enhanced_engine = McpEnhancedIntelligenceEngine::new(base_engine).await;
        
        assert!(enhanced_engine.is_ok());
    }
    
    #[tokio::test]
    async fn test_enhanced_development_context() {
        let base_engine = MockIntelligenceEngine;
        let enhanced_engine = McpEnhancedIntelligenceEngine::new(base_engine).await.unwrap();
        
        let context = enhanced_engine.get_development_context("test query").await;
        assert_eq!(context.development_phase, "testing");
        assert_eq!(context.active_story, "mcp integration");
    }
}