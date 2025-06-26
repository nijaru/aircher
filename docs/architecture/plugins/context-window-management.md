# Context Window Management Specification

## Overview

Based on Warp's successful approach to achieving 71% on SWE-bench and 52% on Terminal-Bench, intelligent context window management is crucial for AI coding success. This specification outlines our approach to smart file chunking, scrollable results, and context-dependent tool availability.

## Core Principles (Learned from Warp's Success)

### 1. **Smart File Chunking**
- **Limit display to manageable chunks**: 100 lines per file by default
- **Enable scrolling capability**: Allow agents to "scroll" through large files
- **Maintain reasonable context window**: Prevent token limit overflows
- **Preserve logical structure**: Keep functions/classes intact when possible

### 2. **Context-Dependent Tool Availability**
- **Restrict tools during long-running commands**: When interactive commands are active
- **Preserve context window**: Use same agent with restricted tool set
- **Dynamic tool availability**: Tools available change based on current state

### 3. **Scrollable Results**
- **Incremental viewing**: Allow agents to request more content as needed
- **Chunk management**: Track which chunks have been viewed
- **Memory efficiency**: Don't load entire large files into memory

## Implementation Architecture

### Context Window Manager (Python MCP Server)

```python
class ContextWindowManager:
    """Manages context windows with Warp-inspired smart chunking."""
    
    def __init__(self, config: ContextConfig):
        self.config = config
        self.chunk_cache = LRUCache(maxsize=1000)
        self.scroll_state = {}
        
    async def chunk_file_content(self, file_path: Path, strategy: ChunkingStrategy) -> List[FileChunk]:
        """Chunk file content using specified strategy."""
        content = await self._read_file_safe(file_path)
        
        if strategy == ChunkingStrategy.LINES:
            return self._chunk_by_lines(content, self.config.max_lines_per_chunk)
        elif strategy == ChunkingStrategy.FUNCTIONS:
            return await self._chunk_by_functions(content, file_path)
        elif strategy == ChunkingStrategy.SEMANTIC:
            return await self._chunk_semantically(content, file_path)
        else:
            raise ValueError(f"Unknown chunking strategy: {strategy}")
    
    async def get_file_chunk(self, file_path: Path, chunk_index: int = 0, 
                           max_lines: int = 100) -> FileChunk:
        """Get specific chunk of file with scrolling support."""
        cache_key = f"{file_path}:{chunk_index}:{max_lines}"
        
        if cache_key in self.chunk_cache:
            return self.chunk_cache[cache_key]
        
        content = await self._read_file_safe(file_path)
        lines = content.split('\n')
        
        start_line = chunk_index * max_lines
        end_line = min(start_line + max_lines, len(lines))
        
        chunk_content = '\n'.join(lines[start_line:end_line])
        
        chunk = FileChunk(
            file_path=str(file_path),
            chunk_index=chunk_index,
            line_start=start_line + 1,  # 1-indexed for display
            line_end=end_line,
            content=chunk_content,
            total_lines=len(lines),
            has_more=end_line < len(lines),
            has_previous=chunk_index > 0
        )
        
        self.chunk_cache[cache_key] = chunk
        return chunk
    
    async def get_scrollable_search_results(self, query: str, files: List[Path],
                                          max_lines_per_file: int = 100,
                                          context_lines: int = 3) -> ScrollableSearchResults:
        """Get search results with scrolling capability."""
        results = []
        
        for file_path in files:
            try:
                file_results = await self._search_file_chunked(
                    file_path, query, max_lines_per_file, context_lines
                )
                if file_results:
                    results.extend(file_results)
            except Exception as e:
                logger.warning(f"Failed to search file {file_path}: {e}")
        
        return ScrollableSearchResults(
            query=query,
            total_files=len(files),
            files_with_matches=len([r for r in results if r.matches]),
            results=results,
            can_scroll=any(r.has_more for r in results)
        )
    
    def _chunk_by_lines(self, content: str, max_lines: int) -> List[FileChunk]:
        """Simple line-based chunking."""
        lines = content.split('\n')
        chunks = []
        
        for i in range(0, len(lines), max_lines):
            chunk_lines = lines[i:i + max_lines]
            chunks.append(FileChunk(
                chunk_index=i // max_lines,
                line_start=i + 1,
                line_end=min(i + max_lines, len(lines)),
                content='\n'.join(chunk_lines),
                total_lines=len(lines),
                has_more=i + max_lines < len(lines),
                has_previous=i > 0
            ))
        
        return chunks
    
    async def _chunk_by_functions(self, content: str, file_path: Path) -> List[FileChunk]:
        """Chunk by function/class boundaries to preserve logical structure."""
        # Use AST parsing to identify function/class boundaries
        try:
            from .ast_parser import ASTParser
            parser = ASTParser()
            
            ast_nodes = await parser.parse_file(file_path)
            function_boundaries = self._extract_function_boundaries(ast_nodes)
            
            return self._create_function_chunks(content, function_boundaries)
        except Exception as e:
            logger.warning(f"Function chunking failed, falling back to line chunking: {e}")
            return self._chunk_by_lines(content, self.config.max_lines_per_chunk)
    
    async def _chunk_semantically(self, content: str, file_path: Path) -> List[FileChunk]:
        """Semantic chunking based on code structure and meaning."""
        # Combine AST analysis with semantic similarity
        function_chunks = await self._chunk_by_functions(content, file_path)
        
        # Group related functions together if they fit in context window
        semantic_chunks = []
        current_chunk_content = []
        current_chunk_lines = 0
        
        for func_chunk in function_chunks:
            chunk_size = len(func_chunk.content.split('\n'))
            
            if current_chunk_lines + chunk_size <= self.config.max_lines_per_chunk:
                current_chunk_content.append(func_chunk.content)
                current_chunk_lines += chunk_size
            else:
                if current_chunk_content:
                    semantic_chunks.append(self._merge_chunk_content(current_chunk_content))
                current_chunk_content = [func_chunk.content]
                current_chunk_lines = chunk_size
        
        if current_chunk_content:
            semantic_chunks.append(self._merge_chunk_content(current_chunk_content))
        
        return semantic_chunks

@dataclass
class FileChunk:
    """A chunk of file content with metadata."""
    file_path: str = ""
    chunk_index: int = 0
    line_start: int = 1
    line_end: int = 1
    content: str = ""
    total_lines: int = 0
    has_more: bool = False
    has_previous: bool = False
    
@dataclass 
class SearchResultChunk:
    """Search result within a file chunk."""
    file_path: str
    chunk_index: int
    line_start: int
    line_end: int
    content: str
    matches: List[MatchLocation]
    has_more: bool
    total_matches_in_file: int

@dataclass
class ScrollableSearchResults:
    """Search results with scrolling capability."""
    query: str
    total_files: int
    files_with_matches: int
    results: List[SearchResultChunk]
    can_scroll: bool

class ChunkingStrategy(Enum):
    """Different strategies for chunking file content."""
    LINES = "lines"           # Simple line-based chunking
    FUNCTIONS = "functions"   # Chunk by function/class boundaries  
    SEMANTIC = "semantic"     # Intelligent semantic chunking
```

### Context-Dependent Tool Manager

```python
class ContextDependentToolManager:
    """Manages tool availability based on current context state."""
    
    def __init__(self):
        self.active_sessions = {}
        self.tool_restrictions = {
            "interactive_command_active": [
                "send_input_to_command",
                "get_command_output", 
                "terminate_command"
            ],
            "large_file_viewing": [
                "scroll_file_chunk",
                "get_next_chunk",
                "get_previous_chunk"
            ],
            "search_results_active": [
                "get_next_search_results",
                "get_search_chunk_details",
                "refine_search_query"
            ]
        }
    
    def get_available_tools(self, session_context: SessionContext) -> List[str]:
        """Get tools available in current context."""
        base_tools = [
            "project_analyze",
            "context_score_files", 
            "task_detect",
            "dependency_graph",
            "success_patterns",
            "cross_project_insights",
            "smart_context_assembly"
        ]
        
        # Add context-specific tools
        if session_context.has_active_interactive_command:
            return self.tool_restrictions["interactive_command_active"]
        elif session_context.is_viewing_large_file:
            return base_tools + self.tool_restrictions["large_file_viewing"]
        elif session_context.has_active_search:
            return base_tools + self.tool_restrictions["search_results_active"]
        else:
            # Full tool set available
            return base_tools + [
                "edit_files",
                "create_file", 
                "search_files",
                "run_command"
            ]
    
    def is_tool_restricted(self, tool_name: str, session_context: SessionContext) -> bool:
        """Check if tool is restricted in current context."""
        available_tools = self.get_available_tools(session_context)
        return tool_name not in available_tools

@dataclass
class SessionContext:
    """Current session context for tool availability decisions."""
    session_id: str
    has_active_interactive_command: bool = False
    interactive_command_id: Optional[str] = None
    is_viewing_large_file: bool = False
    current_file_chunk: Optional[FileChunk] = None
    has_active_search: bool = False
    search_results: Optional[ScrollableSearchResults] = None
    token_usage: int = 0
    max_token_limit: int = 200000
```

### Integration with MCP Tools

```python
# Enhanced edit_files tool with chunking support
async def edit_files_with_chunking(edits: List[FileEdit], 
                                 chunk_large_files: bool = True) -> EditResults:
    """Edit files with automatic chunking for large files."""
    results = []
    
    for edit in edits:
        file_path = Path(edit.file_path)
        
        # Check if file is large and should be chunked
        if chunk_large_files and await _is_large_file(file_path):
            # Find the chunk containing the search text
            chunk = await _find_chunk_with_text(file_path, edit.search)
            if chunk:
                edit_result = await _edit_file_chunk(chunk, edit)
                results.append(edit_result)
            else:
                results.append(EditResult(
                    file_path=str(file_path),
                    success=False,
                    error_message="Search text not found in any chunk",
                    recovery_suggestion="Try searching file first to locate text"
                ))
        else:
            # Standard edit for smaller files
            edit_result = await _edit_file_standard(edit)
            results.append(edit_result)
    
    return EditResults(
        total_edits=len(edits),
        successful_edits=sum(1 for r in results if r.success),
        failed_edits=sum(1 for r in results if not r.success),
        results=results
    )

# Enhanced search_files tool with scrolling
async def search_files_with_scrolling(query: str, 
                                    file_patterns: List[str],
                                    max_lines_per_file: int = 100,
                                    chunk_index: int = 0) -> ScrollableSearchResults:
    """Search files with chunking and scrolling support."""
    context_manager = ContextWindowManager(config)
    
    matching_files = await _find_matching_files(file_patterns)
    
    return await context_manager.get_scrollable_search_results(
        query=query,
        files=matching_files,
        max_lines_per_file=max_lines_per_file,
        context_lines=3
    )
```

## Configuration

```toml
# Context window management settings
[context_window]
max_lines_per_chunk = 100
max_chunks_in_memory = 50
chunking_strategy = "semantic"  # lines, functions, semantic
context_lines_around_matches = 3
enable_scrolling = true
cache_chunks = true
cache_size_mb = 100

[context_window.tool_restrictions]
interactive_commands = ["send_input_to_command", "terminate_command"]
large_file_viewing = ["scroll_file_chunk", "get_next_chunk"] 
search_active = ["get_next_search_results", "refine_search_query"]

[context_window.performance]
chunk_cache_ttl = "1h"
max_concurrent_chunks = 10
async_chunk_loading = true
```

## Performance Targets

Based on Warp's success metrics:

- **Chunk Loading**: <50ms for standard chunks (100 lines)
- **Search Results**: <200ms for scrollable search across 1000 files
- **Context Switch**: <100ms when changing tool availability
- **Memory Usage**: <50MB for chunk cache
- **Cache Hit Rate**: >80% for frequently accessed chunks

This context window management system enables the same scrolling and chunking patterns that contributed to Warp's benchmark success while maintaining the universal compatibility of our MCP architecture.