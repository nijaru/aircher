/// Test advanced Git workflow with SmartCommitTool
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
    println!("🔧 TESTING ADVANCED GIT WORKFLOW");
    println!("================================\n");

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
        println!("Test: Smart Git commit workflow");
        println!("-------------------------------\n");

        // Test 1: Check git status and create intelligent commit
        let request = "Check what files I have staged for commit, then create a smart commit with an intelligent message";
        println!("Request: {}\n", request);
        println!("Expected: smart_commit tool usage with auto-generated message\n");

        let start = std::time::Instant::now();

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(45),
            agent.process_message(request, provider, "gpt-oss")
        ).await;

        match result {
            Ok(Ok((response, status_messages))) => {
                let duration = start.elapsed();
                println!("✅ Completed in {:.1}s", duration.as_secs_f64());
                println!("Tool calls made: {}", status_messages.len());

                println!("\n📋 Tool execution sequence:");
                for (i, msg) in status_messages.iter().enumerate() {
                    println!("  {}: {}", i + 1, msg);
                }

                // Check if git tools were used
                let used_git_status = status_messages.iter().any(|m| m.contains("run_command") && m.contains("git"));
                let used_smart_commit = status_messages.iter().any(|m| m.contains("smart_commit"));

                println!("\n🔍 Git workflow analysis:");
                println!("  Git status checked: {}", used_git_status);
                println!("  Smart commit used: {}", used_smart_commit);

                if used_smart_commit {
                    println!("\n✅ SUCCESS: Advanced Git workflow executed!");
                    println!("The agent used our sophisticated SmartCommitTool!");
                } else if used_git_status {
                    println!("\n⚠️ PARTIAL: Basic git command used, but not advanced tools");
                } else {
                    println!("\n❌ NO GIT: No git operations detected");
                }

                println!("\n📄 Response (first 800 chars):");
                println!("{}", response.chars().take(800).collect::<String>());
                if response.len() > 800 {
                    println!("... ({} more chars)", response.len() - 800);
                }
            }
            Ok(Err(e)) => {
                println!("❌ Error: {}", e);
            }
            Err(_) => {
                let duration = start.elapsed();
                println!("❌ TIMEOUT after {:.1}s", duration.as_secs_f64());
            }
        }

        println!("\n\nTest 2: Multi-step branch creation and commit");
        println!("----------------------------------------------\n");

        let request2 = "Create a new branch called 'parser-enhancements' and then commit my staged changes with a smart message";
        println!("Request: {}\n", request2);
        println!("Expected: branch_management + smart_commit tools\n");

        let start = std::time::Instant::now();

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            agent.process_message(request2, provider, "gpt-oss")
        ).await;

        match result {
            Ok(Ok((response, status_messages))) => {
                let duration = start.elapsed();
                println!("✅ Completed in {:.1}s", duration.as_secs_f64());
                println!("Tool calls made: {}", status_messages.len());

                let used_branch_mgmt = status_messages.iter().any(|m| m.contains("branch_management"));
                let used_smart_commit = status_messages.iter().any(|m| m.contains("smart_commit"));

                println!("\n🔍 Advanced workflow analysis:");
                println!("  Branch management: {}", used_branch_mgmt);
                println!("  Smart commit: {}", used_smart_commit);

                if used_branch_mgmt && used_smart_commit {
                    println!("\n🎉 BREAKTHROUGH: Multi-tool Git workflow working!");
                    println!("Both BranchManagementTool AND SmartCommitTool executed!");
                } else {
                    println!("\n⚠️ Partial workflow completion");
                }

                println!("\nFirst 500 chars of response:");
                println!("{}", response.chars().take(500).collect::<String>());
            }
            Ok(Err(e)) => {
                println!("❌ Error: {}", e);
            }
            Err(_) => {
                println!("❌ TIMEOUT - Multi-step workflow took too long");
            }
        }
    } else {
        println!("❌ No Ollama provider available");
    }

    Ok(())
}