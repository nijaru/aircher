use anyhow::Result;
use clap::{Arg, Command};
use std::env;
use tracing::{error, info};

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

        // Handle session subcommands first
        if let Some(session_matches) = matches.subcommand_matches("session") {
            return self.handle_session_commands(session_matches).await;
        }

        let message = matches.get_one::<String>("message");
        let tui_mode = matches.get_flag("tui");

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
                if tui_mode {
                    // TUI mode
                    if let Err(e) = self.handle_tui(&matches).await {
                        eprintln!("❌ Error: {}", e);
                        std::process::exit(1);
                    }
                } else {
                    // Interactive mode
                    if let Err(e) = self.handle_interactive(&matches).await {
                        eprintln!("❌ Error: {}", e);
                        std::process::exit(1);
                    }
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
}
