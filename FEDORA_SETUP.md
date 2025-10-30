# Fedora Setup for Aircher + vLLM

**Hardware**: i9-13900KF, 32GB DDR5, RTX 4090 (24GB VRAM)
**Tailscale**: `nick@fedora`

## Quick Start (When Fedora Available)

```bash
# SSH to Fedora
ssh nick@fedora

# Install Python development headers (CRITICAL - missing this caused vLLM failure)
sudo dnf install -y python3.13-devel

# Kill any existing vLLM processes
pkill -9 -f "vllm serve"

# Start vLLM with GPT-OSS-20B
source ~/vllm-env/bin/activate
bash ~/start_vllm.sh > ~/vllm.log 2>&1 &

# Monitor startup (takes 30-60 seconds)
tail -f ~/vllm.log

# Wait for: "Uvicorn running on http://0.0.0.0:11435"
```

## Test from Mac

```bash
# Test connectivity
curl http://nick@fedora:11435/v1/models

# Should return: {"object":"list","data":[{"id":"openai/gpt-oss-20b",...}]}

# Test Aircher with vLLM
./target/release/aircher --provider openai --model "openai/gpt-oss-20b" "Hello from vLLM!"
```

## vLLM Configuration

**File**: `~/start_vllm.sh`
```bash
#!/bin/bash
source ~/vllm-env/bin/activate

vllm serve openai/gpt-oss-20b \
  --gpu-memory-utilization 0.8 \
  --max-model-len 16384 \
  --host 0.0.0.0 \
  --port 11435 \
  --quantization mxfp4
```

**Mac Config**: `~/.config/aircher/config.toml`
```toml
[model]
provider = "openai"
model = "openai/gpt-oss-20b"

[providers.openai]
base_url = "http://nick@fedora:11435/v1"
api_key_env = "VLLM_API_KEY"  # Dummy value
timeout_seconds = 300
max_retries = 3
```

## What Went Wrong Last Time

**Error**: `fatal error: Python.h: No such file or directory`

**Root Cause**: vLLM torch compilation needs Python development headers

**Fix**: Install `python3.13-devel` package

**Model Load**: Model loaded successfully (13.72 GB in VRAM) but failed during torch.compile step

## Performance Expectations

**vLLM (expected)**:
- First load: ~30-60 seconds (model into VRAM)
- Per-response: ~2-3 seconds (much faster than Ollama)
- VRAM usage: ~14-16 GB (MXFP4 quantization)

**Ollama (current)**:
- First load: ~4-5 seconds
- Per-response: ~7 seconds
- Local on Mac (13GB model)

**Speedup**: vLLM should be ~2-3x faster than Ollama

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

**Last Attempt** (Oct 30, 2025):
- ✅ vLLM installed successfully
- ✅ Model downloaded (GPT-OSS-20B)
- ✅ Model loaded into GPU (13.72 GB)
- ❌ Failed at torch.compile (missing Python.h)
- 🔧 **Fix**: `sudo dnf install python3.13-devel`

**Current Workaround**: Using Ollama on Mac (slower but works)
