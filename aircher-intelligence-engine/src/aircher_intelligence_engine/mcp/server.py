"""
Aircher Intelligence Engine MCP Server

Implements sophisticated AI intelligence capabilities with Warp-inspired patterns
for universal compatibility across all MCP clients.
"""

import asyncio
import logging
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

from mcp.server import Server
from mcp.server.models import InitializationOptions
from mcp.server.stdio import stdio_server
from mcp.types import (
    CallToolRequest,
    EmbeddedResource,
    ImageContent,
    ListResourcesRequest,
    ListToolsRequest,
    ReadResourceRequest,
    Resource,
    TextContent,
    Tool,
)

from ..tools.sophisticated_editing import SophisticatedEditingTool
from ..tools.interactive_commands import InteractiveCommandTool
from ..tools.context_intelligence import ContextIntelligenceTool
from ..tools.file_operations import FileOperationsTool
from ..context.window_manager import ContextWindowManager
from ..providers.fallback_manager import FallbackChainManager
from ..storage.database_manager import DatabaseManager

logger = logging.getLogger(__name__)


class AircherIntelligenceServer:
    """
    Universal MCP server providing intelligent context management and AI tools.
    
    Features:
    - Sophisticated editing with fuzzy matching and error recovery
    - Long-running interactive command support with PTY control
    - Model fallback chains (claude-4-sonnet → claude-3.7-sonnet → gemini-2.5-pro → gpt-4.1)
    - Context window management with smart file chunking
    - Cross-project learning and pattern recognition
    """
    
    def __init__(self, config_path: Optional[Path] = None):
        self.server = Server("aircher-intelligence-engine")
        self.config_path = config_path or Path.home() / ".aircher" / "config.toml"
        
        # Initialize core components
        self.context_manager = ContextWindowManager()
        self.fallback_manager = FallbackChainManager()
        self.database_manager = DatabaseManager()
        
        # Initialize tool handlers
        self.editing_tool = SophisticatedEditingTool(self.context_manager, self.database_manager)
        self.interactive_tool = InteractiveCommandTool(self.context_manager)
        self.context_tool = ContextIntelligenceTool(self.database_manager, self.context_manager)
        self.file_tool = FileOperationsTool(self.context_manager)
        
        # Setup MCP handlers
        self._setup_handlers()
    
    def _setup_handlers(self):
        """Set up MCP protocol handlers."""
        
        @self.server.list_tools()
        async def handle_list_tools() -> List[Tool]:
            """List all available tools."""
            return [
                # Core Warp-inspired tools for benchmark success
                Tool(
                    name="edit_files",
                    description="Edit multiple files with sophisticated fuzzy matching, Jaro-Winkler similarity, and detailed error recovery",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "edits": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "file_path": {"type": "string"},
                                        "search_text": {"type": "string"},
                                        "replace_text": {"type": "string"},
                                        "fuzzy_matching": {"type": "boolean", "default": True},
                                        "context_lines": {"type": "integer", "default": 3}
                                    },
                                    "required": ["file_path", "search_text", "replace_text"]
                                }
                            },
                            "detailed_errors": {"type": "boolean", "default": True}
                        },
                        "required": ["edits"]
                    }
                ),
                Tool(
                    name="create_file",
                    description="Create new file with content and automatic directory creation",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "file_path": {"type": "string"},
                            "content": {"type": "string"},
                            "create_directories": {"type": "boolean", "default": True}
                        },
                        "required": ["file_path", "content"]
                    }
                ),
                Tool(
                    name="search_files",
                    description="Search files with scrollable results and smart chunking",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "query": {"type": "string"},
                            "file_patterns": {"type": "array", "items": {"type": "string"}},
                            "max_lines_per_file": {"type": "integer", "default": 100},
                            "context_lines": {"type": "integer", "default": 3}
                        },
                        "required": ["query"]
                    }
                ),
                Tool(
                    name="run_command",
                    description="Execute shell commands with long-running and interactive support",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "command": {"type": "string"},
                            "working_directory": {"type": "string"},
                            "interactive": {"type": "boolean", "default": False},
                            "timeout": {"type": "integer", "default": 30}
                        },
                        "required": ["command"]
                    }
                ),
                # Advanced intelligence tools
                Tool(
                    name="project_analyze",
                    description="Comprehensive project analysis with intelligent insights",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "project_path": {"type": "string"},
                            "analysis_depth": {"type": "string", "enum": ["shallow", "medium", "deep"], "default": "medium"}
                        },
                        "required": ["project_path"]
                    }
                ),
                Tool(
                    name="context_score_files",
                    description="Score file relevance for intelligent context assembly",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "files": {"type": "array", "items": {"type": "string"}},
                            "task_context": {"type": "string"},
                            "max_files": {"type": "integer", "default": 10}
                        },
                        "required": ["files", "task_context"]
                    }
                ),
                Tool(
                    name="cross_project_insights",
                    description="Extract insights and patterns across multiple projects",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "project_paths": {"type": "array", "items": {"type": "string"}},
                            "insight_type": {"type": "string", "enum": ["patterns", "dependencies", "success_factors"]}
                        },
                        "required": ["project_paths", "insight_type"]
                    }
                )
            ]
        
        @self.server.call_tool()
        async def handle_call_tool(name: str, arguments: Dict[str, Any]) -> List[Union[TextContent, ImageContent, EmbeddedResource]]:
            """Handle tool execution with sophisticated error handling."""
            try:
                logger.info(f"Executing tool: {name} with arguments: {arguments}")
                
                if name == "edit_files":
                    result = await self.editing_tool.edit_files(**arguments)
                elif name == "create_file":
                    result = await self.file_tool.create_file(**arguments)
                elif name == "search_files":
                    result = await self.file_tool.search_files(**arguments)
                elif name == "run_command":
                    result = await self.interactive_tool.run_command(**arguments)
                elif name == "project_analyze":
                    result = await self.context_tool.project_analyze(**arguments)
                elif name == "context_score_files":
                    result = await self.context_tool.context_score_files(**arguments)
                elif name == "cross_project_insights":
                    result = await self.context_tool.cross_project_insights(**arguments)
                else:
                    raise ValueError(f"Unknown tool: {name}")
                
                return [TextContent(type="text", text=str(result))]
                
            except Exception as e:
                logger.error(f"Tool execution failed: {name} - {str(e)}")
                error_result = {
                    "success": False,
                    "error": str(e),
                    "tool": name,
                    "recovery_suggestions": self._get_recovery_suggestions(name, e)
                }
                return [TextContent(type="text", text=str(error_result))]
        
        @self.server.list_resources()
        async def handle_list_resources() -> List[Resource]:
            """List available resources."""
            return [
                Resource(
                    uri="aircher://config",
                    name="Aircher Configuration",
                    description="Current Aircher Intelligence Engine configuration",
                    mimeType="application/json"
                ),
                Resource(
                    uri="aircher://status",
                    name="System Status",
                    description="Current system status and health metrics",
                    mimeType="application/json"
                )
            ]
        
        @self.server.read_resource()
        async def handle_read_resource(uri: str) -> str:
            """Read resource content."""
            if uri == "aircher://config":
                return await self._get_config_resource()
            elif uri == "aircher://status":
                return await self._get_status_resource()
            else:
                raise ValueError(f"Unknown resource: {uri}")
    
    def _get_recovery_suggestions(self, tool_name: str, error: Exception) -> List[str]:
        """Get recovery suggestions for tool failures."""
        suggestions = []
        
        if "file not found" in str(error).lower():
            suggestions.append("Check if the file path exists and is accessible")
            suggestions.append("Try using absolute paths instead of relative paths")
        
        if "permission denied" in str(error).lower():
            suggestions.append("Check file permissions and ownership")
            suggestions.append("Ensure the process has read/write access to the file")
        
        if tool_name == "edit_files":
            suggestions.extend([
                "Enable fuzzy matching if exact text match fails",
                "Check for whitespace differences in search text",
                "Use smaller, more unique search strings"
            ])
        
        return suggestions
    
    async def _get_config_resource(self) -> str:
        """Get configuration resource."""
        # TODO: Implement configuration reading
        return '{"status": "Configuration reading not implemented yet"}'
    
    async def _get_status_resource(self) -> str:
        """Get status resource."""
        # TODO: Implement status monitoring
        return '{"status": "healthy", "version": "0.1.0"}'
    
    async def run(self):
        """Run the MCP server."""
        # Initialize database
        await self.database_manager.initialize()
        
        # Initialize providers
        await self.fallback_manager.initialize()
        
        logger.info("Starting Aircher Intelligence Engine MCP Server...")
        
        # Run server with stdio transport
        async with stdio_server() as (read_stream, write_stream):
            await self.server.run(
                read_stream,
                write_stream,
                InitializationOptions(
                    server_name="aircher-intelligence-engine",
                    server_version="0.1.0",
                    capabilities=self.server.get_capabilities(
                        notification_options=None,
                        experimental_capabilities=None,
                    ),
                ),
            )


if __name__ == "__main__":
    # Configure logging
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        handlers=[
            logging.FileHandler(Path.home() / ".aircher" / "logs" / "mcp-server.log"),
            logging.StreamHandler(sys.stderr)
        ]
    )
    
    # Create logs directory if it doesn't exist
    log_dir = Path.home() / ".aircher" / "logs"
    log_dir.mkdir(parents=True, exist_ok=True)
    
    # Run server
    server = AircherIntelligenceServer()
    asyncio.run(server.run())