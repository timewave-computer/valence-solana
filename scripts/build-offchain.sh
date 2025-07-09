#!/bin/bash
set -e

# This script builds off-chain components using your system's cargo
# Requirements:
#   - Rust toolchain (rustup or system package)
#   - Can use latest Rust edition (including Edition 2024)
#   - Full access to crates.io ecosystem
#
# Note: This intentionally does NOT use nix develop to avoid
# conflicts with Solana's platform tools

echo "=== Building Off-Chain Components ==="
echo "This builds client libraries, SDKs, and services"
echo "Using system cargo: $(which cargo || echo 'NOT FOUND')"
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Change to project root
cd "$(dirname "$0")/.."

# Track build results
FAILED_BUILDS=()

# Function to build a component
build_component() {
    local component_name=$1
    local component_path=$2
    
    echo -e "${YELLOW}Building ${component_name}...${NC}"
    
    if cd "$component_path" && cargo build --release 2>&1; then
        echo -e "${GREEN}✓ ${component_name} built successfully${NC}"
        cd - > /dev/null
        return 0
    else
        echo -e "${RED}✗ Failed to build ${component_name}${NC}"
        cd - > /dev/null
        FAILED_BUILDS+=("$component_name")
        return 1
    fi
}

# Build SDK
build_component "Valence SDK" "programs/sdk"

# Build Session Builder Service
build_component "Session Builder" "programs/services/session_builder"

# Build tests (but don't run them)
echo -e "${YELLOW}Building test suite...${NC}"
if cargo build --tests --workspace --exclude valence-kernel; then
    echo -e "${GREEN}✓ Test suite built successfully${NC}"
else
    echo -e "${RED}✗ Failed to build test suite${NC}"
    FAILED_BUILDS+=("Test suite")
fi

echo ""
echo "=============================="
echo "Build Summary"
echo "=============================="

if [ ${#FAILED_BUILDS[@]} -eq 0 ]; then
    echo -e "${GREEN}✓ All off-chain components built successfully!${NC}"
    
    echo ""
    echo "Built artifacts:"
    echo "  - SDK library: target/release/libvalence_sdk.rlib"
    echo "  - SDK CLI: target/release/valence-cli"
    echo "  - Session Builder: target/release/session_builder"
else
    echo -e "${RED}✗ Build failed for the following components:${NC}"
    for component in "${FAILED_BUILDS[@]}"; do
        echo "  - $component"
    done
    exit 1
fi

echo ""
echo "Next steps:"
echo "1. Run tests: cargo test --workspace --exclude valence-kernel"
echo "2. Run services: ./target/release/session_builder"
echo "3. Use SDK: ./target/release/valence-cli --help"