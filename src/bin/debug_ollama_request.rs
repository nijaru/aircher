/// Debug Ollama request to see exact payload and error
use anyhow::Result;
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 OLLAMA REQUEST DEBUG");
    println!("=======================\n");

    let client = reqwest::Client::new();

    // Test 1: Test the payload our provider sends
    println!("1. Testing Ollama /api/chat with our payload format...");

    let payload = json!({
        "model": "gpt-oss",
        "messages": [
            {
                "role": "user",
                "content": "Say hello"
            }
        ],
        "stream": false
    });

    println!("   📝 Payload: {}", serde_json::to_string_pretty(&payload)?);

    match tokio::time::timeout(
        Duration::from_secs(15),
        client.post("http://localhost:11434/api/chat")
            .json(&payload)
            .send()
    ).await {
        Ok(Ok(response)) => {
            let status = response.status();
            let text = response.text().await?;
            println!("   📊 Status: {}", status);
            println!("   📄 Response: {}", text);
        }
        Ok(Err(e)) => {
            println!("   ❌ Request error: {}", e);
        }
        Err(_) => {
            println!("   ⏰ Request timed out");
        }
    }

    // Test 2: Test simple /api/generate (which we know works from docs)
    println!("\n2. Testing Ollama /api/generate (simple format)...");

    let simple_payload = json!({
        "model": "gpt-oss",
        "prompt": "Say hello",
        "stream": false
    });

    println!("   📝 Payload: {}", serde_json::to_string_pretty(&simple_payload)?);

    match tokio::time::timeout(
        Duration::from_secs(15),
        client.post("http://localhost:11434/api/generate")
            .json(&simple_payload)
            .send()
    ).await {
        Ok(Ok(response)) => {
            let status = response.status();
            let text = response.text().await?;
            println!("   📊 Status: {}", status);
            println!("   📄 Response: {}", text.chars().take(500).collect::<String>());
        }
        Ok(Err(e)) => {
            println!("   ❌ Request error: {}", e);
        }
        Err(_) => {
            println!("   ⏰ Request timed out");
        }
    }

    // Test 3: Test if gpt-oss model exists and supports chat format
    println!("\n3. Testing model info...");

    match tokio::time::timeout(
        Duration::from_secs(10),
        client.get("http://localhost:11434/api/tags").send()
    ).await {
        Ok(Ok(response)) => {
            if let Ok(text) = response.text().await {
                if let Ok(models_data) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(models) = models_data["models"].as_array() {
                        let gpt_oss_exists = models.iter().any(|m| {
                            m["name"].as_str().unwrap_or("").starts_with("gpt-oss")
                        });
                        println!("   🤖 gpt-oss model exists: {}", gpt_oss_exists);

                        // Show all model names
                        println!("   📋 Available models:");
                        for model in models {
                            if let Some(name) = model["name"].as_str() {
                                println!("      - {}", name);
                            }
                        }
                    }
                }
            }
        }
        Ok(Err(e)) => {
            println!("   ❌ Model info error: {}", e);
        }
        Err(_) => {
            println!("   ⏰ Model info timed out");
        }
    }

    Ok(())
}