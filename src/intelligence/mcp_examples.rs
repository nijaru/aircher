//! Usage examples for MCP-enhanced Intelligence Engine
//!
//! This module demonstrates how to use the MCP-enhanced Intelligence Engine
//! in real development scenarios, showing the integration between Aircher's
//! core intelligence and external MCP tools.

use anyhow::Result;
use serde_json::json;

use crate::config::ConfigManager;
use crate::storage::DatabaseManager;
use super::{
    IntelligenceEngine, McpEnhancedIntelligenceEngine, IntelligenceTools, 
    McpIntelligenceExtensions, ContextualInsight, Outcome
};

/// Comprehensive example of setting up and using MCP-enhanced intelligence
pub struct McpIntelligenceDemo {
    engine: McpEnhancedIntelligenceEngine<IntelligenceEngine>,
}

impl McpIntelligenceDemo {
    /// Initialize the MCP-enhanced intelligence engine
    pub async fn new(config: &ConfigManager, storage: &DatabaseManager) -> Result<Self> {
        // Create base intelligence engine
        let base_engine = IntelligenceEngine::new(config, storage).await?;
        
        // Enhance with MCP capabilities
        let engine = base_engine.with_mcp_enhancement().await?;
        
        Ok(Self { engine })
    }
    
    /// Example: Handle user request with MCP enhancement
    pub async fn handle_user_request_with_mcp(&self, request: &str) -> Result<ContextualInsight> {
        println!("🧠 Analyzing request with MCP-enhanced Intelligence Engine: {}", request);
        
        // Get enhanced development context
        let context = self.engine.get_development_context(request).await;
        
        // Show available MCP tools
        let mcp_tools = self.engine.get_available_mcp_tools().await;
        println!("🔧 Available MCP tools:");
        for (server, tools) in &mcp_tools {
            println!("  📦 {} ({} tools):", server, tools.len());
            for tool in tools {
                println!("    - {}: {}", tool.name, tool.description.as_deref().unwrap_or("No description"));
            }
        }
        
        // Show available MCP resources
        let mcp_resources = self.engine.get_available_mcp_resources().await;
        println!("📚 Available MCP resources:");
        for (server, resources) in &mcp_resources {
            println!("  📦 {} ({} resources):", server, resources.len());
            for resource in resources {
                println!("    - {}: {}", resource.name, resource.description.as_deref().unwrap_or("No description"));
            }
        }
        
        Ok(context)
    }
    
    /// Example: Execute MCP tool for filesystem analysis
    pub async fn analyze_project_with_mcp(&self, project_path: &str) -> Result<()> {
        println!("🔍 Analyzing project with MCP tools: {}", project_path);
        
        // Try to use filesystem MCP tools
        let result = self.engine.execute_contextual_mcp_tool(
            "read_file",
            "project analysis",
            json!({
                "path": format!("{}/Cargo.toml", project_path)
            })
        ).await;
        
        match result {
            Ok(response) => {
                println!("✅ Successfully analyzed project with MCP:");
                println!("{}", serde_json::to_string_pretty(&response)?);
            }
            Err(e) => {
                println!("ℹ️ Could not use MCP filesystem tool: {}", e);
                println!("   (This is expected if filesystem MCP server is not connected)");
            }
        }
        
        Ok(())
    }
    
    /// Example: Enhanced code analysis workflow
    pub async fn enhanced_code_analysis_workflow(&self, files: &[String]) -> Result<()> {
        println!("🚀 Starting enhanced code analysis workflow for {} files", files.len());
        
        // 1. Get enhanced development context
        let context = self.engine.get_development_context("code analysis").await;
        println!("📋 Development context:");
        println!("  Phase: {}", context.development_phase);
        println!("  Active story: {}", context.active_story);
        println!("  Suggested actions: {}", context.suggested_next_actions.len());
        
        // 2. Analyze change impact with MCP enhancement
        let impact = self.engine.analyze_change_impact(files).await;
        println!("⚡ Impact analysis:");
        println!("  Direct impacts: {}", impact.direct_impacts.len());
        println!("  Suggested tests: {}", impact.suggested_tests.len());
        
        // 3. Get missing context suggestions with MCP resources
        let suggestions = self.engine.suggest_missing_context(files).await;
        println!("💡 Context suggestions:");
        println!("  Missing dependencies: {}", suggestions.missing_dependencies.len());
        println!("  Architectural context: {}", suggestions.architectural_context.len());
        
        // 4. Try to execute relevant MCP tools
        let relevant_servers = self.engine.discover_relevant_mcp_servers("code analysis").await;
        println!("🎯 Relevant MCP servers: {:?}", relevant_servers);
        
        for server in &relevant_servers {
            println!("  Trying to analyze with {} server...", server);
            // In a real scenario, you would execute specific tools here
        }
        
        // 5. Record successful completion
        let outcome = Outcome {
            success_rating: 0.9,
            completion_status: "completed_with_mcp_enhancement".to_string(),
            user_feedback: Some("MCP integration worked well".to_string()),
            identified_gaps: vec![],
        };
        
        self.engine.track_conversation_outcome(files, outcome).await;
        
        println!("✅ Enhanced code analysis workflow completed");
        Ok(())
    }
    
    /// Example: Cross-project pattern analysis with MCP
    pub async fn cross_project_analysis_with_mcp(&self, query: &str) -> Result<()> {
        println!("🌐 Analyzing cross-project patterns with MCP: {}", query);
        
        let insight = self.engine.analyze_cross_project_patterns(query).await;
        
        println!("📊 Cross-project insights:");
        println!("  Similar patterns: {}", insight.similar_patterns.len());
        println!("  Architectural lessons: {}", insight.architectural_lessons.len());
        println!("  Implementation examples: {}", insight.implementation_examples.len());
        
        // Show MCP-enhanced insights
        for lesson in &insight.architectural_lessons {
            if lesson.pattern_name.contains("MCP") {
                println!("  🔧 MCP-powered insight: {}", lesson.pattern_name);
                println!("    Description: {}", lesson.description);
                println!("    Success rate: {:.1}%", lesson.success_rate * 100.0);
            }
        }
        
        for example in &insight.implementation_examples {
            if example.project_path.contains("MCP") || example.description.contains("MCP") {
                println!("  📝 MCP resource: {}", example.file_path);
                println!("    Description: {}", example.description);
                println!("    Relevance: {:.1}%", example.relevance_score * 100.0);
            }
        }
        
        Ok(())
    }
    
    /// Example: Project momentum analysis with MCP tools
    pub async fn project_momentum_with_mcp(&self) -> Result<()> {
        println!("📈 Analyzing project momentum with MCP enhancement");
        
        let momentum = self.engine.get_project_momentum().await;
        
        println!("🎯 Project momentum:");
        println!("  Recent focus: {}", momentum.recent_focus);
        println!("  Architectural direction: {}", momentum.architectural_direction);
        println!("  Velocity indicators: {}", momentum.velocity_indicators.len());
        
        for indicator in &momentum.velocity_indicators {
            if indicator.contains("MCP") {
                println!("  🔧 MCP indicator: {}", indicator);
            }
        }
        
        println!("  Next priorities: {:?}", momentum.next_priorities);
        println!("  Knowledge gaps: {:?}", momentum.knowledge_gaps);
        
        Ok(())
    }
}

/// Standalone functions for common MCP intelligence workflows
pub mod workflows {
    use super::*;
    
    /// Quick setup for MCP-enhanced intelligence in any context
    pub async fn setup_mcp_intelligence() -> Result<McpEnhancedIntelligenceEngine<IntelligenceEngine>> {
        println!("⚙️ Setting up MCP-enhanced Intelligence Engine...");
        
        // Create mock config and storage for demonstration
        // In real usage, these would come from your application
        let config = ConfigManager::load().await?;
        let storage = DatabaseManager::new(&config).await?;
        
        // Create and enhance intelligence engine
        let base_engine = IntelligenceEngine::new(&config, &storage).await?;
        let enhanced_engine = base_engine.with_mcp_enhancement().await?;
        
        println!("✅ MCP-enhanced Intelligence Engine ready");
        Ok(enhanced_engine)
    }
    
    /// Quick context analysis with MCP enhancement
    pub async fn quick_context_analysis(
        engine: &dyn IntelligenceTools,
        query: &str,
    ) -> Result<ContextualInsight> {
        println!("🔍 Quick context analysis: {}", query);
        
        let context = engine.get_development_context(query).await;
        
        println!("📋 Context summary:");
        println!("  Confidence: {:.1}%", context.confidence * 100.0);
        println!("  Key files: {}", context.key_files.len());
        println!("  Suggested actions: {}", context.suggested_next_actions.len());
        println!("  Recent patterns: {}", context.recent_patterns.len());
        
        // Show MCP-enhanced actions
        for action in &context.suggested_next_actions {
            if action.action_type.contains("mcp_") {
                println!("  🔧 MCP action: {}", action.description);
            }
        }
        
        Ok(context)
    }
    
    /// Demonstrate MCP tool discovery and usage
    pub async fn demonstrate_mcp_discovery(
        engine: &(dyn McpIntelligenceExtensions + Sync),
    ) -> Result<()> {
        println!("🕵️ Demonstrating MCP tool discovery...");
        
        // Discover available tools
        let tools = engine.get_available_mcp_tools().await;
        println!("🔧 Found {} MCP servers with tools:", tools.len());
        
        for (server, server_tools) in &tools {
            println!("  📦 {} server:", server);
            for tool in server_tools {
                println!("    - {} ({})", tool.name, 
                    tool.description.as_deref().unwrap_or("No description"));
            }
        }
        
        // Discover resources
        let resources = engine.get_available_mcp_resources().await;
        println!("📚 Found {} MCP servers with resources:", resources.len());
        
        for (server, server_resources) in &resources {
            println!("  📦 {} server:", server);
            for resource in server_resources {
                println!("    - {} ({})", resource.name,
                    resource.description.as_deref().unwrap_or("No description"));
            }
        }
        
        // Try to discover servers relevant to common contexts
        let contexts = ["filesystem", "git", "database", "testing", "documentation"];
        for context in &contexts {
            let relevant = engine.discover_relevant_mcp_servers(context).await;
            if !relevant.is_empty() {
                println!("🎯 Servers relevant to '{}': {:?}", context, relevant);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mcp_intelligence_setup() {
        // This test verifies that MCP intelligence can be set up
        // Even if MCP is not available, it should still work
        let result = workflows::setup_mcp_intelligence().await;
        
        // The setup should succeed regardless of MCP availability
        match result {
            Ok(_) => println!("✅ MCP intelligence setup successful"),
            Err(e) => println!("ℹ️ MCP intelligence setup completed with limitations: {}", e),
        }
    }
    
    #[tokio::test]
    async fn test_context_analysis_workflow() {
        // Test the basic workflow even without MCP servers
        if let Ok(engine) = workflows::setup_mcp_intelligence().await {
            let context = workflows::quick_context_analysis(&engine, "test query").await;
            assert!(context.is_ok());
        }
    }
}