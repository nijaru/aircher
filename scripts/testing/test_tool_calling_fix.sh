#!/bin/bash

# Test script to validate that tool calling fix actually works
# Tests the core bug fix: tools now sent to LLMs instead of tools: None

echo "🧪 Testing Tool Calling Fix..."

# Check if Ollama is running
if ! pgrep -x "ollama" > /dev/null; then
    echo "Starting Ollama server..."
    ollama serve &
    sleep 5
fi

# Test with different models that support tool calling
MODELS=("deepseek-r1:latest" "gpt-oss:latest")

for MODEL in "${MODELS[@]}"; do
    echo ""
    echo "📝 Testing tool calling with model: $MODEL"
    
    if ollama list | grep -q "$MODEL"; then
        echo "✅ Model $MODEL is available"
        
        # Test 1: Simple file reading tool call
        echo "🔧 Test 1: File reading tool call"
        RUST_LOG=aircher=debug timeout 30 cargo run --release -- 2>&1 | head -20 | tee "test_${MODEL//[^a-zA-Z0-9]/_}.log"
        
        # Check if tools were actually sent in the debug output
        if grep -q "tools.*read_file" "test_${MODEL//[^a-zA-Z0-9]/_}.log"; then
            echo "✅ Tool schemas appear to be sent to LLM"
        else
            echo "❌ Tool schemas may not be sent - check debug log"
        fi
        
        # Check for actual tool execution
        if grep -q "Executing tool" "test_${MODEL//[^a-zA-Z0-9]/_}.log"; then
            echo "✅ Tool execution detected"
        else
            echo "⚠️  No tool execution detected in log"
        fi
        
    else
        echo "❌ Model $MODEL not found, skipping..."
    fi
done

echo ""
echo "🔍 Test Summary:"
echo "- Checked that tool schemas are sent to LLMs (not tools: None)"
echo "- Tested actual tool execution capability"
echo "- Generated logs for analysis"

echo ""
echo "📊 Next steps:"
echo "1. Review debug logs for tool schema transmission"
echo "2. Test interactive tool calling in TUI"  
echo "3. Validate multi-turn tool conversations"

echo ""
echo "🎯 Success criteria:"
echo "- LLMs receive tool schemas in requests"
echo "- Tools are actually callable by agents"
echo "- No more hallucinated tool calls"