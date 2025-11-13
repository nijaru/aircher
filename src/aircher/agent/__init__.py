"""Core agent implementation using LangGraph."""

from datetime import datetime
from typing import Any, Optional

from langchain_core.messages import AIMessage, BaseMessage, HumanMessage, SystemMessage
from langchain_core.runnables import RunnableConfig
from langgraph.graph import END, START, StateGraph
from langgraph.graph.message import add_messages
from langgraph.prebuilt import ToolNode
from loguru import logger
from pydantic import BaseModel

from ..config import get_settings
from ..modes import AgentMode, get_mode_capabilities


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

    def __init__(self, model_name: str = "gpt-4o-mini") -> None:
        self.settings = get_settings()
        self.model_name = model_name

        # Initialize tools (will be added later)
        self.tools: list[Any] = []

        # Initialize LangGraph workflow
        self.graph = self._build_graph()

    def _build_graph(self) -> Any:
        """Build the LangGraph workflow."""
        workflow = StateGraph(AgentState)

        # Add nodes
        workflow.add_node("classify_intent", self._classify_intent)
        workflow.add_node("validate_permissions", self._validate_permissions)
        workflow.add_node("select_tools", self._select_tools)
        workflow.add_node("execute_task", self._execute_task)
        workflow.add_node("generate_response", self._generate_response)
        workflow.add_node("update_memory", self._update_memory)

        # Add edges
        workflow.add_edge(START, "classify_intent")
        workflow.add_edge("classify_intent", "validate_permissions")
        workflow.add_edge("validate_permissions", "select_tools")
        workflow.add_edge("select_tools", "execute_task")
        workflow.add_edge("execute_task", "generate_response")
        workflow.add_edge("generate_response", "update_memory")
        workflow.add_edge("update_memory", END)

        return workflow.compile()

    async def _classify_intent(self, state: AgentState) -> AgentState:
        """Classify user intent from messages."""
        user_request = state.user_request or (
            state.messages[-1].content if state.messages else ""
        )

        # Simple intent classification (can be enhanced with ML)
        intent = self._classify_intent_simple(str(user_request))

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
        """Select appropriate tools based on intent and mode."""
        intent = state.intent
        user_request = state.user_request

        # Get mode capabilities
        capabilities = get_mode_capabilities(state.current_mode)

        # Generate tool call plan based on intent
        tool_calls = self._generate_tool_plan(intent, user_request, capabilities)

        logger.info(f"Generated tool plan: {len(tool_calls)} tool calls")

        state.tool_calls = tool_calls
        state.tools_available = [tool.name for tool in self.tools]
        state.context["tool_plan"] = tool_calls
        return state

    def _generate_tool_plan(
        self, intent: str, request: str, capabilities: Any
    ) -> list[dict[str, Any]]:
        """Generate a plan of tool calls based on intent."""
        tool_calls: list[dict[str, Any]] = []

        # For now, just return empty tool calls until tools are fully implemented
        if intent == "read" and capabilities.can_read_files:
            # Will add read_file tool call later
            pass
        elif intent == "write" and capabilities.can_write_files:
            # Will add write_file tool call later
            pass

        return tool_calls

    async def _execute_task(self, state: AgentState) -> AgentState:
        """Execute the task using selected tools."""
        tool_calls = state.tool_calls

        results = []

        for tool_call in tool_calls:
            tool_name = tool_call["tool"]
            parameters = tool_call["parameters"]

            # For now, just log the tool call
            logger.info(f"Would execute tool {tool_name} with parameters {parameters}")
            results.append(
                {
                    "tool": tool_name,
                    "parameters": parameters,
                    "result": f"Tool {tool_name} execution simulated",
                }
            )

        # Update state with results
        state.metadata["tool_results"] = results
        state.context["execution_complete"] = True
        return state

    async def _generate_response(self, state: AgentState) -> AgentState:
        """Generate final response to the user."""
        intent = state.intent
        tool_results = state.metadata.get("tool_results", [])

        # Generate response based on tool results
        if tool_results:
            response = self._format_tool_results(tool_results)
        else:
            response = f"I understand you want to {intent}. This is a simulated response - tools will be fully implemented soon."

        state.response = response
        state.context["response_generated"] = True
        return state

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
        # TODO: Implement memory updates
        state.context["memory_updated"] = True
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

        initial_state = AgentState(
            current_mode=mode,
            messages=[HumanMessage(content=message)],
            session_id=session_id,
            user_request=message,
        )

        config = RunnableConfig(
            configurable={"thread_id": session_id}, recursion_limit=50
        )

        result = await self.graph.ainvoke(initial_state, config=config)
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
