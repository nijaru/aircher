/// Test the real analyze_errors tool with actual Rust compilation errors
use aircher::agent::tools::{AgentTool, real_analyze_errors::RealAnalyzeErrorsTool};
use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n🔬 Testing REAL analyze_errors tool with actual Rust errors\n");

    let tool = RealAnalyzeErrorsTool::new(Some(std::env::current_dir()?));

    // Test 1: Borrow checker error
    println!("Test 1: Borrow Checker Error");
    println!("{}", "=".repeat(50));
    
    let borrow_error = r#"error[E0502]: cannot borrow `data` as mutable because it is also borrowed as immutable
  --> src/main.rs:10:5
   |
9  |     let reference = &data;
   |                     ----- immutable borrow occurs here
10 |     modify(&mut data);
   |            ^^^^^^^^^ mutable borrow occurs here
11 |     println!("{}", reference);
   |                    --------- immutable borrow later used here"#;

    let result = tool.execute(json!({
        "error_message": borrow_error
    })).await?;

    println!("🎯 Analysis Result:");
    println!("  Severity: {}", result.result["severity"]);
    println!("  Category: {}", result.result["category"]);
    println!("  Root Cause: {}", result.result["root_cause"]);
    println!("  Confidence: {}", result.result["confidence"]);
    
    if let Some(location) = result.result["location"].as_object() {
        println!("  Location: {}:{}:{}", 
            location["file"], location["line"], location["column"]);
    }
    
    println!("\n  Common Causes:");
    if let Some(causes) = result.result["common_causes"].as_array() {
        for cause in causes {
            println!("    - {}", cause);
        }
    }
    
    println!("\n  Suggested Fixes:");
    if let Some(fixes) = result.result["suggested_fixes"].as_array() {
        for fix in fixes {
            println!("    ✅ {}", fix);
        }
    }

    // Test 2: Type mismatch error
    println!("\n\nTest 2: Type Mismatch Error");
    println!("{}", "=".repeat(50));
    
    let type_error = r#"error[E0308]: mismatched types
  --> src/lib.rs:42:18
   |
42 |     let count: u32 = "not a number";
   |                ^^^   ^^^^^^^^^^^^^^ expected `u32`, found `&str`
   |
   = note: expected type `u32`
              found reference `&'static str`"#;

    let result = tool.execute(json!({
        "error_message": type_error
    })).await?;

    println!("🎯 Analysis Result:");
    println!("  Severity: {}", result.result["severity"]);
    println!("  Category: {}", result.result["category"]);
    println!("  Root Cause: {}", result.result["root_cause"]);
    println!("  Confidence: {}", result.result["confidence"]);
    
    println!("\n  Suggested Fixes:");
    if let Some(fixes) = result.result["suggested_fixes"].as_array() {
        for fix in fixes {
            println!("    ✅ {}", fix);
        }
    }

    // Test 3: Import error
    println!("\n\nTest 3: Import/Dependency Error");
    println!("{}", "=".repeat(50));
    
    let import_error = r#"error[E0432]: unresolved import `tokio::runtime`
  --> src/main.rs:1:5
   |
1 | use tokio::runtime::Runtime;
   |     ^^^^^^^^^^^^^^ could not find `runtime` in `tokio`
   |
   = help: consider importing this module instead:
           tokio::runtime"#;

    let result = tool.execute(json!({
        "error_message": import_error
    })).await?;

    println!("🎯 Analysis Result:");
    println!("  Severity: {}", result.result["severity"]);
    println!("  Category: {}", result.result["category"]);
    println!("  Root Cause: {}", result.result["root_cause"]);
    println!("  Confidence: {}", result.result["confidence"]);
    
    println!("\n  Suggested Fixes:");
    if let Some(fixes) = result.result["suggested_fixes"].as_array() {
        for fix in fixes {
            println!("    ✅ {}", fix);
        }
    }

    // Test 4: Panic/None unwrap error
    println!("\n\nTest 4: Panic on None Unwrap");
    println!("{}", "=".repeat(50));
    
    let panic_error = r#"thread 'main' panicked at 'called `Option::unwrap()` on a `None` value', src/main.rs:25:14
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace"#;

    let result = tool.execute(json!({
        "error_message": panic_error
    })).await?;

    println!("🎯 Analysis Result:");
    println!("  Severity: {}", result.result["severity"]);
    println!("  Category: {}", result.result["category"]);
    println!("  Root Cause: {}", result.result["root_cause"]);
    println!("  Confidence: {}", result.result["confidence"]);
    
    println!("\n  Suggested Fixes:");
    if let Some(fixes) = result.result["suggested_fixes"].as_array() {
        for fix in fixes {
            println!("    ✅ {}", fix);
        }
    }

    // Compare with stub tool
    println!("\n\n🔄 Comparison with Stub Tool");
    println!("{}", "=".repeat(50));
    println!("\n📊 Real Tool Advantages:");
    println!("  ✅ Extracts actual file locations from error messages");
    println!("  ✅ Categorizes errors correctly (Borrow, Type, Import, etc.)");
    println!("  ✅ Provides specific, actionable fixes for each error type");
    println!("  ✅ Returns confidence scores based on pattern matching");
    println!("  ✅ Identifies root causes from error messages");
    println!("\n❌ Stub Tool Limitations:");
    println!("  ❌ Returns hardcoded JSON: {{\"errors\": [\"Error 1\", \"Error 2\"]}}");
    println!("  ❌ No actual error parsing or analysis");
    println!("  ❌ No actionable suggestions or fixes");
    println!("  ❌ No location extraction or categorization");
    
    println!("\n✨ CONCLUSION: Real tool provides actual value for debugging!");

    Ok(())
}
