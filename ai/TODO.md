# TODO

## Current Sprint: Python POC - Knowledge Graph + Episodic Memory

### Strategic Pivot (2025-10-27)
**Decision**: Build Python POC to validate memory approach before continuing Rust implementation

**Why**:
- Validate research hypothesis: "Does memory improve agent performance?"
- Python allows faster iteration (1-2 weeks vs 4-6 weeks in Rust)
- Keep 86K lines of Rust investment (semantic search, tools, providers)
- Port proven design to Rust after validation

### High Priority (Week 1-2: POC)
- [x] Set up Python POC project structure ✅
- [ ] Implement knowledge graph extraction
  - [ ] Tree-sitter Rust parsing
  - [ ] Extract: files, functions, classes, imports
  - [ ] Build NetworkX graph (nodes + edges)
  - [ ] Query interface (what's in file X? what calls Y?)
- [ ] Implement episodic memory
  - [ ] SQLite database for action tracking
  - [ ] Record: tool calls, files touched, success/failure
  - [ ] Query: history, co-edit patterns
- [ ] Create benchmark harness
  - [ ] Test with/without memory
  - [ ] Metrics: tool calls, files examined, success rate
- [ ] Validate on real tasks
  - [ ] Bug fixing scenarios
  - [ ] Feature addition tasks
  - [ ] Refactoring operations

### After POC (If Successful)

**Port to Rust (3-4 weeks)**:
- [ ] Port knowledge graph builder to Rust
- [ ] Port episodic memory layer
- [ ] Integrate with existing Aircher infrastructure
- [ ] Wire up ACP protocol (stdio transport)
- [ ] Test in Zed editor

**If POC Shows >25% Improvement**:
- [ ] Write blog post series (4-5 posts)
- [ ] Consider academic paper submission
- [ ] Open source the memory system
- [ ] Contribute learnings to Aider/Continue.dev community

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
- ✅ Documentation reorganization complete (agent-contexts v0.1.1)
  - Created ai/ directory (TODO.md, STATUS.md, DECISIONS.md, RESEARCH.md)
  - Moved research findings to ai/research/
  - Eliminated internal/ directory (not needed for open-source)
  - Cleaned up docs/ structure (archived old planning directories)
  - Fixed external/agent-contexts submodule location
  - Removed deprecated pattern files (CODE_STANDARDS, etc.)
  - Updated all @internal/ references to @ai/ or @docs/

**Next Session**:
- Complete knowledge graph extraction (tree-sitter → NetworkX)
- Test graph queries on Aircher codebase
- Begin episodic memory implementation

## Notes
- Week 1 Success: 4 production tools (2,110+ lines, 21+ tests)
- Competitive parity: 17-21% → 23-27%
- Focus: Agent scaffolding (interfaces, guardrails, memory) not model reasoning
