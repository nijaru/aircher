"""SQLite session storage for Aircher."""

import json
import sqlite3
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional

from loguru import logger

from ..protocol import ACPMessage, ACPSession, AgentMode


class SessionStorage:
    """SQLite-based session storage."""

    def __init__(self, db_path: Path | None = None):
        if db_path is None:
            db_path = Path.home() / ".aircher" / "sessions.db"

        self.db_path = db_path
        self.db_path.parent.mkdir(parents=True, exist_ok=True)

        self._init_database()

    def _init_database(self):
        """Initialize database tables."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute("""
                CREATE TABLE IF NOT EXISTS sessions (
                    id TEXT PRIMARY KEY,
                    created_at TIMESTAMP NOT NULL,
                    last_activity TIMESTAMP NOT NULL,
                    mode TEXT NOT NULL,
                    user_id TEXT,
                    metadata TEXT
                )
            """)

            conn.execute("""
                CREATE TABLE IF NOT EXISTS messages (
                    id TEXT PRIMARY KEY,
                    session_id TEXT NOT NULL,
                    type TEXT NOT NULL,
                    timestamp TIMESTAMP NOT NULL,
                    data TEXT NOT NULL,
                    FOREIGN KEY (session_id) REFERENCES sessions (id) ON DELETE CASCADE
                )
            """)

            conn.execute("""
                CREATE TABLE IF NOT EXISTS tool_calls (
                    id TEXT PRIMARY KEY,
                    session_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    parameters TEXT NOT NULL,
                    status TEXT NOT NULL,
                    result TEXT,
                    error TEXT,
                    start_time TIMESTAMP,
                    end_time TIMESTAMP,
                    FOREIGN KEY (session_id) REFERENCES sessions (id) ON DELETE CASCADE
                )
            """)

            # Create indexes for performance
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_sessions_last_activity ON sessions (last_activity)"
            )
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_messages_session_timestamp ON messages (session_id, timestamp)"
            )
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_tool_calls_session ON tool_calls (session_id)"
            )

            conn.commit()

    def create_session(self, session: ACPSession) -> bool:
        """Create a new session."""
        try:
            with sqlite3.connect(self.db_path) as conn:
                conn.execute(
                    """
                    INSERT INTO sessions (id, created_at, last_activity, mode, user_id, metadata)
                    VALUES (?, ?, ?, ?, ?, ?)
                    """,
                    (
                        session.id,
                        session.created_at.isoformat(),
                        session.last_activity.isoformat(),
                        session.mode.value,
                        session.user_id,
                        json.dumps(session.metadata) if session.metadata else None,
                    ),
                )
                conn.commit()
                return True
        except sqlite3.Error as e:
            logger.error(f"Failed to create session: {e}")
            return False

    def get_session(self, session_id: str) -> ACPSession | None:
        """Get session by ID."""
        try:
            with sqlite3.connect(self.db_path) as conn:
                conn.row_factory = sqlite3.Row
                cursor = conn.execute(
                    "SELECT * FROM sessions WHERE id = ?", (session_id,)
                )
                row = cursor.fetchone()

                if row:
                    return ACPSession(
                        id=row["id"],
                        created_at=datetime.fromisoformat(row["created_at"]),
                        last_activity=datetime.fromisoformat(row["last_activity"]),
                        mode=AgentMode(row["mode"]),
                        user_id=row["user_id"],
                        metadata=json.loads(row["metadata"]) if row["metadata"] else {},
                    )
                return None
        except sqlite3.Error as e:
            logger.error(f"Failed to get session: {e}")
            return None

    def update_session(self, session: ACPSession) -> bool:
        """Update session information."""
        try:
            with sqlite3.connect(self.db_path) as conn:
                conn.execute(
                    """
                    UPDATE sessions
                    SET last_activity = ?, mode = ?, user_id = ?, metadata = ?
                    WHERE id = ?
                    """,
                    (
                        session.last_activity.isoformat(),
                        session.mode.value,
                        session.user_id,
                        json.dumps(session.metadata) if session.metadata else None,
                        session.id,
                    ),
                )
                conn.commit()
                return True
        except sqlite3.Error as e:
            logger.error(f"Failed to update session: {e}")
            return False

    def delete_session(self, session_id: str) -> bool:
        """Delete a session and all associated data."""
        try:
            with sqlite3.connect(self.db_path) as conn:
                conn.execute("DELETE FROM sessions WHERE id = ?", (session_id,))
                conn.commit()
                return True
        except sqlite3.Error as e:
            logger.error(f"Failed to delete session: {e}")
            return False

    def list_sessions(self, limit: int = 100, offset: int = 0) -> list[ACPSession]:
        """List sessions ordered by last activity."""
        try:
            with sqlite3.connect(self.db_path) as conn:
                conn.row_factory = sqlite3.Row
                cursor = conn.execute(
                    """
                    SELECT * FROM sessions
                    ORDER BY last_activity DESC
                    LIMIT ? OFFSET ?
                    """,
                    (limit, offset),
                )

                sessions = []
                for row in cursor.fetchall():
                    sessions.append(
                        ACPSession(
                            id=row["id"],
                            created_at=datetime.fromisoformat(row["created_at"]),
                            last_activity=datetime.fromisoformat(row["last_activity"]),
                            mode=AgentMode(row["mode"]),
                            user_id=row["user_id"],
                            metadata=json.loads(row["metadata"])
                            if row["metadata"]
                            else {},
                        )
                    )

                return sessions
        except sqlite3.Error as e:
            logger.error(f"Failed to list sessions: {e}")
            return []

    def store_message(self, message: ACPMessage) -> bool:
        """Store a message."""
        try:
            with sqlite3.connect(self.db_path) as conn:
                conn.execute(
                    """
                    INSERT INTO messages (id, session_id, type, timestamp, data)
                    VALUES (?, ?, ?, ?, ?)
                    """,
                    (
                        message.id,
                        message.session_id,
                        message.type.value,
                        message.timestamp.isoformat(),
                        json.dumps(message.to_dict()),
                    ),
                )
                conn.commit()
                return True
        except sqlite3.Error as e:
            logger.error(f"Failed to store message: {e}")
            return False

    def get_messages(
        self, session_id: str, limit: int = 100, offset: int = 0
    ) -> list[dict[str, Any]]:
        """Get messages for a session."""
        try:
            with sqlite3.connect(self.db_path) as conn:
                conn.row_factory = sqlite3.Row
                cursor = conn.execute(
                    """
                    SELECT data FROM messages
                    WHERE session_id = ?
                    ORDER BY timestamp ASC
                    LIMIT ? OFFSET ?
                    """,
                    (session_id, limit, offset),
                )

                messages = []
                for row in cursor.fetchall():
                    messages.append(json.loads(row["data"]))

                return messages
        except sqlite3.Error as e:
            logger.error(f"Failed to get messages: {e}")
            return []

    def cleanup_old_sessions(self, days: int = 30) -> int:
        """Clean up sessions older than specified days."""
        try:
            with sqlite3.connect(self.db_path) as conn:
                cursor = conn.execute(
                    f"""
                    DELETE FROM sessions
                    WHERE last_activity < datetime('now', '-{days} days')
                    """
                )
                deleted_count = cursor.rowcount
                conn.commit()
                logger.info(f"Cleaned up {deleted_count} old sessions")
                return deleted_count
        except sqlite3.Error as e:
            logger.error(f"Failed to cleanup old sessions: {e}")
            return 0

    def get_session_stats(self) -> dict[str, Any]:
        """Get session statistics."""
        try:
            with sqlite3.connect(self.db_path) as conn:
                cursor = conn.execute("SELECT COUNT(*) as count FROM sessions")
                total_sessions = cursor.fetchone()["count"]

                cursor = conn.execute("""
                    SELECT mode, COUNT(*) as count
                    FROM sessions
                    GROUP BY mode
                """)
                sessions_by_mode = {
                    row["mode"]: row["count"] for row in cursor.fetchall()
                }

                cursor = conn.execute("""
                    SELECT COUNT(*) as count
                    FROM sessions
                    WHERE last_activity > datetime('now', '-1 day')
                """)
                active_sessions = cursor.fetchone()["count"]

                return {
                    "total_sessions": total_sessions,
                    "sessions_by_mode": sessions_by_mode,
                    "active_sessions_24h": active_sessions,
                }
        except sqlite3.Error as e:
            logger.error(f"Failed to get session stats: {e}")
            return {}
