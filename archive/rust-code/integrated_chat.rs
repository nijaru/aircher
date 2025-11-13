// Example showing the integrated cost + context + model selection system
use anyhow::Result;
use aircher::cost::{ModelConfiguration, ModelTier};
use aircher::context::ContextDisplay;
use aircher::providers::{ChatRequest, Message, MessageRole};

/// Example of how the integrated system works for users
#[tokio::main]
async fn main() -> Result<()> {
    // Load user's model configuration
    let model_config = ModelConfiguration::default();

    // Show current configuration
    println!("{}", model_config.get_summary());

    // Example conversations with different providers and tasks

    println!("\n🔄 Example Conversations:\n");

    // 1. Commit message (light task)
    simulate_chat(
        &model_config,
        "claude",
        Some("commit_messages"),
        "Fix memory leak in buffer allocation",
        100, 50
    ).await?;

    // 2. Architecture planning (planning task)
    simulate_chat(
        &model_config,
        "claude",
        Some("architecture"),
        "Design a microservices architecture for this e-commerce platform",
        2000, 1500
    ).await?;

    // 3. General question (main model)
    simulate_chat(
        &model_config,
        "openai",
        None,
        "Explain the differences between microservices and monoliths",
        800, 1200
    ).await?;

    // 4. Summary task (light model)
    simulate_chat(
        &model_config,
        "gemini",
        Some("summaries"),
        "Summarize this 50-page technical document: [document content]",
        8000, 400
    ).await?;

    // 5. Free local model
    simulate_chat(
        &model_config,
        "ollama",
        Some("quick_questions"),
        "How do I iterate over a HashMap in Rust?",
        200, 300
    ).await?;

    Ok(())
}

async fn simulate_chat(
    config: &ModelConfiguration,
    provider: &str,
    task: Option<&str>,
    user_message: &str,
    input_tokens: u32,
    output_tokens: u32,
) -> Result<()> {
    // 1. Select the appropriate model
    let (model, tier, reason) = config.select_model(provider, task);

    // 2. Estimate cost (simplified)
    let estimated_cost = estimate_cost(provider, &model, input_tokens, output_tokens);

    // 3. Create context display
    let context_display = ContextDisplay::new(
        input_tokens,
        output_tokens,
        get_context_window(&model),
        model.clone(),
        provider.to_string(),
        estimated_cost,
    );

    // 4. Show what would happen
    println!("┌─ Chat Request ─────────────────────────────────────────────────");
    println!("│ Provider: {} | Task: {:?}", provider, task);
    println!("│ Model: {} ({})", model, tier.as_str());
    println!("│ Reason: {}", reason);
    println!("│ Message: \"{}\"", truncate_message(user_message, 50));
    println!("├─ Usage & Cost ────────────────────────────────────────────────");
    println!("│ {}", context_display.format_compact());

    // Show any warnings
    if let Some(warning) = context_display.needs_attention() {
        println!("│ ⚠️  {}", warning);
    }

    println!("└────────────────────────────────────────────────────────────────");
    println!();

    Ok(())
}

fn estimate_cost(provider: &str, model: &str, input_tokens: u32, output_tokens: u32) -> Option<f64> {
    // Simplified cost estimation
    match (provider, model) {
        ("openai", "gpt-4o") => {
            Some((input_tokens as f64 / 1_000_000.0) * 5.0 + (output_tokens as f64 / 1_000_000.0) * 15.0)
        }
        ("openai", "gpt-4o-mini") => {
            Some((input_tokens as f64 / 1_000_000.0) * 0.15 + (output_tokens as f64 / 1_000_000.0) * 0.6)
        }
        ("claude", "claude-3-5-sonnet-20241022") => {
            Some((input_tokens as f64 / 1_000_000.0) * 3.0 + (output_tokens as f64 / 1_000_000.0) * 15.0)
        }
        ("claude", "claude-3-5-haiku-20241022") => {
            Some((input_tokens as f64 / 1_000_000.0) * 0.25 + (output_tokens as f64 / 1_000_000.0) * 1.25)
        }
        ("gemini", _) => {
            Some((input_tokens as f64 / 1_000_000.0) * 0.075 + (output_tokens as f64 / 1_000_000.0) * 0.30)
        }
        ("ollama", _) => Some(0.0), // Free
        _ => None,
    }
}

fn get_context_window(model: &str) -> u32 {
    match model {
        m if m.contains("claude") => 200_000,
        m if m.contains("gpt-4") => 128_000,
        m if m.contains("gemini") => 1_000_000,
        m if m.contains("llama") => 128_000,
        _ => 8_000,
    }
}

fn truncate_message(msg: &str, max_len: usize) -> String {
    if msg.len() <= max_len {
        msg.to_string()
    } else {
        format!("{}...", &msg[..max_len.saturating_sub(3)])
    }
}
