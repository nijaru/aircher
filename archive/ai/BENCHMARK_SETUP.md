# Benchmark Setup - Practical Validation

**Created**: 2025-10-30
**Goal**: Run Aircher against standardized agent benchmarks in isolated containers
**Approach**: Start small, validate integration, then scale up

## Why Benchmarks Over Theoretical Validation

**Reality**: We have 9,100 lines of architecture that:
- ✅ Compiles successfully
- ✅ Passes 92.5% of tests (258/279)
- ✅ Has hybrid architecture wired correctly (17/17 integration tests)
- ❌ But no proof it performs well on **actual agent tasks**

**Solution**: Run against standardized benchmarks that test real-world agent capabilities

## Recommended Benchmark: Terminal-Bench

**Why Terminal-Bench First**:
1. ✅ Terminal/CLI focused (aligns with Aircher's ACP design)
2. ✅ Only 80 tasks (manageable, can run in hours not days)
3. ✅ Published leaderboard (credible comparison)
4. ✅ Current SOTA: Factory Droid 58.8%, Claude Code 43.2%
5. ✅ Tests actual agent capabilities: file ops, git, debugging, refactoring

**Target**: Beat Claude Code's 43.2% baseline (realistic first goal)
**Stretch**: Approach Factory Droid's 58.8% (would be competitive)

## Containerized Setup (Isolation + Safety)

### Why Containers:
- ✅ Isolated file system (won't pollute local machine)
- ✅ Repeatable environment (same setup every run)
- ✅ Clean state per task (no cross-contamination)
- ✅ Safe for file operations (delete container after)

### Architecture:
```
Docker Container
├── Terminal-Bench harness (Node.js)
├── Aircher binary (Rust, --release)
├── Test workspace (isolated /workspace)
└── Results output (mounted volume)
```

### Dockerfile:
```dockerfile
FROM rust:1.70-slim as builder

# Build Aircher
WORKDIR /build
COPY . .
RUN cargo build --release --bin aircher

FROM node:18-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Terminal-Bench
WORKDIR /tbench
RUN npm install -g terminal-bench

# Copy Aircher binary
COPY --from=builder /build/target/release/aircher /usr/local/bin/aircher

# Create workspace for test runs
WORKDIR /workspace

# Entry point runs benchmark
CMD ["tbench", "run", "--agent", "aircher", "--output", "/results"]
```

## Step-by-Step Integration Plan

### Phase 1: Validate Integration (10 tasks, 1-2 hours)

**Goal**: Prove Aircher can communicate with benchmark harness

**Steps**:
1. **Build Docker image**:
   ```bash
   cd /Users/nick/github/nijaru/aircher
   docker build -t aircher-bench:latest -f Dockerfile.bench .
   ```

2. **Run minimal test** (10 tasks):
   ```bash
   docker run --rm \
     -v $(pwd)/benchmark-results:/results \
     aircher-bench:latest \
     tbench run --agent aircher --tasks 10 --dataset core-v0
   ```

3. **Check results**:
   ```bash
   cat benchmark-results/summary.json
   # Expected: Some pass/fail results, ACP communication working
   ```

**Success Criteria**:
- ✅ Container runs without crashing
- ✅ Aircher responds to ACP protocol
- ✅ At least 1 task completes (pass or fail doesn't matter yet)
- ✅ Results written to JSON

**If This Fails**: Fix ACP integration, not architecture

### Phase 2: Baseline Run (80 tasks, 4-6 hours)

**Goal**: Establish actual performance baseline

**Steps**:
1. **Run full Terminal-Bench**:
   ```bash
   docker run --rm \
     -v $(pwd)/benchmark-results:/results \
     --name aircher-baseline \
     aircher-bench:latest \
     tbench run --agent aircher --dataset core-v0 --output /results/baseline.json
   ```

2. **Monitor progress**:
   ```bash
   docker logs -f aircher-baseline
   ```

3. **Analyze results**:
   ```bash
   tbench report --input benchmark-results/baseline.json --format markdown > benchmark-results/baseline-report.md
   ```

**Key Metrics**:
- **Overall success rate**: X% (compare to Claude Code 43.2%, Factory Droid 58.8%)
- **Tool call efficiency**: Average calls per task
- **Execution time**: Average seconds per task
- **Token usage**: Total tokens consumed
- **Failure modes**: What types of tasks fail

**Expected Baseline** (realistic):
- First run: 25-35% (below Claude Code, but working)
- After fixes: 35-45% (competitive with Claude Code)

### Phase 3: Optimization (Optional, based on results)

**If baseline is <35%**: Focus on fixing critical issues
- Tool execution errors
- ACP communication problems
- Missing tool implementations

**If baseline is 35-45%**: Competitive, document and move on
- Memory systems likely not impacting benchmark yet
- Hybrid architecture validated as functional

**If baseline is >45%**: Strong performance, analyze what's working
- Which features contributed?
- Can we extrapolate to full agent workflows?

## Alternative: SWE-bench (If Terminal-Bench Goes Well)

**Setup**: Similar containerized approach
**Dataset**: SWE-bench Verified (500 tasks) or Lite (300 tasks)
**Timeline**: 1-2 days for setup, 8-12 hours for run
**Target**: 25-35% baseline (70%+ is SOTA, unrealistic first try)

**Advantage**: Industry-standard benchmark, more credible for research
**Disadvantage**: Harder tasks, requires more sophisticated repository handling

## What This Proves vs What It Doesn't

### ✅ Benchmarks WILL Prove:
1. Agent can complete real tasks (file operations, git, debugging)
2. ACP protocol works in practice (not just unit tests)
3. Tool implementations are functional (not just stubs)
4. Architecture doesn't crash under load (stability)
5. Competitive positioning vs other agents (objective comparison)

### ❌ Benchmarks WON'T Prove:
1. 60% tool call reduction (would need A/B comparison with/without memory)
2. 90% research speedup (benchmarks don't isolate research tasks)
3. Memory systems improve performance (benchmarks are stateless tasks)
4. Hybrid architecture advantages (need long-running workflows to see benefits)

### 🤔 What We Learn Either Way:
- **If score is low (<35%)**: Core functionality needs work, forget optimizations
- **If score is medium (35-45%)**: Agent works, competitive baseline established
- **If score is high (>45%)**: Agent is strong, validate what's working

## Pragmatic Success Criteria

**Minimum Viable**: Agent completes benchmarks without crashing (any score)
**Realistic Goal**: 35-45% on Terminal-Bench (match Claude Code)
**Stretch Goal**: 45-55% on Terminal-Bench (approach Factory Droid)

**Most Important**: Honest assessment of what works and what needs improvement

## Timeline Estimate

- **Setup + Docker**: 2-4 hours
- **Phase 1 (10 tasks)**: 1-2 hours
- **Phase 2 (80 tasks)**: 4-6 hours
- **Analysis**: 1-2 hours
- **Total**: 1-2 days for complete validation

## Next Steps

1. **Immediate**: Create Dockerfile.bench
2. **Day 1**: Run Phase 1 (10 tasks) to validate integration
3. **Day 2**: Run Phase 2 (80 tasks) if Phase 1 succeeds
4. **Day 3**: Analyze results, document findings

**Decision Point**: If Phase 1 fails, fix core issues. If Phase 2 shows <25%, revisit architecture. If >35%, document and declare success.

## Files to Create

1. `Dockerfile.bench` - Containerized benchmark environment
2. `scripts/run-benchmark.sh` - Helper script for running benchmarks
3. `benchmark-results/` - Output directory (gitignored)
4. `ai/BENCHMARK_RESULTS.md` - Document actual results when available

## Resources

- **Terminal-Bench**: https://www.tbench.ai/
- **SWE-bench**: https://www.swebench.com/
- **Existing plan**: ai/research/benchmark-integration-plan.md (comprehensive 500+ line plan)
- **Test results**: ai/TEST_RESULTS_2025-10-30.md (current validation status)

## ⚠️ Blockers Discovered (Oct 30, 2025)

### Issue 1: Terminal-Bench Not Published
**Problem**: `npm install -g @terminal-bench/cli` returns 404
**Evidence**: `npm ERR! 404  '@terminal-bench/cli@*' is not in this registry.`
**Impact**: Cannot run Terminal-Bench until package is published
**Workaround**: Manual validation tasks (see ai/MANUAL_VALIDATION_TASKS.md)

### Issue 2: Rust Version Mismatch
**Problem**: Dockerfile uses `rust:1.70-slim`, but Cargo.lock requires Rust 1.79+
**Evidence**: `lock file version 4 was found, but this version of Cargo does not understand`
**Impact**: Docker build fails to compile project
**Fix**: Update Dockerfile to use `rust:1.79-slim` or newer

### Issue 3: Docker Build Size (FIXED ✅)
**Problem**: Docker transferred 61GB during build (target/ directory = 57GB)
**Evidence**:
```
$ du -sh /Users/nick/github/nijaru/aircher
57G	/Users/nick/github/nijaru/aircher

$ du -sh target/*
52G	target/debug
4.9G	target/release
```
**Impact**: Slow builds, wasted bandwidth
**Fix**: Created `.dockerignore` excluding target/, models/, and other large directories

### Issue 4: Model Files in Docker Image
**Problem**: models/ directory (261MB) contains swerank-embed-small.safetensors
**Impact**: Larger Docker images
**Fix**: `.dockerignore` excludes models/, download inside container if needed

## Alternative Validation Strategies

Given the blockers above, consider these alternatives:

### Option A: Manual Validation Tasks ⭐ RECOMMENDED
**Why**: Can run immediately, validates real capabilities
**How**: See ai/MANUAL_VALIDATION_TASKS.md (5 realistic tasks)
**Time**: 1-2 hours
**Proves**: Agent can complete actual coding tasks

### Option B: Wait for Terminal-Bench Release
**Why**: Industry-standard benchmark, public leaderboard
**How**: Monitor https://www.tbench.ai/ for npm package release
**Time**: Unknown (weeks? months?)
**Proves**: Competitive positioning vs other agents

### Option C: SWE-bench Verified
**Why**: More credible than Terminal-Bench, widely recognized
**How**: Set up SWE-bench harness (Python evaluation framework)
**Time**: 1-2 days setup + 4-6 hours running
**Proves**: Real-world bug fixing capability

### Option D: Context Awareness Improvement
**Why**: High-value feature from user insight
**How**: Implement Phase 1 from ai/CONTEXT_AWARENESS_IMPROVEMENT.md
**Time**: 1 hour
**Proves**: Agent can make better decisions with context visibility
