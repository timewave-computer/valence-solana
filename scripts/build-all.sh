#!/bin/bash
set -e

echo "=== Building Complete Valence Solana Project ==="
echo ""

# Change to project root
cd "$(dirname "$0")/.."

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Build on-chain programs first
echo -e "${YELLOW}Step 1: Building on-chain programs...${NC}"
if ./scripts/build-onchain.sh; then
    echo -e "${GREEN}✓ On-chain programs built successfully${NC}"
else
    echo -e "${RED}✗ On-chain build failed${NC}"
    exit 1
fi

echo ""

# Build off-chain components
echo -e "${YELLOW}Step 2: Building off-chain components...${NC}"
if ./scripts/build-offchain.sh; then
    echo -e "${GREEN}✓ Off-chain components built successfully${NC}"
else
    echo -e "${RED}✗ Off-chain build failed${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}=== Complete Build Successful ===${NC}"
echo ""
echo "All components have been built:"
echo "- On-chain programs (.so files) in target/deploy/"
echo "- Off-chain binaries in target/release/"
echo "- Libraries (.rlib files) in target/release/"
echo ""
echo "To deploy and run:"
echo "1. Deploy programs: solana program deploy target/deploy/<program>.so"
echo "2. Run services: ./target/release/session_builder"
echo "3. Use CLI: ./target/release/valence-cli --help"