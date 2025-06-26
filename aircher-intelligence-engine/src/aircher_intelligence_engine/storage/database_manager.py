"""
Database manager for multi-database SQLite architecture.
"""

import asyncio
import logging
from pathlib import Path
from typing import Any, Dict

import aiosqlite

logger = logging.getLogger(__name__)


class DatabaseManager:
    """Manages multiple SQLite databases for different data types."""
    
    def __init__(self, data_dir: Path = None):
        self.data_dir = data_dir or Path.home() / ".aircher" / "data"
        self.databases = {}
    
    async def initialize(self):
        """Initialize database connections."""
        logger.info("Initializing database manager")
        self.data_dir.mkdir(parents=True, exist_ok=True)
        
        # TODO: Initialize actual database connections
        db_names = ["conversations", "knowledge", "file_index", "sessions"]
        for db_name in db_names:
            db_path = self.data_dir / f"{db_name}.db"
            # self.databases[db_name] = await aiosqlite.connect(str(db_path))
            logger.info(f"Database {db_name} ready at {db_path}")
    
    async def close(self):
        """Close all database connections."""
        for db_name, connection in self.databases.items():
            if connection:
                await connection.close()
                logger.info(f"Closed database: {db_name}")