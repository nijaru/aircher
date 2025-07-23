use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, error};

use crate::agent::{AgentController, ProjectContext};
use crate::agent::conversation::ProgrammingLanguage;
use crate::config::ConfigManager;
use crate::providers::ProviderManager;
use crate::intelligence::IntelligenceEngine;
use crate::sessions::SessionManager;
use crate::storage::DatabaseManager;

/// Start an interactive AI coding assistant session
pub async fn run_agent_session(
    config: &ConfigManager,
    provider_name: Option<String>,
    model_name: Option<String>,
) -> Result<()> {
    info!("Starting AI agent session...");
    
    // Initialize provider
    let provider_manager = ProviderManager::new(config).await?;
    let provider_name = provider_name.unwrap_or_else(|| "ollama".to_string());
    
    let provider = provider_manager
        .get_provider_or_host(&provider_name)
        .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", provider_name))?;
    
    // Initialize storage and intelligence
    let storage = DatabaseManager::new(config).await?;
    let intelligence = IntelligenceEngine::new(config, &storage).await?;
    
    // Detect project context
    let project_root = std::env::current_dir()?;
    let project_context = detect_project_context(&project_root)?;
    
    info!("Project detected: {} at {}", 
        project_context.language.to_string(), 
        project_context.root_path.display()
    );
    
    // Create agent controller - need to work around the trait object issue
    // For now, let's create a simpler test without the full agent
    
    println!("🤖 AI Agent functionality is being implemented...");
    println!("Project detected: {} at {}", 
        project_context.language.to_string(), 
        project_context.root_path.display()
    );
    
    return Ok(());
    
    /*
    // TODO: Fix provider trait object issue
    let mut agent = AgentController::new(
        Box::new(provider.clone()),
        intelligence,
        project_context,
    )?;
    
    // Simple REPL for testing
    println!("\n🤖 AI Coding Assistant Ready!");
    println!("Type 'exit' to quit, 'clear' to clear conversation\n");
    
    let stdin = std::io::stdin();
    let mut line = String::new();
    
    loop {
        print!("> ");
        std::io::Write::flush(&mut std::io::stdout())?;
        
        line.clear();
        stdin.read_line(&mut line)?;
        let input = line.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input == "exit" {
            break;
        }
        
        if input == "clear" {
            agent.clear_conversation();
            println!("Conversation cleared.\n");
            continue;
        }
        
        // Process the message
        match agent.process_message(input).await {
            Ok(response) => {
                println!("\n{}\n", response);
            }
            Err(e) => {
                error!("Error processing message: {}", e);
                println!("\n❌ Error: {}\n", e);
            }
        }
    }
    
    println!("Goodbye!");
    Ok(())
    */
}

/// Detect project context from the current directory
fn detect_project_context(root: &PathBuf) -> Result<ProjectContext> {
    // Detect programming language based on files
    let language = if root.join("Cargo.toml").exists() {
        ProgrammingLanguage::Rust
    } else if root.join("package.json").exists() {
        if root.join("tsconfig.json").exists() {
            ProgrammingLanguage::TypeScript
        } else {
            ProgrammingLanguage::JavaScript
        }
    } else if root.join("requirements.txt").exists() || root.join("pyproject.toml").exists() {
        ProgrammingLanguage::Python
    } else if root.join("go.mod").exists() {
        ProgrammingLanguage::Go
    } else if root.join("pom.xml").exists() {
        ProgrammingLanguage::Java
    } else if root.join("*.csproj").exists() {
        ProgrammingLanguage::CSharp
    } else {
        ProgrammingLanguage::Other("Unknown".to_string())
    };
    
    // Detect framework
    let framework = match &language {
        ProgrammingLanguage::Rust => {
            if root.join("Cargo.toml").exists() {
                let content = std::fs::read_to_string(root.join("Cargo.toml"))?;
                if content.contains("actix-web") {
                    Some("Actix Web".to_string())
                } else if content.contains("rocket") {
                    Some("Rocket".to_string())
                } else if content.contains("tokio") {
                    Some("Tokio".to_string())
                } else {
                    None
                }
            } else {
                None
            }
        }
        ProgrammingLanguage::JavaScript | ProgrammingLanguage::TypeScript => {
            if root.join("package.json").exists() {
                let content = std::fs::read_to_string(root.join("package.json"))?;
                if content.contains("react") {
                    Some("React".to_string())
                } else if content.contains("vue") {
                    Some("Vue".to_string())
                } else if content.contains("angular") {
                    Some("Angular".to_string())
                } else if content.contains("express") {
                    Some("Express".to_string())
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    };
    
    Ok(ProjectContext {
        root_path: root.clone(),
        language,
        framework,
        recent_changes: Vec::new(), // Would populate from git
    })
}

impl ProgrammingLanguage {
    pub fn to_string(&self) -> &str {
        match self {
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Go => "Go",
            Self::Java => "Java",
            Self::CSharp => "C#",
            Self::Cpp => "C++",
            Self::Other(s) => s,
        }
    }
}