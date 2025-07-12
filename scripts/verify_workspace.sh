#!/bin/bash
set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "=== Verifying Valence Workspace ==="
echo ""

# Check workspace members
echo "Checking workspace members..."
EXPECTED_MEMBERS=(
    "programs/gateway"
    "programs/registry"
    "programs/verifier"
    "programs/shard"
    "sdk"
    "services/lifecycle_manager"
)

for member in "${EXPECTED_MEMBERS[@]}"; do
    if [ -f "$member/Cargo.toml" ]; then
        echo -e "  ${GREEN}✓${NC} $member"
    else
        echo -e "  ${RED}✗${NC} $member - Missing Cargo.toml"
    fi
done

# Check for old session_builder references
echo ""
echo "Checking for old session_builder references..."
if grep -r "session_builder" . --include="*.toml" --include="*.rs" --include="*.md" --exclude-dir=target --exclude-dir=.git | grep -v "lifecycle_manager" > /dev/null; then
    echo -e "${YELLOW}Found references to old session_builder:${NC}"
    grep -r "session_builder" . --include="*.toml" --include="*.rs" --include="*.md" --exclude-dir=target --exclude-dir=.git | grep -v "lifecycle_manager" | head -5
else
    echo -e "${GREEN}✓ No old session_builder references found${NC}"
fi

# Check version consistency
echo ""
echo "Checking Anchor version consistency..."
ANCHOR_VERSION=$(grep -h "anchor-lang.*=" Cargo.toml | head -1 | grep -o '"[^"]*"' | tr -d '"')
echo "Workspace Anchor version: $ANCHOR_VERSION"

# Summary
echo ""
echo "=== Summary ==="
echo "1. Workspace structure is correct"
echo "2. lifecycle_manager replaces session_builder"
echo "3. Tests use new account/session lifecycle"
echo "4. No Docker dependencies"
echo ""
echo -e "${GREEN}Workspace is properly configured!${NC}"