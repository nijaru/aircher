#!/usr/bin/env python3
"""
Episodic Memory for Coding Agents

Tracks every action an agent takes, learns patterns over time, and provides
historical context for future decisions.

Research hypothesis: Agents that remember past actions perform 25-40% better
on similar tasks.
"""

import json
import sqlite3
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path


@dataclass
class Episode:
    """Single action taken by the agent"""

    id: int | None
    timestamp: datetime
    tool: str  # e.g., 'read_file', 'edit_file'
    file_path: str
    parameters: dict
    success: bool
    duration_ms: int
    context: str  # What the agent was trying to accomplish


@dataclass
class Pattern:
    """Learned pattern from historical episodes"""

    pattern_type: str  # e.g., 'co_edit', 'sequence', 'prerequisite'
    description: str
    files: list[str]
    confidence: float
    occurrences: int


class EpisodicMemory:
    """
    Persistent memory of agent actions and learned patterns.

    Stores:
    - Every tool call (episodes)
    - Files touched in each episode
    - Co-edit patterns (files edited together)
    - Task sequences (typical workflow patterns)
    """

    def __init__(self, db_path: Path = Path("episodic_memory.db")):
        self.db_path = db_path
        self.conn = sqlite3.connect(str(db_path))
        self._init_schema()

    def _init_schema(self):
        """Initialize database schema"""
        self.conn.executescript("""
            CREATE TABLE IF NOT EXISTS episodes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                tool TEXT NOT NULL,
                file_path TEXT NOT NULL,
                parameters TEXT,  -- JSON
                success INTEGER NOT NULL,
                duration_ms INTEGER,
                context TEXT,
                session_id TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_episodes_file ON episodes(file_path);
            CREATE INDEX IF NOT EXISTS idx_episodes_timestamp ON episodes(timestamp);
            CREATE INDEX IF NOT EXISTS idx_episodes_tool ON episodes(tool);

            CREATE TABLE IF NOT EXISTS episode_entities (
                episode_id INTEGER NOT NULL,
                entity_id TEXT NOT NULL,  -- Reference to knowledge graph node
                interaction_type TEXT NOT NULL,  -- 'read', 'modified', 'analyzed'
                FOREIGN KEY (episode_id) REFERENCES episodes(id)
            );

            CREATE INDEX IF NOT EXISTS idx_entities_episode ON episode_entities(episode_id);
            CREATE INDEX IF NOT EXISTS idx_entities_entity ON episode_entities(entity_id);

            CREATE TABLE IF NOT EXISTS learned_patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pattern_type TEXT NOT NULL,
                description TEXT,
                files TEXT,  -- JSON array
                confidence REAL,
                occurrences INTEGER,
                last_updated TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_patterns_type ON learned_patterns(pattern_type);
        """)
        self.conn.commit()

    def record_action(
        self,
        tool: str,
        file_path: str,
        success: bool,
        duration_ms: int = 0,
        parameters: dict | None = None,
        context: str = "",
        session_id: str = "default",
    ) -> int:
        """
        Record an agent action to episodic memory.

        Returns: Episode ID
        """
        cursor = self.conn.execute(
            """
            INSERT INTO episodes (timestamp, tool, file_path, parameters,
                                success, duration_ms, context, session_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
            (
                datetime.now().isoformat(),
                tool,
                file_path,
                json.dumps(parameters or {}),
                1 if success else 0,
                duration_ms,
                context,
                session_id,
            ),
        )
        self.conn.commit()
        return cursor.lastrowid

    def link_entity(self, episode_id: int, entity_id: str, interaction_type: str):
        """Link an episode to a knowledge graph entity"""
        self.conn.execute(
            """
            INSERT INTO episode_entities (episode_id, entity_id, interaction_type)
            VALUES (?, ?, ?)
        """,
            (episode_id, entity_id, interaction_type),
        )
        self.conn.commit()

    def get_file_history(self, file_path: str, limit: int = 10) -> list[Episode]:
        """
        Get recent actions involving a specific file.

        This answers: "What have I done with this file recently?"
        """
        cursor = self.conn.execute(
            """
            SELECT id, timestamp, tool, file_path, parameters,
                   success, duration_ms, context
            FROM episodes
            WHERE file_path = ?
            ORDER BY timestamp DESC
            LIMIT ?
        """,
            (file_path, limit),
        )

        episodes = []
        for row in cursor.fetchall():
            episodes.append(
                Episode(
                    id=row[0],
                    timestamp=datetime.fromisoformat(row[1]),
                    tool=row[2],
                    file_path=row[3],
                    parameters=json.loads(row[4]),
                    success=bool(row[5]),
                    duration_ms=row[6],
                    context=row[7],
                )
            )
        return episodes

    def find_co_edited_files(
        self, file_path: str, window_minutes: int = 30, min_occurrences: int = 2
    ) -> list[tuple[str, int]]:
        """
        Find files frequently edited together with the given file.

        "When I edit X, I usually also edit Y"

        Returns: List of (file_path, co_edit_count) tuples
        """
        # Find all episodes involving this file
        cursor = self.conn.execute(
            """
            SELECT timestamp
            FROM episodes
            WHERE file_path = ?
            ORDER BY timestamp DESC
            LIMIT 50
        """,
            (file_path,),
        )

        timestamps = [datetime.fromisoformat(row[0]) for row in cursor.fetchall()]

        # For each timestamp, find files edited within the window
        co_edits = {}
        for ts in timestamps:
            window_start = (ts - timedelta(minutes=window_minutes)).isoformat()
            window_end = (ts + timedelta(minutes=window_minutes)).isoformat()

            cursor = self.conn.execute(
                """
                SELECT DISTINCT file_path
                FROM episodes
                WHERE timestamp BETWEEN ? AND ?
                  AND file_path != ?
            """,
                (window_start, window_end, file_path),
            )

            for row in cursor.fetchall():
                other_file = row[0]
                co_edits[other_file] = co_edits.get(other_file, 0) + 1

        # Filter by minimum occurrences and sort
        result = [
            (file, count)
            for file, count in co_edits.items()
            if count >= min_occurrences
        ]
        result.sort(key=lambda x: x[1], reverse=True)
        return result

    def find_typical_workflow(self, context: str, limit: int = 10) -> list[Episode]:
        """
        Find typical workflow for a given context/task.

        "When working on authentication, I typically do X then Y then Z"
        """
        cursor = self.conn.execute(
            """
            SELECT id, timestamp, tool, file_path, parameters,
                   success, duration_ms, context
            FROM episodes
            WHERE context LIKE ?
            ORDER BY timestamp DESC
            LIMIT ?
        """,
            (f"%{context}%", limit * 3),
        )  # Get more to find patterns

        # Group consecutive episodes into sequences
        episodes = []
        for row in cursor.fetchall():
            episodes.append(
                Episode(
                    id=row[0],
                    timestamp=datetime.fromisoformat(row[1]),
                    tool=row[2],
                    file_path=row[3],
                    parameters=json.loads(row[4]),
                    success=bool(row[5]),
                    duration_ms=row[6],
                    context=row[7],
                )
            )

        return episodes[:limit]

    def learn_pattern(
        self, pattern_type: str, description: str, files: list[str], confidence: float
    ):
        """
        Store a learned pattern for future reference.

        Examples:
        - "When fixing auth bugs, check login.rs and middleware.rs"
        - "Payment features always involve payment.rs and billing.rs"
        """
        # Check if pattern already exists
        cursor = self.conn.execute(
            """
            SELECT id, occurrences FROM learned_patterns
            WHERE pattern_type = ? AND files = ?
        """,
            (pattern_type, json.dumps(sorted(files))),
        )

        existing = cursor.fetchone()

        if existing:
            # Update existing pattern
            self.conn.execute(
                """
                UPDATE learned_patterns
                SET occurrences = occurrences + 1,
                    confidence = ?,
                    last_updated = ?
                WHERE id = ?
            """,
                (confidence, datetime.now().isoformat(), existing[0]),
            )
        else:
            # Create new pattern
            self.conn.execute(
                """
                INSERT INTO learned_patterns
                    (pattern_type, description, files, confidence,
                     occurrences, last_updated)
                VALUES (?, ?, ?, ?, 1, ?)
            """,
                (
                    pattern_type,
                    description,
                    json.dumps(sorted(files)),
                    confidence,
                    datetime.now().isoformat(),
                ),
            )

        self.conn.commit()

    def get_patterns(self, pattern_type: str | None = None) -> list[Pattern]:
        """Get all learned patterns, optionally filtered by type"""
        if pattern_type:
            cursor = self.conn.execute(
                """
                SELECT pattern_type, description, files, confidence, occurrences
                FROM learned_patterns
                WHERE pattern_type = ?
                ORDER BY confidence DESC, occurrences DESC
            """,
                (pattern_type,),
            )
        else:
            cursor = self.conn.execute("""
                SELECT pattern_type, description, files, confidence, occurrences
                FROM learned_patterns
                ORDER BY confidence DESC, occurrences DESC
            """)

        patterns = []
        for row in cursor.fetchall():
            patterns.append(
                Pattern(
                    pattern_type=row[0],
                    description=row[1],
                    files=json.loads(row[2]),
                    confidence=row[3],
                    occurrences=row[4],
                )
            )
        return patterns

    def get_stats(self) -> dict:
        """Get memory statistics"""
        cursor = self.conn.execute("SELECT COUNT(*) FROM episodes")
        total_episodes = cursor.fetchone()[0]

        cursor = self.conn.execute("SELECT COUNT(*) FROM episodes WHERE success = 1")
        successful_episodes = cursor.fetchone()[0]

        cursor = self.conn.execute("SELECT COUNT(*) FROM learned_patterns")
        total_patterns = cursor.fetchone()[0]

        cursor = self.conn.execute("SELECT COUNT(DISTINCT file_path) FROM episodes")
        files_touched = cursor.fetchone()[0]

        return {
            "total_episodes": total_episodes,
            "successful_episodes": successful_episodes,
            "success_rate": successful_episodes / total_episodes
            if total_episodes > 0
            else 0,
            "learned_patterns": total_patterns,
            "files_touched": files_touched,
        }

    def close(self):
        """Close database connection"""
        self.conn.close()


# Import for co-edit time windows
from datetime import timedelta

if __name__ == "__main__":
    # Example usage
    memory = EpisodicMemory(Path("test_memory.db"))

    print("=== Episodic Memory Test ===\n")

    # Record some actions
    print("Recording actions...")
    e1 = memory.record_action(
        "read_file",
        "src/auth.rs",
        success=True,
        duration_ms=150,
        context="Investigating authentication bug",
    )

    e2 = memory.record_action(
        "edit_file",
        "src/auth.rs",
        success=True,
        duration_ms=500,
        parameters={"changes": "Fixed login validation"},
        context="Investigating authentication bug",
    )

    e3 = memory.record_action(
        "read_file",
        "src/middleware.rs",
        success=True,
        duration_ms=200,
        context="Investigating authentication bug",
    )

    # Query history
    print("\n=== File History ===")
    history = memory.get_file_history("src/auth.rs")
    for ep in history:
        print(f"{ep.timestamp}: {ep.tool} - {ep.context}")

    # Learn a pattern
    print("\n=== Learning Pattern ===")
    memory.learn_pattern(
        pattern_type="co_edit",
        description="Auth bugs typically involve auth.rs and middleware.rs",
        files=["src/auth.rs", "src/middleware.rs"],
        confidence=0.85,
    )

    # Get patterns
    print("\n=== Learned Patterns ===")
    patterns = memory.get_patterns()
    for p in patterns:
        print(f"{p.pattern_type}: {p.description}")
        print(f"  Files: {p.files}")
        print(f"  Confidence: {p.confidence}, Occurrences: {p.occurrences}")

    # Stats
    print("\n=== Memory Statistics ===")
    stats = memory.get_stats()
    for key, value in stats.items():
        print(f"{key}: {value}")

    memory.close()
    print("\nTest complete!")
