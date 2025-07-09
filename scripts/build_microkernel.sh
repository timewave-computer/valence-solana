#!/bin/bash
# Build script for Valence Protocol Microkernel components

set -e

echo "=== Building Valence Protocol Microkernel ==="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to build a program
build_program() {
    local program_name=$1
    local program_path=$2
    
    echo -e "${YELLOW}Building ${program_name}...${NC}"
    
    if cd "$program_path" && cargo build-sbf 2>/dev/null || cargo build-bpf 2>/dev/null; then
        echo -e "${GREEN}✓ ${program_name} built successfully${NC}"
        return 0
    else
        echo -e "${RED}✗ Failed to build ${program_name}${NC}"
        return 1
    fi
}

# Track build results
FAILED_BUILDS=()

# Build singleton services
echo "Building Singleton Services..."
echo "=============================="

build_program "Session Factory" "programs/session_factory" || FAILED_BUILDS+=("Session Factory")
build_program "Function Verification Table" "programs/function_verification_table" || FAILED_BUILDS+=("Function Verification Table")
build_program "Entrypoint" "programs/entrypoint" || FAILED_BUILDS+=("Entrypoint")

echo ""

# Build per-program components
echo "Building Per-Program Components..."
echo "=================================="

build_program "Shard Contract" "programs/shard_contract" || FAILED_BUILDS+=("Shard Contract")
build_program "Eval" "programs/eval" || FAILED_BUILDS+=("Eval")
build_program "Valence Session" "programs/valence_session" || FAILED_BUILDS+=("Valence Session")

echo ""

# Build verification functions
echo "Building Verification Functions..."
echo "================================="

build_program "Basic Permission Verifier" "programs/verification_functions/basic_permission" || FAILED_BUILDS+=("Basic Permission Verifier")
build_program "Parameter Constraint Verifier" "programs/verification_functions/parameter_constraint" || FAILED_BUILDS+=("Parameter Constraint Verifier")
build_program "ZK Proof Verifier" "programs/verification_functions/zk_proof" || FAILED_BUILDS+=("ZK Proof Verifier")

echo ""
echo "=============================="
echo "Build Summary"
echo "=============================="

if [ ${#FAILED_BUILDS[@]} -eq 0 ]; then
    echo -e "${GREEN}✓ All programs built successfully!${NC}"
    
    # List deployed program files
    echo ""
    echo "Deployed programs:"
    find target/deploy -name "*.so" -type f | while read -r file; do
        echo "  - $(basename "$file")"
    done
else
    echo -e "${RED}✗ Build failed for the following programs:${NC}"
    for program in "${FAILED_BUILDS[@]}"; do
        echo "  - $program"
    done
    exit 1
fi

echo ""
echo "Next steps:"
echo "1. Deploy singleton services first (Session Factory, FVT, Entrypoint)"
echo "2. Deploy your program's Shard and Eval"
echo "3. Register your program with Entrypoint"
echo "4. Create and register capabilities"
echo "5. Deploy sessions via Session Factory"