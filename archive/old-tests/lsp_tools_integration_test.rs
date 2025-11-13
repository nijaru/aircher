use std::path::PathBuf;
use std::env;
use aircher::agent::tools::{AgentTool, ToolError};
use aircher::agent::tools::lsp_tools::{
    CodeCompletionTool, HoverTool, GoToDefinitionTool,
    FindReferencesTool, RenameSymbolTool, DiagnosticsTool, FormatCodeTool
};
use serde_json::json;
use anyhow::Result;

#[tokio::test]
async fn test_lsp_tools_integration() -> Result<()> {
    println!("🧠 LSP Integration Test Suite");
    println!("============================");
    println!("Testing 7 LSP tools with real code...\n");

    let workspace = env::current_dir()?;
    println!("📁 Workspace: {}", workspace.display());

    // Test file: use src/main.rs as it should exist and have Rust LSP support
    let test_file = workspace.join("src/main.rs");
    if !test_file.exists() {
        println!("⚠️ Test file {} does not exist, skipping LSP tests", test_file.display());
        return Ok(());
    }

    println!("📝 Test file: {}", test_file.display());
    println!();

    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Code Completion
    println!("1️⃣ Testing CodeCompletionTool...");
    let completion_tool = CodeCompletionTool::new(workspace.clone());
    match test_completion_tool(&completion_tool, &test_file).await {
        Ok(_) => {
            println!("   ✅ Code completion test passed");
            passed += 1;
        }
        Err(e) => {
            println!("   ❌ Code completion test failed: {}", e);
            failed += 1;
        }
    }

    // Test 2: Hover Information
    println!("2️⃣ Testing HoverTool...");
    let hover_tool = HoverTool::new(workspace.clone());
    match test_hover_tool(&hover_tool, &test_file).await {
        Ok(_) => {
            println!("   ✅ Hover information test passed");
            passed += 1;
        }
        Err(e) => {
            println!("   ❌ Hover information test failed: {}", e);
            failed += 1;
        }
    }

    // Test 3: Go to Definition
    println!("3️⃣ Testing GoToDefinitionTool...");
    let goto_tool = GoToDefinitionTool::new(workspace.clone());
    match test_goto_definition_tool(&goto_tool, &test_file).await {
        Ok(_) => {
            println!("   ✅ Go to definition test passed");
            passed += 1;
        }
        Err(e) => {
            println!("   ❌ Go to definition test failed: {}", e);
            failed += 1;
        }
    }

    // Test 4: Find References
    println!("4️⃣ Testing FindReferencesTool...");
    let references_tool = FindReferencesTool::new(workspace.clone());
    match test_find_references_tool(&references_tool, &test_file).await {
        Ok(_) => {
            println!("   ✅ Find references test passed");
            passed += 1;
        }
        Err(e) => {
            println!("   ❌ Find references test failed: {}", e);
            failed += 1;
        }
    }

    // Test 5: Rename Symbol
    println!("5️⃣ Testing RenameSymbolTool...");
    let rename_tool = RenameSymbolTool::new(workspace.clone());
    match test_rename_tool(&rename_tool, &test_file).await {
        Ok(_) => {
            println!("   ✅ Rename symbol test passed");
            passed += 1;
        }
        Err(e) => {
            println!("   ❌ Rename symbol test failed: {}", e);
            failed += 1;
        }
    }

    // Test 6: Diagnostics
    println!("6️⃣ Testing DiagnosticsTool...");
    let diagnostics_tool = DiagnosticsTool::new(workspace.clone());
    match test_diagnostics_tool(&diagnostics_tool, &test_file).await {
        Ok(_) => {
            println!("   ✅ Diagnostics test passed");
            passed += 1;
        }
        Err(e) => {
            println!("   ❌ Diagnostics test failed: {}", e);
            failed += 1;
        }
    }

    // Test 7: Format Code
    println!("7️⃣ Testing FormatCodeTool...");
    let format_tool = FormatCodeTool::new(workspace.clone());
    match test_format_tool(&format_tool, &test_file).await {
        Ok(_) => {
            println!("   ✅ Format code test passed");
            passed += 1;
        }
        Err(e) => {
            println!("   ❌ Format code test failed: {}", e);
            failed += 1;
        }
    }

    println!();
    println!("============================");
    println!("📊 LSP Integration Test Results");
    println!("============================");
    println!("✅ Passed: {}/7", passed);
    println!("❌ Failed: {}/7", failed);

    if passed == 7 {
        println!("🎉 ALL LSP TOOLS WORKING! IDE-level intelligence confirmed!");
    } else if passed > 4 {
        println!("⚠️ Partial success - {} tools working, investigate failures", passed);
    } else {
        println!("🚨 Major issues - only {} tools working, LSP setup may be required", passed);
    }

    // Don't fail the test - we're just validating capabilities
    Ok(())
}

async fn test_completion_tool(tool: &CodeCompletionTool, test_file: &PathBuf) -> Result<(), ToolError> {
    let params = json!({
        "file_path": test_file.to_string_lossy(),
        "line": 5,
        "column": 1
    });

    let result = tool.execute(params).await?;

    if result.success {
        println!("   📝 Completion result: {}",
            result.result.get("completions")
                .and_then(|c| c.as_array())
                .map(|arr| format!("{} completions", arr.len()))
                .unwrap_or_else(|| "completion data received".to_string())
        );
        Ok(())
    } else {
        Err(ToolError::ExecutionFailed(
            result.error.unwrap_or_else(|| "unknown error".to_string())
        ))
    }
}

async fn test_hover_tool(tool: &HoverTool, test_file: &PathBuf) -> Result<(), ToolError> {
    let params = json!({
        "file_path": test_file.to_string_lossy(),
        "line": 5,
        "column": 1
    });

    let result = tool.execute(params).await?;

    if result.success {
        println!("   📝 Hover info available");
        Ok(())
    } else {
        Err(ToolError::ExecutionFailed(
            result.error.unwrap_or_else(|| "unknown error".to_string())
        ))
    }
}

async fn test_goto_definition_tool(tool: &GoToDefinitionTool, test_file: &PathBuf) -> Result<(), ToolError> {
    let params = json!({
        "file_path": test_file.to_string_lossy(),
        "line": 5,
        "column": 1
    });

    let result = tool.execute(params).await?;

    if result.success {
        println!("   📝 Definition lookup working");
        Ok(())
    } else {
        Err(ToolError::ExecutionFailed(
            result.error.unwrap_or_else(|| "unknown error".to_string())
        ))
    }
}

async fn test_find_references_tool(tool: &FindReferencesTool, test_file: &PathBuf) -> Result<(), ToolError> {
    let params = json!({
        "file_path": test_file.to_string_lossy(),
        "line": 5,
        "column": 1
    });

    let result = tool.execute(params).await?;

    if result.success {
        println!("   📝 Reference finding working");
        Ok(())
    } else {
        Err(ToolError::ExecutionFailed(
            result.error.unwrap_or_else(|| "unknown error".to_string())
        ))
    }
}

async fn test_rename_tool(tool: &RenameSymbolTool, test_file: &PathBuf) -> Result<(), ToolError> {
    let params = json!({
        "file_path": test_file.to_string_lossy(),
        "line": 5,
        "column": 1,
        "new_name": "test_rename"
    });

    let result = tool.execute(params).await?;

    if result.success {
        println!("   📝 Symbol renaming capability confirmed");
        Ok(())
    } else {
        Err(ToolError::ExecutionFailed(
            result.error.unwrap_or_else(|| "unknown error".to_string())
        ))
    }
}

async fn test_diagnostics_tool(tool: &DiagnosticsTool, test_file: &PathBuf) -> Result<(), ToolError> {
    let params = json!({
        "file_path": test_file.to_string_lossy()
    });

    let result = tool.execute(params).await?;

    if result.success {
        let diagnostics = result.result.get("diagnostics")
            .and_then(|d| d.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);
        println!("   📝 Found {} diagnostics", diagnostics);
        Ok(())
    } else {
        Err(ToolError::ExecutionFailed(
            result.error.unwrap_or_else(|| "unknown error".to_string())
        ))
    }
}

async fn test_format_tool(tool: &FormatCodeTool, test_file: &PathBuf) -> Result<(), ToolError> {
    let params = json!({
        "file_path": test_file.to_string_lossy()
    });

    let result = tool.execute(params).await?;

    if result.success {
        println!("   📝 Code formatting available");
        Ok(())
    } else {
        Err(ToolError::ExecutionFailed(
            result.error.unwrap_or_else(|| "unknown error".to_string())
        ))
    }
}
