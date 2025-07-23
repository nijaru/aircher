use anyhow::Result;
use serde_json::Value;
use tracing::{debug, info, warn};

use crate::intelligence::IntelligenceEngine;
use crate::providers::{LLMProvider, ChatRequest, Message, MessageRole};
use crate::agent::tools::{ToolRegistry, ToolCall};
use crate::agent::parser::{ToolCallParser, format_tool_results};
use crate::agent::conversation::{CodingConversation, Message as ConvMessage, MessageRole as ConvRole, ProjectContext};

pub struct AgentController {
    tools: ToolRegistry,
    intelligence: IntelligenceEngine,
    parser: ToolCallParser,
    conversation: CodingConversation,
    max_iterations: usize,
}

impl AgentController {
    pub fn new(
        intelligence: IntelligenceEngine,
        project_context: ProjectContext,
    ) -> Result<Self> {
        Ok(Self {
            tools: ToolRegistry::default(),
            intelligence,
            parser: ToolCallParser::new()?,
            conversation: CodingConversation {
                messages: Vec::new(),
                project_context,
                active_files: Vec::new(),
                task_list: Vec::new(),
            },
            max_iterations: 10, // Prevent infinite loops
        })
    }
    
    /// Process a user message and return the assistant's response
    pub async fn process_message(&mut self, user_message: &str, provider: &dyn LLMProvider) -> Result<String> {
        info!("Processing user message: {}", user_message);
        
        // Add user message to conversation
        self.conversation.messages.push(ConvMessage {
            role: ConvRole::User,
            content: user_message.to_string(),
            tool_calls: None,
            timestamp: chrono::Utc::now(),
        });
        
        let mut iterations = 0;
        let mut final_response = String::new();
        
        loop {
            iterations += 1;
            if iterations > self.max_iterations {
                warn!("Max iterations reached, stopping tool execution");
                break;
            }
            
            // Build chat request with system prompt
            let mut messages = vec![
                Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: MessageRole::System,
                    content: self.build_system_prompt(),
                    timestamp: chrono::Utc::now(),
                    tokens_used: None,
                    cost: None,
                },
            ];
            
            // Add conversation history
            for msg in &self.conversation.messages {
                messages.push(Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: match msg.role {
                        ConvRole::User => MessageRole::User,
                        ConvRole::Assistant => MessageRole::Assistant,
                        ConvRole::System => MessageRole::System,
                        ConvRole::Tool => MessageRole::User, // Tools results go as user messages
                    },
                    content: msg.content.clone(),
                    timestamp: msg.timestamp,
                    tokens_used: None,
                    cost: None,
                });
            }
            
            // Get response from LLM
            let request = ChatRequest {
                messages,
                model: "llama3.3".to_string(), // Default model, will be overridden by provider
                temperature: Some(0.7),
                max_tokens: Some(2000),
                stream: false,
                tools: None,
            };
            
            let response = provider.chat(&request).await?;
            let assistant_message = response.content;
            
            // Parse tool calls
            let (clean_text, tool_calls) = self.parser.parse_structured(&assistant_message)?;
            
            if tool_calls.is_empty() {
                // No tool calls, this is the final response
                final_response = clean_text.clone();
                
                // Add assistant message to conversation
                self.conversation.messages.push(ConvMessage {
                    role: ConvRole::Assistant,
                    content: clean_text,
                    tool_calls: None,
                    timestamp: chrono::Utc::now(),
                });
                
                break;
            } else {
                // Execute tool calls
                info!("Executing {} tool calls", tool_calls.len());
                let tool_results = self.execute_tools(&tool_calls).await;
                
                // Add assistant message with tool calls
                self.conversation.messages.push(ConvMessage {
                    role: ConvRole::Assistant,
                    content: clean_text,
                    tool_calls: Some(tool_calls.iter().map(|tc| crate::agent::conversation::ToolCall {
                        tool_name: tc.name.clone(),
                        parameters: tc.parameters.clone(),
                        result: None,
                    }).collect()),
                    timestamp: chrono::Utc::now(),
                });
                
                // Format and add tool results
                let formatted_results = format_tool_results(&tool_results);
                self.conversation.messages.push(ConvMessage {
                    role: ConvRole::Tool,
                    content: formatted_results,
                    tool_calls: None,
                    timestamp: chrono::Utc::now(),
                });
            }
        }
        
        Ok(final_response)
    }
    
    /// Execute a list of tool calls
    async fn execute_tools(&self, tool_calls: &[ToolCall]) -> Vec<(String, Result<Value, String>)> {
        let mut results = Vec::new();
        
        for call in tool_calls {
            debug!("Executing tool: {} with params: {}", call.name, call.parameters);
            
            if let Some(tool) = self.tools.get(&call.name) {
                match tool.execute(call.parameters.clone()).await {
                    Ok(output) => {
                        if output.success {
                            results.push((call.name.clone(), Ok(output.result)));
                        } else {
                            let error = output.error.unwrap_or_else(|| "Unknown error".to_string());
                            results.push((call.name.clone(), Err(error)));
                        }
                    }
                    Err(e) => {
                        results.push((call.name.clone(), Err(e.to_string())));
                    }
                }
            } else {
                results.push((call.name.clone(), Err(format!("Unknown tool: {}", call.name))));
            }
        }
        
        results
    }
    
    /// Build the system prompt for the coding assistant
    fn build_system_prompt(&self) -> String {
        format!(r#"You are an AI coding assistant helping with a {} project at {}.

You have access to the following tools:
{}

To use a tool, wrap your tool calls in <tool_use> blocks:
<tool_use>
<tool>tool_name</tool><params>{{"param": "value"}}</params>
</tool_use>

Guidelines:
1. Always read files before editing them
2. Use search_code to find relevant code across the project
3. Make focused, specific edits rather than rewriting entire files
4. Verify your changes by reading the file again after editing
5. Run tests or build commands to ensure changes work
6. Explain your reasoning and the changes you're making

Current context:
- Project root: {}
- Active files: {:?}
- Recent tasks: {} pending, {} completed

Be concise but thorough. Focus on solving the user's problem effectively."#,
            self.conversation.project_context.language.to_string(),
            self.conversation.project_context.root_path.display(),
            self.format_tool_list(),
            self.conversation.project_context.root_path.display(),
            self.conversation.active_files,
            self.conversation.task_list.iter().filter(|t| matches!(t.status, crate::agent::conversation::TaskStatus::Pending)).count(),
            self.conversation.task_list.iter().filter(|t| matches!(t.status, crate::agent::conversation::TaskStatus::Completed)).count(),
        )
    }
    
    /// Format the list of available tools
    fn format_tool_list(&self) -> String {
        let tools = self.tools.list_tools();
        tools.iter()
            .map(|info| format!("- {}: {}", info.name, info.description))
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Add a tool to the registry
    pub fn register_tool(&mut self, tool: Box<dyn crate::agent::tools::AgentTool>) {
        self.tools.register(tool);
    }
    
    /// Get the current conversation
    pub fn conversation(&self) -> &CodingConversation {
        &self.conversation
    }
    
    /// Clear the conversation history
    pub fn clear_conversation(&mut self) {
        self.conversation.messages.clear();
    }
}

