# Analysis Report: src/agent/core.rs

**Date**: November 3, 2025
**File Size**: 2,365 lines
**Purpose**: Core Agent implementation for Aircher - unified ACP-compatible agent backend

---

## 1. Main Structs

### `Agent` (lines 31-66)
**Purpose**: Unified agent that serves both TUI and ACP modes with hybrid architecture combining SOTA patterns

**22 Fields**:

#### Core Components
1. `tools: Arc<ToolRegistry>` - Registry of available tools for execution
2. `intelligence: Arc<IntelligenceEngine>` - Intelligence engine for context-aware operations
3. `unified_intelligence: Arc<UnifiedIntelligenceEngine>` - Automatic middleware layer
4. `auth_manager: Arc<AuthManager>` - Authentication and credential management
5. `parser: ToolCallParser` - XML and JSON tool call parsing
6. `conversation: Arc<Mutex<CodingConversation>>` - Thread-safe conversation state
7. `reasoning: Arc<AgentReasoning>` - Intelligent planning capabilities
8. `context_manager: Arc<DynamicContextManager>` - Dynamic context with intelligent pruning

#### Orchestration & Planning
9. `orchestrator: Option<Arc<TaskOrchestrator>>` - Task decomposition (created on-demand)
10. `plan_generator: Arc<Mutex<PlanGenerator>>` - Plan generation logic
11. `multi_turn_reasoning: Arc<Mutex<MultiTurnReasoningEngine>>` - Systematic problem solving
12. `is_orchestration_agent: bool` - Prevents infinite recursion
13. `max_iterations: usize` - Loop prevention (default: 10)

#### Week 7-8 Hybrid Architecture Components
14. `event_bus: SharedEventBus` - Agent-wide event communication (Week 7)
15. `lsp_manager: Arc<LspManager>` - Global diagnostics with LSP integration (Week 7)
16. `current_mode: Arc<RwLock<AgentMode>>` - Plan (read-only) vs Build (modify) mode (Week 7 Day 3)
17. `snapshot_manager: Option<Arc<SnapshotManager>>` - Git snapshots for safe experimentation (Week 7 Day 5)
18. `model_router: Arc<ModelRouter>` - Cost-aware model selection (Week 7 Day 6-7)
19. `agent_registry: Arc<AgentRegistry>` - 7 specialized agent configurations (Week 8 Day 1-2)
20. `research_manager: Arc<ResearchSubAgentManager>` - Parallel research sub-agents (Week 8 Day 3-4)

#### Extensibility
21. `skill_manager: Arc<SkillManager>` - User-extensible capabilities (Week 10 Phase 2)

---

## 2. Public API Methods

### Construction (2 methods)
- `new(intelligence, auth_manager, project_context) -> Result<Self>` (line 69)
  - Standard agent creation
- `new_with_approval(...) -> Result<(Self, approval_rx)>` (line 81)
  - Agent with approval workflow for dangerous operations
  - Returns approval channel for UI integration

### Accessors (9 methods)
- `event_bus() -> &SharedEventBus` (line 243) - Get event bus reference
- `lsp_manager() -> &Arc<LspManager>` (line 248) - Get LSP manager
- `model_router() -> &Arc<ModelRouter>` (line 253) - Get model router
- `skill_manager() -> &Arc<SkillManager>` (line 258) - Get skills manager
- `agent_registry() -> &Arc<AgentRegistry>` (line 282) - Get specialized agents
- `research_manager() -> &Arc<ResearchSubAgentManager>` (line 287) - Get research manager
- `current_mode() -> AgentMode` (line 292) - Get current Plan/Build mode
- `set_mode(new_mode, reason)` (line 297) - Change mode with transition event
- `snapshot_manager() -> Option<&Arc<SnapshotManager>>` (line 316) - Get Git snapshots

### Skills API (3 methods - Week 10 Phase 2)
- `list_skills() -> Result<Vec<SkillMetadata>>` (line 265) - List discovered skills
- `get_skill(name) -> Result<Option<SkillMetadata>>` (line 270) - Get specific skill
- `reload_skills() -> Result<Vec<SkillMetadata>>` (line 277) - Force rescan

### Core Functionality (8+ methods)
- `convert_tools_to_provider_format() -> Vec<Tool>` (line 321) - Convert to LLM provider format
- `process_message(user_message, provider, model) -> Result<(String, Vec<String>)>` (line 333)
  - Main message processing with tool execution loop
  - Returns (response, tool_names_used)
- `process_message_direct(...)` (line 1547) - Direct processing without orchestration
- `list_tools() -> Result<Vec<String>>` (line 1714) - List available tool names
- `send_message(...)` (line 1723) - Send message with provider/model
- `send_message_streaming(...)` (line 1778) - Streaming message processing
- `execute_single_tool(...)` (line 1804) - Execute individual tool
- `get_history(session_id) -> Result<Vec<AgentResponse>>` (line 1997) - Conversation history
- `end_session(session_id) -> Result<()>` (line 2197) - Clean up session

### ACP Protocol Support
Implements `AcpAgent` trait when `acp` feature is enabled (line 7):
- Initialize, new session, prompt, authenticate, load session, cancel

---

## 3. Dependencies on Other Modules

### External Dependencies
- `anyhow` - Error handling with `Result`
- `tracing` - Structured logging (debug, info, warn)
- `agent_client_protocol` - ACP trait and types (conditional)

### Internal Module Dependencies

#### `crate::auth`
- `AuthManager` - Credential and token management

#### `crate::intelligence`
- `IntelligenceEngine` - Core intelligence layer
- `UnifiedIntelligenceEngine` - Automatic middleware wrapper

#### `crate::providers`
- `LLMProvider` - Provider abstraction trait
- `ChatRequest`, `Message`, `MessageRole`, `PricingModel` - Chat types

#### `crate::agent::*` (18 sub-modules)
1. **tools** - `ToolRegistry`, tool implementations
2. **parser** - `ToolCallParser` for XML/JSON parsing
3. **conversation** - `CodingConversation`, conversation state
4. **reasoning** - `AgentReasoning`, `TaskStatus`
5. **dynamic_context** - `DynamicContextManager` for context pruning
6. **task_orchestrator** - `TaskOrchestrator` for task decomposition
7. **plan_mode** - `PlanGenerator`, `PlanMode` for planning
8. **multi_turn_reasoning** - `MultiTurnReasoningEngine` for systematic solving
9. **events** - `SharedEventBus`, `AgentEvent`, `FileOperation`, `AgentMode`
10. **lsp_manager** - `LspManager` for LSP integration
11. **agent_mode** - `ModeTransition` for mode changes
12. **git_snapshots** - `SnapshotManager` for Git operations
13. **model_router** - `ModelRouter` for cost-aware selection
14. **specialized_agents** - `AgentRegistry` for specialized configs
15. **research_subagents** - `ResearchSubAgentManager` for parallel research
16. **skills** - `SkillManager` for user extensibility
17. **approval_modes** - `PendingChange` for approval workflow
18. **tools::approval_registry** - Approval-enabled tool creation

#### `crate::semantic_search`
- `SemanticCodeSearch` - Semantic code search engine

#### `crate::config`
- `ConfigManager` - Configuration loading (line 167)

---

## 4. TODO Comments and Potential Issues

### TODOs Found (4 instances)

#### 1. **Multi-Provider Model Support** (line 173)
```rust
// TODO: Add support for other providers (openai, google, openrouter)
```
**Location**: Model router initialization in `new_internal()`
**Context**: Currently only supports Anthropic models for single model override
**Impact**: Users cannot use single model override with OpenAI/Google/OpenRouter
**Priority**: Medium - Feature enhancement

#### 2. **User Model Override** (line 372)
```rust
None, // TODO: Support user model override
```
**Location**: Inside `process_message()` method
**Context**: Hardcoded `None` for model override parameter
**Impact**: Users cannot override model on per-message basis
**Priority**: Low - Enhancement for flexibility

#### 3. **Session ID Management** (line 849)
```rust
let session_id = "current_session".to_string(); // TODO: Get from actual session management
```
**Location**: Tool execution tracking in episodic memory
**Context**: Hardcoded session ID instead of using actual session state
**Impact**: Memory tracking doesn't properly correlate with sessions
**Priority**: Medium - Affects memory system accuracy

#### 4. **Task ID Tracking** (line 855)
```rust
task_id: None, // TODO: Track task IDs
```
**Location**: File interaction recording in episodic memory
**Context**: Not tracking task IDs for file operations
**Impact**: Cannot correlate file interactions with specific tasks
**Priority**: Medium - Reduces memory system effectiveness

#### 5. **Line Range Extraction** (line 858)
```rust
line_range: None, // TODO: Extract from edit_file params
```
**Location**: File interaction recording
**Context**: Not capturing which lines were edited
**Impact**: Less precise memory of what was changed
**Priority**: Low - Nice-to-have detail

---

## 5. Architecture Observations

### Strengths
1. **Well-structured hybrid architecture** - Clean separation of concerns across 22 components
2. **Comprehensive feature set** - Integrates 7 major SOTA patterns (Week 7-8)
3. **Thread-safe design** - Extensive use of `Arc` and async locks
4. **Extensibility** - Skills manager enables user customization
5. **Event-driven** - Event bus for loose coupling
6. **Safety features** - Git snapshots, mode enforcement, approval workflows

### Potential Issues
1. **High complexity** - 22 fields, 25+ methods, 18 module dependencies
2. **Memory overhead** - Many `Arc` wrapped components could be heavy
3. **TODO backlog** - 5 TODOs indicate incomplete features
4. **Session management incomplete** - Hardcoded session IDs affect memory system
5. **Single provider limitation** - Model override only works with Anthropic

### Design Patterns
1. **Dependency Injection** - Components passed into constructor
2. **Builder Pattern** - Two constructors (with/without approval)
3. **Arc-Mutex Pattern** - Thread-safe shared state
4. **Strategy Pattern** - Pluggable tools, providers, modes
5. **Event Bus Pattern** - Decoupled communication
6. **Registry Pattern** - Tool registry, agent registry

---

## 6. Integration Points

### External Integrations
1. **ACP Protocol** - Implements `AcpAgent` trait when enabled
2. **LSP Servers** - rust-analyzer, pyright, gopls, typescript-language-server
3. **Git** - Snapshot manager for version control integration
4. **LLM Providers** - OpenAI, Anthropic, Gemini, Ollama (via trait)
5. **Skills System** - User-defined skills from `.aircher/skills/`

### Internal Integrations
1. **Episodic Memory** - DuckDB tracking (lines 849-858)
2. **Event Bus** - File operations, mode changes, snapshots
3. **Dynamic Context** - Intelligent context pruning and management
4. **Tool Execution** - Registry-based tool discovery and execution
5. **Model Routing** - Cost-aware model selection

---

## 7. Testing Implications

### Testable Components
- ✅ Tool execution (via `execute_single_tool`)
- ✅ Mode transitions (Plan ↔ Build)
- ✅ Event emission (via event bus)
- ✅ Model routing (cost-aware selection)
- ✅ Skills discovery and execution

### Testing Challenges
- ⚠️ Complex initialization (22 components)
- ⚠️ Async/thread-safety (many locks)
- ⚠️ External dependencies (LSP, Git, providers)
- ⚠️ State management (conversation, memory)

### Missing Test Coverage
- ❌ Session ID tracking (hardcoded values)
- ❌ Task ID correlation
- ❌ Multi-provider model override
- ❌ Line range extraction from tool params

---

## 8. Recommendations

### High Priority
1. **Implement proper session management** - Replace hardcoded "current_session" string
2. **Add multi-provider support** - Extend model override to OpenAI, Google, OpenRouter
3. **Task ID tracking** - Wire task IDs through tool execution chain

### Medium Priority
1. **Extract line ranges** - Parse edit_file parameters for precise tracking
2. **User model override** - Support per-message model selection
3. **Reduce complexity** - Consider facade pattern to simplify initialization

### Low Priority
1. **Documentation** - Add module-level doc comments
2. **Metrics** - Add telemetry for model routing, mode transitions
3. **Error handling** - More specific error types instead of `anyhow::Result`

---

## Summary

`src/agent/core.rs` is a **well-architected but complex** unified agent implementation that successfully integrates 7 SOTA patterns (event bus, LSP, mode separation, Git snapshots, model routing, specialized agents, research sub-agents).

**Strengths**: Comprehensive features, thread-safe design, extensible via skills
**Weaknesses**: High complexity (22 fields), incomplete session/task tracking, single-provider model override limitation

**Overall Assessment**: Production-ready core with some TODOs that should be addressed for full memory system effectiveness and multi-provider flexibility.

**Lines of Code**: 2,365 lines
**Public API**: 25+ methods
**Dependencies**: 18 internal modules + 3 external crates
**TODOs**: 5 (2 medium priority, 3 low priority)
