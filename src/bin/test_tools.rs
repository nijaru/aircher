use aircher::agent::tools::ToolRegistry;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("🧪 Testing Aircher Tool Registry\n");

    let registry = ToolRegistry::default();
    let tools = registry.list_tools();

    println!("Available tools ({} total):", tools.len());
    for tool in &tools {
        println!("  - {}: {}", tool.name, tool.description);
    }

    // Test if new web tools are available
    let has_web_browse = tools.iter().any(|t| t.name == "web_browse");
    let has_web_search = tools.iter().any(|t| t.name == "web_search");
    let has_build_project = tools.iter().any(|t| t.name == "build_project");

    println!("\n🌐 Web tools availability:");
    println!("  web_browse: {}", has_web_browse);
    println!("  web_search: {}", has_web_search);
    println!("  build_project: {}", has_build_project);

    // Test core file tools
    let has_read_file = tools.iter().any(|t| t.name == "read_file");
    let has_write_file = tools.iter().any(|t| t.name == "write_file");
    println!("\n📁 File tools availability:");
    println!("  read_file: {}", has_read_file);
    println!("  write_file: {}", has_write_file);

    // Try to execute web_browse tool if available
    if let Some(web_tool) = registry.get("web_browse") {
        println!("\n🧪 Testing web_browse tool...");
        let params = json!({"url": "https://httpbin.org/json"});
        match web_tool.execute(params).await {
            Ok(result) => {
                println!("✅ Web browse SUCCESS: {}", result.success);
                if result.success {
                    let content_str = result.result.to_string();
                    println!("📄 Content length: {} chars", content_str.len());
                    if content_str.len() > 100 {
                        println!("📄 Content preview: {}...", &content_str[..100]);
                    }
                } else {
                    println!("❌ Tool returned success=false");
                    if let Some(error) = result.error {
                        println!("❌ Error: {}", error);
                    }
                }
            }
            Err(e) => {
                println!("❌ Web browse FAILED: {}", e);
            }
        }
    } else {
        println!("❌ web_browse tool not found in registry");
    }

    // Try web search
    if let Some(search_tool) = registry.get("web_search") {
        println!("\n🧪 Testing web_search tool...");
        let params = json!({"query": "rust programming"});
        match search_tool.execute(params).await {
            Ok(result) => {
                println!("✅ Web search SUCCESS: {}", result.success);
                if result.success {
                    println!("🔍 Search completed successfully");
                } else {
                    println!("❌ Tool returned success=false");
                    if let Some(error) = result.error {
                        println!("❌ Error: {}", error);
                    }
                }
            }
            Err(e) => {
                println!("❌ Web search FAILED: {}", e);
            }
        }
    } else {
        println!("❌ web_search tool not found in registry");
    }

    // Test read_file tool with Cargo.toml
    if let Some(read_tool) = registry.get("read_file") {
        println!("\n🧪 Testing read_file tool...");
        let params = json!({"path": "Cargo.toml"});
        match read_tool.execute(params).await {
            Ok(result) => {
                println!("✅ Read file SUCCESS: {}", result.success);
            }
            Err(e) => {
                println!("❌ Read file FAILED: {}", e);
            }
        }
    } else {
        println!("❌ read_file tool not found in registry");
    }

    println!("\n🏁 Tool testing completed!");
}