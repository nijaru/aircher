/// Debug multi-tool execution
use anyhow::Result;
use std::sync::Arc;

use aircher::auth::AuthManager;
use aircher::config::ConfigManager;
use aircher::providers::{ProviderManager, ChatRequest, Message, MessageRole};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 DEBUG: What does LLM actually return for multi-step tasks?");
    println!("==========================================================\n");

    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let provider_manager = Arc::new(ProviderManager::new(&config, auth_manager.clone()).await?);

    if let Some(provider) = provider_manager.get_provider("ollama") {
        // Direct test with provider
        let request = ChatRequest {
            model: "gpt-oss".to_string(),
            messages: vec![
                Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: MessageRole::System,
                    content: "You are an AI assistant with access to tools. Use tools to complete tasks. For multi-step tasks, call tools sequentially - do not try to answer everything at once.".to_string(),
                    timestamp: chrono::Utc::now(),
                    tokens_used: None,
                    cost: None,
                },
                Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: MessageRole::User,
                    content: "List the files in src/agent/tools/, then read the first 10 lines of mod.rs from that directory".to_string(),
                    timestamp: chrono::Utc::now(),
                    tokens_used: None,
                    cost: None,
                },
            ],
            tools: Some(vec![
                aircher::providers::Tool {
                    name: "list_files".to_string(),
                    description: "List files in a directory".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string"}
                        }
                    }),
                },
                aircher::providers::Tool {
                    name: "read_file".to_string(),
                    description: "Read contents of a file".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string"},
                            "line_start": {"type": "number"},
                            "line_end": {"type": "number"}
                        }
                    }),
                },
            ]),
            temperature: Some(0.7),
            max_tokens: Some(2000),
            stream: false,
        };

        println!("Sending request to LLM with 2 tools available...\n");

        match provider.chat(&request).await {
            Ok(response) => {
                println!("📊 LLM RESPONSE:");
                println!("  Content length: {} chars", response.content.len());
                println!("  Tool calls: {:?}", response.tool_calls);

                if let Some(ref calls) = response.tool_calls {
                    println!("\n🔧 Tool calls requested: {}", calls.len());
                    for (i, call) in calls.iter().enumerate() {
                        println!("  {}. {} with args: {:?}", i + 1, call.name, call.arguments);
                    }

                    if calls.len() == 1 {
                        println!("\n❌ PROBLEM IDENTIFIED:");
                        println!("LLM only calls ONE tool even for multi-step tasks!");
                        println!("The LLM is not understanding it should call tools sequentially.");
                    } else if calls.len() > 1 {
                        println!("\n✅ Good: LLM requested multiple tools");
                    }
                } else {
                    println!("\n⚠️ No structured tool calls in response");
                }

                println!("\n📄 Response content (first 500 chars):");
                println!("{}", response.content.chars().take(500).collect::<String>());

                // Check if LLM is trying to answer without calling all tools
                let mentions_files = response.content.contains(".rs") || response.content.contains("mod.rs");
                let mentions_code = response.content.contains("use ") || response.content.contains("impl");

                if mentions_files && mentions_code && response.tool_calls.as_ref().map(|c| c.len()).unwrap_or(0) < 2 {
                    println!("\n🚨 CRITICAL ISSUE:");
                    println!("LLM is HALLUCINATING results instead of calling tools!");
                    println!("It mentions files AND code but only called {} tool(s)",
                             response.tool_calls.as_ref().map(|c| c.len()).unwrap_or(0));
                }
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