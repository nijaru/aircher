use aircher::agent::unified::UnifiedAgent;
use aircher::agent::conversation::{ProjectContext, ProgrammingLanguage};
use aircher::auth::AuthManager;
use aircher::config::ConfigManager;
use aircher::intelligence::IntelligenceEngine;
use aircher::providers::{LLMProvider, ModelInfo, PricingModel, ChatRequest, ChatResponse};
use aircher::storage::DatabaseManager;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use std::env;
use std::path::PathBuf;

/// Mock provider for testing intelligent agent without API calls
struct MockIntelligentProvider {
    name: String,
    response_with_tools: bool,
}

#[async_trait]
impl LLMProvider for MockIntelligentProvider {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn chat(&self, _request: &ChatRequest) -> Result<ChatResponse> {
        let content = if self.response_with_tools {
            r#"I'll help you analyze that code. Let me examine the file structure first.

<tool_use>
<tool>list_files</tool><params>{"path": "src", "recursive": false}</params>
</tool_use>

Now let me search for the specific pattern:

<tool_use>
<tool>search_code</tool><params>{"pattern": "error handling", "path": "src"}</params>
</tool_use>"#
        } else {
            "Based on my analysis, the code has good error handling patterns using Result types and proper error propagation."
        };
        
        Ok(ChatResponse {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.to_string(),
            role: aircher::providers::MessageRole::Assistant,
            tokens_used: 100,
            cost: Some(0.001),
            model: "mock-intelligent".to_string(),
            tool_calls: None,
            finish_reason: aircher::providers::FinishReason::Stop,
        })
    }
    
    async fn stream_chat(&self, _request: &ChatRequest) -> Result<aircher::providers::ChatStream> {
        unimplemented!("Stream not needed for test")
    }
    
    fn get_models(&self) -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: "mock-intelligent".to_string(),
            name: "Mock Intelligent Model".to_string(),
            context_window: 100000,
            max_output_tokens: Some(4096),
            supports_tools: true,
            supports_streaming: false,
            pricing: Some(aircher::providers::ModelPricing {
                input_cost_per_1k: 0.001,
                output_cost_per_1k: 0.002,
                currency: "USD".to_string(),
            }),
        }]
    }
    
    fn supports_tools(&self) -> bool {
        true
    }
    
    fn pricing_model(&self) -> PricingModel {
        PricingModel::PerToken {
            input_cost_per_1m: 0.001,
            output_cost_per_1m: 0.002,
            currency: "USD".to_string(),
        }
    }
}

#[tokio::test]
async fn test_intelligent_task_decomposition() {
    // Setup test environment
    let config = ConfigManager::load().await.unwrap();
    let db_manager = DatabaseManager::new(&config).await.unwrap();
    let intelligence = IntelligenceEngine::new(&config, &db_manager).await.unwrap();
    let auth_manager = Arc::new(AuthManager::new().unwrap());
    
    let project_context = ProjectContext {
        root_path: env::current_dir().unwrap(),
        language: ProgrammingLanguage::Rust,
        framework: Some("cargo".to_string()),
        recent_changes: Vec::new(),
    };
    
    // Create intelligent agent
    let mut agent = UnifiedAgent::new(intelligence, auth_manager, project_context)
        .await
        .unwrap();
    
    // Test with mock provider
    let provider = MockIntelligentProvider {
        name: "mock".to_string(),
        response_with_tools: true,
    };
    
    // Test complex request that should trigger task decomposition
    let complex_request = "Analyze the error handling patterns in this codebase and suggest improvements";
    
    let (response, tool_messages) = agent
        .process_message(complex_request, &provider, "mock-intelligent")
        .await
        .unwrap();
    
    // Verify intelligent processing occurred
    assert!(!response.is_empty(), "Should have generated a response");
    println!("Intelligent response: {}", response);
    
    // Check for tool execution indicators
    if !tool_messages.is_empty() {
        println!("Tools executed: {:?}", tool_messages);
        assert!(tool_messages.iter().any(|m| m.contains("🎯") || m.contains("🔧")),
            "Should have tool execution indicators");
    }
}

#[tokio::test]
async fn test_reasoning_engine_fallback() {
    // Setup test environment
    let config = ConfigManager::load().await.unwrap();
    let db_manager = DatabaseManager::new(&config).await.unwrap();
    let intelligence = IntelligenceEngine::new(&config, &db_manager).await.unwrap();
    let auth_manager = Arc::new(AuthManager::new().unwrap());
    
    let project_context = ProjectContext {
        root_path: env::current_dir().unwrap(),
        language: ProgrammingLanguage::Rust,
        framework: Some("cargo".to_string()),
        recent_changes: Vec::new(),
    };
    
    // Create intelligent agent
    let mut agent = UnifiedAgent::new(intelligence, auth_manager, project_context)
        .await
        .unwrap();
    
    // Test with mock provider that doesn't use tools
    let provider = MockIntelligentProvider {
        name: "mock".to_string(),
        response_with_tools: false,
    };
    
    // Test simple request that should work even if reasoning fails
    let simple_request = "What is the purpose of this project?";
    
    let (response, _) = agent
        .process_message(simple_request, &provider, "mock-intelligent")
        .await
        .unwrap();
    
    // Verify fallback worked
    assert!(!response.is_empty(), "Should have generated a response even with fallback");
    println!("Fallback response: {}", response);
}

#[tokio::test]
async fn test_pattern_learning() {
    // Setup test environment
    let config = ConfigManager::load().await.unwrap();
    let db_manager = DatabaseManager::new(&config).await.unwrap();
    let intelligence = IntelligenceEngine::new(&config, &db_manager).await.unwrap();
    let auth_manager = Arc::new(AuthManager::new().unwrap());
    
    let project_context = ProjectContext {
        root_path: env::current_dir().unwrap(),
        language: ProgrammingLanguage::Rust,
        framework: Some("cargo".to_string()),
        recent_changes: Vec::new(),
    };
    
    // Create intelligent agent
    let mut agent = UnifiedAgent::new(intelligence, auth_manager, project_context)
        .await
        .unwrap();
    
    let provider = MockIntelligentProvider {
        name: "mock".to_string(),
        response_with_tools: true,
    };
    
    // Execute same type of request multiple times
    let request1 = "Find all TODO comments in the code";
    let (response1, _) = agent.process_message(request1, &provider, "mock-intelligent")
        .await
        .unwrap();
    
    // Second similar request should potentially benefit from pattern learning
    let request2 = "Find all FIXME comments in the code";
    let (response2, _) = agent.process_message(request2, &provider, "mock-intelligent")
        .await
        .unwrap();
    
    // Both should succeed
    assert!(!response1.is_empty());
    assert!(!response2.is_empty());
    
    println!("Pattern learning test completed");
}
