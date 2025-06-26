# Modular Performance Architecture for Aircher Intelligence Engine

## Overview

The Aircher Intelligence Engine uses a **modular backend architecture** that allows performance-critical components to be implemented in different languages while maintaining a unified Python interface. This enables starting with pure Python for rapid development while incrementally optimizing bottlenecks with Rust or Mojo.

## Performance-Based Component Strategy

### **Phase 1: Hybrid Architecture (Python + Rust for Critical Paths)**

Start with Rust for the most performance-critical components that Python cannot handle efficiently:

#### **🔥 Critical Performance Components → Start with Rust**

**1. File System Analysis Engine**
```python
# Python Interface
from aircher_intelligence.backends import FileSystemBackend

class FileSystemAnalyzer:
    def __init__(self, backend: FileSystemBackend = None):
        self.backend = backend or self._get_optimal_backend()
    
    async def walk_project(self, path: Path, patterns: List[str]) -> ProjectStructure:
        return await self.backend.walk_project(path, patterns)
    
    def _get_optimal_backend(self) -> FileSystemBackend:
        try:
            from .backends.rust import RustFileSystemBackend
            return RustFileSystemBackend()
        except ImportError:
            from .backends.python import PythonFileSystemBackend
            return PythonFileSystemBackend()
```

**Why Rust for File System:**
- **10-50x faster** than Python's `os.walk()`
- Parallel directory traversal with `walkdir` crate
- Pattern matching with optimized regex
- Memory-efficient for large codebases (>100k files)

**2. AST Parsing and Code Analysis**
```python
# Python Interface
from aircher_intelligence.backends import ASTBackend

class CodeAnalyzer:
    def __init__(self, backend: ASTBackend = None):
        self.backend = backend or self._get_optimal_backend()
    
    async def parse_file(self, file_path: Path, language: str) -> ASTNode:
        return await self.backend.parse_file(file_path, language)
    
    async def extract_dependencies(self, file_path: Path) -> List[Dependency]:
        return await self.backend.extract_dependencies(file_path)
```

**Why Rust for AST:**
- **5-20x faster** than Python tree-sitter bindings
- Native tree-sitter performance
- Parallel parsing across files
- Memory-efficient AST traversal

**3. Pattern Matching Engine**
```python
# Python Interface
from aircher_intelligence.backends import PatternBackend

class PatternMatcher:
    def __init__(self, backend: PatternBackend = None):
        self.backend = backend or self._get_optimal_backend()
    
    async def find_patterns(self, content: str, patterns: List[Pattern]) -> List[Match]:
        return await self.backend.find_patterns(content, patterns)
```

**Why Rust for Patterns:**
- **3-10x faster** than Python regex
- Compiled regex with optimizations
- SIMD string operations

#### **✅ Acceptable Python Performance → Start with Python**

**1. Vector Operations and Embeddings**
```python
# Pure Python with numpy/scipy
import numpy as np
from sentence_transformers import SentenceTransformer

class EmbeddingEngine:
    def __init__(self):
        self.model = SentenceTransformer('all-MiniLM-L6-v2')
    
    async def get_embedding(self, text: str) -> np.ndarray:
        # Python + numpy is fast enough for this
        return await asyncio.to_thread(self.model.encode, text)
    
    def cosine_similarity(self, a: np.ndarray, b: np.ndarray) -> float:
        # numpy is highly optimized for this
        return float(np.dot(a, b) / (np.linalg.norm(a) * np.linalg.norm(b)))
```

**Why Python is Fine:**
- numpy/scipy are already optimized with BLAS/LAPACK
- Sentence transformers handle GPU acceleration
- **Performance difference minimal** for embedding operations

**2. Git Operations**
```python
# Pure Python with optimized libraries
import git
from dulwich import porcelain

class GitAnalyzer:
    async def get_recent_changes(self, repo_path: Path, days: int = 7) -> List[GitChange]:
        # GitPython/dulwich are fast enough for git ops
        pass
```

**Why Python is Fine:**
- Git operations are I/O bound, not CPU bound
- GitPython/dulwich are mature and optimized
- **Rust benefit would be minimal**

### **Phase 2: Optional Optimizations → Consider Rust Later**

**3. Database Operations**
```python
# Python with async database drivers
import aiosqlite
import asyncpg

class DatabaseManager:
    async def execute_query(self, query: str, params: tuple) -> List[dict]:
        # aiosqlite/asyncpg are already fast
        pass
```

**Why Python is Fine Initially:**
- Database drivers are already optimized
- I/O bound, not CPU bound
- SQLite/PostgreSQL do the heavy lifting

## Backend Interface Design

### Universal Backend Protocol
```python
# aircher_intelligence/backends/base.py
from abc import ABC, abstractmethod
from typing import Protocol, Union, Any
from pathlib import Path

class FileSystemBackend(Protocol):
    """Protocol for file system analysis backends."""
    
    async def walk_project(self, path: Path, include_patterns: List[str], 
                          exclude_patterns: List[str]) -> ProjectStructure:
        """Walk project directory with pattern filtering."""
        ...
    
    async def get_file_info(self, path: Path) -> FileInfo:
        """Get detailed file information."""
        ...
    
    async def watch_changes(self, path: Path, callback: Callable) -> None:
        """Watch for file system changes."""
        ...

class ASTBackend(Protocol):
    """Protocol for AST parsing backends."""
    
    async def parse_file(self, file_path: Path, language: str) -> ASTNode:
        """Parse file into AST."""
        ...
    
    async def extract_dependencies(self, file_path: Path) -> List[Dependency]:
        """Extract import/dependency information."""
        ...
    
    async def extract_symbols(self, file_path: Path) -> List[Symbol]:
        """Extract functions, classes, variables."""
        ...

class PatternBackend(Protocol):
    """Protocol for pattern matching backends."""
    
    async def find_patterns(self, content: str, patterns: List[Pattern]) -> List[Match]:
        """Find pattern matches in content."""
        ...
    
    async def similarity_search(self, query: str, corpus: List[str], 
                               threshold: float) -> List[SimilarityMatch]:
        """Find similar text in corpus."""
        ...
```

### Python Implementation (MVP)
```python
# aircher_intelligence/backends/python/filesystem.py
import os
import asyncio
from pathlib import Path
import fnmatch
from typing import List, Iterator

class PythonFileSystemBackend:
    """Pure Python file system backend for initial implementation."""
    
    async def walk_project(self, path: Path, include_patterns: List[str], 
                          exclude_patterns: List[str]) -> ProjectStructure:
        """Walk project using Python's os.walk with async wrapper."""
        def _sync_walk():
            files = []
            dirs = []
            
            for root, dirnames, filenames in os.walk(path):
                # Filter directories
                dirnames[:] = [d for d in dirnames 
                              if not self._should_exclude(Path(root) / d, exclude_patterns)]
                
                for filename in filenames:
                    file_path = Path(root) / filename
                    if self._should_include(file_path, include_patterns, exclude_patterns):
                        files.append(FileInfo.from_path(file_path))
                
                dirs.extend([Path(root) / d for d in dirnames])
            
            return ProjectStructure(files=files, directories=dirs)
        
        return await asyncio.to_thread(_sync_walk)
    
    def _should_include(self, path: Path, include_patterns: List[str], 
                       exclude_patterns: List[str]) -> bool:
        """Check if file should be included based on patterns."""
        # Check exclude patterns first
        for pattern in exclude_patterns:
            if fnmatch.fnmatch(str(path), pattern):
                return False
        
        # If no include patterns, include all (that aren't excluded)
        if not include_patterns:
            return True
        
        # Check include patterns
        for pattern in include_patterns:
            if fnmatch.fnmatch(str(path), pattern):
                return True
        
        return False

# aircher_intelligence/backends/python/ast_parser.py
import tree_sitter
import asyncio
from pathlib import Path

class PythonASTBackend:
    """Pure Python AST backend using tree-sitter."""
    
    def __init__(self):
        self.parsers = self._init_parsers()
    
    def _init_parsers(self) -> dict:
        """Initialize tree-sitter parsers for different languages."""
        # Note: This will be slow in pure Python but functional
        return {
            'python': tree_sitter.Language.build_library('python.so', ['tree-sitter-python']),
            'javascript': tree_sitter.Language.build_library('js.so', ['tree-sitter-javascript']),
            'rust': tree_sitter.Language.build_library('rust.so', ['tree-sitter-rust']),
            # Add more languages as needed
        }
    
    async def parse_file(self, file_path: Path, language: str) -> ASTNode:
        """Parse file into AST using tree-sitter."""
        def _sync_parse():
            if language not in self.parsers:
                raise ValueError(f"Unsupported language: {language}")
            
            parser = tree_sitter.Parser()
            parser.set_language(self.parsers[language])
            
            with open(file_path, 'rb') as f:
                source_code = f.read()
            
            tree = parser.parse(source_code)
            return ASTNode.from_tree_sitter(tree.root_node, source_code)
        
        return await asyncio.to_thread(_sync_parse)
```

### Rust Implementation (Performance Critical)
```python
# aircher_intelligence/backends/rust/filesystem.py
import asyncio
from pathlib import Path
from typing import List, Optional

class RustFileSystemBackend:
    """Rust-powered file system backend for high performance."""
    
    def __init__(self):
        try:
            import aircher_intelligence_core
            self.core = aircher_intelligence_core
        except ImportError:
            raise ImportError("Rust backend not available. Install with: pip install aircher-intelligence[rust]")
    
    async def walk_project(self, path: Path, include_patterns: List[str], 
                          exclude_patterns: List[str]) -> ProjectStructure:
        """Walk project using parallel Rust implementation."""
        def _rust_walk():
            # Delegate to optimized Rust implementation
            result = self.core.walk_project_parallel(
                str(path),
                include_patterns,
                exclude_patterns,
                max_threads=8  # Parallel directory traversal
            )
            return ProjectStructure.from_rust(result)
        
        return await asyncio.to_thread(_rust_walk)

# aircher_intelligence/backends/rust/ast_parser.py
class RustASTBackend:
    """Rust-powered AST backend for high performance parsing."""
    
    def __init__(self):
        try:
            import aircher_intelligence_core
            self.core = aircher_intelligence_core
        except ImportError:
            raise ImportError("Rust backend not available")
    
    async def parse_file(self, file_path: Path, language: str) -> ASTNode:
        """Parse file using optimized Rust tree-sitter."""
        def _rust_parse():
            # Native Rust tree-sitter with parallel processing
            result = self.core.parse_file_fast(str(file_path), language)
            return ASTNode.from_rust(result)
        
        return await asyncio.to_thread(_rust_parse)
    
    async def extract_dependencies(self, file_path: Path) -> List[Dependency]:
        """Extract dependencies using optimized Rust analysis."""
        def _rust_deps():
            result = self.core.extract_dependencies_batch([str(file_path)])
            return [Dependency.from_rust(dep) for dep in result]
        
        return await asyncio.to_thread(_rust_deps)
```

## Implementation Strategy

### **Phase 1: MVP with Critical Rust Components**

**Priority Order:**
1. **🔥 Start with Rust**: File system walking + AST parsing (biggest bottlenecks)
2. **✅ Pure Python**: Everything else (MCP protocol, embeddings, git, database)

**Rationale:**
- File system operations will be used constantly and are Python's biggest weakness
- AST parsing is CPU-intensive and benefits hugely from Rust
- Other components have good Python performance

### **Phase 2: Gradual Optimization**

**Profile and optimize incrementally:**
1. **Measure**: Use `cProfile` and `py-spy` to find bottlenecks
2. **Optimize**: Move hot paths to Rust one by one
3. **Benchmark**: Verify performance improvements

### **Phase 3: Mojo Migration Path**

**When Mojo stabilizes (1-2 years):**
```python
# Future: Mojo backend option
class MojoFileSystemBackend:
    """Mojo-powered backend for maximum performance."""
    
    def __init__(self):
        try:
            import aircher_intelligence_mojo
            self.mojo = aircher_intelligence_mojo
        except ImportError:
            raise ImportError("Mojo backend not available")
```

## Python Libraries for Good Performance

### **Recommended High-Performance Python Stack**

```python
# High-performance Python dependencies
dependencies = [
    # Vector operations - excellent performance
    "numpy>=1.24.0",           # BLAS/LAPACK optimized
    "scipy>=1.11.0",           # Scientific computing
    "scikit-learn>=1.3.0",     # ML algorithms
    
    # Embeddings - GPU accelerated
    "sentence-transformers>=2.2.0",  # Optimized transformers
    "transformers>=4.35.0",          # HuggingFace models
    
    # Async I/O - excellent performance  
    "aiofiles>=23.0.0",        # Async file operations
    "aiohttp>=3.9.0",          # Async HTTP
    "asyncio-mqtt>=0.16.0",    # Async messaging
    
    # Database - optimized drivers
    "aiosqlite>=0.19.0",       # Async SQLite
    "asyncpg>=0.29.0",         # Fast PostgreSQL driver
    
    # File watching - native performance
    "watchfiles>=0.21.0",      # Rust-based file watching
    
    # Regex - compiled patterns
    "regex>=2023.0.0",         # Optimized regex engine
    
    # JSON/Parsing - fast implementations
    "orjson>=3.9.0",          # Fast JSON parsing
    "msgpack>=1.0.0",         # Efficient serialization
]
```

## Backend Selection Logic

```python
# aircher_intelligence/backends/__init__.py
from typing import Type, Dict, Any
import logging

logger = logging.getLogger(__name__)

class BackendManager:
    """Manages backend selection based on availability and performance requirements."""
    
    def __init__(self, prefer_performance: bool = True):
        self.prefer_performance = prefer_performance
        self._backends = self._discover_backends()
    
    def get_filesystem_backend(self) -> FileSystemBackend:
        """Get the best available file system backend."""
        if self.prefer_performance:
            try:
                from .rust import RustFileSystemBackend
                logger.info("Using Rust filesystem backend for maximum performance")
                return RustFileSystemBackend()
            except ImportError:
                logger.warning("Rust backend not available, falling back to Python")
        
        from .python import PythonFileSystemBackend
        logger.info("Using Python filesystem backend")
        return PythonFileSystemBackend()
    
    def get_ast_backend(self) -> ASTBackend:
        """Get the best available AST parsing backend."""
        if self.prefer_performance:
            try:
                from .rust import RustASTBackend
                logger.info("Using Rust AST backend for maximum performance")
                return RustASTBackend()
            except ImportError:
                logger.warning("Rust AST backend not available, falling back to Python")
        
        from .python import PythonASTBackend
        logger.info("Using Python AST backend")
        return PythonASTBackend()
    
    def _discover_backends(self) -> Dict[str, Any]:
        """Discover available backends."""
        backends = {"python": True}
        
        try:
            import aircher_intelligence_core
            backends["rust"] = True
            logger.info("Rust performance backend available")
        except ImportError:
            backends["rust"] = False
            logger.info("Rust backend not available - install with pip install aircher-intelligence[rust]")
        
        try:
            import aircher_intelligence_mojo
            backends["mojo"] = True
            logger.info("Mojo performance backend available")
        except ImportError:
            backends["mojo"] = False
        
        return backends

# Usage in main server
backend_manager = BackendManager(prefer_performance=True)
filesystem_backend = backend_manager.get_filesystem_backend()
ast_backend = backend_manager.get_ast_backend()
```

## Performance Benchmarks to Target

| Operation | Pure Python | Python+Rust Target | Improvement |
|-----------|-------------|-------------------|-------------|
| **Walk 10k files** | 2-5s | 0.2-0.5s | **10x faster** |
| **Parse 100 files** | 5-15s | 0.5-1.5s | **10x faster** |
| **Pattern match in 1MB** | 100-200ms | 10-20ms | **10x faster** |
| **Dependency graph** | 10-30s | 1-3s | **10x faster** |

This modular approach gives us the **best of both worlds**: rapid Python development with Rust performance where it matters most! 🚀