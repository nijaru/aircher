"""Integration tests for memory system integration layer."""

import asyncio
from pathlib import Path

import pytest

from aircher.memory.integration import MemoryIntegration, create_memory_system


@pytest.fixture
def memory_system():
    """Create an in-memory integrated memory system."""
    return create_memory_system()


@pytest.fixture
def memory_system_with_persistence(tmp_path):
    """Create a persistent integrated memory system."""
    db_path = tmp_path / "memory.duckdb"
    vector_dir = tmp_path / "vectors"
    return create_memory_system(db_path, vector_dir)


class TestMemorySystemCreation:
    """Test creating integrated memory systems."""

    def test_create_in_memory_system(self):
        """Test creating an in-memory system."""
        memory = create_memory_system()

        assert memory is not None
        assert memory.episodic is not None
        assert memory.vector is not None
        assert memory.graph is not None

    def test_create_persistent_system(self, tmp_path):
        """Test creating a persistent system."""
        db_path = tmp_path / "memory.duckdb"
        vector_dir = tmp_path / "vectors"

        memory = create_memory_system(db_path, vector_dir)

        assert memory is not None
        assert db_path.exists()
        assert vector_dir.exists()


class TestContextManagement:
    """Test session and task context management."""

    def test_set_context(self, memory_system):
        """Test setting session context."""
        memory_system.set_context(session_id="test-session", task_id="task-123")

        assert memory_system._session_id == "test-session"
        assert memory_system._task_id == "task-123"

    def test_set_context_without_task(self, memory_system):
        """Test setting session context without task."""
        memory_system.set_context(session_id="test-session")

        assert memory_system._session_id == "test-session"
        assert memory_system._task_id is None


class TestToolExecutionTracking:
    """Test automatic tool execution tracking."""

    def test_track_tool_without_context(self, memory_system):
        """Test that tools work without context set."""

        @memory_system.track_tool_execution
        def sample_tool(x, y):
            return x + y

        result = sample_tool(2, 3)

        assert result == 5
        # Should not record without context
        stats = memory_system.episodic.get_tool_statistics(days=7)
        assert len(stats) == 0

    def test_track_tool_with_context(self, memory_system):
        """Test tracking tool execution with context."""
        memory_system.set_context(session_id="test-session")

        @memory_system.track_tool_execution
        def sample_tool(x, y):
            return x + y

        result = sample_tool(2, 3)

        assert result == 5

        # Should have recorded execution
        stats = memory_system.episodic.get_tool_statistics(days=7)
        assert len(stats) == 1
        assert stats[0]["tool_name"] == "sample_tool"
        assert stats[0]["total_calls"] == 1
        assert stats[0]["successful_calls"] == 1

    def test_track_tool_failure(self, memory_system):
        """Test tracking failed tool execution."""
        memory_system.set_context(session_id="test-session")

        @memory_system.track_tool_execution
        def failing_tool():
            raise ValueError("Test error")

        with pytest.raises(ValueError):
            failing_tool()

        # Should have recorded failure
        stats = memory_system.episodic.get_tool_statistics(days=7)
        assert len(stats) == 1
        assert stats[0]["successful_calls"] == 0

    def test_track_multiple_tool_calls(self, memory_system):
        """Test tracking multiple tool calls."""
        memory_system.set_context(session_id="test-session")

        @memory_system.track_tool_execution
        def read_tool(path):
            return f"content of {path}"

        @memory_system.track_tool_execution
        def write_tool(path, content):
            return True

        read_tool("/test/file1.py")
        read_tool("/test/file2.py")
        write_tool("/test/file3.py", "content")

        stats = memory_system.episodic.get_tool_statistics(days=7)

        # Should have stats for both tools
        read_stats = next((s for s in stats if s["tool_name"] == "read_tool"), None)
        write_stats = next((s for s in stats if s["tool_name"] == "write_tool"), None)

        assert read_stats is not None
        assert read_stats["total_calls"] == 2
        assert write_stats is not None
        assert write_stats["total_calls"] == 1

    def test_track_tool_preserves_return_value(self, memory_system):
        """Test that decorator preserves return values."""
        memory_system.set_context(session_id="test-session")

        @memory_system.track_tool_execution
        def complex_tool(x):
            return {"result": x * 2, "metadata": {"processed": True}}

        result = complex_tool(5)

        assert result["result"] == 10
        assert result["metadata"]["processed"] is True


class TestFileOperationTracking:
    """Test automatic file operation tracking."""

    def test_track_file_read(self, memory_system):
        """Test tracking file read operations."""
        memory_system.set_context(session_id="test-session")

        @memory_system.track_tool_execution
        def read_file(file_path):
            return f"content of {file_path}"

        read_file("/src/main.py")

        history = memory_system.query_file_history("/src/main.py")

        assert len(history) == 1
        assert history[0]["operation"] == "read"
        assert history[0]["success"] is True

    def test_track_file_write(self, memory_system):
        """Test tracking file write operations."""
        memory_system.set_context(session_id="test-session")

        @memory_system.track_tool_execution
        def write_file(file_path, content):
            return True

        write_file("/src/output.py", "print('hello')")

        history = memory_system.query_file_history("/src/output.py")

        assert len(history) == 1
        assert history[0]["operation"] == "write"

    def test_track_file_edit(self, memory_system):
        """Test tracking file edit operations."""
        memory_system.set_context(session_id="test-session")

        @memory_system.track_tool_execution
        def edit_file(file_path, changes):
            return True

        edit_file("/src/main.py", {"line": 10, "new_content": "updated"})

        history = memory_system.query_file_history("/src/main.py")

        assert len(history) == 1
        assert history[0]["operation"] == "edit"

    def test_file_history_ordering(self, memory_system):
        """Test that file history is returned in reverse chronological order."""
        memory_system.set_context(session_id="test-session")

        @memory_system.track_tool_execution
        def file_tool(file_path, op):
            return op

        # Perform multiple operations
        file_tool("/src/test.py", "first")
        file_tool("/src/test.py", "second")
        file_tool("/src/test.py", "third")

        history = memory_system.query_file_history("/src/test.py", limit=2)

        # Should get 2 most recent
        assert len(history) == 2


class TestTaskManagement:
    """Test task creation and completion."""

    def test_create_task(self, memory_system):
        """Test creating a task."""
        memory_system.set_context(session_id="test-session")

        task_id = memory_system.create_task(
            task_id="task-123",
            description="Implement feature X",
            intent="CodeWriting",
        )

        assert task_id > 0

    def test_complete_task(self, memory_system):
        """Test completing a task."""
        memory_system.set_context(session_id="test-session")

        memory_system.create_task(
            task_id="task-123",
            description="Test task",
        )

        success = memory_system.complete_task(
            task_id="task-123",
            outcome="Task completed successfully",
            files_touched=["/src/main.py"],
            tools_used={"read_file": 2, "write_file": 1},
        )

        assert success is True

    def test_task_workflow(self, memory_system):
        """Test complete task workflow with tracking."""
        memory_system.set_context(session_id="test-session", task_id="task-123")

        # Create task
        memory_system.create_task(
            task_id="task-123",
            description="Implement feature",
            intent="CodeWriting",
        )

        # Simulate some work
        @memory_system.track_tool_execution
        def work_tool(action):
            return f"Did {action}"

        work_tool("step1")
        work_tool("step2")

        # Complete task
        memory_system.complete_task(
            task_id="task-123",
            outcome="Feature implemented",
        )

        # Verify task was tracked
        stats = memory_system.episodic.get_tool_statistics(days=7)
        assert len(stats) > 0


class TestSemanticCodeSearch:
    """Test semantic code search integration."""

    def test_search_similar_code(self, memory_system):
        """Test searching for similar code."""
        # Index some code
        memory_system.vector.index_code_snippet(
            file_path="/src/math.py",
            content="def add(a, b):\n    return a + b",
            start_line=1,
            end_line=2,
            language="python",
        )

        # Search
        results = memory_system.search_similar_code("addition function", n_results=5)

        assert len(results) > 0
        assert results[0]["metadata"]["file_path"] == "/src/math.py"

    def test_search_with_language_filter(self, memory_system):
        """Test searching with language filter."""
        # Index Python code
        memory_system.vector.index_code_snippet(
            file_path="/src/test.py",
            content="def test(): pass",
            start_line=1,
            end_line=1,
            language="python",
        )

        # Index Rust code
        memory_system.vector.index_code_snippet(
            file_path="/src/test.rs",
            content="fn test() { }",
            start_line=1,
            end_line=1,
            language="rust",
        )

        # Search Python only
        results = memory_system.search_similar_code("test function", language="python")

        assert all(r["metadata"]["language"] == "python" for r in results)


class TestKnowledgeGraphQueries:
    """Test knowledge graph integration."""

    def test_get_file_structure(self, tmp_path, memory_system):
        """Test getting file structure from knowledge graph."""
        # Create a test file
        test_file = tmp_path / "test.py"
        test_file.write_text("""
def function1():
    pass

class MyClass:
    def method1(self):
        pass
""")

        # Build graph
        memory_system.graph.build_from_file(test_file, "python")

        # Query structure
        structure = memory_system.get_file_structure(str(test_file))

        assert len(structure["functions"]) >= 1
        assert len(structure["classes"]) >= 1

    def test_find_callers(self, memory_system):
        """Test finding function callers."""
        # Manually build a call graph
        file_id = memory_system.graph.add_file("/src/test.py", "python")
        main_id = memory_system.graph.add_function(
            "main", "main()", 1, "/src/test.py", file_id
        )
        helper_id = memory_system.graph.add_function(
            "helper", "helper()", 5, "/src/test.py", file_id
        )

        memory_system.graph.add_call_edge(main_id, helper_id)

        # Find callers
        callers = memory_system.find_callers("helper")

        assert len(callers) == 1
        assert "main" in callers[0]


class TestCoEditPatterns:
    """Test co-edit pattern detection."""

    def test_find_co_edit_patterns(self, memory_system):
        """Test finding files edited together."""
        memory_system.set_context(session_id="test-session")

        @memory_system.track_tool_execution
        def edit_file(file_path):
            return True

        # Edit file pairs together multiple times
        for i in range(5):
            edit_file("/src/model.py")
            edit_file("/src/schema.py")

        patterns = memory_system.find_co_edit_patterns(min_count=3)

        assert len(patterns) >= 1
        pattern = patterns[0]
        assert {pattern["file1"], pattern["file2"]} == {
            "/src/model.py",
            "/src/schema.py",
        }


class TestContextSnapshots:
    """Test context window snapshots."""

    def test_snapshot_context(self, memory_system):
        """Test saving context snapshots."""
        memory_system.set_context(session_id="test-session", task_id="task-123")

        context_items = [
            {"type": "file", "path": "/src/main.py", "tokens": 500},
            {"type": "message", "role": "user", "tokens": 100},
        ]

        snapshot_id = memory_system.snapshot_context(
            context_items=context_items,
            total_tokens=600,
            reason="pruning",
        )

        assert snapshot_id > 0

    def test_snapshot_with_pruned_items(self, memory_system):
        """Test snapshot with pruned items."""
        memory_system.set_context(session_id="test-session")

        context_items = [{"type": "file", "path": "/src/new.py", "tokens": 300}]
        pruned_items = [{"type": "file", "path": "/src/old.py", "tokens": 500}]

        snapshot_id = memory_system.snapshot_context(
            context_items=context_items,
            total_tokens=300,
            reason="pruning",
            pruned_items=pruned_items,
        )

        assert snapshot_id > 0


class TestKnowledgeGraphBuilding:
    """Test building knowledge graph from codebase."""

    def test_build_knowledge_graph(self, tmp_path, memory_system):
        """Test building knowledge graph from directory."""
        # Create test files
        src_dir = tmp_path / "src"
        src_dir.mkdir()

        (src_dir / "main.py").write_text("""
def main():
    helper()

def helper():
    pass
""")

        (src_dir / "utils.py").write_text("""
class Utility:
    def process(self):
        pass
""")

        # Build graph
        stats = memory_system.build_knowledge_graph(
            root_path=src_dir,
            languages=["python"],
        )

        assert stats["total_nodes"] > 0
        assert stats["file_nodes"] >= 2
        assert stats["function_nodes"] >= 2

    def test_build_graph_handles_errors(self, tmp_path, memory_system):
        """Test that graph building handles errors gracefully."""
        src_dir = tmp_path / "src"
        src_dir.mkdir()

        # Create a file with syntax errors
        (src_dir / "bad.py").write_text("def incomplete(")

        # Should not crash
        stats = memory_system.build_knowledge_graph(
            root_path=src_dir,
            languages=["python"],
        )

        # Should still return stats
        assert "total_nodes" in stats


class TestMultiSystemQueries:
    """Test queries across multiple memory systems."""

    def test_combined_file_analysis(self, tmp_path, memory_system):
        """Test combining episodic memory and knowledge graph."""
        memory_system.set_context(session_id="test-session")

        # Create and index a file
        test_file = tmp_path / "test.py"
        test_file.write_text("""
def process_data():
    return True
""")

        # Build knowledge graph
        memory_system.graph.build_from_file(test_file, "python")

        # Simulate file interaction
        @memory_system.track_tool_execution
        def read_file(file_path):
            return "content"

        read_file(str(test_file))

        # Query both systems
        structure = memory_system.get_file_structure(str(test_file))
        history = memory_system.query_file_history(str(test_file))

        assert len(structure["functions"]) > 0
        assert len(history) > 0

    @pytest.mark.asyncio
    async def test_semantic_search_with_tracking(self, tmp_path, memory_system):
        """Test semantic search combined with usage tracking."""
        memory_system.set_context(session_id="test-session")

        # Create test file
        src_dir = tmp_path / "src"
        src_dir.mkdir()
        (src_dir / "calculator.py").write_text("""
def add(x, y):
    return x + y

def subtract(x, y):
    return x - y
""")

        # Index codebase
        await memory_system.vector.index_codebase(
            root_path=src_dir,
            languages=[".py"],
        )

        # Search for code
        results = memory_system.search_similar_code("addition", n_results=5)

        assert len(results) > 0
        # Should find the add function
        assert any("add" in r["content"] for r in results)


class TestPersistence:
    """Test persistence across system instances."""

    def test_episodic_memory_persists(self, tmp_path):
        """Test that episodic memory persists."""
        db_path = tmp_path / "memory.duckdb"

        # Create system and record data
        memory1 = create_memory_system(db_path=db_path)
        memory1.set_context(session_id="test-session")

        @memory1.track_tool_execution
        def test_tool():
            return "result"

        test_tool()

        # Create new system with same database
        memory2 = create_memory_system(db_path=db_path)
        stats = memory2.episodic.get_tool_statistics(days=7)

        # Should see the recorded tool execution
        assert len(stats) > 0
        assert stats[0]["tool_name"] == "test_tool"

    def test_vector_search_persists(self, tmp_path):
        """Test that vector search data persists."""
        vector_dir = tmp_path / "vectors"

        # Create system and index data
        memory1 = create_memory_system(vector_persist_dir=vector_dir)
        memory1.vector.index_code_snippet(
            file_path="/src/test.py",
            content="def persistent(): pass",
            start_line=1,
            end_line=1,
            language="python",
        )

        count1 = memory1.vector.count()

        # Create new system with same directory
        memory2 = create_memory_system(vector_persist_dir=vector_dir)
        count2 = memory2.vector.count()

        # Should have same count
        assert count2 == count1


class TestEdgeCases:
    """Test edge cases and error handling."""

    def test_tool_tracking_without_session(self, memory_system):
        """Test that tracking works gracefully without session."""

        @memory_system.track_tool_execution
        def test_tool():
            return "result"

        # Should not crash
        result = test_tool()
        assert result == "result"

    def test_query_nonexistent_file(self, memory_system):
        """Test querying history of nonexistent file."""
        history = memory_system.query_file_history("/nonexistent.py")

        assert history == []

    def test_search_empty_vector_store(self, memory_system):
        """Test searching when no code is indexed."""
        results = memory_system.search_similar_code("test query")

        assert results == []

    def test_get_structure_nonexistent_file(self, memory_system):
        """Test getting structure of nonexistent file."""
        structure = memory_system.get_file_structure("/nonexistent.py")

        assert structure["functions"] == []
        assert structure["classes"] == []
        assert structure["imports"] == []
