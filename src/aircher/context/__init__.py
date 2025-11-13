"""Dynamic context management for intelligent pruning."""

from datetime import datetime, timedelta
from enum import Enum
from typing import Any
from uuid import UUID, uuid4

from langchain_core.messages import AIMessage, BaseMessage, HumanMessage, ToolMessage
from loguru import logger
from pydantic import BaseModel, Field

from ..memory.integration import MemoryIntegration


class ContextItemType(str, Enum):
    """Types of context items."""

    SYSTEM_PROMPT = "system_prompt"
    TASK_STATE = "task_state"
    USER_MESSAGE = "user_message"
    ASSISTANT_RESPONSE = "assistant_response"
    TOOL_RESULT = "tool_result"
    CODE_SNIPPET = "code_snippet"
    KNOWLEDGE_GRAPH_QUERY = "knowledge_graph_query"


class ContextItem(BaseModel):
    """A single item in the context window."""

    id: UUID = Field(default_factory=uuid4)
    timestamp: datetime = Field(default_factory=datetime.now)
    item_type: ContextItemType
    content: BaseMessage | dict[str, Any]
    token_cost: int = 0
    sticky: bool = False  # Never remove if True
    task_id: str | None = None
    dependencies: list[UUID] = Field(default_factory=list)
    relevance_score: float = 1.0  # Explicit relevance (1.0 = default)
    metadata: dict[str, Any] = Field(default_factory=dict)

    class Config:
        """Pydantic config."""

        arbitrary_types_allowed = True


class ContextWindow(BaseModel):
    """Manages dynamic context with intelligent pruning."""

    session_id: str
    current_task_id: str | None = None
    items: list[ContextItem] = Field(default_factory=list)
    token_count: int = 0
    token_limit: int = 120000  # ~80% of 150k context window
    pruning_threshold: float = 0.8  # Prune when at 80% capacity
    pruning_count: int = 0
    memory: MemoryIntegration | None = None

    class Config:
        """Pydantic config."""

        arbitrary_types_allowed = True

    def add_item(
        self,
        item_type: ContextItemType,
        content: BaseMessage | dict[str, Any],
        token_cost: int | None = None,
        sticky: bool = False,
        task_id: str | None = None,
        dependencies: list[UUID] | None = None,
        relevance_score: float = 1.0,
        metadata: dict[str, Any] | None = None,
    ) -> ContextItem:
        """Add an item to the context window."""
        # Estimate token cost if not provided
        if token_cost is None:
            token_cost = self._estimate_tokens(content)

        item = ContextItem(
            item_type=item_type,
            content=content,
            token_cost=token_cost,
            sticky=sticky,
            task_id=task_id or self.current_task_id,
            dependencies=dependencies or [],
            relevance_score=relevance_score,
            metadata=metadata or {},
        )

        self.items.append(item)
        self.token_count += token_cost

        logger.debug(
            f"Added {item_type} to context ({token_cost} tokens, "
            f"total: {self.token_count}/{self.token_limit})"
        )

        # Check if we need to prune
        if self.should_prune():
            logger.info(f"Context at {self._get_usage_percent():.1f}% - triggering pruning")
            self.prune_context()

        return item

    def should_prune(self) -> bool:
        """Check if context should be pruned."""
        return self.token_count >= (self.token_limit * self.pruning_threshold)

    def prune_context(self) -> int:
        """Prune context by removing least relevant items.

        Returns:
            Number of tokens removed
        """
        start_tokens = self.token_count
        start_items = len(self.items)

        if not self.items:
            logger.debug("No items to prune")
            return 0

        # 1. Calculate relevance score for each item
        scored_items: list[tuple[int, ContextItem, float]] = []
        for idx, item in enumerate(self.items):
            if not item.sticky:  # Never remove sticky items
                score = self._calculate_relevance(item)
                scored_items.append((idx, item, score))

        if not scored_items:
            logger.debug("All items are sticky, cannot prune")
            return 0

        # 2. Sort by relevance (lowest first)
        scored_items.sort(key=lambda x: x[2])

        # 3. Remove bottom 30% by token count (not item count)
        target_removal = int(self.token_count * 0.3)
        removed_tokens = 0
        removed_items: list[tuple[int, ContextItem]] = []

        for idx, item, score in scored_items:
            if removed_tokens >= target_removal:
                break

            removed_tokens += item.token_cost
            removed_items.append((idx, item))
            logger.debug(f"Marking for removal: {item.item_type} (score: {score:.2f})")

        # 4. Before removing: summarize to episodic memory
        for _, item in removed_items:
            self._summarize_to_episodic(item)

        # 5. Remove from context (reverse order to preserve indices)
        removed_items.sort(key=lambda x: x[0], reverse=True)
        for idx, _ in removed_items:
            self.items.pop(idx)

        # 6. Update token count
        self.token_count -= removed_tokens
        self.pruning_count += 1

        logger.info(
            f"Pruned {len(removed_items)} items: {start_tokens} → {self.token_count} tokens "
            f"({removed_tokens} removed, {start_items} → {len(self.items)} items)"
        )

        return removed_tokens

    def _calculate_relevance(self, item: ContextItem) -> float:
        """Calculate relevance score for a context item.

        Factors:
        1. Time decay (older = less relevant)
        2. Task association (current task = more relevant)
        3. Dependencies (referenced by others = more relevant)
        4. Item type (some types more important)
        5. Explicit relevance score
        """
        score = 1.0

        # Factor 1: Time decay (exponential, half-life ~1 hour)
        age_minutes = (datetime.now() - item.timestamp).total_seconds() / 60
        time_score = 2 ** (-age_minutes / 60.0)  # Half-life of 60 minutes
        score *= time_score

        # Factor 2: Task association (current task = 2x boost)
        if self.current_task_id and item.task_id == self.current_task_id:
            score *= 2.0

        # Factor 3: Dependencies (items that others depend on)
        dependency_boost = 1.0 + (self._count_dependents(item.id) * 0.2)
        score *= dependency_boost

        # Factor 4: Item type multipliers
        type_multiplier = {
            ContextItemType.SYSTEM_PROMPT: 100.0,  # Never remove (marked sticky)
            ContextItemType.TASK_STATE: 2.0,  # Keep task state
            ContextItemType.USER_MESSAGE: 1.5,  # Keep user intent
            ContextItemType.ASSISTANT_RESPONSE: 1.2,  # Keep responses
            ContextItemType.TOOL_RESULT: 0.8,  # Tool results decay
            ContextItemType.CODE_SNIPPET: 0.7,  # Code snippets decay faster
            ContextItemType.KNOWLEDGE_GRAPH_QUERY: 0.6,  # Can re-query
        }.get(item.item_type, 1.0)
        score *= type_multiplier

        # Factor 5: Explicit relevance score
        score *= item.relevance_score

        # Clamp to reasonable range
        return max(0.0, min(100.0, score))

    def _count_dependents(self, item_id: UUID) -> int:
        """Count how many items depend on this item."""
        return sum(1 for item in self.items if item_id in item.dependencies)

    def _summarize_to_episodic(self, item: ContextItem) -> None:
        """Summarize pruned item to episodic memory."""
        if not self.memory:
            return

        try:
            # Record that this context was pruned
            metadata = {
                "item_type": item.item_type.value,
                "timestamp": item.timestamp.isoformat(),
                "task_id": item.task_id,
                "relevance_score": item.relevance_score,
            }

            # Extract file paths if applicable
            file_path = item.metadata.get("file_path")

            if file_path and item.item_type in (
                ContextItemType.CODE_SNIPPET,
                ContextItemType.TOOL_RESULT,
            ):
                # Record file interaction
                logger.debug(f"Recording pruned {item.item_type} for {file_path}")

        except Exception as e:
            logger.warning(f"Failed to summarize to episodic memory: {e}")

    def _estimate_tokens(self, content: BaseMessage | dict[str, Any]) -> int:
        """Estimate token count for content.

        Rough estimate: 1 token ≈ 4 characters for English text.
        """
        if isinstance(content, BaseMessage):
            text = str(content.content)
        elif isinstance(content, dict):
            text = str(content)
        else:
            text = str(content)

        # Rough estimate: 4 chars per token
        return len(text) // 4

    def _get_usage_percent(self) -> float:
        """Get current token usage as percentage."""
        return (self.token_count / self.token_limit) * 100

    def get_messages(self) -> list[BaseMessage]:
        """Extract LangChain messages from context items."""
        messages = []
        for item in self.items:
            if isinstance(item.content, BaseMessage):
                messages.append(item.content)
        return messages

    def clear(self) -> None:
        """Clear all context items."""
        self.items.clear()
        self.token_count = 0
        logger.info("Context window cleared")
