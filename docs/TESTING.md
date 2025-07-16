# Aircher Testing Framework

## Overview

Aircher includes a comprehensive testing framework that enables testing of all major components, including the TUI interface, without requiring actual terminal interactions or external dependencies.

## Test Architecture

### Dependency Injection

The testing framework uses dependency injection to replace real components with mock implementations:

```rust
// Enable testing features
cargo test --features testing

// Mock implementations replace real components
MockProvider          // Simulates AI provider responses
MockIntelligenceTools // Simulates intelligence engine
MockSessionManager    // Simulates session persistence
```

### Test Categories

#### 1. Unit Tests
- Individual component functionality
- Provider-specific behavior
- Intelligence engine algorithms
- Session management operations

#### 2. Integration Tests
- Complete TUI workflows
- Multi-provider scenarios
- Session persistence flows
- Error handling paths

#### 3. Performance Tests
- Response time validation
- Memory usage patterns
- Concurrent operation handling

## Running Tests

### All Tests
```bash
cargo test
```

### TUI Integration Tests
```bash
cargo test --test tui_integration_tests --features testing
```

### Specific Test Categories
```bash
# Session management tests
cargo test --test session_tests

# Intelligence engine tests
cargo test --test intelligence_tests

# Provider integration tests
cargo test --test integration_tests
```

## Test Structure

### Mock Components

#### MockProvider
```rust
let mock_provider = MockProvider::new("test-provider".to_string());
mock_provider.add_response("Mock AI response".to_string());

let response = mock_provider.chat(&request).await?;
assert_eq!(response.content, "Mock AI response");
```

#### MockIntelligenceTools
```rust
let mock_intelligence = MockIntelligenceTools::new();
let context = mock_intelligence.get_development_context("query").await;
assert_eq!(context.development_phase, "Mock phase");
```

#### MockSessionManager
```rust
let mock_session_manager = MockSessionManager::new();
let session = mock_session_manager.create_session(
    "Test Session".to_string(),
    "provider".to_string(),
    "model".to_string(),
    None,
    vec![],
).await?;
```

### Test Scenarios

#### Complete TUI Workflow
```rust
#[tokio::test]
async fn test_tui_session_flow() -> Result<()> {
    // Setup mocks
    let mock_provider = MockProvider::new("test-provider".to_string());
    let mock_intelligence = MockIntelligenceTools::new();
    let mock_session_manager = MockSessionManager::new();
    
    // Test session creation
    let session = mock_session_manager.create_session(/*...*/).await?;
    
    // Test message exchange
    let user_message = Message::user("Hello".to_string());
    mock_session_manager.add_message(&session.id, &user_message).await?;
    
    // Test AI response
    let ai_response = mock_provider.chat(&request).await?;
    
    // Test intelligence context
    let context = mock_intelligence.get_development_context("query").await;
    
    // Verify all interactions
    assert_eq!(mock_provider.get_call_count(), 1);
    // ... additional assertions
}
```

## Test Data Management

### Temporary Databases
```rust
// Tests use isolated temporary databases
let temp_dir = tempdir()?;
let sessions_db = temp_dir.path().join("test_sessions.db");
```

### Test Serialization
```rust
// Tests run serially to avoid database conflicts
static TEST_MUTEX: Mutex<()> = Mutex::new(());
let _guard = TEST_MUTEX.lock().await;
```

## Authentication Testing

All providers use simple API key authentication:
- **Claude**: `ANTHROPIC_API_KEY`
- **Gemini**: `GOOGLE_API_KEY`
- **OpenAI**: `OPENAI_API_KEY`
- **OpenRouter**: `OPENROUTER_API_KEY`

Mock providers simulate authentication without requiring real API keys.

## Test Coverage

### What We Can Test
- ✅ **Session Management** - Create, persist, load conversations
- ✅ **Intelligence Engine** - Context analysis, impact assessment
- ✅ **Provider Interactions** - Multiple AI provider scenarios
- ✅ **Error Handling** - Graceful degradation
- ✅ **Performance** - Response time validation
- ✅ **TUI Workflows** - Complete user interaction flows

### What Requires Manual Testing
- ❌ **Terminal Rendering** - Visual TUI appearance
- ❌ **Keyboard Input** - Actual key press handling
- ❌ **Real API Calls** - Live provider responses
- ❌ **File System Monitoring** - Actual file change detection

## Adding New Tests

### Creating Mock Components
1. Implement the relevant trait for your mock
2. Add call logging for verification
3. Provide configurable responses
4. Include in the `testing` module

### Writing Integration Tests
1. Create test scenarios that mirror real usage
2. Use dependency injection to insert mocks
3. Verify all expected interactions occurred
4. Test error conditions and edge cases

### Test Best Practices
- Use isolated test data (temporary directories)
- Test both success and failure paths
- Verify mock interactions (call counts, parameters)
- Keep tests focused and independent
- Use descriptive test names and assertions

## Debugging Tests

### Verbose Output
```bash
cargo test -- --nocapture
```

### Specific Test Debugging
```bash
cargo test --test tui_integration_tests test_tui_session_flow -- --nocapture
```

### Mock Call Inspection
```rust
let calls = mock_intelligence.get_calls();
println!("Intelligence calls: {:?}", calls);
```

This testing framework ensures that Aircher's complex TUI and intelligence systems can be thoroughly validated without external dependencies, providing confidence in the system's reliability and correctness.