#!/bin/bash

# Valence Protocol Test Runner
# Runs all integration tests for the singleton architecture

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Running Valence Protocol Integration Tests${NC}"
echo "=========================================="

# Check if in correct directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Must be run from project root${NC}"
    exit 1
fi

# Set test environment variables
export RUST_LOG=${RUST_LOG:-info}
export RUST_BACKTRACE=1

# Run different test suites
run_test_suite() {
    local test_name=$1
    echo -e "\n${BLUE}Running $test_name tests...${NC}"
    
    if cargo test --test integration $test_name -- --nocapture; then
        echo -e "${GREEN}✓ $test_name tests passed${NC}"
        return 0
    else
        echo -e "${RED}✗ $test_name tests failed${NC}"
        return 1
    fi
}

# Keep track of failures
failed_tests=()

# Run each test suite
test_suites=(
    "processor_singleton"
    "scheduler_singleton"
    "diff_singleton"
    "end_to_end"
)

for suite in "${test_suites[@]}"; do
    if ! run_test_suite "$suite"; then
        failed_tests+=("$suite")
    fi
done

# Summary
echo -e "\n${BLUE}Test Summary${NC}"
echo "============"

if [ ${#failed_tests[@]} -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Failed test suites:${NC}"
    for test in "${failed_tests[@]}"; do
        echo -e "  ${RED}✗ $test${NC}"
    done
    exit 1
fi