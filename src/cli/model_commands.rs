use anyhow::Result;
use clap::{Arg, ArgMatches, Command};

use crate::cost::{ModelConfiguration, ModelTier};

pub fn model_command() -> Command {
    Command::new("model")
        .about("Configure model selection (main/light/coding models per provider)")
        .subcommand(
            Command::new("show")
                .about("Show current model configuration")
        )
        .subcommand(
            Command::new("set")
                .about("Set model for provider and tier")
                .arg(Arg::new("provider")
                    .help("Provider (claude, openai, gemini, ollama)")
                    .required(true)
                    .index(1))
                .arg(Arg::new("tier")
                    .help("Model tier (main, light, coding)")
                    .required(true)
                    .index(2))
                .arg(Arg::new("model")
                    .help("Model name")
                    .required(true)
                    .index(3))
        )
        .subcommand(
            Command::new("task")
                .about("Configure which tier to use for a task")
                .arg(Arg::new("task")
                    .help("Task name")
                    .required(true)
                    .index(1))
                .arg(Arg::new("tier")
                    .help("Tier to use (main, light, coding)")
                    .required(true)
                    .index(2))
        )
        .subcommand(
            Command::new("test")
                .about("Test model selection for a provider/task")
                .arg(Arg::new("provider")
                    .help("Provider to test")
                    .required(true)
                    .index(1))
                .arg(Arg::new("task")
                    .help("Task to test (optional)")
                    .index(2))
        )
        .subcommand(
            Command::new("suggestions")
                .about("Show optimization suggestions")
        )
        .subcommand(
            Command::new("examples")
                .about("Show example configurations")
        )
}

pub async fn handle_model_command(matches: &ArgMatches) -> Result<()> {
    // Load current configuration
    let mut config = load_model_config().await?;

    match matches.subcommand() {
        Some(("show", _)) => {
            println!("{}", config.get_summary());
        }

        Some(("set", sub_matches)) => {
            let provider = sub_matches.get_one::<String>("provider").unwrap();
            let tier_str = sub_matches.get_one::<String>("tier").unwrap();
            let model = sub_matches.get_one::<String>("model").unwrap();

            let tier = ModelTier::from_str(tier_str)
                .ok_or_else(|| anyhow::anyhow!("Invalid tier '{}'. Use: main, light, or coding", tier_str))?;

            config.set_model(provider, tier, model);
            save_model_config(&config).await?;

            println!("✅ Set {} {} model to {}", provider, tier.as_str(), model);
            show_tier_description(tier);
        }

        Some(("task", sub_matches)) => {
            let task = sub_matches.get_one::<String>("task").unwrap();
            let tier_str = sub_matches.get_one::<String>("tier").unwrap();

            let tier = ModelTier::from_str(tier_str)
                .ok_or_else(|| anyhow::anyhow!("Invalid tier '{}'. Use: main, light, or coding", tier_str))?;

            config.set_task_tier(task, tier);
            save_model_config(&config).await?;

            println!("✅ Set '{}' task to use {} models", task, tier.as_str());
            show_tier_description(tier);
        }

        Some(("test", sub_matches)) => {
            let provider = sub_matches.get_one::<String>("provider").unwrap();
            let task = sub_matches.get_one::<String>("task");

            let (model, tier, reason) = config.select_model(provider, task.map(|s| s.as_str()));

            println!("🧪 Model Selection Test:");
            println!("  Provider: {}", provider);
            if let Some(task) = task {
                println!("  Task: {}", task);
            } else {
                println!("  Task: (none specified)");
            }
            println!("  Selected: {} ({})", model, tier.as_str());
            println!("  Reason: {}", reason);
            println!();
            show_tier_description(tier);
        }

        Some(("suggestions", _)) => {
            let suggestions = config.get_suggestions();
            println!("💡 Optimization Suggestions:\n");
            
            if suggestions.is_empty() {
                println!("Your configuration looks great! 🎉");
            } else {
                for (i, suggestion) in suggestions.iter().enumerate() {
                    println!("{}. {}", i + 1, suggestion);
                }
            }
        }

        Some(("examples", _)) => {
            show_example_configurations();
        }

        _ => {
            println!("Use 'aircher model --help' to see available commands");
        }
    }

    Ok(())
}

fn show_tier_description(tier: ModelTier) {
    println!("📝 {}: {}", tier.as_str().to_uppercase(), tier.description());
}

fn show_example_configurations() {
    println!(r#"
📖 Example Model Configurations:

💰 Cost-Optimized:
  aircher model set claude light claude-3-5-haiku-20241022
  aircher model set openai light gpt-4o-mini
  aircher model set openai main gpt-4o-mini

🏆 Quality-First:
  aircher model set claude coding claude-3-5-sonnet-20241022
  aircher model set openai coding gpt-4o
  aircher model set gemini coding gemini-2.5-pro

🆓 Privacy/Local:
  aircher model set ollama main llama3.3
  aircher model set ollama coding codellama
  aircher model set ollama light llama3.1

🎯 Balanced (Recommended):
  aircher model set claude main claude-3-5-sonnet-20241022
  aircher model set claude light claude-3-5-haiku-20241022
  aircher model set openai light gpt-4o-mini
  aircher model set gemini light gemini-1.5-flash

🔧 Custom Task Routing:
  aircher model task documentation light     # Use fast models for docs
  aircher model task architecture coding    # Use precise models for architecture
  aircher model task brainstorming main     # Use balanced models for brainstorming

💡 Model Tier Guide:
  main   = General purpose, good balance of quality and cost
  light  = Fast and cheap for simple tasks (summaries, commits)
  coding = Specialized for code (reviews, debugging, generation)
"#);
}

async fn load_model_config() -> Result<ModelConfiguration> {
    // In real implementation, load from ~/.config/aircher/models.toml
    // For now, return default
    Ok(ModelConfiguration::default())
    
    // Real implementation would be:
    // let config_path = get_config_dir()?.join("models.toml");
    // if config_path.exists() {
    //     let content = tokio::fs::read_to_string(config_path).await?;
    //     Ok(toml::from_str(&content)?)
    // } else {
    //     Ok(ModelConfiguration::default())
    // }
}

async fn save_model_config(config: &ModelConfiguration) -> Result<()> {
    // In real implementation, save to ~/.config/aircher/models.toml
    // For now, just log
    tracing::info!("Would save model configuration");
    
    // Real implementation would be:
    // let config_path = get_config_dir()?.join("models.toml");
    // let content = toml::to_string_pretty(config)?;
    // tokio::fs::write(config_path, content).await?;
    
    Ok(())
}

/// Show the current model that would be selected for each provider
pub fn show_current_selection_summary(config: &ModelConfiguration) -> String {
    let mut summary = String::new();
    
    summary.push_str("🤖 Current Model Selection:\n");
    
    for provider in ["claude", "openai", "gemini", "ollama"] {
        if let Some(models) = config.get_provider_models(provider) {
            summary.push_str(&format!("  {}: {}\n", provider, models.main));
        }
    }
    
    summary.push_str("\n💡 Use 'aircher model show' for full configuration");
    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_command_structure() {
        let cmd = model_command();
        let subcommands: Vec<_> = cmd.get_subcommands().map(|sc| sc.get_name()).collect();
        
        assert!(subcommands.contains(&"show"));
        assert!(subcommands.contains(&"set"));
        assert!(subcommands.contains(&"task"));
        assert!(subcommands.contains(&"test"));
    }
}