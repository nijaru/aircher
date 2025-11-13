"""DuckDB-based episodic memory for tracking agent actions and learning patterns."""

import json
from datetime import datetime
from pathlib import Path
from typing import Any

import duckdb


class DuckDBMemory:
    """Episodic memory system using DuckDB for analytical queries."""

    def __init__(self, db_path: Path | None = None):
        """Initialize DuckDB connection and create schema.

        Args:
            db_path: Path to DuckDB database file. If None, uses in-memory database.
        """
        if db_path is None:
            self.conn = duckdb.connect(":memory:")
        else:
            db_path.parent.mkdir(parents=True, exist_ok=True)
            self.conn = duckdb.connect(str(db_path))

        self._create_schema()

    def _create_schema(self) -> None:
        """Create all episodic memory tables."""
        # Create sequences for auto-incrementing IDs
        self.conn.execute("CREATE SEQUENCE IF NOT EXISTS seq_tool_executions START 1")
        self.conn.execute("CREATE SEQUENCE IF NOT EXISTS seq_file_interactions START 1")
        self.conn.execute("CREATE SEQUENCE IF NOT EXISTS seq_task_history START 1")
        self.conn.execute("CREATE SEQUENCE IF NOT EXISTS seq_context_snapshots START 1")
        self.conn.execute("CREATE SEQUENCE IF NOT EXISTS seq_learned_patterns START 1")

        # Tool execution history
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS tool_executions (
                id INTEGER PRIMARY KEY DEFAULT nextval('seq_tool_executions'),
                timestamp TIMESTAMP NOT NULL,
                session_id VARCHAR NOT NULL,
                task_id VARCHAR,
                tool_name VARCHAR NOT NULL,
                parameters JSON NOT NULL,
                result JSON,
                success BOOLEAN NOT NULL,
                error_message TEXT,
                duration_ms INTEGER,
                context_tokens INTEGER
            )
        """)
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_session ON tool_executions(session_id)"
        )
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tool_timestamp ON tool_executions(tool_name, timestamp)"
        )
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_task ON tool_executions(task_id)"
        )

        # File interaction tracking
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS file_interactions (
                id INTEGER PRIMARY KEY DEFAULT nextval('seq_file_interactions'),
                timestamp TIMESTAMP NOT NULL,
                session_id VARCHAR NOT NULL,
                task_id VARCHAR,
                file_path VARCHAR NOT NULL,
                operation VARCHAR NOT NULL,
                line_range JSON,
                success BOOLEAN NOT NULL,
                context TEXT,
                changes_summary TEXT
            )
        """)
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file ON file_interactions(file_path)"
        )
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_session_file ON file_interactions(session_id, file_path)"
        )
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_operation ON file_interactions(operation, timestamp)"
        )

        # Task history
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS task_history (
                id INTEGER PRIMARY KEY DEFAULT nextval('seq_task_history'),
                task_id VARCHAR UNIQUE NOT NULL,
                session_id VARCHAR NOT NULL,
                description TEXT NOT NULL,
                intent VARCHAR,
                status VARCHAR NOT NULL,
                started_at TIMESTAMP NOT NULL,
                completed_at TIMESTAMP,
                files_touched JSON,
                tools_used JSON,
                outcome TEXT
            )
        """)
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_status ON task_history(status)"
        )
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_session_task ON task_history(session_id)"
        )

        # Context window snapshots
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS context_snapshots (
                id INTEGER PRIMARY KEY DEFAULT nextval('seq_context_snapshots'),
                timestamp TIMESTAMP NOT NULL,
                session_id VARCHAR NOT NULL,
                task_id VARCHAR,
                context_items JSON NOT NULL,
                total_tokens INTEGER NOT NULL,
                pruned_items JSON,
                reason VARCHAR NOT NULL
            )
        """)
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_session_snapshot ON context_snapshots(session_id)"
        )

        # Learned patterns
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS learned_patterns (
                id INTEGER PRIMARY KEY DEFAULT nextval('seq_learned_patterns'),
                pattern_type VARCHAR NOT NULL,
                pattern_data JSON NOT NULL,
                confidence FLOAT NOT NULL,
                observed_count INTEGER NOT NULL,
                first_seen TIMESTAMP NOT NULL,
                last_seen TIMESTAMP NOT NULL
            )
        """)
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pattern_type ON learned_patterns(pattern_type)"
        )

    def record_tool_execution(
        self,
        session_id: str,
        tool_name: str,
        parameters: dict[str, Any],
        result: Any,
        success: bool,
        task_id: str | None = None,
        error_message: str | None = None,
        duration_ms: int | None = None,
        context_tokens: int | None = None,
    ) -> int:
        """Record a tool execution.

        Returns:
            ID of the inserted record.
        """
        result_cursor = self.conn.execute(
            """
            INSERT INTO tool_executions
            (timestamp, session_id, task_id, tool_name, parameters, result,
             success, error_message, duration_ms, context_tokens)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            [
                datetime.now(),
                session_id,
                task_id,
                tool_name,
                json.dumps(parameters),
                json.dumps(result) if result is not None else None,
                success,
                error_message,
                duration_ms,
                context_tokens,
            ],
        )
        # Get the last inserted ID
        id_cursor = self.conn.execute("SELECT currval('seq_tool_executions')")
        return id_cursor.fetchone()[0]

    def record_file_interaction(
        self,
        session_id: str,
        file_path: str,
        operation: str,
        success: bool,
        task_id: str | None = None,
        line_range: dict[str, int] | None = None,
        context: str | None = None,
        changes_summary: str | None = None,
    ) -> int:
        """Record a file interaction.

        Args:
            operation: One of: read, write, edit, search, analyze

        Returns:
            ID of the inserted record.
        """
        self.conn.execute(
            """
            INSERT INTO file_interactions
            (timestamp, session_id, task_id, file_path, operation,
             line_range, success, context, changes_summary)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            [
                datetime.now(),
                session_id,
                task_id,
                file_path,
                operation,
                json.dumps(line_range) if line_range else None,
                success,
                context,
                changes_summary,
            ],
        )
        # Get the last inserted ID
        id_cursor = self.conn.execute("SELECT currval('seq_file_interactions')")
        return id_cursor.fetchone()[0]

    def get_file_history(self, file_path: str, limit: int = 5) -> list[dict]:
        """Get recent interaction history for a file.

        Returns:
            List of interaction records, most recent first.
        """
        result = self.conn.execute(
            """
            SELECT timestamp, session_id, operation, success, context, changes_summary
            FROM file_interactions
            WHERE file_path = ?
            ORDER BY timestamp DESC
            LIMIT ?
            """,
            [file_path, limit],
        )
        return [
            {
                "timestamp": row[0],
                "session_id": row[1],
                "operation": row[2],
                "success": row[3],
                "context": row[4],
                "changes_summary": row[5],
            }
            for row in result.fetchall()
        ]

    def find_co_edit_patterns(
        self, min_count: int = 3, within_seconds: int = 300
    ) -> list[dict]:
        """Find files that are frequently edited together.

        Args:
            min_count: Minimum number of co-edits to report.
            within_seconds: Time window for co-edits (default 5 minutes).

        Returns:
            List of co-edit patterns with file pairs and counts.
        """
        result = self.conn.execute(
            """
            SELECT
                f1.file_path as file1,
                f2.file_path as file2,
                COUNT(*) as co_edit_count
            FROM file_interactions f1
            JOIN file_interactions f2
                ON f1.session_id = f2.session_id
                AND f1.file_path < f2.file_path
                AND f1.operation IN ('edit', 'write')
                AND f2.operation IN ('edit', 'write')
                AND EPOCH(f2.timestamp) - EPOCH(f1.timestamp) BETWEEN 0 AND ?
            GROUP BY f1.file_path, f2.file_path
            HAVING COUNT(*) >= ?
            ORDER BY co_edit_count DESC
            """,
            [within_seconds, min_count],
        )
        return [
            {"file1": row[0], "file2": row[1], "count": row[2]}
            for row in result.fetchall()
        ]

    def get_tool_statistics(self, days: int = 7) -> list[dict]:
        """Get tool usage statistics for the past N days.

        Returns:
            List of tools with usage counts and success rates.
        """
        result = self.conn.execute(
            """
            SELECT
                tool_name,
                COUNT(*) as total_calls,
                SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful_calls,
                AVG(duration_ms) as avg_duration_ms
            FROM tool_executions
            WHERE timestamp >= CURRENT_TIMESTAMP - INTERVAL (? || ' days')::INTERVAL
            GROUP BY tool_name
            ORDER BY total_calls DESC
            """,
            [str(days)],
        )
        return [
            {
                "tool_name": row[0],
                "total_calls": row[1],
                "successful_calls": row[2],
                "avg_duration_ms": row[3],
                "success_rate": row[2] / row[1] if row[1] > 0 else 0.0,
            }
            for row in result.fetchall()
        ]

    def create_task(
        self,
        task_id: str,
        session_id: str,
        description: str,
        intent: str | None = None,
    ) -> int:
        """Create a new task record.

        Args:
            intent: One of: CodeReading, CodeWriting, ProjectFixing, ProjectExploration

        Returns:
            ID of the inserted record.
        """
        self.conn.execute(
            """
            INSERT INTO task_history
            (task_id, session_id, description, intent, status, started_at)
            VALUES (?, ?, ?, ?, 'active', ?)
            """,
            [task_id, session_id, description, intent, datetime.now()],
        )
        # Get the last inserted ID
        id_cursor = self.conn.execute("SELECT currval('seq_task_history')")
        return id_cursor.fetchone()[0]

    def complete_task(
        self,
        task_id: str,
        outcome: str,
        files_touched: list[str] | None = None,
        tools_used: dict[str, int] | None = None,
    ) -> bool:
        """Mark a task as completed.

        Returns:
            True if task was found and updated.
        """
        # Check if task exists before updating
        result = self.conn.execute(
            "SELECT COUNT(*) FROM task_history WHERE task_id = ?", [task_id]
        ).fetchone()
        task_exists = result[0] > 0

        if task_exists:
            self.conn.execute(
                """
                UPDATE task_history
                SET status = 'completed',
                    completed_at = ?,
                    outcome = ?,
                    files_touched = ?,
                    tools_used = ?
                WHERE task_id = ?
                """,
                [
                    datetime.now(),
                    outcome,
                    json.dumps(files_touched) if files_touched else None,
                    json.dumps(tools_used) if tools_used else None,
                    task_id,
                ],
            )

        return task_exists

    def snapshot_context(
        self,
        session_id: str,
        context_items: list[dict],
        total_tokens: int,
        reason: str,
        task_id: str | None = None,
        pruned_items: list[dict] | None = None,
    ) -> int:
        """Save a context window snapshot.

        Args:
            reason: One of: pruning, task_switch, manual_snapshot

        Returns:
            ID of the inserted record.
        """
        self.conn.execute(
            """
            INSERT INTO context_snapshots
            (timestamp, session_id, task_id, context_items, total_tokens,
             pruned_items, reason)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            """,
            [
                datetime.now(),
                session_id,
                task_id,
                json.dumps(context_items),
                total_tokens,
                json.dumps(pruned_items) if pruned_items else None,
                reason,
            ],
        )
        # Get the last inserted ID
        id_cursor = self.conn.execute("SELECT currval('seq_context_snapshots')")
        return id_cursor.fetchone()[0]

    def close(self) -> None:
        """Close the database connection."""
        self.conn.close()
