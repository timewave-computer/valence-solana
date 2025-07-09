#!/bin/bash
# Verification script for Valence Protocol implementation

set -e

echo "=== Verifying Valence Protocol Implementation ==="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Track results
CHECKS_PASSED=0
CHECKS_FAILED=0

# Function to check if a file exists
check_file() {
    local file=$1
    local description=$2
    
    if [ -f "$file" ]; then
        echo -e "${GREEN}✓${NC} $description"
        ((CHECKS_PASSED++))
    else
        echo -e "${RED}✗${NC} $description (missing: $file)"
        ((CHECKS_FAILED++))
    fi
}

# Function to check if a directory exists
check_dir() {
    local dir=$1
    local description=$2
    
    if [ -d "$dir" ]; then
        echo -e "${GREEN}✓${NC} $description"
        ((CHECKS_PASSED++))
    else
        echo -e "${RED}✗${NC} $description (missing: $dir)"
        ((CHECKS_FAILED++))
    fi
}

echo "Checking Core Programs..."
echo "========================"
check_dir "programs/session_factory" "Session Factory program"
check_dir "programs/function_verification_table" "Function Verification Table program"
check_dir "programs/entrypoint" "Entrypoint program"
check_dir "programs/shard_contract" "Shard Contract program"
check_dir "programs/eval" "Eval program"
check_dir "programs/valence_session" "Valence Session program"

echo ""
echo "Checking Verification Functions..."
echo "================================="
check_dir "programs/verification_functions/basic_permission" "Basic Permission verifier"
check_dir "programs/verification_functions/parameter_constraint" "Parameter Constraint verifier"
check_dir "programs/verification_functions/zk_proof" "ZK Proof verifier"

echo ""
echo "Checking Documentation..."
echo "======================="
check_file "docs/DEPLOYMENT_GUIDE.md" "Deployment Guide"
check_file "docs/CAPABILITY_EXAMPLES.md" "Capability Examples"
check_file "docs/TROUBLESHOOTING.md" "Troubleshooting Guide"
check_file "IMPLEMENTATION_SUMMARY.md" "Implementation Summary"
check_file "work/functional_arch.md" "Functional Architecture"

echo ""
echo "Checking Tests..."
echo "================"
check_file "tests/src/session_factory_tests.rs" "Session Factory tests"
check_file "tests/src/function_verification_table_tests.rs" "FVT tests"
check_file "tests/src/e2e_microkernel_tests.rs" "E2E tests"
check_file "tests/src/performance_tests.rs" "Performance tests"
check_file "tests/src/basic_functionality_test.rs" "Basic functionality tests"

echo ""
echo "Checking Examples..."
echo "==================="
check_file "examples/simple_capability_demo.rs" "Simple capability demo"
check_file "examples/defi_protocol/deployment_script.rs" "DeFi protocol example"
check_file "examples/README.md" "Examples documentation"

echo ""
echo "Checking Build System..."
echo "======================"
check_file "Cargo.toml" "Workspace Cargo.toml"
check_file "Anchor.toml" "Anchor configuration"
check_file "scripts/build_microkernel.sh" "Build script"
check_file "scripts/test_microkernel.sh" "Test runner script"

echo ""
echo "Checking Program Structure..."
echo "============================"
for program in session_factory function_verification_table entrypoint shard_contract eval valence_session; do
    if [ -d "programs/$program" ]; then
        echo -e "${YELLOW}Checking $program...${NC}"
        check_file "programs/$program/Cargo.toml" "  Cargo.toml"
        check_file "programs/$program/src/lib.rs" "  lib.rs"
        check_file "programs/$program/src/error.rs" "  error.rs"
        check_file "programs/$program/src/state.rs" "  state.rs"
    fi
done

echo ""
echo "=============================="
echo "Verification Summary"
echo "=============================="
echo -e "Checks passed: ${GREEN}$CHECKS_PASSED${NC}"
echo -e "Checks failed: ${RED}$CHECKS_FAILED${NC}"

if [ $CHECKS_FAILED -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✅ All implementation files are present!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Run './scripts/build_microkernel.sh' to build all programs"
    echo "2. Run './scripts/test_microkernel.sh' to run all tests"
    echo "3. Deploy singleton services (Session Factory, FVT, Entrypoint)"
    echo "4. Deploy your program using the examples as a guide"
    exit 0
else
    echo ""
    echo -e "${RED}❌ Some files are missing. Please check the output above.${NC}"
    exit 1
fi