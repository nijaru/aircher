use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tracing::{debug, info};

use crate::ui::{Message, MessageRole};

/// Context information extracted from conversation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionContext {
    pub current_task: String,
    pub recent_files: Vec<String>,
    pub active_tools: Vec<String>,
    pub key_decisions: Vec<String>,
    pub unresolved_issues: Vec<String>,
    pub project_type: Option<String>,
    pub programming_languages: HashSet<String>,
    pub frameworks: Vec<String>,
    pub error_patterns: Vec<String>,
    pub success_patterns: Vec<String>,
}

impl Default for CompactionContext {
    fn default() -> Self {
        Self {
            current_task: String::new(),
            recent_files: Vec::new(),
            active_tools: Vec::new(),
            key_decisions: Vec::new(),
            unresolved_issues: Vec::new(),
            project_type: None,
            programming_languages: HashSet::new(),
            frameworks: Vec::new(),
            error_patterns: Vec::new(),
            success_patterns: Vec::new(),
        }
    }
}

/// Analyzer for extracting meaningful context from conversation history
pub struct ConversationAnalyzer {
    // Regex patterns for detecting various context elements
    file_pattern: Regex,
    error_pattern: Regex,
    success_pattern: Regex,
    command_pattern: Regex,
    tool_pattern: Regex,
    decision_pattern: Regex,
}

impl ConversationAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            file_pattern: Regex::new(r"(?i)(?:file|path|src/|\.rs|\.js|\.py|\.go|\.java|\.cpp|\.h|\.tsx?|\.vue|\.svelte|Cargo\.toml|package\.json|go\.mod|requirements\.txt)")?,
            error_pattern: Regex::new(r"(?i)(?:error|failed|failure|exception|panic|crash|bug|issue|problem|broken)")?,
            success_pattern: Regex::new(r"(?i)(?:success|successful|fixed|resolved|solved|working|completed|done|finished)")?,
            command_pattern: Regex::new(r"(?i)(?:cargo|npm|yarn|go|python|pip|git|docker|kubectl|make)")?,
            tool_pattern: Regex::new(r"🔧|✓|⚠️|🎯|executing|executed|running|completed")?,
            decision_pattern: Regex::new(r"(?i)(?:decided|chosen|selected|will use|going with|opted for|conclusion)")?,
        })
    }
    
    /// Analyze conversation messages to extract context
    pub fn analyze_conversation(&self, messages: &[Message]) -> Result<CompactionContext> {
        let mut context = CompactionContext::default();
        
        // Focus on recent messages (last 10-15) for current context
        let recent_count = 15.min(messages.len());
        let recent_messages: Vec<_> = messages
            .iter()
            .rev()
            .take(recent_count)
            .collect();
        
        info!("Analyzing {} recent messages for compaction context", recent_messages.len());
        
        // Extract current task from most recent user messages
        context.current_task = self.extract_current_task(&recent_messages);
        
        // Analyze all messages for comprehensive context
        for message in messages {
            self.extract_file_mentions(message, &mut context.recent_files);
            self.extract_tools_and_commands(message, &mut context.active_tools);
            self.extract_decisions(message, &mut context.key_decisions);
            self.extract_issues(message, &mut context.unresolved_issues);
            self.extract_patterns(message, &mut context);
        }
        
        // Detect project type and languages
        context.project_type = self.detect_project_type(&context.recent_files);
        context.programming_languages = self.detect_languages(&context.recent_files);
        context.frameworks = self.detect_frameworks(&context.recent_files);
        
        // Deduplicate and prioritize
        self.prioritize_context(&mut context);
        
        debug!("Extracted context: current_task={}, files={}, tools={}, project_type={:?}", 
            context.current_task, context.recent_files.len(), context.active_tools.len(), context.project_type);
        
        Ok(context)
    }
    
    /// Extract the current task from recent user messages
    fn extract_current_task(&self, recent_messages: &[&Message]) -> String {
        let user_messages: Vec<_> = recent_messages
            .iter()
            .filter(|m| m.role == MessageRole::User)
            .take(3) // Last 3 user messages
            .collect();
        
        if user_messages.is_empty() {
            return "General development work".to_string();
        }
        
        // Look for imperative statements, questions, or specific requests
        for message in &user_messages {
            let content = message.content.to_lowercase();
            
            // Look for specific task indicators
            if content.contains("implement") || content.contains("create") || content.contains("build") {
                if let Some(task) = self.extract_task_from_content(&message.content) {
                    return task;
                }
            }
            
            if content.contains("fix") || content.contains("debug") || content.contains("error") {
                return format!("Debugging and fixing issues in {}", 
                    self.extract_subject_from_content(&message.content));
            }
            
            if content.contains("refactor") || content.contains("improve") || content.contains("optimize") {
                return format!("Refactoring and improving {}", 
                    self.extract_subject_from_content(&message.content));
            }
            
            if content.contains("test") {
                return format!("Testing {}", 
                    self.extract_subject_from_content(&message.content));
            }
        }
        
        // Fallback: use the first sentence of the most recent message
        if let Some(first_message) = user_messages.first() {
            let first_sentence = first_message.content
                .split('.')
                .next()
                .unwrap_or(&first_message.content)
                .trim();
            
            if !first_sentence.is_empty() && first_sentence.len() < 100 {
                return first_sentence.to_string();
            }
        }
        
        "Continuing development work".to_string()
    }
    
    /// Extract task description from content
    fn extract_task_from_content(&self, content: &str) -> Option<String> {
        // Look for patterns like "implement X", "create Y", "build Z"
        let task_patterns = [
            r"(?i)implement\s+([^.!?]+)",
            r"(?i)create\s+([^.!?]+)",
            r"(?i)build\s+([^.!?]+)",
            r"(?i)add\s+([^.!?]+)",
            r"(?i)develop\s+([^.!?]+)",
        ];
        
        for pattern in &task_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(content) {
                    if let Some(task) = captures.get(1) {
                        let task_desc = task.as_str().trim();
                        if task_desc.len() > 5 && task_desc.len() < 80 {
                            return Some(format!("Implementing {}", task_desc));
                        }
                    }
                }
            }
        }
        
        None
    }
    
    /// Extract subject/focus area from content
    fn extract_subject_from_content(&self, content: &str) -> String {
        // Look for file names, function names, or specific topics
        if let Some(captures) = self.file_pattern.captures(content) {
            return captures.get(0).unwrap().as_str().to_string();
        }
        
        // Fallback to first few words
        content
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(30)
            .collect()
    }
    
    /// Extract file paths and names mentioned in messages
    fn extract_file_mentions(&self, message: &Message, files: &mut Vec<String>) {
        let content = &message.content;
        
        // Common file patterns
        let file_patterns = [
            r"([a-zA-Z_][a-zA-Z0-9_]*/)*[a-zA-Z_][a-zA-Z0-9_]*\.[a-zA-Z0-9]+",
            r"src/[a-zA-Z_][a-zA-Z0-9_/]*\.[a-zA-Z0-9]+",
            r"Cargo\.toml|package\.json|go\.mod|requirements\.txt|pyproject\.toml",
            r"\./[a-zA-Z_][a-zA-Z0-9_/]*",
        ];
        
        for pattern in &file_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for cap in re.captures_iter(content) {
                    if let Some(file) = cap.get(0) {
                        let file_str = file.as_str().to_string();
                        if !files.contains(&file_str) && file_str.len() < 100 {
                            files.push(file_str);
                        }
                    }
                }
            }
        }
    }
    
    /// Extract tools and commands mentioned
    fn extract_tools_and_commands(&self, message: &Message, tools: &mut Vec<String>) {
        let content = &message.content;
        
        // Look for tool execution indicators
        if self.tool_pattern.is_match(content) {
            // Extract tool names from patterns like "🔧 Executing tool: read_file"
            if let Ok(re) = Regex::new(r"(?i)(?:executing|executed|running|using)\s+(?:tool:?\s*)?([a-zA-Z_][a-zA-Z0-9_]*)") {
                for cap in re.captures_iter(content) {
                    if let Some(tool) = cap.get(1) {
                        let tool_str = tool.as_str().to_string();
                        if !tools.contains(&tool_str) {
                            tools.push(tool_str);
                        }
                    }
                }
            }
        }
        
        // Look for command mentions
        if self.command_pattern.is_match(content) {
            let commands = ["cargo", "npm", "yarn", "go", "python", "pip", "git", "docker", "kubectl", "make"];
            for cmd in &commands {
                if content.to_lowercase().contains(cmd) && !tools.contains(&cmd.to_string()) {
                    tools.push(cmd.to_string());
                }
            }
        }
    }
    
    /// Extract key decisions made
    fn extract_decisions(&self, message: &Message, decisions: &mut Vec<String>) {
        if self.decision_pattern.is_match(&message.content) {
            // Extract sentences containing decision keywords
            for sentence in message.content.split('.') {
                let sentence = sentence.trim();
                if self.decision_pattern.is_match(sentence) && sentence.len() < 150 {
                    decisions.push(sentence.to_string());
                }
            }
        }
    }
    
    /// Extract unresolved issues
    fn extract_issues(&self, message: &Message, issues: &mut Vec<String>) {
        if self.error_pattern.is_match(&message.content) && !self.success_pattern.is_match(&message.content) {
            // Look for error descriptions
            for sentence in message.content.split('.') {
                let sentence = sentence.trim();
                if self.error_pattern.is_match(sentence) && sentence.len() > 10 && sentence.len() < 150 {
                    issues.push(sentence.to_string());
                }
            }
        }
    }
    
    /// Extract error and success patterns for context
    fn extract_patterns(&self, message: &Message, context: &mut CompactionContext) {
        if self.error_pattern.is_match(&message.content) {
            context.error_patterns.push(message.content.clone());
        }
        
        if self.success_pattern.is_match(&message.content) {
            context.success_patterns.push(message.content.clone());
        }
    }
    
    /// Detect project type from files
    fn detect_project_type(&self, files: &[String]) -> Option<String> {
        for file in files {
            if file.contains("Cargo.toml") {
                return Some("rust".to_string());
            }
            if file.contains("package.json") {
                return Some("node".to_string());
            }
            if file.contains("go.mod") {
                return Some("go".to_string());
            }
            if file.contains("requirements.txt") || file.contains("pyproject.toml") {
                return Some("python".to_string());
            }
            if file.contains("pom.xml") || file.contains("build.gradle") {
                return Some("java".to_string());
            }
        }
        None
    }
    
    /// Detect programming languages from file extensions
    fn detect_languages(&self, files: &[String]) -> HashSet<String> {
        let mut languages = HashSet::new();
        
        for file in files {
            if file.ends_with(".rs") { languages.insert("rust".to_string()); }
            if file.ends_with(".js") || file.ends_with(".jsx") { languages.insert("javascript".to_string()); }
            if file.ends_with(".ts") || file.ends_with(".tsx") { languages.insert("typescript".to_string()); }
            if file.ends_with(".py") { languages.insert("python".to_string()); }
            if file.ends_with(".go") { languages.insert("go".to_string()); }
            if file.ends_with(".java") { languages.insert("java".to_string()); }
            if file.ends_with(".cpp") || file.ends_with(".cc") || file.ends_with(".cxx") { languages.insert("cpp".to_string()); }
            if file.ends_with(".c") { languages.insert("c".to_string()); }
            if file.ends_with(".rb") { languages.insert("ruby".to_string()); }
            if file.ends_with(".php") { languages.insert("php".to_string()); }
        }
        
        languages
    }
    
    /// Detect frameworks from files and content
    fn detect_frameworks(&self, files: &[String]) -> Vec<String> {
        let mut frameworks = Vec::new();
        
        for file in files {
            if file.contains("package.json") {
                frameworks.extend(["react", "vue", "angular", "express", "nextjs"].iter().map(|s| s.to_string()));
            }
            if file.contains("Cargo.toml") {
                frameworks.extend(["tokio", "actix", "rocket", "warp"].iter().map(|s| s.to_string()));
            }
        }
        
        frameworks.truncate(3); // Limit to most relevant
        frameworks
    }
    
    /// Prioritize and limit context elements to most relevant
    fn prioritize_context(&self, context: &mut CompactionContext) {
        // Limit arrays to most recent/relevant items
        context.recent_files.truncate(8);
        context.active_tools.truncate(6);
        context.key_decisions.truncate(5);
        context.unresolved_issues.truncate(4);
        context.error_patterns.truncate(3);
        context.success_patterns.truncate(3);
        
        // Remove duplicates
        context.recent_files.dedup();
        context.active_tools.dedup();
        context.key_decisions.dedup();
        context.unresolved_issues.dedup();
    }
}

impl CompactionContext {
    /// Generate smart compaction prompt based on analyzed context
    pub fn generate_smart_prompt(&self, conversation: &str) -> String {
        let mut prompt = String::new();
        
        // Start with task-focused summary instruction
        if !self.current_task.is_empty() {
            prompt.push_str(&format!(
                "Create a targeted summary of this conversation optimized for continuing work on: {}\n\n",
                self.current_task
            ));
        } else {
            prompt.push_str("Create a comprehensive summary of this conversation preserving key development context:\n\n");
        }
        
        // Add focus areas based on context
        prompt.push_str("Focus especially on preserving:\n");
        
        if !self.recent_files.is_empty() {
            prompt.push_str(&format!(
                "- Work on files: {}\n",
                self.recent_files.join(", ")
            ));
        }
        
        if !self.active_tools.is_empty() {
            prompt.push_str(&format!(
                "- Tools and commands used: {}\n",
                self.active_tools.join(", ")
            ));
        }
        
        if !self.key_decisions.is_empty() {
            prompt.push_str("- Important decisions and conclusions reached\n");
        }
        
        if !self.unresolved_issues.is_empty() {
            prompt.push_str("- Current issues and debugging context\n");
        }
        
        // Add domain-specific priorities
        if let Some(project_type) = &self.project_type {
            prompt.push_str(&format!(
                "- {} project specifics: ",
                project_type
            ));
            
            match project_type.as_str() {
                "rust" => prompt.push_str("compilation errors, Cargo dependencies, test results, performance insights\n"),
                "node" => prompt.push_str("npm/yarn commands, package.json changes, test results, build issues\n"),
                "python" => prompt.push_str("import errors, virtual env setup, pip dependencies, test results\n"),
                "go" => prompt.push_str("go mod issues, build errors, test results, package structure\n"),
                "java" => prompt.push_str("classpath issues, build tool config, test results, dependencies\n"),
                _ => prompt.push_str("build system, dependencies, test results\n"),
            }
        }
        
        prompt.push_str("- Current project state and next steps\n");
        prompt.push_str("- Any unresolved issues or error states\n\n");
        
        prompt.push_str("Maintain enough technical detail for seamless continuation of the current work while compacting efficiently.\n\n");
        
        if !self.programming_languages.is_empty() {
            prompt.push_str(&format!(
                "[Languages in use: {}]\n\n",
                self.programming_languages.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
            ));
        }
        
        prompt.push_str("Conversation to summarize:\n\n");
        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_context_analysis() {
        let analyzer = ConversationAnalyzer::new().unwrap();
        let messages = vec![
            Message::user("I'm working on implementing authentication in src/auth.rs".to_string()),
            Message::assistant("I'll help you implement authentication. Let me read the current file.".to_string()),
            Message::user("The login function is failing with JWT parsing errors".to_string()),
        ];
        
        let context = analyzer.analyze_conversation(&messages).unwrap();
        
        assert!(!context.current_task.is_empty());
        assert!(context.recent_files.iter().any(|f| f.contains("auth.rs")));
        assert_eq!(context.project_type, Some("rust".to_string()));
        assert!(context.programming_languages.contains("rust"));
    }
    
    #[test]
    fn test_smart_prompt_generation() {
        let context = CompactionContext {
            current_task: "Implementing user authentication".to_string(),
            recent_files: vec!["src/auth.rs".to_string(), "Cargo.toml".to_string()],
            active_tools: vec!["read_file".to_string(), "edit_file".to_string()],
            project_type: Some("rust".to_string()),
            programming_languages: ["rust".to_string()].into_iter().collect(),
            ..Default::default()
        };
        
        let prompt = context.generate_smart_prompt("test conversation");
        
        assert!(prompt.contains("Implementing user authentication"));
        assert!(prompt.contains("src/auth.rs"));
        assert!(prompt.contains("rust project specifics"));
        assert!(prompt.contains("compilation errors"));
    }
}