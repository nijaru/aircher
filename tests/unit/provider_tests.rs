use chrono::Utc;

use aircher::providers::{
    ChatRequest, ChatResponse, FinishReason, Message, MessageRole, PricingModel
};

#[test]
fn test_message_creation() {
    let user_msg = Message::user("Hello world".to_string());
    
    assert_eq!(user_msg.role, MessageRole::User);
    assert_eq!(user_msg.content, "Hello world");
    assert!(!user_msg.id.is_empty());
    assert!(user_msg.tokens_used.is_none());
    assert!(user_msg.cost.is_none());
    
    // Timestamp should be recent
    let now = Utc::now();
    let diff = now - user_msg.timestamp;
    assert!(diff.num_seconds() < 2);
}

#[test]
fn test_message_roles() {
    let system_msg = Message::system("System prompt".to_string());
    assert_eq!(system_msg.role, MessageRole::System);
    
    let user_msg = Message::user("User message".to_string());
    assert_eq!(user_msg.role, MessageRole::User);
    
    let assistant_msg = Message::assistant("AI response".to_string());
    assert_eq!(assistant_msg.role, MessageRole::Assistant);
    
    let tool_msg = Message::new(MessageRole::Tool, "Tool output".to_string());
    assert_eq!(tool_msg.role, MessageRole::Tool);
}

#[test]
fn test_chat_request_creation() {
    let messages = vec![
        Message::system("You are a helpful assistant".to_string()),
        Message::user("Hello".to_string()),
    ];
    let model = "test-model".to_string();
    
    let request = ChatRequest::new(messages.clone(), model.clone());
    
    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.model, model);
    assert!(!request.stream);
    assert!(request.max_tokens.is_none());
    assert!(request.temperature.is_none());
    assert!(request.tools.is_none());
}

#[test]
fn test_chat_request_modifications() {
    let messages = vec![Message::user("Test".to_string())];
    let model = "test-model".to_string();
    
    let request = ChatRequest::new(messages, model)
        .with_streaming()
        .with_max_tokens(1000)
        .with_temperature(0.7);
    
    assert!(request.stream);
    assert_eq!(request.max_tokens, Some(1000));
    assert_eq!(request.temperature, Some(0.7));
}

#[test]
fn test_simple_chat_request() {
    let request = ChatRequest::simple("Hello AI".to_string(), "gpt-4".to_string());
    
    assert_eq!(request.messages.len(), 1);
    assert_eq!(request.messages[0].role, MessageRole::User);
    assert_eq!(request.messages[0].content, "Hello AI");
    assert_eq!(request.model, "gpt-4");
}

#[test]
fn test_chat_response_structure() {
    let response = ChatResponse {
        id: "test-response".to_string(),
        content: "Hello! How can I help you?".to_string(),
        role: MessageRole::Assistant,
        model: "test-model".to_string(),
        tokens_used: 150,
        cost: Some(0.0015),
        finish_reason: FinishReason::Stop,
        tool_calls: None,
    };
    
    assert_eq!(response.id, "test-response");
    assert_eq!(response.content, "Hello! How can I help you?");
    assert_eq!(response.role, MessageRole::Assistant);
    assert_eq!(response.model, "test-model");
    assert_eq!(response.tokens_used, 150);
    assert_eq!(response.cost, Some(0.0015));
    assert_eq!(response.finish_reason, FinishReason::Stop);
    assert!(response.tool_calls.is_none());
}

#[test]
fn test_finish_reasons() {
    // Test all finish reason variants
    let reasons = vec![
        FinishReason::Stop,
        FinishReason::Length,
        FinishReason::ToolCalls,
        FinishReason::ContentFilter,
        FinishReason::Error,
    ];
    
    for reason in reasons {
        // Should be able to clone and debug
        let _cloned = reason.clone();
        let _debug_str = format!("{:?}", reason);
    }
}

#[test]
fn test_pricing_model_variants() {
    let per_token = PricingModel::PerToken {
        input_cost_per_1m: 1.0,
        output_cost_per_1m: 2.0,
        currency: "USD".to_string(),
    };
    
    match per_token {
        PricingModel::PerToken { input_cost_per_1m, output_cost_per_1m, currency } => {
            assert_eq!(input_cost_per_1m, 1.0);
            assert_eq!(output_cost_per_1m, 2.0);
            assert_eq!(currency, "USD");
        }
        _ => panic!("Expected PerToken variant"),
    }
    
    let free = PricingModel::Free;
    match free {
        PricingModel::Free => {} // Expected
        _ => panic!("Expected Free variant"),
    }
}

#[test]
fn test_message_role_equality() {
    assert_eq!(MessageRole::User, MessageRole::User);
    assert_eq!(MessageRole::Assistant, MessageRole::Assistant);
    assert_eq!(MessageRole::System, MessageRole::System);
    assert_eq!(MessageRole::Tool, MessageRole::Tool);
    
    assert_ne!(MessageRole::User, MessageRole::Assistant);
    assert_ne!(MessageRole::System, MessageRole::Tool);
}

#[test]
fn test_message_unique_ids() {
    let msg1 = Message::user("Test 1".to_string());
    let msg2 = Message::user("Test 2".to_string());
    let msg3 = Message::user("Test 1".to_string()); // Same content
    
    // All messages should have unique IDs
    assert_ne!(msg1.id, msg2.id);
    assert_ne!(msg1.id, msg3.id);
    assert_ne!(msg2.id, msg3.id);
    
    // But same content is preserved
    assert_eq!(msg1.content, msg3.content);
}

#[test]
fn test_chat_request_with_empty_messages() {
    let empty_messages = vec![];
    let request = ChatRequest::new(empty_messages, "test-model".to_string());
    
    assert_eq!(request.messages.len(), 0);
    assert_eq!(request.model, "test-model");
}

#[test]
fn test_message_timestamp_ordering() {
    let msg1 = Message::user("First".to_string());
    
    // Small delay to ensure different timestamps
    std::thread::sleep(std::time::Duration::from_millis(1));
    
    let msg2 = Message::user("Second".to_string());
    
    assert!(msg2.timestamp > msg1.timestamp);
}