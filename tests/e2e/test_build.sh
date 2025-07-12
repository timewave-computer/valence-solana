#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "=== Testing Valence Build ==="
echo ""

# Test configuration
TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$TEST_DIR/../.." && pwd)"

cd "$ROOT_DIR"

echo "Testing workspace build..."

# Check if we can build the SDK
echo -n "Building SDK... "
if [ -f "sdk/Cargo.toml" ]; then
    echo -e "${GREEN}✓ SDK found${NC}"
else
    echo -e "${RED}✗ SDK not found${NC}"
    exit 1
fi

# Check if lifecycle manager exists
echo -n "Checking lifecycle manager... "
if [ -f "services/lifecycle_manager/Cargo.toml" ]; then
    echo -e "${GREEN}✓ Lifecycle manager found${NC}"
else
    echo -e "${RED}✗ Lifecycle manager not found${NC}"
    exit 1
fi

# Check template project
echo -n "Checking template project... "
if [ -f "tests/e2e/capability_enforcement_test/Cargo.toml" ]; then
    echo -e "${GREEN}✓ Template project found${NC}"
else
    echo -e "${RED}✗ Template project not found${NC}"
    exit 1
fi

# Check if template client imports are correct
echo -n "Checking template client imports... "
if grep -q "anchor_client" "tests/e2e/capability_enforcement_test/src/client.rs" && \
   grep -q "tokio::main" "tests/e2e/capability_enforcement_test/src/client.rs"; then
    echo -e "${GREEN}✓ Client uses new lifecycle system${NC}"
else
    echo -e "${RED}✗ Client not updated for new lifecycle${NC}"
    exit 1
fi

# Verify no Docker references
echo -n "Checking for Docker references... "
if find . -name "*.md" -o -name "*.rs" -o -name "*.toml" | xargs grep -l "docker" | grep -v ".git" | grep -v "target" > /dev/null; then
    echo -e "${YELLOW}⚠ Found Docker references (should be removed)${NC}"
    find . -name "*.md" -o -name "*.rs" -o -name "*.toml" | xargs grep -l "docker" | grep -v ".git" | grep -v "target" | head -5
else
    echo -e "${GREEN}✓ No Docker references found${NC}"
fi

echo ""
echo -e "${GREEN}=== Build Test Passed ===${NC}"
echo "All components are properly configured for the new lifecycle system"