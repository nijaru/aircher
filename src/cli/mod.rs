use anyhow::Result;
use clap::{Arg, Command};
use std::env;
use tracing::{error, info};

use crate::config::ConfigManager;
use crate::providers::{ChatRequest, Message, ProviderManager};
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
            self.providers = Some(ProviderManager::new(&self.config).await?);
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
            .get_matches_from(args);

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

        // Create TUI manager first
        let mut tui_manager = TuiManager::new(&self.config).await?;

        // Get providers
        let providers = self.get_providers().await?;

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

    pub async fn available_providers(&mut self) -> Result<Vec<String>> {
        let providers = self.get_providers().await?;
        Ok(providers.list_all())
    }
}
