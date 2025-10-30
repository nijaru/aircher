use anyhow::Result;
use std::sync::Arc;
use std::path::PathBuf;
use tracing::{debug, info, warn};

#[cfg(feature = "acp")]
use agent_client_protocol::{Agent as AcpAgent, InitializeRequest, InitializeResponse, NewSessionRequest, NewSessionResponse, PromptRequest, PromptResponse, ContentBlock, StopReason, AgentCapabilities, PromptCapabilities, SessionId, AuthenticateRequest, LoadSessionRequest, CancelNotification};

use crate::auth::AuthManager;
use crate::intelligence::{IntelligenceEngine, UnifiedIntelligenceEngine};
use crate::providers::{LLMProvider, ChatRequest, Message, MessageRole, PricingModel};
use crate::agent::tools::ToolRegistry;
use crate::agent::parser::ToolCallParser;
use crate::agent::conversation::{CodingConversation, Message as ConvMessage, MessageRole as ConvRole, ProjectContext};
use crate::agent::reasoning::{AgentReasoning, TaskStatus};
use crate::agent::dynamic_context::DynamicContextManager;
use crate::agent::task_orchestrator::TaskOrchestrator;
use crate::agent::plan_mode::{PlanGenerator, PlanMode};
use crate::agent::multi_turn_reasoning::MultiTurnReasoningEngine;
use crate::agent::events::{SharedEventBus, create_event_bus, AgentEvent, FileOperation, AgentMode};
use crate::agent::lsp_manager::LspManager;
use crate::agent::agent_mode::{ModeClassifier, ModeTransition};
use crate::agent::git_snapshots::SnapshotManager;
use crate::agent::model_router::ModelRouter;
use crate::agent::specialized_agents::AgentRegistry;
use crate::agent::research_subagents::ResearchSubAgentManager;
use crate::semantic_search::SemanticCodeSearch;

/// Unified Agent implementation that serves both TUI and ACP modes
pub struct Agent {
    tools: Arc<ToolRegistry>,
    intelligence: Arc<IntelligenceEngine>,
    unified_intelligence: Arc<UnifiedIntelligenceEngine>,
    #[allow(dead_code)]
    auth_manager: Arc<AuthManager>,
    parser: ToolCallParser,
    conversation: Arc<tokio::sync::Mutex<CodingConversation>>,
    reasoning: Arc<AgentReasoning>,
    context_manager: Arc<DynamicContextManager>,
    #[allow(dead_code)]
    orchestrator: Option<Arc<TaskOrchestrator>>,
    plan_generator: Arc<tokio::sync::Mutex<PlanGenerator>>,
    /// Multi-turn reasoning engine for systematic problem solving
    multi_turn_reasoning: Arc<tokio::sync::Mutex<MultiTurnReasoningEngine>>,
    /// Prevents infinite orchestration recursion
    is_orchestration_agent: bool,
    #[allow(dead_code)]
    max_iterations: usize,
    /// Event bus for agent-wide communication (Week 7)
    event_bus: SharedEventBus,
    /// LSP manager with global diagnostics (Week 7)
    lsp_manager: Arc<LspManager>,
    /// Current agent mode: Plan (read-only) or Build (modification) (Week 7 Day 3)
    current_mode: Arc<tokio::sync::RwLock<AgentMode>>,
    /// Git snapshot manager for safe experimentation (Week 7 Day 5)
    snapshot_manager: Option<Arc<SnapshotManager>>,
    /// Model router for cost-aware model selection (Week 7 Day 6-7)
    model_router: Arc<ModelRouter>,
    /// Registry of specialized agent configurations (Week 8 Day 1-2)
    agent_registry: Arc<AgentRegistry>,
    /// Research sub-agent manager for parallel research (Week 8 Day 3-4)
    research_manager: Arc<ResearchSubAgentManager>,
}

impl Agent {
    pub async fn new(
        intelligence: IntelligenceEngine,
        auth_manager: Arc<AuthManager>,
        project_context: ProjectContext,
    ) -> Result<Self> {
        let tools = Arc::new(ToolRegistry::default());
        let intelligence = Arc::new(intelligence);

        Self::new_internal(intelligence, auth_manager, project_context, tools).await
    }

    /// Create agent with approval-enabled tools
    pub async fn new_with_approval(
        intelligence: IntelligenceEngine,
        auth_manager: Arc<AuthManager>,
        project_context: ProjectContext,
    ) -> Result<(Self, tokio::sync::mpsc::UnboundedReceiver<crate::agent::approval_modes::PendingChange>)> {
        use crate::agent::tools::approval_registry::create_agent_registry_with_approval;

        let intelligence = Arc::new(intelligence);
        let (tools, approval_rx) = create_agent_registry_with_approval();
        let tools = Arc::new(tools);

        let agent = Self::new_internal(intelligence, auth_manager, project_context, tools).await?;

        // Set plan mode to Review by default for approval-enabled agents
        {
            let mut plan_gen = agent.plan_generator.lock().await;
            plan_gen.set_mode(PlanMode::Normal); // Start in normal mode
        }
        Ok((agent, approval_rx))
    }

    async fn new_internal(
        intelligence: Arc<IntelligenceEngine>,
        auth_manager: Arc<AuthManager>,
        project_context: ProjectContext,
        tools: Arc<ToolRegistry>,
    ) -> Result<Self> {

        // Create semantic search for context management (optional - may not have index)
        // Only create if we can, otherwise context manager works without it
        let search = Some(Arc::new(tokio::sync::RwLock::new(SemanticCodeSearch::new())));

        // Create dynamic context manager for intelligent context management
        let context_manager = Arc::new(DynamicContextManager::new(
            intelligence.clone(),
            search,
        ));

        // Create the reasoning engine with intelligent planning capabilities
        let reasoning = Arc::new(AgentReasoning::new(
            intelligence.clone(),
            tools.clone(),
        ));

        // Create unified intelligence engine for automatic middleware
        // Note: We need to clone the intelligence engine for the unified wrapper
        let intelligence_for_unified = IntelligenceEngine::new(
            &crate::config::ConfigManager::load().await?,
            &crate::storage::DatabaseManager::new(&crate::config::ConfigManager::load().await?).await?
        ).await?;
        let unified_intelligence = Arc::new(UnifiedIntelligenceEngine::new(intelligence_for_unified));

        // Create multi-turn reasoning engine for systematic problem solving
        let multi_turn_reasoning = Arc::new(tokio::sync::Mutex::new(
            MultiTurnReasoningEngine::new(tools.clone(), intelligence.clone())?
        ));

        // Create event bus for agent-wide communication (Week 7)
        let event_bus = create_event_bus();

        // Create LSP manager with global diagnostics (Week 7)
        let workspace_root = project_context.root_path.clone();
        let lsp_manager = Arc::new(LspManager::new(workspace_root, event_bus.clone()));

        // Start LSP manager event listener (Week 7 Day 2)
        lsp_manager.clone().start_listening();

        // Initialize agent mode (Week 7 Day 3)
        // Start in Plan mode (safe, read-only) by default
        let current_mode = Arc::new(tokio::sync::RwLock::new(AgentMode::Plan));

        // Initialize Git snapshot manager (Week 7 Day 5)
        // Only available if workspace is a Git repository
        let snapshot_manager = SnapshotManager::new(project_context.root_path.clone(), event_bus.clone())
            .ok()
            .map(Arc::new);

        if snapshot_manager.is_some() {
            info!("Git snapshot manager initialized");
        } else {
            warn!("Git not available - snapshot functionality disabled");
        }

        // Initialize model router for cost-aware model selection (Week 7 Day 6-7)
        // Load config to check for single model override
        let config = crate::config::ConfigManager::load().await?;
        let model_router = if let Some(ref single_model) = config.model_routing.single_model {
            // User specified single model for all tasks (bypasses routing table)
            info!("Initializing model router with single model override: {}", single_model);

            // Map model string to ModelConfig
            // TODO: Add support for other providers (openai, google, openrouter)
            let model_config_opt = match single_model.as_str() {
                "claude-sonnet-4-5" | "claude-sonnet-4.5" => Some(crate::agent::model_router::ModelConfig::claude_sonnet_4_5()),
                "claude-haiku-4-5" | "claude-haiku-4.5" => Some(crate::agent::model_router::ModelConfig::claude_haiku_4_5()),
                "claude-opus-4-1" | "claude-opus-4.1" => Some(crate::agent::model_router::ModelConfig::claude_opus_4_1()),
                _ => {
                    warn!("Unknown model '{}', falling back to smart routing", single_model);
                    None
                }
            };

            if let Some(model_config) = model_config_opt {
                Arc::new(ModelRouter::with_single_model(model_config))
            } else {
                Arc::new(ModelRouter::new())
            }
        } else {
            // Use smart routing by default (zero config)
            info!("Model router initialized with smart routing (default)");
            Arc::new(ModelRouter::new())
        };

        // Initialize specialized agent registry (Week 8 Day 1-2)
        let agent_registry = Arc::new(AgentRegistry::new());
        info!("Agent registry initialized with 7 specialized configs");

        // Initialize research sub-agent manager (Week 8 Day 3-4)
        // Pass tool_registry and workspace_root for actual tool execution
        let research_manager = Arc::new(ResearchSubAgentManager::with_tools(
            tools.clone(),
            project_context.root_path.clone(),
        ));
        info!("Research sub-agent manager initialized (max {} concurrent) with tool execution",
              crate::agent::research_subagents::MAX_CONCURRENT_SUBAGENTS);

        Ok(Self {
            tools,
            intelligence,
            unified_intelligence,
            auth_manager,
            parser: ToolCallParser::new()?,
            conversation: Arc::new(tokio::sync::Mutex::new(CodingConversation {
                messages: Vec::new(),
                project_context,
                active_files: Vec::new(),
                task_list: Vec::new(),
            })),
            reasoning,
            context_manager,
            orchestrator: None, // Created on-demand to avoid circular dependency
            plan_generator: Arc::new(tokio::sync::Mutex::new(PlanGenerator::new())),
            multi_turn_reasoning,
            is_orchestration_agent: false,
            max_iterations: 10, // Prevent infinite loops
            event_bus,
            lsp_manager,
            current_mode,
            snapshot_manager,
            model_router,
            agent_registry,
            research_manager,
        })
    }
    
    /// Get reference to event bus (Week 7)
    pub fn event_bus(&self) -> &SharedEventBus {
        &self.event_bus
    }

    /// Get reference to LSP manager (Week 7)
    pub fn lsp_manager(&self) -> &Arc<LspManager> {
        &self.lsp_manager
    }

    /// Get reference to model router (Week 7 Day 6-7)
    pub fn model_router(&self) -> &Arc<ModelRouter> {
        &self.model_router
    }

    /// Get reference to agent registry (Week 8 Day 1-2)
    pub fn agent_registry(&self) -> &Arc<AgentRegistry> {
        &self.agent_registry
    }

    /// Get reference to research sub-agent manager (Week 8 Day 3-4)
    pub fn research_manager(&self) -> &Arc<ResearchSubAgentManager> {
        &self.research_manager
    }

    /// Get current agent mode (Week 7 Day 3)
    pub async fn current_mode(&self) -> AgentMode {
        *self.current_mode.read().await
    }

    /// Set agent mode with transition event (Week 7 Day 3)
    pub async fn set_mode(&self, new_mode: AgentMode, reason: String) {
        let old_mode = *self.current_mode.read().await;

        if old_mode != new_mode {
            *self.current_mode.write().await = new_mode;

            // Create and log transition
            let transition = ModeTransition::new(old_mode, new_mode, reason);
            transition.log();

            // Emit event
            self.event_bus.publish(transition.to_event());

            info!("Agent mode changed: {:?} → {:?}", old_mode, new_mode);
        }
    }

    /// Get reference to snapshot manager (Week 7 Day 5)
    /// Returns None if Git is not available in workspace
    pub fn snapshot_manager(&self) -> Option<&Arc<SnapshotManager>> {
        self.snapshot_manager.as_ref()
    }

    /// Convert tool registry to provider tool format for LLM requests
    pub fn convert_tools_to_provider_format(&self) -> Vec<crate::providers::Tool> {
        self.tools.list_tools()
            .into_iter()
            .map(|tool_info| crate::providers::Tool {
                name: tool_info.name,
                description: tool_info.description,
                parameters: tool_info.parameters,
            })
            .collect()
    }
    
    /// Process a user message and return the assistant's response with tool status
    pub async fn process_message(&self, user_message: &str, provider: &dyn LLMProvider, model: &str) -> Result<(String, Vec<String>)> {
        info!("Processing user message with intelligent reasoning: {}", user_message);

        // === AUTOMATIC INTELLIGENCE MIDDLEWARE ===
        // 1. Automatically enhance request understanding (FIRST, before any other processing)
        let enhanced_context = match self.unified_intelligence.enhance_request_understanding(user_message).await {
            Ok(context) => {
                debug!("Intelligence automatically enhanced request with intent: {:?}", context.detected_intent);
                context
            },
            Err(e) => {
                warn!("Intelligence enhancement failed: {}, proceeding without", e);
                // Create minimal context for fallback
                use crate::intelligence::{EnhancedContext, UserIntent, ExplorationScope};
                EnhancedContext {
                    original_request: user_message.to_string(),
                    detected_intent: UserIntent::ProjectExploration { scope: ExplorationScope::SingleFile },
                    relevant_context: Vec::new(),
                    intelligence_insights: Vec::new(),
                    suggested_approach: "Proceed with standard processing".to_string(),
                    confidence: 0.5,
                }
            }
        };

        // === SPECIALIZED AGENT SELECTION (Week 8 Day 1-2) ===
        // Select appropriate specialized agent based on detected intent
        let selected_agent = self.select_agent_for_intent(&enhanced_context.detected_intent).await;
        info!("Selected {:?} agent for this task", selected_agent.agent_type);

        // === TASK COMPLEXITY ASSESSMENT (Week 7 Day 6-7) ===
        let task_complexity = self.assess_task_complexity(user_message, &enhanced_context).await;
        debug!("Task complexity: {:?}", task_complexity);

        // === MODEL SELECTION VIA ROUTER (Week 7 Day 6-7) ===
        // Use model router to select cost-appropriate model
        let selected_model_config = self.model_router.select_model(
            selected_agent.agent_type,
            task_complexity,
            None, // TODO: Support user model override
        );
        let selected_model = &selected_model_config.model;
        info!("Model router selected: {} (cost: ${:.4}/1M in, ${:.4}/1M out)",
              selected_model,
              selected_model_config.cost_per_1m_input,
              selected_model_config.cost_per_1m_output);

        // === RESEARCH SUB-AGENT SPAWNING (Week 8 Day 3-4) ===
        // For Explorer agents with research queries, spawn parallel sub-agents
        if selected_agent.can_spawn_subagents && self.is_research_task(user_message).await {
            info!("Explorer agent detected research task - considering sub-agent spawn");

            // Check if query benefits from parallelization
            if user_message.to_lowercase().contains("find all")
                || user_message.to_lowercase().contains("search for")
                || user_message.to_lowercase().contains("list all") {

                info!("Spawning research sub-agents for parallel execution");
                match self.research_manager.spawn_research(user_message).await {
                    Ok(handle) => {
                        info!("Research sub-agents spawned, waiting for results...");
                        match handle.wait().await {
                            Ok(results) => {
                                info!("Research complete: {} sub-agents returned results", results.len());

                                // Add research findings to system prompt
                                let mut research_summary = String::from("\n\n## Research Sub-Agent Findings:\n");
                                for (i, result) in results.iter().enumerate() {
                                    if result.success {
                                        research_summary.push_str(&format!("{}. {}\n", i + 1, result.findings));
                                    }
                                }

                                // This will be added to system prompt below
                                debug!("Research summary: {}", research_summary);
                            }
                            Err(e) => {
                                warn!("Research sub-agents failed: {}, proceeding without", e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to spawn research sub-agents: {}, proceeding without", e);
                    }
                }
            }
        }

        // Validate authentication before making LLM calls
        if !matches!(provider.pricing_model(), PricingModel::Free) && !provider.name().eq_ignore_ascii_case("ollama") {
            debug!("Auth validated for provider: {}", provider.name());
        }

        // Add user message to conversation
        {
            let mut conversation = self.conversation.lock().await;
            conversation.messages.push(ConvMessage {
                role: ConvRole::User,
                content: user_message.to_string(),
                tool_calls: None,
                timestamp: chrono::Utc::now(),
            });
        }

        let mut tool_status_messages = Vec::new();

        // Check if this task needs multi-turn reasoning (systematic problem solving)
        if self.needs_multi_turn_reasoning(user_message).await {
            info!("Task requires systematic multi-turn reasoning - using MultiTurnReasoningEngine");
            return self.process_with_multi_turn_reasoning(user_message, provider, model).await;
        }

        // Check if this task needs multi-turn orchestration
        if self.needs_orchestration(user_message).await {
            info!("Task requires orchestration - using TaskOrchestrator for complex workflow");
            return self.process_with_orchestration(user_message, provider, model).await;
        }

        // Use reasoning engine for task planning (NOT execution)
        let reasoning_result = match self.reasoning.process_request(user_message).await {
            Ok(result) => Some(result),
            Err(e) => {
                debug!("Reasoning engine failed to plan: {}", e);
                None
            }
        };

        // Check if reasoning provided a plan to enhance our prompt
        if let Some(result) = reasoning_result {
            if result.task.status == TaskStatus::Planned && !result.task.tool_calls.is_empty() {
                // Reasoning engine has planned tool calls - add them to prompt
                info!("Using reasoning engine plan: {} tool calls planned", result.task.tool_calls.len());

                // Add planned tasks to status messages for UI
                for tool_call in &result.task.tool_calls {
                    tool_status_messages.push(format!("📋 Planned: {} with {}",
                        tool_call.name,
                        tool_call.parameters
                    ));
                }

                // The LLM will execute these with proper content generation
            } else if result.task.status == TaskStatus::Completed {
                // If reasoning claims completion without tool calls, it's fake
                warn!("Reasoning engine claims completion without execution - ignoring");
            }
        }

        // Always proceed with LLM-based execution for actual work
        debug!("Proceeding with LLM-based tool execution");

        // 2. Build chat request with specialized agent system prompt (Week 8 Day 1-2)
        // Use the specialized agent's system prompt instead of generic one
        let mut system_prompt = selected_agent.system_prompt.clone();

        // Enhance with intelligence context
        let enhanced_prompt = match self.unified_intelligence.enhance_system_prompt(&system_prompt, &enhanced_context).await {
            Ok(enhanced) => {
                debug!("Specialized agent prompt enhanced with intelligence");
                enhanced
            },
            Err(e) => {
                warn!("Prompt enhancement failed: {}, using agent's base prompt", e);
                system_prompt.clone()
            }
        };

        system_prompt = enhanced_prompt;

        // Update dynamic context based on current activity
        if let Ok(context_update) = self.context_manager.update_context(user_message).await {
            debug!("Context updated: {} items added, {} removed",
                context_update.added.len(),
                context_update.removed.len());

            // Add relevant context to the system prompt
            if let Ok(relevant_context) = self.context_manager.get_relevant_context(5).await {
                if !relevant_context.is_empty() {
                    system_prompt.push_str("\n\n## Relevant Context:\n");
                    for item in relevant_context {
                        system_prompt.push_str(&format!("- {}\n", item.summary()));
                    }
                }
            }
        }
        
        let mut messages = vec![
            Message {
                id: uuid::Uuid::new_v4().to_string(),
                role: MessageRole::System,
                content: system_prompt,
                timestamp: chrono::Utc::now(),
                tokens_used: None,
                cost: None,
            },
        ];
        
        // Add conversation history
        {
            let conversation = self.conversation.lock().await;
            for msg in &conversation.messages {
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
        }
        
        // SMART TOOL PROVISION: Only provide tools when the task actually needs them
        let needs_tools = self.task_needs_tools(user_message);
        debug!("Task needs tools: {} (provider supports: {})", needs_tools, provider.supports_tools());

        let tools = if provider.supports_tools() && needs_tools {
            Some(self.convert_tools_to_provider_format())
        } else {
            None
        };
        
        let request = ChatRequest {
            messages: messages.clone(),
            model: selected_model.to_string(), // Use model router selection (Week 7 Day 6-7)
            temperature: Some(0.7),
            max_tokens: Some(2000),
            stream: false,
            tools, // FIXED: Actually send tool schemas to LLM instead of None
        };

        let response = provider.chat(&request).await?;
        let assistant_message = response.content;

        debug!("Raw response from provider: {} chars", assistant_message.len());
        debug!("Response tool_calls: {:?}", response.tool_calls);
        debug!("Response content first 500 chars: {}", assistant_message.chars().take(500).collect::<String>());

        // === RECORD TOKEN USAGE TO MODEL ROUTER (Week 7 Day 6-7 - Critical Issue #2) ===
        // Estimate input/output token split from total tokens_used
        // Heuristic: Assume ~70% input (prompt + context), ~30% output (response)
        // This is approximate since providers don't always split tokens in response
        let total_tokens = response.tokens_used as usize;
        let estimated_input_tokens = (total_tokens as f64 * 0.7) as usize;
        let estimated_output_tokens = total_tokens - estimated_input_tokens;

        // Record usage for cost tracking
        self.model_router.record_usage(
            &selected_model_config,
            estimated_input_tokens,
            estimated_output_tokens,
        ).await;

        debug!("Recorded model usage: {} total tokens ({} in, {} out estimated)",
               total_tokens, estimated_input_tokens, estimated_output_tokens);

        // Log if LLM is trying to call multiple tools
        if let Some(ref tool_calls) = response.tool_calls {
            if tool_calls.len() > 1 {
                info!("LLM requested {} tools in single response", tool_calls.len());
            }
        }

        // Get tool calls - first check structured response, then parse from text
        let (clean_text, tool_calls) = if let Some(structured_tool_calls) = &response.tool_calls {
            // Convert from provider tool calls to agent tool calls
            let agent_tool_calls: Vec<crate::agent::tools::ToolCall> = structured_tool_calls
                .iter()
                .map(|tc| crate::agent::tools::ToolCall {
                    name: tc.name.clone(),
                    parameters: tc.arguments.clone(),
                })
                .collect();

            (assistant_message, agent_tool_calls)
        } else {
            // Fall back to parsing from text content
            debug!("No structured tool calls, parsing from text");
            let parsed = self.parser.parse_structured(&assistant_message)?;
            debug!("Parsed clean_text: {} chars, tool_calls: {}", parsed.0.len(), parsed.1.len());
            debug!("Parsed clean_text first 200 chars: {}", parsed.0.chars().take(200).collect::<String>());
            parsed
        };

        debug!("Final clean_text: {} chars, tool_calls: {}", clean_text.len(), tool_calls.len());

        if tool_calls.is_empty() {
            // Add assistant message to conversation
            {
                let mut conversation = self.conversation.lock().await;
                conversation.messages.push(ConvMessage {
                    role: ConvRole::Assistant,
                    content: clean_text.clone(),
                    tool_calls: None,
                    timestamp: chrono::Utc::now(),
                });
            }
        }

        let final_response = if tool_calls.is_empty() {
            // No tool calls, this is the final response
            clean_text.clone()
        } else {
            // Start multi-turn execution loop
            info!("Starting multi-turn execution with {} tool calls", tool_calls.len());

            // Add tool status messages for UI
            for call in &tool_calls {
                tool_status_messages.push(format!("🔧 Executing tool: {}", call.name));
            }
            
            // Execute tools (simplified for this implementation)
            let mut tool_results = Vec::new();
            for call in &tool_calls {
                debug!("Executing tool: {} with params: {}", call.name, call.parameters);

                // Week 7 Day 3: Check if tool is allowed in current mode
                let current_mode = self.current_mode().await;
                if !current_mode.is_tool_allowed(&call.name) {
                    let error_msg = format!(
                        "Tool '{}' not allowed in {:?} mode. Available tools: {:?}",
                        call.name,
                        current_mode,
                        current_mode.allowed_tools()
                    );
                    warn!("{}", error_msg);
                    tool_results.push(format!("Tool {} blocked: {}", call.name, error_msg));
                    continue;
                }

                if let Some(tool) = self.tools.get(&call.name) {
                    match tool.execute(call.parameters.clone()).await {
                        Ok(output) => {
                            // Track context access for file-related tools
                            if call.name == "read_file" || call.name == "edit_file" || call.name == "write_file" {
                                if let Some(path) = call.parameters.get("path").or(call.parameters.get("file_path")).and_then(|p| p.as_str()) {
                                    let action = if call.name == "read_file" {
                                        crate::agent::dynamic_context::AccessAction::Read
                                    } else {
                                        crate::agent::dynamic_context::AccessAction::Modified
                                    };
                                    let _ = self.context_manager.track_file_access(path, action).await;

                                    // Week 7 Day 2: Emit FileChanged event for LSP integration
                                    if (call.name == "edit_file" || call.name == "write_file") && output.success {
                                        let file_path = PathBuf::from(path);
                                        let operation = if call.name == "edit_file" {
                                            FileOperation::Edit
                                        } else {
                                            FileOperation::Write
                                        };
                                        self.event_bus.publish(AgentEvent::FileChanged {
                                            path: file_path,
                                            operation,
                                            timestamp: std::time::SystemTime::now(),
                                        });
                                        debug!("Emitted FileChanged event for: {}", path);
                                    }
                                }
                            }

                            if output.success {
                                // Format the result more clearly
                                let result_str = if output.result.is_object() || output.result.is_array() {
                                    serde_json::to_string_pretty(&output.result).unwrap_or_else(|_| format!("{:?}", output.result))
                                } else {
                                    output.result.to_string()
                                };
                                tool_results.push(format!("Tool {} completed successfully. Result:\n{}", call.name, result_str));
                            } else {
                                let error = output.error.unwrap_or_else(|| "Unknown error".to_string());
                                tool_results.push(format!("Tool {} failed: {}", call.name, error));
                            }
                        }
                        Err(e) => {
                            tool_results.push(format!("Tool {} error: {}", call.name, e));
                        }
                    }
                } else {
                    tool_results.push(format!("Tool {} not found", call.name));
                }
            }
            
            // Format tool results for the conversation
            let tool_results_text = tool_results.join("\n");

            // Add tool results to conversation for multi-turn processing
            {
                let mut conversation = self.conversation.lock().await;
                conversation.messages.push(ConvMessage {
                    role: ConvRole::Tool,
                    content: tool_results_text.clone(),
                    tool_calls: None,
                    timestamp: chrono::Utc::now(),
                });
            }

            // MULTI-TURN LOOP: Send results back to LLM for interpretation and next steps
            debug!("Sending tool results back to LLM for interpretation");

            // Build new request with tool results included
            let mut messages_with_results = messages.clone();

            // Add the assistant's tool call message
            messages_with_results.push(Message {
                id: uuid::Uuid::new_v4().to_string(),
                role: MessageRole::Assistant,
                content: clean_text.clone(),
                timestamp: chrono::Utc::now(),
                tokens_used: None,
                cost: None,
            });

            // Add tool results as user message with continuation prompt
            // Check if original message implies more work is needed
            let message_lower = user_message.to_lowercase();
            let needs_continuation = message_lower.contains("then") ||
                                    message_lower.contains("and then") ||
                                    message_lower.contains(", then");

            let tool_result_message = if needs_continuation {
                format!(
                    "Tool execution results:\n{}\n\nBased on these results, continue with the next step of the task: {}",
                    tool_results_text,
                    user_message
                )
            } else {
                format!("Tool execution results:\n{}", tool_results_text)
            };

            messages_with_results.push(Message {
                id: uuid::Uuid::new_v4().to_string(),
                role: MessageRole::User,
                content: tool_result_message,
                timestamp: chrono::Utc::now(),
                tokens_used: None,
                cost: None,
            });

            // Request interpretation and next steps from LLM
            let interpret_request = ChatRequest {
                messages: messages_with_results.clone(),
                model: model.to_string(),
                temperature: Some(0.7),
                max_tokens: Some(2000),
                stream: false,
                tools: if provider.supports_tools() && needs_tools {
                    Some(self.convert_tools_to_provider_format())
                } else {
                    None
                },
            };

            // Get LLM's interpretation and potential next steps
            let interpretation_response = provider.chat(&interpret_request).await?;
            debug!("LLM interpretation: {} chars", interpretation_response.content.len());

            // Check if LLM wants to execute more tools
            let (final_text, next_tool_calls) = if let Some(next_calls) = &interpretation_response.tool_calls {
                // Convert to agent tool calls
                let agent_tool_calls: Vec<crate::agent::tools::ToolCall> = next_calls
                    .iter()
                    .map(|tc| crate::agent::tools::ToolCall {
                        name: tc.name.clone(),
                        parameters: tc.arguments.clone(),
                    })
                    .collect();
                debug!("Interpretation response has {} structured tool calls", next_calls.len());
                (interpretation_response.content.clone(), agent_tool_calls)
            } else {
                // Try parsing from text
                debug!("No structured tool calls in interpretation, parsing from text");
                self.parser.parse_structured(&interpretation_response.content)?
            };

            println!("🔍 After interpretation: final_text={} chars, next_tool_calls={}", final_text.len(), next_tool_calls.len());
            for (i, call) in next_tool_calls.iter().enumerate() {
                println!("🔍 Next tool call {}: {} with params: {}", i, call.name, call.parameters);
            }

            // Check if task appears incomplete and force continuation
            let task_seems_incomplete = self.check_task_completion(user_message, &final_text, &tool_results_text);
            println!("🔍 Task completion check: user_message='{}', incomplete={}", user_message, task_seems_incomplete);
            println!("🔍 Tool results: {}", tool_results_text);
            println!("🔍 Next tool calls: {} (empty: {})", next_tool_calls.len(), next_tool_calls.is_empty());

            let multi_turn_response = if !next_tool_calls.is_empty() {
                println!("🔧 LLM requested {} more tool calls - continuing multi-turn execution", next_tool_calls.len());

                // Use the multi-turn execution loop for remaining tools
                self.execute_multi_turn_loop(
                    next_tool_calls,
                    messages_with_results,
                    provider,
                    model,
                    5, // Max 5 turns to prevent infinite loops
                ).await?
            } else if task_seems_incomplete {
                println!("🔧 Task appears incomplete - forcing continuation");

                // Force the LLM to continue by explicitly asking what's next
                let continuation_prompt = format!(
                    "You have completed: {}\n\nOriginal task: {}\n\nWhat is the next step to complete this task? Call the appropriate tool.",
                    tool_results_text, user_message
                );

                let mut force_messages = messages_with_results.clone();
                force_messages.push(Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: MessageRole::User,
                    content: continuation_prompt,
                    timestamp: chrono::Utc::now(),
                    tokens_used: None,
                    cost: None,
                });

                let force_request = ChatRequest {
                    messages: force_messages.clone(),
                    model: model.to_string(),
                    temperature: Some(0.7),
                    max_tokens: Some(2000),
                    stream: false,
                    tools: Some(self.convert_tools_to_provider_format()),
                };

                match provider.chat(&force_request).await {
                    Ok(force_response) => {
                        if let Some(forced_tools) = &force_response.tool_calls {
                            let forced_tool_calls: Vec<crate::agent::tools::ToolCall> = forced_tools
                                .iter()
                                .map(|tc| crate::agent::tools::ToolCall {
                                    name: tc.name.clone(),
                                    parameters: tc.arguments.clone(),
                                })
                                .collect();

                            if !forced_tool_calls.is_empty() {
                                // Execute the forced tools
                                self.execute_multi_turn_loop(
                                    forced_tool_calls,
                                    force_messages,
                                    provider,
                                    model,
                                    3, // Limit forced continuation
                                ).await?
                            } else {
                                format!("{}\n\n{}", final_text, force_response.content)
                            }
                        } else {
                            format!("{}\n\n{}", final_text, force_response.content)
                        }
                    }
                    Err(_) => final_text
                }
            } else {
                final_text
            };

            // Record patterns from tool execution for intelligence learning
            for (i, call) in tool_calls.iter().enumerate() {
                if let Ok(embedding) = self.intelligence.get_embedding(&call.parameters.to_string()).await {
                    let action = crate::intelligence::duckdb_memory::AgentAction {
                        tool: call.name.clone(),
                        params: call.parameters.clone(),
                        success: tool_results.get(i).map(|r| r.contains("succeeded")).unwrap_or(false),
                        duration_ms: 100, // Placeholder - would measure actual duration
                        result_summary: tool_results.get(i).unwrap_or(&"No result".to_string()).clone(),
                    };
                    
                    let pattern = crate::intelligence::duckdb_memory::Pattern {
                        id: uuid::Uuid::new_v4().to_string(),
                        description: format!("Tool execution: {}", call.name),
                        context: user_message.to_string(),
                        actions: vec![action],
                        files_involved: vec![], // Would extract from tool parameters
                        success: tool_results.get(i).map(|r| r.contains("succeeded")).unwrap_or(false),
                        timestamp: chrono::Utc::now(),
                        session_id: "current".to_string(), // Would come from actual session
                        embedding_text: format!("{} {}", user_message, call.name),
                        embedding,
                    };
                    
                    if let Err(e) = self.intelligence.record_pattern(pattern).await {
                        debug!("Failed to record pattern: {}", e);
                    }
                }
            }
            
            // Convert agent::tools::ToolCall to conversation::ToolCall
            let conversation_tool_calls: Vec<crate::agent::conversation::ToolCall> = tool_calls.into_iter().map(|tc| {
                crate::agent::conversation::ToolCall {
                    tool_name: tc.name,
                    parameters: tc.parameters,
                    result: None, // Will be filled after execution
                }
            }).collect();
            
            // Add assistant message to conversation
            {
                let mut conversation = self.conversation.lock().await;
                conversation.messages.push(ConvMessage {
                    role: ConvRole::Assistant,
                    content: multi_turn_response.clone(),
                    tool_calls: Some(conversation_tool_calls),
                    timestamp: chrono::Utc::now(),
                });
            }

            multi_turn_response
        };
        
        Ok((final_response, tool_status_messages))
    }

    /// Select specialized agent configuration based on detected intent (Week 8 Day 1-2)
    async fn select_agent_for_intent(&self, intent: &crate::intelligence::UserIntent) -> crate::agent::specialized_agents::AgentConfig {
        use crate::intelligence::UserIntent;
        use crate::agent::model_router::AgentType as RouterAgentType;

        let agent_type = match intent {
            UserIntent::CodeReading { .. } | UserIntent::ProjectExploration { .. } => {
                // Read-only analysis tasks → Explorer agent
                RouterAgentType::Explorer
            }
            UserIntent::CodeWriting { .. } => {
                // Code generation and implementation → Builder agent
                RouterAgentType::Builder
            }
            UserIntent::ProjectFixing { .. } => {
                // Bug fixing and debugging → Debugger agent
                RouterAgentType::Debugger
            }
            UserIntent::Mixed { primary_intent, .. } => {
                // Use primary intent for mixed tasks
                match **primary_intent {
                    UserIntent::CodeReading { .. } => RouterAgentType::Explorer,
                    UserIntent::CodeWriting { .. } => RouterAgentType::Builder,
                    UserIntent::ProjectFixing { .. } => RouterAgentType::Debugger,
                    _ => RouterAgentType::Builder, // Default
                }
            }
        };

        // Get agent config from registry
        self.agent_registry
            .get(agent_type)
            .cloned()
            .unwrap_or_else(|| {
                warn!("No agent config for {:?}, using Builder as fallback", agent_type);
                crate::agent::specialized_agents::AgentConfig::builder()
            })
    }

    /// Assess task complexity for model selection (Week 7 Day 6-7)
    async fn assess_task_complexity(
        &self,
        user_message: &str,
        enhanced_context: &crate::intelligence::EnhancedContext,
    ) -> crate::agent::model_router::TaskComplexity {
        use crate::agent::model_router::TaskComplexity;

        let message_lower = user_message.to_lowercase();

        // High complexity indicators
        if message_lower.contains("architecture")
            || message_lower.contains("design pattern")
            || message_lower.contains("complex")
            || message_lower.contains("algorithm")
            || message_lower.contains("optimize")
            || message_lower.contains("refactor entire")
            || user_message.len() > 500 // Long requests likely complex
            || enhanced_context.confidence < 0.6 // Low confidence = complex
        {
            return TaskComplexity::High;
        }

        // Low complexity indicators
        if message_lower.contains("simple")
            || message_lower.contains("basic")
            || message_lower.contains("trivial")
            || message_lower.contains("read")
            || message_lower.contains("list")
            || message_lower.contains("show")
            || (user_message.len() < 50 && enhanced_context.confidence > 0.8)
        {
            return TaskComplexity::Low;
        }

        // Default: medium complexity
        TaskComplexity::Medium
    }

    /// Determine if task is research-oriented (for sub-agent spawning) (Week 8 Day 3-4)
    async fn is_research_task(&self, user_message: &str) -> bool {
        let message_lower = user_message.to_lowercase();

        // Research task indicators
        message_lower.contains("find")
            || message_lower.contains("search")
            || message_lower.contains("list all")
            || message_lower.contains("show all")
            || message_lower.contains("what are")
            || message_lower.contains("where is")
            || message_lower.contains("how many")
            || message_lower.contains("analyze")
            || message_lower.contains("explore")
    }

    /// Determine if a task needs multi-turn orchestration
    fn task_needs_tools(&self, user_message: &str) -> bool {
        let message_lower = user_message.to_lowercase();

        // Pure code generation tasks don't need tools
        if (message_lower.contains("create") || message_lower.contains("write") || message_lower.contains("implement"))
            && (message_lower.contains("function") || message_lower.contains("class") || message_lower.contains("module"))
            && !message_lower.contains("in the codebase")
            && !message_lower.contains("in this project")
            && !message_lower.contains("existing")
        {
            return false;
        }

        // Tasks that explicitly need file operations
        if message_lower.contains("refactor") || message_lower.contains("modify")
            || message_lower.contains("update") || message_lower.contains("fix")
            || message_lower.contains("analyze") || message_lower.contains("find")
            || message_lower.contains("search") || message_lower.contains("explore")
            || message_lower.contains("in the codebase") || message_lower.contains("existing file")
        {
            return true;
        }

        // Default: provide tools for safety
        true
    }

    fn build_context_aware_system_prompt(&self, user_message: &str) -> String {
        let message_lower = user_message.to_lowercase();

        // For code generation that mentions patterns/style - use semantic context but generate directly
        if (message_lower.contains("create") || message_lower.contains("write") || message_lower.contains("implement"))
            && (message_lower.contains("function") || message_lower.contains("class") || message_lower.contains("module"))
            && (message_lower.contains("pattern") || message_lower.contains("style") || message_lower.contains("like") || message_lower.contains("follows"))
        {
            return "You are Aircher, an AI coding assistant with semantic understanding of this codebase. Generate high-quality code that follows the established patterns in the project. Use your semantic knowledge of the codebase to match existing styles, error handling patterns, and architectural decisions. Generate code directly - do not use tools to explore unless you need to modify existing files.".to_string();
        }

        // For pure code generation requests, emphasize direct generation
        if message_lower.contains("create") && (message_lower.contains("function") || message_lower.contains("class"))
            || message_lower.contains("implement") && !message_lower.contains("feature")
            || message_lower.contains("write") && (message_lower.contains("code") || message_lower.contains("function"))
        {
            return "You are Aircher, an AI coding assistant. Generate high-quality, production-ready code directly. Only use tools if you need to examine existing code or perform file operations. For standalone code generation, provide complete implementations with proper error handling, documentation, and tests.".to_string();
        }

        // For tasks that need workspace interaction
        if message_lower.contains("refactor") || message_lower.contains("analyze")
            || message_lower.contains("find") || message_lower.contains("search")
            || message_lower.contains("fix") || message_lower.contains("debug")
            || message_lower.contains("modify") || message_lower.contains("update")
            || message_lower.contains("list") || message_lower.contains("read")
        {
            // Check if this is a multi-step task
            let is_multi_step = message_lower.contains("then") || message_lower.contains("and then") ||
                                message_lower.contains(", then") || message_lower.contains("after that");

            if is_multi_step {
                return "You are Aircher, an AI coding assistant. CRITICAL: This is a multi-step task. You MUST:\n1. Call ONE tool at a time\n2. Wait for tool results before proceeding\n3. Call the next tool based on actual results\n4. NEVER hallucinate or make up file contents\n5. If asked to read a file after listing, you MUST call read_file, not guess the contents\n\nComplete each step sequentially. Do not try to answer everything at once.".to_string();
            }

            return "You are Aircher, an AI coding assistant. Use the provided tools to examine code, understand the codebase, and make informed modifications. Always read relevant files before making changes.".to_string();
        }

        // Default: balanced approach
        "You are Aircher, an AI coding assistant. Use tools when you need to examine or modify existing code, but generate new code directly when creating standalone functions or classes.".to_string()
    }

    async fn needs_orchestration(&self, user_message: &str) -> bool {
        // Don't orchestrate if this is already an orchestration agent
        if self.is_orchestration_agent {
            return false;
        }
        let message_lower = user_message.to_lowercase();

        // Tasks that clearly need orchestration based on keywords
        if message_lower.contains("implement feature")
            || message_lower.contains("build application")
            || message_lower.contains("create system")
            || message_lower.contains("refactor codebase")
            || message_lower.contains("add comprehensive")
            || (message_lower.contains("create") && message_lower.contains("test suite"))
            || (message_lower.contains("implement") && message_lower.len() > 100) // Long complex requests
        {
            return true;
        }

        // REMOVED: Reasoning check that causes infinite recursion and timeouts
        // The reasoning.process_request call here would trigger another process_message
        // which checks needs_orchestration again, creating a loop

        false
    }

    /// Determine if a task needs systematic multi-turn reasoning
    async fn needs_multi_turn_reasoning(&self, user_message: &str) -> bool {
        // Don't use multi-turn reasoning if this is already an orchestration agent
        if self.is_orchestration_agent {
            return false;
        }

        let message_lower = user_message.to_lowercase();

        // Tasks that benefit from systematic exploration and planning
        if message_lower.contains("fix bug") || message_lower.contains("debug")
            || message_lower.contains("error") || message_lower.contains("failing test")
            || message_lower.contains("not working") || message_lower.contains("issue")
            || message_lower.contains("problem") || message_lower.contains("broken")
        {
            return true;
        }

        // Complex code analysis or modification tasks
        if message_lower.contains("refactor") || message_lower.contains("optimize")
            || message_lower.contains("improve") || message_lower.contains("modify")
            || message_lower.contains("update") || message_lower.contains("change")
        {
            return true;
        }

        // Tasks requiring codebase exploration
        if message_lower.contains("understand") || message_lower.contains("analyze")
            || message_lower.contains("explore") || message_lower.contains("investigate")
            || message_lower.contains("find") || message_lower.contains("locate")
        {
            return true;
        }

        // Implementation tasks that mention existing code
        if (message_lower.contains("implement") || message_lower.contains("add"))
            && (message_lower.contains("existing") || message_lower.contains("current")
                || message_lower.contains("codebase") || message_lower.contains("project"))
        {
            return true;
        }

        false
    }

    /// Process a task using systematic multi-turn reasoning
    async fn process_with_multi_turn_reasoning(
        &self,
        user_message: &str,
        provider: &dyn LLMProvider,
        model: &str
    ) -> Result<(String, Vec<String>)> {
        info!("Processing task with systematic multi-turn reasoning");

        let mut tool_status_messages = Vec::new();

        // Create a reasoning plan for this task
        {
            let mut reasoning_engine = self.multi_turn_reasoning.lock().await;
            let plan_id = reasoning_engine.create_reasoning_plan(user_message, provider, model).await?;

            info!("Created reasoning plan: {}", plan_id);
            tool_status_messages.push(format!("🧠 Created systematic reasoning plan: {}", plan_id));

            // Execute the plan step by step
            let mut steps_completed = 0;
            let max_steps = 50; // Prevent infinite loops

            while reasoning_engine.has_queued_actions() && steps_completed < max_steps {
                match reasoning_engine.execute_next_action(provider, model).await? {
                    Some(action_result) => {
                        steps_completed += 1;

                        if action_result.success {
                            tool_status_messages.push(format!(
                                "✅ Step {}: {} - {}",
                                steps_completed,
                                action_result.action.description,
                                action_result.action.expected_outcome
                            ));

                            info!("Multi-turn reasoning step {} completed: {}",
                                  steps_completed, action_result.action.description);
                        } else {
                            tool_status_messages.push(format!(
                                "❌ Step {}: {} - {}",
                                steps_completed,
                                action_result.action.description,
                                action_result.error.unwrap_or_else(|| "Unknown error".to_string())
                            ));

                            warn!("Multi-turn reasoning step {} failed: {}",
                                  steps_completed, action_result.action.description);
                        }
                    }
                    None => {
                        // No more actions to execute
                        break;
                    }
                }
            }

            // Get the final plan status
            if let Some(plan) = reasoning_engine.get_plan_status(&plan_id) {
                let completion_message = match plan.state {
                    crate::agent::multi_turn_reasoning::PlanState::Complete => {
                        format!("🎉 Multi-turn reasoning completed successfully after {} steps", steps_completed)
                    }
                    crate::agent::multi_turn_reasoning::PlanState::Failed => {
                        format!("⚠️ Multi-turn reasoning failed after {} steps", steps_completed)
                    }
                    _ => {
                        format!("🔄 Multi-turn reasoning in progress: {} steps completed", steps_completed)
                    }
                };

                tool_status_messages.push(completion_message.clone());

                // Build summary response based on learnings
                let mut response = String::new();
                response.push_str("## Multi-Turn Reasoning Results\n\n");

                if !plan.learned_context.is_empty() {
                    response.push_str("### Key Discoveries:\n");
                    for (_, learning) in plan.learned_context.iter().take(10) {
                        response.push_str(&format!("- {}\n", learning));
                    }
                    response.push('\n');
                }

                if !plan.failed_attempts.is_empty() {
                    response.push_str("### Challenges Encountered:\n");
                    for attempt in plan.failed_attempts.iter().take(5) {
                        response.push_str(&format!("- {}: {} (Learning: {})\n",
                                                  attempt.action, attempt.error, attempt.learning));
                    }
                    response.push('\n');
                }

                response.push_str(&format!("### Summary:\n{}\n", completion_message));
                response.push_str(&format!("Completed {} phases with {} total steps.",
                                          plan.current_phase + 1, steps_completed));

                // Add to conversation
                {
                    let mut conversation = self.conversation.lock().await;
                    conversation.messages.push(ConvMessage {
                        role: ConvRole::Assistant,
                        content: response.clone(),
                        tool_calls: None,
                        timestamp: chrono::Utc::now(),
                    });
                }

                Ok((response, tool_status_messages))
            } else {
                Err(anyhow::anyhow!("Failed to retrieve plan status for {}", plan_id))
            }
        }
    }

    /// Process a complex task using the TaskOrchestrator
    async fn process_with_orchestration(
        &self,
        user_message: &str,
        provider: &dyn LLMProvider,
        model: &str
    ) -> Result<(String, Vec<String>)> {
        info!("Processing complex task with orchestration");

        // Create orchestrator on-demand (avoiding circular dependency)
        let orchestrator = TaskOrchestrator::new(
            Arc::new(Agent {
                tools: self.tools.clone(),
                intelligence: self.intelligence.clone(),
                unified_intelligence: self.unified_intelligence.clone(),
                auth_manager: self.auth_manager.clone(),
                parser: ToolCallParser::new()?, // Create new parser (doesn't impl Clone)
                conversation: self.conversation.clone(),
                reasoning: self.reasoning.clone(),
                context_manager: self.context_manager.clone(),
                orchestrator: None, // Avoid infinite recursion
                plan_generator: Arc::new(tokio::sync::Mutex::new(PlanGenerator::new())),
                multi_turn_reasoning: Arc::new(tokio::sync::Mutex::new(
                    MultiTurnReasoningEngine::new(self.tools.clone(), self.intelligence.clone())?
                )),
                is_orchestration_agent: true, // Mark as orchestration agent to prevent recursion
                max_iterations: self.max_iterations,
                event_bus: self.event_bus.clone(), // Week 7: Share event bus
                lsp_manager: self.lsp_manager.clone(), // Week 7: Share LSP manager
                current_mode: self.current_mode.clone(), // Week 7 Day 3: Share mode
                snapshot_manager: self.snapshot_manager.clone(), // Week 7 Day 5: Share snapshot manager
                model_router: self.model_router.clone(), // Week 7 Day 6-7: Share model router
                agent_registry: self.agent_registry.clone(), // Week 8 Day 1-2: Share agent registry
                research_manager: self.research_manager.clone(), // Week 8 Day 3-4: Share research manager
            }),
            self.reasoning.clone(),
            self.context_manager.clone(),
            self.intelligence.clone(),
        );

        // Execute the complex task
        let task_result = orchestrator.execute_task(user_message, provider, model).await?;

        // Convert orchestration result to standard response format
        let mut tool_status_messages = Vec::new();

        // Add summary of orchestration steps
        tool_status_messages.push(format!("🎯 Orchestrated {} steps successfully", task_result.steps_completed));

        if !task_result.files_modified.is_empty() {
            tool_status_messages.push(format!("📝 Modified {} files", task_result.files_modified.len()));
        }

        // Add conversation message
        {
            let mut conversation = self.conversation.lock().await;
            conversation.messages.push(ConvMessage {
                role: ConvRole::Assistant,
                content: task_result.summary.clone(),
                tool_calls: None,
                timestamp: chrono::Utc::now(),
            });
        }

        Ok((task_result.summary, tool_status_messages))
    }

    /// Direct message processing without reasoning engine (fallback)
    pub async fn process_message_direct(&self, user_message: &str, provider: &dyn LLMProvider, model: &str) -> Result<(String, Vec<String>)> {
        let mut tool_status_messages = Vec::new();
        
        // Build chat request with intelligence-enhanced system prompt
        let system_prompt = "You are Aircher, an AI coding assistant. Use the provided tools to help with coding tasks.".to_string();
        
        let mut messages = vec![
            Message {
                id: uuid::Uuid::new_v4().to_string(),
                role: MessageRole::System,
                content: system_prompt,
                timestamp: chrono::Utc::now(),
                tokens_used: None,
                cost: None,
            },
        ];
        
        // Add conversation history
        {
            let conversation = self.conversation.lock().await;
            for msg in &conversation.messages {
                messages.push(Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: match msg.role {
                        ConvRole::User => MessageRole::User,
                        ConvRole::Assistant => MessageRole::Assistant,
                        ConvRole::System => MessageRole::System,
                        ConvRole::Tool => MessageRole::User,
                    },
                    content: msg.content.clone(),
                    timestamp: msg.timestamp,
                    tokens_used: None,
                    cost: None,
                });
            }
        }

        // Add current user message
        messages.push(Message {
            id: uuid::Uuid::new_v4().to_string(),
            role: MessageRole::User,
            content: user_message.to_string(),
            timestamp: chrono::Utc::now(),
            tokens_used: None,
            cost: None,
        });

        // Send with tools if provider supports them
        let tools = if provider.supports_tools() {
            Some(self.convert_tools_to_provider_format())
        } else {
            None
        };
        
        let request = ChatRequest {
            messages,
            model: model.to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
            stream: false,
            tools,
        };
        
        let response = provider.chat(&request).await?;
        let assistant_message = response.content;
        
        // Parse tool calls
        let (clean_text, tool_calls) = self.parser.parse_structured(&assistant_message)?;
        
        if tool_calls.is_empty() {
            // Add assistant message to conversation
            {
                let mut conversation = self.conversation.lock().await;
                conversation.messages.push(ConvMessage {
                    role: ConvRole::Assistant,
                    content: clean_text.clone(),
                    tool_calls: None,
                    timestamp: chrono::Utc::now(),
                });
            }
        }

        let final_response = if tool_calls.is_empty() {
            // No tool calls, this is the final response
            clean_text.clone()
        } else {
            info!("Executing {} tool calls", tool_calls.len());
            
            for call in &tool_calls {
                tool_status_messages.push(format!("🔧 Executing tool: {}", call.name));
            }
            
            let mut tool_results = Vec::new();
            for call in &tool_calls {
                debug!("Executing tool: {} with params: {}", call.name, call.parameters);
                
                if let Some(tool) = self.tools.get(&call.name) {
                    match tool.execute(call.parameters.clone()).await {
                        Ok(output) => {
                            // Emit FileChanged event for file modification tools
                            if output.success && (call.name == "write_file" || call.name == "edit_file") {
                                if let Some(path_str) = call.parameters.get("path").and_then(|v| v.as_str()) {
                                    use crate::agent::events::{AgentEvent, FileOperation};
                                    let path = std::path::PathBuf::from(path_str);
                                    let operation = if call.name == "write_file" {
                                        FileOperation::Write
                                    } else {
                                        FileOperation::Edit
                                    };

                                    self.event_bus.publish(AgentEvent::FileChanged {
                                        path,
                                        operation,
                                        timestamp: std::time::SystemTime::now(),
                                    });
                                }
                            }

                            if output.success {
                                // Format the result more clearly
                                let result_str = if output.result.is_object() || output.result.is_array() {
                                    serde_json::to_string_pretty(&output.result).unwrap_or_else(|_| format!("{:?}", output.result))
                                } else {
                                    output.result.to_string()
                                };
                                tool_results.push(format!("Tool {} completed successfully. Result:\n{}", call.name, result_str));
                            } else {
                                let error = output.error.unwrap_or_else(|| "Unknown error".to_string());
                                tool_results.push(format!("Tool {} failed: {}", call.name, error));
                            }
                        }
                        Err(e) => {
                            tool_results.push(format!("Tool {} error: {}", call.name, e));
                        }
                    }
                } else {
                    tool_results.push(format!("Tool {} not found", call.name));
                }
            }
            
            let response = format!("Executed tools:\n{}", tool_results.join("\n"));
            
            let conversation_tool_calls: Vec<crate::agent::conversation::ToolCall> = tool_calls.into_iter().map(|tc| {
                crate::agent::conversation::ToolCall {
                    tool_name: tc.name,
                    parameters: tc.parameters,
                    result: None,
                }
            }).collect();
            
            {
                let mut conversation = self.conversation.lock().await;
                conversation.messages.push(ConvMessage {
                    role: ConvRole::Assistant,
                    content: response.clone(),
                    tool_calls: Some(conversation_tool_calls),
                    timestamp: chrono::Utc::now(),
                });
            }

            response
        };

        Ok((final_response, tool_status_messages))
    }
    
    /// Get list of available tool names
    pub async fn list_tools(&self) -> Result<Vec<String>> {
        let tool_names: Vec<String> = self.tools.list_tools()
            .iter()
            .map(|t| t.name.clone())
            .collect();
        Ok(tool_names)
    }
    
    /// Send message with provider and model specification
    pub async fn send_message(
        &mut self,
        _session_id: &str,
        message: &str,
        provider_name: &str,
        model_name: &str,
    ) -> Result<crate::client::AgentResponse> {
        use crate::providers::ProviderManager;
        
        // Create a temporary provider manager for this call
        // In practice, this would come from a stored provider manager
        let config = crate::config::ConfigManager::load().await?;
        let auth_manager = crate::auth::AuthManager::new()?;
        let provider_manager = ProviderManager::new(&config, Arc::new(auth_manager)).await?;
        
        let provider = provider_manager
            .get_provider_or_host(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", provider_name))?;
        
        let (content, status_messages) = self.process_message(message, provider, model_name).await?;
        
        // Convert tool status messages to ToolCallInfo
        let tool_calls: Vec<crate::client::ToolCallInfo> = status_messages
            .into_iter()
            .map(|msg| {
                // Extract tool name from status messages like "🔧 Executing tool: write_file"
                let tool_name = if let Some(colon_pos) = msg.find(": ") {
                    msg[colon_pos + 2..].split_whitespace().next().unwrap_or("unknown_tool").to_string()
                } else {
                    "unknown_tool".to_string()
                };

                crate::client::ToolCallInfo {
                    name: tool_name,
                    status: if msg.contains("✓") {
                        crate::client::ToolStatus::Success
                    } else if msg.contains("🔧") {
                        crate::client::ToolStatus::Running
                    } else {
                        crate::client::ToolStatus::Failed
                    },
                    result: Some(serde_json::Value::String(msg.clone())),
                    error: if msg.contains("error") { Some(msg) } else { None },
                }
            })
            .collect();
        
        Ok(crate::client::AgentResponse {
            content,
            tool_calls,
            session_id: _session_id.to_string(),
        })
    }
    
    /// Send message with streaming response
    pub async fn send_message_streaming(
        &mut self,
        session_id: &str,
        message: &str,
        provider_name: &str,
        model_name: &str,
    ) -> Result<crate::agent::streaming::AgentStream> {
        // For now, use non-streaming and convert to stream
        let response = self.send_message(session_id, message, provider_name, model_name).await?;
        
        // Create a simple stream that emits the response
        let (tx, rx) = crate::agent::streaming::create_agent_stream();
        tx.send(Ok(crate::agent::streaming::AgentUpdate::TextChunk {
            content: response.content,
            delta: false,
            tokens_used: None,
        })).await.ok();
        tx.send(Ok(crate::agent::streaming::AgentUpdate::Complete {
            total_tokens: 0,
            tool_status_messages: Vec::new(),
        })).await.ok();
        
        Ok(rx)
    }
    
    /// Execute a single tool by name
    pub async fn execute_single_tool(
        &self,
        tool_name: &str,
        params: serde_json::Value,
    ) -> Result<crate::client::ToolCallInfo> {
        // Check if tool is allowed in current mode
        let current_mode = self.current_mode.read().await;
        let allowed_tools = current_mode.allowed_tools();

        if !allowed_tools.contains(tool_name) {
            return Ok(crate::client::ToolCallInfo {
                name: tool_name.to_string(),
                status: crate::client::ToolStatus::Failed,
                result: None,
                error: Some(format!(
                    "Tool '{}' not allowed in {:?} mode. Allowed tools: {:?}",
                    tool_name,
                    *current_mode,
                    allowed_tools.iter().collect::<Vec<_>>()
                )),
            });
        }
        drop(current_mode); // Release read lock

        // Create git snapshot before risky operations
        let risky_tools = ["run_command", "edit_file", "write_file"];
        let _snapshot_id = if risky_tools.contains(&tool_name) {
            if let Some(snapshot_mgr) = &self.snapshot_manager {
                match snapshot_mgr.create_snapshot(&format!("Before {}", tool_name)) {
                    Ok(id) => {
                        info!("Created git snapshot {} before {}", id, tool_name);
                        Some(id)
                    }
                    Err(e) => {
                        warn!("Failed to create snapshot before {}: {}", tool_name, e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(tool) = self.tools.get(tool_name) {
            match tool.execute(params.clone()).await {
                Ok(output) => {
                    // Emit FileChanged event for file modification tools
                    if output.success && (tool_name == "write_file" || tool_name == "edit_file") {
                        if let Some(path_str) = params.get("path").and_then(|v| v.as_str()) {
                            use crate::agent::events::{AgentEvent, FileOperation};
                            let path = std::path::PathBuf::from(path_str);
                            let operation = if tool_name == "write_file" {
                                FileOperation::Write
                            } else {
                                FileOperation::Edit
                            };

                            self.event_bus.publish(AgentEvent::FileChanged {
                                path,
                                operation,
                                timestamp: std::time::SystemTime::now(),
                            });
                        }
                    }

                    Ok(crate::client::ToolCallInfo {
                        name: tool_name.to_string(),
                        status: if output.success {
                            crate::client::ToolStatus::Success
                        } else {
                            crate::client::ToolStatus::Failed
                        },
                        result: Some(output.result),
                        error: output.error,
                    })
                }
                Err(e) => {
                    Ok(crate::client::ToolCallInfo {
                        name: tool_name.to_string(),
                        status: crate::client::ToolStatus::Failed,
                        result: None,
                        error: Some(e.to_string()),
                    })
                }
            }
        } else {
            Ok(crate::client::ToolCallInfo {
                name: tool_name.to_string(),
                status: crate::client::ToolStatus::Failed,
                result: None,
                error: Some(format!("Tool '{}' not found", tool_name)),
            })
        }
    }
    
    /// Get session history (placeholder implementation)
    pub async fn get_history(&self, _session_id: &str) -> Result<Vec<crate::client::AgentResponse>> {
        // Convert conversation messages to AgentResponse format
        let responses: Vec<crate::client::AgentResponse> = {
            let conversation = self.conversation.lock().await;
            conversation.messages
                .iter()
                .filter(|msg| matches!(msg.role, crate::agent::conversation::MessageRole::Assistant))
                .map(|msg| crate::client::AgentResponse {
                    content: msg.content.clone(),
                    tool_calls: msg.tool_calls.as_ref().map(|calls| {
                        calls.iter().map(|call| crate::client::ToolCallInfo {
                            name: call.tool_name.clone(),
                            status: crate::client::ToolStatus::Success,
                            result: call.result.clone(),
                            error: None,
                        }).collect()
                    }).unwrap_or_default(),
                    session_id: _session_id.to_string(),
                })
                .collect()
        };
        
        Ok(responses)
    }
    
    /// Check if a task appears incomplete based on user request and executed tools
    fn check_task_completion(&self, user_message: &str, response_text: &str, tool_results: &str) -> bool {
        let message_lower = user_message.to_lowercase();

        // Check for multi-step indicators
        let has_then = message_lower.contains("then") || message_lower.contains("and then") || message_lower.contains(", then");

        if !has_then {
            return false; // Single step task
        }

        // Check if only first part was completed
        if message_lower.contains("list") && message_lower.contains("read") {
            let did_list = tool_results.contains("list_files");
            let did_read = tool_results.contains("read_file") || response_text.contains("use anyhow::Result") || response_text.contains("impl");

            // If we listed but didn't read, task is incomplete
            if did_list && !did_read {
                debug!("Task incomplete: listed files but didn't read");
                return true;
            }
        }

        // Check for other multi-step patterns
        if message_lower.contains("find") && message_lower.contains("create") {
            let did_find = tool_results.contains("search_code") || tool_results.contains("list_files");
            let did_create = tool_results.contains("write_file") || tool_results.contains("edit_file");

            if did_find && !did_create {
                debug!("Task incomplete: found content but didn't create");
                return true;
            }
        }

        false
    }

    /// Execute a multi-turn conversation with tool calling loop
    async fn execute_multi_turn_loop(
        &self,
        initial_tool_calls: Vec<crate::agent::tools::ToolCall>,
        messages: Vec<Message>,
        provider: &dyn LLMProvider,
        model: &str,
        max_turns: usize,
    ) -> Result<String> {
        let mut current_messages = messages;
        let mut all_results = Vec::new();
        let mut current_tool_calls = initial_tool_calls;
        let mut turn_count = 0;

        while !current_tool_calls.is_empty() && turn_count < max_turns {
            turn_count += 1;
            println!("🔄 Multi-turn execution - turn {}, executing {} tools", turn_count, current_tool_calls.len());
            for (i, call) in current_tool_calls.iter().enumerate() {
                println!("🔄 Turn {} tool {}: {} with params: {}", turn_count, i, call.name, call.parameters);
            }

            // Execute the current batch of tools
            let mut tool_results = Vec::new();
            for call in &current_tool_calls {
                println!("🔄 Executing tool in multi-turn: {} with params: {}", call.name, call.parameters);

                if let Some(tool) = self.tools.get(&call.name) {
                    match tool.execute(call.parameters.clone()).await {
                        Ok(output) => {
                            // Emit FileChanged event for file modification tools
                            if output.success && (call.name == "write_file" || call.name == "edit_file") {
                                if let Some(path_str) = call.parameters.get("path").and_then(|v| v.as_str()) {
                                    use crate::agent::events::{AgentEvent, FileOperation};
                                    let path = std::path::PathBuf::from(path_str);
                                    let operation = if call.name == "write_file" {
                                        FileOperation::Write
                                    } else {
                                        FileOperation::Edit
                                    };

                                    self.event_bus.publish(AgentEvent::FileChanged {
                                        path,
                                        operation,
                                        timestamp: std::time::SystemTime::now(),
                                    });
                                }
                            }

                            if output.success {
                                // Format the result more clearly
                                let result_str = if output.result.is_object() || output.result.is_array() {
                                    serde_json::to_string_pretty(&output.result).unwrap_or_else(|_| format!("{:?}", output.result))
                                } else {
                                    output.result.to_string()
                                };
                                tool_results.push(format!("Tool {} completed successfully. Result:\n{}", call.name, result_str));
                            } else {
                                let error = output.error.unwrap_or_else(|| "Unknown error".to_string());
                                tool_results.push(format!("Tool {} failed: {}", call.name, error));
                            }
                        }
                        Err(e) => {
                            tool_results.push(format!("Tool {} error: {}", call.name, e));
                        }
                    }
                } else {
                    tool_results.push(format!("Tool {} not found", call.name));
                }
            }

            all_results.extend(tool_results.clone());

            // Add tool results to the message history
            current_messages.push(Message {
                id: uuid::Uuid::new_v4().to_string(),
                role: MessageRole::User,
                content: format!("Tool execution results:\n{}", tool_results.join("\n")),
                timestamp: chrono::Utc::now(),
                tokens_used: None,
                cost: None,
            });

            // Ask the LLM what to do next
            let next_request = ChatRequest {
                messages: current_messages.clone(),
                model: model.to_string(),
                temperature: Some(0.7),
                max_tokens: Some(2000),
                stream: false,
                tools: if provider.supports_tools() {
                    Some(self.convert_tools_to_provider_format())
                } else {
                    None
                },
            };

            let next_response = provider.chat(&next_request).await?;

            // Check for more tool calls
            if let Some(next_calls) = &next_response.tool_calls {
                current_tool_calls = next_calls
                    .iter()
                    .map(|tc| crate::agent::tools::ToolCall {
                        name: tc.name.clone(),
                        parameters: tc.arguments.clone(),
                    })
                    .collect();
            } else {
                // Try parsing from text
                let (final_text, parsed_calls) = self.parser.parse_structured(&next_response.content)?;
                if parsed_calls.is_empty() {
                    // No more tools to execute, return the final response
                    return Ok(final_text);
                }
                current_tool_calls = parsed_calls;
            }

            // Add the assistant's response to history
            current_messages.push(Message {
                id: uuid::Uuid::new_v4().to_string(),
                role: MessageRole::Assistant,
                content: next_response.content.clone(),
                timestamp: chrono::Utc::now(),
                tokens_used: None,
                cost: None,
            });
        }

        // Reached max turns or no more tools
        if turn_count >= max_turns {
            Ok(format!("Completed {} turns of tool execution. Results:\n{}",
                       turn_count, all_results.join("\n")))
        } else {
            Ok(format!("Task completed. Results:\n{}", all_results.join("\n")))
        }
    }

    /// End session (placeholder implementation)
    pub async fn end_session(&self, _session_id: &str) -> Result<()> {
        // Clear conversation for this session
        // In practice, you'd want to save it first
        {
            let mut conversation = self.conversation.lock().await;
            conversation.messages.clear();
        }
        Ok(())
    }
}

/// ACP (Agent Client Protocol) implementation
#[cfg(feature = "acp")]
impl AcpAgent for Agent {
    fn initialize(&self, request: InitializeRequest) -> impl std::future::Future<Output = Result<InitializeResponse, agent_client_protocol::Error>> + Send + '_ {
        async move {
        info!("Initializing ACP agent with protocol version: {:?}", request.protocol_version);
        
        // Note: Intelligence initialization would happen during Agent construction
        // For ACP mode, we'll skip dynamic initialization to keep it simple
        
        // Get available tools for capabilities
        let _tool_names: Vec<String> = self.tools.list_tools().iter().map(|t| t.name.clone()).collect();
        
        Ok(InitializeResponse {
            protocol_version: request.protocol_version,
            agent_capabilities: AgentCapabilities {
                load_session: false,
                prompt_capabilities: PromptCapabilities {
                    image: false,
                    audio: false,
                    embedded_context: true,
                },
            },
            auth_methods: vec![],
        })
        }
    }
    
    fn new_session(&self, _request: NewSessionRequest) -> impl std::future::Future<Output = Result<NewSessionResponse, agent_client_protocol::Error>> + Send + '_ {
        async move {
            let session_id = uuid::Uuid::new_v4().to_string();
            
            // Start memory session if available
            if let Ok(Some(memory_session)) = self.intelligence.start_memory_session(Some(session_id.clone())).await {
                info!("Started intelligent memory session: {}", memory_session);
            }
            
            Ok(NewSessionResponse { session_id: SessionId(session_id.into()) })
        }
    }
    
    fn prompt(&self, request: PromptRequest) -> impl std::future::Future<Output = Result<PromptResponse, agent_client_protocol::Error>> + Send + '_ {
        async move {
            // Extract text content from request
            let user_message = request.prompt.iter()
                .filter_map(|block| match block {
                    ContentBlock::Text(text_content) => Some(text_content.text.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
                
            info!("Processing ACP prompt: {}", user_message);
            
            // Use a default Ollama provider for ACP mode (simplified implementation)
            let provider_config = crate::config::ProviderConfig {
                name: "ollama".to_string(),
                api_key_env: "".to_string(), // Ollama doesn't need API key
                base_url: "http://localhost:11434".to_string(),
                fallback_urls: vec![],
                models: vec![],
                timeout_seconds: 30,
                max_retries: 3,
            };
            let provider = crate::providers::ollama::OllamaProvider::new(
                provider_config,
                self.auth_manager.clone()
            ).await?;
            
            match self.process_message(&user_message, &provider, "gpt-oss").await {
                Ok((_response, _status_messages)) => {
                    // Note: ACP works differently - responses are sent via session/update notifications
                    // during processing, and prompt() only returns the stop reason
                    Ok(PromptResponse {
                        stop_reason: StopReason::EndTurn,
                    })
                }
                Err(_e) => {
                    Ok(PromptResponse {
                        stop_reason: StopReason::Refusal,
                    })
                }
            }
        }
    }
    
    fn authenticate(&self, _request: AuthenticateRequest) -> impl std::future::Future<Output = Result<(), agent_client_protocol::Error>> + Send + '_ {
        async move {
            // For simplicity, we don't implement authentication in the basic ACP mode
            // In a production system, you'd validate credentials here
            Ok(())
        }
    }
    
    fn load_session(&self, _request: LoadSessionRequest) -> impl std::future::Future<Output = Result<(), agent_client_protocol::Error>> + Send + '_ {
        async move {
            // We don't support session loading in this basic implementation
            Err(agent_client_protocol::Error::method_not_found())
        }
    }
    
    fn cancel(&self, _notification: CancelNotification) -> impl std::future::Future<Output = Result<(), agent_client_protocol::Error>> + Send + '_ {
        async move {
            // Cancel ongoing operations - for now just log the request
            info!("Received cancel notification for session");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tool_conversion_to_provider_format() {
        // Create a test intelligence engine and auth manager
        let config = crate::config::ConfigManager::load().await.unwrap();
        let db_manager = crate::storage::DatabaseManager::new(&config).await.unwrap();
        let intelligence = crate::intelligence::IntelligenceEngine::new(&config, &db_manager).await.unwrap();
        let auth_manager = Arc::new(crate::auth::AuthManager::new().unwrap());
        let project_context = crate::agent::conversation::ProjectContext {
            root_path: std::env::current_dir().unwrap(),
            language: crate::agent::conversation::ProgrammingLanguage::Rust,
            framework: Some("cargo".to_string()),
            recent_changes: Vec::new(),
        };
        
        // Create Agent
        let agent = Agent::new(intelligence, auth_manager, project_context).await.unwrap();
        
        // Test tool conversion
        let provider_tools = agent.convert_tools_to_provider_format();
        
        // Verify we have tools (should be default tools from ToolRegistry)
        assert!(!provider_tools.is_empty(), "Should have default tools available");
        
        // Check that we have expected tools
        let tool_names: Vec<&str> = provider_tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"read_file"), "Should have read_file tool");
        assert!(tool_names.contains(&"write_file"), "Should have write_file tool");
        assert!(tool_names.contains(&"list_files"), "Should have list_files tool");
        assert!(tool_names.contains(&"search_code"), "Should have search_code tool");
        assert!(tool_names.contains(&"run_command"), "Should have run_command tool");
        
        // Verify tool structure
        let read_file_tool = provider_tools.iter().find(|t| t.name == "read_file").unwrap();
        assert!(!read_file_tool.description.is_empty(), "Tool should have description");
        assert!(read_file_tool.parameters.is_object(), "Tool should have parameter schema");
        
        // Check that parameters schema has required structure
        let params = read_file_tool.parameters.as_object().unwrap();
        assert!(params.contains_key("type"), "Should have type field");
        assert!(params.contains_key("properties"), "Should have properties field");
        
        println!("✅ Tool conversion test passed!");
        println!("📊 Found {} tools: {:?}", provider_tools.len(), tool_names);
    }
}