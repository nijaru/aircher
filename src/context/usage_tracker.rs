use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub context_window: u32,
    pub percentage_used: f32,
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUsageTracker {
    pub conversation_history: Vec<ContextUsage>,
    pub running_total: u32,
    pub session_peak: u32,
}

impl ContextUsageTracker {
    pub fn new() -> Self {
        Self {
            conversation_history: Vec::new(),
            running_total: 0,
            session_peak: 0,
        }
    }

    pub fn record_usage(&mut self, usage: ContextUsage) {
        self.running_total += usage.input_tokens + usage.output_tokens;
        if self.running_total > self.session_peak {
            self.session_peak = self.running_total;
        }
        self.conversation_history.push(usage);
    }

    pub fn format_usage_display(&self, current_usage: &ContextUsage) -> String {
        let total_tokens = current_usage.input_tokens + current_usage.output_tokens;
        let percentage = (total_tokens as f32 / current_usage.context_window as f32) * 100.0;
        
        format!(
            "📊 Context: {}/{} tokens ({:.1}%) | Session: {} total | Peak: {}",
            total_tokens,
            current_usage.context_window,
            percentage,
            self.running_total,
            self.session_peak
        )
    }

    pub fn get_efficiency_insights(&self) -> Option<String> {
        if self.conversation_history.len() < 2 {
            return None;
        }

        let avg_usage = self.running_total / self.conversation_history.len() as u32;
        let last_usage = &self.conversation_history.last()?;
        let current_total = last_usage.input_tokens + last_usage.output_tokens;

        if current_total > avg_usage * 2 {
            Some("💡 This request used significantly more context than average".to_string())
        } else if percentage_used_trend_increasing(&self.conversation_history) {
            Some("📈 Context usage trending upward - consider summarizing if needed".to_string())
        } else {
            None
        }
    }
}

fn percentage_used_trend_increasing(history: &[ContextUsage]) -> bool {
    if history.len() < 3 {
        return false;
    }

    let recent = &history[history.len()-3..];
    recent[0].percentage_used < recent[1].percentage_used && 
    recent[1].percentage_used < recent[2].percentage_used
}

impl Default for ContextUsageTracker {
    fn default() -> Self {
        Self::new()
    }
}