"""Terminal-Bench adapter for Aircher agent."""

import sys
from pathlib import Path
from typing import Any

from loguru import logger
from pydantic import BaseModel, Field

# Import Terminal-Bench base classes
try:
    from terminal_bench.agents.base_agent import AgentResult, BaseAgent
    from terminal_bench.agents.failure_mode import FailureMode
    from terminal_bench.terminal.tmux_session import TmuxSession
except ImportError as e:
    logger.error(f"Failed to import Terminal-Bench dependencies: {e}")
    raise

# Import Aircher components
import sys

sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from aircher.models import ModelRouter


class BashCommandResponse(BaseModel):
    """Response model for bash commands."""

    commands: list[str] = Field(
        description="List of shell commands to execute sequentially"
    )
    explanation: str = Field(
        description="Brief explanation of what these commands will accomplish"
    )


class AircherTerminalBenchAdapter(BaseAgent):
    """Terminal-Bench adapter for Aircher agent.

    This adapter uses Aircher's model router for LLM calls but executes
    commands directly in the Terminal-Bench tmux session.
    """

    PROMPT_TEMPLATE = """You are an expert systems engineer. You will be given a task to complete in a Linux terminal environment.

Your job is to provide the exact shell commands needed to complete the task. Return your response as a JSON object with:
- "commands": a list of shell commands to run (in order)
- "explanation": a brief explanation of what the commands do

Task: {instruction}

Think step by step:
1. What needs to be done?
2. What commands are needed?
3. In what order should they run?

Return only valid JSON in the format:
{{
  "commands": ["command1", "command2", ...],
  "explanation": "brief explanation"
}}"""

    @staticmethod
    def name() -> str:
        return "aircher"

    def __init__(
        self,
        model_name: str = "anthropic/claude-sonnet-4.5",
        **kwargs: Any,
    ) -> None:
        super().__init__(**kwargs)
        self.model_name = model_name
        self._logger = logger.getChild(__name__)

        # Initialize Aircher's model router for smart model selection
        try:
            self.model_router = ModelRouter(
                default_model=model_name,
                enable_fallback=True,
                session_id="terminal_bench",
            )
            self._logger.info(f"Initialized Aircher model router with: {model_name}")
        except Exception as e:
            self._logger.error(f"Failed to initialize model router: {e}")
            raise

        self._running_total_input_tokens = 0
        self._running_total_output_tokens = 0

    def perform_task(
        self,
        instruction: str,
        session: TmuxSession,
        logging_dir: Path | None = None,
    ) -> AgentResult:
        """Execute a task using Aircher's infrastructure.

        Args:
            instruction: The task instruction
            session: Tmux session to execute commands in
            logging_dir: Optional directory for logging

        Returns:
            AgentResult with token usage and failure mode
        """
        self._running_total_input_tokens = 0
        self._running_total_output_tokens = 0

        # Render instruction with prompt template if provided
        rendered_instruction = self._render_instruction(instruction)

        # Format prompt
        prompt = self.PROMPT_TEMPLATE.format(instruction=rendered_instruction)

        # Log prompt if logging enabled
        if logging_dir is not None:
            prompt_path = logging_dir / "prompt.txt"
            prompt_path.write_text(prompt)

        try:
            # Get LLM from model router
            llm = self.model_router.get_model(
                model_name=self.model_name,
                temperature=0.3,  # Lower temperature for more deterministic commands
            )

            # Call LLM with structured output
            self._logger.info(f"Calling LLM for task: {instruction[:80]}...")

            # Use LangChain's with_structured_output for Pydantic models
            structured_llm = llm.with_structured_output(BashCommandResponse)
            response = structured_llm.invoke(prompt)

            # Log response if logging enabled
            if logging_dir is not None:
                response_path = logging_dir / "response.json"
                response_path.write_text(response.model_dump_json(indent=2))

            # Track token usage (estimate for now - will get real usage from model_router)
            # OpenRouter and most providers return usage in response metadata
            input_tokens = len(prompt.split()) * 1.3  # Rough estimate
            output_tokens = len(response.model_dump_json().split()) * 1.3

            self._running_total_input_tokens += int(input_tokens)
            self._running_total_output_tokens += int(output_tokens)

            # Execute commands in tmux session
            self._logger.info(f"Executing {len(response.commands)} commands")
            for i, cmd in enumerate(response.commands, 1):
                self._logger.debug(f"Command {i}/{len(response.commands)}: {cmd}")
                session.send_keys([cmd, "Enter"], block=True)

            return AgentResult(
                total_input_tokens=self._running_total_input_tokens,
                total_output_tokens=self._running_total_output_tokens,
                failure_mode=FailureMode.NONE,
            )

        except Exception as e:
            self._logger.error(f"Task execution failed: {e}", exc_info=True)

            # Log error if logging enabled
            if logging_dir is not None:
                error_path = logging_dir / "error.txt"
                error_path.write_text(f"{type(e).__name__}: {str(e)}")

            return AgentResult(
                total_input_tokens=self._running_total_input_tokens,
                total_output_tokens=self._running_total_output_tokens,
                failure_mode=FailureMode.FATAL_LLM_PARSE_ERROR,
            )
