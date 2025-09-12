use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::providers::{ChatRequest, ChatResponse, FinishReason, LLMProvider, MessageRole};
use crate::sessions::{Session, SessionMessage, MessageRole as SessionMessageRole};
use crate::intelligence::tools::IntelligenceTools;
use crate::intelligence::{ContextualInsight, ImpactAnalysis, ProjectMomentum, ContextSuggestions, Outcome, CrossProjectInsight};

/// Mock LLM Provider for testing
pub struct MockProvider {
    pub name: String,
    pub responses: Arc<Mutex<Vec<String>>>,
    pub call_count: Arc<Mutex<usize>>,
}

impl MockProvider {
    pub fn new(name: String) -> Self {
        Self {
            name,
            responses: Arc::new(Mutex::new(vec!["Mock response".to_string()])),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn add_response(&self, response: String) {
        self.responses.lock().unwrap().push(response);
    }

    pub fn get_call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }
}

#[async_trait]
impl LLMProvider for MockProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn chat(&self, _req: &ChatRequest) -> Result<ChatResponse> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;
        
        let responses = self.responses.lock().unwrap();
        let response_text = responses.get(0).unwrap_or(&"Default mock response".to_string()).clone();
        
        Ok(ChatResponse {
            id: "mock-id".to_string(),
            content: response_text,
            role: MessageRole::Assistant,
            model: "mock-model".to_string(),
            tokens_used: 100,
            cost: Some(0.01),
            finish_reason: FinishReason::Stop,
            tool_calls: None,
        })
    }

    async fn stream(&self, _req: &ChatRequest) -> Result<crate::providers::ResponseStream> {
        // For testing, we'll just return a simple stream
        let (_tx, rx) = tokio::sync::mpsc::channel(1);
        Ok(rx)
    }

    fn supports_tools(&self) -> bool { true }
    fn supports_vision(&self) -> bool { false }
    fn context_window(&self) -> u32 { 4096 }
    fn pricing_model(&self) -> crate::providers::PricingModel { 
        crate::providers::PricingModel::PerToken {
            input_cost_per_1m: 3.0,
            output_cost_per_1m: 15.0,
            currency: "USD".to_string(),
        }
    }
    fn calculate_cost(&self, _input_tokens: u32, _output_tokens: u32) -> Option<f64> { Some(0.01) }
    fn get_pricing(&self) -> Option<crate::providers::PricingInfo> { None }
    async fn get_usage_info(&self) -> Result<Option<crate::providers::UsageInfo>> { Ok(None) }
    fn usage_warning_threshold(&self) -> Option<f64> { None }
    async fn health_check(&self) -> Result<bool> { Ok(true) }
    async fn list_available_models(&self) -> Result<Vec<String>> { 
        Ok(vec!["mock-model-1".to_string(), "mock-model-2".to_string()])
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Mock Intelligence Tools for testing
pub struct MockIntelligenceTools {
    pub insights: Arc<Mutex<Vec<ContextualInsight>>>,
    pub call_log: Arc<Mutex<Vec<String>>>,
}

impl MockIntelligenceTools {
    pub fn new() -> Self {
        Self {
            insights: Arc::new(Mutex::new(vec![])),
            call_log: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn add_insight(&self, insight: ContextualInsight) {
        self.insights.lock().unwrap().push(insight);
    }

    pub fn get_calls(&self) -> Vec<String> {
        self.call_log.lock().unwrap().clone()
    }
}

#[async_trait]
impl IntelligenceTools for MockIntelligenceTools {
    async fn get_development_context(&self, query: &str) -> ContextualInsight {
        self.call_log.lock().unwrap().push(format!("get_development_context: {}", query));
        
        let insights = self.insights.lock().unwrap();
        insights.first().cloned().unwrap_or_else(|| ContextualInsight {
            development_phase: "Mock phase".to_string(),
            active_story: "Mock story".to_string(),
            key_files: vec![],
            architectural_context: vec![],
            recent_patterns: vec![],
            suggested_next_actions: vec![],
            confidence: 0.8,
        })
    }

    async fn analyze_change_impact(&self, files: &[String]) -> ImpactAnalysis {
        self.call_log.lock().unwrap().push(format!("analyze_change_impact: {:?}", files));
        ImpactAnalysis {
            direct_impacts: vec!["Direct impact".to_string()],
            indirect_impacts: vec!["Indirect impact".to_string()],
            risk_areas: vec!["Risk area".to_string()],
            suggested_tests: vec!["Test suggestion".to_string()],
        }
    }

    async fn suggest_missing_context(&self, current_files: &[String]) -> ContextSuggestions {
        self.call_log.lock().unwrap().push(format!("suggest_missing_context: {:?}", current_files));
        ContextSuggestions {
            missing_dependencies: vec!["Missing dep".to_string()],
            architectural_context: vec!["Arch context".to_string()],
            historical_context: vec!["History".to_string()],
            confidence: 0.8,
        }
    }

    async fn track_conversation_outcome(&self, files: &[String], _outcome: Outcome) -> () {
        self.call_log.lock().unwrap().push(format!("track_conversation_outcome: {:?}", files));
    }

    async fn get_project_momentum(&self) -> ProjectMomentum {
        self.call_log.lock().unwrap().push("get_project_momentum".to_string());
        ProjectMomentum {
            recent_focus: "Mock focus".to_string(),
            velocity_indicators: vec!["High velocity".to_string()],
            architectural_direction: "Clean architecture".to_string(),
            next_priorities: vec!["Priority 1".to_string()],
            knowledge_gaps: vec!["Gap 1".to_string()],
        }
    }

    async fn add_project_directory(&self, path: &str) -> Result<(), String> {
        self.call_log.lock().unwrap().push(format!("add_project_directory: {}", path));
        Ok(())
    }

    async fn analyze_cross_project_patterns(&self, query: &str) -> CrossProjectInsight {
        self.call_log.lock().unwrap().push(format!("analyze_cross_project_patterns: {}", query));
        CrossProjectInsight {
            similar_patterns: vec![],
            architectural_lessons: vec![],
            user_preferences: vec![],
            implementation_examples: vec![],
        }
    }

    async fn load_ai_configuration(&self) -> crate::intelligence::AiConfiguration {
        self.call_log.lock().unwrap().push("load_ai_configuration".to_string());
        crate::intelligence::AiConfiguration {
            global_instructions: Some("Mock global instructions".to_string()),
            project_instructions: Some("Mock project instructions".to_string()),
            cursor_rules: None,
            copilot_instructions: None,
            legacy_claude: None,
            custom_instructions: vec![],
        }
    }
    
    async fn search_code_semantically(&self, query: &str, _limit: usize) -> Result<Vec<crate::intelligence::CodeSearchResult>, String> {
        self.call_log.lock().unwrap().push(format!("search_code_semantically: {}", query));
        Ok(vec![])
    }
    
    async fn analyze_code_structure(&self, file_path: &str) -> Result<crate::intelligence::ast_analysis::ASTAnalysis, String> {
        self.call_log.lock().unwrap().push(format!("analyze_code_structure: {}", file_path));
        Ok(crate::intelligence::ast_analysis::ASTAnalysis {
            file_path: std::path::PathBuf::from(file_path),
            language: "rust".to_string(),
            functions: vec![],
            classes: vec![],
            imports: vec![],
            exports: vec![],
            complexity_metrics: crate::intelligence::ast_analysis::ComplexityMetrics {
                cyclomatic_complexity: 1,
                cognitive_complexity: 1,
                nesting_depth: 1,
                lines_of_code: 10,
                comment_ratio: 0.1,
            },
            patterns: vec![],
            dependencies: vec![],
        })
    }
    
    async fn get_code_insights(&self, file_path: &str) -> Result<crate::intelligence::CodeInsights, String> {
        self.call_log.lock().unwrap().push(format!("get_code_insights: {}", file_path));
        Ok(crate::intelligence::CodeInsights {
            file_path: file_path.to_string(),
            language: "rust".to_string(),
            quality_score: 0.8,
            complexity_summary: "Mock complexity".to_string(),
            key_functions: vec![],
            dependencies: vec![],
            patterns: vec![],
            suggestions: vec![],
            ast_analysis: None,
        })
    }
    
    async fn initialize_project_memory(&mut self, project_root: std::path::PathBuf) -> Result<(), String> {
        self.call_log.lock().unwrap().push(format!("initialize_project_memory: {}", project_root.display()));
        Ok(())
    }
    
    async fn start_session(&self, session_id: Option<String>) -> Result<Option<String>, String> {
        self.call_log.lock().unwrap().push(format!("start_session: {:?}", session_id));
        Ok(Some("mock_session_123".to_string()))
    }
    
    async fn record_learning(
        &self,
        session_id: &str,
        user_query: &str,
        files_involved: &[String],
        tools_used: &[String],
        _outcome: Outcome,
    ) -> Result<(), String> {
        self.call_log.lock().unwrap().push(format!("record_learning: {} {} {:?} {:?}", session_id, user_query, files_involved, tools_used));
        Ok(())
    }
    
    async fn get_relevant_patterns(&self, query: &str, session_id: &str) -> Result<Vec<String>, String> {
        self.call_log.lock().unwrap().push(format!("get_relevant_patterns: {} {}", query, session_id));
        Ok(vec!["Mock pattern 1".to_string(), "Mock pattern 2".to_string()])
    }
}

/// Mock Session Manager for testing
pub struct MockSessionManager {
    pub sessions: Arc<Mutex<HashMap<String, Session>>>,
    pub messages: Arc<Mutex<HashMap<String, Vec<SessionMessage>>>>,
    pub call_log: Arc<Mutex<Vec<String>>>,
}

impl MockSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
            call_log: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn add_session(&self, session: Session) {
        self.sessions.lock().unwrap().insert(session.id.clone(), session);
    }

    pub fn get_calls(&self) -> Vec<String> {
        self.call_log.lock().unwrap().clone()
    }
}

#[async_trait]
impl SessionManagerTrait for MockSessionManager {
    async fn create_session(
        &self,
        title: String,
        provider: String,
        model: String,
        description: Option<String>,
        tags: Vec<String>,
    ) -> Result<Session> {
        self.call_log.lock().unwrap().push(format!("create_session: {}", title));
        
        let session = Session {
            id: "mock-session-id".to_string(),
            title,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            provider,
            model,
            total_cost: 0.0,
            total_tokens: 0,
            message_count: 0,
            tags,
            is_archived: false,
            description,
        };
        
        self.add_session(session.clone());
        Ok(session)
    }

    async fn load_session(&self, session_id: &str) -> Result<Option<Session>> {
        self.call_log.lock().unwrap().push(format!("load_session: {}", session_id));
        Ok(self.sessions.lock().unwrap().get(session_id).cloned())
    }

    async fn add_message(&self, session_id: &str, message: &crate::sessions::Message) -> Result<()> {
        self.call_log.lock().unwrap().push(format!("add_message: {}", session_id));
        
        // Convert to SessionMessage
        let session_message = SessionMessage {
            id: message.id.clone(),
            session_id: session_id.to_string(),
            role: match message.role {
                crate::sessions::MessageRole::System => SessionMessageRole::System,
                crate::sessions::MessageRole::User => SessionMessageRole::User,
                crate::sessions::MessageRole::Assistant => SessionMessageRole::Assistant,
                crate::sessions::MessageRole::Tool => SessionMessageRole::Tool,
            },
            content: message.content.clone(),
            timestamp: message.timestamp,
            tokens_used: message.tokens_used,
            cost: message.cost,
            provider: "mock".to_string(),
            model: "mock".to_string(),
            finish_reason: None,
            sequence_number: 1,
        };
        
        self.messages.lock().unwrap()
            .entry(session_id.to_string())
            .or_insert_with(Vec::new)
            .push(session_message);
        
        Ok(())
    }

    async fn load_session_messages(&self, session_id: &str) -> Result<Vec<SessionMessage>> {
        self.call_log.lock().unwrap().push(format!("load_session_messages: {}", session_id));
        Ok(self.messages.lock().unwrap()
            .get(session_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn save_session(&self, session: &Session) -> Result<()> {
        self.call_log.lock().unwrap().push(format!("save_session: {}", session.id));
        self.sessions.lock().unwrap().insert(session.id.clone(), session.clone());
        Ok(())
    }
}

// Use the trait from the sessions module
pub use crate::sessions::SessionManagerTrait;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::ChatRequest;

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockProvider::new("test".to_string());
        provider.add_response("Test response".to_string());
        
        let request = ChatRequest::simple("Test".to_string(), "test-model".to_string());
        let response = provider.chat(&request).await.unwrap();
        
        assert_eq!(response.content, "Test response");
        assert_eq!(provider.get_call_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_intelligence_tools() {
        let tools = MockIntelligenceTools::new();
        
        let insight = tools.get_development_context("test query").await;
        assert_eq!(insight.development_phase, "Mock phase");
        
        let calls = tools.get_calls();
        assert_eq!(calls.len(), 1);
        assert!(calls[0].contains("get_development_context"));
    }

    #[tokio::test]
    async fn test_mock_session_manager() {
        let manager = MockSessionManager::new();
        
        let session = manager.create_session(
            "Test Session".to_string(),
            "test".to_string(),
            "test-model".to_string(),
            None,
            vec![],
        ).await.unwrap();
        
        assert_eq!(session.title, "Test Session");
        
        let loaded = manager.load_session(&session.id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().title, "Test Session");
    }
}