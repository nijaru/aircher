/// Test multi-step task execution workflows
use anyhow::Result;
use std::sync::Arc;

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
    println!("🔧 TESTING MULTI-STEP TASK EXECUTION");
    println!("====================================\n");

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

    if let Some(provider) = provider_manager.get_provider("ollama") {
        println!("🎯 Test 1: Multi-step file analysis workflow");
        println!("----------------------------------------\n");

        let request = "List all the Rust files in src/agent/tools/, then read the first 50 lines of git_tools.rs and tell me what git operations are supported";

        println!("Request: {}", request);
        println!("\n⏱️ Processing multi-step task...");

        match agent.process_message(request, provider, "gpt-oss").await {
            Ok((response, status_messages)) => {
                println!("\n📊 RESULT:");
                println!("Response length: {} characters", response.len());
                println!("Tool calls made: {} calls", status_messages.len());

                println!("\n📋 Tool Execution Sequence:");
                for (i, msg) in status_messages.iter().enumerate() {
                    println!("  Step {}: {}", i + 1, msg);
                }

                // Analyze multi-step execution
                let has_list_step = status_messages.iter().any(|m| m.contains("list_files"));
                let has_read_step = status_messages.iter().any(|m| m.contains("read_file"));
                let multiple_steps = status_messages.len() >= 2;
                let has_git_info = response.to_lowercase().contains("git") &&
                                    (response.contains("commit") || response.contains("branch") || response.contains("pr"));

                println!("\n🔍 MULTI-STEP ANALYSIS:");
                println!("  List files step: {}", has_list_step);
                println!("  Read file step: {}", has_read_step);
                println!("  Multiple tool calls: {}", multiple_steps);
                println!("  Git operations described: {}", has_git_info);

                if has_list_step && has_read_step && multiple_steps && has_git_info {
                    println!("\n✅ SUCCESS: Agent executed multi-step workflow correctly");
                } else {
                    println!("\n⚠️ PARTIAL: Some steps may be missing");
                }

                println!("\n📄 Agent Response (first 500 chars):");
                println!("---START---");
                println!("{}", response.chars().take(500).collect::<String>());
                if response.len() > 500 {
                    println!("... (truncated {} more characters)", response.len() - 500);
                }
                println!("---END---");
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }

        println!("\n\n🎯 Test 2: Code analysis and modification workflow");
        println!("----------------------------------------------\n");

        let request2 = "Find all TODO comments in the codebase, then create a summary file TODO_SUMMARY.md listing them";

        println!("Request: {}", request2);
        println!("\n⏱️ Processing multi-step task...");

        match agent.process_message(request2, provider, "gpt-oss").await {
            Ok((response, status_messages)) => {
                println!("\n📊 RESULT:");
                println!("Response length: {} characters", response.len());
                println!("Tool calls made: {} calls", status_messages.len());

                println!("\n📋 Tool Execution Sequence:");
                for (i, msg) in status_messages.iter().enumerate() {
                    println!("  Step {}: {}", i + 1, msg);
                }

                let has_search = status_messages.iter().any(|m| m.contains("search_code"));
                let has_write = status_messages.iter().any(|m| m.contains("write_file"));
                let workflow_complete = has_search && has_write;

                println!("\n🔍 WORKFLOW ANALYSIS:");
                println!("  Search for TODOs: {}", has_search);
                println!("  Write summary file: {}", has_write);
                println!("  Workflow complete: {}", workflow_complete);

                if workflow_complete {
                    println!("\n✅ SUCCESS: Complex workflow executed successfully");

                    // Clean up the test file
                    if std::path::Path::new("TODO_SUMMARY.md").exists() {
                        std::fs::remove_file("TODO_SUMMARY.md").ok();
                        println!("🧹 Cleaned up test file");
                    }
                } else {
                    println!("\n❌ FAILURE: Workflow incomplete");
                }
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }

        println!("\n\n🎯 Test 3: Error recovery workflow");
        println!("----------------------------------\n");

        let request3 = "Try to read a non-existent file called DOES_NOT_EXIST.txt, then explain what went wrong";

        println!("Request: {}", request3);
        println!("\n⏱️ Processing error recovery task...");

        match agent.process_message(request3, provider, "gpt-oss").await {
            Ok((response, status_messages)) => {
                println!("\n📊 RESULT:");
                println!("Response length: {} characters", response.len());
                println!("Tool calls made: {} calls", status_messages.len());

                let attempted_read = status_messages.iter().any(|m| m.contains("read_file"));
                let explained_error = response.to_lowercase().contains("not exist") ||
                                      response.to_lowercase().contains("not found") ||
                                      response.to_lowercase().contains("error");
                let recovered = response.len() > 50 && explained_error;

                println!("\n🔍 ERROR RECOVERY ANALYSIS:");
                println!("  Attempted file read: {}", attempted_read);
                println!("  Explained error: {}", explained_error);
                println!("  Recovered gracefully: {}", recovered);

                if attempted_read && recovered {
                    println!("\n✅ SUCCESS: Agent handled error gracefully");
                } else {
                    println!("\n❌ FAILURE: Error recovery needs improvement");
                }

                println!("\n📄 Error explanation:");
                println!("{}", response);
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }
    } else {
        println!("❌ No Ollama provider available");
    }

    Ok(())
}