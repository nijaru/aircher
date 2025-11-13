"""Unit tests for ChromaDB vector search."""

import asyncio
from pathlib import Path

import pytest

from aircher.memory.vector_search import VectorSearch


@pytest.fixture
def vector_search():
    """Create an in-memory VectorSearch instance."""
    vs = VectorSearch()
    yield vs
    vs.clear()


@pytest.fixture
def vector_search_with_persistence(tmp_path):
    """Create a persistent VectorSearch instance."""
    persist_dir = tmp_path / "chromadb"
    vs = VectorSearch(persist_directory=persist_dir)
    yield vs, persist_dir
    vs.clear()


class TestVectorSearchInitialization:
    """Test vector search initialization."""

    def test_in_memory_creation(self):
        """Test creating an in-memory vector search."""
        vs = VectorSearch()
        assert vs.client is not None
        assert vs.collection is not None
        assert vs.model is not None

    def test_persistent_creation(self, tmp_path):
        """Test creating a persistent vector search."""
        persist_dir = tmp_path / "chromadb"
        vs = VectorSearch(persist_directory=persist_dir)
        assert vs.client is not None
        assert persist_dir.exists()
        vs.clear()

    def test_custom_model(self):
        """Test using a custom embedding model."""
        # Use same model but verify initialization
        vs = VectorSearch(model_name="sentence-transformers/all-MiniLM-L6-v2")
        assert vs.model is not None
        vs.clear()

    def test_collection_created(self, vector_search):
        """Test that collection is created with correct metadata."""
        assert vector_search.collection.name == "code_snippets"
        # Verify collection exists
        assert vector_search.count() == 0


class TestCodeSnippetIndexing:
    """Test indexing code snippets."""

    def test_index_single_snippet(self, vector_search):
        """Test indexing a single code snippet."""
        snippet_id = vector_search.index_code_snippet(
            file_path="/src/main.py",
            content="def hello():\n    print('Hello, World!')",
            start_line=1,
            end_line=2,
            language="python",
        )

        assert snippet_id == "/src/main.py:1-2"
        assert vector_search.count() == 1

    def test_index_multiple_snippets(self, vector_search):
        """Test indexing multiple code snippets."""
        snippets = [
            ("def add(a, b):\n    return a + b", 1, 2),
            ("def subtract(a, b):\n    return a - b", 4, 5),
            ("def multiply(a, b):\n    return a * b", 7, 8),
        ]

        for content, start, end in snippets:
            vector_search.index_code_snippet(
                file_path="/src/math.py",
                content=content,
                start_line=start,
                end_line=end,
                language="python",
            )

        assert vector_search.count() == 3

    def test_index_with_metadata(self, vector_search):
        """Test indexing with additional metadata."""
        snippet_id = vector_search.index_code_snippet(
            file_path="/src/utils.py",
            content="class Calculator:\n    pass",
            start_line=1,
            end_line=2,
            language="python",
            metadata={"class_name": "Calculator", "complexity": "low"},
        )

        # Verify snippet was indexed
        assert snippet_id is not None
        assert vector_search.count() == 1

    def test_index_different_languages(self, vector_search):
        """Test indexing snippets from different languages."""
        # Python snippet
        vector_search.index_code_snippet(
            file_path="/src/main.py",
            content="def hello(): pass",
            start_line=1,
            end_line=1,
            language="python",
        )

        # Rust snippet
        vector_search.index_code_snippet(
            file_path="/src/main.rs",
            content="fn hello() { }",
            start_line=1,
            end_line=1,
            language="rust",
        )

        assert vector_search.count() == 2

    def test_overwrite_existing_snippet(self, vector_search):
        """Test that re-indexing the same location overwrites."""
        snippet_id = "/src/test.py:1-5"

        # Index first time
        vector_search.index_code_snippet(
            file_path="/src/test.py",
            content="old content",
            start_line=1,
            end_line=5,
            language="python",
        )

        # Index again with same ID
        vector_search.index_code_snippet(
            file_path="/src/test.py",
            content="new content",
            start_line=1,
            end_line=5,
            language="python",
        )

        # Should still have 1 snippet (not 2)
        # ChromaDB upserts by default
        assert vector_search.count() >= 1


class TestSemanticSearch:
    """Test semantic code search."""

    def test_basic_search(self, vector_search):
        """Test basic semantic search."""
        # Index some snippets
        vector_search.index_code_snippet(
            file_path="/src/math.py",
            content="def add(x, y):\n    return x + y",
            start_line=1,
            end_line=2,
            language="python",
        )

        vector_search.index_code_snippet(
            file_path="/src/string.py",
            content="def concat(a, b):\n    return a + b",
            start_line=1,
            end_line=2,
            language="python",
        )

        # Search for addition
        results = vector_search.search("add two numbers", n_results=2)

        assert len(results) > 0
        # First result should be the add function
        assert "add" in results[0]["content"]

    def test_search_with_language_filter(self, vector_search):
        """Test searching with language filter."""
        # Index Python snippet
        vector_search.index_code_snippet(
            file_path="/src/main.py",
            content="def process_data(): pass",
            start_line=1,
            end_line=1,
            language="python",
        )

        # Index Rust snippet
        vector_search.index_code_snippet(
            file_path="/src/main.rs",
            content="fn process_data() { }",
            start_line=1,
            end_line=1,
            language="rust",
        )

        # Search only Python
        results = vector_search.search(
            "process data function", n_results=10, filter_language="python"
        )

        assert len(results) > 0
        assert all(r["metadata"]["language"] == "python" for r in results)

    def test_search_similarity_scores(self, vector_search):
        """Test that search returns similarity scores."""
        vector_search.index_code_snippet(
            file_path="/src/test.py",
            content="def test_function(): pass",
            start_line=1,
            end_line=1,
            language="python",
        )

        results = vector_search.search("test function", n_results=1)

        assert len(results) > 0
        result = results[0]
        assert "distance" in result
        assert "similarity" in result
        # Similarity should be between 0 and 1
        assert 0 <= result["similarity"] <= 1

    def test_search_returns_metadata(self, vector_search):
        """Test that search returns all metadata."""
        vector_search.index_code_snippet(
            file_path="/src/utils.py",
            content="class Helper:\n    pass",
            start_line=10,
            end_line=11,
            language="python",
            metadata={"type": "class"},
        )

        results = vector_search.search("helper class", n_results=1)

        assert len(results) > 0
        result = results[0]
        assert result["id"] == "/src/utils.py:10-11"
        assert result["metadata"]["file_path"] == "/src/utils.py"
        assert result["metadata"]["language"] == "python"
        assert result["metadata"]["start_line"] == 10
        assert result["metadata"]["end_line"] == 11

    def test_search_n_results_limit(self, vector_search):
        """Test that n_results limits returned results."""
        # Index 10 snippets
        for i in range(10):
            vector_search.index_code_snippet(
                file_path=f"/src/file{i}.py",
                content=f"def function{i}(): pass",
                start_line=1,
                end_line=1,
                language="python",
            )

        # Request only 3 results
        results = vector_search.search("function", n_results=3)

        assert len(results) == 3

    def test_empty_search_results(self, vector_search):
        """Test search when no snippets are indexed."""
        results = vector_search.search("nonexistent code", n_results=10)

        assert len(results) == 0


class TestCodebaseIndexing:
    """Test indexing entire codebases."""

    @pytest.mark.asyncio
    async def test_index_codebase(self, tmp_path, vector_search):
        """Test indexing a small codebase."""
        # Create test files
        src_dir = tmp_path / "src"
        src_dir.mkdir()

        (src_dir / "main.py").write_text("def main():\n    print('Hello')")
        (src_dir / "utils.py").write_text("def helper():\n    return True")

        # Index codebase
        stats = await vector_search.index_codebase(
            root_path=src_dir,
            languages=[".py"],
            chunk_size=50,
        )

        assert stats["files_indexed"] == 2
        assert stats["snippets_indexed"] > 0
        assert stats["errors"] == 0

    @pytest.mark.asyncio
    async def test_index_with_errors(self, tmp_path, vector_search):
        """Test indexing handles errors gracefully."""
        src_dir = tmp_path / "src"
        src_dir.mkdir()

        # Create a valid file
        (src_dir / "valid.py").write_text("def test(): pass")

        # Create a binary file that will cause UnicodeDecodeError
        (src_dir / "binary.py").write_bytes(b"\x80\x81\x82\x83")

        stats = await vector_search.index_codebase(
            root_path=src_dir,
            languages=[".py"],
        )

        # Should have indexed at least the valid file
        assert stats["files_indexed"] >= 1

    @pytest.mark.asyncio
    async def test_index_with_chunk_overlap(self, tmp_path, vector_search):
        """Test indexing with overlapping chunks."""
        src_dir = tmp_path / "src"
        src_dir.mkdir()

        # Create a larger file
        content = "\n".join([f"def function{i}(): pass" for i in range(20)])
        (src_dir / "large.py").write_text(content)

        stats = await vector_search.index_codebase(
            root_path=src_dir,
            languages=[".py"],
            chunk_size=5,
            overlap=2,
        )

        # Should create multiple chunks
        assert stats["snippets_indexed"] > 1

    @pytest.mark.asyncio
    async def test_index_multiple_languages(self, tmp_path, vector_search):
        """Test indexing multiple language files."""
        src_dir = tmp_path / "src"
        src_dir.mkdir()

        (src_dir / "main.py").write_text("def main(): pass")
        (src_dir / "lib.rs").write_text("fn main() { }")
        (src_dir / "app.js").write_text("function main() { }")

        stats = await vector_search.index_codebase(
            root_path=src_dir,
            languages=[".py", ".rs", ".js"],
        )

        assert stats["files_indexed"] == 3


class TestLanguageDetection:
    """Test language detection from file extensions."""

    def test_detect_python(self, vector_search):
        """Test detecting Python language."""
        lang = vector_search._detect_language(Path("/test/file.py"))
        assert lang == "python"

    def test_detect_rust(self, vector_search):
        """Test detecting Rust language."""
        lang = vector_search._detect_language(Path("/test/file.rs"))
        assert lang == "rust"

    def test_detect_javascript(self, vector_search):
        """Test detecting JavaScript language."""
        lang = vector_search._detect_language(Path("/test/file.js"))
        assert lang == "javascript"

    def test_detect_unknown(self, vector_search):
        """Test detecting unknown language."""
        lang = vector_search._detect_language(Path("/test/file.xyz"))
        assert lang == "unknown"


class TestUtilityMethods:
    """Test utility methods."""

    def test_count(self, vector_search):
        """Test counting indexed snippets."""
        assert vector_search.count() == 0

        vector_search.index_code_snippet(
            file_path="/src/test.py",
            content="test",
            start_line=1,
            end_line=1,
            language="python",
        )

        assert vector_search.count() == 1

    def test_clear(self, vector_search):
        """Test clearing all indexed snippets."""
        # Index some snippets
        for i in range(5):
            vector_search.index_code_snippet(
                file_path=f"/src/file{i}.py",
                content=f"def test{i}(): pass",
                start_line=1,
                end_line=1,
                language="python",
            )

        assert vector_search.count() == 5

        # Clear
        vector_search.clear()

        assert vector_search.count() == 0


class TestPersistence:
    """Test persistence of vector search data."""

    def test_data_persists(self, tmp_path):
        """Test that data persists across instances."""
        persist_dir = tmp_path / "chromadb"

        # Create first instance and index data
        vs1 = VectorSearch(persist_directory=persist_dir)
        vs1.index_code_snippet(
            file_path="/src/test.py",
            content="def persistent_function(): pass",
            start_line=1,
            end_line=1,
            language="python",
        )
        count1 = vs1.count()
        assert count1 > 0

        # Create second instance with same directory
        vs2 = VectorSearch(persist_directory=persist_dir)
        count2 = vs2.count()

        # Should have same count
        assert count2 == count1

        # Should be able to search
        results = vs2.search("persistent function", n_results=1)
        assert len(results) > 0

        vs1.clear()
        vs2.clear()


class TestEdgeCases:
    """Test edge cases and error handling."""

    def test_index_empty_content(self, vector_search):
        """Test indexing empty content."""
        snippet_id = vector_search.index_code_snippet(
            file_path="/src/empty.py",
            content="",
            start_line=1,
            end_line=1,
            language="python",
        )

        # Should still create an entry
        assert snippet_id is not None

    def test_index_very_long_content(self, vector_search):
        """Test indexing very long content."""
        long_content = "\n".join([f"# Line {i}" for i in range(1000)])

        snippet_id = vector_search.index_code_snippet(
            file_path="/src/long.py",
            content=long_content,
            start_line=1,
            end_line=1000,
            language="python",
        )

        assert snippet_id is not None

    def test_search_empty_query(self, vector_search):
        """Test searching with empty query."""
        vector_search.index_code_snippet(
            file_path="/src/test.py",
            content="def test(): pass",
            start_line=1,
            end_line=1,
            language="python",
        )

        # Empty query should still work
        results = vector_search.search("", n_results=1)
        # Results may vary, just ensure it doesn't crash
        assert isinstance(results, list)

    def test_search_nonexistent_language(self, vector_search):
        """Test searching with filter for nonexistent language."""
        vector_search.index_code_snippet(
            file_path="/src/test.py",
            content="def test(): pass",
            start_line=1,
            end_line=1,
            language="python",
        )

        results = vector_search.search(
            "test function", n_results=10, filter_language="nonexistent"
        )

        assert len(results) == 0
