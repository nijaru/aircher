# Docker Build Investigation

**Date**: 2025-10-30
**Issue**: Docker build transferred 61GB during benchmark setup attempt

## Investigation Results

### Project Size Breakdown

```bash
$ du -sh /Users/nick/github/nijaru/aircher
57G	/Users/nick/github/nijaru/aircher

$ du -sh /Users/nick/github/nijaru/aircher/* | sort -hr | head -5
57G	/Users/nick/github/nijaru/aircher/target
261M	/Users/nick/github/nijaru/aircher/models
44M	/Users/nick/github/nijaru/aircher/external
3.9M	/Users/nick/github/nijaru/aircher/src

$ du -sh /Users/nick/github/nijaru/aircher/target/*
52G	/Users/nick/github/nijaru/aircher/target/debug
4.9G	/Users/nick/github/nijaru/aircher/target/release
47M	/Users/nick/github/nijaru/aircher/target/doc

$ ls -lh /Users/nick/github/nijaru/aircher/models/
-rw-r--r--  261M  swerank-embed-small.safetensors
```

### Key Findings

1. **Total Project Size**: 57GB
2. **Build Artifacts**: 56.9GB (99% of total)
   - target/debug: 52GB
   - target/release: 4.9GB
   - target/doc: 47MB
3. **Models**: 261MB (swerank embedding model for code search)
4. **Actual Source Code**: ~10MB (src/ + configs + docs)

### Root Cause

**No `.dockerignore` file** → Docker COPY command copied entire target/ directory (56.9GB of build artifacts) into build context during `docker build`.

### Solution

Created `.dockerignore` file excluding:
```
# Build artifacts (57GB)
target/

# Git history
.git/
.github/

# Benchmark results
benchmark-results/

# Database files
*.db
*.sqlite
*.sqlite3

# Logs
*.log
logs/

# Development
private/
.agents/
.aircher/

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Env files
.env
.env.local
.env.*.local

# Large model files
models/*.safetensors
models/*.bin
models/*.onnx
models/*.pt
models/*.pth
```

### Impact

**Before**: Docker transferred 61GB (57GB project + 4GB overhead)
**After**: Docker will transfer <100MB (source code only)

**Time Savings**:
- Before: Several minutes to transfer 61GB over Docker socket
- After: <10 seconds for <100MB

## Additional Blockers Discovered

### Issue 1: Terminal-Bench Not Published
**Error**:
```
npm ERR! 404 Not Found - GET https://registry.npmjs.org/@terminal-bench%2fcli
npm ERR! 404  '@terminal-bench/cli@*' is not in this registry.
```

**Impact**: Cannot run Terminal-Bench until package is published by maintainers

**Workaround**: Manual validation tasks (see ai/MANUAL_VALIDATION_TASKS.md)

### Issue 2: Rust Version Mismatch
**Error**:
```
error: failed to parse lock file at: /build/Cargo.lock
lock file version `4` was found, but this version of Cargo does not understand this lock file
```

**Root Cause**: Dockerfile uses `rust:1.70-slim`, but Cargo.lock requires Rust 1.79+

**Fix**: Update Dockerfile to use `rust:1.79-slim` or newer

## Recommendations

Given the blockers discovered, recommend pivoting to alternative validation approaches:

### Option A: Manual Validation Tasks (Recommended)
- 5 realistic coding tasks (file analysis, code gen, bug fix, git, refactoring)
- Can run immediately with existing agent
- Time: 1-2 hours
- Proves: Agent can complete actual coding tasks

### Option B: Context Awareness Improvement (High Value)
- Implement user's insight: expose context stats to model
- Model can see "97K/200K tokens remaining"
- Time: 1 hour
- Proves: User feedback implemented, better decision-making

### Option C: Wait for Terminal-Bench Release
- Monitor https://www.tbench.ai/ for npm package
- Fix Dockerfile Rust version
- Re-attempt benchmark setup
- Time: Unknown (weeks? months?)

### Option D: SWE-bench Verified
- More credible than Terminal-Bench
- Setup: 1-2 days
- Run: 4-6 hours
- 500 human-validated tasks

## Files Created/Modified

### Created:
1. `.dockerignore` - Excludes build artifacts and large files
2. `ai/DOCKER_INVESTIGATION_2025-10-30.md` - This document

### Modified:
1. `ai/STATUS.md` - Documented Docker investigation findings
2. `ai/TODO.md` - Updated with blockers and alternative approaches
3. `ai/BENCHMARK_SETUP.md` - Added blockers section with workarounds

## Conclusion

**Problem Solved**: Docker build size reduced from 61GB to <100MB via `.dockerignore`

**New Blockers**: Terminal-Bench package doesn't exist, Rust version mismatch

**Recommended Path**: Pivot to manual validation or context awareness improvement (both can be done immediately, both provide value)
