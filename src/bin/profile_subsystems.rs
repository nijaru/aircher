/// Profile each subsystem individually to find real performance bottlenecks
use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;

use aircher::auth::AuthManager;
use aircher::config::ConfigManager;
use aircher::storage::DatabaseManager;
use aircher::intelligence::IntelligenceEngine;
use aircher::agent::{
    core::Agent,
    conversation::{ProjectContext, ProgrammingLanguage},
    reasoning::AgentReasoning,
    dynamic_context::DynamicContextManager,
    tools::ToolRegistry,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔬 SUBSYSTEM PERFORMANCE PROFILER");
    println!("==================================\n");

    // Setup
    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let db_manager = DatabaseManager::new(&config).await?;

    // Test 1: IntelligenceEngine operations
    println!("📊 Test 1: IntelligenceEngine Operations");

    let intel_create_start = Instant::now();
    let intelligence = Arc::new(IntelligenceEngine::new(&config, &db_manager).await?);
    println!("   Creation: {:?}", intel_create_start.elapsed());

    let embedding_start = Instant::now();
    let _embedding = intelligence.get_embedding("test query").await?;
    println!("   get_embedding: {:?}", embedding_start.elapsed());

    let suggestions_start = Instant::now();
    let _suggestions = intelligence.get_suggestions("test query", None).await?;
    println!("   get_suggestions: {:?}", suggestions_start.elapsed());

    let predict_start = Instant::now();
    let _files = intelligence.predict_file_changes("src/main.rs").await?;
    println!("   predict_file_changes: {:?}", predict_start.elapsed());

    // Test 2: AgentReasoning operations
    println!("\n📊 Test 2: AgentReasoning Operations");

    let reasoning_create_start = Instant::now();
    let tools = Arc::new(ToolRegistry::new());
    let reasoning = AgentReasoning::new(intelligence.clone(), tools);
    println!("   Creation: {:?}", reasoning_create_start.elapsed());

    let process_start = Instant::now();
    let _result = reasoning.process_request("Create a test file").await?;
    println!("   process_request: {:?}", process_start.elapsed());

    // Test 3: DynamicContextManager operations
    println!("\n📊 Test 3: DynamicContextManager Operations");

    let context_create_start = Instant::now();
    let context_manager = DynamicContextManager::new(
        intelligence.clone(),
        None
    );
    println!("   Creation: {:?}", context_create_start.elapsed());

    let update_start = Instant::now();
    let _update = context_manager.update_context("test message").await?;
    println!("   update_context: {:?}", update_start.elapsed());

    let get_context_start = Instant::now();
    let _context = context_manager.get_relevant_context(5).await?;
    println!("   get_relevant_context: {:?}", get_context_start.elapsed());

    // Test 4: Multiple calls to see caching behavior
    println!("\n📊 Test 4: Caching Behavior");

    for i in 1..=3 {
        let start = Instant::now();
        let _embedding = intelligence.get_embedding(&format!("test query {}", i)).await?;
        println!("   Embedding call {}: {:?}", i, start.elapsed());
    }

    for i in 1..=3 {
        let start = Instant::now();
        let _suggestions = intelligence.get_suggestions(&format!("test query {}", i), None).await?;
        println!("   Suggestions call {}: {:?}", i, start.elapsed());
    }

    // Test 5: DatabaseManager operations
    println!("\n📊 Test 5: DatabaseManager Operations");

    for i in 1..=5 {
        let start = Instant::now();
        let _db = DatabaseManager::new(&config).await?;
        println!("   DB creation {}: {:?}", i, start.elapsed());
    }

    // Test 6: Agent creation timing
    println!("\n📊 Test 6: Agent Creation Timing");

    let project_context = ProjectContext {
        root_path: std::env::current_dir()?,
        language: ProgrammingLanguage::Rust,
        framework: None,
        recent_changes: Vec::new(),
    };

    let agent_start = Instant::now();
    let agent_intelligence = IntelligenceEngine::new(&config, &db_manager).await?;
    let _agent = Agent::new(agent_intelligence, auth_manager, project_context).await?;
    println!("   Agent creation: {:?}", agent_start.elapsed());

    println!("\n🎯 PROFILING COMPLETE");
    println!("\nNext: Identify bottlenecks and implement proper fixes:");
    println!("- If DB operations are slow: Add connection pooling");
    println!("- If file I/O is slow: Add caching");
    println!("- If computation is slow: Add memoization");
    println!("- If network is slow: Add async batching");

    Ok(())
}