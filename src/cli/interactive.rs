use anyhow::{Context, Result};
use std::io::{self, Write};
use tracing::{debug, info};

use crate::providers::{ChatRequest, Message, MessageRole, ProviderManager};

pub struct InteractiveSession {
    messages: Vec<Message>,
    provider_name: String,
    model: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

impl InteractiveSession {
    pub fn new(
        provider_name: String,
        model: String,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Self {
        Self {
            messages: Vec::new(),
            provider_name,
            model,
            max_tokens,
            temperature,
        }
    }

    pub async fn run(&mut self, providers: &ProviderManager) -> Result<()> {
        info!(
            "Starting interactive session with {} using {}",
            self.provider_name, self.model
        );

        println!("🏹 Aircher Interactive Mode");
        println!("Provider: {} | Model: {}", self.provider_name, self.model);
        println!("Type your message and press Enter. Use /help for commands, /quit to exit.\n");

        loop {
            // Display prompt
            print!("> ");
            io::stdout().flush().context("Failed to flush stdout")?;

            // Read user input
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .context("Failed to read input")?;
            let input = input.trim();

            // Handle empty input
            if input.is_empty() {
                continue;
            }

            // Handle commands
            if input.starts_with('/') {
                if self.handle_command(input).await? {
                    break; // Exit requested
                }
                continue;
            }

            // Process chat message
            if let Err(e) = self.process_message(input, providers).await {
                eprintln!("❌ Error: {}", e);
                continue;
            }
        }

        Ok(())
    }

    async fn handle_command(&mut self, command: &str) -> Result<bool> {
        match command {
            "/help" => {
                println!("\n📖 Available Commands:");
                println!("  /help     - Show this help message");
                println!("  /quit     - Exit interactive mode");
                println!("  /clear    - Clear conversation history");
                println!("  /history  - Show conversation history");
                println!("  /model    - Show current model information");
                println!();
                Ok(false)
            }
            "/quit" | "/exit" => {
                println!("👋 Goodbye!");
                Ok(true)
            }
            "/clear" => {
                self.messages.clear();
                println!("🧹 Conversation history cleared");
                Ok(false)
            }
            "/history" => {
                self.show_history();
                Ok(false)
            }
            "/model" => {
                println!("📊 Current Configuration:");
                println!("  Provider: {}", self.provider_name);
                println!("  Model: {}", self.model);
                if let Some(temp) = self.temperature {
                    println!("  Temperature: {}", temp);
                }
                if let Some(tokens) = self.max_tokens {
                    println!("  Max Tokens: {}", tokens);
                }
                println!();
                Ok(false)
            }
            _ => {
                eprintln!("❌ Unknown command: {}", command);
                eprintln!("   Use /help to see available commands");
                Ok(false)
            }
        }
    }

    fn show_history(&self) {
        if self.messages.is_empty() {
            println!("📝 No conversation history");
            return;
        }

        println!("\n📝 Conversation History:");
        println!("{}", "─".repeat(50));

        for (i, message) in self.messages.iter().enumerate() {
            let prefix = match message.role {
                MessageRole::User => "👤 You:",
                MessageRole::Assistant => "🤖 AI:",
                MessageRole::System => "⚙️ System:",
                MessageRole::Tool => "🔧 Tool:",
            };

            println!("{} {}", prefix, message.content);

            // Add separator between messages (except last)
            if i < self.messages.len() - 1 {
                println!("{}", "─".repeat(30));
            }
        }
        println!("{}\n", "─".repeat(50));
    }

    async fn process_message(&mut self, input: &str, providers: &ProviderManager) -> Result<()> {
        debug!("Processing message: {}", input);

        // Add user message to history
        let user_message = Message::user(input.to_string());
        self.messages.push(user_message);

        // Get provider
        let provider = providers
            .get_provider_or_host(&self.provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", self.provider_name))?;

        // Create chat request with full conversation history
        let mut request = ChatRequest::new(self.messages.clone(), self.model.clone());
        if let Some(max_tokens) = self.max_tokens {
            request.max_tokens = Some(max_tokens);
        }
        if let Some(temperature) = self.temperature {
            request.temperature = Some(temperature);
        }

        // Send request and display response
        print!("🤖 ");
        io::stdout().flush().context("Failed to flush stdout")?;

        match provider.chat(&request).await {
            Ok(response) => {
                println!("{}", response.content);

                // Show usage info
                if let Some(cost) = response.cost {
                    println!("💰 ${:.4} ({} tokens)", cost, response.tokens_used);
                } else {
                    println!("📊 {} tokens", response.tokens_used);
                }
                println!();

                // Add assistant response to history
                let assistant_message = Message::new(MessageRole::Assistant, response.content);
                self.messages.push(assistant_message);
            }
            Err(e) => {
                return Err(e.context("Chat request failed"));
            }
        }

        Ok(())
    }
}
