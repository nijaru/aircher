# TODO

## Current Sprint: Week 1 Complete ✅ → Week 2 Starting

### High Priority
- [x] Reorganize documentation (agent-contexts structure) ✅ COMPLETE
  - [x] Create ai/ directory
  - [x] Move files to appropriate locations
  - [x] Update CLAUDE.md references
- [ ] Week 2: LM-Centric Tool Interfaces
  - [ ] Add windowing to read_file (max 100 lines)
  - [ ] Add linting/validation to edit_file (tree-sitter syntax check)
  - [ ] Limit search_code results (max 50, ranked)
- [ ] Week 2: Memory Integration
  - [ ] Wire DuckDBMemory to tool execution
  - [ ] Record every tool call to episodic memory
  - [ ] Basic pattern retrieval before execution

### Week 2 Goals (Code Understanding + Memory)

**Code Understanding Tools (50% effort)**:
- [ ] search_code - integrate semantic search with memory boost
- [ ] analyze_code - AST-based analysis
- [ ] find_references - symbol tracking
- [ ] find_definition - definition lookup

**Memory Activation (50% effort)**:
- [ ] Wire DuckDB to tool registry
- [ ] Record actions (tool, params, success, duration)
- [ ] Store successful patterns
- [ ] Query episodic memory for similar tasks
- [ ] Repository auto-scanning (Devin-style knowledge extraction)

### Week 3 Preview (ACP Protocol)
- [ ] stdio transport implementation (JSON-RPC)
- [ ] ACP Agent trait compliance
- [ ] Session management
- [ ] Test with Zed editor
- [ ] Deprecate TUI testing (use Zed going forward)

### Backlog
- [ ] Error guardrails (linting, auto-reject bad edits)
- [ ] Context management (last 5 interactions, collapse older)
- [ ] Intent classification operational
- [ ] Dynamic context management activation

## Daily Focus (2025-10-27)

**Completed Today**:
- ✅ Week 1 file tools complete (4/4)
- ✅ All internal docs updated
- ✅ Documentation reorganization complete (agent-contexts structure)
  - Created ai/ directory (TODO.md, STATUS.md, DECISIONS.md, RESEARCH.md)
  - Moved research findings to ai/research/
  - Updated CLAUDE.md and REFERENCE.md
  - Cleaned up consolidated files

**Next Session**:
- Plan Week 2 implementation (LM-centric interfaces + memory)
- Begin read_file windowing implementation
- Research tree-sitter validation patterns

## Notes
- Week 1 Success: 4 production tools (2,110+ lines, 21+ tests)
- Competitive parity: 17-21% → 23-27%
- Focus: Agent scaffolding (interfaces, guardrails, memory) not model reasoning
