use anyhow::Result;
use clap::{Arg, Command};
use std::env;
use tracing::{error, info};

use crate::commands::search::{SearchArgs, handle_search_command};
use crate::commands::embedding::{EmbeddingArgs, handle_embedding_command};
use crate::commands::model::{ModelArgs, ModelCommand, TaskTypeArg, handle_model_command};
use crate::commands::config::{ConfigArgs, ConfigCommand, handle_config_command};
use crate::config::ConfigManager;
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
                            .about("Show current configuration")
                    )
                    .subcommand(
                        Command::new("get")
                            .about("Get specific configuration value")
                            .arg(
                                Arg::new("key")
                                    .help("Configuration key in dot notation")
                                    .required(true)
                                    .index(1),
                            )
                    )
                    .subcommand(
                        Command::new("set")
                            .about("Set configuration value")
                            .arg(
                                Arg::new("key")
                                    .help("Configuration key in dot notation")
                                    .required(true)
                                    .index(1),
                            )
                            .arg(
                                Arg::new("value")
                                    .help("Configuration value")
                                    .required(true)
                                    .index(2),
                            )
                            .arg(
                                Arg::new("local")
                                    .long("local")
                                    .help("Save to local config instead of global")
                                    .action(clap::ArgAction::SetTrue)
                            )
                    )
                    .subcommand(
                        Command::new("status")
                            .about("Show configuration hierarchy status")
                    )
                    .subcommand(
                        Command::new("init")
                            .about("Create a sample configuration file")
                            .arg(
                                Arg::new("local")
                                    .long("local")
                                    .help("Create local config instead of global")
                                    .action(clap::ArgAction::SetTrue)
                            )
                            .arg(
                                Arg::new("force")
                                    .long("force")
                                    .help("Force overwrite existing config")
                                    .action(clap::ArgAction::SetTrue)
                            )
                    )
                    .subcommand(
                        Command::new("edit")
                            .about("Edit configuration file in $EDITOR")
                            .arg(
                                Arg::new("local")
                                    .long("local")
                                    .help("Edit local config instead of global")
                                    .action(clap::ArgAction::SetTrue)
                            )
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
                            .about("Perform semantic code search with advanced filtering")
                            .arg(
                                Arg::new("query")
                                    .help("Search query (e.g., \"error handling patterns\", \"database connection\")")
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
                            .arg(
                                Arg::new("file_types")
                                    .long("file-types")
                                    .help("Filter by file types (e.g., \"rs,py,js\" or \"rust,python,javascript\")")
                                    .value_delimiter(',')
                                    .action(clap::ArgAction::Append),
                            )
                            .arg(
                                Arg::new("languages")
                                    .long("languages")
                                    .help("Filter by programming languages (e.g., \"rust,python\")")
                                    .value_delimiter(',')
                                    .action(clap::ArgAction::Append),
                            )
                            .arg(
                                Arg::new("scope")
                                    .long("scope")
                                    .help("Filter by code scope (e.g., \"functions,classes,modules\")")
                                    .value_delimiter(',')
                                    .action(clap::ArgAction::Append),
                            )
                            .arg(
                                Arg::new("chunk_types")
                                    .long("chunk-types")
                                    .help("Filter by chunk types (e.g., \"function,class,module,comment\")")
                                    .value_delimiter(',')
                                    .action(clap::ArgAction::Append),
                            )
                            .arg(
                                Arg::new("min_similarity")
                                    .long("min-similarity")
                                    .help("Minimum similarity threshold (0.0-1.0)")
                                    .value_parser(clap::value_parser!(f32)),
                            )
                            .arg(
                                Arg::new("max_similarity")
                                    .long("max-similarity")
                                    .help("Maximum similarity threshold (0.0-1.0)")
                                    .value_parser(clap::value_parser!(f32)),
                            )
                            .arg(
                                Arg::new("exclude")
                                    .long("exclude")
                                    .help("Exclude patterns (e.g., \"test,bench,example\")")
                                    .value_delimiter(',')
                                    .action(clap::ArgAction::Append),
                            )
                            .arg(
                                Arg::new("include")
                                    .long("include")
                                    .help("Include only patterns (e.g., \"src,lib\")")
                                    .value_delimiter(',')
                                    .action(clap::ArgAction::Append),
                            )
                            .arg(
                                Arg::new("debug_filters")
                                    .long("debug-filters")
                                    .help("Show debug information about filtering")
                                    .action(clap::ArgAction::SetTrue),
                            )
                            .arg(
                                Arg::new("preset")
                                    .long("preset")
                                    .help("Use a saved search preset")
                                    .value_name("NAME"),
                            )
                            .arg(
                                Arg::new("save_preset")
                                    .long("save-preset")
                                    .help("Save current search as a preset")
                                    .value_name("NAME"),
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
                        Command::new("set")
                            .about("Set model directly (bypass TUI)")
                            .arg(
                                Arg::new("model")
                                    .help("Model name (e.g., claude-3-5-sonnet)")
                                    .required(true)
                                    .index(1),
                            )
                            .arg(
                                Arg::new("provider")
                                    .short('p')
                                    .long("provider")
                                    .help("Provider (optional, will auto-detect if not specified)")
                                    .value_name("PROVIDER"),
                            )
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
                    )
                    .subcommand(
                        Command::new("select")
                            .about("Interactive model selection TUI")
                    )
                    .subcommand(
                        Command::new("test")
                            .about("Test current model with a simple request")
                            .arg(
                                Arg::new("message")
                                    .short('m')
                                    .long("message")
                                    .help("Optional custom message to test with")
                                    .value_name("MESSAGE"),
                            )
                    )
                    .subcommand(
                        Command::new("auto-select")
                            .about("Intelligent auto-selection based on task type")
                            .arg(
                                Arg::new("task")
                                    .short('t')
                                    .long("task")
                                    .help("Task type for intelligent selection")
                                    .value_name("TYPE")
                                    .value_parser(["code-search", "documentation-search", "conceptual-search", "cross-language-search", "large-codebase-search", "quick-search", "high-accuracy-search"]),
                            )
                            .arg(
                                Arg::new("prefer-local")
                                    .long("prefer-local")
                                    .help("Prefer local models (embedded/Ollama)")
                                    .action(clap::ArgAction::SetTrue),
                            )
                            .arg(
                                Arg::new("prefer-fast")
                                    .long("prefer-fast")
                                    .help("Prefer fast models over accurate ones")
                                    .action(clap::ArgAction::SetTrue),
                            )
                            .arg(
                                Arg::new("prefer-accurate")
                                    .long("prefer-accurate")
                                    .help("Prefer accurate models over fast ones")
                                    .action(clap::ArgAction::SetTrue),
                            )
                    )
                    .subcommand(
                        Command::new("providers")
                            .about("Show available providers and their status")
                    )
            )
            .subcommand(
                Command::new("embedding")
                    .about("Embedding model management (default: list with current marked)")
                    .subcommand(
                        Command::new("list")
                            .about("List all available embedding models with current selection marked")
                    )
                    .subcommand(
                        Command::new("set")
                            .about("Set embedding model ('auto' for intelligent selection, or specific model name)")
                            .arg(
                                Arg::new("model")
                                    .help("Model name or 'auto' for intelligent selection")
                                    .required(true)
                                    .index(1),
                            )
                    )
                    .subcommand(
                        Command::new("verify")
                            .about("Verify current embedding model is working")
                            .arg(
                                Arg::new("text")
                                    .help("Optional sample text to verify with")
                                    .index(1),
                            )
                    )
                    .subcommand(
                        Command::new("update")
                            .about("Update embedding models to latest versions")
                            .arg(
                                Arg::new("check-only")
                                    .long("check-only")
                                    .help("Check for updates without installing")
                                    .action(clap::ArgAction::SetTrue),
                            )
                    )
                    .subcommand(
                        Command::new("clean")
                            .about("Clean up unused models and stale indices")
                            .arg(
                                Arg::new("models")
                                    .long("models")
                                    .help("Remove unused model versions")
                                    .action(clap::ArgAction::SetTrue),
                            )
                            .arg(
                                Arg::new("indices")
                                    .long("indices")
                                    .help("Remove stale search indices")
                                    .action(clap::ArgAction::SetTrue),
                            )
                            .arg(
                                Arg::new("all")
                                    .long("all")
                                    .help("Remove everything (nuclear option)")
                                    .action(clap::ArgAction::SetTrue),
                            )
                    )
                    .subcommand(
                        Command::new("status")
                            .about("Show storage usage and cleanup recommendations")
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
        let config_args = match matches.subcommand() {
            Some(("show", _)) => ConfigArgs {
                command: ConfigCommand::Show,
            },
            Some(("get", sub_matches)) => ConfigArgs {
                command: ConfigCommand::Get {
                    key: sub_matches.get_one::<String>("key").unwrap().clone(),
                },
            },
            Some(("set", sub_matches)) => ConfigArgs {
                command: ConfigCommand::Set {
                    key: sub_matches.get_one::<String>("key").unwrap().clone(),
                    value: sub_matches.get_one::<String>("value").unwrap().clone(),
                    local: sub_matches.get_flag("local"),
                },
            },
            Some(("status", _)) => ConfigArgs {
                command: ConfigCommand::Status,
            },
            Some(("init", sub_matches)) => ConfigArgs {
                command: ConfigCommand::Init {
                    local: sub_matches.get_flag("local"),
                    force: sub_matches.get_flag("force"),
                },
            },
            Some(("edit", sub_matches)) => ConfigArgs {
                command: ConfigCommand::Edit {
                    local: sub_matches.get_flag("local"),
                },
            },
            _ => {
                eprintln!("❌ Unknown config command");
                std::process::exit(1);
            }
        };

        handle_config_command(config_args, &mut self.config).await
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
                
                // Extract new filter options with defaults
                let file_types = sub_matches.get_many::<String>("file_types")
                    .map(|v| v.cloned().collect());
                let languages = sub_matches.get_many::<String>("languages")
                    .map(|v| v.cloned().collect());
                let scope = sub_matches.get_many::<String>("scope")
                    .map(|v| v.cloned().collect());
                let chunk_types = sub_matches.get_many::<String>("chunk_types")
                    .map(|v| v.cloned().collect());
                let min_similarity = sub_matches.get_one::<f32>("min_similarity").copied();
                let max_similarity = sub_matches.get_one::<f32>("max_similarity").copied();
                let exclude = sub_matches.get_many::<String>("exclude")
                    .map(|v| v.cloned().collect());
                let include = sub_matches.get_many::<String>("include")
                    .map(|v| v.cloned().collect());
                let debug_filters = sub_matches.get_flag("debug_filters");
                let preset = sub_matches.get_one::<String>("preset").cloned();
                let save_preset = sub_matches.get_one::<String>("save_preset").cloned();
                
                SearchCommand::Query { 
                    query, 
                    limit, 
                    path,
                    file_types,
                    languages,
                    scope,
                    chunk_types,
                    min_similarity,
                    max_similarity,
                    exclude,
                    include,
                    debug_filters,
                    preset,
                    save_preset
                }
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
        let model_command = match matches.subcommand() {
            Some(("current", _)) => ModelCommand::Current,
            
            Some(("set", sub_matches)) => {
                let model = sub_matches.get_one::<String>("model").unwrap().clone();
                let provider = sub_matches.get_one::<String>("provider").cloned();
                ModelCommand::Set { model, provider }
            }
            
            Some(("list", sub_matches)) => {
                let provider = sub_matches.get_one::<String>("provider").cloned();
                ModelCommand::List { provider }
            }
            
            Some(("select", _)) => ModelCommand::Select,
            
            Some(("test", sub_matches)) => {
                let message = sub_matches.get_one::<String>("message").cloned();
                ModelCommand::Test { message }
            }
            
            Some(("auto-select", sub_matches)) => {
                let task = sub_matches.get_one::<String>("task").map(|t| match t.as_str() {
                    "code-search" => TaskTypeArg::CodeSearch,
                    "documentation-search" => TaskTypeArg::DocumentationSearch,
                    "conceptual-search" => TaskTypeArg::ConceptualSearch,
                    "cross-language-search" => TaskTypeArg::CrossLanguageSearch,
                    "large-codebase-search" => TaskTypeArg::LargeCodebaseSearch,
                    "quick-search" => TaskTypeArg::QuickSearch,
                    "high-accuracy-search" => TaskTypeArg::HighAccuracySearch,
                    _ => TaskTypeArg::CodeSearch,
                });
                let prefer_local = sub_matches.get_flag("prefer-local");
                let prefer_fast = sub_matches.get_flag("prefer-fast");
                let prefer_accurate = sub_matches.get_flag("prefer-accurate");
                
                ModelCommand::AutoSelect { 
                    task, 
                    prefer_local, 
                    prefer_fast, 
                    prefer_accurate 
                }
            }
            
            Some(("providers", _)) => ModelCommand::Providers,
            
            _ => {
                eprintln!("❌ Unknown model subcommand");
                std::process::exit(1);
            }
        };
        
        let model_args = ModelArgs { command: model_command };
        // Create providers first to avoid borrowing issues
        let _providers = if self.providers.is_none() {
            let provider_manager = ProviderManager::new(&self.config).await?;
            self.providers = Some(provider_manager);
        };
        let providers = self.providers.as_ref().unwrap();
        handle_model_command(model_args, &mut self.config, providers).await
    }
    
    async fn handle_embedding_commands(&mut self, matches: &clap::ArgMatches) -> Result<()> {
        use crate::commands::embedding::EmbeddingCommand;
        
        let embedding_command = match matches.subcommand() {
            Some(("list", _)) => Some(EmbeddingCommand::List),
            
            Some(("set", sub_matches)) => {
                let model = sub_matches.get_one::<String>("model").unwrap().clone();
                Some(EmbeddingCommand::Set { model })
            }
            
            Some(("verify", sub_matches)) => {
                let text = sub_matches.get_one::<String>("text").cloned();
                Some(EmbeddingCommand::Verify { text })
            }
            
            Some(("update", sub_matches)) => {
                let check_only = sub_matches.get_flag("check-only");
                Some(EmbeddingCommand::Update { check_only })
            }
            
            Some(("clean", sub_matches)) => {
                let models = sub_matches.get_flag("models");
                let indices = sub_matches.get_flag("indices");
                let all = sub_matches.get_flag("all");
                Some(EmbeddingCommand::Clean { models, indices, all })
            }
            
            Some(("status", _)) => Some(EmbeddingCommand::Status),
            
            None => Some(EmbeddingCommand::List), // Default: show list with current marked
            
            _ => {
                eprintln!("❌ Unknown embedding subcommand");
                std::process::exit(1);
            }
        };
        
        let embedding_args = EmbeddingArgs { command: embedding_command };
        handle_embedding_command(embedding_args).await
    }
}
