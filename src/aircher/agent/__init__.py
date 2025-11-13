"""Core agent implementation using LangGraph."""

from datetime import datetime
from pathlib import Path
from typing import Any, Optional

from langchain_core.messages import AIMessage, BaseMessage, HumanMessage, SystemMessage
from langchain_core.runnables import RunnableConfig
from langchain_openai import ChatOpenAI
from langgraph.graph import END, START, StateGraph
from langgraph.graph.message import add_messages
from langgraph.prebuilt import ToolNode
from loguru import logger
from pydantic import BaseModel

from ..config import get_settings
from ..context import ContextItemType, ContextWindow
from ..memory.integration import MemoryIntegration, create_memory_system
from ..models import ModelRouter
from ..modes import AgentMode, get_mode_capabilities
from ..tools import BashTool, ListDirectoryTool, ReadFileTool, SearchFilesTool, WriteFileTool
from ..tools.manager import ToolManager


class AgentState(BaseModel):
    """State for the LangGraph agent."""

    messages: list[BaseMessage] = []
    current_mode: AgentMode = AgentMode.READ
    context: dict[str, Any] = {}
    tools_available: list[str] = []
    metadata: dict[str, Any] = {}
    session_id: str = ""
    user_request: str = ""
    intent: str = ""
    tool_calls: list[dict[str, Any]] = []
    response: str = ""


class AircherAgent:
    """Main agent implementation using LangGraph."""

    def __init__(
        self,
        model_name: str = "gpt-4o-mini",
        data_dir: Path | None = None,
        enable_memory: bool = True,
    ) -> None:
        self.settings = get_settings()
        self.model_name = model_name
        self.enable_memory = enable_memory

        # Set up data directory
        if data_dir is None:
            data_dir = Path.home() / ".aircher" / "data"
        self.data_dir = data_dir
        self.data_dir.mkdir(parents=True, exist_ok=True)

        # Initialize memory systems
        if self.enable_memory:
            db_path = self.data_dir / "episodic.duckdb"
            vector_dir = self.data_dir / "vectors"
            self.memory: MemoryIntegration | None = create_memory_system(
                db_path=db_path, vector_persist_dir=vector_dir
            )
            logger.info("Memory systems initialized")
        else:
            self.memory = None
            logger.info("Memory systems disabled")

        # Initialize context window for dynamic context management
        self.context_window: ContextWindow | None = None  # Created per-session

        # Initialize model router for smart model selection and cost tracking
        self.model_router: ModelRouter | None = None  # Created per-session

        # Initialize tool manager for bundled tools
        self.tool_manager = ToolManager()

        # Initialize tools
        self.tools: list[Any] = self._load_tools()
        logger.info(f"Loaded {len(self.tools)} tools")

        # Initialize LLM (will be replaced with model router in run())
        try:
            self.llm = ChatOpenAI(
                model=self.model_name,
                temperature=0.7,
            )
            logger.info(f"Initialized LLM: {self.model_name}")
        except Exception as e:
            logger.warning(f"Failed to initialize LLM: {e}. Will use fallback.")
            self.llm = None

        # Initialize LangGraph workflow
        self.graph = self._build_graph()

    def _load_tools(self) -> list[Any]:
        """Load available tools based on configuration."""
        tools = []

        # Command execution tool (needed by SearchFilesTool)
        bash_tool = BashTool(self.tool_manager)
        tools.append(bash_tool)

        # File operation tools
        tools.append(ReadFileTool())
        tools.append(WriteFileTool())
        tools.append(ListDirectoryTool())
        tools.append(SearchFilesTool(bash_tool))

        return tools

    def _build_graph(self) -> Any:
        """Build the LangGraph workflow with conditional edges."""
        workflow = StateGraph(AgentState)

        # Add nodes
        workflow.add_node("classify_intent", self._classify_intent)
        workflow.add_node("validate_permissions", self._validate_permissions)
        workflow.add_node("select_tools", self._select_tools)
        workflow.add_node("execute_task", self._execute_task)
        workflow.add_node("generate_response", self._generate_response)
        workflow.add_node("update_memory", self._update_memory)
        workflow.add_node("handle_error", self._handle_error)

        # Add edges with conditional routing
        workflow.add_edge(START, "classify_intent")
        workflow.add_edge("classify_intent", "validate_permissions")

        # Conditional: Skip to response if permissions denied
        workflow.add_conditional_edges(
            "validate_permissions",
            self._route_after_permissions,
            {
                "continue": "select_tools",
                "denied": "generate_response",
            }
        )

        # Conditional: Skip execution if no tools selected
        workflow.add_conditional_edges(
            "select_tools",
            self._route_after_tool_selection,
            {
                "execute": "execute_task",
                "skip": "generate_response",
            }
        )

        # Conditional: Handle errors or continue to response
        workflow.add_conditional_edges(
            "execute_task",
            self._route_after_execution,
            {
                "success": "generate_response",
                "error": "handle_error",
                "partial": "generate_response",
            }
        )

        workflow.add_edge("handle_error", "generate_response")
        workflow.add_edge("generate_response", "update_memory")
        workflow.add_edge("update_memory", END)

        return workflow.compile()

    def _route_after_permissions(self, state: AgentState) -> str:
        """Route based on permission validation result."""
        if state.context.get("permissions_validated"):
            return "continue"
        else:
            return "denied"

    def _route_after_tool_selection(self, state: AgentState) -> str:
        """Route based on whether tools were selected."""
        if state.tool_calls and len(state.tool_calls) > 0:
            return "execute"
        else:
            logger.info("No tools selected, skipping execution")
            return "skip"

    def _route_after_execution(self, state: AgentState) -> str:
        """Route based on execution results."""
        tool_results = state.metadata.get("tool_results", [])

        if not tool_results:
            return "success"

        # Check for errors
        errors = [r for r in tool_results if "error" in r]
        successes = [r for r in tool_results if "result" in r]

        if errors and not successes:
            return "error"
        elif errors and successes:
            return "partial"
        else:
            return "success"

    async def _handle_error(self, state: AgentState) -> AgentState:
        """Handle errors from tool execution."""
        tool_results = state.metadata.get("tool_results", [])
        errors = [r for r in tool_results if "error" in r]

        logger.error(f"Handling {len(errors)} tool execution errors")

        # Store error context
        state.context["has_errors"] = True
        state.context["error_count"] = len(errors)
        state.context["errors"] = [
            {
                "tool": r["tool"],
                "error": r["error"],
                "parameters": r.get("parameters", {})
            }
            for r in errors
        ]

        # Could implement retry logic here in the future
        # For now, just log and continue to response generation

        return state

    async def _classify_intent(self, state: AgentState) -> AgentState:
        """Classify user intent from messages with memory context."""
        user_request = state.user_request or (
            state.messages[-1].content if state.messages else ""
        )

        # Simple intent classification (can be enhanced with ML)
        intent = self._classify_intent_simple(str(user_request))

        # Enhance with memory context if available
        if self.memory:
            try:
                # Check if we've seen similar requests before
                # This would use vector search in production
                tool_stats = self.memory.get_tool_statistics(days=7)
                state.context["recent_tool_usage"] = tool_stats

                # Store common patterns for context
                if tool_stats:
                    most_used_tools = [
                        stat["tool_name"]
                        for stat in sorted(
                            tool_stats, key=lambda x: x["total_calls"], reverse=True
                        )[:3]
                    ]
                    state.context["most_used_tools"] = most_used_tools
                    logger.debug(f"Most used tools: {most_used_tools}")

            except Exception as e:
                logger.warning(f"Failed to query memory for intent classification: {e}")

        logger.info(f"Classified intent: {intent} for session {state.session_id}")

        state.intent = intent
        state.context["intent"] = intent
        return state

    def _classify_intent_simple(self, request: str) -> str:
        """Simple rule-based intent classification."""
        request_lower = request.lower()

        if any(
            word in request_lower
            for word in ["read", "show", "list", "display", "what", "how"]
        ):
            return "read"
        elif any(
            word in request_lower
            for word in ["write", "create", "modify", "change", "update", "add"]
        ):
            return "write"
        elif any(
            word in request_lower for word in ["search", "find", "look for", "grep"]
        ):
            return "search"
        elif any(word in request_lower for word in ["delete", "remove", "clean"]):
            return "delete"
        elif any(word in request_lower for word in ["run", "execute", "build", "test"]):
            return "execute"
        else:
            return "general"

    async def _validate_permissions(self, state: AgentState) -> AgentState:
        """Validate permissions for the requested operation."""
        intent = state.intent
        mode = state.current_mode

        # Get mode capabilities
        capabilities = get_mode_capabilities(mode)

        # Check if intent requires elevated permissions
        required_capability = self._get_required_capability(intent)

        # Validate permissions
        if not self._has_capability(capabilities, required_capability):
            logger.warning(f"Permission denied for intent {intent} in mode {mode}")
            state.response = (
                f"Permission denied: {intent} operations require {required_capability}"
            )
            return state

        logger.info(f"Permissions validated for intent {intent} in mode {mode}")
        state.context["permissions_validated"] = True
        return state

    def _get_required_capability(self, intent: str) -> str:
        """Get the required capability for an intent."""
        if intent in ["read", "search", "general"]:
            return "can_read_files"
        elif intent in ["write", "execute"]:
            return "can_write_files"
        elif intent in ["delete"]:
            return "can_execute_commands"
        else:
            return "can_read_files"

    def _has_capability(self, capabilities: Any, required_capability: str) -> bool:
        """Check if the mode has the required capability."""
        return getattr(capabilities, required_capability, False)

    async def _select_tools(self, state: AgentState) -> AgentState:
        """Select appropriate tools based on intent, mode, and memory."""
        intent = state.intent
        user_request = state.user_request

        # Get mode capabilities
        capabilities = get_mode_capabilities(state.current_mode)

        # Query memory for context to inform tool selection
        memory_context = {}
        if self.memory:
            try:
                # Extract file paths from request
                files_mentioned = self._extract_file_paths(user_request)

                for file_path in files_mentioned:
                    # Get file history from episodic memory
                    history = self.memory.query_file_history(file_path, limit=3)
                    if history:
                        memory_context[file_path] = {
                            "last_operation": history[0]["operation"],
                            "recent_changes": len(history),
                        }

                # Find co-edit patterns (files frequently edited together)
                co_edit_patterns = self.memory.find_co_edit_patterns(min_count=2)
                if co_edit_patterns and files_mentioned:
                    # Suggest related files based on co-edit patterns
                    related_files = set()
                    for pattern in co_edit_patterns:
                        if pattern["file1"] in files_mentioned:
                            related_files.add(pattern["file2"])
                        elif pattern["file2"] in files_mentioned:
                            related_files.add(pattern["file1"])

                    if related_files:
                        state.context["suggested_files"] = list(related_files)
                        logger.info(
                            f"Memory suggests also checking: {related_files}"
                        )

                state.context["memory_context"] = memory_context

            except Exception as e:
                logger.warning(f"Failed to query memory for tool selection: {e}")

        # Generate tool call plan based on intent
        tool_calls = self._generate_tool_plan(intent, user_request, capabilities)

        logger.info(f"Generated tool plan: {len(tool_calls)} tool calls")

        state.tool_calls = tool_calls
        state.tools_available = [tool.name for tool in self.tools]
        state.context["tool_plan"] = tool_calls
        return state

    def _extract_file_paths(self, text: str) -> list[str]:
        """Extract file paths from text using simple heuristics."""
        import re

        # Match common file path patterns
        patterns = [
            r"[\w/\.\-]+\.(py|rs|js|ts|md|json|yaml|yml|toml|txt)",  # Files with extensions
            r"/[\w/\.\-]+",  # Absolute paths
            r"[\w/\.\-]+/[\w/\.\-]+",  # Relative paths with subdirectories
        ]

        file_paths = set()
        for pattern in patterns:
            matches = re.findall(pattern, text)
            file_paths.update(matches)

        return list(file_paths)

    def _generate_tool_plan(
        self, intent: str, request: str, capabilities: Any
    ) -> list[dict[str, Any]]:
        """Generate a plan of tool calls based on intent using LLM."""
        tool_calls: list[dict[str, Any]] = []

        # If no LLM available, fall back to simple rule-based planning
        if not self.llm:
            return self._generate_tool_plan_fallback(intent, request, capabilities)

        # Get available tools based on capabilities
        available_tools = self._get_available_tools_for_capabilities(capabilities)

        if not available_tools:
            logger.warning("No tools available for current capabilities")
            return tool_calls

        # Create tool planning prompt
        tool_descriptions = "\n".join([
            f"- {tool.name}: {tool.description}"
            for tool in available_tools
        ])

        planning_prompt = f"""You are an intelligent agent planner. Given a user request, determine which tools to use and in what order.

Available Tools:
{tool_descriptions}

User Intent: {intent}
User Request: {request}

Respond with a JSON array of tool calls. Each tool call should have:
- "tool": the tool name
- "parameters": a dict of parameters for the tool
- "reasoning": why this tool is needed

Example response:
[
  {{
    "tool": "read_file",
    "parameters": {{"path": "src/main.py"}},
    "reasoning": "Need to read the file to understand its contents"
  }}
]

If no tools are needed, respond with an empty array: []

Tool plan:"""

        try:
            # Use LLM to plan tool calls
            response = self.llm.invoke(planning_prompt)
            response_text = response.content

            # Extract JSON from response (handle markdown code blocks)
            import json as json_module
            import re

            # Try to find JSON in code blocks first
            json_match = re.search(r'```(?:json)?\s*(\[[\s\S]*?\])\s*```', response_text)
            if json_match:
                json_text = json_match.group(1)
            else:
                # Try to find JSON array directly
                json_match = re.search(r'\[[\s\S]*\]', response_text)
                json_text = json_match.group(0) if json_match else "[]"

            tool_calls = json_module.loads(json_text)
            logger.info(f"LLM generated {len(tool_calls)} tool calls")

        except Exception as e:
            logger.error(f"Failed to generate tool plan with LLM: {e}")
            # Fallback to rule-based planning
            tool_calls = self._generate_tool_plan_fallback(intent, request, capabilities)

        return tool_calls

    def _generate_tool_plan_fallback(
        self, intent: str, request: str, capabilities: Any
    ) -> list[dict[str, Any]]:
        """Fallback rule-based tool planning when LLM is unavailable."""
        tool_calls: list[dict[str, Any]] = []

        # Simple rule-based planning
        if intent == "read" and capabilities.can_read_files:
            # Extract file paths from request
            file_paths = self._extract_file_paths(request)
            for file_path in file_paths[:1]:  # Limit to first file for now
                tool_calls.append({
                    "tool": "read_file",
                    "parameters": {"path": file_path},
                    "reasoning": "User requested to read file"
                })
        elif intent == "write" and capabilities.can_write_files:
            file_paths = self._extract_file_paths(request)
            for file_path in file_paths[:1]:
                tool_calls.append({
                    "tool": "write_file",
                    "parameters": {"path": file_path, "content": ""},
                    "reasoning": "User requested to write file"
                })
        elif intent == "search" and capabilities.can_read_files:
            tool_calls.append({
                "tool": "search_files",
                "parameters": {"pattern": "", "path": "."},
                "reasoning": "User requested to search"
            })

        return tool_calls

    def _get_available_tools_for_capabilities(self, capabilities: Any) -> list[Any]:
        """Filter tools based on mode capabilities."""
        available_tools = []

        for tool in self.tools:
            # Map tool names to required capabilities
            if tool.name in ["read_file", "list_directory", "search_files"]:
                if capabilities.can_read_files:
                    available_tools.append(tool)
            elif tool.name in ["write_file"]:
                if capabilities.can_write_files:
                    available_tools.append(tool)
            elif tool.name in ["bash"]:
                if capabilities.can_execute_commands:
                    available_tools.append(tool)

        return available_tools

    async def _execute_task(self, state: AgentState) -> AgentState:
        """Execute the task using selected tools."""
        tool_calls = state.tool_calls
        results = []

        # Find tools by name
        tool_map = {tool.name: tool for tool in self.tools}

        for tool_call in tool_calls:
            tool_name = tool_call["tool"]
            parameters = tool_call["parameters"]

            start_time = datetime.now()

            try:
                # Find the tool
                tool = tool_map.get(tool_name)
                if not tool:
                    logger.warning(f"Tool {tool_name} not found")
                    results.append(
                        {
                            "tool": tool_name,
                            "parameters": parameters,
                            "error": f"Tool {tool_name} not found",
                        }
                    )
                    continue

                # Execute the tool
                logger.info(f"Executing tool {tool_name} with parameters {parameters}")

                # Wrap memory tracking if enabled
                if self.memory:
                    tool_func = self.memory.track_tool_execution(tool.execute)
                else:
                    tool_func = tool.execute

                # Execute (handle both sync and async)
                if hasattr(tool.execute, "__call__"):
                    result = await tool_func(**parameters)
                else:
                    result = await tool_func(**parameters)

                duration_ms = int((datetime.now() - start_time).total_seconds() * 1000)

                results.append(
                    {
                        "tool": tool_name,
                        "parameters": parameters,
                        "result": result,
                        "duration_ms": duration_ms,
                    }
                )

                logger.info(f"Tool {tool_name} completed in {duration_ms}ms")

            except Exception as e:
                duration_ms = int((datetime.now() - start_time).total_seconds() * 1000)
                logger.error(f"Tool {tool_name} failed: {e}")

                results.append(
                    {
                        "tool": tool_name,
                        "parameters": parameters,
                        "error": str(e),
                        "duration_ms": duration_ms,
                    }
                )

        # Update state with results
        state.metadata["tool_results"] = results
        state.context["execution_complete"] = True
        return state

    async def _generate_response(self, state: AgentState) -> AgentState:
        """Generate final response to the user using LLM."""
        intent = state.intent
        user_request = state.user_request
        tool_results = state.metadata.get("tool_results", [])

        # If no LLM, use template-based response
        if not self.llm:
            response = self._generate_response_fallback(intent, tool_results)
            state.response = response
            state.context["response_generated"] = True
            return state

        # Build context for LLM
        context_parts = [
            f"User Request: {user_request}",
            f"Intent: {intent}",
        ]

        # Add memory context if available
        if state.context.get("suggested_files"):
            context_parts.append(
                f"Related Files (from memory): {', '.join(state.context['suggested_files'])}"
            )

        # Add tool execution results
        if tool_results:
            results_summary = "\n".join([
                f"- {result.get('tool', 'unknown')}: "
                f"{'Success' if 'result' in result else 'Failed'} "
                f"({result.get('duration_ms', 0)}ms)"
                for result in tool_results
            ])
            context_parts.append(f"\nTool Executions:\n{results_summary}")

            # Add detailed results
            for result in tool_results:
                if "result" in result:
                    context_parts.append(
                        f"\n{result['tool']} result:\n{result['result']}"
                    )
                elif "error" in result:
                    context_parts.append(
                        f"\n{result['tool']} error:\n{result['error']}"
                    )

        context = "\n".join(context_parts)

        # Create response generation prompt
        response_prompt = f"""You are Aircher, an intelligent coding agent. Generate a helpful, natural response to the user based on the context below.

{context}

Respond directly to the user. Be concise but informative. If tools were executed, explain what was done and what was found. If there were errors, explain them clearly. If you suggested related files from memory, mention them helpfully.

Response:"""

        try:
            # Generate response with LLM
            response = self.llm.invoke(response_prompt)
            response_text = response.content

            logger.info("Generated LLM response")
            state.response = response_text

        except Exception as e:
            logger.error(f"Failed to generate response with LLM: {e}")
            # Fallback to template response
            response = self._generate_response_fallback(intent, tool_results)
            state.response = response

        # Add assistant response to context window
        if self.context_window:
            assistant_msg = AIMessage(content=state.response)
            self.context_window.add_item(
                item_type=ContextItemType.ASSISTANT_RESPONSE,
                content=assistant_msg,
            )
            logger.debug(
                f"Added assistant response to context window "
                f"({self.context_window.token_count}/{self.context_window.token_limit} tokens)"
            )

        state.context["response_generated"] = True
        return state

    def _generate_response_fallback(
        self, intent: str, tool_results: list[dict[str, Any]]
    ) -> str:
        """Fallback template-based response generation."""
        if tool_results:
            return self._format_tool_results(tool_results)
        else:
            return f"I understand you want to {intent}. Ready to assist when tools are executed."

    def _format_tool_results(self, tool_results: list[dict[str, Any]]) -> str:
        """Format tool results into a human-readable response."""
        if not tool_results:
            return "No results to display."

        responses = []
        for result in tool_results:
            tool_name = result["tool"]
            if "result" in result:
                responses.append(f"{tool_name}: {result['result']}")
            elif "error" in result:
                responses.append(f"Error with {tool_name}: {result['error']}")

        return "\n\n".join(responses)

    async def _update_memory(self, state: AgentState) -> AgentState:
        """Update memory systems with interaction."""
        if not self.memory:
            state.context["memory_updated"] = False
            return state

        try:
            # Set context for memory tracking
            self.memory.set_context(
                session_id=state.session_id,
                task_id=state.context.get("task_id"),
            )

            # Record tool executions from metadata
            tool_results = state.metadata.get("tool_results", [])
            for result in tool_results:
                tool_name = result.get("tool")
                parameters = result.get("parameters", {})
                success = "error" not in result
                error_msg = result.get("error") if not success else None
                duration_ms = result.get("duration_ms")

                self.memory.episodic.record_tool_execution(
                    session_id=state.session_id,
                    tool_name=tool_name,
                    parameters=parameters,
                    result=result.get("result"),
                    success=success,
                    error_message=error_msg,
                    duration_ms=duration_ms,
                )

            # Save context snapshot
            context_items = [
                {
                    "type": "message",
                    "content": state.user_request,
                    "role": "user",
                },
                {
                    "type": "intent",
                    "intent": state.intent,
                },
                {
                    "type": "tool_results",
                    "count": len(tool_results),
                },
            ]

            self.memory.snapshot_context(
                context_items=context_items,
                total_tokens=0,  # Will be calculated when LLM is integrated
                reason="task_completion",
            )

            logger.info(
                f"Updated memory: {len(tool_results)} tool executions recorded"
            )
            state.context["memory_updated"] = True

        except Exception as e:
            logger.error(f"Failed to update memory: {e}")
            state.context["memory_updated"] = False
            state.context["memory_error"] = str(e)

        return state

    async def run(
        self,
        message: str,
        mode: AgentMode = AgentMode.READ,
        session_id: str | None = None,
    ) -> dict[str, Any]:
        """Run the agent with a message."""
        if session_id is None:
            session_id = f"session_{datetime.now().strftime('%Y%m%d_%H%M%S')}"

        # Initialize model router for this session
        self.model_router = ModelRouter(
            default_model=self.model_name,
            enable_fallback=True,
            session_id=session_id,
        )
        logger.info(f"Initialized model router for session {session_id}")

        # Get the LLM for main agent (use task-based routing)
        try:
            self.llm = self.model_router.get_model(
                model_name=self.model_router.select_model_for_task("main_agent"),
                temperature=0.7,
            )
            logger.info(f"Using model: {self.model_name} for main agent")
        except Exception as e:
            logger.warning(f"Failed to get model from router: {e}")
            # Fallback to existing self.llm if available
            if not self.llm:
                raise

        # Initialize context window for this session
        self.context_window = ContextWindow(
            session_id=session_id,
            memory=self.memory,
        )
        logger.info(f"Initialized context window for session {session_id}")

        # Add system prompt as sticky item
        system_prompt = SystemMessage(
            content="You are Aircher, an intelligent coding assistant with memory capabilities."
        )
        self.context_window.add_item(
            item_type=ContextItemType.SYSTEM_PROMPT,
            content=system_prompt,
            sticky=True,  # Never remove system prompt
        )

        # Add user message
        user_msg = HumanMessage(content=message)
        self.context_window.add_item(
            item_type=ContextItemType.USER_MESSAGE,
            content=user_msg,
        )

        initial_state = AgentState(
            current_mode=mode,
            messages=[system_prompt, user_msg],
            session_id=session_id,
            user_request=message,
        )

        config = RunnableConfig(
            configurable={"thread_id": session_id}, recursion_limit=50
        )

        result = await self.graph.ainvoke(initial_state, config=config)

        # Get cost summary after execution
        if self.model_router:
            cost_summary = self.model_router.get_cost_summary()
            logger.info(
                f"Session cost: ${cost_summary['total_cost']:.4f} "
                f"({cost_summary['total_tokens']} tokens, {cost_summary['call_count']} calls)"
            )
            # Add cost info to result
            if isinstance(result, dict):
                result["cost_summary"] = cost_summary
            elif hasattr(result, "metadata"):
                result.metadata["cost_summary"] = cost_summary

        # Handle both dict and AgentState return types
        if hasattr(result, "model_dump"):
            return result.model_dump()
        else:
            return result

    def get_available_tools(self) -> list[dict[str, Any]]:
        """Get list of available tools."""
        return [
            {
                "name": tool.name,
                "description": tool.description,
                "schema": tool.get_input_schema().parameters,
            }
            for tool in self.tools
        ]

    async def spawn_sub_agent(
        self,
        agent_type: str,
        task: str,
        session_id: str | None = None,
        model_name: str = "gpt-4o-mini",  # Cheaper model for sub-agents
    ) -> dict[str, Any]:
        """Spawn a specialized sub-agent to handle a specific task.

        Args:
            agent_type: Type of agent ("code_reading", "code_writing", "project_fixing")
            task: The task for the sub-agent to perform
            session_id: Optional session ID (will be auto-generated if not provided)
            model_name: LLM model to use (defaults to cheaper model)

        Returns:
            dict with sub-agent results including response and tool executions
        """
        from .sub_agents import create_sub_agent

        # Create sub-agent with access to memory
        sub_agent = create_sub_agent(
            agent_type=agent_type,
            model_name=model_name,
            memory=self.memory,
            parent_session_id=session_id or "main",
        )

        logger.info(f"Spawned {agent_type} sub-agent for task: {task[:50]}...")

        # Run the sub-agent
        result = await sub_agent.run(task=task, session_id=session_id)

        logger.info(f"{agent_type} sub-agent completed with response: {result.get('response', 'N/A')[:100]}...")

        return result
