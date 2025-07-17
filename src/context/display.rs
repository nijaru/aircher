use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextDisplay {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub context_window: u32,
    pub model: String,
    pub provider: String,
    pub estimated_cost: Option<f64>,
}

impl ContextDisplay {
    pub fn new(
        input_tokens: u32,
        output_tokens: u32,
        context_window: u32,
        model: String,
        provider: String,
        estimated_cost: Option<f64>,
    ) -> Self {
        Self {
            input_tokens,
            output_tokens,
            context_window,
            model,
            provider,
            estimated_cost,
        }
    }

    /// Generate a compact context usage display
    pub fn format_compact(&self) -> String {
        let total_tokens = self.input_tokens + self.output_tokens;
        let percentage = if self.context_window > 0 {
            (total_tokens as f32 / self.context_window as f32) * 100.0
        } else {
            0.0
        };

        let cost_str = if let Some(cost) = self.estimated_cost {
            if cost == 0.0 {
                "🆓".to_string()
            } else {
                format!("${:.3}", cost)
            }
        } else {
            "?".to_string()
        };

        format!(
            "💰 {} | 📊 {}/{} ({:.1}%) | {}",
            cost_str,
            format_tokens(total_tokens),
            format_tokens(self.context_window),
            percentage,
            self.model
        )
    }

    /// Generate a detailed context usage display
    pub fn format_detailed(&self) -> String {
        let total_tokens = self.input_tokens + self.output_tokens;
        let percentage = if self.context_window > 0 {
            (total_tokens as f32 / self.context_window as f32) * 100.0
        } else {
            0.0
        };

        let mut details = String::new();
        
        details.push_str(&format!("📊 Context Usage:\n"));
        details.push_str(&format!("  Model: {} ({})\n", self.model, self.provider));
        details.push_str(&format!("  Input tokens:  {}\n", format_tokens(self.input_tokens)));
        details.push_str(&format!("  Output tokens: {}\n", format_tokens(self.output_tokens)));
        details.push_str(&format!("  Total used:    {} / {} ({:.1}%)\n", 
            format_tokens(total_tokens), 
            format_tokens(self.context_window), 
            percentage
        ));

        if let Some(cost) = self.estimated_cost {
            if cost == 0.0 {
                details.push_str("  Cost: Free 🆓\n");
            } else {
                details.push_str(&format!("  Cost: ${:.4}\n", cost));
            }
        }

        // Add usage warnings/tips
        if percentage > 90.0 {
            details.push_str("  ⚠️  Near context limit - consider summarizing conversation\n");
        } else if percentage > 75.0 {
            details.push_str("  📈 High context usage - monitor for performance\n");
        } else if percentage < 10.0 {
            details.push_str("  ✅ Low context usage - plenty of room\n");
        }

        details
    }

    /// Get a context usage indicator for status bars
    pub fn get_status_indicator(&self) -> String {
        let total_tokens = self.input_tokens + self.output_tokens;
        let percentage = if self.context_window > 0 {
            (total_tokens as f32 / self.context_window as f32) * 100.0
        } else {
            0.0
        };

        let indicator = match percentage as u32 {
            0..=25 => "▁",
            26..=50 => "▃", 
            51..=75 => "▅",
            76..=90 => "▇",
            _ => "█",
        };

        let cost_indicator = if let Some(cost) = self.estimated_cost {
            if cost == 0.0 {
                "🆓"
            } else if cost < 0.01 {
                "💰"
            } else {
                "💸"
            }
        } else {
            "💰"
        };

        format!("{} {} {:.0}%", cost_indicator, indicator, percentage)
    }

    /// Check if context usage is concerning
    pub fn needs_attention(&self) -> Option<String> {
        let total_tokens = self.input_tokens + self.output_tokens;
        let percentage = if self.context_window > 0 {
            (total_tokens as f32 / self.context_window as f32) * 100.0
        } else {
            0.0
        };

        if percentage > 95.0 {
            Some("Context nearly full - responses may be truncated".to_string())
        } else if percentage > 85.0 {
            Some("High context usage - consider starting a new conversation".to_string())
        } else if let Some(cost) = self.estimated_cost {
            if cost > 0.10 {
                Some(format!("High cost request: ${:.3}", cost))
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Format token count in a human-readable way
pub fn format_tokens(tokens: u32) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f32 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}k", tokens as f32 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

/// Create a progress bar for context usage
pub fn context_progress_bar(used: u32, total: u32, width: usize) -> String {
    if total == 0 {
        return "▁".repeat(width);
    }

    let percentage = used as f32 / total as f32;
    let filled = (percentage * width as f32) as usize;
    let empty = width.saturating_sub(filled);

    let bar_char = if percentage > 0.9 {
        "█" // Red zone
    } else if percentage > 0.75 {
        "▇" // Orange zone  
    } else if percentage > 0.5 {
        "▅" // Yellow zone
    } else {
        "▃" // Green zone
    };

    format!("{}{}", bar_char.repeat(filled), "▁".repeat(empty))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(500), "500");
        assert_eq!(format_tokens(1500), "1.5k");
        assert_eq!(format_tokens(1_500_000), "1.5M");
    }

    #[test]
    fn test_context_display() {
        let display = ContextDisplay::new(
            1000, 500, 8000, 
            "gpt-4o".to_string(), 
            "openai".to_string(), 
            Some(0.025)
        );

        let compact = display.format_compact();
        assert!(compact.contains("$0.025"));
        assert!(compact.contains("1.5k/8.0k"));
        assert!(compact.contains("gpt-4o"));
    }

    #[test]
    fn test_needs_attention() {
        let high_context = ContextDisplay::new(
            7500, 500, 8000,
            "test".to_string(),
            "test".to_string(),
            None
        );
        
        assert!(high_context.needs_attention().is_some());
        
        let normal_context = ContextDisplay::new(
            1000, 500, 8000,
            "test".to_string(), 
            "test".to_string(),
            None
        );
        
        assert!(normal_context.needs_attention().is_none());
    }

    #[test]
    fn test_progress_bar() {
        let bar = context_progress_bar(2000, 8000, 10);
        assert_eq!(bar.len(), 10);
        // Should be about 25% filled
        assert!(bar.starts_with("▃▃"));
    }
}