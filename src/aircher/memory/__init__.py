"""Memory system interfaces for Aircher.

Three-layer memory architecture:
1. DuckDB - Episodic memory for tracking agent actions and learning patterns
2. ChromaDB - Vector search for semantic code retrieval
3. Knowledge Graph - Code structure and relationships via tree-sitter

Usage:
    from aircher.memory import create_memory_system

    memory = create_memory_system(
        db_path=Path("data/memory.duckdb"),
        vector_persist_dir=Path("data/chroma"),
    )

    # Set context
    memory.set_context(session_id="sess_123", task_id="task_456")

    # Track tool executions automatically
    @memory.track_tool_execution
    def read_file(file_path: str) -> str:
        return Path(file_path).read_text()

    # Query memory
    history = memory.query_file_history("src/main.py")
    similar = memory.search_similar_code("function to read files")
    structure = memory.get_file_structure("src/main.py")
"""

from .duckdb_memory import DuckDBMemory
from .integration import MemoryIntegration, create_memory_system
from .knowledge_graph import KnowledgeGraph
from .tree_sitter_extractor import TreeSitterExtractor
from .vector_search import VectorSearch

__all__ = [
    "DuckDBMemory",
    "VectorSearch",
    "KnowledgeGraph",
    "TreeSitterExtractor",
    "MemoryIntegration",
    "create_memory_system",
]
