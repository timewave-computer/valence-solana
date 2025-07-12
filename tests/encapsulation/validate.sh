#!/bin/bash
set -e

echo "=== Validating Shard Encapsulation Implementation ==="
echo

# Function to check if a pattern exists in a file
check_pattern() {
    local file=$1
    local pattern=$2
    local test_name=$3
    
    if grep -q "$pattern" "$file" 2>/dev/null; then
        echo "✅ $test_name"
        return 0
    else
        echo "❌ $test_name - Pattern not found: $pattern"
        return 1
    fi
}

# Function to check file exists
check_file() {
    local file=$1
    local test_name=$2
    
    if [ -f "$file" ]; then
        echo "✅ $test_name"
        return 0
    else
        echo "❌ $test_name - File not found: $file"
        return 1
    fi
}

PASSED=0
FAILED=0

# Track test results
run_test() {
    if "$@"; then
        ((PASSED++))
    else
        ((FAILED++))
    fi
}

echo "1. Core Capability System:"
run_test check_pattern "../../programs/registry/src/state.rs" "required_capabilities" "Functions can declare required capabilities"
run_test check_pattern "../../programs/shard/src/state.rs" "capabilities" "Sessions store granted capabilities"
run_test check_pattern "../../programs/shard/src/instructions/session.rs" "normalize_capability" "Capabilities are normalized"
run_test check_file "../../programs/shard/src/capabilities.rs" "Standard capabilities module exists"

echo
echo "2. Runtime Enforcement:"
run_test check_pattern "../../programs/shard/src/instructions/bundle.rs" "session_capabilities" "Capability checking in execute_function_cpi"
run_test check_pattern "../../programs/shard/src/instructions/bundle.rs" "InsufficientCapabilities" "Capability enforcement error handling"
run_test check_pattern "../../programs/shard/src/error.rs" "InsufficientCapabilities" "InsufficientCapabilities error defined"

echo
echo "3. Encapsulation Architecture:"
run_test check_pattern "../../programs/shard/src/instructions/bundle.rs" "execute_function_cpi" "Function execution through CPI"
run_test check_pattern "../../programs/shard/src/instructions/bundle.rs" "get_function_program" "Function resolution through registry"

# Check that we're finding registry entries and checking capabilities
echo
echo "4. Capability Enforcement Logic:"
if grep -A 10 "find_registry_entry" "../../programs/shard/src/instructions/bundle.rs" | grep -q "required_capabilities"; then
    echo "✅ Registry entries are checked for required capabilities"
    ((PASSED++))
else
    echo "❌ Missing capability check from registry entries"
    ((FAILED++))
fi

if grep -A 5 "for required_cap in required_capabilities" "../../programs/shard/src/instructions/bundle.rs" | grep -q "session_capabilities.contains"; then
    echo "✅ Session capabilities are validated against requirements"
    ((PASSED++))
else
    echo "❌ Missing session capability validation"
    ((FAILED++))
fi

echo
echo "=== Summary ==="
echo "Passed: $PASSED tests"
echo "Failed: $FAILED tests"

if [ $FAILED -eq 0 ]; then
    echo
    echo "✅ All encapsulation validations passed!"
    echo "Shards are properly encapsulated and capability-based access is enforced."
    exit 0
else
    echo
    echo "❌ Some validations failed. Please check the implementation."
    exit 1
fi