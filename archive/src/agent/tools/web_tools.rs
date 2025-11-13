use super::{AgentTool, ToolError, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use reqwest::Client;
use scraper::{Html, Selector};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct WebBrowsingTool {
    #[allow(dead_code)]
    client: Client,
}

#[derive(Debug, Deserialize)]
struct WebBrowseParams {
    url: String,
    #[serde(default)]
    selector: Option<String>,
    #[serde(default = "default_timeout")]
    timeout_seconds: u64,
    #[serde(default)]
    #[allow(dead_code)]
    follow_redirects: bool,
}

#[derive(Debug, Clone)]
pub struct WebSearchTool {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct WebSearchParams {
    query: String,
    #[serde(default = "default_max_results")]
    max_results: usize,
    #[serde(default)]
    search_engine: String, // "duckduckgo", "google", etc.
}

fn default_timeout() -> u64 { 30 }
fn default_max_results() -> usize { 10 }

impl WebBrowsingTool {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Aircher AI Agent/1.0 (Web Browsing)")
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client }
    }

    async fn fetch_and_parse(&self, url: &str, selector: Option<&str>, timeout: u64) -> Result<String, ToolError> {
        // Create client with specific timeout
        let client = Client::builder()
            .user_agent("Aircher AI Agent/1.0 (Web Browsing)")
            .timeout(Duration::from_secs(timeout))
            .build()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create HTTP client: {}", e)))?;

        // Fetch the URL
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to fetch URL {}: {}", url, e)))?;

        // Check status
        if !response.status().is_success() {
            return Err(ToolError::ExecutionFailed(format!(
                "HTTP error {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown error")
            )));
        }

        // Get content type before consuming response
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("text/html")
            .to_string();

        // Get text content
        let text = response
            .text()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read response body: {}", e)))?;

        // If it's HTML and we have a selector, parse it
        if content_type.contains("text/html") {
            let document = Html::parse_document(&text);

            if let Some(sel) = selector {
                match Selector::parse(sel) {
                    Ok(selector) => {
                        let mut results = Vec::new();
                        for element in document.select(&selector) {
                            results.push(element.text().collect::<Vec<_>>().join(" "));
                        }
                        if results.is_empty() {
                            return Ok(format!("No elements found matching selector: {}", sel));
                        }
                        return Ok(results.join("\n\n"));
                    }
                    Err(e) => {
                        return Err(ToolError::InvalidParameters(format!("Invalid CSS selector '{}': {}", sel, e)));
                    }
                }
            }

            // No selector - extract main content
            self.extract_main_content(&document)
        } else {
            // Non-HTML content, return as-is (with length limit)
            Ok(if text.len() > 50000 {
                format!("{}... [Content truncated - {} total characters]", &text[..50000], text.len())
            } else {
                text
            })
        }
    }

    fn extract_main_content(&self, document: &Html) -> Result<String, ToolError> {
        let mut content_parts = Vec::new();

        // Try to extract title
        if let Ok(title_selector) = Selector::parse("title") {
            if let Some(title) = document.select(&title_selector).next() {
                let title_text = title.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !title_text.is_empty() {
                    content_parts.push(format!("Title: {}", title_text));
                }
            }
        }

        // Try to extract main content areas
        let content_selectors = [
            "main",
            "article",
            "[role=\"main\"]",
            ".main-content",
            ".content",
            "#content",
            ".post-content",
            ".entry-content",
        ];

        let mut found_main_content = false;
        for sel_str in &content_selectors {
            if let Ok(selector) = Selector::parse(sel_str) {
                for element in document.select(&selector) {
                    let text = element.text().collect::<Vec<_>>().join(" ");
                    let cleaned = text.trim();
                    if cleaned.len() > 100 { // Only include substantial content
                        content_parts.push(cleaned.to_string());
                        found_main_content = true;
                        break;
                    }
                }
            }
            if found_main_content { break; }
        }

        // Fallback: extract from body
        if !found_main_content {
            if let Ok(body_selector) = Selector::parse("body") {
                if let Some(body) = document.select(&body_selector).next() {
                    let text = body.text().collect::<Vec<_>>().join(" ");
                    let cleaned = text.trim();
                    if cleaned.len() > 50 {
                        content_parts.push(cleaned.to_string());
                    }
                }
            }
        }

        // Combine and limit length
        let content = content_parts.join("\n\n");
        Ok(if content.len() > 10000 {
            format!("{}... [Content truncated - {} total characters]", &content[..10000], content.len())
        } else {
            content
        })
    }
}

#[async_trait]
impl AgentTool for WebBrowsingTool {
    fn name(&self) -> &str {
        "web_browse"
    }

    fn description(&self) -> &str {
        "Browse web pages and extract content. Can fetch any URL and optionally use CSS selectors to extract specific content."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to browse",
                    "format": "uri"
                },
                "selector": {
                    "type": "string",
                    "description": "Optional CSS selector to extract specific content (e.g., 'h1', '.content', '#main')"
                },
                "timeout_seconds": {
                    "type": "integer",
                    "description": "Request timeout in seconds",
                    "default": 30,
                    "minimum": 1,
                    "maximum": 120
                },
                "follow_redirects": {
                    "type": "boolean",
                    "description": "Whether to follow HTTP redirects",
                    "default": true
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: WebBrowseParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        // Validate URL
        if params.url.is_empty() {
            return Err(ToolError::InvalidParameters("URL cannot be empty".to_string()));
        }

        // Basic URL validation
        if !params.url.starts_with("http://") && !params.url.starts_with("https://") {
            return Err(ToolError::InvalidParameters("URL must start with http:// or https://".to_string()));
        }

        let content = self.fetch_and_parse(
            &params.url,
            params.selector.as_deref(),
            params.timeout_seconds
        ).await?;

        Ok(ToolOutput {
            success: true,
            result: json!({
                "url": params.url,
                "content": content,
                "selector_used": params.selector,
                "content_length": content.len()
            }),
            error: None,
            usage: None,
        })
    }
}

impl WebSearchTool {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Aircher AI Agent/1.0 (Web Search)")
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client }
    }

    async fn search_duckduckgo(&self, query: &str, max_results: usize) -> Result<Vec<serde_json::Value>, ToolError> {
        // DuckDuckGo Instant Answer API
        let url = format!("https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
                         urlencoding::encode(query));

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Search request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ToolError::ExecutionFailed(format!("Search API returned {}", response.status())));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse search response: {}", e)))?;

        let mut results = Vec::new();

        // Extract instant answer if available
        if let Some(answer) = json.get("Answer").and_then(|a| a.as_str()) {
            if !answer.is_empty() {
                results.push(json!({
                    "title": "Instant Answer",
                    "url": json.get("AbstractURL").unwrap_or(&json!("")),
                    "content": answer,
                    "type": "instant_answer"
                }));
            }
        }

        // Extract abstract if available
        if let Some(abstract_text) = json.get("Abstract").and_then(|a| a.as_str()) {
            if !abstract_text.is_empty() {
                results.push(json!({
                    "title": json.get("Heading").unwrap_or(&json!("Summary")),
                    "url": json.get("AbstractURL").unwrap_or(&json!("")),
                    "content": abstract_text,
                    "type": "abstract"
                }));
            }
        }

        // Extract related topics
        if let Some(topics) = json.get("RelatedTopics").and_then(|t| t.as_array()) {
            for topic in topics.iter().take(max_results.saturating_sub(results.len())) {
                if let (Some(text), Some(url)) = (
                    topic.get("Text").and_then(|t| t.as_str()),
                    topic.get("FirstURL").and_then(|u| u.as_str())
                ) {
                    if !text.is_empty() {
                        results.push(json!({
                            "title": text.split(" - ").next().unwrap_or(text),
                            "url": url,
                            "content": text,
                            "type": "related_topic"
                        }));
                    }
                }
            }
        }

        // If no results, provide a fallback message
        if results.is_empty() {
            results.push(json!({
                "title": "No direct results",
                "url": format!("https://duckduckgo.com/?q={}", urlencoding::encode(query)),
                "content": format!("No instant answers found for '{}'. You may want to browse the search page directly.", query),
                "type": "fallback"
            }));
        }

        Ok(results)
    }
}

#[async_trait]
impl AgentTool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for information. Returns summaries and links for the query."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results to return",
                    "default": 10,
                    "minimum": 1,
                    "maximum": 20
                },
                "search_engine": {
                    "type": "string",
                    "description": "Search engine to use",
                    "enum": ["duckduckgo"],
                    "default": "duckduckgo"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let params: WebSearchParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

        if params.query.trim().is_empty() {
            return Err(ToolError::InvalidParameters("Search query cannot be empty".to_string()));
        }

        let results = match params.search_engine.as_str() {
            "duckduckgo" | "" => self.search_duckduckgo(&params.query, params.max_results).await?,
            engine => return Err(ToolError::InvalidParameters(format!("Unsupported search engine: {}", engine))),
        };

        Ok(ToolOutput {
            success: true,
            result: json!({
                "query": params.query,
                "search_engine": "duckduckgo",
                "results": results,
                "result_count": results.len()
            }),
            error: None,
            usage: None,
        })
    }
}
