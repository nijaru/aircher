use anyhow::Result;
use clap::{Args, Subcommand};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    Terminal,
};
use std::io::stdout;
use tracing::{info, debug};

use crate::config::ConfigManager;
use crate::providers::ProviderManager;
use crate::ui::enhanced_selection::{EnhancedSelectionModal, SelectionResult};
use crate::cost::auto_selection::{AutoSelectionEngine, SelectionCriteria, TaskType};
use crate::cost::EmbeddingManager;

#[derive(Debug, Args)]
pub struct ModelArgs {
    #[command(subcommand)]
    pub command: ModelCommand,
}

#[derive(Debug, Subcommand)]
pub enum ModelCommand {
    /// Show current active model
    Current,
    
    /// Set model directly (bypass TUI)
    Set {
        /// Model name (e.g., claude-3-5-sonnet)
        model: String,
        
        /// Provider (optional, will auto-detect if not specified)
        #[arg(short, long)]
        provider: Option<String>,
    },
    
    /// List available models for a provider
    List {
        /// Provider to list models for (optional, shows all if not specified)
        provider: Option<String>,
    },
    
    /// Interactive model selection TUI
    Select,
    
    /// Test current model with a simple request
    Test {
        /// Optional custom message to test with
        #[arg(short, long)]
        message: Option<String>,
    },
    
    /// Intelligent auto-selection based on task type
    AutoSelect {
        /// Task type for intelligent selection
        #[arg(short, long, value_enum)]
        task: Option<TaskTypeArg>,
        
        /// Prefer local models (embedded/Ollama)
        #[arg(long)]
        prefer_local: bool,
        
        /// Prefer fast models over accurate ones
        #[arg(long)]
        prefer_fast: bool,
        
        /// Prefer accurate models over fast ones
        #[arg(long)]
        prefer_accurate: bool,
    },
    
    /// Show available providers and their status
    Providers,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum TaskTypeArg {
    CodeSearch,
    DocumentationSearch,
    ConceptualSearch,
    CrossLanguageSearch,
    LargeCodebaseSearch,
    QuickSearch,
    HighAccuracySearch,
}

impl From<TaskTypeArg> for TaskType {
    fn from(arg: TaskTypeArg) -> Self {
        match arg {
            TaskTypeArg::CodeSearch => TaskType::CodeSearch,
            TaskTypeArg::DocumentationSearch => TaskType::DocumentationSearch,
            TaskTypeArg::ConceptualSearch => TaskType::ConceptualSearch,
            TaskTypeArg::CrossLanguageSearch => TaskType::CrossLanguageSearch,
            TaskTypeArg::LargeCodebaseSearch => TaskType::LargeCodebaseSearch,
            TaskTypeArg::QuickSearch => TaskType::QuickSearch,
            TaskTypeArg::HighAccuracySearch => TaskType::HighAccuracySearch,
        }
    }
}

pub async fn handle_model_command(args: ModelArgs, config: &mut ConfigManager, providers: &ProviderManager) -> Result<()> {
    match args.command {
        ModelCommand::Current => {
            show_current_model(config).await
        },
        ModelCommand::Set { model, provider } => {
            set_model(config, &model, provider.as_deref()).await
        },
        ModelCommand::List { provider } => {
            list_models(config, providers, provider.as_deref()).await
        },
        ModelCommand::Select => {
            interactive_model_selection(config, providers).await
        },
        ModelCommand::Test { message } => {
            test_model(config, providers, message.as_deref()).await
        },
        ModelCommand::AutoSelect { task, prefer_local, prefer_fast, prefer_accurate } => {
            auto_select_model(config, task, prefer_local, prefer_fast, prefer_accurate).await
        },
        ModelCommand::Providers => {
            show_providers(config, providers).await
        },
    }
}

async fn show_current_model(config: &ConfigManager) -> Result<()> {
    println!("Current Configuration:");
    println!("  Provider: {}", config.global.default_provider);
    println!("  Model: {}", config.global.default_model);
    
    println!("  Authenticated: {}", 
            if config.has_api_key(&config.global.default_provider) { "✅ Yes" } else { "❌ No" });
    
    Ok(())
}

async fn set_model(config: &mut ConfigManager, model: &str, provider: Option<&str>) -> Result<()> {
    // If provider is specified, use it; otherwise try to auto-detect
    let target_provider = if let Some(p) = provider {
        p.to_string()
    } else {
        // Simple auto-detection based on model name patterns
        if model.contains("claude") {
            "claude".to_string()
        } else if model.contains("gpt") || model.contains("o1") {
            "openai".to_string()
        } else if model.contains("gemini") {
            "gemini".to_string()
        } else if model.contains("llama") || model.contains("mistral") {
            "ollama".to_string()
        } else {
            // Default to current provider
            config.global.default_provider.clone()
        }
    };
    
    // Update configuration
    config.global.default_provider = target_provider.clone();
    config.global.default_model = model.to_string();
    
    // Save configuration
    config.save().await?;
    
    println!("✅ Model updated:");
    println!("  Provider: {}", target_provider);
    println!("  Model: {}", model);
    
    info!("Model configuration updated: {}:{}", target_provider, model);
    Ok(())
}

async fn list_models(config: &ConfigManager, providers: &ProviderManager, provider_filter: Option<&str>) -> Result<()> {
    println!("Available Models:");
    println!();
    
    for (provider_name, provider_config) in &config.providers {
        // Apply provider filter if specified
        if let Some(filter) = provider_filter {
            if provider_name != filter {
                continue;
            }
        }
        
        println!("📡 {} ({})", provider_name, 
                if config.is_provider_enabled(provider_name) { "enabled" } else { "disabled" });
        
        // Check authentication status
        let auth_status = match provider_name.as_str() {
            "ollama" => "🏠 Local (no auth required)".to_string(),
            _ => if config.has_api_key(provider_name) {
                "✅ Authenticated".to_string()
            } else {
                "❌ Not authenticated".to_string()
            }
        };
        println!("   Status: {}", auth_status);
        
        // List models for this provider
        if let Some(models) = config.get_models_for_provider(provider_name) {
            for model in models {
                let current_marker = if provider_name == &config.global.default_provider 
                                      && model.name == config.global.default_model {
                    " ← current"
                } else {
                    ""
                };
                
                println!("   - {} ({}k context, ${:.2}/${:.2} per 1M){}",
                        model.name,
                        model.context_window / 1000,
                        model.input_cost_per_1m,
                        model.output_cost_per_1m,
                        current_marker);
            }
        } else {
            println!("   - No models configured");
        }
        println!();
    }
    
    Ok(())
}

async fn interactive_model_selection(config: &mut ConfigManager, providers: &ProviderManager) -> Result<()> {
    info!("Starting interactive model selection TUI");
    
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    
    // Initialize enhanced selection modal
    let mut modal = EnhancedSelectionModal::new();
    modal.initialize(providers, config).await?;
    modal.show();
    
    let mut result: Option<SelectionResult> = None;
    
    // Main TUI loop
    loop {
        // Draw the modal
        terminal.draw(|f| {
            modal.render(f, f.area());
        })?;
        
        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            if key.modifiers.contains(ratatui::crossterm::event::KeyModifiers::CONTROL) {
                                break;  // Ctrl+Q to quit
                            } else {
                                if let Some(selection) = modal.handle_input('q').await? {
                                    result = Some(selection);
                                    break;
                                }
                            }
                        },
                        KeyCode::Char(c) => {
                            if let Some(selection) = modal.handle_input(c).await? {
                                result = Some(selection);
                                break;
                            }
                        },
                        KeyCode::Enter => {
                            if let Some(selection) = modal.handle_input('\n').await? {
                                result = Some(selection);
                                break;
                            }
                        },
                        KeyCode::Esc => {
                            if let Some(selection) = modal.handle_input('\x1b').await? {
                                result = Some(selection);
                                break;
                            }
                            if !modal.is_visible() {
                                break;  // Modal was closed
                            }
                        },
                        KeyCode::Up => {
                            modal.handle_input('k').await?;
                        },
                        KeyCode::Down => {
                            modal.handle_input('j').await?;
                        },
                        KeyCode::Left => {
                            modal.handle_input('h').await?;
                        },
                        KeyCode::Right => {
                            modal.handle_input('l').await?;
                        },
                        KeyCode::Backspace => {
                            modal.handle_input('\x08').await?;
                        },
                        _ => {},
                    }
                }
            }
        }
    }
    
    // Cleanup terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    
    // Apply selection if made
    if let Some(selection) = result {
        config.global.default_provider = selection.provider.clone();
        config.global.default_model = selection.model.clone();
        config.save().await?;
        
        println!("✅ Model selection applied:");
        println!("  Provider: {}", selection.provider);
        println!("  Model: {}", selection.model);
        if let Some(host) = selection.host {
            println!("  Host: {}", host);
        }
        
        info!("Model selection completed: {}:{}", selection.provider, selection.model);
    } else {
        println!("Model selection cancelled.");
    }
    
    Ok(())
}

async fn test_model(config: &ConfigManager, providers: &ProviderManager, message: Option<&str>) -> Result<()> {
    let test_message = message.unwrap_or("Hello! Please respond with a brief test message.");
    
    println!("Testing current model configuration...");
    println!("  Provider: {}", config.global.default_provider);
    println!("  Model: {}", config.global.default_model);
    println!("  Message: {}", test_message);
    println!();
    
    // Get the provider
    let provider = providers.get_provider_or_host(&config.global.default_provider)
        .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", config.global.default_provider))?;
    
    // Create a simple chat request
    use crate::providers::{ChatRequest, Message, MessageRole};
    let messages = vec![Message::new(MessageRole::User, test_message.to_string())];
    let request = ChatRequest::new(messages, config.global.default_model.clone());
    
    // Send the request
    println!("📤 Sending test request...");
    match provider.chat(&request).await {
        Ok(response) => {
            println!("✅ Test successful!");
            println!("📥 Response: {}", response.content);
            println!("📊 Tokens used: {}", response.tokens_used);
            if let Some(cost) = response.cost {
                println!("💰 Cost: ${:.4}", cost);
            }
        },
        Err(e) => {
            println!("❌ Test failed: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

async fn auto_select_model(
    config: &mut ConfigManager, 
    task: Option<TaskTypeArg>,
    prefer_local: bool,
    prefer_fast: bool,
    prefer_accurate: bool
) -> Result<()> {
    println!("🧠 Starting intelligent model auto-selection...");
    
    // Set up selection criteria
    let mut criteria = SelectionCriteria::default();
    
    if let Some(task_type) = task {
        criteria.task_type = task_type.into();
    }
    
    criteria.prefer_local = prefer_local;
    criteria.prefer_fast = prefer_fast;
    criteria.prefer_accurate = prefer_accurate;
    
    println!("📋 Selection criteria:");
    println!("  Task type: {:?}", criteria.task_type);
    println!("  Prefer local: {}", criteria.prefer_local);
    println!("  Prefer fast: {}", criteria.prefer_fast);
    println!("  Prefer accurate: {}", criteria.prefer_accurate);
    println!();
    
    // Initialize embedding manager and auto-selection engine
    let embedding_config = crate::cost::EmbeddingConfig::default();
    let mut embedding_manager = EmbeddingManager::new(embedding_config);
    let mut auto_engine = AutoSelectionEngine::new();
    
    // Perform intelligent selection
    println!("🔍 Analyzing available models...");
    match auto_engine.select_optimal_model(&mut embedding_manager, &criteria).await {
        Ok(selected_model) => {
            println!("✅ Optimal model selected:");
            println!("  Model: {}", selected_model.name);
            println!("  Provider: {}", selected_model.provider);
            println!("  Size: {}MB", selected_model.size_mb);
            println!("  Description: {}", selected_model.description);
            println!();
            
            // Update configuration
            config.global.default_provider = "embedding".to_string();  // Special provider for embedding models
            config.global.default_model = selected_model.name.clone();
            config.save().await?;
            
            println!("💾 Configuration updated!");
            
            // Show performance stats if available
            if let Some(perf) = auto_engine.get_model_performance(&selected_model.name) {
                println!("📊 Performance stats:");
                println!("  Avg response time: {:.0}ms", perf.avg_response_time_ms);
                println!("  Reliability score: {:.1}%", perf.reliability_score * 100.0);
            }
        },
        Err(e) => {
            println!("❌ Auto-selection failed: {}", e);
            println!("💡 Try running 'aircher model list' to see available models");
            return Err(e);
        }
    }
    
    Ok(())
}

async fn show_providers(config: &ConfigManager, providers: &ProviderManager) -> Result<()> {
    println!("Available Providers:");
    println!();
    
    let provider_names = providers.list_providers();
    
    for provider_name in &provider_names {
        let current_marker = if provider_name == &config.global.default_provider {
            " ← current"
        } else {
            ""
        };
        
        let icon = match provider_name.as_str() {
            "claude" => "🤖",
            "openai" => "🧠", 
            "gemini" => "⭐",
            "ollama" => "🏠",
            "openrouter" => "🌐",
            _ => "📡",
        };
        
        println!("{} {}{}", icon, provider_name, current_marker);
        
        // Show provider status
        if let Some(provider_config) = config.providers.get(provider_name) {
            let status = if config.is_provider_enabled(provider_name) {
                if provider_name == "ollama" {
                    "🟢 Available (local)"
                } else if config.has_api_key(provider_name) {
                    "🟢 Authenticated"
                } else {
                    "🟡 Not authenticated"
                }
            } else {
                "🔴 Disabled"
            };
            
            println!("   Status: {}", status);
            if !provider_config.models.is_empty() {
                println!("   Default model: {}", provider_config.models[0].name);
            }
        }
        
        println!();
    }
    
    Ok(())
}