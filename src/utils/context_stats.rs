// Context window statistics formatting utilities

use crate::intelligence::working_memory::ContextWindowStats;

/// Format context window statistics as a human-readable string
///
/// # Format
/// Returns a string in the format:
/// `"Context: N items, X/Y tokens (Z% full), P prunings"`
///
/// # Example
/// ```
/// use aircher::intelligence::working_memory::ContextWindowStats;
/// use aircher::utils::context_stats::format_context_stats;
///
/// let stats = ContextWindowStats {
///     total_items: 42,
///     total_tokens: 90000,
///     max_tokens: 180000,
///     utilization: 50.0,
///     pruning_count: 3,
///     sticky_items: 1,
/// };
///
/// let formatted = format_context_stats(&stats);
/// assert_eq!(
///     formatted,
///     "Context: 42 items, 90000/180000 tokens (50.0% full), 3 prunings"
/// );
/// ```
pub fn format_context_stats(stats: &ContextWindowStats) -> String {
    format!(
        "Context: {} items, {}/{} tokens ({:.1}% full), {} prunings",
        stats.total_items,
        stats.total_tokens,
        stats.max_tokens,
        stats.utilization,
        stats.pruning_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_context_stats() {
        let stats = ContextWindowStats {
            total_items: 42,
            total_tokens: 90000,
            max_tokens: 180000,
            utilization: 50.0,
            pruning_count: 3,
            sticky_items: 1,
        };

        let formatted = format_context_stats(&stats);
        assert_eq!(
            formatted,
            "Context: 42 items, 90000/180000 tokens (50.0% full), 3 prunings"
        );
    }

    #[test]
    fn test_format_context_stats_high_utilization() {
        let stats = ContextWindowStats {
            total_items: 150,
            total_tokens: 175000,
            max_tokens: 180000,
            utilization: 97.2,
            pruning_count: 15,
            sticky_items: 2,
        };

        let formatted = format_context_stats(&stats);
        assert_eq!(
            formatted,
            "Context: 150 items, 175000/180000 tokens (97.2% full), 15 prunings"
        );
    }

    #[test]
    fn test_format_context_stats_no_pruning() {
        let stats = ContextWindowStats {
            total_items: 10,
            total_tokens: 5000,
            max_tokens: 180000,
            utilization: 2.8,
            pruning_count: 0,
            sticky_items: 1,
        };

        let formatted = format_context_stats(&stats);
        assert_eq!(
            formatted,
            "Context: 10 items, 5000/180000 tokens (2.8% full), 0 prunings"
        );
    }
}
