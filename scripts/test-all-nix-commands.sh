#!/bin/bash
# Test script for all Valence Solana nix commands

set -e

echo "=========================================="
echo "🧪 Testing All Valence Solana Nix Commands"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print test status
print_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to run a command and capture its status
run_test() {
    local test_name="$1"
    local command="$2"
    local allow_failure="$3"
    
    print_test "Running: $test_name"
    echo "Command: $command"
    echo ""
    
    if eval "$command"; then
        print_success "$test_name completed successfully"
        echo ""
        return 0
    else
        if [ "$allow_failure" = "true" ]; then
            print_warning "$test_name failed (allowed)"
        else
            print_error "$test_name failed"
        fi
        echo ""
        return 1
    fi
}

# Track test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

test_command() {
    local name="$1"
    local cmd="$2"
    local allow_failure="$3"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if run_test "$name" "$cmd" "$allow_failure"; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
}

echo "Starting comprehensive test suite..."
echo ""

# Test 1: Environment info
test_command "Environment Info" "nix run .#env-info"

# Test 2: Setup Solana
test_command "Solana Setup" "nix run .#setup-solana"

# Test 3: Fast build (crate2nix only)
test_command "Fast Build (crate2nix)" "nix run .#build-fast"

# Test 4: Individual crate build
test_command "Individual Crate Build" "nix run .#build-crate authorization"

# Test 5: Build all workspace members
test_command "All Workspace Members" "nix build .#all-workspace-members"

# Test 6: Build without IDL
test_command "Build without IDL" "nix run .#build-no-idl"

# Test 7: IDL generation (separate derivation)
test_command "IDL Generation" "nix run .#generate-idls" "true"

# Test 8: Full build (with IDL)
test_command "Full Build with IDL" "nix run .#build" "true"

# Test 9: Individual package builds
echo "Testing individual package builds..."
packages=("account_factory" "authorization" "registry" "storage_account" "valence-utils")
for package in "${packages[@]}"; do
    test_command "Package: $package" "nix build .#$package"
done

# Test 10: Test runner
test_command "Test Runner" "nix run .#test -- authorization --lib" "true"

# Test 11: Clear cache
test_command "Clear Cache" "nix run .#clear-cache"

# Test 12: Check if anchor wrapper works
test_command "Anchor Wrapper" "nix run .#anchor-wrapper -- --version"

# Test 13: Check development shell
test_command "Development Shell Check" "nix develop --command bash -c 'echo \"Dev shell works!\" && which cargo && which anchor'"

# Test 14: Verify derivation outputs
echo "Checking derivation outputs..."
test_command "IDL Derivation Output" "nix path-info .#idl-derivation && ls -la \$(nix path-info .#idl-derivation)/"

# Test 15: Check if deployment artifacts exist
if [ -d "target/deploy" ]; then
    print_success "Deployment artifacts found:"
    ls -la target/deploy/
else
    print_warning "No deployment artifacts found in target/deploy/"
fi

# Test 16: Check if IDL files exist
if [ -d "target/idl" ]; then
    print_success "IDL files found:"
    ls -la target/idl/
else
    print_warning "No IDL files found in target/idl/"
fi

echo ""
echo "=========================================="
echo "🏁 Test Suite Summary"
echo "=========================================="
echo "Total tests: $TOTAL_TESTS"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed! The nix environment is working perfectly.${NC}"
    exit 0
elif [ $PASSED_TESTS -gt $((FAILED_TESTS * 2)) ]; then
    echo -e "${YELLOW}⚠️  Most tests passed, but there are some issues to address.${NC}"
    exit 1
else
    echo -e "${RED}❌ Many tests failed. The environment needs attention.${NC}"
    exit 1
fi 