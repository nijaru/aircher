use anyhow::Result;
use clap::{Arg, Command};
use std::env;
use std::rc::Rc;
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


pub struct CliApp {
    config: ConfigManager,
    providers: Option<Rc<ProviderManager>>,
}

impl CliApp {
    pub async fn new() -> Result<Self> {
        let config = ConfigManager::load().await?;

        Ok(Self {
            config,
            providers: None,
        })
    }

    async fn get_providers(&mut self) -> Result<Rc<ProviderManager>> {
        if self.providers.is_none() {
            let provider_manager = ProviderManager::new(&self.config).await?;
            self.providers = Some(Rc::new(provider_manager));
        }
        Ok(self.providers.as_ref().unwrap().clone())
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
                Command::new("auth")
                    .about("Authentication management")
                    .subcommand(
                        Command::new("login")
                            .about("Add or update API key for a provider")
                            .arg(Arg::new("provider")
                                .help("Provider name (anthropic, openai, gemini, etc.)")
                                .required(true)
                                .index(1))
                            .arg(Arg::new("key")
                                .help("API key (if not provided, will prompt)")
                                .long("key")
                                .short('k')
                                .value_name("API_KEY"))
                    )
                    .subcommand(
                        Command::new("logout")
                            .about("Remove API key for a provider")
                            .arg(Arg::new("provider")
                                .help("Provider name")
                                .required(true)
                                .index(1))
                    )
                    .subcommand(
                        Command::new("status")
                            .about("Show authentication status")
                            .arg(Arg::new("provider")
                                .help("Specific provider to check (optional)")
                                .index(1))
                    )
                    .subcommand(
                        Command::new("test")
                            .about("Test API key validity for a provider")
                            .arg(Arg::new("provider")
                                .help("Provider name")
                                .required(true)
                                .index(1))
                    )
                    .subcommand(
                        Command::new("list")
                            .about("List all configured providers")
                    )
                    .subcommand(
                        Command::new("clear")
                            .about("Clear all stored API keys (use with caution)")
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
            .subcommand(
                Command::new("mcp")
                    .about("Model Context Protocol (MCP) client management")
                    .subcommand(
                        Command::new("list")
                            .about("List configured MCP servers")
                            .arg(
                                Arg::new("verbose")
                                    .long("verbose")
                                    .help("Show detailed server information")
                                    .action(clap::ArgAction::SetTrue),
                            )
                    )
                    .subcommand(
                        Command::new("add")
                            .about("Add a new MCP server configuration")
                            .arg(
                                Arg::new("name")
                                    .help("Server name (unique identifier)")
                                    .required(true)
                                    .index(1),
                            )
                            .arg(
                                Arg::new("server_type")
                                    .help("Server type")
                                    .required(true)
                                    .index(2)
                                    .value_parser(["stdio", "http"]),
                            )
                            .arg(
                                Arg::new("command")
                                    .long("command")
                                    .help("Command to execute (for stdio servers)")
                                    .value_name("CMD"),
                            )
                            .arg(
                                Arg::new("args")
                                    .long("args")
                                    .help("Command arguments (for stdio servers)")
                                    .value_delimiter(' ')
                                    .action(clap::ArgAction::Append),
                            )
                            .arg(
                                Arg::new("url")
                                    .long("url")
                                    .help("Server URL (for HTTP servers)")
                                    .value_name("URL"),
                            )
                            .arg(
                                Arg::new("api_key")
                                    .long("api-key")
                                    .help("API key for authentication")
                                    .value_name("KEY"),
                            )
                            .arg(
                                Arg::new("description")
                                    .short('d')
                                    .long("description")
                                    .help("Server description")
                                    .value_name("DESC"),
                            )
                            .arg(
                                Arg::new("tags")
                                    .long("tags")
                                    .help("Tags for categorization")
                                    .value_delimiter(',')
                                    .action(clap::ArgAction::Append),
                            )
                    )
                    .subcommand(
                        Command::new("remove")
                            .about("Remove an MCP server configuration")
                            .arg(
                                Arg::new("name")
                                    .help("Server name to remove")
                                    .required(true)
                                    .index(1),
                            )
                            .arg(
                                Arg::new("force")
                                    .long("force")
                                    .help("Force removal without confirmation")
                                    .action(clap::ArgAction::SetTrue),
                            )
                    )
                    .subcommand(
                        Command::new("connect")
                            .about("Connect to an MCP server")
                            .arg(
                                Arg::new("name")
                                    .help("Server name to connect to")
                                    .required(true)
                                    .index(1),
                            )
                            .arg(
                                Arg::new("timeout")
                                    .long("timeout")
                                    .help("Connection timeout in seconds")
                                    .value_name("SECS")
                                    .default_value("30")
                                    .value_parser(clap::value_parser!(u64)),
                            )
                    )
                    .subcommand(
                        Command::new("disconnect")
                            .about("Disconnect from an MCP server")
                            .arg(
                                Arg::new("name")
                                    .help("Server name to disconnect from")
                                    .required(true)
                                    .index(1),
                            )
                    )
                    .subcommand(
                        Command::new("status")
                            .about("Show server status")
                            .arg(
                                Arg::new("name")
                                    .help("Server name (show all if not specified)")
                                    .index(1),
                            )
                    )
                    .subcommand(
                        Command::new("tools")
                            .about("List available tools from connected servers")
                            .arg(
                                Arg::new("server")
                                    .long("server")
                                    .help("Filter by server name")
                                    .value_name("NAME"),
                            )
                            .arg(
                                Arg::new("tag")
                                    .long("tag")
                                    .help("Filter by tag")
                                    .value_name("TAG"),
                            )
                    )
                    .subcommand(
                        Command::new("resources")
                            .about("List available resources from connected servers")
                            .arg(
                                Arg::new("server")
                                    .long("server")
                                    .help("Filter by server name")
                                    .value_name("NAME"),
                            )
                    )
                    .subcommand(
                        Command::new("call")
                            .about("Execute a tool from an MCP server")
                            .arg(
                                Arg::new("tool")
                                    .help("Tool name (format: server_name.tool_name or just tool_name)")
                                    .required(true)
                                    .index(1),
                            )
                            .arg(
                                Arg::new("args")
                                    .long("args")
                                    .help("Tool arguments as JSON")
                                    .value_name("JSON"),
                            )
                            .arg(
                                Arg::new("pretty")
                                    .long("pretty")
                                    .help("Pretty print output")
                                    .action(clap::ArgAction::SetTrue),
                            )
                    )
                    .subcommand(
                        Command::new("get")
                            .about("Get a resource from an MCP server")
                            .arg(
                                Arg::new("uri")
                                    .help("Resource URI")
                                    .required(true)
                                    .index(1),
                            )
                            .arg(
                                Arg::new("server")
                                    .long("server")
                                    .help("Server name (auto-detect if not specified)")
                                    .value_name("NAME"),
                            )
                    )
            )
            .subcommand(
                Command::new("benchmark")
                    .about("Vector search performance benchmarking")
                    .subcommand(
                        Command::new("vector")
                            .about("Benchmark the current vector search implementation")
                    )
                    .subcommand(
                        Command::new("performance")
                            .about("Run comprehensive performance benchmark")
                    )
                    .subcommand(
                        Command::new("tune")
                            .about("Tune hnswlib-rs parameters for optimal performance")
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
        
        if let Some(auth_matches) = matches.subcommand_matches("auth") {
            return self.handle_auth_commands(auth_matches).await;
        }
        
        if let Some(embedding_matches) = matches.subcommand_matches("embedding") {
            return self.handle_embedding_commands(embedding_matches).await;
        }
        
        if let Some(mcp_matches) = matches.subcommand_matches("mcp") {
            return self.handle_mcp_commands(mcp_matches).await;
        }
        
        if let Some(session_matches) = matches.subcommand_matches("session") {
            return self.handle_session_commands(session_matches).await;
        }
        
        if let Some(benchmark_matches) = matches.subcommand_matches("benchmark") {
            return self.handle_benchmark_commands(benchmark_matches).await;
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


    async fn handle_tui(&mut self, matches: &clap::ArgMatches) -> Result<()> {
        let provider_name = matches.get_one::<String>("provider").unwrap();
        let model = matches.get_one::<String>("model").unwrap();

        info!(
            "Starting TUI mode: provider={}, model={}",
            provider_name, model
        );

        // Clone config first
        let config = self.config.clone();

        // Check API key status (but don't fail immediately)
        let has_api_key = self.check_api_key(provider_name).is_ok();
        
        // Try to get providers (may fail without API keys, but TUI can handle this)
        let providers = if has_api_key {
            match self.get_providers().await {
                Ok(providers) => Some(providers),
                Err(e) => {
                    // Provider initialization failed, likely due to API key issues
                    // Log the error but proceed with demo mode
                    info!("Provider initialization failed (entering demo mode): {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Create TUI manager with optional providers
        let mut tui_manager = TuiManager::new_with_auth_state(&config, providers, provider_name.clone(), model.clone()).await?;

        // Run TUI (it will handle auth setup internally if needed)
        tui_manager.run().await
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
            self.providers = Some(Rc::new(provider_manager));
        };
        let providers = self.providers.as_ref().unwrap();
        handle_model_command(model_args, &mut self.config, providers).await
    }
    
    async fn handle_auth_commands(&mut self, matches: &clap::ArgMatches) -> Result<()> {
        use crate::auth::{AuthManager, cli::AuthCommand};
        
        let mut auth_manager = AuthManager::new()?;
        
        // Collect command args for parsing
        let mut args = vec!["auth".to_string()];
        
        match matches.subcommand() {
            Some(("login", sub_matches)) => {
                args.push("login".to_string());
                args.push(sub_matches.get_one::<String>("provider").unwrap().clone());
                if let Some(key) = sub_matches.get_one::<String>("key") {
                    args.push("--key".to_string());
                    args.push(key.clone());
                }
            }
            Some(("logout", sub_matches)) => {
                args.push("logout".to_string());
                args.push(sub_matches.get_one::<String>("provider").unwrap().clone());
            }
            Some(("status", sub_matches)) => {
                args.push("status".to_string());
                if let Some(provider) = sub_matches.get_one::<String>("provider") {
                    args.push(provider.clone());
                }
            }
            Some(("test", sub_matches)) => {
                args.push("test".to_string());
                args.push(sub_matches.get_one::<String>("provider").unwrap().clone());
            }
            Some(("list", _)) => {
                args.push("list".to_string());
            }
            Some(("clear", _)) => {
                args.push("clear".to_string());
            }
            _ => {
                eprintln!("❌ Unknown auth subcommand");
                std::process::exit(1);
            }
        }
        
        let auth_command = AuthCommand::parse_from_args(&args)?;
        
        // Get provider manager if needed for testing
        let provider_manager = if matches!(auth_command, AuthCommand::Test { .. }) {
            self.get_providers().await.ok()
        } else {
            None
        };
        
        auth_command.execute(
            &self.config, 
            &mut auth_manager, 
            provider_manager.as_deref()
        ).await
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
    
    async fn handle_mcp_commands(&mut self, matches: &clap::ArgMatches) -> Result<()> {
        use crate::commands::mcp::{McpCommand, ServerType, handle_mcp_command};
        
        let mcp_command = match matches.subcommand() {
            Some(("list", sub_matches)) => {
                let verbose = sub_matches.get_flag("verbose");
                Some(McpCommand::List { verbose })
            }
            
            Some(("add", sub_matches)) => {
                let name = sub_matches.get_one::<String>("name").unwrap().clone();
                let server_type_str = sub_matches.get_one::<String>("server_type").unwrap();
                let server_type = match server_type_str.as_str() {
                    "stdio" => ServerType::Stdio,
                    "http" => ServerType::Http,
                    _ => {
                        eprintln!("❌ Invalid server type: {}", server_type_str);
                        std::process::exit(1);
                    }
                };
                let command = sub_matches.get_one::<String>("command").cloned();
                let args = sub_matches.get_many::<String>("args")
                    .map(|v| v.cloned().collect())
                    .or(Some(vec![]));
                let url = sub_matches.get_one::<String>("url").cloned();
                let api_key = sub_matches.get_one::<String>("api_key").cloned();
                let description = sub_matches.get_one::<String>("description").cloned();
                let tags = sub_matches.get_many::<String>("tags")
                    .map(|v| v.cloned().collect())
                    .or(Some(vec![]));
                
                Some(McpCommand::Add {
                    name,
                    server_type,
                    command,
                    args,
                    url,
                    api_key,
                    description,
                    tags,
                })
            }
            
            Some(("remove", sub_matches)) => {
                let name = sub_matches.get_one::<String>("name").unwrap().clone();
                let force = sub_matches.get_flag("force");
                Some(McpCommand::Remove { name, force })
            }
            
            Some(("connect", sub_matches)) => {
                let name = sub_matches.get_one::<String>("name").unwrap().clone();
                let timeout = *sub_matches.get_one::<u64>("timeout").unwrap();
                Some(McpCommand::Connect { name, timeout })
            }
            
            Some(("disconnect", sub_matches)) => {
                let name = sub_matches.get_one::<String>("name").unwrap().clone();
                Some(McpCommand::Disconnect { name })
            }
            
            Some(("status", sub_matches)) => {
                let name = sub_matches.get_one::<String>("name").cloned();
                Some(McpCommand::Status { name })
            }
            
            Some(("tools", sub_matches)) => {
                let server = sub_matches.get_one::<String>("server").cloned();
                let tag = sub_matches.get_one::<String>("tag").cloned();
                Some(McpCommand::Tools { server, tag })
            }
            
            Some(("resources", sub_matches)) => {
                let server = sub_matches.get_one::<String>("server").cloned();
                Some(McpCommand::Resources { server })
            }
            
            Some(("call", sub_matches)) => {
                let tool = sub_matches.get_one::<String>("tool").unwrap().clone();
                let args = sub_matches.get_one::<String>("args").cloned();
                let pretty = sub_matches.get_flag("pretty");
                Some(McpCommand::Call { tool, args, pretty })
            }
            
            Some(("get", sub_matches)) => {
                let uri = sub_matches.get_one::<String>("uri").unwrap().clone();
                let server = sub_matches.get_one::<String>("server").cloned();
                Some(McpCommand::Get { uri, server })
            }
            
            None => Some(McpCommand::List { verbose: false }), // Default: list servers
            
            _ => {
                eprintln!("❌ Unknown MCP subcommand");
                std::process::exit(1);
            }
        };
        
        if let Some(command) = mcp_command {
            let mcp_args = crate::commands::mcp::McpArgs { command };
            handle_mcp_command(mcp_args).await
        } else {
            Ok(())
        }
    }
    
    #[cfg(any(test, feature = "benchmarks"))]
    async fn handle_benchmark_commands(&mut self, matches: &clap::ArgMatches) -> Result<()> {
        use crate::benchmarks::vector_benchmark::benchmark_current_implementation;
        use crate::benchmarks::hnswlib_benchmark::{run_performance_benchmark, tune_hnswlib_parameters};
        
        match matches.subcommand() {
            Some(("vector", _)) => {
                println!("🚀 Starting vector search benchmark...");
                let result = benchmark_current_implementation().await?;
                println!("\n✅ Benchmark completed successfully!");
                println!("{}", result.format_summary());
                Ok(())
            }
            
            Some(("performance", _)) => {
                println!("🏃 Running comprehensive performance benchmark...");
                run_performance_benchmark().await?;
                Ok(())
            }
            
            Some(("tune", _)) => {
                println!("🔧 Running parameter tuning for hnswlib-rs...");
                tune_hnswlib_parameters().await
            }
            
            _ => {
                eprintln!("❌ Unknown benchmark subcommand");
                eprintln!("Available subcommands:");
                eprintln!("  vector      - Benchmark current vector search implementation");
                eprintln!("  performance - Run comprehensive performance benchmark");
                eprintln!("  tune        - Tune hnswlib-rs parameters");
                std::process::exit(1);
            }
        }
    }
    
    #[cfg(not(any(test, feature = "benchmarks")))]
    async fn handle_benchmark_commands(&mut self, _matches: &clap::ArgMatches) -> Result<()> {
        eprintln!("❌ Benchmarking not available. Compile with '--features benchmarks' to enable.");
        std::process::exit(1);
    }
    
}
