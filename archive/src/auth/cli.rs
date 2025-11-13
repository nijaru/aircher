use anyhow::{Context, Result};
use clap::{Arg, Command};
use std::io::{self, Write};
use tracing::{info, warn};

use super::{AuthManager, AuthStatus};
use super::oauth::OAuthHandler;
use crate::config::ConfigManager;
use crate::providers::ProviderManager;

#[derive(Debug)]
pub enum AuthCommand {
    Login { provider: String, api_key: Option<String> },
    LoginOAuth { provider: String },
    Logout { provider: String },
    Status { provider: Option<String> },
    Test { provider: String },
    List,
    Clear,
}

impl AuthCommand {
    pub fn parse_from_args(args: &[String]) -> Result<Self> {
        let matches = Command::new("auth")
            .about("Manage API authentication for providers")
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
                Command::new("login-oauth")
                    .about("Authenticate using OAuth (Claude Max subscription)")
                    .arg(Arg::new("provider")
                        .help("Provider name (anthropic for Claude Max)")
                        .required(true)
                        .index(1))
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
            .try_get_matches_from(args)?;

        match matches.subcommand() {
            Some(("login", sub_matches)) => {
                let provider = sub_matches.get_one::<String>("provider").unwrap().clone();
                let api_key = sub_matches.get_one::<String>("key").cloned();
                Ok(AuthCommand::Login { provider, api_key })
            }
            Some(("login-oauth", sub_matches)) => {
                let provider = sub_matches.get_one::<String>("provider").unwrap().clone();
                Ok(AuthCommand::LoginOAuth { provider })
            }
            Some(("logout", sub_matches)) => {
                let provider = sub_matches.get_one::<String>("provider").unwrap().clone();
                Ok(AuthCommand::Logout { provider })
            }
            Some(("status", sub_matches)) => {
                let provider = sub_matches.get_one::<String>("provider").cloned();
                Ok(AuthCommand::Status { provider })
            }
            Some(("test", sub_matches)) => {
                let provider = sub_matches.get_one::<String>("provider").unwrap().clone();
                Ok(AuthCommand::Test { provider })
            }
            Some(("list", _)) => {
                Ok(AuthCommand::List)
            }
            Some(("clear", _)) => {
                Ok(AuthCommand::Clear)
            }
            _ => {
                anyhow::bail!("No auth subcommand provided. Use 'auth --help' for usage.")
            }
        }
    }

    pub async fn execute(
        &self,
        config: &ConfigManager,
        auth_manager: &AuthManager,
        _provider_manager: Option<&ProviderManager>,
    ) -> Result<()> {
        match self {
            AuthCommand::Login { provider, api_key } => {
                self.handle_login(provider, api_key.as_deref(), config, auth_manager).await
            }
            AuthCommand::LoginOAuth { provider } => {
                self.handle_login_oauth(provider, auth_manager).await
            }
            AuthCommand::Logout { provider } => {
                self.handle_logout(provider, auth_manager).await
            }
            AuthCommand::Status { provider } => {
                self.handle_status(provider.as_deref(), config, auth_manager).await
            }
            AuthCommand::Test { provider } => {
                self.handle_test(provider, config, auth_manager).await
            }
            AuthCommand::List => {
                self.handle_list(config, auth_manager).await
            }
            AuthCommand::Clear => {
                self.handle_clear(auth_manager).await
            }
        }
    }

    async fn handle_login(
        &self,
        provider: &str,
        api_key: Option<&str>,
        config: &ConfigManager,
        auth_manager: &AuthManager,
    ) -> Result<()> {
        // Validate provider exists
        let provider_config = config.get_provider(provider)
            .context(format!("Provider '{}' not found in configuration", provider))?;

        // Check if provider needs an API key
        if provider_config.api_key_env.is_empty() {
            println!("ℹ️  Provider '{}' doesn't require an API key (local provider)", provider);
            return Ok(());
        }

        // Get API key (from argument or prompt)
        let key = match api_key {
            Some(k) => k.to_string(),
            None => {
                print!("Enter API key for {}: ", provider);
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                input.trim().to_string()
            }
        };

        if key.is_empty() {
            anyhow::bail!("API key cannot be empty");
        }

        // Validate key format (basic checks)
        if let Err(msg) = Self::validate_api_key_format(provider, &key) {
            warn!("⚠️  Warning: {}", msg);
            print!("Continue anyway? (y/N): ");
            io::stdout().flush()?;

            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm)?;
            if !confirm.trim().to_lowercase().starts_with('y') {
                println!("Cancelled.");
                return Ok(());
            }
        }

        // Store the key
        auth_manager.store_api_key(provider, &key).await
            .context("Failed to store API key")?;

        println!("✓ API key stored for provider '{}'", provider);
        println!("💡 You can test it with: aircher auth test {}", provider);

        Ok(())
    }

    async fn handle_login_oauth(
        &self,
        provider: &str,
        auth_manager: &AuthManager,
    ) -> Result<()> {
        // Currently only support Anthropic/Claude Max OAuth
        if provider != "anthropic" && provider != "claude" {
            anyhow::bail!("OAuth login is currently only supported for 'anthropic' (Claude Max subscription)");
        }

        println!("🔐 Starting OAuth login for Claude Max subscription...");

        // Check if in SSH session
        if OAuthHandler::is_ssh_session() {
            println!("⚠️  SSH session detected. You'll need to manually open the URL in a browser.");
        }

        // Create OAuth handler
        let oauth_handler = OAuthHandler::new_anthropic_pro();

        // Start auth flow (opens browser with URL, returns state and code_verifier for PKCE)
        let (auth_url, state, code_verifier) = oauth_handler.start_auth_flow().await?;

        println!("\n📋 If the browser didn't open automatically, visit:");
        println!("   {}\n", auth_url);

        // Start callback server and wait for OAuth code
        println!("⏳ Waiting for authentication in browser...");
        let auth_code = oauth_handler.start_callback_server(&state).await
            .context("Failed to receive OAuth callback")?;

        info!("✓ Received authorization code from OAuth callback");

        // Exchange code for access token (with PKCE code_verifier and state)
        println!("🔄 Exchanging authorization code for access token...");
        let access_token = oauth_handler.exchange_code_for_token(&auth_code, &code_verifier, &state).await
            .context("Failed to exchange code for token")?;

        // Store OAuth token (using a special key to differentiate from API key)
        auth_manager.store_oauth_token("anthropic", &access_token).await
            .context("Failed to store OAuth token")?;

        println!("✓ OAuth token stored successfully!");
        println!("💡 You can now use Claude Max subscription models");
        println!("   Test with: aircher auth test anthropic");

        Ok(())
    }

    async fn handle_logout(
        &self,
        provider: &str,
        auth_manager: &AuthManager,
    ) -> Result<()> {
        auth_manager.remove_api_key(provider).await
            .context("Failed to remove API key")?;

        println!("✓ API key removed for provider '{}'", provider);
        Ok(())
    }

    async fn handle_status(
        &self,
        specific_provider: Option<&str>,
        config: &ConfigManager,
        auth_manager: &AuthManager,
    ) -> Result<()> {
        if let Some(provider) = specific_provider {
            let info = auth_manager.get_provider_status(provider, config).await;
            Self::print_provider_status(&info);
        } else {
            let summary = auth_manager.get_auth_summary(config).await;
            println!("Authentication Status:\n{}", summary);
        }
        Ok(())
    }

    async fn handle_test(
        &self,
        provider: &str,
        config: &ConfigManager,
        auth_manager: &AuthManager,
    ) -> Result<()> {
        println!("Testing authentication for '{}'...", provider);

        let result = auth_manager.test_provider_auth(provider, config).await?;

        match result.status {
            AuthStatus::Authenticated => {
                println!("✓ Authentication successful for '{}'", provider);
                if let Some(usage) = &result.usage_info {
                    Self::print_usage_info(usage);
                }
            }
            _ => {
                println!("✗ Authentication failed for '{}'", provider);
                if let Some(error) = &result.error_message {
                    println!("Error: {}", error);
                }
            }
        }

        Ok(())
    }

    async fn handle_list(
        &self,
        config: &ConfigManager,
        auth_manager: &AuthManager,
    ) -> Result<()> {
        println!("Available Providers:");

        for (provider_name, provider_config) in &config.providers {
            let needs_auth = !provider_config.api_key_env.is_empty();
            let auth_info = auth_manager.get_provider_status(provider_name, config).await;

            let status_icon = match auth_info.status {
                AuthStatus::Authenticated => "✓",
                AuthStatus::NotConfigured => if needs_auth { "○" } else { "✓" },
                _ => "✗",
            };

            let auth_status = if needs_auth {
                match auth_info.status {
                    AuthStatus::Authenticated => "authenticated",
                    AuthStatus::NotConfigured => "not configured",
                    _ => "needs setup",
                }
            } else {
                "local (no auth needed)"
            };

            println!("  {} {} - {} models ({})",
                status_icon,
                provider_name,
                provider_config.models.len(),
                auth_status
            );
        }

        Ok(())
    }

    async fn handle_clear(&self, auth_manager: &AuthManager) -> Result<()> {
        print!("⚠️  This will remove ALL stored API keys. Are you sure? (y/N): ");
        io::stdout().flush()?;

        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;

        if confirm.trim().to_lowercase().starts_with('y') {
            auth_manager.clear_all().await?;
            println!("✓ All API keys cleared");
        } else {
            println!("Cancelled.");
        }

        Ok(())
    }

    fn validate_api_key_format(provider: &str, key: &str) -> Result<(), String> {
        match provider {
            "anthropic" | "claude" => {
                if !key.starts_with("sk-ant-") {
                    return Err("Anthropic API keys should start with 'sk-ant-'".to_string());
                }
            }
            "openai" => {
                if !key.starts_with("sk-") {
                    return Err("OpenAI API keys should start with 'sk-'".to_string());
                }
            }
            "gemini" | "google" => {
                if key.len() < 20 {
                    return Err("Google API keys are typically longer".to_string());
                }
            }
            _ => {
                // No specific validation for other providers
            }
        }
        Ok(())
    }

    fn print_provider_status(info: &super::ProviderAuthInfo) {
        let status_icon = match info.status {
            AuthStatus::Authenticated => "✓",
            AuthStatus::NotConfigured => "○",
            AuthStatus::Invalid => "✗",
            AuthStatus::Expired => "⚠",
            AuthStatus::RateLimited => "⚠",
            AuthStatus::NetworkError => "⚠",
        };

        println!("{} Provider: {}", status_icon, info.provider);

        if let Some(key) = &info.masked_key {
            println!("  API Key: {}", key);
        }

        if let Some(validated) = info.last_validated {
            println!("  Last Validated: {}", validated.format("%Y-%m-%d %H:%M:%S UTC"));
        }

        if let Some(error) = &info.error_message {
            println!("  Error: {}", error);
        }

        if let Some(usage) = &info.usage_info {
            Self::print_usage_info(usage);
        }
    }

    fn print_usage_info(usage: &super::ProviderUsageInfo) {
        println!("  Usage Information:");

        if let (Some(used), Some(limit)) = (usage.requests_used, usage.requests_limit) {
            let percentage = (used as f64 / limit as f64 * 100.0) as u32;
            println!("    Requests: {}/{} ({}%)", used, limit, percentage);
        }

        if let (Some(used), Some(limit)) = (usage.tokens_used, usage.tokens_limit) {
            let percentage = (used as f64 / limit as f64 * 100.0) as u32;
            println!("    Tokens: {}/{} ({}%)", used, limit, percentage);
        }

        if let (Some(used), Some(limit)) = (usage.cost_used, usage.cost_limit) {
            let percentage = (used / limit * 100.0) as u32;
            println!("    Cost: ${:.2}/${:.2} ({}%)", used, limit, percentage);
        }

        if let Some(reset) = usage.reset_time {
            println!("    Resets: {}", reset.format("%Y-%m-%d %H:%M:%S UTC"));
        }
    }
}
