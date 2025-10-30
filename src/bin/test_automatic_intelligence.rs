/// Test automatic intelligence middleware integration
use anyhow::Result;
use aircher::intelligence::{IntelligenceEngine, UnifiedIntelligenceEngine};
use aircher::config::ConfigManager;
use aircher::storage::DatabaseManager;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧠 Testing Automatic Intelligence Middleware Integration");
    println!("{}", "=".repeat(60));

    // Initialize intelligence engine
    let config = ConfigManager::load().await?;
    let db_manager = DatabaseManager::new(&config).await?;
    let base_intelligence = Arc::new(IntelligenceEngine::new(&config, &db_manager).await?);

    // Create unified intelligence engine for automatic middleware
    let unified_intelligence = UnifiedIntelligenceEngine::new(base_intelligence);

    println!("✅ UnifiedIntelligenceEngine created successfully");

    // Test 1: Request Enhancement
    println!("\n📥 Testing automatic request enhancement...");
    let test_requests = vec![
        "Fix the authentication bug in login.rs",
        "Create a new user service with proper error handling",
        "Analyze the performance of the database queries",
        "Refactor the auth module to use better patterns",
    ];

    for (i, request) in test_requests.iter().enumerate() {
        println!("\n🔍 Test {}: \"{}\"", i + 1, request);

        match unified_intelligence.enhance_request_understanding(request).await {
            Ok(enhanced_context) => {
                println!("  ✅ Intent detected: {:?}", enhanced_context.detected_intent);
                println!("  ✅ Confidence: {:.2}", enhanced_context.confidence);
                println!("  ✅ Suggested approach: {}", enhanced_context.suggested_approach);
                println!("  ✅ Context items: {}", enhanced_context.relevant_context.len());
                println!("  ✅ Intelligence insights: {}", enhanced_context.intelligence_insights.len());
            }
            Err(e) => {
                println!("  ❌ Enhancement failed: {}", e);
            }
        }
    }

    // Test 2: System Prompt Enhancement
    println!("\n📝 Testing automatic system prompt enhancement...");
    let base_prompt = "You are Aircher, an AI coding assistant.";
    let test_request = "Fix the authentication bug in login.rs";

    // First get enhanced context
    match unified_intelligence.enhance_request_understanding(test_request).await {
        Ok(enhanced_context) => {
            match unified_intelligence.enhance_system_prompt(base_prompt, &enhanced_context).await {
                Ok(enhanced_prompt) => {
                    println!("  ✅ Base prompt: {}", base_prompt);
                    println!("  ✅ Enhanced prompt length: {} chars", enhanced_prompt.len());
                    println!("  ✅ Enhancement successful: {}", enhanced_prompt.len() > base_prompt.len());
                    if enhanced_prompt.len() > base_prompt.len() {
                        println!("  ℹ️  First 200 chars: {}", enhanced_prompt.chars().take(200).collect::<String>());
                    }
                }
                Err(e) => {
                    println!("  ❌ System prompt enhancement failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("  ❌ Could not get enhanced context for prompt test: {}", e);
        }
    }

    // Test 3: Response Enhancement
    println!("\n📤 Testing automatic response enhancement...");
    let test_response = "I've identified the issue in your authentication code.";

    match unified_intelligence.enhance_request_understanding(test_request).await {
        Ok(enhanced_context) => {
            match unified_intelligence.enhance_response_quality(test_response, test_request, &enhanced_context).await {
                Ok(enhanced_response) => {
                    println!("  ✅ Original response: {}", test_response);
                    println!("  ✅ Enhanced response length: {} chars", enhanced_response.final_response.len());
                    println!("  ✅ Intelligence additions: {}", enhanced_response.intelligence_additions.len());
                    println!("  ✅ Enhancement successful: {}", enhanced_response.final_response.len() > test_response.len());

                    if !enhanced_response.intelligence_additions.is_empty() {
                        println!("  ℹ️  Intelligence additions:");
                        for addition in &enhanced_response.intelligence_additions {
                            println!("     - {:?}: {}", addition.addition_type, addition.content.chars().take(100).collect::<String>());
                        }
                    }
                }
                Err(e) => {
                    println!("  ❌ Response enhancement failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("  ❌ Could not get enhanced context for response test: {}", e);
        }
    }

    println!("\n🎉 Automatic Intelligence Middleware Test Complete!");
    println!("{}", "=".repeat(60));

    Ok(())
}