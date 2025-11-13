"""Sub-agent implementations for specialized tasks."""

from abc import ABC, abstractmethod
from datetime import datetime
from pathlib import Path
from typing import Any

from langchain_core.messages import AIMessage, BaseMessage, HumanMessage
from langchain_openai import ChatOpenAI
from langgraph.graph import END, START, StateGraph
from loguru import logger
from pydantic import BaseModel

from ..memory.integration import MemoryIntegration
from ..models import ModelRouter
from ..modes import AgentMode, get_mode_capabilities
from ..tools import BashTool, ListDirectoryTool, ReadFileTool, SearchFilesTool, WriteFileTool
from ..tools.manager import ToolManager


class SubAgentState(BaseModel):
    """State for sub-agents."""

    messages: list[BaseMessage] = []
    task: str = ""
    context: dict[str, Any] = {}
    tool_calls: list[dict[str, Any]] = []
    results: list[dict[str, Any]] = []
    response: str = ""
    session_id: str = ""
    parent_session_id: str = ""


class BaseSubAgent(ABC):
    """Base class for specialized sub-agents."""

    def __init__(
        self,
        model_name: str = "gpt-4o-mini",  # Default to cheaper model
        memory: MemoryIntegration | None = None,
        parent_session_id: str | None = None,
    ):
        self.model_name = model_name
        self.memory = memory
        self.parent_session_id = parent_session_id or "root"
        self.agent_type = self.__class__.__name__

        # Initialize model router for cost tracking (will be set per-run)
        self.model_router: ModelRouter | None = None
        self.llm = None  # Will be set when model_router is initialized

        # Initialize tool manager for bundled tools
        self.tool_manager = ToolManager()

        # Load allowed tools
        self.tools = self._load_tools()
        logger.info(f"{self.agent_type} loaded {len(self.tools)} tools")

        # Build workflow
        self.graph = self._build_graph()

    @abstractmethod
    def _load_tools(self) -> list[Any]:
        """Load tools specific to this sub-agent type."""
        pass

    @abstractmethod
    def get_system_prompt(self) -> str:
        """Get the system prompt for this sub-agent."""
        pass

    def _build_graph(self) -> Any:
        """Build the sub-agent workflow graph."""
        workflow = StateGraph(SubAgentState)

        # Sub-agents have a simpler workflow
        workflow.add_node("plan", self._plan)
        workflow.add_node("execute", self._execute)
        workflow.add_node("respond", self._respond)

        # Linear workflow for sub-agents
        workflow.add_edge(START, "plan")
        workflow.add_edge("plan", "execute")
        workflow.add_edge("execute", "respond")
        workflow.add_edge("respond", END)

        return workflow.compile()

    async def _plan(self, state: SubAgentState) -> SubAgentState:
        """Plan which tools to use for the task."""
        task = state.task

        if not self.llm:
            # Fallback to simple planning
            state.tool_calls = []
            return state

        # Create tool descriptions
        tool_descriptions = "\n".join([
            f"- {tool.name}: {tool.description}"
            for tool in self.tools
        ])

        system_prompt = self.get_system_prompt()

        planning_prompt = f"""{system_prompt}

Available Tools:
{tool_descriptions}

Task: {task}

Plan which tools to use and in what order. Respond with a JSON array:
[
  {{
    "tool": "tool_name",
    "parameters": {{}},
    "reasoning": "why this tool is needed"
  }}
]

If no tools are needed, respond with: []

Tool plan:"""

        try:
            response = self.llm.invoke(planning_prompt)
            response_text = response.content

            # Extract JSON
            import json as json_module
            import re

            json_match = re.search(r'```(?:json)?\s*(\[[\s\S]*?\])\s*```', response_text)
            if json_match:
                json_text = json_match.group(1)
            else:
                json_match = re.search(r'\[[\s\S]*\]', response_text)
                json_text = json_match.group(0) if json_match else "[]"

            state.tool_calls = json_module.loads(json_text)
            logger.info(f"{self.agent_type} planned {len(state.tool_calls)} tool calls")

        except Exception as e:
            logger.error(f"{self.agent_type} planning failed: {e}")
            state.tool_calls = []

        return state

    async def _execute(self, state: SubAgentState) -> SubAgentState:
        """Execute planned tools."""
        tool_map = {tool.name: tool for tool in self.tools}
        results = []

        for tool_call in state.tool_calls:
            tool_name = tool_call["tool"]
            parameters = tool_call["parameters"]

            start_time = datetime.now()

            try:
                tool = tool_map.get(tool_name)
                if not tool:
                    logger.warning(f"{self.agent_type}: Tool {tool_name} not available")
                    results.append({
                        "tool": tool_name,
                        "error": f"Tool {tool_name} not available to {self.agent_type}"
                    })
                    continue

                # Execute tool
                logger.info(f"{self.agent_type} executing {tool_name}")

                # Wrap with memory tracking if available
                if self.memory:
                    tool_func = self.memory.track_tool_execution(tool.execute)
                else:
                    tool_func = tool.execute

                result = await tool_func(**parameters)
                duration_ms = int((datetime.now() - start_time).total_seconds() * 1000)

                results.append({
                    "tool": tool_name,
                    "parameters": parameters,
                    "result": result,
                    "duration_ms": duration_ms,
                })

            except Exception as e:
                duration_ms = int((datetime.now() - start_time).total_seconds() * 1000)
                logger.error(f"{self.agent_type}: {tool_name} failed: {e}")
                results.append({
                    "tool": tool_name,
                    "error": str(e),
                    "duration_ms": duration_ms,
                })

        state.results = results
        return state

    async def _respond(self, state: SubAgentState) -> SubAgentState:
        """Generate response based on execution results."""
        task = state.task
        results = state.results

        if not self.llm:
            # Simple fallback
            state.response = f"{self.agent_type} completed task"
            return state

        # Build context
        results_text = "\n".join([
            f"- {r['tool']}: {'Success' if 'result' in r else 'Failed'}"
            + (f"\n  Result: {r.get('result', 'N/A')}" if 'result' in r else f"\n  Error: {r.get('error', 'Unknown')}")
            for r in results
        ])

        response_prompt = f"""{self.get_system_prompt()}

Task: {task}

Tool Execution Results:
{results_text}

Provide a concise summary of what was accomplished and any findings.

Response:"""

        try:
            response = self.llm.invoke(response_prompt)
            state.response = response.content
        except Exception as e:
            logger.error(f"{self.agent_type} response generation failed: {e}")
            state.response = f"{self.agent_type} completed with {len(results)} tool executions"

        return state

    async def run(
        self,
        task: str,
        session_id: str | None = None,
        context: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        """Run the sub-agent with a task."""
        if session_id is None:
            session_id = f"{self.agent_type}_{datetime.now().strftime('%Y%m%d_%H%M%S')}"

        # Initialize model router for this sub-agent execution
        self.model_router = ModelRouter(
            default_model=self.model_name,
            enable_fallback=True,
            session_id=session_id,
        )
        logger.info(f"{self.agent_type} initialized model router for session {session_id}")

        # Get the LLM for sub-agent (use sub_agent task routing)
        try:
            self.llm = self.model_router.get_model(
                model_name=self.model_router.select_model_for_task("sub_agent"),
                temperature=0.5,  # Lower temp for focused responses
            )
            logger.info(f"{self.agent_type} using model: {self.model_name}")
        except Exception as e:
            logger.warning(f"{self.agent_type} failed to get model from router: {e}")
            self.llm = None

        # Set memory context if available
        if self.memory:
            self.memory.set_context(
                session_id=session_id,
                task_id=f"{self.parent_session_id}::{session_id}",
            )

        initial_state = SubAgentState(
            task=task,
            session_id=session_id,
            parent_session_id=self.parent_session_id,
            context=context or {},
        )

        result = await self.graph.ainvoke(initial_state)

        # Get cost summary after execution
        if self.model_router:
            cost_summary = self.model_router.get_cost_summary()
            logger.info(
                f"{self.agent_type} cost: ${cost_summary['total_cost']:.4f} "
                f"({cost_summary['total_tokens']} tokens, {cost_summary['call_count']} calls)"
            )
            # Add cost info to result
            if isinstance(result, dict):
                result["cost_summary"] = cost_summary
            elif hasattr(result, "context"):
                result.context["cost_summary"] = cost_summary

        # Handle both dict and SubAgentState return types
        if hasattr(result, "model_dump"):
            return result.model_dump()
        else:
            return result


class CodeReadingAgent(BaseSubAgent):
    """Specialized agent for reading and understanding code."""

    def _load_tools(self) -> list[Any]:
        """Load READ mode tools only."""
        # Create bash_tool for SearchFilesTool (but don't expose it)
        bash_tool = BashTool(self.tool_manager)
        return [
            ReadFileTool(),
            ListDirectoryTool(),
            SearchFilesTool(bash_tool),
        ]

    def get_system_prompt(self) -> str:
        """System prompt for code reading."""
        return """You are a CodeReading specialist. Your role is to:
- Read and analyze code files
- Search for patterns and symbols
- Understand code structure and dependencies
- Provide clear explanations of code behavior

You can ONLY read files, not modify them. Focus on understanding and explaining."""


class CodeWritingAgent(BaseSubAgent):
    """Specialized agent for writing and modifying code."""

    def _load_tools(self) -> list[Any]:
        """Load WRITE mode tools."""
        # Create bash_tool for SearchFilesTool (but don't expose it)
        bash_tool = BashTool(self.tool_manager)
        return [
            ReadFileTool(),
            WriteFileTool(),
            ListDirectoryTool(),
            SearchFilesTool(bash_tool),
        ]

    def get_system_prompt(self) -> str:
        """System prompt for code writing."""
        return """You are a CodeWriting specialist. Your role is to:
- Create new code files
- Modify existing code
- Implement features and improvements
- Follow best practices and patterns

You can read and write files. Focus on clean, maintainable implementations."""


class ProjectFixingAgent(BaseSubAgent):
    """Specialized agent for debugging and fixing issues."""

    def _load_tools(self) -> list[Any]:
        """Load full toolset including bash."""
        bash_tool = BashTool(self.tool_manager)
        return [
            bash_tool,
            ReadFileTool(),
            WriteFileTool(),
            ListDirectoryTool(),
            SearchFilesTool(bash_tool),
        ]

    def get_system_prompt(self) -> str:
        """System prompt for project fixing."""
        return """You are a ProjectFixing specialist. Your role is to:
- Diagnose and fix bugs
- Run tests and analyze failures
- Debug runtime issues
- Apply fixes and verify results

You have full access to read, write, and execute commands. Focus on systematic debugging."""


def create_sub_agent(
    agent_type: str,
    model_name: str = "gpt-4o-mini",
    memory: MemoryIntegration | None = None,
    parent_session_id: str | None = None,
) -> BaseSubAgent:
    """Factory function to create sub-agents."""
    agents = {
        "code_reading": CodeReadingAgent,
        "code_writing": CodeWritingAgent,
        "project_fixing": ProjectFixingAgent,
    }

    agent_class = agents.get(agent_type.lower())
    if not agent_class:
        raise ValueError(f"Unknown agent type: {agent_type}. Choose from {list(agents.keys())}")

    return agent_class(
        model_name=model_name,
        memory=memory,
        parent_session_id=parent_session_id,
    )
