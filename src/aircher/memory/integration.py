"""Integration layer for memory systems with tool tracking."""

import functools
import time
from collections.abc import Callable
from pathlib import Path

from .duckdb_memory import DuckDBMemory
from .knowledge_graph import KnowledgeGraph
from .vector_search import VectorSearch


class MemoryIntegration:
    """Unified interface for all memory systems with automatic tool tracking."""

    def __init__(
        self,
        episodic_memory: DuckDBMemory,
        vector_search: VectorSearch,
        knowledge_graph: KnowledgeGraph,
    ):
        """Initialize memory integration.

        Args:
            episodic_memory: DuckDB episodic memory instance.
            vector_search: ChromaDB vector search instance.
            knowledge_graph: NetworkX knowledge graph instance.
        """
        self.episodic = episodic_memory
        self.vector = vector_search
        self.graph = knowledge_graph

        # Context for tracking (set by agent)
        self._session_id: str | None = None
        self._task_id: str | None = None

    def set_context(self, session_id: str, task_id: str | None = None) -> None:
        """Set the current session and task context for tracking.

        Args:
            session_id: Current session ID.
            task_id: Optional task ID within the session.
        """
        self._session_id = session_id
        self._task_id = task_id

    def track_tool_execution(self, tool_func: Callable) -> Callable:
        """Decorator to automatically track tool executions.

        Usage:
            @memory.track_tool_execution
            def read_file(file_path: str) -> str:
                ...

        Args:
            tool_func: The tool function to wrap.

        Returns:
            Wrapped function that records execution to episodic memory.
        """

        @functools.wraps(tool_func)
        def wrapper(*args, **kwargs):
            if not self._session_id:
                # No context set, just execute without tracking
                return tool_func(*args, **kwargs)

            tool_name = tool_func.__name__
            parameters = {"args": args, "kwargs": kwargs}
            start_time = time.time()
            result = None
            success = False
            error_message = None

            try:
                result = tool_func(*args, **kwargs)
                success = True
                return result
            except Exception as e:
                error_message = str(e)
                raise
            finally:
                duration_ms = int((time.time() - start_time) * 1000)

                # Record to episodic memory
                self.episodic.record_tool_execution(
                    session_id=self._session_id,
                    task_id=self._task_id,
                    tool_name=tool_name,
                    parameters=parameters,
                    result=result,
                    success=success,
                    error_message=error_message,
                    duration_ms=duration_ms,
                )

                # If it's a file operation, record that too
                if "file" in tool_name.lower():
                    self._track_file_operation(
                        tool_name, args, kwargs, success, error_message
                    )

        return wrapper

    def _track_file_operation(
        self,
        tool_name: str,
        args: tuple,
        kwargs: dict,
        success: bool,
        error_message: str | None,
    ) -> None:
        """Track file-specific operations."""
        # Extract file path from args/kwargs
        file_path = None
        if args:
            file_path = str(args[0])
        elif "file_path" in kwargs:
            file_path = str(kwargs["file_path"])
        elif "path" in kwargs:
            file_path = str(kwargs["path"])

        if not file_path:
            return

        # Determine operation type
        operation = "read"
        if "write" in tool_name or "create" in tool_name:
            operation = "write"
        elif "edit" in tool_name or "modify" in tool_name:
            operation = "edit"
        elif "search" in tool_name:
            operation = "search"
        elif "analyze" in tool_name:
            operation = "analyze"

        # Record file interaction
        self.episodic.record_file_interaction(
            session_id=self._session_id,
            task_id=self._task_id,
            file_path=file_path,
            operation=operation,
            success=success,
        )

    def query_file_history(self, file_path: str, limit: int = 5) -> list[dict]:
        """Query recent interactions with a file.

        Args:
            file_path: Path to the file.
            limit: Maximum number of history entries to return.

        Returns:
            List of recent interactions.
        """
        return self.episodic.get_file_history(file_path, limit)

    def search_similar_code(
        self,
        query: str,
        n_results: int = 10,
        language: str | None = None,
    ) -> list[dict]:
        """Search for semantically similar code snippets.

        Args:
            query: Natural language or code query.
            n_results: Number of results to return.
            language: Optional language filter.

        Returns:
            List of matching code snippets with similarity scores.
        """
        return self.vector.search(query, n_results, language)

    def get_file_structure(self, file_path: str) -> dict:
        """Get the structure of a file from the knowledge graph.

        Args:
            file_path: Path to the file.

        Returns:
            Dictionary with functions, classes, and imports.
        """
        return self.graph.get_file_contents(file_path)

    def find_callers(self, function_name: str) -> list[str]:
        """Find all functions that call a given function.

        Args:
            function_name: Name of the function.

        Returns:
            List of caller function signatures.
        """
        return self.graph.get_callers(function_name)

    def find_co_edit_patterns(self, min_count: int = 3) -> list[dict]:
        """Find files that are frequently edited together.

        Args:
            min_count: Minimum co-edit count to report.

        Returns:
            List of co-edit patterns.
        """
        return self.episodic.find_co_edit_patterns(min_count)

    def get_tool_statistics(self, days: int = 7) -> list[dict]:
        """Get tool usage statistics.

        Args:
            days: Number of days to look back.

        Returns:
            List of tool statistics with usage counts and success rates.
        """
        return self.episodic.get_tool_statistics(days)

    def snapshot_context(
        self,
        context_items: list[dict],
        total_tokens: int,
        reason: str,
        pruned_items: list[dict] | None = None,
    ) -> int:
        """Save a context window snapshot.

        Args:
            context_items: List of context items with metadata.
            total_tokens: Total token count.
            reason: Reason for snapshot (pruning, task_switch, etc.).
            pruned_items: Optional list of items that were pruned.

        Returns:
            ID of the snapshot record.
        """
        return self.episodic.snapshot_context(
            session_id=self._session_id,
            task_id=self._task_id,
            context_items=context_items,
            total_tokens=total_tokens,
            reason=reason,
            pruned_items=pruned_items,
        )

    def create_task(
        self, task_id: str, description: str, intent: str | None = None
    ) -> int:
        """Create a new task in episodic memory.

        Args:
            task_id: Unique task identifier.
            description: Task description.
            intent: Optional intent classification.

        Returns:
            ID of the task record.
        """
        return self.episodic.create_task(
            task_id=task_id,
            session_id=self._session_id,
            description=description,
            intent=intent,
        )

    def complete_task(
        self,
        task_id: str,
        outcome: str,
        files_touched: list[str] | None = None,
        tools_used: dict[str, int] | None = None,
    ) -> bool:
        """Mark a task as completed.

        Args:
            task_id: Task identifier.
            outcome: Description of the task outcome.
            files_touched: List of files modified during task.
            tools_used: Dictionary of tool usage counts.

        Returns:
            True if task was successfully updated.
        """
        return self.episodic.complete_task(
            task_id=task_id,
            outcome=outcome,
            files_touched=files_touched,
            tools_used=tools_used,
        )

    def build_knowledge_graph(self, root_path: Path, languages: list[str]) -> dict:
        """Build knowledge graph from a codebase.

        Args:
            root_path: Root directory of the codebase.
            languages: List of languages to process (python, rust, etc.).

        Returns:
            Statistics about the built graph.
        """
        language_extensions = {
            "python": ".py",
            "rust": ".rs",
            "javascript": ".js",
            "typescript": ".ts",
        }

        for language in languages:
            ext = language_extensions.get(language)
            if not ext:
                continue

            # Find all files with this extension
            for file_path in root_path.rglob(f"*{ext}"):
                try:
                    self.graph.build_from_file(file_path, language)
                except Exception as e:
                    print(f"Error building graph for {file_path}: {e}")

        return self.graph.stats()


def create_memory_system(
    db_path: Path | None = None,
    vector_persist_dir: Path | None = None,
) -> MemoryIntegration:
    """Factory function to create a complete memory system.

    Args:
        db_path: Path for DuckDB database (None for in-memory).
        vector_persist_dir: Directory for ChromaDB persistence (None for in-memory).

    Returns:
        Configured MemoryIntegration instance.
    """
    episodic = DuckDBMemory(db_path)
    vector = VectorSearch(vector_persist_dir)
    graph = KnowledgeGraph()

    return MemoryIntegration(episodic, vector, graph)
