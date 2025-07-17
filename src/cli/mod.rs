use anyhow::Result;
use clap::{Arg, Command};
use std::env;
use tracing::{error, info};
use toml;

use crate::commands::search::{SearchArgs, handle_search_command};
use crate::commands::embedding::{EmbeddingArgs, handle_embedding_command};
use crate::config::{ConfigManager, toml_config::ArcherConfig};
use crate::providers::{ChatRequest, Message, ProviderManager};
use crate::sessions::{SessionFilter, ExportFormat, SessionManager, MessageRole};
use crate::storage::DatabaseManager;
use crate::ui::TuiManager;

mod interactive;
use interactive::InteractiveSession;

pub struct CliApp {
    config: ConfigManager,
    providers: Option<ProviderManager>,
}

impl CliApp {
    pub async fn new() -> Result<Self> {
        let config = ConfigManager::load().await?;

        Ok(Self {
            config,
            providers: None,
        })
    }

    async fn get_providers(&mut self) -> Result<&ProviderManager> {
        if self.providers.is_none() {
            let provider_manager = ProviderManager::new(&self.config).await?;
            self.providers = Some(provider_manager);
        }
        Ok(self.providers.as_ref().unwrap())
    }

    pub async fn run(&mut self, args: Vec<String>) -> Result<()> {
        let matches = Command::new("aircher")
            .version(env!("CARGO_PKG_VERSION"))
            .about("Advanced AI terminal assistant")
            .author("Nick Russo <nijaru7@gmail.com>")
            .arg(
                Arg::new("message")
                    .help("Message to send to AI")
                    .value_name("MESSAGE")
                    .required(false)
                    .index(1),
            )
            .arg(
                Arg::new("model")
                    .long("model")
                    .short('m')
                    .help("Model to use")
                    .value_name("MODEL")
                    .default_value("claude-3-5-sonnet-20241022"),
            )
            .arg(
                Arg::new("provider")
                    .long("provider")
                    .short('p')
                    .help("Provider to use")
                    .value_name("PROVIDER")
                    .default_value("claude"),
            )
            .arg(
                Arg::new("max-tokens")
                    .long("max-tokens")
                    .help("Maximum tokens to generate")
                    .value_name("TOKENS")
                    .value_parser(clap::value_parser!(u32)),
            )
            .arg(
                Arg::new("temperature")
                    .long("temperature")
                    .help("Sampling temperature (0.0-1.0)")
                    .value_name("TEMP")
                    .value_parser(clap::value_parser!(f32)),
            )
            .arg(
                Arg::new("tui")
                    .long("tui")
                    .help("Launch TUI interface")
                    .action(clap::ArgAction::SetTrue),
            )
            .subcommand(
                Command::new("config")
                    .about("Configuration management")
                    .subcommand(
                        Command::new("show")
                            .about("Show all configuration")
                    )
                    .subcommand(
                        Command::new("get")
                            .about("Get specific configuration value")
                            .arg(
                                Arg::new("key")
                                    .help("Configuration key")
                                    .required(true)
                                    .index(1),
                            )
                    )
                    .subcommand(
                        Command::new("set")
                            .about("Set configuration value")
                            .arg(
                                Arg::new("key")
                                    .help("Configuration key")
                                    .required(true)
                                    .index(1),
                            )
                            .arg(
                                Arg::new("value")
                                    .help("Configuration value")
                                    .required(true)
                                    .index(2),
                            )
                    )
                    .subcommand(
                        Command::new("unset")
                            .about("Remove configuration value")
                            .arg(
                                Arg::new("key")
                                    .help("Configuration key")
                                    .required(true)
                                    .index(1),
                            )
                    )
                    .subcommand(
                        Command::new("edit")
                            .about("Open config file in $EDITOR")
                    )
                    .subcommand(
                        Command::new("reset")
                            .about("Reset to default configuration")
                    )
                    .subcommand(
                        Command::new("validate")
                            .about("Check configuration validity")
                    )
                    .subcommand(
                        Command::new("path")
                            .about("Show config file path")
                    )
            )
            .subcommand(
                Command::new("search")
                    .about("Semantic code search")
                    .subcommand(
                        Command::new("index")
                            .about("Index directory for semantic search")
                            .arg(
                                Arg::new("path")
                                    .help("Directory path to index")
                                    .value_name("PATH")
                                    .default_value(".")
                                    .index(1),
                            )
                            .arg(
                                Arg::new("force")
                                    .long("force")
                                    .help("Force re-indexing")
                                    .action(clap::ArgAction::SetTrue),
                            )
                    )
                    .subcommand(
                        Command::new("query")
                            .about("Perform semantic code search")
                            .arg(
                                Arg::new("query")
                                    .help("Search query")
                                    .required(true)
                                    .index(1),
                            )
                            .arg(
                                Arg::new("limit")
                                    .short('l')
                                    .long("limit")
                                    .help("Maximum number of results")
                                    .value_name("NUM")
                                    .default_value("10")
                                    .value_parser(clap::value_parser!(usize)),
                            )
                            .arg(
                                Arg::new("path")
                                    .short('p')
                                    .long("path")
                                    .help("Directory to search in")
                                    .value_name("PATH")
                                    .default_value("."),
                            )
                    )
                    .subcommand(
                        Command::new("stats")
                            .about("Show search index statistics")
                            .arg(
                                Arg::new("path")
                                    .help("Directory path")
                                    .value_name("PATH")
                                    .default_value(".")
                                    .index(1),
                            )
                    )
            )
            .subcommand(
                Command::new("model")
                    .about("Model management")
                    .subcommand(
                        Command::new("current")
                            .about("Show currently configured models")
                    )
                    .subcommand(
                        Command::new("list")
                            .about("List all available models")
                            .arg(
                                Arg::new("provider")
                                    .long("provider")
                                    .help("Filter by provider")
                                    .value_name("NAME"),
                            )
                            .arg(
                                Arg::new("type")
                                    .long("type")
                                    .help("Filter by type (chat/embedding)")
                                    .value_name("TYPE"),
                            )
                    )
                    .subcommand(
                        Command::new("test")
                            .about("Test model connections")
                            .arg(
                                Arg::new("provider")
                                    .long("provider")
                                    .help("Test specific provider")
                                    .value_name("NAME"),
                            )
                    )
                    .subcommand(
                        Command::new("providers")
                            .about("Show available providers and their models")
                    )
            )
            .subcommand(
                Command::new("embedding")
                    .about("Legacy embedding management (use 'model' instead)")
                    .subcommand(
                        Command::new("status")
                            .about("Show embedding model status")
                    )
                    .subcommand(
                        Command::new("setup")
                            .about("Setup embedding models")
                            .arg(
                                Arg::new("interactive")
                                    .long("interactive")
                                    .help("Use interactive setup")
                                    .action(clap::ArgAction::SetTrue),
                            )
                            .arg(
                                Arg::new("force")
                                    .long("force")
                                    .help("Force re-download")
                                    .action(clap::ArgAction::SetTrue),
                            )
                    )
                    .subcommand(
                        Command::new("list")
                            .about("List available embedding models")
                    )
                    .subcommand(
                        Command::new("test")
                            .about("Test embedding functionality")
                            .arg(
                                Arg::new("text")
                                    .help("Sample text to embed")
                                    .index(1),
                            )
                    )
            )
            .subcommand(
                Command::new("session")
                    .about("Session management commands")
                    .subcommand(
                        Command::new("list")
                            .about("List all sessions")
                            .arg(
                                Arg::new("provider")
                                    .long("provider")
                                    .help("Filter by provider")
                                    .value_name("PROVIDER"),
                            )
                    )
                    .subcommand(
                        Command::new("new")
                            .about("Create a new session")
                            .arg(
                                Arg::new("title")
                                    .help("Session title")
                                    .required(true)
                                    .index(1),
                            )
                            .arg(
                                Arg::new("provider")
                                    .long("provider")
                                    .short('p')
                                    .help("Provider to use")
                                    .default_value("claude"),
                            )
                            .arg(
                                Arg::new("model")
                                    .long("model")
                                    .short('m')
                                    .help("Model to use")
                                    .default_value("claude-3-5-sonnet-20241022"),
                            )
                    )
                    .subcommand(
                        Command::new("load")
                            .about("Load an existing session")
                            .arg(
                                Arg::new("id")
                                    .help("Session ID")
                                    .required(true)
                                    .index(1),
                            )
                    )
                    .subcommand(
                        Command::new("export")
                            .about("Export a session")
                            .arg(
                                Arg::new("id")
                                    .help("Session ID")
                                    .required(true)
                                    .index(1),
                            )
                            .arg(
                                Arg::new("format")
                                    .long("format")
                                    .short('f')
                                    .help("Export format")
                                    .value_name("FORMAT")
                                    .default_value("json")
                                    .value_parser(["json", "markdown", "csv", "plain"]),
                            )
                    )
            )
            .get_matches_from(args);

        // Handle subcommands first
        if let Some(config_matches) = matches.subcommand_matches("config") {
            return self.handle_config_commands(config_matches).await;
        }
        
        if let Some(search_matches) = matches.subcommand_matches("search") {
            return self.handle_search_commands(search_matches).await;
        }
        
        if let Some(model_matches) = matches.subcommand_matches("model") {
            return self.handle_model_commands(model_matches).await;
        }
        
        if let Some(embedding_matches) = matches.subcommand_matches("embedding") {
            return self.handle_embedding_commands(embedding_matches).await;
        }
        
        if let Some(session_matches) = matches.subcommand_matches("session") {
            return self.handle_session_commands(session_matches).await;
        }

        let message = matches.get_one::<String>("message");

        match message {
            Some(msg) => {
                // One-shot mode
                if let Err(e) = self.handle_one_shot(msg, &matches).await {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
                Ok(())
            }
            None => {
                // Default to TUI mode (modern CLI pattern)
                // Only use interactive mode if explicitly disabled
                if let Err(e) = self.handle_tui(&matches).await {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
                Ok(())
            }
        }
    }

    async fn handle_one_shot(&mut self, message: &str, matches: &clap::ArgMatches) -> Result<()> {
        let provider_name = matches.get_one::<String>("provider").unwrap();
        let model = matches.get_one::<String>("model").unwrap();

        info!(
            "One-shot request: provider={}, model={}",
            provider_name, model
        );

        // Check if we have API key first
        self.check_api_key(provider_name)?;

        // Get provider
        let providers = self.get_providers().await?;
        let provider = providers
            .get_provider_or_host(provider_name)
            .ok_or_else(|| {
                anyhow::anyhow!("Provider '{}' not found or not configured", provider_name)
            })?;

        // Create chat request
        let messages = vec![Message::user(message.to_string())];
        let mut request = ChatRequest::new(messages, model.clone());

        if let Some(max_tokens) = matches.get_one::<u32>("max-tokens") {
            request.max_tokens = Some(*max_tokens);
        }
        if let Some(temperature) = matches.get_one::<f32>("temperature") {
            request.temperature = Some(*temperature);
        }

        // Send request
        match provider.chat(&request).await {
            Ok(response) => {
                println!("{}", response.content);

                // Show usage info if available
                if let Some(cost) = response.cost {
                    eprintln!("\n💰 Cost: ${:.4} ({} tokens)", cost, response.tokens_used);
                } else {
                    eprintln!("\n📊 Tokens: {}", response.tokens_used);
                }

                Ok(())
            }
            Err(e) => {
                error!("Chat request failed: {}", e);
                eprintln!("❌ Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    async fn handle_interactive(&mut self, matches: &clap::ArgMatches) -> Result<()> {
        let provider_name = matches.get_one::<String>("provider").unwrap();
        let model = matches.get_one::<String>("model").unwrap();

        info!(
            "Starting interactive mode: provider={}, model={}",
            provider_name, model
        );

        // Check if we have API key first
        self.check_api_key(provider_name)?;

        // Get providers
        let providers = self.get_providers().await?;

        // Verify provider exists
        if providers.get_provider_or_host(provider_name).is_none() {
            return Err(anyhow::anyhow!(
                "Provider '{}' not found or not configured",
                provider_name
            ));
        }

        // Create and run interactive session
        let mut session = InteractiveSession::new(
            provider_name.clone(),
            model.clone(),
            matches.get_one::<u32>("max-tokens").copied(),
            matches.get_one::<f32>("temperature").copied(),
        );

        session.run(providers).await
    }

    async fn handle_tui(&mut self, matches: &clap::ArgMatches) -> Result<()> {
        let provider_name = matches.get_one::<String>("provider").unwrap();
        let model = matches.get_one::<String>("model").unwrap();

        info!(
            "Starting TUI mode: provider={}, model={}",
            provider_name, model
        );

        // Check if we have API key first
        self.check_api_key(provider_name)?;

        // Clone config first
        let config = self.config.clone();

        // Get providers
        let providers = self.get_providers().await?;

        // Create TUI manager
        let mut tui_manager = TuiManager::new(&config, providers).await?;

        // Verify provider exists
        if providers.get_provider_or_host(provider_name).is_none() {
            return Err(anyhow::anyhow!(
                "Provider '{}' not found or not configured",
                provider_name
            ));
        }

        // Run TUI
        tui_manager.run(providers).await
    }

    fn check_api_key(&self, provider_name: &str) -> Result<()> {
        let env_var = match provider_name {
            "claude" => "ANTHROPIC_API_KEY",
            "gemini" => "GOOGLE_API_KEY",
            "openrouter" => "OPENROUTER_API_KEY",
            _ => return Err(anyhow::anyhow!("Unknown provider: {}", provider_name)),
        };

        if env::var(env_var).is_err() {
            return Err(anyhow::anyhow!(
                "Missing API key for {}: Please set {} environment variable\n\
                Example: export {}=your_api_key_here",
                provider_name,
                env_var,
                env_var
            ));
        }

        Ok(())
    }


    async fn handle_session_commands(&mut self, matches: &clap::ArgMatches) -> Result<()> {

        // Create session manager directly
        let database_manager = DatabaseManager::new(&self.config).await?;
        let session_manager = SessionManager::new(&database_manager).await?;

        match matches.subcommand() {
            Some(("list", sub_matches)) => {
                let mut filter = SessionFilter::default();
                if let Some(provider) = sub_matches.get_one::<String>("provider") {
                    filter.provider = Some(provider.clone());
                }

                let sessions = session_manager.search_sessions(&filter, Some(50)).await?;
                
                if sessions.is_empty() {
                    println!("No sessions found.");
                    return Ok(());
                }

                println!("{:<36} {:<20} {:<15} {:<20} {:<8} {:<10}", 
                    "ID", "Title", "Provider", "Model", "Messages", "Cost");
                println!("{}", "-".repeat(120));
                
                for session in sessions {
                    println!("{:<36} {:<20} {:<15} {:<20} {:<8} ${:<10.4}", 
                        session.id,
                        if session.title.len() > 20 { 
                            format!("{}...", &session.title[..17])
                        } else { 
                            session.title 
                        },
                        session.provider,
                        if session.model.len() > 20 { 
                            format!("{}...", &session.model[..17])
                        } else { 
                            session.model 
                        },
                        session.message_count,
                        session.total_cost
                    );
                }
            }
            
            Some(("new", sub_matches)) => {
                let title = sub_matches.get_one::<String>("title").unwrap();
                let provider = sub_matches.get_one::<String>("provider").unwrap();
                let model = sub_matches.get_one::<String>("model").unwrap();

                let session = session_manager.create_session(
                    title.clone(),
                    provider.clone(),
                    model.clone(),
                    None,
                    vec![]
                ).await?;

                println!("✅ Created new session:");
                println!("   ID: {}", session.id);
                println!("   Title: {}", session.title);
                println!("   Provider: {}", session.provider);
                println!("   Model: {}", session.model);
            }
            
            Some(("load", sub_matches)) => {
                let session_id = sub_matches.get_one::<String>("id").unwrap();
                
                if let Some(session) = session_manager.load_session(session_id).await? {
                    println!("✅ Loaded session:");
                    println!("   ID: {}", session.id);
                    println!("   Title: {}", session.title);
                    println!("   Provider: {}", session.provider);
                    println!("   Model: {}", session.model);
                    println!("   Messages: {}", session.message_count);
                    println!("   Total cost: ${:.4}", session.total_cost);
                    
                    // Load and display recent messages
                    let messages = session_manager.load_session_messages(session_id).await?;
                    if !messages.is_empty() {
                        println!("\n📝 Recent messages:");
                        for (_i, msg) in messages.iter().rev().take(5).enumerate() {
                            let role = match msg.role {
                                MessageRole::User => "User",
                                MessageRole::Assistant => "Assistant",
                                MessageRole::System => "System",
                                MessageRole::Tool => "Tool",
                            };
                            let content = if msg.content.len() > 100 {
                                format!("{}...", &msg.content[..97])
                            } else {
                                msg.content.clone()
                            };
                            println!("   {}: {}", role, content);
                        }
                    }
                } else {
                    eprintln!("❌ Session not found: {}", session_id);
                    std::process::exit(1);
                }
            }
            
            Some(("export", sub_matches)) => {
                let session_id = sub_matches.get_one::<String>("id").unwrap();
                let format_str = sub_matches.get_one::<String>("format").unwrap();
                
                let format = match format_str.as_str() {
                    "json" => ExportFormat::Json,
                    "markdown" => ExportFormat::Markdown,
                    "csv" => ExportFormat::Csv,
                    "plain" => ExportFormat::Plain,
                    _ => {
                        eprintln!("❌ Unknown format: {}", format_str);
                        std::process::exit(1);
                    }
                };

                let exported_data = session_manager.export_session(session_id, format).await?;
                println!("{}", exported_data);
            }
            
            _ => {
                eprintln!("❌ Unknown session subcommand");
                std::process::exit(1);
            }
        }

        Ok(())
    }
    
    async fn handle_config_commands(&mut self, matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("show", _)) => {
                let config = ArcherConfig::load()?;
                println!("{}", toml::to_string_pretty(&config)?);
            }
            
            Some(("get", sub_matches)) => {
                let key = sub_matches.get_one::<String>("key").unwrap();
                let config = ArcherConfig::load()?;
                if let Some(value) = config.get_value(key)? {
                    println!("{}", value);
                } else {
                    eprintln!("❌ Configuration key '{}' not found", key);
                    std::process::exit(1);
                }
            }
            
            Some(("set", sub_matches)) => {
                let key = sub_matches.get_one::<String>("key").unwrap();
                let value = sub_matches.get_one::<String>("value").unwrap();
                let mut config = ArcherConfig::load()?;
                config.set_value(key, value)?;
                config.save()?;
                println!("✅ Configuration updated: {} = {}", key, value);
            }
            
            Some(("unset", sub_matches)) => {
                let key = sub_matches.get_one::<String>("key").unwrap();
                let mut config = ArcherConfig::load()?;
                config.unset_value(key)?;
                config.save()?;
                println!("✅ Configuration key '{}' removed", key);
            }
            
            Some(("edit", _)) => {
                let config_path = ArcherConfig::config_file_path()?;
                let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
                
                // Ensure config file exists
                let _config = ArcherConfig::load()?;
                
                // Open in editor
                let output = std::process::Command::new(&editor)
                    .arg(&config_path)
                    .status()?;
                    
                if output.success() {
                    println!("✅ Configuration file edited: {}", config_path.display());
                } else {
                    eprintln!("❌ Editor exited with error");
                    std::process::exit(1);
                }
            }
            
            Some(("reset", _)) => {
                let config = ArcherConfig::default();
                config.save()?;
                println!("✅ Configuration reset to defaults");
            }
            
            Some(("validate", _)) => {
                match ArcherConfig::load() {
                    Ok(_) => println!("✅ Configuration is valid"),
                    Err(e) => {
                        eprintln!("❌ Configuration validation failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            
            Some(("path", _)) => {
                let config_path = ArcherConfig::config_file_path()?;
                println!("{}", config_path.display());
            }
            
            _ => {
                eprintln!("❌ Unknown config subcommand");
                std::process::exit(1);
            }
        }
        
        Ok(())
    }
    
    async fn handle_search_commands(&mut self, matches: &clap::ArgMatches) -> Result<()> {
        use std::path::PathBuf;
        use crate::commands::search::SearchCommand;
        
        let search_command = match matches.subcommand() {
            Some(("index", sub_matches)) => {
                let path = sub_matches.get_one::<String>("path")
                    .map(|s| PathBuf::from(s))
                    .unwrap_or_else(|| PathBuf::from("."));
                let force = sub_matches.get_flag("force");
                SearchCommand::Index { path, force }
            }
            
            Some(("query", sub_matches)) => {
                let query = sub_matches.get_one::<String>("query").unwrap().clone();
                let limit = *sub_matches.get_one::<usize>("limit").unwrap();
                let path = sub_matches.get_one::<String>("path")
                    .map(|s| PathBuf::from(s))
                    .unwrap_or_else(|| PathBuf::from("."));
                SearchCommand::Query { query, limit, path }
            }
            
            Some(("stats", sub_matches)) => {
                let path = sub_matches.get_one::<String>("path")
                    .map(|s| PathBuf::from(s))
                    .unwrap_or_else(|| PathBuf::from("."));
                SearchCommand::Stats { path }
            }
            
            _ => {
                eprintln!("❌ Unknown search subcommand");
                std::process::exit(1);
            }
        };
        
        let search_args = SearchArgs { command: search_command };
        handle_search_command(search_args).await
    }
    
    async fn handle_model_commands(&mut self, matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("current", _)) => {
                let config = ArcherConfig::load()?;
                println!("🤖 Current Model Configuration:\n");
                
                // Show chat models
                println!("Chat Models:");
                if let Some(claude_config) = config.providers.get("claude") {
                    println!("  Claude: {}", claude_config.default_model);
                }
                if let Some(openai_config) = config.providers.get("openai") {
                    println!("  OpenAI: {}", openai_config.default_model);
                }
                if let Some(gemini_config) = config.providers.get("gemini") {
                    println!("  Gemini: {}", gemini_config.default_model);
                }
                if let Some(ollama_config) = config.providers.get("ollama") {
                    println!("  Ollama: {}", ollama_config.default_model);
                }
                
                // Show embedding model
                println!("\nEmbedding Model:");
                println!("  {}: {}", config.embedding.provider, config.embedding.model);
            }
            
            Some(("list", sub_matches)) => {
                let provider_filter = sub_matches.get_one::<String>("provider");
                let type_filter = sub_matches.get_one::<String>("type");
                
                println!("📋 Available Models:\n");
                
                if provider_filter.is_none() || provider_filter == Some(&"claude".to_string()) {
                    if type_filter.is_none() || type_filter == Some(&"chat".to_string()) {
                        println!("Claude (Chat):");
                        println!("  claude-3-5-sonnet-20241022");
                        println!("  claude-3-5-haiku-20241022");
                        println!("  claude-3-opus-20240229");
                        println!();
                    }
                }
                
                if provider_filter.is_none() || provider_filter == Some(&"ollama".to_string()) {
                    if type_filter.is_none() || type_filter == Some(&"embedding".to_string()) {
                        println!("Ollama (Embedding):");
                        println!("  nomic-embed-text (recommended)");
                        println!("  mxbai-embed-large");
                        println!("  all-MiniLM-L6-v2");
                        println!();
                    }
                }
                
                println!("💡 Use 'aircher config set providers.<provider>.default_model <model>' to configure");
            }
            
            Some(("test", sub_matches)) => {
                let provider_filter = sub_matches.get_one::<String>("provider");
                
                println!("🧪 Testing model connections...");
                
                let _providers = self.get_providers().await?;
                
                if provider_filter.is_none() || provider_filter == Some(&"claude".to_string()) {
                    // For now, just check if the provider exists in config
                    let config = ArcherConfig::load()?;
                    if let Some(claude_config) = config.providers.get("claude") {
                        if claude_config.api_key.is_some() {
                            println!("  Claude: ✅ API key configured");
                        } else {
                            println!("  Claude: ❌ No API key found");
                        }
                    } else {
                        println!("  Claude: ❌ Not configured");
                    }
                }
                
                println!("\n💡 Check API keys with: aircher config show");
            }
            
            Some(("providers", _)) => {
                println!("🏭 Available Providers:\n");
                
                println!("Chat Providers:");
                println!("  • Claude (Anthropic) - GPT-4 class models");
                println!("  • OpenAI - GPT models");
                println!("  • Gemini (Google) - Gemini models");
                println!("  • Ollama - Local models");
                println!("  • OpenRouter - Access to multiple providers");
                
                println!("\nEmbedding Providers:");
                println!("  • Ollama - Local embedding models (recommended)");
                println!("  • OpenAI - text-embedding-3-small/large");
                
                println!("\n⚙️  Configure with: aircher config set providers.<provider>.api_key <key>");
            }
            
            _ => {
                eprintln!("❌ Unknown model subcommand");
                std::process::exit(1);
            }
        }
        
        Ok(())
    }
    
    async fn handle_embedding_commands(&mut self, matches: &clap::ArgMatches) -> Result<()> {
        use crate::commands::embedding::EmbeddingCommand;
        
        let embedding_command = match matches.subcommand() {
            Some(("status", _)) => EmbeddingCommand::Status,
            
            Some(("setup", sub_matches)) => {
                let force = sub_matches.get_flag("force");
                let interactive = sub_matches.get_flag("interactive");
                EmbeddingCommand::Setup { force, interactive }
            }
            
            Some(("list", _)) => EmbeddingCommand::List,
            
            Some(("test", sub_matches)) => {
                let text = sub_matches.get_one::<String>("text").cloned();
                EmbeddingCommand::Test { text }
            }
            
            _ => {
                eprintln!("❌ Unknown embedding subcommand");
                std::process::exit(1);
            }
        };
        
        let embedding_args = EmbeddingArgs { command: embedding_command };
        handle_embedding_command(embedding_args).await
    }
}
