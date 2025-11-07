# Aircher Agent Modes

**Created**: 2025-10-30
**Purpose**: Document available agent modes and how to test with file tools

## Available Modes

### 1. CLI Mode (Single-Shot) ✅ WORKING
**Command**: `./target/release/aircher "message"`
**Features**:
- Single-shot responses
- No file tool access
- Fast, lightweight
- **Tested**: 5/5 bug identification tests passed (100%)

**Use case**: Quick questions, bug identification from inline code

**Example**:
```bash
./target/release/aircher --provider ollama --model "gpt-oss:latest" "What's 2+2?"
```

### 2. ACP Mode (Agent Client Protocol) ⚠️ REQUIRES REBUILD
**Command**: `./target/release/aircher --acp`
**Build**: `cargo build --release --features acp`
**Features**:
- Full agent capabilities with file tools
- JSON-RPC over stdin/stdout
- Designed for editor integration (Zed, VSCode, Neovim)
- ✅ Built successfully with ACP feature enabled

**Use case**: Integration with editors, full agent functionality

**Status**: Compiled successfully but requires ACP-compatible client to use

### 3. TUI Mode (Terminal UI) ❓ STATUS UNKNOWN
**Command**: `./target/release/aircher` (no arguments)
**Features**:
- Interactive terminal UI
- Full agent capabilities
- File tools available

**Status**: Code exists but may not be functional (user reported "we dont have a tui")

## Agent Architecture Findings

### Tool Registry
**Location**: `src/agent/tools/mod.rs`
**Status**: ✅ All tools registered by default

**Available tools**:
- File operations: ReadFileTool, WriteFileTool, EditFileTool, ListFilesTool
- Code analysis: SearchCodeTool, AnalyzeCodeTool, FindDefinitionTool
- System: RunCommandTool
- LSP tools: CodeCompletionTool, HoverTool, GoToDefinitionTool, FindReferencesTool, etc.
- Git tools: SmartCommitTool, CreatePRTool, BranchManagementTool, TestRunnerTool
- Web tools: WebBrowsingTool, WebSearchTool
- Build tools: BuildSystemTool

### Model Configuration Issue

**Problem**: `test_agent_pipeline` binary uses "claude-sonnet-4-5" model regardless of config

**Root cause**: Model routing happens in `Agent::process_message()` at runtime, but the provider was already initialized with a specific model at agent creation time.

**Config structure**:
```toml
[global]
default_provider = "ollama"
default_model = "gpt-oss:latest"

[model]
provider = "ollama"
model = "gpt-oss:latest"
```

## Test Results Summary

### Simple Bug Fixing (CLI Mode) ✅ 100% SUCCESS
**Tests**: 5 Python bugs (syntax, logic, import, type, off-by-one)
**Success Rate**: 5/5 = 100%
**Average**: 506 tokens, ~10 seconds per test
**Limitation**: Single-shot mode, no file tool access

**Files**: See `/tmp/*_bug.py` and `TEST_RESULTS.md`

### Agent Pipeline Test ❌ MODEL CONFIGURATION ISSUE
**Test**: `./target/release/test_agent_pipeline`
**Status**: Fails - tries to use "claude-sonnet-4-5" in Ollama
**Root cause**: Provider/model initialization sequence issue

## Recommended Next Steps

### Option 1: Simple File-Based Testing (EASIEST)
1. Create bug files in current directory
2. Ask Aircher via CLI to read and fix specific files
3. Verify fixes manually

**Example**:
```bash
# Create bug file
cat > bug.py << 'EOF'
def get_last_three(items):
    return items[-4:-1]  # Bug: should be [-3:]
EOF

# Ask Aircher for help (single-shot)
./target/release/aircher --provider ollama --model "gpt-oss:latest" \
  "Read bug.py and explain how to fix the off-by-one error"
```

**Limitation**: Agent can't directly read/write files in CLI mode

### Option 2: Use ACP Mode (REQUIRES CLIENT)
1. Build with ACP: `cargo build --release --features acp`
2. Write simple ACP client script
3. Test file tools through ACP protocol

**Limitation**: Need to write an ACP client

### Option 3: Fix test_agent_pipeline (REQUIRES DEBUG)
1. Debug model routing in Agent initialization
2. Ensure config overrides work properly
3. Run full agent pipeline tests

**Limitation**: Complex debugging required

## For SWE-bench Testing

**Requirement**: Agent must be able to:
1. Read files from disk ✅ (tools exist)
2. Make edits ✅ (tools exist)
3. Run commands (tests, builds) ✅ (tools exist)
4. Check results ✅ (tools exist)

**Blocker**: Need agent mode (not single-shot CLI) to access file tools

**Best path forward**: Either fix test_agent_pipeline or write simple ACP client
