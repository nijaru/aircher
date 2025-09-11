# Concrete Intelligence Integration Plan

## Exact Integration Points in AgentController

### 1. Activate Intelligence Engine (Line 33)
```rust
// CURRENT (src/agent/controller.rs:33)
_intelligence: intelligence,

// CHANGE TO:
intelligence: Arc::new(Mutex::new(intelligence)),
```

### 2. Enhance System Prompt with Intelligence (Line 114)
```rust
// CURRENT (src/agent/controller.rs:114)
content: self.build_system_prompt(),

// CHANGE TO:
content: self.build_intelligent_system_prompt(user_message).await?,

// NEW METHOD:
async fn build_intelligent_system_prompt(&self, user_message: &str) -> Result<String> {
    let base_prompt = self.build_system_prompt();
    
    // Get intelligence insights
    let intelligence = self.intelligence.lock().await;
    
    // Find similar past situations
    let suggestions = intelligence.get_suggestions(user_message, None).await?;
    
    // Predict related files
    if let Some(file) = extract_file_mention(user_message) {
        let related_files = intelligence.predict_file_changes(&file).await?;
        
        return Ok(format!(
            "{}\n\n## Intelligence Context:\n{}\nRelated files: {:?}",
            base_prompt, suggestions, related_files
        ));
    }
    
    Ok(format!("{}\n\n## Intelligence Context:\n{}", base_prompt, suggestions))
}
```

### 3. Track Tool Execution (Line 191)
```rust
// CURRENT (src/agent/controller.rs:191)
let tool_results = self.execute_tools(&tool_calls).await;

// CHANGE TO:
let (tool_results, actions_taken) = self.execute_tools_with_tracking(&tool_calls).await;

// NEW METHOD:
async fn execute_tools_with_tracking(&self, tool_calls: &[ToolCall]) 
    -> (Vec<(String, Result<Value>)>, Vec<AgentAction>) 
{
    let mut results = Vec::new();
    let mut actions = Vec::new();
    
    for call in tool_calls {
        let start = std::time::Instant::now();
        let result = self.tools.execute(&call.name, &call.parameters).await;
        
        // Track the action
        actions.push(AgentAction {
            tool: call.name.clone(),
            params: call.parameters.clone(),
            success: result.is_ok(),
            duration_ms: start.elapsed().as_millis() as u64,
            result_summary: match &result {
                Ok(v) => format!("Success: {}", summarize_value(v)),
                Err(e) => format!("Error: {}", e),
            },
        });
        
        results.push((call.name.clone(), result));
    }
    
    (results, actions)
}
```

### 4. Record Pattern After Completion (After Line 166)
```rust
// AFTER BREAK (when conversation completes)
// ADD THIS before returning:

// Record the interaction pattern
if !tool_status_messages.is_empty() {
    self.record_interaction_pattern(
        user_message,
        &actions_taken,
        &final_response,
        iterations == 1 && !final_response.contains("error")
    ).await?;
}

// NEW METHOD:
async fn record_interaction_pattern(
    &self,
    user_message: &str,
    actions: &[AgentAction],
    response: &str,
    success: bool,
) -> Result<()> {
    let intelligence = self.intelligence.lock().await;
    
    // Extract files mentioned or modified
    let files = extract_files_from_actions(actions);
    
    // Create pattern
    let pattern = Pattern {
        id: uuid::Uuid::new_v4().to_string(),
        description: user_message.to_string(),
        context: user_message.to_string(),
        actions: actions.to_vec(),
        files_involved: files,
        success,
        timestamp: chrono::Utc::now(),
        session_id: self.conversation.session_id.clone(),
        embedding_text: user_message.to_string(),
    };
    
    intelligence.record_pattern(pattern).await?;
    Ok(())
}
```

## File Extraction Helpers

```rust
fn extract_file_mention(text: &str) -> Option<String> {
    // Look for file patterns like "main.rs", "src/lib.rs", etc.
    let file_regex = regex::Regex::new(r"\b[\w/]+\.\w+\b").unwrap();
    file_regex.find(text).map(|m| m.as_str().to_string())
}

fn extract_files_from_actions(actions: &[AgentAction]) -> Vec<String> {
    let mut files = Vec::new();
    for action in actions {
        if action.tool == "read_file" || action.tool == "write_file" {
            if let Some(path) = action.params.get("path").and_then(|v| v.as_str()) {
                files.push(path.to_string());
            }
        }
    }
    files.dedup();
    files
}

fn summarize_value(value: &Value) -> String {
    match value {
        Value::String(s) => {
            if s.len() > 100 {
                format!("{}...", &s[..100])
            } else {
                s.clone()
            }
        }
        Value::Object(map) => format!("{} fields", map.len()),
        Value::Array(arr) => format!("{} items", arr.len()),
        _ => value.to_string(),
    }
}
```

## Modified Process Message Flow

```rust
pub async fn process_message(&mut self, user_message: &str, provider: &dyn LLMProvider, model: &str) 
    -> Result<(String, Vec<String>)> 
{
    // 1. Validate auth (unchanged)
    self.validate_auth_for_request(provider, model).await?;
    
    // 2. Initialize pattern tracking
    let mut all_actions = Vec::new();
    
    // 3. Add user message (unchanged)
    self.conversation.messages.push(...);
    
    // 4. Main loop
    loop {
        // Build INTELLIGENT system prompt
        let system_prompt = self.build_intelligent_system_prompt(user_message).await?;
        
        // ... rest of loop ...
        
        // Track actions when executing tools
        let (tool_results, actions) = self.execute_tools_with_tracking(&tool_calls).await;
        all_actions.extend(actions);
        
        // ... rest of loop ...
    }
    
    // 5. Record pattern before returning
    if !all_actions.is_empty() {
        self.record_interaction_pattern(
            user_message,
            &all_actions,
            &final_response,
            !final_response.contains("error") && !final_response.contains("failed")
        ).await?;
    }
    
    Ok((final_response, tool_status_messages))
}
```

## Dependencies to Add

```toml
# In Cargo.toml
regex = "1.0"  # For file extraction
```

## Testing the Integration

### Test 1: Intelligence Enhancement
```rust
// User asks: "Fix compilation error in main.rs"
// Intelligence should provide:
// - "Similar error fixed by updating imports (85% success)"
// - "main.rs often changes with lib.rs"
```

### Test 2: Pattern Recording
```rust
// After fixing error:
// Pattern recorded with:
// - Actions: [read_file, edit_file, run_command]
// - Files: ["main.rs", "lib.rs"]
// - Success: true
```

### Test 3: Learning Over Time
```rust
// Second similar query:
// Intelligence suggests: "Based on past success: check imports first"
// Confidence increases with each success
```

## Thread Safety Fix for DuckDB

Since DuckDBConnection isn't Send, wrap operations:

```rust
// In duckdb_memory.rs
pub async fn record_pattern(&self, pattern: Pattern) -> Result<()> {
    // Use spawn_blocking for database operations
    let db = self.db.clone();
    tokio::task::spawn_blocking(move || {
        let db = db.blocking_lock();
        // ... database operations ...
    }).await??;
    Ok(())
}
```

## Value This Brings

With these changes:
1. **Every query** benefits from past learnings
2. **Every action** is tracked and analyzed
3. **Every success** makes the agent smarter
4. **Every failure** is avoided next time

The agent transforms from a stateless tool executor to an **intelligent assistant that learns and improves**.