/// Test agent with semantic search integration for contextual code generation
use anyhow::Result;
use std::sync::Arc;

use aircher::auth::AuthManager;
use aircher::config::ConfigManager;
use aircher::storage::DatabaseManager;
use aircher::intelligence::IntelligenceEngine;
use aircher::semantic_search::SemanticCodeSearch;
use aircher::agent::{
    core::Agent,
    conversation::{ProjectContext, ProgrammingLanguage},
};
use aircher::providers::ProviderManager;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 TESTING SEMANTIC-ENHANCED CODE GENERATION");
    println!("===========================================\n");

    let config = ConfigManager::default();
    let auth_manager = Arc::new(AuthManager::new()?);
    let db_manager = DatabaseManager::new(&config).await?;

    // Initialize semantic search for the current project
    println!("🔧 Initializing semantic search for project...");
    let project_root = std::env::current_dir()?;
    let mut semantic_search = SemanticCodeSearch::new();

    // Try to load existing index first
    if let Ok(_) = semantic_search.load_persisted_index().await {
        println!("✅ Loaded existing semantic search index");
    } else {
        println!("📚 Building new semantic search index...");
        semantic_search.index_directory(&project_root).await?;
        println!("✅ Index built successfully");
    }

    // Create intelligence engine WITH semantic search
    let intelligence = IntelligenceEngine::with_semantic_search(&config, &db_manager, semantic_search).await?;

    let provider_manager = Arc::new(ProviderManager::new(&config, auth_manager.clone()).await?);

    let project_context = ProjectContext {
        root_path: std::env::current_dir()?,
        language: ProgrammingLanguage::Rust,
        framework: None,
        recent_changes: Vec::new(),
    };

    let agent = Agent::new(intelligence, auth_manager, project_context).await?;

    if let Some(provider) = provider_manager.get_provider("ollama") {
        println!("🧠 Testing contextual code generation with semantic understanding...\n");

        // Test 1: Code generation that should understand existing patterns
        let request = "Create a Rust function for parsing HTTP headers that follows the patterns used in our providers module. Include proper error handling and tests like we do elsewhere in the codebase.";

        println!("Request: {}", request);
        println!("\n⏱️ Processing with semantic context...");

        match agent.process_message(request, provider, "gpt-oss").await {
            Ok((response, status_messages)) => {
                println!("\n📊 RESPONSE ANALYSIS:");
                println!("Response length: {} characters", response.len());
                println!("Status messages: {} items", status_messages.len());

                println!("\n📋 Status Messages:");
                for (i, msg) in status_messages.iter().enumerate() {
                    println!("  {}: {}", i + 1, msg);
                }

                println!("\n📄 SEMANTIC-ENHANCED RESPONSE:");
                println!("---START---");
                println!("{}", response);
                println!("---END---");

                // Analyze the response for contextual awareness
                let has_project_patterns = response.contains("anyhow::Result") || response.contains("Result<");
                let has_error_handling = response.contains("Error") || response.contains("Err(");
                let has_tests = response.contains("#[test]") || response.contains("mod test");
                let has_http_context = response.contains("header") || response.contains("HTTP");
                let has_provider_patterns = response.contains("async") || response.contains("serde");

                println!("\n🔍 CONTEXTUAL ANALYSIS:");
                println!("  Uses project Result patterns: {}", has_project_patterns);
                println!("  Proper error handling: {}", has_error_handling);
                println!("  Includes tests: {}", has_tests);
                println!("  HTTP context awareness: {}", has_http_context);
                println!("  Provider-style patterns: {}", has_provider_patterns);

                if has_project_patterns && has_error_handling && has_tests {
                    println!("\n✅ SUCCESS: Generated contextually-aware code matching project patterns");
                } else {
                    println!("\n⚠️ PARTIAL: Code generated but missing some contextual patterns");
                }

                // Test 2: Check if agent can reference existing code
                println!("\n\n🔍 Testing codebase awareness...");
                let awareness_request = "What patterns do we use for HTTP client configuration in our existing providers?";

                match agent.process_message(awareness_request, provider, "gpt-oss").await {
                    Ok((awareness_response, _)) => {
                        println!("Codebase awareness response length: {} characters", awareness_response.len());

                        let mentions_providers = awareness_response.to_lowercase().contains("provider");
                        let mentions_ollama = awareness_response.to_lowercase().contains("ollama");
                        let mentions_client = awareness_response.to_lowercase().contains("client");

                        println!("🔍 AWARENESS ANALYSIS:");
                        println!("  References providers: {}", mentions_providers);
                        println!("  Mentions Ollama: {}", mentions_ollama);
                        println!("  Discusses HTTP clients: {}", mentions_client);

                        if mentions_providers && mentions_client {
                            println!("✅ Agent demonstrates codebase awareness");
                        } else {
                            println!("❌ Agent lacks codebase awareness");
                        }

                        println!("\nCodebase awareness response:");
                        println!("---START---");
                        println!("{}", awareness_response);
                        println!("---END---");
                    }
                    Err(e) => println!("❌ Failed to test codebase awareness: {}", e),
                }
            }
            Err(e) => {
                println!("❌ Error processing request: {}", e);
                println!("Error chain: {:?}", e.chain().collect::<Vec<_>>());
            }
        }
    } else {
        println!("❌ No Ollama provider available");
    }

    Ok(())
}