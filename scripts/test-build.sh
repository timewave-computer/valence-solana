#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "================================================"
echo "Valence Component Build Test (with Nix)"
echo "================================================"
echo ""

# Arrays to track results
declare -a SUCCEEDED=()
declare -a FAILED=()
declare -a SKIPPED=()

# Function to test building a component
test_component() {
    local component_path="$1"
    local component_name="$2"
    
    echo -e "${BLUE}Testing: $component_name${NC}"
    echo "Path: $component_path"
    
    if [ ! -d "$component_path" ]; then
        echo -e "${YELLOW}[SKIP]${NC} Directory not found"
        SKIPPED+=("$component_name")
        echo ""
        return
    fi
    
    if [ ! -f "$component_path/Cargo.toml" ]; then
        echo -e "${YELLOW}[SKIP]${NC} No Cargo.toml found"
        SKIPPED+=("$component_name")
        echo ""
        return
    fi
    
    echo "Running: cargo build (in nix develop)"
    
    # Run cargo build in nix develop environment
    if nix develop -c bash -c "cd '$component_path' && cargo build 2>&1" > build_output.tmp 2>&1; then
        echo -e "${GREEN}[SUCCESS]${NC} Build completed successfully"
        SUCCEEDED+=("$component_name")
    else
        echo -e "${RED}[FAILED]${NC} Build failed"
        FAILED+=("$component_name")
        echo ""
        echo "Error details:"
        grep -E "error\[E[0-9]+\]:|error:|^error:" build_output.tmp | head -n 5
        echo ""
        echo "First 10 lines of output:"
        head -n 10 build_output.tmp
    fi
    
    rm -f build_output.tmp
    echo ""
    echo "---"
    echo ""
}

# Test workspace root
test_component "." "Workspace Root"

# Test programs
echo -e "${BLUE}=== PROGRAMS ===${NC}"
echo ""

for prog in registry shard test_function; do
    test_component "programs/$prog" "Program: $prog"
done

# Test SDK
echo -e "${BLUE}=== SDK ===${NC}"
echo ""
test_component "sdk" "SDK"

# Note: No services directory in current project structure

# Test integration tests
echo -e "${BLUE}=== TEST PROJECTS ===${NC}"
echo ""
test_component "tests/integration" "Integration Tests"

# Summary
echo ""
echo "================================================"
echo "Build Test Summary"
echo "================================================"
echo -e "${GREEN}Succeeded:${NC} ${#SUCCEEDED[@]}"
if [ ${#SUCCEEDED[@]} -gt 0 ]; then
    printf "  - %s\n" "${SUCCEEDED[@]}"
fi

echo ""
echo -e "${RED}Failed:${NC} ${#FAILED[@]}"
if [ ${#FAILED[@]} -gt 0 ]; then
    printf "  - %s\n" "${FAILED[@]}"
fi

echo ""
echo -e "${YELLOW}Skipped:${NC} ${#SKIPPED[@]}"
if [ ${#SKIPPED[@]} -gt 0 ]; then
    printf "  - %s\n" "${SKIPPED[@]}"
fi

echo ""
echo "================================================"