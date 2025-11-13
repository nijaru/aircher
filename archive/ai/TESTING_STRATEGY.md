# Local Testing Strategy with Ollama

**Purpose**: Safe, repeatable testing of integrated Week 7-8 hybrid architecture without API costs

**Status**: Ready for implementation (Critical Issue #4)

## Why Ollama?

**Benefits**:
- ✅ Free - No API costs, unlimited testing
- ✅ Local - Fast iteration, no rate limits
- ✅ Repeatable - Same model, same results
- ✅ Safe - No production data sent to external APIs
- ✅ Can test tool execution - Real file operations in isolated environment

## Prerequisites

### Install Ollama

```bash
# macOS
brew install ollama

# Linux
curl -fsSL https://ollama.com/install.sh | sh

# Or download from https://ollama.com/download
```

### Pull Test Models

```bash
# Pull coding model (recommended)
ollama pull qwen2.5-coder:latest

# Or smaller model for quick tests
ollama pull llama3.2:latest

# Or instruction-following model
ollama pull mistral:latest
```

## Running Ollama

### Start Ollama Server

```bash
# Terminal 1: Start Ollama
ollama serve

# Check status
ollama list
```

### Test Model Works

```bash
# Terminal 2: Test query
ollama run qwen2.5-coder "Write a hello world in Rust"

# If successful, you'll see generated code
```

## Testing Aircher with Ollama

### 1. Configure for Ollama

Edit `config.toml` or set environment variable:

```bash
export OLLAMA_HOST="http://localhost:11434"
```

Or in your config file:
```toml
[providers.ollama]
base_url = "http://localhost:11434"
default_model = "qwen2.5-coder"
```

### 2. Test Basic Agent Execution

```bash
# Compile Aircher
cargo build --release

# Run with Ollama provider
RUST_LOG=info ./target/release/aircher --provider ollama --model qwen2.5-coder

# Test basic query
> "Hello, can you help me understand this codebase?"

# Test file operations
> "Read the file src/main.rs"

# Test code search
> "Find all authentication code in src/"
```

### 3. Test Integrated Week 7-8 Features

**Event Bus + LSP Integration**:
```bash
# Test file edit triggers event
> "Edit src/test.rs and add a comment"

# Check logs for:
# - "FileChanged event emitted"
# - "LSP manager received FileChanged event"
```

**Mode Enforcement**:
```bash
# Start in Plan mode (default)
> "Write a new file test.txt"

# Should see: "Tool 'write_file' not allowed in Plan mode"

# Switch to Build mode
> "/mode build"

# Now try again
> "Write a new file test.txt"

# Should succeed
```

**Model Router**:
```bash
# Check logs for model selection
RUST_LOG=debug ./target/release/aircher --provider ollama

# Should see:
# - "Selected Explorer agent for this task"
# - "Task complexity: Low"
# - "Model router selected: ollama/qwen2.5-coder"
# - "Recorded model usage: X total tokens"
```

**Specialized Agent Selection**:
```bash
# Test Explorer agent
> "Analyze the architecture of this codebase"

# Check logs: "Selected Explorer agent"

# Test Builder agent
> "Implement a new function to parse JSON"

# Check logs: "Selected Builder agent"

# Test Debugger agent
> "Fix the compilation error in src/main.rs"

# Check logs: "Selected Debugger agent"
```

**Research Sub-Agents**:
```bash
# Test parallel research
> "Find all error handling patterns in the codebase"

# Check logs:
# - "Explorer agent detected research task"
# - "Spawning research sub-agents"
# - "Decomposed into X sub-tasks"
# - "Research complete: X sub-agents returned results"

# Verify real results (not fake data)
# Should see actual file listings, search results, etc.
```

## Test Scenarios

### Scenario 1: Read-Only Exploration

```bash
# Purpose: Test Plan mode + Explorer agent + Research sub-agents

ollama run qwen2.5-coder

Aircher prompt:
> "Explore the codebase and tell me what the main components are"

Expected:
- Agent stays in Plan mode
- Explorer agent selected
- Research sub-agents spawned (if query triggers parallel search)
- No file modifications
- Real findings from file_searcher sub-agents
```

### Scenario 2: Code Implementation

```bash
# Purpose: Test Build mode + Builder agent + Model routing

ollama run qwen2.5-coder

Aircher prompt:
> "/mode build"
> "Add a new helper function to src/utils.rs that formats dates"

Expected:
- Builder agent selected
- Model router selects appropriate model (Sonnet for medium complexity)
- write_file or edit_file executes
- FileChanged event emitted
- LSP manager receives event (if LSP servers running)
- Token usage recorded to model router
```

### Scenario 3: Bug Fixing

```bash
# Purpose: Test Debugger agent + Git snapshots + LSP feedback

ollama run qwen2.5-coder

Aircher prompt:
> "Fix the type error in src/agent/core.rs:123"

Expected:
- Debugger agent selected
- Git snapshot created before edit
- edit_file executes
- FileChanged event emitted
- LSP diagnostics received (if rust-analyzer running)
- If fix fails, can rollback to snapshot
```

### Scenario 4: Cost Tracking

```bash
# Purpose: Verify model router tracks all usage

ollama run qwen2.5-coder

# Run multiple queries of varying complexity
> "Read src/main.rs"  # Low complexity
> "Analyze the architecture"  # High complexity
> "Add a comment to src/lib.rs"  # Medium complexity

# Check model usage report
> "/stats"

Expected:
- Total tokens tracked
- Per-model breakdown
- Cost estimates (even for local models, for testing)
- Cost savings percentage (vs always using most expensive)
```

## Validation Checklist

After testing, verify these claims:

### Week 7 Integration
- [ ] Event bus: File operations emit FileChanged events (check logs)
- [ ] LSP manager: Receives FileChanged events (check logs)
- [ ] Mode enforcement: Plan mode blocks write operations (test manually)
- [ ] Git snapshots: Created before risky operations (check logs)
- [ ] Model router: Selects models based on complexity (check logs)
- [ ] Token tracking: Records usage after every LLM call (check logs)

### Week 8 Integration
- [ ] Specialized agents: Explorer/Builder/Debugger selected based on intent (check logs)
- [ ] Research sub-agents: Spawned for research queries (check logs)
- [ ] Real execution: Sub-agents return actual file/search results (verify output)
- [ ] No coding sub-agents: Builder/Debugger never spawn sub-agents (check logs)

### Performance Metrics
- [ ] Measure tool call count for research task (target: 60% reduction vs baseline)
- [ ] Measure research speed with sub-agents (target: 90% improvement vs serial)
- [ ] Verify sub-agents execute in parallel (check timestamps in logs)
- [ ] Confirm no sub-agents for coding tasks (0% usage)

## Cleanup

After testing:

```bash
# Stop Ollama
pkill ollama

# Remove test data
rm -rf /tmp/aircher-test-*

# Or use git to reset if testing in actual workspace
git reset --hard HEAD
git clean -fd
```

## Container-Based Isolation (Alternative)

For fully isolated testing:

```dockerfile
# Dockerfile.test
FROM rust:latest

# Install Ollama
RUN curl -fsSL https://ollama.com/install.sh | sh

# Copy project
COPY . /app
WORKDIR /app

# Pull test model
RUN ollama serve & sleep 5 && ollama pull qwen2.5-coder

# Run tests
CMD ["cargo", "test", "--workspace"]
```

Build and run:
```bash
docker build -f Dockerfile.test -t aircher-test .
docker run --rm aircher-test
```

## Troubleshooting

**Ollama not responding**:
```bash
# Check if running
ps aux | grep ollama

# Restart
killall ollama
ollama serve
```

**Model not found**:
```bash
# List available models
ollama list

# Pull if needed
ollama pull qwen2.5-coder
```

**Connection refused**:
```bash
# Check Ollama is listening
curl http://localhost:11434/api/tags

# Should return JSON with model list
```

**Out of memory**:
```bash
# Use smaller model
ollama pull llama3.2:1b

# Or increase Docker memory limit
```

## Next Steps (Week 9)

Once local testing validates integration:

1. **Run benchmark tasks** (from INTEGRATION_REVIEW.md):
   - Multi-file refactoring
   - Bug fixing workflow
   - New feature implementation
   - Codebase exploration

2. **Collect metrics**:
   - Tool calls per task
   - Time to completion
   - Token usage
   - Sub-agent spawn counts

3. **Compare with Claude Code**:
   - Same tasks, same codebase
   - Measure 60% tool reduction claim
   - Measure 90% research speedup claim
   - Measure 40% cost savings claim

4. **Document findings**:
   - Update INTEGRATION_REVIEW.md with results
   - Create benchmark report
   - Prepare for research paper (Week 10)

## Summary

**Testing Strategy**: Use Ollama for free, local, repeatable testing of all integrated Week 7-8 features

**Key Benefits**:
- No API costs
- Fast iteration
- Safe (no external data)
- Can test real tool execution
- Validates all 7 hybrid architecture components

**Status**: Ready to implement - just need to install Ollama and follow test scenarios above
