# Aircher Intelligence Engine MCP Server Technical Specification

## Overview

The **Aircher Intelligence Engine** is a universal MCP server that provides intelligent context management for AI-powered development. Built in Python and deployed via uvx, it offers cross-project learning, file relevance scoring, task detection, and intelligent context assembly for any MCP-compatible AI tool.

## Architecture Principles

### Universal Intelligence Layer
- **Universal Compatibility**: Works with Claude Desktop, VS Code, Cursor, and any MCP-compatible tool
- **Cross-Project Learning**: Pattern recognition and insights across entire codebase
- **Intelligent Context Management**: AI-driven file relevance scoring and task detection
- **Real-Time Analysis**: Live project analysis and dependency tracking
- **Secure by Design**: Comprehensive permissions and sandboxing

### Python-First Design
- **Rapid Development**: Fast iteration for AI algorithms and intelligence logic
- **Rich Ecosystem**: Leverage Python's excellent libraries for file analysis, AI integration
- **uvx Deployment**: Modern Python CLI tool deployment with automatic dependency management
- **Async Architecture**: Modern async/await for high-performance concurrent operations

## Core MCP Tools

The Aircher Intelligence Engine provides these core tools to AI assistants:

```python
AIRCHER_MCP_TOOLS = {
    "edit_files": {
        "description": "Multi-file search and replace editing with fuzzy matching and error recovery",
        "parameters": {
            "edits": "List of file edits with search/replace patterns",
            "absolute_paths": "Use absolute paths to prevent directory confusion (default: true)",
            "fuzzy_matching": "Enable fuzzy string matching for inexact matches (default: true)",
            "detailed_errors": "Provide detailed failure information for recovery (default: true)"
        },
        "returns": "EditResults with success/failure status and recovery guidance"
    },
    
    "create_file": {
        "description": "Create new files with path validation and directory creation",
        "parameters": {
            "file_path": "Absolute path for new file",
            "content": "File content to write",
            "create_dirs": "Create parent directories if needed (default: true)",
            "overwrite": "Allow overwriting existing files (default: false)"
        },
        "returns": "FileCreationResult with validation and creation status"
    },
    
    "search_files": {
        "description": "Intelligent file search with context window management",
        "parameters": {
            "query": "Search query or pattern",
            "file_patterns": "File type patterns to search within",
            "max_lines_per_file": "Maximum lines to show per file (default: 100)",
            "show_context": "Lines of context around matches (default: 3)",
            "enable_scrolling": "Allow incremental result viewing (default: true)"
        },
        "returns": "SearchResults with chunked, scrollable results"
    },
    
    "run_command": {
        "description": "Execute shell commands with long-running command support and safety analysis",
        "parameters": {
            "command": "Shell command to execute",
            "interactive": "Enable interactive mode for REPLs, vim, etc. (default: false)",
            "safety_level": "Command safety assessment (safe, moderate, dangerous, system)",
            "timeout": "Command timeout in seconds (default: 30)",
            "working_directory": "Working directory for command execution"
        },
        "returns": "CommandResult with output, safety assessment, and interaction capabilities"
    },
    
    "project_analyze": {
        "description": "Comprehensive project structure analysis and component detection",
        "parameters": {
            "path": "Project root directory path",
            "include_patterns": "File patterns to include (optional)",
            "exclude_patterns": "File patterns to exclude (optional)",
            "depth": "Maximum directory depth (default: 10)"
        },
        "returns": "ProjectAnalysis with components, structure, technologies, dependencies"
    },
    
    "context_score_files": {
        "description": "AI-driven file relevance scoring for current development task",
        "parameters": {
            "files": "List of file paths to score",
            "task_context": "Current task description or context",
            "task_type": "Type of task (debugging, feature, refactor, etc.)",
            "max_files": "Maximum number of files to return (default: 20)"
        },
        "returns": "Scored list of files with relevance scores and reasoning"
    },
    
    "task_detect": {
        "description": "Automatic detection of current development task type and context",
        "parameters": {
            "project_path": "Project root directory",
            "recent_changes": "Recent git changes (optional)",
            "active_files": "Currently modified files (optional)",
            "user_intent": "User's stated intent or query (optional)"
        },
        "returns": "TaskDetection with type, confidence, relevant files, suggested actions"
    },
    
    "dependency_graph": {
        "description": "Build and query file relationship networks and dependency analysis",
        "parameters": {
            "project_path": "Project root directory", 
            "target_files": "Specific files to analyze dependencies for (optional)",
            "graph_type": "Type of graph: imports, calls, tests, docs (default: all)",
            "max_depth": "Maximum dependency depth (default: 5)"
        },
        "returns": "DependencyGraph with relationships, clusters, critical paths"
    },
    
    "success_patterns": {
        "description": "Learn from and apply historical success patterns across projects",
        "parameters": {
            "task_type": "Type of task to find patterns for",
            "project_context": "Current project context and technologies",
            "similar_projects": "Include patterns from similar projects (default: true)",
            "confidence_threshold": "Minimum confidence for returned patterns (default: 0.7)"
        },
        "returns": "SuccessPatterns with applicable solutions, anti-patterns, recommendations"
    },
    
    "cross_project_insights": {
        "description": "Apply learnings and insights from similar contexts across projects",
        "parameters": {
            "current_context": "Current development context or problem",
            "project_technologies": "Technologies used in current project",
            "include_worktrees": "Include insights from other git worktrees (default: true)",
            "similarity_threshold": "Minimum similarity score (default: 0.6)"
        },
        "returns": "CrossProjectInsights with related solutions, patterns, recommendations"
    },
    
    "smart_context_assembly": {
        "description": "Optimize context for AI tools based on token limits and relevance",
        "parameters": {
            "target_files": "Files to consider for context",
            "task_description": "Description of current task",
            "token_limit": "Maximum tokens for assembled context",
            "model_name": "Target model for token estimation (optional)",
            "preserve_structure": "Maintain logical code structure (default: true)",
            "chunking_strategy": "How to chunk large files (lines, functions, semantic)",
            "enable_scrolling": "Allow incremental context loading (default: true)"
        },
        "returns": "SmartContext with optimized file selection, chunking, and token usage"
    },
    
    "benchmark_validate": {
        "description": "Validate solutions against test harnesses like SWE-bench and Terminal-Bench",
        "parameters": {
            "test_specification": "Test specification or problem description",
            "solution_approach": "Proposed solution approach or implementation",
            "benchmark_type": "Type of benchmark (swe-bench, terminal-bench, custom)",
            "validation_level": "Depth of validation (syntax, logic, integration)"
        },
        "returns": "ValidationResult with pass/fail status, feedback, and improvement suggestions"
    }
}
```

## Python Implementation Architecture

### Core Server Structure

```python
#!/usr/bin/env python3
"""
Aircher Intelligence Engine MCP Server

Universal intelligent context management for AI-powered development.
Provides cross-project learning, file relevance scoring, task detection,
and intelligent context assembly for any MCP-compatible AI tool.
"""

import asyncio
import json
import logging
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Union
from dataclasses import dataclass, asdict
import argparse

# MCP Protocol
from mcp import Server, types
from mcp.server import stdio_server

# Core Intelligence Components
from .intelligence import (
    ProjectAnalyzer,
    FileRelevanceEngine, 
    TaskDetector,
    DependencyAnalyzer,
    SuccessPatternLearner,
    CrossProjectInsights,
    ContextAssembler
)

# Database and Storage
from .storage import AircherDatabase, ProjectContext
from .config import AircherConfig

# Utilities
from .utils import (
    tokenize_content,
    estimate_tokens,
    sanitize_file_path,
    validate_project_path
)

logger = logging.getLogger("aircher-intelligence")

@dataclass
class ProjectAnalysis:
    """Comprehensive project analysis result."""
    project_path: str
    technologies: List[str]
    components: Dict[str, Any]
    structure: Dict[str, Any]
    dependencies: Dict[str, List[str]]
    entry_points: List[str]
    test_files: List[str]
    config_files: List[str]
    documentation: List[str]
    metrics: Dict[str, Union[int, float]]
    confidence_score: float
    analysis_timestamp: str

@dataclass  
class FileRelevanceScore:
    """File relevance scoring result."""
    file_path: str
    relevance_score: float
    confidence: float
    reasoning: str
    task_alignment: float
    dependency_importance: float
    recency_factor: float
    success_correlation: float

@dataclass
class EditResult:
    """Result of a file edit operation."""
    file_path: str
    success: bool
    error_message: Optional[str] = None
    recovery_suggestion: Optional[str] = None
    fuzzy_match_used: bool = False
    original_search: str = ""
    actual_match: str = ""
    
@dataclass
class SearchResult:
    """File search result with context management."""
    file_path: str
    matches: List[Dict[str, Any]]
    total_matches: int
    lines_shown: int
    total_lines: int
    has_more: bool
    chunk_index: int
    
@dataclass
class CommandResult:
    """Shell command execution result."""
    command: str
    exit_code: int
    stdout: str
    stderr: str
    safety_level: str
    is_interactive: bool
    interaction_capabilities: List[str]
    execution_time: float

@dataclass
class TaskDetection:
    """Task detection analysis result."""
    task_type: str  # debugging, feature, refactor, documentation, testing, maintenance
    confidence: float
    description: str
    relevant_files: List[str]
    suggested_actions: List[str]
    context_indicators: Dict[str, float]
    detection_reasoning: str
    todo_list: List[str]  # Auto-generated todo items based on Warp's planning success

@dataclass
class DependencyGraph:
    """Dependency analysis result."""
    nodes: Dict[str, Dict[str, Any]]
    edges: List[Dict[str, Any]]
    clusters: List[List[str]]
    critical_paths: List[List[str]]
    metrics: Dict[str, Union[int, float]]
    graph_type: str

class AircherIntelligenceServer:
    """Main MCP server for Aircher Intelligence Engine."""
    
    def __init__(self, config_path: Optional[str] = None):
        self.config = AircherConfig.load(config_path)
        self.database = AircherDatabase(self.config.database_path)
        
        # Core intelligence components
        self.project_analyzer = ProjectAnalyzer(self.config, self.database)
        self.relevance_engine = FileRelevanceEngine(self.config, self.database)
        self.task_detector = TaskDetector(self.config, self.database)
        self.dependency_analyzer = DependencyAnalyzer(self.config)
        self.pattern_learner = SuccessPatternLearner(self.config, self.database)
        self.cross_insights = CrossProjectInsights(self.config, self.database)
        self.context_assembler = ContextAssembler(self.config)
        
        # MCP Server setup
        self.server = Server("aircher-intelligence")
        self._register_tools()
        
    def _register_tools(self):
        """Register all MCP tools with the server."""
        
        @self.server.list_tools()
        async def list_tools() -> List[types.Tool]:
            """List all available tools."""
            return [
                types.Tool(
                    name="edit_files",
                    description="Multi-file search and replace editing with fuzzy matching and error recovery",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "edits": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "file_path": {"type": "string", "description": "Absolute path to file"},
                                        "search": {"type": "string", "description": "Text to search for"},
                                        "replace": {"type": "string", "description": "Replacement text"}
                                    },
                                    "required": ["file_path", "search", "replace"]
                                },
                                "description": "List of file edits to perform"
                            },
                            "fuzzy_matching": {"type": "boolean", "description": "Enable fuzzy string matching", "default": True},
                            "detailed_errors": {"type": "boolean", "description": "Provide detailed failure information", "default": True}
                        },
                        "required": ["edits"]
                    }
                ),
                
                types.Tool(
                    name="create_file",
                    description="Create new files with path validation and directory creation",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "file_path": {"type": "string", "description": "Absolute path for new file"},
                            "content": {"type": "string", "description": "File content to write"},
                            "create_dirs": {"type": "boolean", "description": "Create parent directories", "default": True},
                            "overwrite": {"type": "boolean", "description": "Allow overwriting existing files", "default": False}
                        },
                        "required": ["file_path", "content"]
                    }
                ),
                
                types.Tool(
                    name="search_files",
                    description="Intelligent file search with context window management",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "query": {"type": "string", "description": "Search query or pattern"},
                            "file_patterns": {"type": "array", "items": {"type": "string"}, "description": "File patterns to search"},
                            "max_lines_per_file": {"type": "integer", "description": "Maximum lines per file", "default": 100},
                            "show_context": {"type": "integer", "description": "Lines of context around matches", "default": 3},
                            "enable_scrolling": {"type": "boolean", "description": "Allow incremental viewing", "default": True}
                        },
                        "required": ["query"]
                    }
                ),
                
                types.Tool(
                    name="run_command",
                    description="Execute shell commands with long-running support and safety analysis",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "command": {"type": "string", "description": "Shell command to execute"},
                            "interactive": {"type": "boolean", "description": "Enable interactive mode", "default": False},
                            "safety_level": {"type": "string", "enum": ["safe", "moderate", "dangerous", "system"], "description": "Command safety level"},
                            "timeout": {"type": "integer", "description": "Command timeout in seconds", "default": 30},
                            "working_directory": {"type": "string", "description": "Working directory for execution"}
                        },
                        "required": ["command"]
                    }
                ),
                
                types.Tool(
                    name="project_analyze",
                    description="Comprehensive project structure analysis and component detection",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "Project root directory path"},
                            "include_patterns": {"type": "array", "items": {"type": "string"}, "description": "File patterns to include"},
                            "exclude_patterns": {"type": "array", "items": {"type": "string"}, "description": "File patterns to exclude"},
                            "depth": {"type": "integer", "description": "Maximum directory depth", "default": 10}
                        },
                        "required": ["path"]
                    }
                ),
                
                types.Tool(
                    name="context_score_files", 
                    description="AI-driven file relevance scoring for current development task",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "files": {"type": "array", "items": {"type": "string"}, "description": "List of file paths to score"},
                            "task_context": {"type": "string", "description": "Current task description or context"},
                            "task_type": {"type": "string", "enum": ["debugging", "feature", "refactor", "documentation", "testing", "maintenance"], "description": "Type of development task"},
                            "max_files": {"type": "integer", "description": "Maximum number of files to return", "default": 20}
                        },
                        "required": ["files", "task_context"]
                    }
                ),
                
                types.Tool(
                    name="task_detect",
                    description="Automatic detection of current development task type and context", 
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "project_path": {"type": "string", "description": "Project root directory"},
                            "recent_changes": {"type": "array", "items": {"type": "string"}, "description": "Recent git changes"},
                            "active_files": {"type": "array", "items": {"type": "string"}, "description": "Currently modified files"},
                            "user_intent": {"type": "string", "description": "User's stated intent or query"}
                        },
                        "required": ["project_path"]
                    }
                ),
                
                types.Tool(
                    name="dependency_graph",
                    description="Build and query file relationship networks and dependency analysis",
                    inputSchema={
                        "type": "object", 
                        "properties": {
                            "project_path": {"type": "string", "description": "Project root directory"},
                            "target_files": {"type": "array", "items": {"type": "string"}, "description": "Specific files to analyze"},
                            "graph_type": {"type": "string", "enum": ["imports", "calls", "tests", "docs", "all"], "description": "Type of dependency graph", "default": "all"},
                            "max_depth": {"type": "integer", "description": "Maximum dependency depth", "default": 5}
                        },
                        "required": ["project_path"]
                    }
                ),
                
                types.Tool(
                    name="success_patterns",
                    description="Learn from and apply historical success patterns across projects",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "task_type": {"type": "string", "description": "Type of task to find patterns for"},
                            "project_context": {"type": "object", "description": "Current project context and technologies"},
                            "similar_projects": {"type": "boolean", "description": "Include patterns from similar projects", "default": True},
                            "confidence_threshold": {"type": "number", "description": "Minimum confidence for patterns", "default": 0.7}
                        },
                        "required": ["task_type", "project_context"]
                    }
                ),
                
                types.Tool(
                    name="cross_project_insights",
                    description="Apply learnings and insights from similar contexts across projects",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "current_context": {"type": "string", "description": "Current development context or problem"},
                            "project_technologies": {"type": "array", "items": {"type": "string"}, "description": "Technologies used in current project"},
                            "include_worktrees": {"type": "boolean", "description": "Include insights from other git worktrees", "default": True},
                            "similarity_threshold": {"type": "number", "description": "Minimum similarity score", "default": 0.6}
                        },
                        "required": ["current_context", "project_technologies"]
                    }
                ),
                
                types.Tool(
                    name="smart_context_assembly",
                    description="Optimize context for AI tools based on token limits and relevance",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "target_files": {"type": "array", "items": {"type": "string"}, "description": "Files to consider for context"},
                            "task_description": {"type": "string", "description": "Description of current task"},
                            "token_limit": {"type": "integer", "description": "Maximum tokens for assembled context"},
                            "model_name": {"type": "string", "description": "Target model for token estimation"},
                            "preserve_structure": {"type": "boolean", "description": "Maintain logical code structure", "default": True},
                            "chunking_strategy": {"type": "string", "enum": ["lines", "functions", "semantic"], "description": "How to chunk large files", "default": "semantic"},
                            "enable_scrolling": {"type": "boolean", "description": "Allow incremental context loading", "default": True}
                        },
                        "required": ["target_files", "task_description", "token_limit"]
                    }
                ),
                
                types.Tool(
                    name="benchmark_validate",
                    description="Validate solutions against test harnesses like SWE-bench and Terminal-Bench",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "test_specification": {"type": "string", "description": "Test specification or problem description"},
                            "solution_approach": {"type": "string", "description": "Proposed solution approach"},
                            "benchmark_type": {"type": "string", "enum": ["swe-bench", "terminal-bench", "custom"], "description": "Type of benchmark"},
                            "validation_level": {"type": "string", "enum": ["syntax", "logic", "integration"], "description": "Depth of validation", "default": "logic"}
                        },
                        "required": ["test_specification", "solution_approach"]
                    }
                )
            ]
        
        @self.server.call_tool()
        async def call_tool(name: str, arguments: Dict[str, Any]) -> List[types.TextContent]:
            """Handle tool execution with Warp-inspired reliability."""
            try:
                if name == "edit_files":
                    result = await self._edit_files(**arguments)
                elif name == "create_file":
                    result = await self._create_file(**arguments)
                elif name == "search_files":
                    result = await self._search_files(**arguments)
                elif name == "run_command":
                    result = await self._run_command(**arguments)
                elif name == "project_analyze":
                    result = await self._project_analyze(**arguments)
                elif name == "context_score_files":
                    result = await self._context_score_files(**arguments)
                elif name == "task_detect":
                    result = await self._task_detect(**arguments)
                elif name == "dependency_graph":
                    result = await self._dependency_graph(**arguments)
                elif name == "success_patterns":
                    result = await self._success_patterns(**arguments)
                elif name == "cross_project_insights":
                    result = await self._cross_project_insights(**arguments)
                elif name == "smart_context_assembly":
                    result = await self._smart_context_assembly(**arguments)
                elif name == "benchmark_validate":
                    result = await self._benchmark_validate(**arguments)
                else:
                    raise ValueError(f"Unknown tool: {name}")
                
                return [types.TextContent(
                    type="text",
                    text=json.dumps(result, indent=2, default=str)
                )]
                
            except Exception as e:
                logger.error(f"Tool {name} failed: {e}")
                return [types.TextContent(
                    type="text", 
                    text=json.dumps({"error": str(e), "tool": name}, indent=2)
                )]
    
    async def _edit_files(self, edits: List[Dict[str, str]], fuzzy_matching: bool = True, 
                         detailed_errors: bool = True) -> Dict[str, Any]:
        """Perform multi-file edits with Warp-inspired sophistication."""
        results = []
        
        for edit in edits:
            file_path = Path(edit["file_path"]).resolve()
            search_text = edit["search"]
            replace_text = edit["replace"]
            
            try:
                # Read file content
                async with aiofiles.open(file_path, 'r', encoding='utf-8') as f:
                    content = await f.read()
                
                # Try exact match first
                if search_text in content:
                    new_content = content.replace(search_text, replace_text)
                    success = True
                    match_used = search_text
                    fuzzy_used = False
                elif fuzzy_matching:
                    # Try indentation-agnostic match
                    match_used, success = self._fuzzy_string_replace(content, search_text, replace_text)
                    if success:
                        new_content = content.replace(match_used, replace_text)
                        fuzzy_used = True
                    else:
                        success = False
                        new_content = content
                        fuzzy_used = False
                else:
                    success = False
                    new_content = content
                    match_used = ""
                    fuzzy_used = False
                
                if success:
                    # Write updated content
                    async with aiofiles.open(file_path, 'w', encoding='utf-8') as f:
                        await f.write(new_content)
                    
                    results.append(EditResult(
                        file_path=str(file_path),
                        success=True,
                        fuzzy_match_used=fuzzy_used,
                        original_search=search_text,
                        actual_match=match_used
                    ))
                else:
                    # Provide detailed error information
                    error_msg, suggestion = self._analyze_edit_failure(content, search_text, file_path)
                    results.append(EditResult(
                        file_path=str(file_path),
                        success=False,
                        error_message=error_msg,
                        recovery_suggestion=suggestion,
                        original_search=search_text
                    ))
                    
            except Exception as e:
                results.append(EditResult(
                    file_path=str(file_path),
                    success=False,
                    error_message=f"File operation failed: {e}",
                    recovery_suggestion="Check file permissions and path validity",
                    original_search=search_text
                ))
        
        return {
            "results": [asdict(r) for r in results],
            "total_edits": len(edits),
            "successful_edits": sum(1 for r in results if r.success),
            "failed_edits": sum(1 for r in results if not r.success)
        }
    
    async def _create_file(self, file_path: str, content: str, create_dirs: bool = True, 
                          overwrite: bool = False) -> Dict[str, Any]:
        """Create new file with path validation."""
        file_path = Path(file_path).resolve()
        
        # Check if file exists
        if file_path.exists() and not overwrite:
            return {
                "success": False,
                "error": "File already exists",
                "suggestion": "Use overwrite=true or choose different path",
                "file_path": str(file_path)
            }
        
        try:
            # Create parent directories if needed
            if create_dirs:
                file_path.parent.mkdir(parents=True, exist_ok=True)
            
            # Write file
            async with aiofiles.open(file_path, 'w', encoding='utf-8') as f:
                await f.write(content)
            
            return {
                "success": True,
                "file_path": str(file_path),
                "size_bytes": len(content.encode('utf-8')),
                "created_dirs": create_dirs and not file_path.parent.exists()
            }
            
        except Exception as e:
            return {
                "success": False,
                "error": str(e),
                "suggestion": "Check file path validity and permissions",
                "file_path": str(file_path)
            }
    
    async def _search_files(self, query: str, file_patterns: Optional[List[str]] = None,
                           max_lines_per_file: int = 100, show_context: int = 3,
                           enable_scrolling: bool = True) -> Dict[str, Any]:
        """Intelligent file search with context management."""
        # Implementation for chunked file search with scrolling capability
        # This enables the "scrolling" pattern that Warp found effective
        pass
    
    async def _run_command(self, command: str, interactive: bool = False, 
                          safety_level: Optional[str] = None, timeout: int = 30,
                          working_directory: Optional[str] = None) -> Dict[str, Any]:
        """Execute shell command with safety analysis and interactive support."""
        # Implementation for command execution with long-running support
        # Based on Warp's successful long-running command architecture
        pass
    
    async def _benchmark_validate(self, test_specification: str, solution_approach: str,
                                 benchmark_type: str = "custom", validation_level: str = "logic") -> Dict[str, Any]:
        """Validate solutions against benchmark test harnesses."""
        # Implementation for SWE-bench and Terminal-Bench validation
        pass
    
    def _fuzzy_string_replace(self, content: str, search: str, replace: str) -> tuple[str, bool]:
        """Perform fuzzy string matching for inexact search patterns."""
        # Jaro-Winkler similarity implementation like Warp uses
        import difflib
        
        lines = content.split('\n')
        search_lines = search.split('\n')
        
        # Try to find best match using indentation-agnostic comparison
        best_match = None
        best_ratio = 0.0
        
        for i in range(len(lines) - len(search_lines) + 1):
            candidate_lines = lines[i:i + len(search_lines)]
            
            # Compare ignoring leading whitespace
            candidate_stripped = [line.strip() for line in candidate_lines]
            search_stripped = [line.strip() for line in search_lines]
            
            ratio = difflib.SequenceMatcher(None, search_stripped, candidate_stripped).ratio()
            
            if ratio > best_ratio and ratio > 0.8:  # 80% similarity threshold
                best_ratio = ratio
                best_match = '\n'.join(candidate_lines)
        
        if best_match:
            return best_match, True
        else:
            return "", False
    
    def _analyze_edit_failure(self, content: str, search_text: str, file_path: Path) -> tuple[str, str]:
        """Analyze why an edit failed and provide recovery suggestions."""
        # Detailed failure analysis like Warp provides
        if not Path(file_path).exists():
            return "File does not exist", "Check file path and ensure file exists"
        
        if search_text not in content:
            # Check for common issues
            lines = content.split('\n')
            search_lines = search_text.split('\n')
            
            if len(search_lines) == 1:
                # Single line search - check for partial matches
                partial_matches = [line for line in lines if search_text.strip() in line]
                if partial_matches:
                    return f"Exact match not found, but similar lines exist", f"Try searching for: {partial_matches[0][:100]}"
            
            return "Search text not found in file", "Check search text for typos or use fuzzy_matching=true"
        
        return "Unknown edit failure", "Check file permissions and content"
    
    async def _project_analyze(self, path: str, include_patterns: Optional[List[str]] = None, 
                              exclude_patterns: Optional[List[str]] = None, depth: int = 10) -> Dict[str, Any]:
        """Analyze project structure and components."""
        project_path = Path(path).resolve()
        validate_project_path(project_path)
        
        analysis = await self.project_analyzer.analyze(
            project_path, 
            include_patterns=include_patterns,
            exclude_patterns=exclude_patterns,
            max_depth=depth
        )
        
        return asdict(analysis)
    
    async def _context_score_files(self, files: List[str], task_context: str, 
                                  task_type: Optional[str] = None, max_files: int = 20) -> Dict[str, Any]:
        """Score file relevance for current task."""
        file_paths = [Path(f).resolve() for f in files]
        
        scores = await self.relevance_engine.score_files(
            file_paths,
            task_context=task_context,
            task_type=task_type,
            max_results=max_files
        )
        
        return {
            "scored_files": [asdict(score) for score in scores],
            "task_context": task_context,
            "task_type": task_type,
            "total_files_analyzed": len(files)
        }
    
    async def _task_detect(self, project_path: str, recent_changes: Optional[List[str]] = None,
                          active_files: Optional[List[str]] = None, user_intent: Optional[str] = None) -> Dict[str, Any]:
        """Detect current development task type."""
        project_path = Path(project_path).resolve()
        validate_project_path(project_path)
        
        detection = await self.task_detector.detect_task(
            project_path,
            recent_changes=recent_changes,
            active_files=active_files,
            user_intent=user_intent
        )
        
        return asdict(detection)
    
    async def _dependency_graph(self, project_path: str, target_files: Optional[List[str]] = None,
                               graph_type: str = "all", max_depth: int = 5) -> Dict[str, Any]:
        """Build dependency graph for project."""
        project_path = Path(project_path).resolve()
        validate_project_path(project_path)
        
        graph = await self.dependency_analyzer.build_graph(
            project_path,
            target_files=target_files,
            graph_type=graph_type,
            max_depth=max_depth
        )
        
        return asdict(graph)
    
    async def _success_patterns(self, task_type: str, project_context: Dict[str, Any],
                               similar_projects: bool = True, confidence_threshold: float = 0.7) -> Dict[str, Any]:
        """Find applicable success patterns."""
        patterns = await self.pattern_learner.find_patterns(
            task_type=task_type,
            project_context=project_context,
            include_similar_projects=similar_projects,
            min_confidence=confidence_threshold
        )
        
        return {
            "patterns": [asdict(pattern) for pattern in patterns],
            "task_type": task_type,
            "confidence_threshold": confidence_threshold
        }
    
    async def _cross_project_insights(self, current_context: str, project_technologies: List[str],
                                     include_worktrees: bool = True, similarity_threshold: float = 0.6) -> Dict[str, Any]:
        """Get insights from similar project contexts."""
        insights = await self.cross_insights.find_insights(
            current_context=current_context,
            technologies=project_technologies,
            include_worktrees=include_worktrees,
            min_similarity=similarity_threshold
        )
        
        return {
            "insights": [asdict(insight) for insight in insights],
            "current_context": current_context,
            "similarity_threshold": similarity_threshold
        }
    
    async def _smart_context_assembly(self, target_files: List[str], task_description: str,
                                     token_limit: int, model_name: Optional[str] = None,
                                     preserve_structure: bool = True) -> Dict[str, Any]:
        """Assemble optimized context within token limits."""
        file_paths = [Path(f).resolve() for f in target_files]
        
        context = await self.context_assembler.assemble_context(
            files=file_paths,
            task_description=task_description,
            token_limit=token_limit,
            model_name=model_name,
            preserve_structure=preserve_structure
        )
        
        return asdict(context)

    async def run(self):
        """Run the MCP server."""
        async with stdio_server() as (read_stream, write_stream):
            await self.server.run(read_stream, write_stream, self.config.server_options)

def main():
    """Main entry point for the Aircher Intelligence Engine MCP server."""
    parser = argparse.ArgumentParser(description="Aircher Intelligence Engine MCP Server")
    parser.add_argument("--config", help="Configuration file path")
    parser.add_argument("--log-level", default="INFO", help="Logging level")
    parser.add_argument("--database-path", help="Override database path")
    
    args = parser.parse_args()
    
    # Setup logging
    logging.basicConfig(
        level=getattr(logging, args.log_level.upper()),
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    )
    
    # Create and run server
    server = AircherIntelligenceServer(config_path=args.config)
    
    if args.database_path:
        server.config.database_path = args.database_path
    
    try:
        asyncio.run(server.run())
    except KeyboardInterrupt:
        logger.info("Shutting down Aircher Intelligence Engine")
    except Exception as e:
        logger.error(f"Server error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
```

## Intelligence Components

### Project Analyzer
```python
class ProjectAnalyzer:
    """Comprehensive project structure analysis."""
    
    def __init__(self, config: AircherConfig, database: AircherDatabase):
        self.config = config
        self.database = database
        self.language_detectors = self._load_language_detectors()
        self.framework_detectors = self._load_framework_detectors()
    
    async def analyze(self, project_path: Path, include_patterns: Optional[List[str]] = None,
                     exclude_patterns: Optional[List[str]] = None, max_depth: int = 10) -> ProjectAnalysis:
        """Perform comprehensive project analysis."""
        start_time = datetime.now()
        
        # File system analysis
        file_tree = await self._analyze_file_structure(project_path, include_patterns, exclude_patterns, max_depth)
        
        # Technology detection
        technologies = await self._detect_technologies(file_tree)
        
        # Component detection
        components = await self._detect_components(file_tree, technologies)
        
        # Dependency analysis
        dependencies = await self._analyze_dependencies(file_tree, technologies)
        
        # Entry point detection
        entry_points = await self._find_entry_points(file_tree, technologies)
        
        # Test and config file detection
        test_files = await self._find_test_files(file_tree)
        config_files = await self._find_config_files(file_tree)
        documentation = await self._find_documentation(file_tree)
        
        # Calculate metrics
        metrics = await self._calculate_metrics(file_tree, components)
        
        # Overall confidence scoring
        confidence = await self._calculate_confidence(technologies, components, metrics)
        
        analysis = ProjectAnalysis(
            project_path=str(project_path),
            technologies=technologies,
            components=components,
            structure=file_tree,
            dependencies=dependencies,
            entry_points=entry_points,
            test_files=test_files,
            config_files=config_files,
            documentation=documentation,
            metrics=metrics,
            confidence_score=confidence,
            analysis_timestamp=start_time.isoformat()
        )
        
        # Cache analysis result
        await self.database.save_project_analysis(analysis)
        
        return analysis
```

### File Relevance Engine
```python
class FileRelevanceEngine:
    """AI-driven file relevance scoring for development tasks."""
    
    def __init__(self, config: AircherConfig, database: AircherDatabase):
        self.config = config
        self.database = database
        self.embedding_model = self._load_embedding_model()
        self.dependency_analyzer = DependencyAnalyzer(config)
    
    async def score_files(self, files: List[Path], task_context: str, 
                         task_type: Optional[str] = None, max_results: int = 20) -> List[FileRelevanceScore]:
        """Score files for relevance to current task."""
        scores = []
        
        # Get task embedding for semantic similarity
        task_embedding = await self._get_embedding(task_context)
        
        # Process each file
        for file_path in files:
            try:
                score = await self._score_single_file(file_path, task_context, task_embedding, task_type)
                scores.append(score)
            except Exception as e:
                logger.warning(f"Failed to score file {file_path}: {e}")
        
        # Sort by relevance and return top results
        scores.sort(key=lambda x: x.relevance_score, reverse=True)
        return scores[:max_results]
    
    async def _score_single_file(self, file_path: Path, task_context: str, 
                                task_embedding: np.ndarray, task_type: Optional[str]) -> FileRelevanceScore:
        """Score a single file for task relevance."""
        # Base scoring factors
        base_score = await self._calculate_base_score(file_path)
        
        # Semantic similarity to task
        semantic_score = await self._calculate_semantic_similarity(file_path, task_embedding)
        
        # Task type alignment
        task_alignment = await self._calculate_task_alignment(file_path, task_type) if task_type else 0.5
        
        # Dependency importance
        dependency_importance = await self._calculate_dependency_importance(file_path)
        
        # Recency factor (recent changes are more relevant)
        recency_factor = await self._calculate_recency_factor(file_path)
        
        # Success correlation (how often this file was useful in similar tasks)
        success_correlation = await self._calculate_success_correlation(file_path, task_context, task_type)
        
        # Weighted final score
        weights = self.config.relevance_weights
        final_score = (
            base_score * weights.base +
            semantic_score * weights.semantic +
            task_alignment * weights.task_alignment +
            dependency_importance * weights.dependency +
            recency_factor * weights.recency +
            success_correlation * weights.success_correlation
        )
        
        # Generate reasoning
        reasoning = self._generate_relevance_reasoning(
            file_path, semantic_score, task_alignment, dependency_importance, 
            recency_factor, success_correlation
        )
        
        # Calculate overall confidence
        confidence = self._calculate_confidence(
            semantic_score, task_alignment, dependency_importance, success_correlation
        )
        
        return FileRelevanceScore(
            file_path=str(file_path),
            relevance_score=final_score,
            confidence=confidence,
            reasoning=reasoning,
            task_alignment=task_alignment,
            dependency_importance=dependency_importance,
            recency_factor=recency_factor,
            success_correlation=success_correlation
        )
```

## Deployment Configuration

### pyproject.toml
```toml
[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "aircher-intelligence"
version = "0.1.0" 
description = "Universal intelligent context management for AI-powered development"
authors = [
    {name = "Aircher Team", email = "team@aircher.dev"}
]
license = "Elastic-2.0"
readme = "README.md"
requires-python = ">=3.11"
keywords = ["ai", "development", "context", "mcp", "intelligence"]
classifiers = [
    "Development Status :: 3 - Alpha",
    "Intended Audience :: Developers", 
    "License :: Other/Proprietary License",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Topic :: Software Development :: Tools",
    "Topic :: Scientific/Engineering :: Artificial Intelligence"
]

dependencies = [
    # MCP Protocol
    "mcp>=0.1.0",
    
    # Core dependencies
    "aiofiles>=23.0.0",
    "aiohttp>=3.9.0", 
    "asyncio-mqtt>=0.16.0",
    "pydantic>=2.5.0",
    "typer>=0.9.0",
    "rich>=13.0.0",
    
    # AI and ML
    "numpy>=1.24.0",
    "sentence-transformers>=2.2.0",
    "scikit-learn>=1.3.0",
    "transformers>=4.35.0",
    
    # File analysis
    "tree-sitter>=0.20.0",
    "tree-sitter-python>=0.20.0",
    "tree-sitter-javascript>=0.20.0", 
    "tree-sitter-rust>=0.20.0",
    "tree-sitter-go>=0.20.0",
    "tree-sitter-typescript>=0.20.0",
    
    # Git integration
    "gitpython>=3.1.0",
    "dulwich>=0.21.0",
    
    # Database
    "aiosqlite>=0.19.0",
    "sqlalchemy[asyncio]>=2.0.0",
    
    # Configuration
    "pyyaml>=6.0.0",
    "tomli>=2.0.0",
    
    # Utilities
    "watchfiles>=0.21.0",
    "click>=8.0.0",
    "pathspec>=0.11.0"
]

[project.optional-dependencies]
dev = [
    "pytest>=7.0.0",
    "pytest-asyncio>=0.21.0",
    "pytest-cov>=4.0.0",
    "black>=23.0.0",
    "isort>=5.12.0",
    "mypy>=1.7.0",
    "ruff>=0.1.0"
]

[project.scripts]
aircher-intelligence = "aircher_intelligence.server:main"

[project.urls]
Homepage = "https://github.com/aircher/aircher"
Repository = "https://github.com/aircher/aircher"
Documentation = "https://docs.aircher.dev"

[tool.hatch.build.targets.wheel]
packages = ["src/aircher_intelligence"]

[tool.hatch.envs.default]
dependencies = ["pytest", "pytest-asyncio", "pytest-cov"]

[tool.hatch.envs.default.scripts]
test = "pytest {args:tests}"
test-cov = "pytest --cov=aircher_intelligence {args:tests}"
cov-report = ["test-cov", "coverage report"]

[tool.black]
line-length = 100
target-version = ['py311']

[tool.isort]
profile = "black"
line_length = 100

[tool.mypy]
python_version = "3.11"
warn_return_any = true
warn_unused_configs = true
disallow_untyped_defs = true

[tool.ruff]
line-length = 100
select = ["E", "F", "W", "C", "N"]
ignore = ["E501"]
```

### Claude Desktop Integration
```json
{
  "mcpServers": {
    "aircher-intelligence": {
      "command": "uvx",
      "args": ["--from", "aircher-intelligence", "aircher-intelligence"],
      "env": {
        "AIRCHER_CONFIG_PATH": "~/.config/aircher/config.toml",
        "AIRCHER_DATABASE_PATH": ".aircher/db",
        "AIRCHER_LOG_LEVEL": "INFO"
      }
    }
  }
}
```

### VS Code Integration
```json
{
  "mcp.servers": [
    {
      "name": "aircher-intelligence",
      "command": ["uvx", "--from", "aircher-intelligence", "aircher-intelligence"],
      "env": {
        "AIRCHER_CONFIG_PATH": "~/.config/aircher/config.toml", 
        "AIRCHER_DATABASE_PATH": ".aircher/db"
      }
    }
  ]
}
```

## Configuration

### Default Configuration (.aircher/config.toml)
```toml
[intelligence]
# File relevance scoring weights
[intelligence.relevance_weights]
base = 0.15           # File type, size, language
semantic = 0.25       # Semantic similarity to task
task_alignment = 0.20 # Alignment with task type
dependency = 0.15     # Dependency importance
recency = 0.15        # Recent changes
success_correlation = 0.10  # Historical success

# Task detection settings
[intelligence.task_detection]
confidence_threshold = 0.7
git_analysis_enabled = true
file_change_analysis = true
user_intent_weight = 0.4

# Cross-project learning
[intelligence.cross_project]
enabled = true
similarity_threshold = 0.6
max_projects = 100
learning_rate = 0.1

[database]
path = ".aircher/db"
backup_enabled = true
backup_interval = "24h"

[server]
name = "aircher-intelligence"
version = "0.1.0"
timeout = 30.0
max_concurrent_requests = 10

[logging]
level = "INFO"
file = ".aircher/logs/intelligence.log"
max_size = "10MB"
backup_count = 5
```

## Documentation Updates Needed

Based on this analysis, we need to update several documentation files to reflect the Python MCP server + Rust client architecture:

<function_calls>
<invoke name="TodoWrite">
<parameter name="todos">[{"id": "update-directory-structure-1", "content": "Update README.md to use .aircher/ directory structure", "status": "completed", "priority": "high"}, {"id": "update-directory-structure-2", "content": "Update PROJECT_ROADMAP.md to use .aircher/ directory structure", "status": "completed", "priority": "high"}, {"id": "update-directory-structure-3", "content": "Update MASTER_SPEC.md to use .aircher/ directory structure", "status": "completed", "priority": "high"}, {"id": "update-directory-structure-4", "content": "Update storage-architecture.md to use .aircher/ directory structure", "status": "completed", "priority": "high"}, {"id": "update-directory-structure-5", "content": "Update context-management.md to use .aircher/ directory structure", "status": "completed", "priority": "high"}, {"id": "update-directory-structure-6", "content": "Update DEVELOPER_GUIDE.md to use .aircher/ directory structure", "status": "completed", "priority": "medium"}, {"id": "update-directory-structure-7", "content": "Update worktree-management.md to use .aircher/ directory structure", "status": "completed", "priority": "medium"}, {"id": "update-directory-structure-8", "content": "Update configuration-system.md to use .aircher/ directory structure", "status": "completed", "priority": "medium"}, {"id": "update-directory-structure-9", "content": "Update cli-specification.md to use .aircher/ directory structure", "status": "completed", "priority": "medium"}, {"id": "create-mcp-server-spec", "content": "Create technical specification for Aircher Intelligence Engine MCP Server (Python)", "status": "completed", "priority": "high"}, {"id": "update-mcp-integration-language", "content": "Update MCP integration docs to reflect Python server + Rust client architecture", "status": "in_progress", "priority": "medium"}, {"id": "update-master-spec-python", "content": "Update MASTER_SPEC.md to reflect Python MCP server architecture", "status": "pending", "priority": "medium"}, {"id": "update-roadmap-python", "content": "Update PROJECT_ROADMAP.md to include Python MCP server implementation details", "status": "pending", "priority": "medium"}]