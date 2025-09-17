/// Validate SOTA claims through real-world coding workflows
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
};
use aircher::providers::ProviderManager;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔥 VALIDATING SOTA CLAIMS - REAL CODING WORKFLOWS");
    println!("==================================================\n");

    // Setup - measure initialization time
    let setup_start = Instant::now();
    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let db_manager = DatabaseManager::new(&config).await?;
    let intelligence = IntelligenceEngine::new(&config, &db_manager).await?;
    let provider_manager = Arc::new(ProviderManager::new(&config, auth_manager.clone()).await?);

    let project_context = ProjectContext {
        root_path: std::env::current_dir()?,
        language: ProgrammingLanguage::Rust,
        framework: None,
        recent_changes: Vec::new(),
    };

    let agent = Agent::new(intelligence, auth_manager, project_context).await?;
    println!("✅ Setup: {:?}\n", setup_start.elapsed());

    // Test 1: Simple query (should be fast)
    println!("📊 Test 1: Simple Query Performance");
    test_simple_query(&agent, &provider_manager).await?;

    // Test 2: Complex coding task (multi-turn)
    println!("\n📊 Test 2: Complex Coding Workflow");
    test_complex_coding_workflow(&agent, &provider_manager).await?;

    // Test 3: Tool execution chain
    println!("\n📊 Test 3: Tool Execution Chain");
    test_tool_execution_chain(&agent, &provider_manager).await?;

    // Test 4: Error handling and recovery
    println!("\n📊 Test 4: Error Handling & Recovery");
    test_error_handling(&agent, &provider_manager).await?;

    // Test 5: Performance under load
    println!("\n📊 Test 5: Performance Under Load");
    test_performance_load(&agent, &provider_manager).await?;

    println!("\n🎯 VALIDATION COMPLETE");
    println!("Check results above to validate SOTA readiness");

    Ok(())
}

async fn test_simple_query(agent: &Agent, provider_manager: &ProviderManager) -> Result<()> {
    if let Some(provider) = provider_manager.get_provider("ollama") {
        let start = Instant::now();

        let (response, _status) = agent.process_message(
            "What is 2+2?",
            provider,
            "gpt-oss"
        ).await?;

        let duration = start.elapsed();
        println!("   Simple query: {:?}", duration);
        println!("   Response length: {} chars", response.len());

        // Validate: Should be under 2 seconds for simple queries
        if duration.as_secs() > 2 {
            println!("   ❌ FAILED: Simple query too slow (>{:?})", duration);
        } else {
            println!("   ✅ PASSED: Simple query fast enough");
        }
    } else {
        println!("   ⚠️  SKIPPED: No Ollama provider available");
    }

    Ok(())
}

async fn test_complex_coding_workflow(agent: &Agent, provider_manager: &ProviderManager) -> Result<()> {
    if let Some(provider) = provider_manager.get_provider("ollama") {
        let start = Instant::now();

        // Complex multi-step coding task
        let (response, _status) = agent.process_message(
            "Create a new Rust function called calculate_fibonacci that takes a number and returns the fibonacci sequence up to that number. Include error handling and tests.",
            provider,
            "gpt-oss"
        ).await?;

        let duration = start.elapsed();
        println!("   Complex workflow: {:?}", duration);
        println!("   Response length: {} chars", response.len());

        // Validate: Should complete successfully
        let has_function = response.to_lowercase().contains("fibonacci");
        let has_error_handling = response.to_lowercase().contains("error") || response.to_lowercase().contains("result");
        let has_tests = response.to_lowercase().contains("test") || response.to_lowercase().contains("#[test]");

        println!("   Function mentioned: {}", has_function);
        println!("   Error handling: {}", has_error_handling);
        println!("   Tests included: {}", has_tests);

        if has_function && (has_error_handling || has_tests) {
            println!("   ✅ PASSED: Complex workflow completed");
        } else {
            println!("   ❌ FAILED: Complex workflow incomplete");
        }
    } else {
        println!("   ⚠️  SKIPPED: No Ollama provider available");
    }

    Ok(())
}

async fn test_tool_execution_chain(agent: &Agent, provider_manager: &ProviderManager) -> Result<()> {
    if let Some(provider) = provider_manager.get_provider("ollama") {
        let start = Instant::now();

        // Request that should trigger tool usage
        let (response, _status) = agent.process_message(
            "Read the Cargo.toml file and tell me what dependencies this project has",
            provider,
            "gpt-oss"
        ).await?;

        let duration = start.elapsed();
        println!("   Tool execution: {:?}", duration);
        println!("   Response length: {} chars", response.len());

        // Validate: Should mention file contents or dependencies
        let mentions_cargo = response.to_lowercase().contains("cargo") || response.to_lowercase().contains("dependencies");
        let mentions_content = response.to_lowercase().contains("serde") || response.to_lowercase().contains("tokio") || response.to_lowercase().contains("anyhow");

        println!("   Mentions Cargo/deps: {}", mentions_cargo);
        println!("   Shows actual content: {}", mentions_content);

        if mentions_cargo || mentions_content {
            println!("   ✅ PASSED: Tool execution working");
        } else {
            println!("   ❌ FAILED: No evidence of tool execution");
        }
    } else {
        println!("   ⚠️  SKIPPED: No Ollama provider available");
    }

    Ok(())
}

async fn test_error_handling(agent: &Agent, provider_manager: &ProviderManager) -> Result<()> {
    if let Some(provider) = provider_manager.get_provider("ollama") {
        let start = Instant::now();

        // Request that might cause errors
        let result = agent.process_message(
            "Read the file /nonexistent/file.txt and analyze its contents",
            provider,
            "gpt-oss"
        ).await;

        let duration = start.elapsed();
        println!("   Error handling: {:?}", duration);

        match result {
            Ok((response, _status)) => {
                println!("   Response length: {} chars", response.len());
                let handles_error = response.to_lowercase().contains("not found") ||
                                   response.to_lowercase().contains("doesn't exist") ||
                                   response.to_lowercase().contains("cannot") ||
                                   response.to_lowercase().contains("error");

                if handles_error {
                    println!("   ✅ PASSED: Error handled gracefully");
                } else {
                    println!("   ❌ FAILED: Error not handled properly");
                }
            }
            Err(e) => {
                println!("   Error: {}", e);
                println!("   ❌ FAILED: System error (should handle gracefully)");
            }
        }
    } else {
        println!("   ⚠️  SKIPPED: No Ollama provider available");
    }

    Ok(())
}

async fn test_performance_load(agent: &Agent, provider_manager: &ProviderManager) -> Result<()> {
    if let Some(provider) = provider_manager.get_provider("ollama") {
        println!("   Running 5 sequential queries to test load...");

        let start = Instant::now();
        let mut successful = 0;

        for i in 1..=5 {
            let query_start = Instant::now();

            match agent.process_message(
                &format!("What is {}+{}?", i, i),
                provider,
                "gpt-oss"
            ).await {
                Ok(_) => {
                    successful += 1;
                    println!("   Query {}: {:?}", i, query_start.elapsed());
                }
                Err(e) => println!("   Query {} failed: {}", i, e),
            }
        }

        let duration = start.elapsed();
        println!("   Total load test: {:?}", duration);
        println!("   Successful queries: {}/5", successful);

        if successful >= 4 && duration.as_secs() < 15 {
            println!("   ✅ PASSED: Performance under load acceptable");
        } else {
            println!("   ❌ FAILED: Performance under load insufficient");
        }
    } else {
        println!("   ⚠️  SKIPPED: No Ollama provider available");
    }

    Ok(())
}