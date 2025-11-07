#!/usr/bin/env python3

import subprocess
import json
import sys

def test_web_tools():
    """Test if web tools are actually available and working"""

    # Try to create a simple tool registry test
    test_code = '''
use aircher::agent::tools::ToolRegistry;
use serde_json::json;

#[tokio::main]
async fn main() {
    let registry = ToolRegistry::default();
    let tools = registry.list_tools();

    println!("Available tools:");
    for tool in &tools {
        println!("  - {}: {}", tool.name, tool.description);
    }

    // Test if web_browse tool is available
    let has_web_browse = tools.iter().any(|t| t.name == "web_browse");
    let has_web_search = tools.iter().any(|t| t.name == "web_search");
    let has_build_project = tools.iter().any(|t| t.name == "build_project");

    println!("\\nWeb tools available:");
    println!("  web_browse: {}", has_web_browse);
    println!("  web_search: {}", has_web_search);
    println!("  build_project: {}", has_build_project);

    // Try to execute web_browse tool if available
    if let Some(web_tool) = registry.get("web_browse") {
        println!("\\nTesting web_browse tool...");
        let params = json!({"url": "https://httpbin.org/json"});
        match web_tool.execute(params).await {
            Ok(result) => {
                println!("Web browse SUCCESS: {}", result.success);
                if result.success {
                    println!("Content length: {}", result.result.to_string().len());
                }
            }
            Err(e) => {
                println!("Web browse FAILED: {}", e);
            }
        }
    }

    // Try web search
    if let Some(search_tool) = registry.get("web_search") {
        println!("\\nTesting web_search tool...");
        let params = json!({"query": "rust programming"});
        match search_tool.execute(params).await {
            Ok(result) => {
                println!("Web search SUCCESS: {}", result.success);
            }
            Err(e) => {
                println!("Web search FAILED: {}", e);
            }
        }
    }
}
'''

    # Write test file
    with open('/Users/nick/github/nijaru/aircher/test_tools.rs', 'w') as f:
        f.write(test_code)

    return True

if __name__ == "__main__":
    test_web_tools()
    print("Test file created: /Users/nick/github/nijaru/aircher/test_tools.rs")
    print("Run with: cd /Users/nick/github/nijaru/aircher && cargo run --bin test_tools")