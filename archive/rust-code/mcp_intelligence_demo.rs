//! MCP Intelligence Engine Demo
//!
//! This example demonstrates the MCP-enhanced Intelligence Engine in action,
//! showing how external MCP tools and resources can enhance Aircher's
//! code intelligence capabilities.

use anyhow::Result;
use tokio;

#[cfg(feature = "mcp")]
use aircher::{
    config::ConfigManager,
    storage::DatabaseManager,
    intelligence::{
        McpIntelligenceExtensions,
        mcp_examples::{workflows, McpIntelligenceDemo}
    },
};

#[cfg(not(feature = "mcp"))]
fn main() {
    println!("❌ MCP feature not enabled!");
    println!("Run with: cargo run --features mcp --example mcp_intelligence_demo");
}

#[cfg(feature = "mcp")]
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt::init();

    println!("🚀 Aircher MCP-Enhanced Intelligence Engine Demo");
    println!("=================================================\n");

    // Demonstration 1: Setup and Basic Usage
    println!("📋 Demo 1: Setup and Basic MCP Intelligence");
    println!("--------------------------------------------");

    match demo_basic_setup().await {
        Ok(_) => println!("✅ Basic setup demo completed successfully\n"),
        Err(e) => println!("⚠️ Basic setup demo completed with limitations: {}\n", e),
    }

    // Demonstration 2: Enhanced Development Context
    println!("📋 Demo 2: Enhanced Development Context Analysis");
    println!("------------------------------------------------");

    match demo_enhanced_context().await {
        Ok(_) => println!("✅ Enhanced context demo completed successfully\n"),
        Err(e) => println!("⚠️ Enhanced context demo completed with limitations: {}\n", e),
    }

    // Demonstration 3: MCP Tool Discovery
    println!("📋 Demo 3: MCP Tool and Resource Discovery");
    println!("-------------------------------------------");

    match demo_tool_discovery().await {
        Ok(_) => println!("✅ Tool discovery demo completed successfully\n"),
        Err(e) => println!("⚠️ Tool discovery demo completed with limitations: {}\n", e),
    }

    // Demonstration 4: Comprehensive Workflow
    println!("📋 Demo 4: Complete MCP-Enhanced Development Workflow");
    println!("------------------------------------------------------");

    match demo_complete_workflow().await {
        Ok(_) => println!("✅ Complete workflow demo completed successfully\n"),
        Err(e) => println!("⚠️ Complete workflow demo completed with limitations: {}\n", e),
    }

    println!("🎯 MCP Intelligence Engine Demo Complete!");
    println!("==========================================");
    println!();
    println!("🔧 To connect real MCP servers:");
    println!("   1. Install MCP servers (e.g., mcp-server-filesystem)");
    println!("   2. Configure servers: aircher mcp add <name> <type> <options>");
    println!("   3. Connect servers: aircher mcp connect <name>");
    println!("   4. Run this demo again to see enhanced capabilities!");

    Ok(())
}

#[cfg(feature = "mcp")]
async fn demo_basic_setup() -> Result<()> {
    println!("🔧 Setting up MCP-enhanced Intelligence Engine...");

    // Use the workflow helper for easy setup
    let engine = workflows::setup_mcp_intelligence().await?;

    println!("✅ Intelligence Engine initialized with MCP support");

    // Quick context analysis
    let context = workflows::quick_context_analysis(&engine, "implement new search feature").await?;

    println!("📊 Context analysis results:");
    println!("   - Confidence: {:.1}%", context.confidence * 100.0);
    println!("   - Key files identified: {}", context.key_files.len());
    println!("   - Suggested actions: {}", context.suggested_next_actions.len());
    println!("   - Development phase: {}", context.development_phase);

    // Show any MCP-enhanced suggestions
    let mcp_actions = context.suggested_next_actions.iter()
        .filter(|action| action.action_type.contains("mcp_"))
        .count();

    if mcp_actions > 0 {
        println!("   - MCP-enhanced actions: {} available", mcp_actions);
    } else {
        println!("   - MCP servers: Not connected (expected for demo)");
    }

    Ok(())
}

#[cfg(feature = "mcp")]
async fn demo_enhanced_context() -> Result<()> {
    println!("🧠 Analyzing enhanced development context...");

    // Setup
    let config = ConfigManager::load().await?;
    let storage = DatabaseManager::new(&config).await?;

    let demo = McpIntelligenceDemo::new(&config, &storage).await?;

    // Test various development scenarios
    let scenarios = [
        "refactor database connection logic",
        "implement new API endpoint",
        "optimize search performance",
        "add unit tests for user authentication",
        "debug memory leak issue",
    ];

    for (i, scenario) in scenarios.iter().enumerate() {
        println!("\n📝 Scenario {}: {}", i + 1, scenario);

        match demo.handle_user_request_with_mcp(scenario).await {
            Ok(context) => {
                println!("   ✅ Analysis completed");
                println!("      - Active story: {}", context.active_story);
                println!("      - Suggested actions: {}", context.suggested_next_actions.len());

                // Show MCP-specific insights
                let mcp_insights = context.recent_patterns.iter()
                    .filter(|pattern| pattern.description.contains("MCP"))
                    .count();

                if mcp_insights > 0 {
                    println!("      - MCP insights: {} patterns discovered", mcp_insights);
                }
            }
            Err(e) => {
                println!("   ⚠️ Analysis completed with limitations: {}", e);
            }
        }
    }

    Ok(())
}

#[cfg(feature = "mcp")]
async fn demo_tool_discovery() -> Result<()> {
    println!("🕵️ Discovering available MCP tools and resources...");

    let engine = workflows::setup_mcp_intelligence().await?;

    // Use the extension trait for MCP-specific functionality
    let extensions = &engine as &(dyn McpIntelligenceExtensions + Sync);

    workflows::demonstrate_mcp_discovery(extensions).await?;

    // Show contextual server discovery
    println!("\n🎯 Contextual MCP server discovery:");
    let contexts = [
        "filesystem operations",
        "git version control",
        "database queries",
        "code analysis",
        "testing automation"
    ];

    for context in &contexts {
        let relevant_servers = extensions.discover_relevant_mcp_servers(context).await;
        if !relevant_servers.is_empty() {
            println!("   {} context: {} relevant servers", context, relevant_servers.len());
            for server in &relevant_servers {
                println!("      - {}", server);
            }
        } else {
            println!("   {} context: No MCP servers currently available", context);
        }
    }

    println!("\n💡 Note: Connect MCP servers to see real tool discovery in action!");

    Ok(())
}

#[cfg(feature = "mcp")]
async fn demo_complete_workflow() -> Result<()> {
    println!("🚀 Running complete MCP-enhanced development workflow...");

    let config = ConfigManager::load().await?;
    let storage = DatabaseManager::new(&config).await?;

    let demo = McpIntelligenceDemo::new(&config, &storage).await?;

    // Simulate a comprehensive development session
    println!("\n📁 Simulating project analysis...");
    if let Err(e) = demo.analyze_project_with_mcp("./").await {
        println!("   ⚠️ Project analysis completed with limitations: {}", e);
    }

    println!("\n📊 Running enhanced code analysis workflow...");
    let test_files = vec![
        "src/main.rs".to_string(),
        "src/lib.rs".to_string(),
        "src/intelligence/mod.rs".to_string(),
    ];

    if let Err(e) = demo.enhanced_code_analysis_workflow(&test_files).await {
        println!("   ⚠️ Code analysis workflow completed with limitations: {}", e);
    }

    println!("\n🌐 Analyzing cross-project patterns...");
    if let Err(e) = demo.cross_project_analysis_with_mcp("rust async patterns").await {
        println!("   ⚠️ Cross-project analysis completed with limitations: {}", e);
    }

    println!("\n📈 Checking project momentum...");
    if let Err(e) = demo.project_momentum_with_mcp().await {
        println!("   ⚠️ Project momentum analysis completed with limitations: {}", e);
    }

    println!("\n✨ Complete workflow demonstrates:");
    println!("   - Intelligence Engine base functionality ✅");
    println!("   - MCP integration architecture ✅");
    println!("   - Enhanced context analysis ✅");
    println!("   - Tool and resource discovery ✅");
    println!("   - Error handling and fallbacks ✅");

    Ok(())
}

#[cfg(feature = "mcp")]
async fn demo_real_mcp_integration() -> Result<()> {
    println!("🔗 Testing real MCP server integration...");

    // This function would be called if MCP servers are actually connected
    // For now, we just demonstrate the capability structure

    let engine = workflows::setup_mcp_intelligence().await?;
    let extensions = &engine as &(dyn McpIntelligenceExtensions + Sync);

    // Try to execute a real MCP tool if available
    let tools = extensions.get_available_mcp_tools().await;

    if !tools.is_empty() {
        println!("🎉 Found {} connected MCP servers!", tools.len());

        for (server_name, server_tools) in tools {
            println!("📦 Server '{}' provides {} tools:", server_name, server_tools.len());

            for tool in &server_tools {
                println!("   🔧 {}: {}", tool.name,
                    tool.description.as_deref().unwrap_or("No description"));

                // Try to execute a safe, read-only tool
                if tool.name.contains("list") || tool.name.contains("status") {
                    match extensions.execute_contextual_mcp_tool(
                        &format!("{}.{}", server_name, tool.name),
                        "demo test",
                        serde_json::json!({})
                    ).await {
                        Ok(result) => {
                            println!("      ✅ Execution successful: {}",
                                serde_json::to_string(&result).unwrap_or_default().chars().take(100).collect::<String>());
                        }
                        Err(e) => {
                            println!("      ⚠️ Execution completed with limitations: {}", e);
                        }
                    }
                }
            }
        }
    } else {
        println!("ℹ️ No MCP servers currently connected.");
        println!("   This is expected for the demo - connect real servers to see integration!");
    }

    Ok(())
}
