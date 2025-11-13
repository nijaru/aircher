/// Context management tools for inspecting and editing conversation context
///
/// Phase 2: list_context tool - allows agent to see what's in context
/// Phase 3: edit_context tool - allows agent to remove/summarize items (future)

use super::{AgentTool, ToolError, ToolOutput};
use crate::agent::dynamic_context::DynamicContextManager;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

/// Tool for listing context items with token costs
///
/// Allows the agent to inspect what's currently in the conversation context,
/// helping it make informed decisions about context management.
#[derive(Clone)]
pub struct ListContextTool {
    context_manager: Arc<DynamicContextManager>,
}

impl ListContextTool {
    pub fn new(context_manager: Arc<DynamicContextManager>) -> Self {
        Self { context_manager }
    }
}

#[async_trait]
impl AgentTool for ListContextTool {
    fn name(&self) -> &str {
        "list_context"
    }

    fn description(&self) -> &str {
        "List all items currently in the conversation context with their token costs and relevance scores. \
         Use this to understand what's taking up context space and make informed decisions about \
         what to keep or remove. Returns items sorted by relevance (highest first)."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of items to return (default: 20, max: 100)",
                    "default": 20,
                    "minimum": 1,
                    "maximum": 100
                },
                "include_low_relevance": {
                    "type": "boolean",
                    "description": "Include items with low relevance scores (<0.3). Default: false",
                    "default": false
                }
            }
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        // Parse parameters
        let limit = params.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(20)
            .min(100) as usize;

        let include_low_relevance = params.get("include_low_relevance")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Get context summary for overall stats
        let summary = self.context_manager.get_context_summary().await;

        // Get context items (we'll need to add this method to DynamicContextManager)
        // For now, return summary stats
        let remaining_tokens = summary.token_limit.saturating_sub(summary.token_usage);
        let utilization_pct = (summary.utilization * 100.0).round() as u32;

        // Build response with context stats
        let mut response_text = format!(
            "# Context Overview\n\n\
            **Total Usage**: {}/{} tokens ({:.1}% full)\n\
            **Remaining**: {} tokens\n\
            **Items**: {}\n\n",
            summary.token_usage,
            summary.token_limit,
            summary.utilization,
            remaining_tokens,
            summary.total_items
        );

        // Add guidance based on utilization
        if utilization_pct >= 80 {
            response_text.push_str("⚠️ **High utilization** (>80%) - Consider summarizing or removing old items\n\n");
        } else if utilization_pct >= 60 {
            response_text.push_str("⚡ **Moderate utilization** (60-80%) - Monitor context usage\n\n");
        } else {
            response_text.push_str("✅ **Healthy utilization** (<60%) - Plenty of space remaining\n\n");
        }

        // Add top items by relevance
        response_text.push_str("## Top Items by Relevance\n\n");

        let top_items: Vec<_> = summary.top_items
            .iter()
            .filter(|(_, score)| include_low_relevance || *score >= 0.3)
            .take(limit)
            .collect();

        if top_items.is_empty() {
            response_text.push_str("(No items found matching criteria)\n");
        } else {
            for (i, (item_id, score)) in top_items.iter().enumerate() {
                response_text.push_str(&format!(
                    "{}. Item {:?} - Relevance: {:.2}\n",
                    i + 1,
                    item_id,
                    score
                ));
            }
        }

        // Add recommendations
        response_text.push_str("\n## Recommendations\n\n");

        if utilization_pct >= 80 {
            response_text.push_str(
                "- Consider being more concise in responses\n\
                - Summarize completed tasks instead of keeping full discussion\n\
                - Focus only on current task context\n"
            );
        } else if utilization_pct >= 60 {
            response_text.push_str(
                "- Monitor context usage as you continue\n\
                - Consider summarizing older discussions if nearing 80%\n"
            );
        } else {
            response_text.push_str(
                "- Current context usage is healthy\n\
                - Continue current approach\n"
            );
        }

        Ok(ToolOutput {
            success: true,
            result: json!({
                "overview": {
                    "total_tokens": summary.token_usage,
                    "max_tokens": summary.token_limit,
                    "utilization_percent": utilization_pct,
                    "remaining_tokens": remaining_tokens,
                    "total_items": summary.total_items
                },
                "items_shown": top_items.len(),
                "text": response_text
            }),
            error: None,
            usage: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intelligence::IntelligenceEngine;
    use crate::config::ProjectContext;
    use std::path::PathBuf;

    async fn create_test_context_manager() -> Arc<DynamicContextManager> {
        // Create a minimal intelligence engine for testing
        let project_context = ProjectContext {
            root_path: PathBuf::from("/tmp/test"),
            config_path: None,
        };

        let intelligence = Arc::new(
            IntelligenceEngine::new(project_context)
                .await
                .expect("Failed to create intelligence engine")
        );

        Arc::new(DynamicContextManager::new(intelligence, None))
    }

    #[tokio::test]
    async fn test_list_context_basic() {
        let context_manager = create_test_context_manager().await;
        let tool = ListContextTool::new(context_manager);

        let params = json!({});
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);

        // Should have overview data
        let overview = &output.result["overview"];
        assert!(overview["total_tokens"].is_number());
        assert!(overview["max_tokens"].is_number());
    }

    #[tokio::test]
    async fn test_list_context_with_limit() {
        let context_manager = create_test_context_manager().await;
        let tool = ListContextTool::new(context_manager);

        let params = json!({
            "limit": 10,
            "include_low_relevance": true
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.result["items_shown"].as_u64().unwrap() <= 10);
    }

    #[tokio::test]
    async fn test_list_context_schema() {
        let context_manager = create_test_context_manager().await;
        let tool = ListContextTool::new(context_manager);

        let schema = tool.parameters_schema();
        assert!(schema["properties"]["limit"].is_object());
        assert!(schema["properties"]["include_low_relevance"].is_object());
    }
}
