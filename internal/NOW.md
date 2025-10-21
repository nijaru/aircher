# NOW - Current Sprint

**Last Updated**: 2025-10-27
**Current Sprint**: Week 1 of 10 - Real Tool Implementation (File Operations)
**Next Sprint**: Week 2 - Real Tool Implementation (Code Understanding)

See `AGENT_FIRST_ROADMAP.md` for complete 10-week plan.

## 🎯 Mission (Reminder)

Build an intelligent ACP-compatible agent backend. NOT building UI - using Zed/JetBrains/Neovim for frontend.

**Focus**: Agent intelligence research and implementation
**Output**: Research paper + working agent backend via ACP

## 📊 Week 1 Goals (File Operations)

### **Problem**: 9/10 tools are stubs returning fake JSON
### **Solution**: Implement 4 real file operation tools this week

**Success Criteria**:
- [ ] 4 production-quality file tools working
- [ ] Can read/write/edit/list files with real value
- [ ] No crashes or data loss
- [ ] Test coverage for each tool

## ✅ Week 1 Tasks (Day-by-Day)

### Day 1-2: Real `read_file` Tool
**Location**: `src/agent/tools/file_tools.rs` or new file

**Requirements**:
- [ ] Read file from disk with error handling
- [ ] Syntax highlighting via tree-sitter (already have 19+ languages)
- [ ] Context extraction (surrounding lines for functions/classes)
- [ ] Smart truncation for large files (configurable limit)
- [ ] Proper permissions checking
- [ ] Return structured output (content + metadata)

**Testing**:
- [ ] Unit tests (various file types, edge cases)
- [ ] Integration test with agent

### Day 2-3: Real `write_file` Tool
**Location**: Same as `read_file`

**Requirements**:
- [ ] Automatic backup before write
- [ ] Create parent directories if needed
- [ ] Protected file detection (no overwriting lib.rs, main.rs, Cargo.toml without explicit confirmation)
- [ ] Atomic writes with rollback on failure
- [ ] Verify write succeeded
- [ ] Return success/failure with details

**Testing**:
- [ ] Unit tests (new files, overwrites, protected files)
- [ ] Test rollback functionality
- [ ] Integration test

### Day 3-4: Real `edit_file` Tool
**Location**: Same

**Requirements**:
- [ ] Line-based editing (insert, replace, delete lines)
- [ ] Context-aware changes (understand surrounding code)
- [ ] Change validation (syntax check if possible)
- [ ] Diff preview before applying
- [ ] Support multiple edits in one operation
- [ ] Backup + rollback

**Testing**:
- [ ] Unit tests (various edit patterns)
- [ ] Test validation logic
- [ ] Integration test

### Day 4-5: Real `list_files` Tool
**Location**: Same

**Requirements**:
- [ ] Respect .gitignore patterns
- [ ] File metadata (size, modified time, type)
- [ ] Smart sorting (directories first, then by relevance)
- [ ] Configurable depth limit
- [ ] Filter options (by extension, name pattern)
- [ ] Readable output format

**Testing**:
- [ ] Unit tests (various directory structures)
- [ ] Test gitignore respect
- [ ] Integration test

### Day 5-7: Integration & Polish
- [ ] End-to-end testing of all 4 tools
- [ ] Performance optimization
- [ ] Documentation (doc comments)
- [ ] Fix bugs found during testing
- [ ] Code review and cleanup

## 📈 Current Status (Honest Assessment)

### What Works ✅
- **Semantic Search**: Production-ready (6,468 vectors, 19+ languages)
- **Intelligence Framework**: 210+ Rust files implemented
- **Multi-Provider Auth**: OpenAI, Anthropic, Gemini, Ollama
- **Tree-sitter Parsing**: 19+ languages for syntax highlighting
- **Architecture**: Designed and documented

### What Doesn't Work ❌
- **9/10 tools are stubs** - this week fixes 4 of them
- **ACP Protocol**: Not implemented (Week 3)
- **Intent Classification**: Code exists but not operational (Week 5)
- **Dynamic Context**: Implemented but not tested (Week 6)

### Current Competitive Position
- **16-20% feature parity** with Claude Code
- **Strong foundation** but minimal user value
- **This week**: Move to 25-30% parity (4 real tools working)

## 🔬 Research Context

### Why These Tools Matter
**For Research Paper**:
- Need working tools to test intent classification
- Tools are baseline for benchmarking vs Claude Code
- Must work reliably before testing intelligence features

**For Validation**:
- Can't measure agent quality without real tool execution
- Benchmarks require actual task completion
- User studies need functional tools

## 🚫 NOT Doing This Week

- ❌ TUI development or polish
- ❌ ACP protocol implementation (that's Week 3)
- ❌ Intelligence features (Week 5-6)
- ❌ Benchmarking (Week 7-8)
- ❌ Any UI work whatsoever

**Focus**: Just get 4 file tools working correctly.

## 📋 Daily Checklist

### Every Day
- [ ] Work on assigned tool(s) for that day
- [ ] Write tests as you go
- [ ] Document with doc comments
- [ ] Keep zero compiler warnings
- [ ] Update this file with progress

### End of Week
- [ ] All 4 tools working
- [ ] Tests passing
- [ ] Update PROJECT_REALITY.md with new parity estimate
- [ ] Document any decisions in DECISIONS.md
- [ ] Update KNOWLEDGE.md with learnings

## 🎯 Success Metrics (Week 1)

### Minimum Success
- [ ] 4 tools implemented and working
- [ ] No crashes or data loss in testing
- [ ] Basic functionality validated

### Good Success
- [ ] Above + comprehensive tests
- [ ] Above + good documentation
- [ ] Above + clean code (zero warnings)

### Excellent Success
- [ ] Above + performance optimized
- [ ] Above + edge cases handled
- [ ] Above + integration tests passing
- [ ] Ready for Week 2 (code tools)

## 🔄 Next Week Preview (Week 2)

**Code Understanding Tools:**
- `search_code` - Leverage existing semantic search
- `analyze_code` - AST-based analysis
- `find_references` - Symbol tracking
- `get_definition` - Definition lookup

**Week 2 Success**: 8/10 tools real (vs 1/10 currently)

## 📝 Notes & Blockers

### Current Blockers
- None yet

### Questions
- None yet

### Decisions Needed
- None yet

---

**Today's Focus**: [Update daily - which tool are you working on?]

**Blocker Status**: [Update if blocked on anything]

**Tomorrow's Plan**: [What's next?]
