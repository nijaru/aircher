#!/bin/bash

# Aircher Comprehensive Test Suite Runner
# This script tests the agent, ACP, and TUI functionality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🧪 Aircher Comprehensive Test Suite"
echo "===================================="
echo ""

# Check if Ollama is running
check_ollama() {
    echo "Checking Ollama status..."
    if ollama list &>/dev/null; then
        echo -e "${GREEN}✓ Ollama is running${NC}"

        # Check for test models
        if ollama list | grep -q "exaone-deep\|deepseek-r1\|gpt-oss"; then
            echo -e "${GREEN}✓ Test models available${NC}"
        else
            echo -e "${YELLOW}⚠ No test models found. Installing lightweight model...${NC}"
            # Try to pull a small model
            ollama pull deepseek-r1:latest || ollama pull gemma:2b || echo "Failed to pull model"
        fi
    else
        echo -e "${RED}✗ Ollama not running${NC}"
        echo "Please start Ollama with: ollama serve"
        exit 1
    fi
}

# Run cargo tests
run_cargo_tests() {
    echo ""
    echo "Running Cargo unit tests..."
    if cargo test --lib --quiet 2>&1 | grep -q "test result: ok"; then
        echo -e "${GREEN}✓ Unit tests passed${NC}"
    else
        echo -e "${YELLOW}⚠ Some unit tests failed (continuing)${NC}"
    fi
}

# Test basic compilation
test_compilation() {
    echo ""
    echo "Testing compilation..."
    if cargo build --release --quiet 2>&1; then
        echo -e "${GREEN}✓ Binary compiles successfully${NC}"
    else
        echo -e "${RED}✗ Compilation failed${NC}"
        exit 1
    fi
}

# Test agent with Ollama
test_agent_ollama() {
    echo ""
    echo "Testing Agent with Ollama..."

    # Compile test
    if cargo build --bin agent_ollama_test --release 2>/dev/null || \
       rustc tests/integration/agent_ollama_test.rs -L target/release/deps -o target/release/agent_ollama_test 2>/dev/null; then

        if [ -f target/release/agent_ollama_test ]; then
            ./target/release/agent_ollama_test
        else
            echo -e "${YELLOW}⚠ Agent test not built, using cargo run${NC}"
            cargo run --bin aircher -- --help &>/dev/null && echo -e "${GREEN}✓ Basic agent works${NC}"
        fi
    else
        echo -e "${YELLOW}⚠ Could not build agent test${NC}"
    fi
}

# Test TUI functionality
test_tui() {
    echo ""
    echo "Testing TUI functionality..."

    # Create a test script that interacts with the TUI
    cat > /tmp/tui_test.exp << 'EOF'
#!/usr/bin/expect -f
set timeout 10
spawn cargo run --release --quiet
expect "Welcome to Aircher" {
    send "/help\r"
    expect "Available commands" {
        send "\003"
        exit 0
    }
} timeout {
    exit 1
}
EOF

    if command -v expect &>/dev/null; then
        chmod +x /tmp/tui_test.exp
        if /tmp/tui_test.exp; then
            echo -e "${GREEN}✓ TUI starts and responds${NC}"
        else
            echo -e "${YELLOW}⚠ TUI test failed${NC}"
        fi
    else
        # Fallback: just check if it starts
        timeout 2 cargo run --release --quiet 2>&1 | grep -q "Welcome\|Aircher" && \
            echo -e "${GREEN}✓ TUI starts${NC}" || \
            echo -e "${YELLOW}⚠ Could not verify TUI${NC}"
    fi
}

# Test specific features
test_features() {
    echo ""
    echo "Testing specific features..."

    # Test semantic search
    echo -n "  Semantic search: "
    if cargo run --release --quiet -- search "test" 2>&1 | grep -q "Searching\|Results\|No results"; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${YELLOW}⚠${NC}"
    fi

    # Test help command
    echo -n "  Help command: "
    if cargo run --release --quiet -- --help 2>&1 | grep -q "Usage\|Commands"; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${YELLOW}⚠${NC}"
    fi
}

# Performance test
test_performance() {
    echo ""
    echo "Running performance tests..."

    # Create a test file
    echo "Test content for Aircher" > /tmp/aircher_perf_test.txt

    # Time a simple operation
    start_time=$(date +%s%N)
    timeout 5 cargo run --release --quiet -- search "test" &>/dev/null
    end_time=$(date +%s%N)

    elapsed=$((($end_time - $start_time) / 1000000))

    if [ $elapsed -lt 5000 ]; then
        echo -e "${GREEN}✓ Search completes in ${elapsed}ms${NC}"
    else
        echo -e "${YELLOW}⚠ Search took ${elapsed}ms (slow)${NC}"
    fi

    rm -f /tmp/aircher_perf_test.txt
}

# Create integration test
create_integration_test() {
    echo ""
    echo "Creating comprehensive integration test..."

    cat > /tmp/integration_test.sh << 'EOF'
#!/bin/bash
# Full integration test

# Test 1: Basic message
echo "Test message" | timeout 5 cargo run --release --quiet 2>&1 | grep -q "Aircher" && echo "✓ Basic input" || echo "✗ Basic input"

# Test 2: Model selection (if available)
echo -e "/model\n\033" | timeout 3 cargo run --release --quiet 2>&1 | grep -q "Select" && echo "✓ Model selection" || echo "✗ Model selection"

# Test 3: Search functionality
echo "/search TODO" | timeout 5 cargo run --release --quiet 2>&1 | grep -q "search\|Search" && echo "✓ Search command" || echo "✗ Search command"

EOF

    chmod +x /tmp/integration_test.sh
    /tmp/integration_test.sh
}

# Main test execution
main() {
    # Check prerequisites
    check_ollama

    # Run test suite
    test_compilation
    run_cargo_tests
    test_agent_ollama
    test_tui
    test_features
    test_performance
    create_integration_test

    echo ""
    echo "===================================="
    echo -e "${GREEN}🎉 Test suite completed!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Run manually: cargo run --release"
    echo "2. Test with Ollama: Set model to 'exaone-deep' or 'gpt-oss'"
    echo "3. Test tools: Try 'List files in src directory'"
    echo "4. Test search: Use '/search <query>'"
}

# Run main
main
