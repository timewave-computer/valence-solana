#!/bin/bash
# Test runner for Valence Protocol Microkernel

set -e

echo "=== Running Valence Protocol Microkernel Tests ==="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test categories
declare -A TEST_SUITES=(
    ["Unit Tests"]="cargo test --workspace --lib"
    ["Integration Tests"]="cargo test --workspace --test '*' -- --test-threads=1"
    ["E2E Microkernel Tests"]="cargo test --test e2e_microkernel_tests -- --nocapture"
    ["Performance Tests"]="cargo test --test performance_tests -- --nocapture"
    ["Verification Function Tests"]="cargo test -p basic_permission_verifier -p parameter_constraint_verifier -p zk_proof_verifier"
)

# Track test results
FAILED_TESTS=()
PASSED_TESTS=()

# Function to run a test suite
run_test_suite() {
    local suite_name=$1
    local test_command=$2
    
    echo -e "${BLUE}Running ${suite_name}...${NC}"
    echo "Command: $test_command"
    echo ""
    
    if eval "$test_command"; then
        echo -e "${GREEN}✓ ${suite_name} passed${NC}"
        PASSED_TESTS+=("$suite_name")
        return 0
    else
        echo -e "${RED}✗ ${suite_name} failed${NC}"
        FAILED_TESTS+=("$suite_name")
        return 1
    fi
}

# Check if solana-test-validator is running
check_validator() {
    if ! pgrep -f "solana-test-validator" > /dev/null; then
        echo -e "${YELLOW}Starting local validator...${NC}"
        solana-test-validator --reset > /dev/null 2>&1 &
        VALIDATOR_PID=$!
        sleep 5
        echo -e "${GREEN}✓ Local validator started (PID: $VALIDATOR_PID)${NC}"
    else
        echo -e "${GREEN}✓ Local validator already running${NC}"
    fi
}

# Main test execution
echo "Checking test environment..."
check_validator
echo ""

# Run each test suite
for suite_name in "${!TEST_SUITES[@]}"; do
    echo "=============================="
    run_test_suite "$suite_name" "${TEST_SUITES[$suite_name]}" || true
    echo ""
done

# Run specific program tests
echo "=============================="
echo -e "${BLUE}Running Program-Specific Tests...${NC}"
echo ""

# Session Factory tests
echo "Testing Session Factory..."
if cargo test -p session_factory -- --nocapture; then
    PASSED_TESTS+=("Session Factory")
else
    FAILED_TESTS+=("Session Factory")
fi

# Function Verification Table tests
echo "Testing Function Verification Table..."
if cargo test -p function_verification_table -- --nocapture; then
    PASSED_TESTS+=("Function Verification Table")
else
    FAILED_TESTS+=("Function Verification Table")
fi

# Summary
echo ""
echo "=============================="
echo "Test Summary"
echo "=============================="

echo -e "${GREEN}Passed (${#PASSED_TESTS[@]}):${NC}"
for test in "${PASSED_TESTS[@]}"; do
    echo "  ✓ $test"
done

if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
    echo ""
    echo -e "${RED}Failed (${#FAILED_TESTS[@]}):${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo "  ✗ $test"
    done
fi

# Calculate pass rate
TOTAL_TESTS=$((${#PASSED_TESTS[@]} + ${#FAILED_TESTS[@]}))
if [ $TOTAL_TESTS -gt 0 ]; then
    PASS_RATE=$(( (${#PASSED_TESTS[@]} * 100) / TOTAL_TESTS ))
    echo ""
    echo "Pass rate: ${PASS_RATE}%"
fi

# Clean up validator if we started it
if [ ! -z "$VALIDATOR_PID" ]; then
    echo ""
    echo -e "${YELLOW}Stopping local validator...${NC}"
    kill $VALIDATOR_PID 2>/dev/null || true
fi

# Exit with appropriate code
if [ ${#FAILED_TESTS[@]} -eq 0 ]; then
    echo ""
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}Some tests failed. Please check the output above.${NC}"
    exit 1
fi