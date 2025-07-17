use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use tracing::info;

use crate::cost::SimpleModelSelector;

pub fn model_config_command() -> Command {
    Command::new("model")
        .about("Configure model selection preferences")
        .subcommand(
            Command::new("list")
                .about("Show current model configuration")
        )
        .subcommand(
            Command::new("set-provider")
                .about("Set default model for a provider")
                .arg(Arg::new("provider")
                    .help("Provider name (claude, openai, gemini, ollama)")
                    .required(true)
                    .index(1))
                .arg(Arg::new("model")
                    .help("Model name")
                    .required(true)
                    .index(2))
        )
        .subcommand(
            Command::new("set-task")
                .about("Set model override for a specific task type")
                .arg(Arg::new("task")
                    .help("Task type (code_review, commit_messages, summaries, etc.)")
                    .required(true)
                    .index(1))
                .arg(Arg::new("model")
                    .help("Model name")
                    .required(true)
                    .index(2))
        )
        .subcommand(
            Command::new("remove-task")
                .about("Remove task-specific model override")
                .arg(Arg::new("task")
                    .help("Task type to remove override for")
                    .required(true)
                    .index(1))
        )
        .subcommand(
            Command::new("suggestions")
                .about("Show optimization suggestions")
        )
        .subcommand(
            Command::new("test")
                .about("Test model selection for different scenarios")
                .arg(Arg::new("provider")
                    .help("Provider to test")
                    .required(true)
                    .index(1))
                .arg(Arg::new("task")
                    .help("Task type to test (optional)")
                    .index(2))
        )
}

pub async fn handle_model_config(matches: &ArgMatches, selector: &mut SimpleModelSelector) -> Result<()> {
    match matches.subcommand() {
        Some(("list", _)) => {
            println!("{}", selector.get_config_summary());
        }
        
        Some(("set-provider", sub_matches)) => {
            let provider = sub_matches.get_one::<String>("provider").unwrap();
            let model = sub_matches.get_one::<String>("model").unwrap();
            
            selector.set_provider_default(provider, model);
            println!("✅ Set {} default model to {}", provider, model);
            
            // Save configuration would happen here
            save_selector_config(selector).await?;
        }
        
        Some(("set-task", sub_matches)) => {
            let task = sub_matches.get_one::<String>("task").unwrap();
            let model = sub_matches.get_one::<String>("model").unwrap();
            
            selector.set_task_override(task, model);
            println!("✅ Set {} task override to {}", task, model);
            
            save_selector_config(selector).await?;
        }
        
        Some(("remove-task", sub_matches)) => {
            let task = sub_matches.get_one::<String>("task").unwrap();
            
            selector.remove_task_override(task);
            println!("✅ Removed {} task override", task);
            
            save_selector_config(selector).await?;
        }
        
        Some(("suggestions", _)) => {
            let suggestions = selector.suggest_optimizations();
            println!("💡 Optimization Suggestions:\n");
            for (i, suggestion) in suggestions.iter().enumerate() {
                println!("{}. {}", i + 1, suggestion);
            }
            
            if suggestions.is_empty() {
                println!("Your configuration looks good! 🎉");
            }
        }
        
        Some(("test", sub_matches)) => {
            let provider = sub_matches.get_one::<String>("provider").unwrap();
            let task = sub_matches.get_one::<String>("task");
            
            let (model, reason) = selector.select_model(provider, task.map(|s| s.as_str()));
            
            println!("🧪 Model Selection Test:");
            println!("  Provider: {}", provider);
            if let Some(task) = task {
                println!("  Task: {}", task);
            }
            println!("  Selected Model: {}", model);
            println!("  Reason: {}", reason);
        }
        
        _ => {
            println!("Use 'aircher model --help' to see available commands");
        }
    }
    
    Ok(())
}

async fn save_selector_config(selector: &SimpleModelSelector) -> Result<()> {
    // In a real implementation, this would save to a config file
    // For now, just log that we would save
    info!("Would save model selector configuration");
    
    // This could save to ~/.config/aircher/model_config.toml or similar
    // let config_path = get_config_dir()?.join("model_config.toml");
    // let toml_content = toml::to_string_pretty(selector)?;
    // fs::write(config_path, toml_content)?;
    
    Ok(())
}

/// Example of how users might configure their preferences
pub fn show_example_configurations() {
    println!(r#"
📖 Example Model Configurations:

💰 Cost-Optimized Setup:
  aircher model set-provider claude claude-3-5-haiku-20241022
  aircher model set-provider openai gpt-4o-mini
  aircher model set-provider gemini gemini-1.5-flash
  aircher model set-task commit_messages gpt-4o-mini

🏆 Quality-First Setup:
  aircher model set-provider claude claude-3-5-sonnet-20241022
  aircher model set-provider openai gpt-4o
  aircher model set-task code_review claude-3-5-sonnet-20241022

🆓 Privacy/Free Setup:
  aircher model set-provider ollama llama3.3
  aircher model set-task code_generation codellama
  aircher model set-task summaries qwen2.5

🎯 Balanced Setup:
  aircher model set-provider claude claude-3-5-sonnet-20241022  # Best reasoning
  aircher model set-provider openai gpt-4o-mini                # Cost-effective
  aircher model set-task commit_messages gpt-4o-mini           # Cheap for simple tasks
  aircher model set-task code_review claude-3-5-sonnet-20241022 # Quality for critical tasks
"#);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_config_command() {
        let cmd = model_config_command();
        assert!(cmd.get_subcommands().any(|sc| sc.get_name() == "list"));
        assert!(cmd.get_subcommands().any(|sc| sc.get_name() == "set-provider"));
        assert!(cmd.get_subcommands().any(|sc| sc.get_name() == "set-task"));
    }
}