"""File operation tools for Aircher."""

import shutil
from pathlib import Path
from typing import Any

from loguru import logger

from ..tools import BaseTool, ToolInput, ToolOutput


class ReadFileTool(BaseTool):
    """Tool for reading file contents."""

    def __init__(self):
        super().__init__(name="read_file", description="Read the contents of a file")

    def get_input_schema(self) -> ToolInput:
        """Get input schema for read file tool."""
        return ToolInput(
            name=self.name,
            description=self.description,
            parameters={
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read",
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Line number to start reading from (0-based)",
                        "default": 0,
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Number of lines to read",
                        "default": 2000,
                    },
                },
                "required": ["path"],
            },
        )

    async def execute(self, **kwargs: Any) -> ToolOutput:
        """Execute file read operation."""
        try:
            path = kwargs["path"]
            offset = kwargs.get("offset", 0)
            limit = kwargs.get("limit", 2000)

            file_path = Path(path)

            # Security check - prevent path traversal
            if not file_path.resolve().is_relative_to(Path.cwd().resolve()):
                return ToolOutput(
                    success=False,
                    error="Access denied: path outside current working directory",
                )

            if not file_path.exists():
                return ToolOutput(success=False, error=f"File not found: {path}")

            if not file_path.is_file():
                return ToolOutput(success=False, error=f"Path is not a file: {path}")

            # Read file content
            with open(file_path, encoding="utf-8", errors="replace") as f:
                lines = f.readlines()

            # Apply offset and limit
            selected_lines = lines[offset : offset + limit]

            # Add line numbers
            numbered_lines = []
            for i, line in enumerate(selected_lines, start=offset + 1):
                numbered_lines.append(f"{i:6d}\t{line.rstrip()}")

            content = "\n".join(numbered_lines)

            metadata = {
                "total_lines": len(lines),
                "lines_returned": len(selected_lines),
                "offset": offset,
                "limit": limit,
                "file_size": file_path.stat().st_size,
                "file_type": file_path.suffix,
            }

            return ToolOutput(success=True, data=content, metadata=metadata)

        except Exception as e:
            logger.error(f"Error reading file: {e}")
            return ToolOutput(success=False, error=f"Failed to read file: {str(e)}")


class WriteFileTool(BaseTool):
    """Tool for writing file contents."""

    def __init__(self):
        super().__init__(name="write_file", description="Write content to a file")

    def get_input_schema(self) -> ToolInput:
        """Get input schema for write file tool."""
        return ToolInput(
            name=self.name,
            description=self.description,
            parameters={
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to write",
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write to the file",
                    },
                    "create_dirs": {
                        "type": "boolean",
                        "description": "Create parent directories if they don't exist",
                        "default": True,
                    },
                    "backup": {
                        "type": "boolean",
                        "description": "Create backup of existing file",
                        "default": True,
                    },
                },
                "required": ["path", "content"],
            },
        )

    async def execute(self, **kwargs: Any) -> ToolOutput:
        """Execute file write operation."""
        try:
            path = kwargs["path"]
            content = kwargs["content"]
            create_dirs = kwargs.get("create_dirs", True)
            backup = kwargs.get("backup", True)

            file_path = Path(path)

            # Security check - prevent path traversal
            if not file_path.resolve().is_relative_to(Path.cwd().resolve()):
                return ToolOutput(
                    success=False,
                    error="Access denied: path outside current working directory",
                )

            # Create parent directories if needed
            if create_dirs:
                file_path.parent.mkdir(parents=True, exist_ok=True)

            # Create backup if file exists and backup is requested
            backup_path = None
            if file_path.exists() and backup:
                backup_path = file_path.with_suffix(file_path.suffix + ".bak")
                shutil.copy2(file_path, backup_path)

            # Write content
            with open(file_path, "w", encoding="utf-8") as f:
                f.write(content)

            metadata = {
                "bytes_written": len(content.encode("utf-8")),
                "backup_created": backup_path is not None,
                "backup_path": str(backup_path) if backup_path else None,
            }

            return ToolOutput(
                success=True,
                data=f"Successfully wrote {len(content)} characters to {path}",
                metadata=metadata,
            )

        except Exception as e:
            logger.error(f"Error writing file: {e}")
            return ToolOutput(success=False, error=f"Failed to write file: {str(e)}")


class ListDirectoryTool(BaseTool):
    """Tool for listing directory contents."""

    def __init__(self):
        super().__init__(
            name="list_directory", description="List contents of a directory"
        )

    def get_input_schema(self) -> ToolInput:
        """Get input schema for list directory tool."""
        return ToolInput(
            name=self.name,
            description=self.description,
            parameters={
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the directory to list",
                        "default": ".",
                    },
                    "show_hidden": {
                        "type": "boolean",
                        "description": "Show hidden files and directories",
                        "default": False,
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "List directories recursively",
                        "default": False,
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum depth for recursive listing",
                        "default": 3,
                    },
                },
                "required": [],
            },
        )

    async def execute(self, **kwargs: Any) -> ToolOutput:
        """Execute directory listing operation."""
        try:
            path = kwargs.get("path", ".")
            show_hidden = kwargs.get("show_hidden", False)
            recursive = kwargs.get("recursive", False)
            max_depth = kwargs.get("max_depth", 3)

            dir_path = Path(path)

            # Security check - prevent path traversal
            if not dir_path.resolve().is_relative_to(Path.cwd().resolve()):
                return ToolOutput(
                    success=False,
                    error="Access denied: path outside current working directory",
                )

            if not dir_path.exists():
                return ToolOutput(success=False, error=f"Directory not found: {path}")

            if not dir_path.is_dir():
                return ToolOutput(
                    success=False, error=f"Path is not a directory: {path}"
                )

            def list_dir(
                directory: Path, current_depth: int = 0
            ) -> list[dict[str, Any]]:
                """Recursively list directory contents."""
                items = []

                try:
                    for item in directory.iterdir():
                        # Skip hidden files unless requested
                        if not show_hidden and item.name.startswith("."):
                            continue

                        stat = item.stat()
                        item_info = {
                            "name": item.name,
                            "path": str(item.relative_to(Path.cwd())),
                            "type": "directory" if item.is_dir() else "file",
                            "size": stat.st_size,
                            "modified": stat.st_mtime,
                            "is_symlink": item.is_symlink(),
                        }

                        items.append(item_info)

                        # Recurse into subdirectories if requested
                        if recursive and item.is_dir() and current_depth < max_depth:
                            sub_items = list_dir(item, current_depth + 1)
                            items.extend(sub_items)

                except PermissionError:
                    logger.warning(f"Permission denied for directory: {directory}")

                return items

            items = list_dir(dir_path)

            # Sort items: directories first, then files, both alphabetically
            items.sort(key=lambda x: (x["type"] != "directory", x["name"].lower()))

            metadata = {
                "total_items": len(items),
                "directories": len([i for i in items if i["type"] == "directory"]),
                "files": len([i for i in items if i["type"] == "file"]),
                "path": str(dir_path.resolve()),
            }

            return ToolOutput(success=True, data=items, metadata=metadata)

        except Exception as e:
            logger.error(f"Error listing directory: {e}")
            return ToolOutput(
                success=False, error=f"Failed to list directory: {str(e)}"
            )


class SearchFilesTool(BaseTool):
    """Tool for searching files using ripgrep."""

    def __init__(self, bash_tool):
        super().__init__(
            name="search_files",
            description="Search for files and content using ripgrep",
        )
        self.bash_tool = bash_tool

    def get_input_schema(self) -> ToolInput:
        """Get input schema for search files tool."""
        return ToolInput(
            name=self.name,
            description=self.description,
            parameters={
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Search pattern (regex supported)",
                    },
                    "path": {
                        "type": "string",
                        "description": "Path to search in",
                        "default": ".",
                    },
                    "file_type": {
                        "type": "string",
                        "description": "File type filter (e.g., 'py', 'rs', 'js')",
                    },
                    "case_sensitive": {
                        "type": "boolean",
                        "description": "Case sensitive search",
                        "default": False,
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results to return",
                        "default": 100,
                    },
                },
                "required": ["pattern"],
            },
        )

    async def execute(self, **kwargs: Any) -> ToolOutput:
        """Execute file search operation."""
        try:
            pattern = kwargs["pattern"]
            path = kwargs.get("path", ".")
            file_type = kwargs.get("file_type")
            case_sensitive = kwargs.get("case_sensitive", False)
            max_results = kwargs.get("max_results", 100)

            # Build ripgrep command
            args = ["--json", "--max-count", str(max_results)]

            if not case_sensitive:
                args.append("--ignore-case")

            if file_type:
                args.extend(["--type", file_type])

            args.extend([pattern, path])

            result = self.bash_tool.ripgrep(pattern, path, args)

            if not result.success:
                return ToolOutput(
                    success=False, error=f"Search failed: {result.stderr}"
                )

            # Parse ripgrep JSON output
            import json

            matches = []
            for line in result.stdout.strip().split("\n"):
                if line:
                    try:
                        match_data = json.loads(line)
                        matches.append(match_data)
                    except json.JSONDecodeError:
                        continue

            metadata = {
                "pattern": pattern,
                "path": path,
                "matches_found": len(matches),
                "case_sensitive": case_sensitive,
                "file_type": file_type,
            }

            return ToolOutput(success=True, data=matches, metadata=metadata)

        except Exception as e:
            logger.error(f"Error searching files: {e}")
            return ToolOutput(success=False, error=f"Failed to search files: {str(e)}")
