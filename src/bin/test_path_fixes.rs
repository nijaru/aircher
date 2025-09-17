/// Test path handling fixes
/// This test verifies that our path corrections work for common problematic paths

use anyhow::Result;
use std::path::Path;

use aircher::agent::tools::file_ops::{ReadFileTool, WriteFileTool};
use aircher::agent::tools::AgentTool;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔧 PATH HANDLING FIXES TEST");
    println!("============================\n");

    // Test 1: Create a file in /tmp with problematic path
    println!("1. Testing write to 'tmp/test_file.txt' (missing leading slash)...");

    let write_tool = WriteFileTool::new();
    let write_params = json!({
        "path": "tmp/test_file.txt",
        "content": "Hello from path fix test"
    });

    match write_tool.execute(write_params).await {
        Ok(output) => {
            if output.success {
                println!("   ✅ Write succeeded: {}", output.result);
            } else {
                println!("   ❌ Write failed: {:?}", output.error);
            }
        }
        Err(e) => {
            println!("   ❌ Write error: {}", e);
        }
    }

    // Test 2: Try to read the file back
    println!("\n2. Testing read from 'tmp/test_file.txt'...");

    let read_tool = ReadFileTool::new();
    let read_params = json!({
        "path": "tmp/test_file.txt"
    });

    match read_tool.execute(read_params).await {
        Ok(output) => {
            if output.success {
                println!("   ✅ Read succeeded");
                if let Some(content) = output.result.get("content") {
                    println!("   📝 Content: {}", content.as_str().unwrap_or(""));
                }
            } else {
                println!("   ❌ Read failed: {:?}", output.error);
            }
        }
        Err(e) => {
            println!("   ❌ Read error: {}", e);
        }
    }

    // Test 3: Test with already absolute path
    println!("\n3. Testing with absolute path '/tmp/test_absolute.txt'...");

    let write_params_abs = json!({
        "path": "/tmp/test_absolute.txt",
        "content": "Hello from absolute path test"
    });

    match write_tool.execute(write_params_abs).await {
        Ok(output) => {
            if output.success {
                println!("   ✅ Absolute write succeeded");
            } else {
                println!("   ❌ Absolute write failed: {:?}", output.error);
            }
        }
        Err(e) => {
            println!("   ❌ Absolute write error: {}", e);
        }
    }

    // Test 4: Verify files were created in correct locations
    println!("\n4. Verifying file locations...");

    if Path::new("/tmp/test_file.txt").exists() {
        println!("   ✅ /tmp/test_file.txt exists (path correction worked)");
    } else {
        println!("   ❌ /tmp/test_file.txt not found");
    }

    if Path::new("/tmp/test_absolute.txt").exists() {
        println!("   ✅ /tmp/test_absolute.txt exists");
    } else {
        println!("   ❌ /tmp/test_absolute.txt not found");
    }

    // Cleanup
    let _ = std::fs::remove_file("/tmp/test_file.txt");
    let _ = std::fs::remove_file("/tmp/test_absolute.txt");

    println!("\n🎯 PATH HANDLING TEST COMPLETE");
    Ok(())
}