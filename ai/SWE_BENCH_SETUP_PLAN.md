# SWE-bench Setup Plan

**Date**: 2025-10-30
**Goal**: Run Aircher agent on SWE-bench to validate capabilities

## Executive Summary

**Answer to your questions:**

1. ✅ **No frontend needed** - SWE-bench runs agents programmatically
2. ✅ **Use your Claude Max + Sonnet 4.5** - Perfect for this
3. ✅ **Authentication**: API key is sufficient (OAuth not needed)
4. ⏰ **Timeline**: 1-2 days setup + 4-6 hours running

## Understanding SWE-bench

### What It Is
- **Benchmark**: 2,294 real GitHub issues from 12 Python projects
- **Task**: Given a codebase + issue, generate a patch that fixes it
- **Evaluation**: Patches tested in Docker containers, scored pass/fail
- **Datasets**:
  - **SWE-bench Full**: 2,294 tasks (comprehensive)
  - **SWE-bench Verified**: 500 tasks (human-filtered, recommended)
  - **SWE-bench Lite**: 300 tasks (easier subset)

### Current SOTA
- **Grok 4**: 75.0%
- **GPT-5**: 74.9%
- **Claude Opus 4.1**: 74.5%
- **Target for us**: >35% would be competitive

## How SWE-bench Works

### Architecture
```
Issue → Agent generates patch → Harness tests patch → Pass/Fail
```

**Two approaches:**

### Approach 1: Use SWE-agent (Recommended) ⭐
**What**: Princeton's tool that runs LMs against SWE-bench
**Advantage**: Mature, battle-tested, SOTA on benchmark
**How it works**:
1. SWE-agent loads GitHub issue
2. Spins up Docker container with repository
3. Runs agent interactively (bash commands, file edits)
4. Agent fixes issue, creates patch
5. Harness evaluates patch

**Integration**:
- SWE-agent is written in Python
- Uses LangChain to call LLMs
- We'd need to add Aircher as a custom agent backend

### Approach 2: Direct Integration (More Work)
**What**: Write our own harness to run Aircher
**Advantage**: Full control, tests our actual agent
**Disadvantage**: Need to implement evaluation harness ourselves

**Steps**:
1. Load SWE-bench dataset
2. For each issue:
   - Clone repo, checkout correct commit
   - Spin up Docker container
   - Run Aircher via ACP protocol
   - Capture patch output
   - Test patch
3. Calculate pass rate

## Recommended Approach: Hybrid ⭐

**Use SWE-agent's harness, integrate Aircher as backend**

### Why This Works
1. **SWE-agent handles**:
   - Docker container management
   - Repository setup
   - Patch extraction
   - Test running
   - Result aggregation

2. **Aircher provides**:
   - Intelligent agent
   - Tool execution
   - Memory systems
   - Our architecture

### Integration Plan

**Step 1: Add Aircher Backend to SWE-agent**

SWE-agent architecture:
```python
# swe-agent uses LangChain to call LLMs
# We need to add a custom backend that calls Aircher via ACP

class AircherBackend(BaseModel):
    def __init__(self):
        # Start Aircher in ACP mode
        self.process = subprocess.Popen(
            ["aircher", "--acp"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        # Initialize ACP session
        self.session_id = self.initialize_session()

    def generate(self, messages):
        # Convert to ACP prompt request
        request = {
            "jsonrpc": "2.0",
            "method": "agent/prompt",
            "params": {
                "session_id": self.session_id,
                "content": convert_messages_to_acp(messages)
            },
            "id": str(uuid.uuid4())
        }

        # Send to Aircher via stdin
        self.process.stdin.write(json.dumps(request).encode())
        self.process.stdin.flush()

        # Read response from stdout
        response = json.loads(self.process.stdout.readline())

        return response["result"]["content"]
```

**Step 2: Configuration**

Create `keys.cfg`:
```yaml
# Model configuration
ANTHROPIC_API_KEY: "sk-ant-..."  # Your Max subscription key
model: "claude-sonnet-4.5"

# Aircher configuration
agent: "aircher"
agent_path: "/path/to/aircher/binary"
```

**Step 3: Run Evaluation**

```bash
# Install SWE-agent
git clone https://github.com/princeton-nlp/SWE-agent
cd SWE-agent
pip install -e .

# Copy our Aircher backend integration
cp /path/to/aircher_backend.py sweagent/agents/aircher.py

# Run on SWE-bench Lite (300 tasks, ~4 hours)
python run.py \
  --model aircher \
  --dataset swebench_lite \
  --config_file keys.cfg \
  --output_dir ./results
```

## Model Configuration

### ⚠️ CRITICAL: OAuth vs API Key

**API Key ≠ Max Subscription Usage**

Your Max subscription requires **OAuth tokens**, not API keys. See `ai/CLAUDE_OAUTH_SETUP.md` for details.

**Three Options:**

### Option 1: Use API Key (Quick Start, ~$90-120 cost) ⭐
```bash
# Accept per-token billing for initial validation
export ANTHROPIC_API_KEY="sk-ant-api03-..."
```
**Pros**: Works immediately, no implementation needed
**Cons**: ~$90-120 for SWE-bench Lite (not covered by Max sub)

### Option 2: Extract Claude Code Tokens (Hybrid, 2-3 hours)
```bash
# If you have Claude Code installed
cat ~/.local/share/claude-code/auth.json
# Copy refresh/access tokens to ~/.local/share/aircher/auth.json
```
**Pros**: Uses Max subscription, minimal implementation
**Cons**: Requires basic OAuth refresh logic in Aircher

### Option 3: Implement Full OAuth (4-6 hours)
Implement OAuth PKCE flow in Aircher
**Pros**: Proper Max subscription usage, reusable
**Cons**: Takes time before we can start testing

**Recommendation**: Start with Option 1 (API key) for quick validation, implement OAuth later for cost savings.

**Aircher configuration** (`~/.config/aircher/config.toml`):
```toml
[model]
provider = "anthropic"
model = "claude-sonnet-4.5"

[providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"  # For Option 1 (API key)
# OR (for Option 2/3 - OAuth):
# auth_file = "~/.local/share/aircher/auth.json"
```

### Why Sonnet 4.5?
- ✅ Best coding model (beats Opus 4.1)
- ✅ 200K context window
- ✅ Fast inference (~100 tokens/sec)
- ✅ Your Max sub removes rate limits
- ✅ ~$3 per task (vs $15 for Opus)

**Cost estimate for SWE-bench Lite (300 tasks)**:
- Input: ~50K tokens/task × 300 = 15M tokens → $45
- Output: ~10K tokens/task × 300 = 3M tokens → $45
- **Total**: ~$90-120 for full run

**Your Max subscription includes API credits**, so this should be covered.

## System Requirements

### Minimum (SWE-bench Lite - 300 tasks)
- **Storage**: 120GB free
- **RAM**: 16GB
- **CPU**: 8 cores
- **OS**: Linux (x86_64) or macOS
- **Docker**: Required
- **Time**: 4-6 hours

### Recommended (SWE-bench Verified - 500 tasks)
- **Storage**: 200GB free
- **RAM**: 32GB
- **CPU**: 16 cores
- **Time**: 8-12 hours

## Timeline

### Day 1: Setup (6-8 hours)
- [ ] Clean up target/ directory (reclaim 57GB)
- [ ] Install SWE-agent dependencies
- [ ] Build Aircher release binary
- [ ] Create Aircher backend integration for SWE-agent
- [ ] Test with single issue (validation)

### Day 2: Initial Run (4-6 hours)
- [ ] Run SWE-bench Lite (300 tasks)
- [ ] Monitor progress, fix issues
- [ ] Collect results

### Day 3: Analysis (2-4 hours)
- [ ] Analyze pass rate
- [ ] Review failure modes
- [ ] Document findings
- [ ] Compare vs SOTA

## Alternative: Quick Start (Simpler)

If SWE-agent integration is too complex, we can do a **manual validation first**:

### Manual SWE-bench Test (2 hours)
1. Pick 5-10 issues from SWE-bench Lite
2. Run Aircher manually on each
3. See if it generates correct patches
4. Test patches in repos
5. Calculate success rate

**Files needed**:
```bash
# Get dataset
python -c "
from datasets import load_dataset
ds = load_dataset('princeton-nlp/SWE-bench_Lite', split='test')
print(ds[0])  # First issue
"
```

## Next Steps

**Immediate (today)**:
1. Clean up target/ directory: `rm -rf target/` (reclaim 57GB)
2. Set up Claude API key: `export ANTHROPIC_API_KEY="..."`
3. Test Aircher with Claude: `cargo run` and verify model works

**Day 1 (tomorrow)**:
1. Clone SWE-agent: `git clone https://github.com/princeton-nlp/SWE-agent`
2. Install dependencies: `cd SWE-agent && pip install -e .`
3. Create Aircher backend integration

**Day 2 (start run)**:
1. Run on single issue (validation)
2. If works, run full SWE-bench Lite
3. Monitor and document

## Questions Answered

### Q: Do we need a frontend?
**A**: No. SWE-agent communicates with Aircher via ACP protocol (stdio). No UI needed.

### Q: What about the model?
**A**: Use your Claude Max subscription with Sonnet 4.5. Export `ANTHROPIC_API_KEY` environment variable. Aircher will use it automatically.

### Q: Should we use OAuth?
**A**: No need. API key authentication is simpler and works fine. Your Max subscription applies to API usage.

### Q: How long will this take?
**A**:
- Setup: 1 day
- Run (Lite): 4-6 hours
- Analysis: 2-4 hours
- **Total**: 2-3 days for complete validation

## Success Criteria

**Minimum**: Agent completes evaluation without crashing (any score)
**Good**: 25-35% on SWE-bench Lite (proves agent works)
**Excellent**: 35-45% on SWE-bench Lite (competitive baseline)
**Outstanding**: >45% (approaching SOTA)

## Resources

- **SWE-bench**: https://github.com/princeton-nlp/SWE-bench
- **SWE-agent**: https://github.com/princeton-nlp/SWE-agent
- **Leaderboard**: https://www.swebench.com/
- **Anthropic Console**: https://console.anthropic.com/
- **Our ACP implementation**: `src/server/stdio.rs`

## Risks and Mitigation

### Risk 1: ACP Integration Complex
**Mitigation**: Start with manual validation (5-10 issues)
**Fallback**: Use SWE-agent's OpenAI backend as template

### Risk 2: Docker Container Issues
**Mitigation**: Test single issue first, debug container setup
**Fallback**: Run without Docker (less reliable but faster debugging)

### Risk 3: Cost Overruns
**Mitigation**: Start with SWE-bench Lite (300 tasks, ~$90)
**Tracking**: Monitor API usage in Anthropic Console
**Your Max sub**: Should cover this with included credits

### Risk 4: Performance Too Slow
**Mitigation**: Use parallel workers (`--max_workers 4`)
**Optimization**: Cache embeddings, reuse containers

## Conclusion

**Recommended path**:
1. ✅ Use SWE-agent harness (battle-tested)
2. ✅ Integrate Aircher via ACP backend
3. ✅ Use your Claude Max + Sonnet 4.5
4. ✅ Start with SWE-bench Lite (300 tasks)
5. ✅ API key authentication (no OAuth needed)

**Timeline**: 2-3 days for complete validation

**Expected outcome**: Objective competitive positioning vs Claude Code, OpenHands, etc.
