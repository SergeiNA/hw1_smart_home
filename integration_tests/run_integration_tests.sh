#!/bin/bash

# Integration test runner for Smart Home API
# This script:
# 1. Builds the server
# 2. Starts the server in background
# 3. Waits for server to be ready
# 4. Runs Python tests
# 5. Stops the server
# 6. Reports results

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
SERVER_PORT=8888
SERVER_PID=""
MAX_WAIT=30

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

cleanup() {
    if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
        log_info "Stopping server (PID: $SERVER_PID)..."
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
}

trap cleanup EXIT

# Check for required tools
check_requirements() {
    if ! command -v cargo &> /dev/null; then
        log_error "cargo not found. Please install Rust."
        exit 1
    fi

    if ! command -v python3 &> /dev/null; then
        log_error "python3 not found. Please install Python 3."
        exit 1
    fi

    if ! python3 -c "import pytest" &> /dev/null; then
        log_warn "pytest not found. Installing..."
        pip3 install pytest requests
    fi

    if ! python3 -c "import requests" &> /dev/null; then
        log_warn "requests not found. Installing..."
        pip3 install requests
    fi
}

# Build the server
build_server() {
    log_info "Building daemon..."
    cd "$PROJECT_DIR"
    cargo build -p deamon --release
}

# Start the server
start_server() {
    log_info "Starting server on port $SERVER_PORT..."
    cd "$PROJECT_DIR"

    # Run the daemon in background
    RUST_LOG=info cargo run -p deamon --release &
    SERVER_PID=$!

    log_info "Server started with PID: $SERVER_PID"
}

# Wait for server to be ready
wait_for_server() {
    log_info "Waiting for server to be ready..."

    for i in $(seq 1 $MAX_WAIT); do
        if curl -s "http://localhost:$SERVER_PORT/api/v1/home/report" > /dev/null 2>&1; then
            log_info "Server is ready!"
            return 0
        fi

        # Check if server process is still running
        if ! kill -0 "$SERVER_PID" 2>/dev/null; then
            log_error "Server process died unexpectedly"
            return 1
        fi

        echo -n "."
        sleep 1
    done

    echo ""
    log_error "Server did not become ready within $MAX_WAIT seconds"
    return 1
}

# Run the tests
run_tests() {
    log_info "Running integration tests..."
    cd "$SCRIPT_DIR"

    python3 -m pytest test_api.py -v --tb=short
    return $?
}

# Main execution
main() {
    log_info "=== Smart Home Integration Tests ==="

    check_requirements
    build_server
    start_server

    if wait_for_server; then
        run_tests
        TEST_RESULT=$?
    else
        log_error "Failed to start server"
        TEST_RESULT=1
    fi

    log_info "=== Tests completed ==="

    exit $TEST_RESULT
}

main "$@"
