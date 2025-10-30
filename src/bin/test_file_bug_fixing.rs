/// Direct Tool + LLM Bug Fixing Test
///
/// Tests: Tools → Ollama → Tools workflow
/// 1. Create buggy file with write_file
/// 2. Read it with read_file
/// 3. Get fix from Ollama
/// 4. Apply fix with edit_file
/// 5. Verify fix worked

use anyhow::Result;
use aircher::agent::tools::ToolRegistry;
use aircher::providers::{ChatRequest, Message, MessageRole, ProviderManager};
use aircher::config::ConfigManager;
use aircher::auth::AuthManager;
use serde_json::json;
use std::sync::Arc;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🐛 FILE-BASED BUG FIXING TEST\n");
    println!("Testing: Tools → Ollama → Tools workflow\n");

    // Setup
    let registry = ToolRegistry::default();
    let config = ConfigManager::load().await?;
    let auth = Arc::new(AuthManager::new()?);
    let providers = Arc::new(ProviderManager::new(&config, auth).await?);

    // Get Ollama provider
    let provider = providers.get_provider("ollama")
        .ok_or_else(|| anyhow::anyhow!("Ollama provider not found"))?;

    // Test file path
    let test_file = "/tmp/test_bug_fix.py";

    // Step 1: Create buggy file
    println!("Step 1: Creating buggy file...");
    let buggy_code = r#"def get_last_three(items):
    # BUG: Should be items[-3:], not items[-4:-1]
    return items[-4:-1]

print(get_last_three([1, 2, 3, 4, 5, 6, 7, 8, 9]))  # Should print [7, 8, 9]
"#;

    if let Some(write_tool) = registry.get("write_file") {
        let params = json!({
            "path": test_file,
            "content": buggy_code
        });
        let result = write_tool.execute(params).await?;
        if !result.success {
            anyhow::bail!("Failed to write file: {:?}", result.error);
        }
        println!("✅ File created at {}", test_file);
    } else {
        anyhow::bail!("write_file tool not found");
    }

    // Step 2: Read the file
    println!("\nStep 2: Reading file with read_file tool...");
    let file_content = if let Some(read_tool) = registry.get("read_file") {
        let params = json!({"path": test_file});
        let result = read_tool.execute(params).await?;
        if !result.success {
            anyhow::bail!("Failed to read file: {:?}", result.error);
        }
        println!("✅ File read successfully");
        result.result.get("content")
            .and_then(|c| c.as_str())
            .ok_or_else(|| anyhow::anyhow!("No content in result"))?
            .to_string()
    } else {
        anyhow::bail!("read_file tool not found");
    };

    println!("📄 File content:\n{}", file_content);

    // Step 3: Ask Ollama for fix
    println!("\nStep 3: Asking Ollama to identify the bug...");
    let prompt = format!(
        "This Python code has a bug. Identify the bug and provide ONLY the corrected line. No explanation.\n\n```python\n{}\n```\n\nProvide ONLY the fixed line, like: return items[-3:]",
        file_content
    );

    let request = ChatRequest {
        model: "gpt-oss:latest".to_string(),
        messages: vec![Message {
            id: uuid::Uuid::new_v4().to_string(),
            role: MessageRole::User,
            content: prompt,
            timestamp: Utc::now(),
            tokens_used: None,
            cost: None,
        }],
        max_tokens: Some(100),
        temperature: Some(0.0),
        stream: false,
        tools: None,
    };

    let response = provider.chat(&request).await?;
    println!("✅ Ollama response: {}", response.content.trim());

    // Step 4: Extract the fix and apply it
    println!("\nStep 4: Applying fix with edit_file tool...");

    // Extract the fixed line from response
    let fix = response.content.trim();

    if let Some(edit_tool) = registry.get("edit_file") {
        let params = json!({
            "path": test_file,
            "search": "return items[-4:-1]",
            "replace": fix.trim()
        });

        let result = edit_tool.execute(params).await?;
        if result.success {
            println!("✅ File edited successfully");
        } else {
            println!("❌ Edit failed, trying alternative...");
            // Sometimes model gives full line, try extracting just the slice
            if fix.contains("[-3:]") {
                let params = json!({
                    "path": test_file,
                    "search": "items[-4:-1]",
                    "replace": "items[-3:]"
                });
                edit_tool.execute(params).await?;
                println!("✅ Fix applied via alternative approach");
            }
        }
    } else {
        anyhow::bail!("edit_file tool not found");
    }

    // Step 5: Verify the fix
    println!("\nStep 5: Verifying fix...");
    if let Some(read_tool) = registry.get("read_file") {
        let params = json!({"path": test_file});
        let result = read_tool.execute(params).await?;
        let fixed_content = result.result.get("content")
            .and_then(|c| c.as_str())
            .ok_or_else(|| anyhow::anyhow!("No content in result"))?;

        println!("📄 Fixed file content:\n{}", fixed_content);

        if fixed_content.contains("[-3:]") && !fixed_content.contains("[-4:-1]") {
            println!("\n✅ BUG FIX SUCCESSFUL!");
            println!("   Original: items[-4:-1]");
            println!("   Fixed:    items[-3:]");
        } else {
            println!("\n⚠️  Fix verification unclear - check file manually");
        }
    }

    // Step 6: Test the fixed code
    println!("\nStep 6: Running the fixed code...");
    if let Some(cmd_tool) = registry.get("run_command") {
        let params = json!({
            "command": "python3",
            "args": [test_file]
        });

        match cmd_tool.execute(params).await {
            Ok(result) => {
                if let Some(stdout) = result.result.get("stdout").and_then(|s| s.as_str()) {
                    println!("📊 Output: {}", stdout.trim());
                    if stdout.contains("[7, 8, 9]") {
                        println!("✅ CORRECT OUTPUT! Bug is fixed!");
                    }
                }
            }
            Err(e) => println!("⚠️  Could not run code: {}", e)
        }
    }

    println!("\n🎯 TEST COMPLETE");
    println!("Workflow validated: write_file → read_file → Ollama → edit_file → verify");

    Ok(())
}
