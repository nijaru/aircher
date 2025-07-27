# Auto-Compaction Strategy for Context Limits

## Overview
Automatically manage conversation context to prevent hitting model token limits while preserving conversation coherence and user experience.

## Core Strategy

### Trigger Conditions
```rust
pub struct CompactionConfig {
    pub trigger_threshold: f64,     // 0.85 = trigger at 85% of context
    pub target_threshold: f64,      // 0.60 = compact down to 60% 
    pub emergency_threshold: f64,   // 0.95 = emergency compact
    pub buffer_tokens: u32,         // 8000 = safety buffer
}
```

### Multi-Level Compaction
1. **Preventive** (85% context): Smart AI summarization
2. **Aggressive** (95% context): Template-based compaction  
3. **Emergency** (API rejection): Mechanical truncation

## Implementation

### 1. Token Estimation
```rust
pub struct ContextMonitor {
    current_tokens: u32,
    context_limit: u32,
    model_name: String,
}

impl ContextMonitor {
    pub fn estimate_tokens(&self, messages: &[Message]) -> u32 {
        // Use tiktoken-rs or similar for accurate counting
        messages.iter()
            .map(|msg| self.count_message_tokens(msg))
            .sum()
    }
    
    pub fn needs_compaction(&self, messages: &[Message]) -> CompactionUrgency {
        let tokens = self.estimate_tokens(messages);
        let percentage = tokens as f64 / self.context_limit as f64;
        
        match percentage {
            p if p >= 0.95 => CompactionUrgency::Emergency,
            p if p >= 0.85 => CompactionUrgency::Recommended, 
            _ => CompactionUrgency::None,
        }
    }
}
```

### 2. Smart Compaction
```rust
pub async fn smart_compact(&mut self, messages: &[Message]) -> Result<Vec<Message>> {
    // Keep system message + last few exchanges intact
    let (system_msgs, conversation) = split_system_and_conversation(messages);
    let (recent_msgs, history) = split_recent_and_history(&conversation, 5);
    
    // Summarize the history while preserving recent context
    let summary = self.summarize_history(history).await?;
    
    // Reconstruct: System + Summary + Recent
    let mut compacted = system_msgs;
    compacted.push(create_summary_message(summary));
    compacted.extend(recent_msgs);
    
    Ok(compacted)
}

async fn summarize_history(&self, history: &[Message]) -> Result<String> {
    let prompt = "Summarize this conversation history, preserving key decisions, code changes, and context needed for ongoing work. Be concise but comprehensive:";
    
    // Use a smaller model for summarization to save costs
    let summary_request = ChatRequest::new(
        history.to_vec(),
        "claude-haiku-3".to_string(), // Cheaper model for summarization
    );
    
    let response = self.provider_manager.chat(&summary_request).await?;
    Ok(response.content)
}
```

### 3. Fallback Strategies
```rust
pub async fn fallback_compact(&mut self, messages: &[Message]) -> Vec<Message> {
    // Strategy 1: Template-based summarization
    if let Ok(compacted) = self.template_compact(messages).await {
        return compacted;
    }
    
    // Strategy 2: Keep system + recent messages
    let mut result = vec![];
    
    // Always keep system messages
    result.extend(messages.iter()
        .filter(|m| m.role == MessageRole::System)
        .cloned());
    
    // Keep last 10 exchanges (20 messages)
    let recent_threshold = messages.len().saturating_sub(20);
    result.extend(messages[recent_threshold..].iter().cloned());
    
    result
}
```

### 4. Error Handling
```rust
impl TuiManager {
    async fn handle_ai_message_with_compaction(&mut self, message: String) -> Result<()> {
        loop {
            // Check if compaction needed
            if self.context_monitor.needs_compaction(&self.messages) != CompactionUrgency::None {
                self.auto_compact().await?;
            }
            
            // Try to send message
            match self.send_to_ai(&message).await {
                Ok(response) => {
                    self.add_message(response);
                    return Ok(());
                }
                Err(e) if self.is_context_error(&e) => {
                    warn!("Context error despite compaction: {}", e);
                    // Emergency compaction and retry once
                    self.emergency_compact().await?;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }
    
    async fn auto_compact(&mut self) -> Result<()> {
        self.add_message(Message::new(
            MessageRole::System,
            "⚡ Compacting conversation history to manage context length...".to_string(),
        ));
        
        let original_count = self.messages.len();
        let original_tokens = self.context_monitor.estimate_tokens(&self.messages);
        
        self.messages = self.smart_compact(&self.messages).await
            .unwrap_or_else(|_| self.fallback_compact(&self.messages).await);
        
        let new_tokens = self.context_monitor.estimate_tokens(&self.messages);
        let saved_tokens = original_tokens.saturating_sub(new_tokens);
        
        self.add_message(Message::new(
            MessageRole::System,
            format!(
                "✅ Compacted {} messages, saved ~{} tokens ({:.1}% reduction)",
                original_count - self.messages.len(),
                saved_tokens,
                (saved_tokens as f64 / original_tokens as f64) * 100.0
            ),
        ));
        
        Ok(())
    }
}
```

## Configuration

### User Controls
```toml
[compaction]
enabled = true
trigger_threshold = 0.85        # Start compacting at 85% context
target_threshold = 0.60         # Compact down to 60% context  
preserve_recent_exchanges = 5   # Always keep last 5 Q&A pairs
use_ai_summarization = true     # Use AI for smart summaries
fallback_to_truncation = true   # Allow mechanical truncation as last resort
notify_user = true              # Show compaction messages

# Model-specific overrides
[compaction.models]
"claude-opus-4" = { trigger_threshold = 0.90 }  # More generous for high-context models
"claude-haiku-3" = { trigger_threshold = 0.80 } # More aggressive for smaller models
```

## Benefits

1. **Transparent**: User sees what's happening
2. **Reliable**: Multiple fallback strategies  
3. **Efficient**: Uses cheaper models for summarization
4. **Preserves Context**: Keeps recent conversation intact
5. **Recovers Gracefully**: Handles API rejections
6. **Configurable**: User can tune behavior

## Integration Points

- **Token Counting**: Integrate with tiktoken-rs or similar
- **Model Selection**: Use cheaper models for summarization  
- **Error Handling**: Hook into existing error recovery
- **UI Feedback**: Show compaction progress in status bar
- **Session Management**: Preserve compacted history in sessions

This approach ensures reliable operation while maintaining conversation quality and user control.