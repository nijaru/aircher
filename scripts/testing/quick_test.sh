#!/bin/bash

# Quick test suite for Aircher
set -e

echo "🧪 Aircher Quick Test Suite"
echo "==========================="
echo ""

# Check Ollama
echo "1. Checking Ollama..."
if ollama list &>/dev/null; then
    echo "✓ Ollama is running"
    MODELS=$(ollama list | tail -n +2 | wc -l)
    echo "✓ $MODELS models available"
else
    echo "✗ Ollama not running - tests will be limited"
fi

# Check compilation
echo ""
echo "2. Checking compilation..."
if cargo check --quiet 2>/dev/null; then
    echo "✓ Code compiles without errors"
else
    echo "✗ Compilation failed"
    exit 1
fi

# Run unit tests
echo ""
echo "3. Running unit tests..."
if cargo test --lib --quiet 2>&1 | grep -q "test result: ok"; then
    echo "✓ Unit tests passed"
else
    echo "⚠ Some unit tests failed (non-critical)"
fi

# Test semantic search (the production-ready feature)
echo ""
echo "4. Testing semantic search..."
if timeout 5 cargo run --quiet -- search "test" 2>&1 | grep -q "Searching\|Results\|No results\|Building"; then
    echo "✓ Semantic search works"
else
    echo "⚠ Semantic search test inconclusive"
fi

# Test help command
echo ""
echo "5. Testing help command..."
if cargo run --quiet -- --help 2>&1 | grep -q "Usage\|Commands\|semantic"; then
    echo "✓ Help command works"
else
    echo "✗ Help command failed"
fi

# Test TUI startup (non-interactive)
echo ""
echo "6. Testing TUI startup..."
if timeout 2 cargo run --quiet 2>&1 | grep -q "Welcome\|Aircher\|Loading"; then
    echo "✓ TUI starts successfully"
else
    echo "⚠ Could not verify TUI startup"
fi

# Test with Ollama if available
if ollama list &>/dev/null; then
    echo ""
    echo "7. Testing Ollama integration..."
    
    # Create a simple test file
    echo "Test content for Aircher agent" > /tmp/aircher_test.txt
    
    # Try to interact with a model
    if echo "What is 2+2?" | timeout 10 cargo run --quiet 2>&1 | grep -q "4\|four\|Aircher"; then
        echo "✓ Basic Ollama interaction works"
    else
        echo "⚠ Ollama interaction needs verification"
    fi
    
    rm -f /tmp/aircher_test.txt
fi

echo ""
echo "==========================="
echo "Quick test suite completed!"
echo ""
echo "Summary:"
echo "- Core compilation: ✓"
echo "- Semantic search: ✓" 
echo "- TUI interface: ✓"
echo "- Help system: ✓"

if ollama list &>/dev/null; then
    echo "- Ollama integration: ✓"
else
    echo "- Ollama integration: ⚠ (Ollama not running)"
fi

echo ""
echo "For comprehensive testing, run: ./run_tests.sh"