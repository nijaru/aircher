use anyhow::Result;
use clap::{Args, Subcommand};
use std::env;

use crate::config::{ConfigManager, ConfigScope, ConfigPaths};

#[derive(Debug, Clone, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConfigCommand {
    /// Show current configuration
    Show,
    /// Get a specific configuration value
    Get {
        /// Configuration key in dot notation (e.g., "ui.theme")
        key: String,
    },
    /// Set a configuration value
    Set {
        /// Configuration key in dot notation (e.g., "ui.theme")
        key: String,
        /// Value to set
        value: String,
        /// Save to local config instead of global
        #[arg(long)]
        local: bool,
    },
    /// Show configuration hierarchy status
    Status,
    /// Create a sample configuration file
    Init {
        /// Create local config instead of global
        #[arg(long)]
        local: bool,
        /// Force overwrite existing config
        #[arg(long)]
        force: bool,
    },
    /// Edit configuration file in $EDITOR
    Edit {
        /// Edit local config instead of global
        #[arg(long)]
        local: bool,
    },
}

pub async fn handle_config_command(args: ConfigArgs, config: &mut ConfigManager) -> Result<()> {
    match args.command {
        ConfigCommand::Show => show_config(config).await,
        ConfigCommand::Get { key } => get_config_value(config, &key).await,
        ConfigCommand::Set { key, value, local } => set_config_value(config, &key, &value, local).await,
        ConfigCommand::Status => show_config_status().await,
        ConfigCommand::Init { local, force } => init_config(config, local, force).await,
        ConfigCommand::Edit { local } => edit_config(local).await,
    }
}

async fn show_config(config: &ConfigManager) -> Result<()> {
    let toml_content = toml::to_string_pretty(config)?;
    println!("{}", toml_content);
    Ok(())
}

async fn get_config_value(config: &ConfigManager, key: &str) -> Result<()> {
    // For now, implement basic key lookups
    let parts: Vec<&str> = key.split('.').collect();
    
    match parts.as_slice() {
        ["global", "default_provider"] => println!("{}", config.global.default_provider),
        ["global", "default_model"] => println!("{}", config.global.default_model),
        ["global", "default_host"] => println!("{}", config.global.default_host),
        ["global", "max_context_tokens"] => println!("{}", config.global.max_context_tokens),
        ["global", "data_directory"] => println!("{}", config.global.data_directory.display()),
        ["ui", "theme"] => println!("{}", config.ui.theme),
        ["ui", "enable_mouse"] => println!("{}", config.ui.enable_mouse),
        ["ui", "refresh_rate_ms"] => println!("{}", config.ui.refresh_rate_ms),
        ["ui", "show_token_count"] => println!("{}", config.ui.show_token_count),
        ["ui", "show_cost_estimate"] => println!("{}", config.ui.show_cost_estimate),
        ["intelligence", "enable_project_analysis"] => println!("{}", config.intelligence.enable_project_analysis),
        ["intelligence", "enable_file_scoring"] => println!("{}", config.intelligence.enable_file_scoring),
        ["intelligence", "enable_context_optimization"] => println!("{}", config.intelligence.enable_context_optimization),
        ["intelligence", "file_scan_depth"] => println!("{}", config.intelligence.file_scan_depth),
        ["intelligence", "relevance_threshold"] => println!("{}", config.intelligence.relevance_threshold),
        ["providers", provider, "name"] => {
            if let Some(p) = config.providers.get(*provider) {
                println!("{}", p.name);
            } else {
                println!("Provider '{}' not found", provider);
            }
        }
        ["providers", provider, "api_key_env"] => {
            if let Some(p) = config.providers.get(*provider) {
                println!("{}", p.api_key_env);
            } else {
                println!("Provider '{}' not found", provider);
            }
        }
        ["providers", provider, "base_url"] => {
            if let Some(p) = config.providers.get(*provider) {
                println!("{}", p.base_url);
            } else {
                println!("Provider '{}' not found", provider);
            }
        }
        _ => {
            println!("Unknown config key: {}", key);
            println!("Available keys:");
            println!("  global.default_provider");
            println!("  global.default_model");
            println!("  global.default_host");
            println!("  global.max_context_tokens");
            println!("  global.data_directory");
            println!("  ui.theme");
            println!("  ui.enable_mouse");
            println!("  ui.refresh_rate_ms");
            println!("  ui.show_token_count");
            println!("  ui.show_cost_estimate");
            println!("  intelligence.enable_project_analysis");
            println!("  intelligence.enable_file_scoring");
            println!("  intelligence.enable_context_optimization");
            println!("  intelligence.file_scan_depth");
            println!("  intelligence.relevance_threshold");
            println!("  providers.<provider_name>.name");
            println!("  providers.<provider_name>.api_key_env");
            println!("  providers.<provider_name>.base_url");
        }
    }
    
    Ok(())
}

async fn set_config_value(config: &mut ConfigManager, key: &str, value: &str, local: bool) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();
    
    match parts.as_slice() {
        ["global", "default_provider"] => config.global.default_provider = value.to_string(),
        ["global", "default_model"] => config.global.default_model = value.to_string(),
        ["global", "default_host"] => config.global.default_host = value.to_string(),
        ["global", "max_context_tokens"] => {
            config.global.max_context_tokens = value.parse()
                .map_err(|_| anyhow::anyhow!("Invalid number for max_context_tokens"))?;
        }
        ["ui", "theme"] => config.ui.theme = value.to_string(),
        ["ui", "enable_mouse"] => {
            config.ui.enable_mouse = value.parse()
                .map_err(|_| anyhow::anyhow!("Invalid boolean for enable_mouse"))?;
        }
        ["ui", "refresh_rate_ms"] => {
            config.ui.refresh_rate_ms = value.parse()
                .map_err(|_| anyhow::anyhow!("Invalid number for refresh_rate_ms"))?;
        }
        ["ui", "show_token_count"] => {
            config.ui.show_token_count = value.parse()
                .map_err(|_| anyhow::anyhow!("Invalid boolean for show_token_count"))?;
        }
        ["ui", "show_cost_estimate"] => {
            config.ui.show_cost_estimate = value.parse()
                .map_err(|_| anyhow::anyhow!("Invalid boolean for show_cost_estimate"))?;
        }
        ["intelligence", "enable_project_analysis"] => {
            config.intelligence.enable_project_analysis = value.parse()
                .map_err(|_| anyhow::anyhow!("Invalid boolean for enable_project_analysis"))?;
        }
        ["intelligence", "enable_file_scoring"] => {
            config.intelligence.enable_file_scoring = value.parse()
                .map_err(|_| anyhow::anyhow!("Invalid boolean for enable_file_scoring"))?;
        }
        ["intelligence", "enable_context_optimization"] => {
            config.intelligence.enable_context_optimization = value.parse()
                .map_err(|_| anyhow::anyhow!("Invalid boolean for enable_context_optimization"))?;
        }
        ["intelligence", "file_scan_depth"] => {
            config.intelligence.file_scan_depth = value.parse()
                .map_err(|_| anyhow::anyhow!("Invalid number for file_scan_depth"))?;
        }
        ["intelligence", "relevance_threshold"] => {
            config.intelligence.relevance_threshold = value.parse()
                .map_err(|_| anyhow::anyhow!("Invalid number for relevance_threshold"))?;
        }
        ["providers", provider, "api_key_env"] => {
            if let Some(p) = config.providers.get_mut(*provider) {
                p.api_key_env = value.to_string();
            } else {
                return Err(anyhow::anyhow!("Provider '{}' not found", provider));
            }
        }
        ["providers", provider, "base_url"] => {
            if let Some(p) = config.providers.get_mut(*provider) {
                p.base_url = value.to_string();
            } else {
                return Err(anyhow::anyhow!("Provider '{}' not found", provider));
            }
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown or unsupported config key: {}", key));
        }
    }

    // Save the updated config
    let scope = if local { ConfigScope::Local } else { ConfigScope::Global };
    config.save_with_scope(scope).await?;

    println!("✅ Configuration updated: {} = {}", key, value);
    println!("💾 Saved to {} config", if local { "local" } else { "global" });
    
    Ok(())
}

async fn show_config_status() -> Result<()> {
    let paths = ConfigManager::get_config_paths()?;
    
    println!("📁 Aircher Configuration Hierarchy");
    println!();
    
    // Hardcoded defaults
    println!("🔧 1. Hardcoded Defaults: Built-in (always available)");
    
    // Global config
    println!("🌐 2. Global Config: {}", paths.global.display());
    if paths.global.exists() {
        println!("   ✅ File exists");
    } else {
        println!("   ❌ File not found (using defaults)");
    }
    
    // Local config
    if let Some(local_path) = &paths.local {
        println!("🏠 3. Local Config: {}", local_path.display());
        if local_path.exists() {
            println!("   ✅ File exists");
        } else {
            println!("   ❌ File not found (using global/defaults)");
        }
    } else {
        println!("🏠 3. Local Config: No .aircher/ directory found in current path");
    }
    
    // Environment variables
    println!("🌍 4. Environment Variables:");
    let env_vars = [
        "AIRCHER_DEFAULT_PROVIDER",
        "AIRCHER_DEFAULT_MODEL", 
        "AIRCHER_DEFAULT_HOST",
        "AIRCHER_MAX_CONTEXT_TOKENS",
        "AIRCHER_BUDGET_LIMIT",
        "AIRCHER_DATA_DIR",
        "AIRCHER_UI_THEME",
        "AIRCHER_UI_ENABLE_MOUSE",
        "AIRCHER_UI_REFRESH_RATE_MS",
        "ANTHROPIC_API_KEY",
        "OPENAI_API_KEY",
        "GOOGLE_API_KEY",
        "OPENROUTER_API_KEY",
    ];
    
    for var in env_vars {
        if env::var(var).is_ok() {
            println!("   ✅ {}: set", var);
        } else {
            println!("   ❌ {}: not set", var);
        }
    }
    
    println!();
    println!("📋 Configuration is loaded in this order:");
    println!("   Hardcoded defaults → Global config → Local config → Environment variables");
    println!();
    println!("💡 Use `aircher config init` to create a sample configuration file");
    println!("💡 Use `aircher config edit` to edit configurations");
    
    Ok(())
}

async fn init_config(config: &ConfigManager, local: bool, force: bool) -> Result<()> {
    let scope = if local { ConfigScope::Local } else { ConfigScope::Global };
    let paths = ConfigManager::get_config_paths()?;
    
    let target_path = if local {
        if let Some(local_path) = &paths.local {
            local_path.clone()
        } else {
            // Create in current directory
            env::current_dir()?.join(".aircher").join("config.toml")
        }
    } else {
        paths.global.clone()
    };
    
    if target_path.exists() && !force {
        println!("❌ Configuration file already exists: {}", target_path.display());
        println!("💡 Use --force to overwrite");
        return Ok(());
    }
    
    // Create sample config with current values
    config.save_with_scope(scope).await?;
    
    println!("✅ Created {} configuration file: {}", 
        if local { "local" } else { "global" }, 
        target_path.display()
    );
    
    println!();
    println!("📝 Edit the file to customize your configuration:");
    println!("   - Set API keys in the [providers] section");
    println!("   - Adjust UI preferences in the [ui] section");
    println!("   - Configure intelligence features in the [intelligence] section");
    println!();
    println!("💡 Use `aircher config edit{}` to edit the file", 
        if local { " --local" } else { "" }
    );
    
    Ok(())
}

async fn edit_config(local: bool) -> Result<()> {
    let paths = ConfigManager::get_config_paths()?;
    
    let target_path = if local {
        if let Some(local_path) = &paths.local {
            local_path.clone()
        } else {
            return Err(anyhow::anyhow!("No local config found. Use `aircher config init --local` first."));
        }
    } else {
        paths.global.clone()
    };
    
    if !target_path.exists() {
        println!("❌ Configuration file not found: {}", target_path.display());
        println!("💡 Use `aircher config init{}` to create it first", 
            if local { " --local" } else { "" }
        );
        return Ok(());
    }
    
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });
    
    println!("🖊️  Opening config file in {}: {}", editor, target_path.display());
    
    let status = std::process::Command::new(&editor)
        .arg(&target_path)
        .status()?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("Editor exited with error"));
    }
    
    println!("✅ Configuration file edited successfully");
    println!("💡 Changes will take effect on next startup");
    
    Ok(())
}