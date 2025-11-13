"""ChromaDB-based vector search for semantic code retrieval."""

import asyncio
from pathlib import Path
from typing import Any

import chromadb
from chromadb.config import Settings
from sentence_transformers import SentenceTransformer


class VectorSearch:
    """Semantic code search using ChromaDB and sentence-transformers."""

    def __init__(
        self,
        persist_directory: Path | None = None,
        model_name: str = "sentence-transformers/all-MiniLM-L6-v2",
    ):
        """Initialize ChromaDB client and embedding model.

        Args:
            persist_directory: Directory to persist ChromaDB data. If None, uses in-memory.
            model_name: Sentence-transformers model name (default: all-MiniLM-L6-v2, 384 dims).
        """
        if persist_directory is None:
            self.client = chromadb.Client(Settings(is_persistent=False))
        else:
            persist_directory.mkdir(parents=True, exist_ok=True)
            self.client = chromadb.Client(
                Settings(
                    is_persistent=True,
                    persist_directory=str(persist_directory),
                )
            )

        # Load sentence-transformers model
        self.model = SentenceTransformer(model_name)

        # Create or get collection
        self.collection = self.client.get_or_create_collection(
            name="code_snippets",
            metadata={"hnsw:space": "cosine"},
        )

    def index_code_snippet(
        self,
        file_path: str,
        content: str,
        start_line: int,
        end_line: int,
        language: str,
        metadata: dict[str, Any] | None = None,
    ) -> str:
        """Index a code snippet for semantic search.

        Args:
            file_path: Path to the file containing the code.
            content: The code snippet content.
            start_line: Starting line number in the file.
            end_line: Ending line number in the file.
            language: Programming language (python, rust, javascript, etc.).
            metadata: Additional metadata to store with the snippet.

        Returns:
            ID of the indexed snippet.
        """
        # Generate unique ID
        snippet_id = f"{file_path}:{start_line}-{end_line}"

        # Create embedding
        embedding = self.model.encode(content).tolist()

        # Prepare metadata
        snippet_metadata = {
            "file_path": file_path,
            "start_line": start_line,
            "end_line": end_line,
            "language": language,
            "length": len(content),
        }
        if metadata:
            snippet_metadata.update(metadata)

        # Add to collection
        self.collection.add(
            ids=[snippet_id],
            embeddings=[embedding],
            documents=[content],
            metadatas=[snippet_metadata],
        )

        return snippet_id

    def search(
        self,
        query: str,
        n_results: int = 10,
        filter_language: str | None = None,
    ) -> list[dict]:
        """Search for semantically similar code snippets.

        Args:
            query: Natural language or code query.
            n_results: Number of results to return.
            filter_language: Optional language filter (python, rust, etc.).

        Returns:
            List of matching snippets with content, metadata, and similarity scores.
        """
        # Create query embedding
        query_embedding = self.model.encode(query).tolist()

        # Build filter
        where_filter = None
        if filter_language:
            where_filter = {"language": filter_language}

        # Query collection
        results = self.collection.query(
            query_embeddings=[query_embedding],
            n_results=n_results,
            where=where_filter,
        )

        # Format results
        formatted_results = []
        for i in range(len(results["ids"][0])):
            formatted_results.append(
                {
                    "id": results["ids"][0][i],
                    "content": results["documents"][0][i],
                    "metadata": results["metadatas"][0][i],
                    "distance": results["distances"][0][i],
                    "similarity": 1 - results["distances"][0][i],  # Cosine similarity
                }
            )

        return formatted_results

    async def index_codebase(
        self,
        root_path: Path,
        languages: list[str] | None = None,
        chunk_size: int = 50,
        overlap: int = 5,
    ) -> dict[str, int]:
        """Index an entire codebase in the background.

        Args:
            root_path: Root directory of the codebase.
            languages: List of file extensions to index (e.g., ['.py', '.rs']).
                       If None, indexes all text files.
            chunk_size: Number of lines per code chunk.
            overlap: Number of overlapping lines between chunks.

        Returns:
            Statistics: {files_indexed, snippets_indexed, errors}.
        """
        if languages is None:
            languages = [".py", ".rs", ".js", ".ts", ".go", ".java", ".cpp", ".c"]

        stats = {"files_indexed": 0, "snippets_indexed": 0, "errors": 0}

        # Find all matching files
        files = []
        for ext in languages:
            files.extend(root_path.rglob(f"*{ext}"))

        # Index files
        for file_path in files:
            try:
                await self._index_file(file_path, chunk_size, overlap)
                stats["files_indexed"] += 1
            except Exception as e:
                stats["errors"] += 1
                print(f"Error indexing {file_path}: {e}")

        # Count total snippets
        stats["snippets_indexed"] = self.collection.count()

        return stats

    async def _index_file(self, file_path: Path, chunk_size: int, overlap: int) -> None:
        """Index a single file by splitting into chunks."""
        # Read file
        try:
            content = file_path.read_text()
        except UnicodeDecodeError:
            # Skip binary files
            return

        lines = content.splitlines()
        language = self._detect_language(file_path)

        # Split into chunks with overlap
        i = 0
        while i < len(lines):
            end = min(i + chunk_size, len(lines))
            chunk = "\n".join(lines[i:end])

            if chunk.strip():  # Skip empty chunks
                self.index_code_snippet(
                    file_path=str(file_path),
                    content=chunk,
                    start_line=i + 1,
                    end_line=end,
                    language=language,
                )

            i += chunk_size - overlap

        # Yield control to event loop
        await asyncio.sleep(0)

    def _detect_language(self, file_path: Path) -> str:
        """Detect programming language from file extension."""
        ext_to_lang = {
            ".py": "python",
            ".rs": "rust",
            ".js": "javascript",
            ".ts": "typescript",
            ".go": "go",
            ".java": "java",
            ".cpp": "cpp",
            ".c": "c",
            ".rb": "ruby",
            ".php": "php",
            ".swift": "swift",
            ".kt": "kotlin",
        }
        return ext_to_lang.get(file_path.suffix, "unknown")

    def clear(self) -> None:
        """Clear all indexed snippets."""
        self.client.delete_collection("code_snippets")
        self.collection = self.client.get_or_create_collection(
            name="code_snippets",
            metadata={"hnsw:space": "cosine"},
        )

    def count(self) -> int:
        """Get the total number of indexed snippets."""
        return self.collection.count()
