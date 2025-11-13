"""Base tool definitions for Aircher."""

from abc import ABC, abstractmethod
from typing import Any

from pydantic import BaseModel

# Import will be available after module initialization
# from .manager import get_tool_manager, get_tool
# from .bash import BashTool, BashResult


class ToolInput(BaseModel):
    """Input schema for tools."""

    name: str
    description: str
    parameters: dict[str, Any]


class ToolOutput(BaseModel):
    """Output schema for tools."""

    success: bool
    data: Any | None = None
    error: str | None = None
    metadata: dict[str, Any] | None = None


class BaseTool(ABC):
    """Base class for all Aircher tools."""

    def __init__(self, name: str, description: str):
        self.name = name
        self.description = description

    @abstractmethod
    async def execute(self, **kwargs: Any) -> ToolOutput:
        """Execute the tool with given parameters."""
        pass

    @abstractmethod
    def get_input_schema(self) -> ToolInput:
        """Get the input schema for this tool."""
        pass

    def __repr__(self) -> str:
        return f"{self.__class__.__name__}(name='{self.name}')"
