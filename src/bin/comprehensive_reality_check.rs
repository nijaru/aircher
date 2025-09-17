/// Comprehensive reality check - test what actually works vs claims
use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};

use aircher::auth::AuthManager;
use aircher::client::local::LocalClient;
use aircher::config::ConfigManager;
use aircher::providers::ProviderManager;

#[derive(Debug)]
struct TestResult {
    name: String,
    passed: bool,
    details: String,
    duration: Duration,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 COMPREHENSIVE REALITY CHECK");
    println!("===============================\n");

    let mut results = Vec::new();

    // Setup
    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let provider_manager = Arc::new(ProviderManager::new(&config, auth_manager.clone()).await?);
    let mut client = LocalClient::new(&config, auth_manager, provider_manager).await?;
    client.init_session().await?;

    // Test 1: Multiple tool types
    results.push(test_file_operations(&client).await);
    results.push(test_search_tools(&client).await);
    results.push(test_system_tools(&client).await);

    // Test 2: Error handling
    results.push(test_error_recovery(&client).await);

    // Test 3: Complex workflows
    results.push(test_multi_step_workflow(&client).await);

    // Test 4: Edge cases
    results.push(test_edge_cases(&client).await);

    // Test 5: Performance under basic load
    results.push(test_basic_performance(&client).await);

    // Summary
    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();

    println!("\n📊 REALITY CHECK RESULTS");
    println!("=========================");

    for result in &results {
        let status = if result.passed { "✅" } else { "❌" };
        println!("{} {} ({:?})", status, result.name, result.duration);
        if !result.passed {
            println!("   💥 {}", result.details);
        }
    }

    println!("\n🎯 HONEST ASSESSMENT: {}/{} tests passed ({:.1}%)",
        passed, total, (passed as f64 / total as f64) * 100.0);

    if passed == total {
        println!("🎉 System is actually functional as claimed!");
    } else {
        println!("⚠️  System has gaps - not production ready");
    }

    Ok(())
}

async fn test_file_operations(client: &LocalClient) -> TestResult {
    let start = Instant::now();
    let name = "File Operations (read/write/edit)".to_string();

    // Test write
    match tokio::time::timeout(
        Duration::from_secs(20),
        client.send_message("Create /tmp/test.txt with 'original content'")
    ).await {
        Ok(Ok(_)) => {
            if !std::path::Path::new("/tmp/test.txt").exists() {
                return TestResult { name, passed: false, details: "Write failed - file not created".to_string(), duration: start.elapsed() };
            }

            // Test edit
            match tokio::time::timeout(
                Duration::from_secs(20),
                client.send_message("Change the content of /tmp/test.txt to 'modified content'")
            ).await {
                Ok(Ok(_)) => {
                    if let Ok(content) = std::fs::read_to_string("/tmp/test.txt") {
                        let _ = std::fs::remove_file("/tmp/test.txt");
                        if content.trim().contains("modified") {
                            TestResult { name, passed: true, details: "All file operations work".to_string(), duration: start.elapsed() }
                        } else {
                            TestResult { name, passed: false, details: format!("Edit failed - content: '{}'", content.trim()), duration: start.elapsed() }
                        }
                    } else {
                        TestResult { name, passed: false, details: "Edit failed - can't read file".to_string(), duration: start.elapsed() }
                    }
                }
                _ => TestResult { name, passed: false, details: "Edit request failed/timed out".to_string(), duration: start.elapsed() }
            }
        }
        _ => TestResult { name, passed: false, details: "Write request failed/timed out".to_string(), duration: start.elapsed() }
    }
}

async fn test_search_tools(client: &LocalClient) -> TestResult {
    let start = Instant::now();
    let name = "Search Tools (code search)".to_string();

    match tokio::time::timeout(
        Duration::from_secs(15),
        client.send_message("Search for 'Agent' in the codebase")
    ).await {
        Ok(Ok(response)) => {
            if response.content.len() > 50 && (response.content.contains("Agent") || response.content.contains("search")) {
                TestResult { name, passed: true, details: "Search returned results".to_string(), duration: start.elapsed() }
            } else {
                TestResult { name, passed: false, details: format!("Search returned minimal results: {}", response.content.chars().take(100).collect::<String>()), duration: start.elapsed() }
            }
        }
        _ => TestResult { name, passed: false, details: "Search request failed/timed out".to_string(), duration: start.elapsed() }
    }
}

async fn test_system_tools(client: &LocalClient) -> TestResult {
    let start = Instant::now();
    let name = "System Tools (run command)".to_string();

    match tokio::time::timeout(
        Duration::from_secs(15),
        client.send_message("Run the command 'echo hello system test'")
    ).await {
        Ok(Ok(response)) => {
            if response.content.contains("hello system test") || response.content.contains("echo") {
                TestResult { name, passed: true, details: "Command execution works".to_string(), duration: start.elapsed() }
            } else {
                TestResult { name, passed: false, details: format!("Command output unexpected: {}", response.content.chars().take(100).collect::<String>()), duration: start.elapsed() }
            }
        }
        _ => TestResult { name, passed: false, details: "Command request failed/timed out".to_string(), duration: start.elapsed() }
    }
}

async fn test_error_recovery(client: &LocalClient) -> TestResult {
    let start = Instant::now();
    let name = "Error Recovery (invalid operations)".to_string();

    // Try to read non-existent file
    match tokio::time::timeout(
        Duration::from_secs(15),
        client.send_message("Read the file /tmp/definitely_does_not_exist.txt")
    ).await {
        Ok(Ok(response)) => {
            if response.content.contains("not found") || response.content.contains("error") || response.content.contains("failed") {
                TestResult { name, passed: true, details: "Error handling works".to_string(), duration: start.elapsed() }
            } else {
                TestResult { name, passed: false, details: format!("No error reported: {}", response.content.chars().take(100).collect::<String>()), duration: start.elapsed() }
            }
        }
        _ => TestResult { name, passed: false, details: "Error test failed/timed out".to_string(), duration: start.elapsed() }
    }
}

async fn test_multi_step_workflow(client: &LocalClient) -> TestResult {
    let start = Instant::now();
    let name = "Multi-Step Workflow".to_string();

    match tokio::time::timeout(
        Duration::from_secs(30),
        client.send_message("Create /tmp/workflow.txt with 'step1', then list the directory contents")
    ).await {
        Ok(Ok(response)) => {
            let file_exists = std::path::Path::new("/tmp/workflow.txt").exists();
            let has_listing = response.content.contains("workflow.txt") || response.content.contains("/tmp");

            if file_exists && has_listing {
                let _ = std::fs::remove_file("/tmp/workflow.txt");
                TestResult { name, passed: true, details: "Multi-step workflow completed".to_string(), duration: start.elapsed() }
            } else {
                TestResult { name, passed: false, details: format!("Workflow incomplete - file exists: {}, has listing: {}", file_exists, has_listing), duration: start.elapsed() }
            }
        }
        _ => TestResult { name, passed: false, details: "Multi-step request failed/timed out".to_string(), duration: start.elapsed() }
    }
}

async fn test_edge_cases(client: &LocalClient) -> TestResult {
    let start = Instant::now();
    let name = "Edge Cases".to_string();

    // Test empty/minimal request
    match tokio::time::timeout(
        Duration::from_secs(10),
        client.send_message("help")
    ).await {
        Ok(Ok(response)) => {
            if response.content.len() > 10 {
                TestResult { name, passed: true, details: "Handles edge cases".to_string(), duration: start.elapsed() }
            } else {
                TestResult { name, passed: false, details: "Minimal response to simple request".to_string(), duration: start.elapsed() }
            }
        }
        _ => TestResult { name, passed: false, details: "Edge case request failed".to_string(), duration: start.elapsed() }
    }
}

async fn test_basic_performance(client: &LocalClient) -> TestResult {
    let start = Instant::now();
    let name = "Basic Performance (3 rapid requests)".to_string();

    let mut all_passed = true;
    let mut details = Vec::new();

    for i in 1..=3 {
        let request_start = Instant::now();
        match tokio::time::timeout(
            Duration::from_secs(10),
            client.send_message(&format!("Echo 'test {}'", i))
        ).await {
            Ok(Ok(response)) => {
                let duration = request_start.elapsed();
                if duration > Duration::from_secs(8) {
                    all_passed = false;
                    details.push(format!("Request {} too slow: {:?}", i, duration));
                }
                if !response.content.contains(&i.to_string()) {
                    all_passed = false;
                    details.push(format!("Request {} wrong response", i));
                }
            }
            _ => {
                all_passed = false;
                details.push(format!("Request {} failed", i));
            }
        }
    }

    TestResult {
        name,
        passed: all_passed,
        details: if all_passed { "Performance acceptable".to_string() } else { details.join("; ") },
        duration: start.elapsed()
    }
}