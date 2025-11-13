"""Unit tests for DuckDB episodic memory."""

import json
from datetime import datetime
from pathlib import Path

import pytest

from aircher.memory.duckdb_memory import DuckDBMemory


@pytest.fixture
def memory():
    """Create an in-memory DuckDB instance for testing."""
    db = DuckDBMemory()
    yield db
    db.close()


@pytest.fixture
def memory_with_file(tmp_path):
    """Create a file-based DuckDB instance for testing persistence."""
    db_path = tmp_path / "test.duckdb"
    db = DuckDBMemory(db_path)
    yield db, db_path
    db.close()


class TestDuckDBMemoryInitialization:
    """Test database initialization and schema creation."""

    def test_in_memory_creation(self):
        """Test creating an in-memory database."""
        db = DuckDBMemory()
        assert db.conn is not None
        db.close()

    def test_file_based_creation(self, tmp_path):
        """Test creating a file-based database."""
        db_path = tmp_path / "test.duckdb"
        db = DuckDBMemory(db_path)
        assert db.conn is not None
        assert db_path.exists()
        db.close()

    def test_schema_creation(self, memory):
        """Test that all tables are created."""
        # Check all tables exist
        result = memory.conn.execute(
            "SELECT name FROM sqlite_master WHERE type='table'"
        ).fetchall()
        table_names = [row[0] for row in result]

        expected_tables = [
            "tool_executions",
            "file_interactions",
            "task_history",
            "context_snapshots",
            "learned_patterns",
        ]

        for table in expected_tables:
            assert table in table_names

    def test_indexes_created(self, memory):
        """Test that indexes are created."""
        # Check some key indexes exist
        result = memory.conn.execute(
            "SELECT name FROM sqlite_master WHERE type='index'"
        ).fetchall()
        index_names = [row[0] for row in result]

        # Should have indexes for session, file, etc.
        assert any("session" in idx for idx in index_names)
        assert any("file" in idx for idx in index_names)


class TestToolExecutionTracking:
    """Test tool execution recording and queries."""

    def test_record_tool_execution_success(self, memory):
        """Test recording a successful tool execution."""
        record_id = memory.record_tool_execution(
            session_id="test-session",
            tool_name="read_file",
            parameters={"file_path": "/test/file.py"},
            result={"content": "test content"},
            success=True,
            duration_ms=150,
        )

        assert record_id > 0

        # Verify record was inserted
        result = memory.conn.execute(
            "SELECT * FROM tool_executions WHERE id = ?", [record_id]
        ).fetchone()

        assert result is not None
        assert result[2] == "test-session"  # session_id
        assert result[4] == "read_file"  # tool_name
        assert result[7] is True  # success

    def test_record_tool_execution_failure(self, memory):
        """Test recording a failed tool execution."""
        record_id = memory.record_tool_execution(
            session_id="test-session",
            tool_name="write_file",
            parameters={"file_path": "/test/file.py"},
            result=None,
            success=False,
            error_message="Permission denied",
            duration_ms=50,
        )

        assert record_id > 0

        result = memory.conn.execute(
            "SELECT success, error_message FROM tool_executions WHERE id = ?",
            [record_id],
        ).fetchone()

        assert result[0] is False
        assert result[1] == "Permission denied"

    def test_record_with_task_id(self, memory):
        """Test recording tool execution with task ID."""
        record_id = memory.record_tool_execution(
            session_id="test-session",
            task_id="task-123",
            tool_name="search",
            parameters={"query": "test"},
            result={"matches": 5},
            success=True,
        )

        result = memory.conn.execute(
            "SELECT task_id FROM tool_executions WHERE id = ?", [record_id]
        ).fetchone()

        assert result[0] == "task-123"

    def test_get_tool_statistics(self, memory):
        """Test retrieving tool usage statistics."""
        # Record multiple tool executions
        for i in range(5):
            memory.record_tool_execution(
                session_id="test-session",
                tool_name="read_file",
                parameters={"file": f"file{i}.py"},
                result={"content": "test"},
                success=True,
                duration_ms=100 + i * 10,
            )

        for i in range(3):
            memory.record_tool_execution(
                session_id="test-session",
                tool_name="write_file",
                parameters={"file": f"file{i}.py"},
                result=None,
                success=i < 2,  # 2 successes, 1 failure
                duration_ms=200,
            )

        stats = memory.get_tool_statistics(days=7)

        # Find read_file stats
        read_stats = next((s for s in stats if s["tool_name"] == "read_file"), None)
        assert read_stats is not None
        assert read_stats["total_calls"] == 5
        assert read_stats["successful_calls"] == 5
        assert read_stats["success_rate"] == 1.0

        # Find write_file stats
        write_stats = next((s for s in stats if s["tool_name"] == "write_file"), None)
        assert write_stats is not None
        assert write_stats["total_calls"] == 3
        assert write_stats["successful_calls"] == 2
        assert write_stats["success_rate"] == pytest.approx(2 / 3)


class TestFileInteractionTracking:
    """Test file interaction recording and queries."""

    def test_record_file_interaction(self, memory):
        """Test recording a file interaction."""
        record_id = memory.record_file_interaction(
            session_id="test-session",
            file_path="/src/main.py",
            operation="read",
            success=True,
            context="Reading main file",
        )

        assert record_id > 0

        result = memory.conn.execute(
            "SELECT file_path, operation FROM file_interactions WHERE id = ?",
            [record_id],
        ).fetchone()

        assert result[0] == "/src/main.py"
        assert result[1] == "read"

    def test_record_with_line_range(self, memory):
        """Test recording file interaction with line range."""
        record_id = memory.record_file_interaction(
            session_id="test-session",
            file_path="/src/main.py",
            operation="edit",
            success=True,
            line_range={"start": 10, "end": 20},
            changes_summary="Updated function signature",
        )

        result = memory.conn.execute(
            "SELECT line_range, changes_summary FROM file_interactions WHERE id = ?",
            [record_id],
        ).fetchone()

        line_range = json.loads(result[0])
        assert line_range["start"] == 10
        assert line_range["end"] == 20
        assert result[1] == "Updated function signature"

    def test_get_file_history(self, memory):
        """Test retrieving file history."""
        file_path = "/src/main.py"

        # Record multiple interactions
        for i, op in enumerate(["read", "edit", "write", "read"]):
            memory.record_file_interaction(
                session_id="test-session",
                file_path=file_path,
                operation=op,
                success=True,
                context=f"Operation {i}",
            )

        history = memory.get_file_history(file_path, limit=3)

        # Should get 3 most recent
        assert len(history) == 3
        # Most recent first
        assert history[0]["context"] == "Operation 3"
        assert history[0]["operation"] == "read"

    def test_find_co_edit_patterns(self, memory):
        """Test finding files that are frequently edited together."""
        session_id = "test-session"

        # Simulate co-editing patterns
        # file1.py and file2.py are edited together 5 times
        for i in range(5):
            memory.record_file_interaction(
                session_id=session_id,
                file_path="/src/file1.py",
                operation="edit",
                success=True,
            )
            memory.record_file_interaction(
                session_id=session_id,
                file_path="/src/file2.py",
                operation="edit",
                success=True,
            )

        # file1.py and file3.py are edited together 2 times (below threshold)
        for i in range(2):
            memory.record_file_interaction(
                session_id=session_id,
                file_path="/src/file1.py",
                operation="edit",
                success=True,
            )
            memory.record_file_interaction(
                session_id=session_id,
                file_path="/src/file3.py",
                operation="edit",
                success=True,
            )

        patterns = memory.find_co_edit_patterns(min_count=3, within_seconds=300)

        # Should find file1.py and file2.py pattern
        assert len(patterns) >= 1
        pattern = patterns[0]
        assert {pattern["file1"], pattern["file2"]} == {
            "/src/file1.py",
            "/src/file2.py",
        }
        assert pattern["count"] >= 3


class TestTaskManagement:
    """Test task creation and completion."""

    def test_create_task(self, memory):
        """Test creating a task."""
        record_id = memory.create_task(
            task_id="task-123",
            session_id="test-session",
            description="Implement new feature",
            intent="CodeWriting",
        )

        assert record_id > 0

        result = memory.conn.execute(
            "SELECT task_id, description, intent, status FROM task_history WHERE id = ?",
            [record_id],
        ).fetchone()

        assert result[0] == "task-123"
        assert result[1] == "Implement new feature"
        assert result[2] == "CodeWriting"
        assert result[3] == "active"

    def test_complete_task(self, memory):
        """Test completing a task."""
        task_id = "task-123"
        memory.create_task(
            task_id=task_id,
            session_id="test-session",
            description="Test task",
        )

        success = memory.complete_task(
            task_id=task_id,
            outcome="Successfully implemented",
            files_touched=["/src/main.py", "/tests/test_main.py"],
            tools_used={"read_file": 3, "write_file": 2},
        )

        assert success is True

        result = memory.conn.execute(
            "SELECT status, outcome, files_touched, tools_used FROM task_history WHERE task_id = ?",
            [task_id],
        ).fetchone()

        assert result[0] == "completed"
        assert result[1] == "Successfully implemented"
        files = json.loads(result[2])
        assert len(files) == 2
        tools = json.loads(result[3])
        assert tools["read_file"] == 3

    def test_complete_nonexistent_task(self, memory):
        """Test completing a task that doesn't exist."""
        success = memory.complete_task(
            task_id="nonexistent",
            outcome="Test",
        )

        assert success is False


class TestContextSnapshots:
    """Test context window snapshot functionality."""

    def test_snapshot_context(self, memory):
        """Test saving a context snapshot."""
        context_items = [
            {"type": "file", "path": "/src/main.py", "tokens": 500},
            {"type": "message", "content": "test", "tokens": 100},
        ]

        record_id = memory.snapshot_context(
            session_id="test-session",
            context_items=context_items,
            total_tokens=600,
            reason="pruning",
        )

        assert record_id > 0

        result = memory.conn.execute(
            "SELECT context_items, total_tokens, reason FROM context_snapshots WHERE id = ?",
            [record_id],
        ).fetchone()

        saved_items = json.loads(result[0])
        assert len(saved_items) == 2
        assert result[1] == 600
        assert result[2] == "pruning"

    def test_snapshot_with_pruned_items(self, memory):
        """Test saving a snapshot with pruned items."""
        context_items = [{"type": "file", "path": "/src/main.py", "tokens": 500}]
        pruned_items = [{"type": "file", "path": "/old/file.py", "tokens": 300}]

        record_id = memory.snapshot_context(
            session_id="test-session",
            task_id="task-123",
            context_items=context_items,
            total_tokens=500,
            reason="pruning",
            pruned_items=pruned_items,
        )

        result = memory.conn.execute(
            "SELECT pruned_items FROM context_snapshots WHERE id = ?", [record_id]
        ).fetchone()

        pruned = json.loads(result[0])
        assert len(pruned) == 1
        assert pruned[0]["path"] == "/old/file.py"


class TestPersistence:
    """Test database persistence."""

    def test_data_persists_across_connections(self, tmp_path):
        """Test that data persists when reopening database."""
        db_path = tmp_path / "test.duckdb"

        # Create database and insert data
        db1 = DuckDBMemory(db_path)
        record_id = db1.record_tool_execution(
            session_id="test-session",
            tool_name="test_tool",
            parameters={"test": "value"},
            result={"success": True},
            success=True,
        )
        db1.close()

        # Reopen database and verify data
        db2 = DuckDBMemory(db_path)
        result = db2.conn.execute(
            "SELECT tool_name FROM tool_executions WHERE id = ?", [record_id]
        ).fetchone()

        assert result is not None
        assert result[0] == "test_tool"
        db2.close()


class TestEdgeCases:
    """Test edge cases and error handling."""

    def test_empty_parameters(self, memory):
        """Test recording with empty parameters."""
        record_id = memory.record_tool_execution(
            session_id="test-session",
            tool_name="test",
            parameters={},
            result=None,
            success=True,
        )

        assert record_id > 0

    def test_null_result(self, memory):
        """Test recording with null result."""
        record_id = memory.record_tool_execution(
            session_id="test-session",
            tool_name="test",
            parameters={"test": "value"},
            result=None,
            success=True,
        )

        result = memory.conn.execute(
            "SELECT result FROM tool_executions WHERE id = ?", [record_id]
        ).fetchone()

        assert result[0] is None

    def test_large_json_data(self, memory):
        """Test handling large JSON data."""
        large_params = {"data": ["item" + str(i) for i in range(1000)]}

        record_id = memory.record_tool_execution(
            session_id="test-session",
            tool_name="test",
            parameters=large_params,
            result=large_params,
            success=True,
        )

        result = memory.conn.execute(
            "SELECT parameters FROM tool_executions WHERE id = ?", [record_id]
        ).fetchone()

        params = json.loads(result[0])
        assert len(params["data"]) == 1000
