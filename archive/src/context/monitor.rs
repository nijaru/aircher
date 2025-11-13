use std::time::Instant;
use serde::{Deserialize, Serialize};

/// Monitors context window usage and determines when compaction is needed
#[derive(Debug, Clone)]
pub struct ContextMonitor {
    /// Current token usage
    pub current_usage: u32,
    /// Total context window size
    pub context_window: u32,
    /// Warning threshold (e.g., 0.75 = 75%)
    pub warning_threshold: f32,
    /// Critical threshold (e.g., 0.90 = 90%)
    pub critical_threshold: f32,
    /// Last time we checked/updated
    pub last_check: Instant,
    /// Last compaction trigger returned
    last_trigger: CompactionTrigger,
}

/// Triggers for when compaction should occur
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompactionTrigger {
    /// No compaction needed
    None,
    /// Usage is high, show warning
    Warning,
    /// Usage is critical, should compact soon
    Critical,
    /// User manually requested compaction
    UserForced,
    /// Periodic compaction based on time/messages
    Periodic,
}

/// Configuration for auto-compaction behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    /// Enable automatic compaction
    pub auto_enabled: bool,
    /// Warning threshold (0.0 - 1.0)
    pub warning_threshold: f32,
    /// Critical threshold (0.0 - 1.0)
    pub critical_threshold: f32,
    /// Minimum messages before allowing compaction
    pub min_messages: u32,
    /// Number of recent messages to keep
    pub keep_recent_messages: usize,
    /// Keep system messages
    pub keep_system_messages: bool,
    /// Keep tool result messages
    pub keep_tool_results: bool,
    /// Summarization depth
    pub summary_depth: SummaryDepth,
    /// Preserve code blocks in summaries
    pub preserve_code_blocks: bool,
    /// Preserve file paths in summaries
    pub preserve_file_paths: bool,
    /// Show warnings to user
    pub show_warnings: bool,
    /// Require user confirmation for non-critical compactions
    pub require_confirmation: bool,
}

/// How detailed the summary should be
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SummaryDepth {
    /// Key points only
    Brief,
    /// Balanced summary
    Standard,
    /// Comprehensive summary
    Detailed,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            auto_enabled: true,
            warning_threshold: 0.75,
            critical_threshold: 0.90,
            min_messages: 10,
            keep_recent_messages: 5,
            keep_system_messages: true,
            keep_tool_results: true,
            summary_depth: SummaryDepth::Standard,
            preserve_code_blocks: true,
            preserve_file_paths: true,
            show_warnings: true,
            require_confirmation: true,
        }
    }
}

impl ContextMonitor {
    /// Create a new context monitor
    pub fn new(context_window: u32, config: &CompactionConfig) -> Self {
        Self {
            current_usage: 0,
            context_window,
            warning_threshold: config.warning_threshold,
            critical_threshold: config.critical_threshold,
            last_check: Instant::now(),
            last_trigger: CompactionTrigger::None,
        }
    }

    /// Update the current token usage
    pub fn update_usage(&mut self, tokens: u32) {
        self.current_usage = tokens;
        self.last_check = Instant::now();
    }

    /// Add tokens to current usage
    pub fn add_tokens(&mut self, tokens: u32) {
        self.current_usage = self.current_usage.saturating_add(tokens);
        self.last_check = Instant::now();
    }

    /// Get current usage as a percentage (0.0 - 1.0)
    pub fn usage_percentage(&self) -> f32 {
        if self.context_window == 0 {
            return 0.0;
        }
        (self.current_usage as f32 / self.context_window as f32).min(1.0)
    }

    /// Get current usage as a percentage (0 - 100)
    pub fn usage_percentage_display(&self) -> u8 {
        (self.usage_percentage() * 100.0) as u8
    }

    /// Check if compaction should be triggered
    pub fn should_compact(&mut self) -> CompactionTrigger {
        let usage = self.usage_percentage();

        let trigger = if usage >= self.critical_threshold {
            CompactionTrigger::Critical
        } else if usage >= self.warning_threshold {
            CompactionTrigger::Warning
        } else {
            CompactionTrigger::None
        };

        // Only return a trigger if it's different from last time
        // This prevents spamming warnings
        if trigger != self.last_trigger {
            self.last_trigger = trigger;
            trigger
        } else {
            CompactionTrigger::None
        }
    }

    /// Force a compaction trigger check (ignoring last trigger)
    pub fn force_check(&mut self) -> CompactionTrigger {
        let usage = self.usage_percentage();

        if usage >= self.critical_threshold {
            CompactionTrigger::Critical
        } else if usage >= self.warning_threshold {
            CompactionTrigger::Warning
        } else {
            CompactionTrigger::None
        }
    }

    /// Get a status message for the current usage
    pub fn get_status_message(&self) -> Option<String> {
        let usage = self.usage_percentage();
        let percentage = self.usage_percentage_display();

        if usage >= self.critical_threshold {
            Some(format!("🚨 Context nearly full ({}%) - Compaction recommended", percentage))
        } else if usage >= self.warning_threshold {
            Some(format!("⚠️ Context usage high ({}%) - Consider compacting", percentage))
        } else if usage >= 0.5 {
            None // Don't show messages below warning threshold
        } else {
            None
        }
    }

    /// Get a short status indicator for the status bar
    pub fn get_status_indicator(&self) -> String {
        let usage = self.usage_percentage();
        let percentage = self.usage_percentage_display();

        let icon = if usage >= self.critical_threshold {
            "🚨"
        } else if usage >= self.warning_threshold {
            "⚠️"
        } else if usage >= 0.5 {
            "📊"
        } else {
            "✅"
        };

        format!("{} {}%", icon, percentage)
    }

    /// Check if we have enough room for a response
    pub fn has_room_for_response(&self, estimated_response_tokens: u32) -> bool {
        let future_usage = self.current_usage.saturating_add(estimated_response_tokens);
        let future_percentage = future_usage as f32 / self.context_window as f32;
        future_percentage < 1.0
    }

    /// Reset the monitor after compaction
    pub fn reset_after_compaction(&mut self, new_usage: u32) {
        self.current_usage = new_usage;
        self.last_check = Instant::now();
        self.last_trigger = CompactionTrigger::None;
    }
}

/// Calculate importance score for a message
pub fn calculate_message_importance(
    content: &str,
    role: &str,
    age_seconds: f64,
) -> f32 {
    let mut score: f32 = 0.0;

    // Recency bonus (messages in last 5 minutes get full bonus)
    let recency_bonus = if age_seconds <= 300.0 {
        1.0
    } else if age_seconds < 1800.0 {
        0.5
    } else {
        0.1
    };
    score += recency_bonus * 0.3;

    // Content type bonuses
    if content.contains("```") {
        score += 0.3; // Code blocks
    }
    if content.contains("File:") || content.contains("Path:") || content.contains("/") {
        score += 0.2; // File references
    }
    if content.contains("Error:") || content.contains("error:") {
        score += 0.2; // Error messages
    }
    if content.contains("DECISION:") || content.contains("TODO:") || content.contains("IMPORTANT:") {
        score += 0.3; // Explicit markers
    }

    // Role bonuses
    match role {
        "tool" => score += 0.4,
        "system" => score += 0.3,
        _ => {}
    }

    // Length penalty for very long messages
    if content.len() > 3000 {
        score -= 0.1;
    }

    score.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_monitor_thresholds() {
        let config = CompactionConfig::default();
        let mut monitor = ContextMonitor::new(1000, &config);

        // Below warning
        monitor.update_usage(500);
        assert_eq!(monitor.usage_percentage(), 0.5);
        assert_eq!(monitor.should_compact(), CompactionTrigger::None);

        // At warning threshold
        monitor.update_usage(750);
        assert_eq!(monitor.usage_percentage(), 0.75);
        assert_eq!(monitor.should_compact(), CompactionTrigger::Warning);

        // At critical threshold
        monitor.update_usage(900);
        assert_eq!(monitor.usage_percentage(), 0.9);
        assert_eq!(monitor.should_compact(), CompactionTrigger::Critical);
    }

    #[test]
    fn test_message_importance() {
        // Recent message with code
        let score1 = calculate_message_importance("```python\nprint('hello')\n```", "user", 60.0);
        assert!(score1 > 0.5);

        // Old message without important content
        let score2 = calculate_message_importance("Just chatting", "user", 3600.0);
        assert!(score2 < 0.3);

        // Tool message (high importance)
        let score3 = calculate_message_importance("File saved: main.py", "tool", 300.0);
        assert!(score3 > 0.6);
    }
}
