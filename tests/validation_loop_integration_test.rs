// Integration tests for AutoGen validation loop (Option B)
// Tests the three Agent methods: find_bug_locations, verify_patch_location, generate_patch

use aircher::agent::{Agent, LocationCandidate, PatchProposal, VerificationResult};
use aircher::config::ConfigManager;
use aircher::providers::{LLMProvider, ChatRequest, ChatResponse, Message, MessageRole, ContentBlock};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio;

// Mock provider that returns predefined responses
struct MockProvider {
    responses: Arc<Mutex<Vec<String>>>,
    call_count: Arc<Mutex<usize>>,
}

impl MockProvider {
    fn new(responses: Vec<String>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    fn get_call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }
}

#[async_trait]
impl LLMProvider for MockProvider {
    async fn chat(&self, _request: &ChatRequest) -> anyhow::Result<ChatResponse> {
        let mut count = self.call_count.lock().unwrap();
        let responses = self.responses.lock().unwrap();

        let response_text = if *count < responses.len() {
            responses[*count].clone()
        } else {
            panic!("MockProvider: No more responses available (call {})", *count);
        };

        *count += 1;

        Ok(ChatResponse {
            content: response_text,
            model: "mock-model".to_string(),
            usage: None,
            finish_reason: Some("stop".to_string()),
        })
    }

    async fn stream_chat(
        &self,
        _request: &ChatRequest,
    ) -> anyhow::Result<
        tokio::sync::mpsc::Receiver<anyhow::Result<String>>,
    > {
        unimplemented!("Streaming not needed for tests")
    }

    fn supports_tools(&self) -> bool {
        false
    }

    fn get_models(&self) -> Vec<aircher::providers::ModelInfo> {
        vec![]
    }

    fn name(&self) -> &str {
        "mock"
    }
}

// Helper to create test agent
async fn create_test_agent() -> anyhow::Result<Agent> {
    let config = ConfigManager::load()?;
    Agent::new(config, None).await
}

#[tokio::test]
async fn test_find_bug_locations_success() {
    // Mock response with 2 location candidates
    let mock_response = r#"```json
[
  {
    "file_path": "src/auth.rs",
    "line_number": 42,
    "confidence": 0.95,
    "reasoning": "Variable name 'user_token' matches bug description"
  },
  {
    "file_path": "src/session.rs",
    "line_number": 128,
    "confidence": 0.75,
    "reasoning": "Authentication logic mentioned in error trace"
  }
]
```"#;

    let provider = MockProvider::new(vec![mock_response.to_string()]);
    let agent = create_test_agent().await.unwrap();

    let candidates = agent.find_bug_locations(
        "Authentication token not validated properly",
        std::path::Path::new("/tmp/test-repo"),
        &provider,
        "mock-model",
    ).await.unwrap();

    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].file_path, PathBuf::from("src/auth.rs"));
    assert_eq!(candidates[0].line_number, Some(42));
    assert!((candidates[0].confidence - 0.95).abs() < 0.01);
    assert_eq!(candidates[1].file_path, PathBuf::from("src/session.rs"));
    assert_eq!(provider.get_call_count(), 1);
}

#[tokio::test]
async fn test_find_bug_locations_no_markdown() {
    // Response without markdown code blocks
    let mock_response = r#"[
  {
    "file_path": "src/main.rs",
    "line_number": null,
    "confidence": 0.6,
    "reasoning": "Main entry point, possible issue"
  }
]"#;

    let provider = MockProvider::new(vec![mock_response.to_string()]);
    let agent = create_test_agent().await.unwrap();

    let candidates = agent.find_bug_locations(
        "Program crashes on startup",
        std::path::Path::new("/tmp/test-repo"),
        &provider,
        "mock-model",
    ).await.unwrap();

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].file_path, PathBuf::from("src/main.rs"));
    assert_eq!(candidates[0].line_number, None);
    assert!((candidates[0].confidence - 0.6).abs() < 0.01);
}

#[tokio::test]
async fn test_verify_patch_location_correct() {
    let mock_response = r#"```json
{
  "is_correct": true,
  "reasoning": "File contains the exact variable mentioned in bug description at line 42",
  "issues": []
}
```"#;

    let provider = MockProvider::new(vec![mock_response.to_string()]);
    let agent = create_test_agent().await.unwrap();

    let proposal = PatchProposal {
        location: LocationCandidate {
            file_path: PathBuf::from("src/auth.rs"),
            line_number: Some(42),
            confidence: 0.95,
            reasoning: "Variable matches".to_string(),
        },
        patch: "--- a/src/auth.rs\n+++ b/src/auth.rs\n@@ -42,1 +42,1 @@\n-    let token = None;\n+    let token = Some(user_token);\n".to_string(),
        reasoning: "Fix token initialization".to_string(),
    };

    let verification = agent.verify_patch_location(
        &proposal,
        "Token should not be None",
        &provider,
        "mock-model",
    ).await.unwrap();

    assert!(verification.is_correct);
    assert!(verification.issues.is_empty());
    assert!(verification.reasoning.contains("exact variable"));
}

#[tokio::test]
async fn test_verify_patch_location_incorrect() {
    let mock_response = r#"{
  "is_correct": false,
  "reasoning": "File doesn't contain the variable mentioned in bug description",
  "issues": [
    "Variable 'user_token' not found in file",
    "Line 42 is in documentation, not implementation"
  ]
}"#;

    let provider = MockProvider::new(vec![mock_response.to_string()]);
    let agent = create_test_agent().await.unwrap();

    let proposal = PatchProposal {
        location: LocationCandidate {
            file_path: PathBuf::from("src/wrong.rs"),
            line_number: Some(42),
            confidence: 0.5,
            reasoning: "Guessed location".to_string(),
        },
        patch: "--- a/src/wrong.rs\n+++ b/src/wrong.rs\n".to_string(),
        reasoning: "Attempted fix".to_string(),
    };

    let verification = agent.verify_patch_location(
        &proposal,
        "Token should not be None",
        &provider,
        "mock-model",
    ).await.unwrap();

    assert!(!verification.is_correct);
    assert_eq!(verification.issues.len(), 2);
    assert!(verification.issues[0].contains("not found"));
}

#[tokio::test]
async fn test_generate_patch_success() {
    let mock_response = r#"```json
{
  "patch": "--- a/src/auth.rs\n+++ b/src/auth.rs\n@@ -42,1 +42,1 @@\n-    let token = None;\n+    let token = validate_token(user_token)?;\n",
  "reasoning": "Changed token initialization to call validate_token function, which performs proper validation before returning Some(token)"
}
```"#;

    let provider = MockProvider::new(vec![mock_response.to_string()]);
    let agent = create_test_agent().await.unwrap();

    let location = LocationCandidate {
        file_path: PathBuf::from("src/auth.rs"),
        line_number: Some(42),
        confidence: 0.95,
        reasoning: "Exact location verified".to_string(),
    };

    let proposal = agent.generate_patch(
        "Token validation missing",
        &location,
        &provider,
        "mock-model",
    ).await.unwrap();

    assert_eq!(proposal.location.file_path, PathBuf::from("src/auth.rs"));
    assert!(proposal.patch.contains("validate_token"));
    assert!(proposal.reasoning.contains("proper validation"));
}

#[tokio::test]
async fn test_validation_loop_simulation() {
    // Simulate full validation loop: find → generate → verify
    let mock_responses = vec![
        // Response 1: find_bug_locations
        r#"[{
            "file_path": "src/auth.rs",
            "line_number": 42,
            "confidence": 0.9,
            "reasoning": "Token validation code"
        }]"#.to_string(),

        // Response 2: generate_patch
        r#"{
            "patch": "--- a/src/auth.rs\n+++ b/src/auth.rs\n@@ -42,1 +42,1 @@\n-    None\n+    Some(token)\n",
            "reasoning": "Fix initialization"
        }"#.to_string(),

        // Response 3: verify_patch_location
        r#"{
            "is_correct": true,
            "reasoning": "Location is correct",
            "issues": []
        }"#.to_string(),
    ];

    let provider = MockProvider::new(mock_responses);
    let agent = create_test_agent().await.unwrap();

    // Step 1: Find candidates
    let candidates = agent.find_bug_locations(
        "Token should not be None",
        std::path::Path::new("/tmp/test-repo"),
        &provider,
        "mock-model",
    ).await.unwrap();

    assert_eq!(candidates.len(), 1);
    let candidate = &candidates[0];

    // Step 2: Generate patch
    let proposal = agent.generate_patch(
        "Token should not be None",
        candidate,
        &provider,
        "mock-model",
    ).await.unwrap();

    assert!(proposal.patch.contains("Some(token)"));

    // Step 3: Verify
    let verification = agent.verify_patch_location(
        &proposal,
        "Token should not be None",
        &provider,
        "mock-model",
    ).await.unwrap();

    assert!(verification.is_correct);
    assert_eq!(provider.get_call_count(), 3);
}

#[tokio::test]
async fn test_multiple_attempts_simulation() {
    // Simulate validation loop with 2 failed attempts, 3rd succeeds
    let mock_responses = vec![
        // Attempt 1: Find 3 candidates
        r#"[
            {"file_path": "src/wrong1.rs", "line_number": 10, "confidence": 0.7, "reasoning": "First guess"},
            {"file_path": "src/wrong2.rs", "line_number": 20, "confidence": 0.6, "reasoning": "Second guess"},
            {"file_path": "src/correct.rs", "line_number": 30, "confidence": 0.9, "reasoning": "Actual location"}
        ]"#.to_string(),

        // Generate patch for candidate 1
        r#"{"patch": "wrong patch 1", "reasoning": "wrong"}"#.to_string(),

        // Verify candidate 1: FAIL
        r#"{"is_correct": false, "reasoning": "Wrong file", "issues": ["Not the right location"]}"#.to_string(),

        // Generate patch for candidate 2
        r#"{"patch": "wrong patch 2", "reasoning": "wrong"}"#.to_string(),

        // Verify candidate 2: FAIL
        r#"{"is_correct": false, "reasoning": "Still wrong", "issues": ["Variable not found"]}"#.to_string(),

        // Generate patch for candidate 3
        r#"{"patch": "correct patch", "reasoning": "correct"}"#.to_string(),

        // Verify candidate 3: SUCCESS
        r#"{"is_correct": true, "reasoning": "Correct location!", "issues": []}"#.to_string(),
    ];

    let provider = MockProvider::new(mock_responses);
    let agent = create_test_agent().await.unwrap();

    // Find candidates
    let candidates = agent.find_bug_locations(
        "Fix the bug",
        std::path::Path::new("/tmp/test-repo"),
        &provider,
        "mock-model",
    ).await.unwrap();

    assert_eq!(candidates.len(), 3);

    // Try candidates in order until one verifies
    let mut attempts = 0;
    let max_attempts = 3;
    let mut success = false;

    for candidate in candidates.iter().take(max_attempts) {
        attempts += 1;

        // Generate patch
        let proposal = agent.generate_patch(
            "Fix the bug",
            candidate,
            &provider,
            "mock-model",
        ).await.unwrap();

        // Verify
        let verification = agent.verify_patch_location(
            &proposal,
            "Fix the bug",
            &provider,
            "mock-model",
        ).await.unwrap();

        if verification.is_correct {
            success = true;
            break;
        }
    }

    assert!(success);
    assert_eq!(attempts, 3); // Took 3 attempts to find correct location
    assert_eq!(provider.get_call_count(), 7); // 1 find + 3*(generate + verify)
}

#[tokio::test]
async fn test_max_attempts_exceeded() {
    // All 3 candidates fail verification
    let mock_responses = vec![
        // Find 3 candidates
        r#"[
            {"file_path": "src/wrong1.rs", "line_number": 10, "confidence": 0.7, "reasoning": "Guess 1"},
            {"file_path": "src/wrong2.rs", "line_number": 20, "confidence": 0.6, "reasoning": "Guess 2"},
            {"file_path": "src/wrong3.rs", "line_number": 30, "confidence": 0.5, "reasoning": "Guess 3"}
        ]"#.to_string(),

        // Attempt 1
        r#"{"patch": "patch1", "reasoning": "try1"}"#.to_string(),
        r#"{"is_correct": false, "reasoning": "Wrong", "issues": ["Bad location"]}"#.to_string(),

        // Attempt 2
        r#"{"patch": "patch2", "reasoning": "try2"}"#.to_string(),
        r#"{"is_correct": false, "reasoning": "Wrong", "issues": ["Bad location"]}"#.to_string(),

        // Attempt 3
        r#"{"patch": "patch3", "reasoning": "try3"}"#.to_string(),
        r#"{"is_correct": false, "reasoning": "Wrong", "issues": ["Bad location"]}"#.to_string(),
    ];

    let provider = MockProvider::new(mock_responses);
    let agent = create_test_agent().await.unwrap();

    let candidates = agent.find_bug_locations(
        "Unfixable bug",
        std::path::Path::new("/tmp/test-repo"),
        &provider,
        "mock-model",
    ).await.unwrap();

    let max_attempts = 3;
    let mut attempts = 0;
    let mut success = false;

    for candidate in candidates.iter().take(max_attempts) {
        attempts += 1;

        let proposal = agent.generate_patch(
            "Unfixable bug",
            candidate,
            &provider,
            "mock-model",
        ).await.unwrap();

        let verification = agent.verify_patch_location(
            &proposal,
            "Unfixable bug",
            &provider,
            "mock-model",
        ).await.unwrap();

        if verification.is_correct {
            success = true;
            break;
        }
    }

    assert!(!success);
    assert_eq!(attempts, 3); // Hit max attempts
    assert_eq!(provider.get_call_count(), 7); // All attempts exhausted
}
