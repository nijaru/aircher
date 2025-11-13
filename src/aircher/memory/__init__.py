"""Memory system interfaces for Aircher."""

from abc import ABC, abstractmethod
from typing import Any

from pydantic import BaseModel


class MemoryEntry(BaseModel):
    """Base class for memory entries."""

    id: str
    timestamp: float
    content: dict[str, Any]
    metadata: dict[str, Any] | None = None


class BaseMemory(ABC):
    """Base class for memory systems."""

    @abstractmethod
    async def store(self, entry: MemoryEntry) -> bool:
        """Store a memory entry."""
        pass

    @abstractmethod
    async def retrieve(self, query: str, limit: int = 10) -> list[MemoryEntry]:
        """Retrieve relevant memory entries."""
        pass

    @abstractmethod
    async def update(self, entry_id: str, updates: dict[str, Any]) -> bool:
        """Update a memory entry."""
        pass

    @abstractmethod
    async def delete(self, entry_id: str) -> bool:
        """Delete a memory entry."""
        pass
