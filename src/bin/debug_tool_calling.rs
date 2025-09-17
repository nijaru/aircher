/// Debug Tool Calling - Inspect what's actually sent to LLM
///
/// This test shows exactly what tool schemas are sent to the LLM
/// and what responses come back to understand the tool calling failure.

use anyhow::Result;
use std::sync::Arc;

use aircher::auth::AuthManager;
use aircher::config::ConfigManager;
use aircher::providers::{ProviderManager, ChatRequest, Message, MessageRole, Tool};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 TOOL CALLING DEBUG");
    println!("=====================\n");

    // Set up provider
    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let provider_manager = ProviderManager::new(&config, auth_manager).await?;

    let provider = provider_manager.get_provider("ollama")
        .ok_or_else(|| anyhow::anyhow!("Ollama provider not found"))?;

    println!("📊 Provider Info:");
    println!("  • Name: {}", provider.name());
    println!("  • Supports Tools: {}", provider.supports_tools());
    println!("  • Context Window: {}", provider.context_window());

    // Create mock tools schema
    let tools = vec![
        Tool {
            name: "write_file".to_string(),
            description: "Write content to a file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The file path to write to"
                    },
                    "content": {
                        "type": "string",
                        "description": "The content to write"
                    }
                },
                "required": ["path", "content"]
            }),
        },
        Tool {
            name: "read_file".to_string(),
            description: "Read content from a file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The file path to read from"
                    }
                },
                "required": ["path"]
            }),
        },
    ];

    println!("\n🔧 Tools Schema Being Sent:");
    for tool in &tools {
        println!("  • {}: {}", tool.name, tool.description);
    }

    // Create test request
    let messages = vec![
        Message {
            id: "system".to_string(),
            role: MessageRole::System,
            content: "You are Aircher, an AI coding assistant. You have access to tools to help with coding tasks. When you need to use a tool, respond with the exact format: <tool>tool_name</tool><params>{\"param\": \"value\"}</params>".to_string(),
            timestamp: chrono::Utc::now(),
            tokens_used: None,
            cost: None,
        },
        Message {
            id: "user".to_string(),
            role: MessageRole::User,
            content: "Please write a test file at /tmp/debug_test.txt with the content 'Hello from debug test'".to_string(),
            timestamp: chrono::Utc::now(),
            tokens_used: None,
            cost: None,
        },
    ];

    let request = ChatRequest {
        messages,
        model: "gpt-oss".to_string(),
        temperature: Some(0.1),
        max_tokens: Some(1000),
        stream: false,
        tools: Some(tools),
    };

    println!("\n📤 Sending Request to LLM:");
    println!("  • Model: gpt-oss");
    println!("  • Tools: {} available", request.tools.as_ref().unwrap().len());
    println!("  • Message: Write test file");

    // First, let's try a direct HTTP call to see what Ollama returns
    println!("\n🌐 Direct HTTP Test:");
    let direct_test = test_ollama_directly().await;
    if let Err(e) = direct_test {
        println!("  ❌ Direct test failed: {}", e);
        return Ok(());
    }

    // Send request and get response
    match provider.chat(&request).await {
        Ok(response) => {
            println!("\n📥 LLM Response:");
            println!("  • Finish Reason: {:?}", response.finish_reason);
            println!("  • Tool Calls: {:?}", response.tool_calls);
            println!("  • Content Length: {} chars", response.content.len());
            println!("\n📝 Full Response Content:");
            println!("---");
            println!("{}", response.content);
            println!("---");

            // Test our parser on this response
            let parser = aircher::agent::parser::ToolCallParser::new()?;
            let parsed_calls = parser.parse(&response.content);

            println!("\n🔍 Parsed Tool Calls:");
            if parsed_calls.is_empty() {
                println!("  ❌ No tool calls found by parser");
                println!("  💡 This explains why tools aren't executing!");
            } else {
                for call in &parsed_calls {
                    println!("  ✅ Found: {} with params: {}", call.name, call.parameters);
                }
            }

            // Also test structured parsing
            let (clean_text, structured_calls) = parser.parse_structured(&response.content)?;
            println!("\n🔍 Structured Parse Results:");
            println!("  • Clean Text: {}", clean_text.chars().take(100).collect::<String>());
            println!("  • Tool Calls: {}", structured_calls.len());
        }
        Err(e) => {
            println!("\n❌ Request Failed: {}", e);
        }
    }

    Ok(())
}

async fn test_ollama_directly() -> Result<()> {
    let client = reqwest::Client::new();

    let tools = serde_json::json!([
        {
            "type": "function",
            "function": {
                "name": "write_file",
                "description": "Write content to a file",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The file path to write to"
                        },
                        "content": {
                            "type": "string",
                            "description": "The content to write"
                        }
                    },
                    "required": ["path", "content"]
                }
            }
        }
    ]);

    let request_body = serde_json::json!({
        "model": "gpt-oss",
        "messages": [
            {
                "role": "system",
                "content": "You are Aircher, an AI coding assistant. You have access to tools to help with tasks. When you need to use a tool, call the appropriate function."
            },
            {
                "role": "user",
                "content": "Please write a test file at /tmp/debug_test.txt with the content 'Hello from debug test'"
            }
        ],
        "tools": tools,
        "stream": false
    });

    println!("  📤 Sending to: http://localhost:11434/api/chat");
    println!("  🔧 Tools in request: 1");

    let response = client
        .post("http://localhost:11434/api/chat")
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        println!("  ❌ HTTP Error: {} - {}", status, error_text);
        return Err(anyhow::anyhow!("HTTP error: {}", status));
    }

    let response_text = response.text().await?;
    println!("  📥 Raw Response Length: {} chars", response_text.len());

    // Try to parse as JSON
    match serde_json::from_str::<serde_json::Value>(&response_text) {
        Ok(json) => {
            println!("  ✅ Valid JSON response");
            if let Some(message) = json.get("message") {
                if let Some(content) = message.get("content") {
                    println!("  📝 Content: {}", content.as_str().unwrap_or("").chars().take(200).collect::<String>());
                }
                if let Some(tool_calls) = message.get("tool_calls") {
                    println!("  🔧 Tool Calls: {}", tool_calls);
                } else {
                    println!("  ❌ No tool_calls field in response");
                }
            }
        }
        Err(e) => {
            println!("  ❌ Invalid JSON: {}", e);
            println!("  📝 Raw response: {}", response_text.chars().take(500).collect::<String>());
        }
    }

    Ok(())
}