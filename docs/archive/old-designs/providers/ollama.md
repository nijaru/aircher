# Ollama Provider

The Ollama provider enables local model execution with zero API costs and complete privacy.

## Overview

- **Provider**: `ollama`
- **Cost**: Free (local execution)
- **Privacy**: Complete (no data leaves your machine)
- **Performance**: Depends on local hardware
- **API Key**: None required
- **Streaming**: ✅ Supported
- **Tools**: ❌ Not supported (yet)

## Configuration

### Auto-Discovery (Recommended)

Aircher automatically discovers Ollama instances! No configuration needed:

```bash
# Aircher will automatically find Ollama on:
# 1. localhost:11434 (default)
# 2. Your Tailscale network (detected automatically)
# 3. Common local network IPs
aircher --provider ollama "Hello, world!"
```

### Manual Configuration

Override auto-discovery with explicit configuration:

```toml
# ~/.config/aircher/config.toml
[providers.ollama]
base_url = "http://100.64.0.1:11434"  # Your specific Tailscale IP
model = "llama3.3"
timeout_seconds = 120
```

### Fallback URLs Configuration

Configure multiple fallback URLs for auto-discovery:

```toml
[providers.ollama]
base_url = ""  # Empty enables auto-discovery
fallback_urls = [
    "http://localhost:11434",
    "http://100.64.0.1:11434",    # Your Tailscale IP
    "http://192.168.1.100:11434", # Your local network IP
]
```

### Discovery Priority

Auto-discovery tries URLs in this order:
1. **Configured fallback_urls** (if any)
2. **localhost:11434** and **127.0.0.1:11434**
3. **Tailscale network** (detected via `tailscale ip` command)
4. **Docker**: `host.docker.internal:11434`
5. **Common local IPs**: 192.168.1.100, 192.168.0.100, 10.0.0.100

### Docker Configuration

```toml
[providers.ollama]
base_url = "http://localhost:11434"
model = "llama3.3"
```

## Supported Models

The Ollama provider automatically discovers available models. Common models include:

- **llama3.3** (70B) - Latest Llama model with excellent performance
- **llama3.1** (8B/70B) - Previous generation with good performance
- **mistral** (7B) - Fast and efficient
- **codellama** (7B/13B/34B) - Code-specialized model
- **phi3** (3.8B) - Microsoft's small but capable model
- **qwen2.5** (7B/32B) - Alibaba's multilingual model

## Usage Examples

### Basic Usage

```bash
# Simple chat
aircher --provider ollama "Explain Rust ownership"

# Interactive mode
aircher --provider ollama
> hello
🤖 Hello! How can I help you today?

# TUI mode
aircher --tui
# Then press Tab to select ollama provider
```

### Model Selection

```bash
# Specific model
aircher --provider ollama --model llama3.3 "Write a function"

# List available models
aircher --provider ollama --list-models
```

### Streaming Responses

```bash
# Streaming is enabled by default
aircher --provider ollama "Tell me a story"
# You'll see the response appear word by word
```

## Installation & Setup

### 1. Install Ollama

```bash
# macOS
brew install ollama

# Linux
curl -fsSL https://ollama.com/install.sh | sh

# Windows
# Download from https://ollama.com/download
```

### 2. Start Ollama Service

```bash
# Start the service
ollama serve

# Or run as daemon (macOS/Linux)
brew services start ollama  # macOS
sudo systemctl start ollama  # Linux
```

### 3. Pull Models

```bash
# Pull a model (this downloads the model files)
ollama pull llama3.3

# Pull multiple models
ollama pull llama3.1
ollama pull mistral
ollama pull codellama
```

### 4. Verify Installation

```bash
# Test with Aircher
aircher --provider ollama "Hello!"

# Or check Ollama directly
ollama list
```

## Tailscale Setup

Aircher automatically discovers Ollama on your Tailscale network! 

### 1. On the Ollama Host

```bash
# Allow external connections
export OLLAMA_HOST=0.0.0.0:11434
ollama serve
```

### 2. On the Client Machine

```bash
# No configuration needed! Auto-discovery will find it
aircher --provider ollama "Hello from Tailscale!"

# Optional: Verify Tailscale connectivity
tailscale ip
curl http://YOUR_TAILSCALE_IP:11434/api/version
```

### 3. Manual Override (if needed)

```toml
# ~/.config/aircher/config.toml - only if auto-discovery fails
[providers.ollama]
base_url = "http://YOUR_TAILSCALE_IP:11434"
```

### How Auto-Discovery Works

1. **Detects your Tailscale IP** using `tailscale ip -4`
2. **Scans the network range** for Ollama instances
3. **Tests connectivity** to each candidate
4. **Selects the first working instance**

This means it works seamlessly across your Tailscale network without manual configuration!

## Performance Optimization

### Model Selection

- **Code tasks**: Use `codellama` for best code understanding
- **General chat**: Use `llama3.3` for best quality
- **Fast responses**: Use `phi3` or `mistral` for speed
- **Multilingual**: Use `qwen2.5` for non-English tasks

### Hardware Considerations

- **RAM**: 8GB+ for 7B models, 16GB+ for 13B models, 32GB+ for 70B models
- **Storage**: 4-40GB per model depending on size
- **CPU**: More cores = faster generation
- **GPU**: CUDA/Metal acceleration supported

### Configuration Tuning

```toml
[providers.ollama]
base_url = "http://localhost:11434"
model = "llama3.3"
timeout_seconds = 300  # Increase for large models
max_tokens = 4096      # Adjust based on needs
temperature = 0.7      # Lower for more focused responses
```

## Troubleshooting

### Common Issues

1. **Connection refused**
   ```bash
   # Check if Ollama is running
   ps aux | grep ollama
   
   # Start if not running
   ollama serve
   ```

2. **Model not found**
   ```bash
   # List available models
   ollama list
   
   # Pull the model
   ollama pull llama3.3
   ```

3. **Slow responses**
   ```bash
   # Check system resources
   top
   
   # Try a smaller model
   aircher --provider ollama --model phi3 "test"
   ```

4. **Tailscale connection issues**
   ```bash
   # Test connectivity
   curl http://TAILSCALE_IP:11434/api/version
   
   # Check firewall
   sudo ufw allow 11434  # Linux
   ```

### Debug Mode

```bash
# Enable debug logging
export RUST_LOG=debug
aircher --provider ollama "test"
```

## Cost Comparison

| Provider | Cost per 1M tokens | Ollama |
|----------|-------------------|---------|
| OpenAI GPT-4 | $30-60 | $0 |
| Claude 3.5 Sonnet | $3-15 | $0 |
| Gemini Pro | $1.25-5 | $0 |
| **Ollama** | **$0** | **$0** |

## Privacy Benefits

- **No API calls**: Everything runs locally
- **No data transmission**: Your code never leaves your machine
- **No logging**: No conversation history stored externally
- **Offline capable**: Works without internet connection
- **GDPR compliant**: No data processing by third parties

## Integration with Aircher Intelligence

The Ollama provider integrates seamlessly with Aircher's Intelligence Engine:

- **Project context**: Local models understand your codebase
- **File analysis**: Privacy-preserving code analysis
- **Session persistence**: Conversations stored locally
- **Cost tracking**: Always shows $0.00 cost
- **Usage limits**: None (unlimited local usage)

## Best Practices

1. **Start with smaller models** for development
2. **Use specific models** for specific tasks (codellama for code)
3. **Configure timeouts** appropriately for your hardware
4. **Monitor system resources** when running large models
5. **Keep models updated** with `ollama pull`

This provider is perfect for privacy-conscious developers, offline work, and cost-sensitive projects.