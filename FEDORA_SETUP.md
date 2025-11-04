# Fedora Setup for Aircher + vLLM

**Hardware**: i9-13900KF, 32GB DDR5, RTX 4090 (24GB VRAM)
**Tailscale**: `nick@fedora`
**Recommended Model**: gpt-oss:20b (13GB, fits comfortably on RTX 4090)

## Quick Start (When Fedora Available)

```bash
# SSH to Fedora
ssh nick@fedora

# Install DuckDB (needed for Aircher build)
cd ~/github/nijaru/aircher
./scripts/install-duckdb-fedora.sh

# Pull latest code
git pull origin main

# Build Aircher (first build takes ~5 minutes)
cargo build --release

# Start vLLM with gpt-oss:20b
source ~/vllm-env/bin/activate
bash ~/start_vllm.sh > ~/vllm.log 2>&1 &

# Monitor startup (takes 30-60 seconds)
tail -f ~/vllm.log

# Wait for: "Uvicorn running on http://0.0.0.0:11435"

# Run Aircher with vLLM
./target/release/aircher --provider openai --model "gpt-oss:20b"
```

## Test from Mac

```bash
# Test connectivity
curl http://nick@fedora:11435/v1/models

# Should return: {"object":"list","data":[{"id":"openai/gpt-oss-20b",...}]}

# Test Aircher with vLLM
./target/release/aircher --provider openai --model "openai/gpt-oss-20b" "Hello from vLLM!"
```

## vLLM Configuration ✅ WORKING

**File**: `~/start_vllm.sh` (UPDATED Nov 3, 2025)
```bash
#!/bin/bash
source ~/vllm-env/bin/activate

# Working configuration (Nov 3, 2025)
vllm serve openai/gpt-oss-20b \
  --gpu-memory-utilization 0.7 \
  --max-model-len 16384 \
  --host 0.0.0.0 \
  --port 11435 \
  --dtype auto \
  --api-key "test-key-for-aircher"  # NEW: Required for Aircher integration
```

**Key Changes from Previous**:
- ✅ Model path: `openai/gpt-oss-20b` (HuggingFace format, not Ollama's `gpt-oss:20b`)
- ✅ GPU utilization: 0.7 (not 0.8 - prevents CUDA OOM)
- ✅ No quantization flag (use `--dtype auto` instead)
- ✅ API key set (required for Aircher's OpenAI provider)

**Mac Config**: `~/.aircher/config.toml` (NOTE: NOT ~/.config/aircher!)
```toml
# vLLM Configuration
[global]
default_provider = "openai"
default_model = "openai/gpt-oss-20b"

[providers.openai]
name = "openai"
base_url = "http://100.93.39.25:11435/v1"
api_key_env = "OPENAI_API_KEY"
timeout_seconds = 300
max_retries = 3
```

**Usage**:
```bash
# Test Aircher with vLLM
OPENAI_API_KEY="test-key-for-aircher" ./target/release/aircher --provider openai --model "openai/gpt-oss-20b" "Hello from vLLM!"
```

## What Went Wrong Last Time

**Error**: `fatal error: Python.h: No such file or directory`

**Root Cause**: vLLM torch compilation needs Python development headers

**Fix**: Install `python3.13-devel` package

**Model Load**: Model loaded successfully (13.72 GB in VRAM) but failed during torch.compile step

## Performance Expectations

**vLLM + gpt-oss:20b on RTX 4090**:
- First load: ~30-60 seconds (model into VRAM)
- Per-response: ~2-3 seconds (fast GPU inference)
- VRAM usage: ~14-16 GB (plenty of headroom on 24GB)
- Tokens/second: ~50-100 (depends on context length)

**vs Ollama on Mac M3 Max**:
- Per-response: ~7-10 seconds (CPU inference)
- **Speedup**: vLLM is ~3-4x faster

**Why vLLM is faster**:
- GPU acceleration (CUDA on RTX 4090)
- Optimized attention kernels
- Better batching and caching
- PagedAttention for memory efficiency

## Alternative: Eager Mode (No Compilation)

If `python3.13-devel` installation fails:

```bash
# Add --enforce-eager flag to skip torch.compile
vllm serve openai/gpt-oss-20b \
  --gpu-memory-utilization 0.8 \
  --max-model-len 16384 \
  --host 0.0.0.0 \
  --port 11435 \
  --quantization mxfp4 \
  --enforce-eager
```

**Trade-off**: Slower inference, but doesn't need compilation

## Monitoring

```bash
# Check vLLM logs
tail -100 ~/vllm.log

# Check GPU usage
nvidia-smi

# Check process
ps aux | grep vllm

# Kill if needed
pkill -9 -f "vllm serve"
```

## Troubleshooting

**Issue**: Model loading takes forever
- **Check**: GPU memory with `nvidia-smi`
- **Expect**: ~14-16 GB VRAM usage
- **Fix**: Reduce `--max-model-len` if OOM

**Issue**: Connection refused from Mac
- **Check**: Firewall on Fedora
- **Check**: Tailscale connectivity: `ping nick@fedora`
- **Check**: Port 11435 is listening: `ss -tlnp | grep 11435`

**Issue**: vLLM crashes
- **Check**: `~/vllm.log` for errors
- **Common**: Python.h missing → install python3.13-devel
- **Common**: OOM → reduce gpu_memory_utilization or max_model_len

## Status

### ✅ SUCCESS (Nov 3, 2025) - vLLM Running!

**What Works**:
- ✅ vLLM server running successfully on Fedora
- ✅ Model: `openai/gpt-oss-20b` loaded (13GB in VRAM)
- ✅ KV cache: 18,736 tokens (0.86 GiB)
- ✅ CUDA graphs captured successfully (4 seconds, 0.84 GiB)
- ✅ Endpoints validated via curl:
  - `/v1/models` → Returns model info
  - `/v1/chat/completions` → Generates responses
- ✅ Aircher code fix: Added OpenAI provider to CLI (commit f217e77)
- ✅ Aircher binary rebuilt with OpenAI support (67M, Nov 3 16:54)

**Performance**:
- Initialization: ~15 seconds (model load + CUDA graph capture)
- Expected latency: 2-3 seconds per response (vs 7-10 seconds with Ollama)
- GPU memory: 19.94 GB / 23.51 GB (84% utilization)
- Expected throughput: ~50-100 tokens/second

**What's Left to Test** (when Fedora back online):
- ⏳ End-to-end Aircher + vLLM integration test
- ⏳ Actual response generation from Aircher CLI
- ⏳ Performance comparison vs Ollama on Mac

**Key Learnings**:
1. **Model path**: Must use HuggingFace format `openai/gpt-oss-20b`, not Ollama format `gpt-oss:20b`
2. **GPU utilization**: 0.7 works reliably; 0.8 caused CUDA OOM during graph capture
3. **Quantization**: Remove `--quantization mxfp4`, use `--dtype auto` instead
4. **API key**: vLLM validates keys! Use `--api-key` flag to set expected key
5. **Config location**: Aircher looks for `~/.aircher/config.toml`, NOT `~/.config/aircher/config.toml`
6. **User help**: User stopped GDM to free GPU resources (good troubleshooting!)

### Previous Attempt (Oct 30, 2025):
- ✅ vLLM installed successfully
- ✅ Model downloaded (GPT-OSS-20B)
- ✅ Model loaded into GPU (13.72 GB)
- ❌ Failed at torch.compile (missing Python.h)
- ✅ **Fixed**: Successfully worked around by using correct configuration
