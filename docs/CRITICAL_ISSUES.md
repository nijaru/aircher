# Critical Issues - January 2025

## 🚨 Critical Functionality Gaps

### 1. **Enhanced Search Display Not Integrated** ✅ FIXED
**Status**: Fixed - Enhanced display now properly integrated
**Impact**: Users now get syntax-highlighted search results
**Location**: `src/search_display.rs` - Enhanced display working

**Resolution**:
- Enhanced `SearchResultDisplay` to use thread-local `SyntaxHighlighter` instances
- Multi-line chunks now use AST-based tree-sitter highlighting
- Single-line results use optimized `BasicHighlighter`
- Added comprehensive tests in `tests/search_display_test.rs`
- Verified working with actual search queries

### 2. **Search Query Command Timeout** ✅ FIXED
**Status**: Fixed with temporary workaround
**Impact**: Search now works with limited index
**Location**: `aircher search query` command

**Issue Details**:
- Root cause: instant-distance takes too long to build HNSW index for large codebases
- Fixed by limiting index to first 1000 vectors until hnswlib-rs migration
- Search now completes in ~40 seconds for initial build
- Subsequent searches are instant (cached in memory)

**Resolution**:
- Modified `vector_search.rs` to avoid automatic index rebuild on load
- Added smart index build detection to only build when needed
- Implemented temporary vector limit (1000) for usable performance
- Full solution requires migration to hnswlib-rs (tracked separately)

### 3. **MCP Integration Has No CLI Interface** ⚠️
**Status**: Fully implemented but inaccessible
**Impact**: Cannot use any MCP functionality
**Location**: `src/mcp/*` - Complete implementation with no CLI

**Issue Details**:
- Complete MCP client implementation (stdio + HTTP transports)
- Mock and real clients fully functional
- Manager for multi-server orchestration ready
- BUT: No CLI commands to access any of it

**Required Actions**:
1. Add `mcp` subcommand to CLI
2. Implement server management commands (add, remove, list)
3. Add tool/resource discovery commands
4. Create testing commands for MCP operations

## 📊 Test Coverage Critical Gaps

### 1. **Search Display - 0% Coverage**
- No tests for `SyntaxHighlighter`
- No tests for `BasicHighlighter`
- No tests for formatting functions
- No tests for path truncation or content preview

### 2. **MCP Transport Layer - Minimal Coverage**
- No tests for actual message transmission
- No tests for process lifecycle (spawn/kill)
- No tests for connection recovery
- No tests for concurrent request handling

### 3. **MCP Real Client - Basic Coverage Only**
- Only construction tests exist
- No integration tests with actual processes
- No tests for error scenarios
- No tests for session initialization

## 🔧 Immediate Action Items

1. **Fix Search Integration** (HIGH PRIORITY)
   - Debug and fix search query timeout
   - Integrate EnhancedSearchDisplay into search command
   - Add comprehensive tests

2. **Add MCP CLI Commands** (HIGH PRIORITY)
   - Design CLI interface for MCP operations
   - Implement basic server management
   - Add tool discovery and execution

3. **Comprehensive Testing** (MEDIUM PRIORITY)
   - Unit tests for all display functions
   - Integration tests for MCP transports
   - End-to-end tests for search functionality

## 📝 Documentation Updates Needed

1. Update `status.md` to reflect actual implementation state
2. Document the disconnected Phase 7 implementation
3. Add MCP usage documentation (once CLI exists)
4. Update tasks.json with these critical issues

## 🎯 Root Cause Analysis

The primary issue appears to be **incomplete integration** of implemented features:
- Code is written but not connected to user-facing interfaces
- Features are tested in isolation but not end-to-end
- Missing the final "wiring" step to make features accessible

This suggests a need for better integration testing and end-to-end validation before marking phases as complete.