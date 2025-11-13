#!/usr/bin/env bash

# Aircher Benchmark Runner
# Usage: ./scripts/run-benchmark.sh [phase1|phase2|custom]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/benchmark-results"
IMAGE_NAME="aircher-bench:latest"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create results directory
mkdir -p "$RESULTS_DIR"

# Function to build Docker image
build_image() {
    log_info "Building Docker image: $IMAGE_NAME"
    cd "$PROJECT_ROOT"
    docker build -t "$IMAGE_NAME" -f Dockerfile.bench .

    if [ $? -eq 0 ]; then
        log_info "✅ Docker image built successfully"
    else
        log_error "❌ Docker build failed"
        exit 1
    fi
}

# Function to check if image exists
check_image() {
    if ! docker image inspect "$IMAGE_NAME" >/dev/null 2>&1; then
        log_warn "Docker image not found. Building..."
        build_image
    else
        log_info "Docker image found: $IMAGE_NAME"
    fi
}

# Phase 1: Validate Integration (10 tasks)
run_phase1() {
    log_info "========================================"
    log_info "Phase 1: Validation Run (10 tasks)"
    log_info "Goal: Prove ACP integration works"
    log_info "Expected time: 10-30 minutes"
    log_info "========================================"

    check_image

    log_info "Running 10 tasks from Terminal-Bench..."
    docker run --rm \
        -v "$RESULTS_DIR:/results" \
        --name aircher-phase1 \
        "$IMAGE_NAME" \
        tbench run --agent aircher --tasks 10 --output /results/phase1.json \
        || log_warn "Phase 1 completed with errors (expected on first run)"

    log_info "Phase 1 complete. Check results: $RESULTS_DIR/phase1.json"

    if [ -f "$RESULTS_DIR/phase1.json" ]; then
        log_info "✅ Results file created"
        log_info "Next: Review results, then run 'phase2' for full benchmark"
    else
        log_error "❌ No results file. Check Docker logs for errors"
    fi
}

# Phase 2: Baseline Run (80 tasks)
run_phase2() {
    log_info "========================================"
    log_info "Phase 2: Full Baseline Run (80 tasks)"
    log_info "Goal: Establish performance baseline"
    log_info "Expected time: 2-6 hours"
    log_info "========================================"

    check_image

    log_warn "This will run for several hours. Continue? (y/n)"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        log_info "Cancelled."
        exit 0
    fi

    log_info "Running full Terminal-Bench (80 tasks)..."
    log_info "Monitor with: docker logs -f aircher-phase2"

    docker run --rm \
        -v "$RESULTS_DIR:/results" \
        --name aircher-phase2 \
        "$IMAGE_NAME" \
        tbench run --agent aircher --dataset core-v0 --output /results/baseline.json

    log_info "Phase 2 complete. Check results: $RESULTS_DIR/baseline.json"

    if [ -f "$RESULTS_DIR/baseline.json" ]; then
        log_info "✅ Baseline run complete"
        log_info "Generating report..."
        docker run --rm \
            -v "$RESULTS_DIR:/results" \
            "$IMAGE_NAME" \
            tbench report --input /results/baseline.json --format markdown --output /results/baseline-report.md

        log_info "Report: $RESULTS_DIR/baseline-report.md"
    else
        log_error "❌ No results file"
    fi
}

# Custom run (user-specified parameters)
run_custom() {
    log_info "========================================"
    log_info "Custom Benchmark Run"
    log_info "========================================"

    check_image

    log_info "Enter benchmark command (or 'bash' for interactive):"
    read -r cmd

    if [ "$cmd" = "bash" ]; then
        log_info "Starting interactive shell..."
        docker run --rm -it \
            -v "$RESULTS_DIR:/results" \
            "$IMAGE_NAME" \
            bash
    else
        log_info "Running custom command: $cmd"
        docker run --rm \
            -v "$RESULTS_DIR:/results" \
            --name aircher-custom \
            "$IMAGE_NAME" \
            $cmd
    fi
}

# Build only (no run)
build_only() {
    log_info "Building Docker image only..."
    build_image
    log_info "Build complete. Run with: ./scripts/run-benchmark.sh phase1"
}

# Show usage
show_usage() {
    cat <<EOF
Aircher Benchmark Runner

Usage: $0 [COMMAND]

Commands:
  phase1     Run Phase 1: Validation (10 tasks, ~30 min)
  phase2     Run Phase 2: Full baseline (80 tasks, ~4 hours)
  custom     Run custom benchmark command
  build      Build Docker image only
  help       Show this help message

Examples:
  $0 phase1                    # Validate integration
  $0 phase2                    # Full benchmark run
  $0 build                     # Just build the image

Results will be saved to: $RESULTS_DIR/

For more details, see: ai/BENCHMARK_SETUP.md
EOF
}

# Main
case "${1:-help}" in
    phase1)
        run_phase1
        ;;
    phase2)
        run_phase2
        ;;
    custom)
        run_custom
        ;;
    build)
        build_only
        ;;
    help|--help|-h)
        show_usage
        ;;
    *)
        log_error "Unknown command: $1"
        show_usage
        exit 1
        ;;
esac
