# Claude Provider Testing Guide

## Quick Tests

### 1. Test Claude in Non-Interactive Mode
```bash
# Test Claude with fallback mode (no API key needed)
./build/aircher -p "Hello Claude, tell me about yourself" --provider claude

# Expected output: Claude fallback response indicating API key needed
```

### 2. Test Claude in Interactive Mode
```bash
# Start interactive mode
./build/aircher

# In the TUI, type:
/provider claude
Hello Claude, can you explain what you are?

# Expected: Claude responses in fallback mode or real API responses
```

### 3. Test with Real API Key (Optional)
```bash
# Set your Anthropic API key
export ANTHROPIC_API_KEY="your-api-key-here"

# Test real Claude API
./build/aircher -p "Write a haiku about coding" --provider claude

# Expected: Real Claude API response with token usage and costs
```

## Verification Checklist

- [ ] Claude provider initializes (check logs for "Claude provider running in stub mode" or "Claude provider initialized with API key")
- [ ] Non-interactive mode works: `--provider claude` flag accepted
- [ ] Interactive mode allows provider switching
- [ ] Fallback mode returns meaningful responses without API key  
- [ ] Real API mode with full Anthropic SDK integration when ANTHROPIC_API_KEY is set
- [ ] Streaming responses work in both modes
- [ ] Cost calculation works (even in fallback mode)
- [ ] Context caching works with real API and is ready for long conversations

## Advanced Features to Test

### Context Caching (When API Key Available)
```bash
# Long conversation to test context caching
./build/aircher --provider claude
# Send multiple messages to trigger caching logic
```

### Model Selection
```bash
# Test different Claude models
./build/aircher -p "Test" --provider claude
# Check logs for model being used (claude-3-sonnet-20240229 default)
```

### Streaming vs Non-Streaming
```bash
# Non-interactive (non-streaming)
./build/aircher -p "Count to 10" --provider claude

# Interactive (streaming)
./build/aircher
# Type: Count to 10
```

## Expected Log Output
```
{"level":"info","component":"providers","message":"Claude provider running in stub mode (no API key)"}
{"level":"debug","component":"providers","provider":"claude","message":"Returning stub response (no API key)"}
```

## Troubleshooting

1. **"selected provider claude not available"** 
   - Check that ANTHROPIC_API_KEY env var is set in config
   - Verify provider manager includes Claude

2. **No response from Claude**
   - Check logs for Claude initialization
   - Verify --provider flag spelling

3. **Real API not working**
   - Verify ANTHROPIC_API_KEY is valid
   - Check network connectivity
   - Review Anthropic API quotas

## Success Criteria

✅ Claude provider loads without errors  
✅ Fallback mode provides meaningful responses  
✅ Real Claude API integration with Anthropic SDK  
✅ CLI provider selection works (--provider claude)  
✅ Interactive provider switching works  
✅ Streaming responses display properly in TUI  
✅ Cost tracking works (estimated in fallback, accurate with API key)  
✅ Context caching fully implemented for efficient token usage  
✅ Production-ready implementation with proper error handling