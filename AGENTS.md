# Aircher - Intelligent ACP-Compatible Coding Agent

**AI-powered coding agent backend with novel memory architecture and ACP protocol support**

## Structure
```
src/aircher/           # Python 3.13+ package
├── agent/             # LangGraph agent implementation
├── memory/            # 3-layer memory (DuckDB + ChromaDB + Knowledge Graph)
├── protocol/          # ACP protocol implementation
├── tools/             # Tool execution framework
├── modes/             # READ/WRITE/ADMIN modes
└── config/            # Configuration management

ai/                    # AI agent working context (read FIRST)
├── STATUS.md          # Current state, blockers, next tasks
├── TODO.md            # Active tasks and backlog
├── PLAN.md            # Implementation roadmap with phase dependencies
├── DECISIONS.md       # Architectural rationale and tradeoffs
├── RESEARCH.md        # Research index ("for X, read Y")
├── research/          # 15 SOTA research files (5,155 lines)
├── design/            # Lightweight API specs (<100 lines each)
└── decisions/         # Superseded decisions archive
```

### AI Context Organization

**Purpose**: The `ai/` directory is the AI agent's workspace for maintaining project state across sessions.

**Session files** (ai/ root - read every session):
- `STATUS.md` - Current state, metrics, blockers (read FIRST)
- `TODO.md` - Active tasks only (completed tasks removed)
- `DECISIONS.md` - Active architectural decisions
- `RESEARCH.md` - Research findings index
- `PLAN.md` - Strategic roadmap with dependencies

**Reference files** (subdirectories - loaded only when needed):
- `research/` - Detailed SOTA analysis (15 files, 5,155 lines)
- `design/` - API specifications and design docs
- `decisions/` - Superseded/split decisions archive

**Token efficiency**: Session files kept current/active only for minimal token usage. Historical content pruned (git preserves all history). Detailed content in subdirectories loaded on demand.

**Workflow**: AI reads STATUS.md first → checks TODO.md → references RESEARCH.md index → loads specific research files as needed → updates STATUS.md on exit.

## Stack
- **Agent**: LangGraph + LangChain
- **Databases**: SQLite (sessions) + DuckDB (analytics) + ChromaDB (vectors)
- **Code Analysis**: tree-sitter + ast-grep
- **CLI**: Click + Rich
- **Package Manager**: uv (fast, modern)
- **Tools**: ripgrep, fd, sd, jq (bundled or fallback)

## Commands
```bash
uv sync --dev              # Install dependencies
uv run pytest tests/       # Run tests (180 tests, 100% pass)
uv run ruff check . --fix  # Lint and format
uv run ty src/ --strict    # Type checking

# ACP Server (IMPLEMENTED)
aircher serve                              # Run as ACP server (stdio)
aircher serve --model gpt-4o               # Use specific model
aircher serve --no-memory                  # Disable memory systems
aircher status                             # Show system status

# Future commands
uv run aircher run --mode read "Help me understand this code"
uv run aircher run --mode write "Add new feature"
uv run aircher run --admin --mode write "Full access, no confirmations"
```

## Workflow Rules (User Preferences)
- **NO AI attribution** in commits/PRs (strip manually)
- **Ask before**: PRs, publishing packages, force operations, resource deletion
- **Commit frequently**, push regularly (no ask for regular commits)
- **Never force push** to main/master
- **Delete files directly** (no archiving unless historical value)
- **/tmp**: Ephemeral only, delete after use, never commit
- **ai/ directory**: Read on session start, update on exit

## Package Management
- **Let uv choose versions**: `uv add requests` (not `uv add requests==2.31.0`)
- **Pin only when**: Reproducibility required, known breaking changes, or explicit request

## Testing Strategy
**Use TDD for**: Systems (DBs, memory), performance-critical code, complex logic
**Skip TDD for**: Simple fixes, prototypes, exploratory code, docs
**Workflow**: Plan → Red (test) → Green (implement) → Refactor → Validate

## Performance Claims Format
**Never compare different abstraction levels**. Always specify:
- Same features (ACID, durability, etc.)
- Same workload, realistic data
- 3+ runs, report median + caveats
- If >50x speedup: Explain WHY

Format: ✅ "X: 20x faster bulk inserts vs Y (both with durability). 10M rows. Random: 2-5x."

## Development Workflow

**Session Start**:
1. Read `ai/STATUS.md` (current phase, blockers, next tasks)
2. Read `ai/TODO.md` (weekly checkboxes)
3. Check `ai/RESEARCH.md` → find relevant research file
4. Navigate via `ai/research/README.md` for detailed guidance

**During Implementation**:
- `ai/PLAN.md` - Context (why, tradeoffs, risks)
- `ai/TODO.md` - Execution (weekly checkboxes)
- `ai/research/` - Technical details (schemas, algorithms, patterns)
- `ai/design/` - API surfaces (class signatures, integration points)

**Session End**:
- Update `ai/STATUS.md` blockers and phase
- Update `ai/TODO.md` checkboxes
- Add learnings to `ai/DECISIONS.md` if architectural

## Architecture Overview

**3-Layer Memory System** (60% tool call reduction validated in POC):
1. **DuckDB**: Episodic memory (tool executions, file interactions, learned patterns)
2. **ChromaDB**: Vector search (semantic code retrieval, sentence-transformers)
3. **Knowledge Graph**: Code structure (tree-sitter extraction, NetworkX)

**Agent Modes**:
- **READ**: Safe mode, file reading only
- **WRITE**: File modification with confirmation
- **ADMIN**: Full access without confirmations (--admin flag)

**Protocols**:
- **ACP**: Universal frontend compatibility (Toad, Zed, Neovim, etc.)
- **MCP**: Planned for extensibility (not priority)

## Current Phase
**Phase 6**: Terminal-Bench evaluation (benchmarking & optimization)

**Completed** (Phases 1-5):
- ✅ Memory systems (DuckDB + ChromaDB + Knowledge Graph) - 152 tests
- ✅ LLM integration, sub-agents, dynamic context management
- ✅ Model routing, cost tracking, multi-provider support - 166 tests
- ✅ ACP protocol (stdio transport, JSON-RPC server) - 180 tests

**In Progress**:
- Terminal-Bench adapter development
- Baseline evaluation and optimization

**Deferred** (non-critical):
- Integration testing with ACP clients (manual testing works)
- Streaming responses (optimize if benchmarks show need)

**Read ai/STATUS.md for detailed current state and blockers**

## Reference Repositories
- `~/github/sst/opencode` - Production agent reference (28.8k stars)
- `~/github/badlogic/pi-mono` - Bash tools philosophy
- `~/github/openai/codex` - OpenAI agent reference
- `~/github/google-gemini/gemini-cli` - Google CLI reference
