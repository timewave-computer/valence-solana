#!/bin/bash
set -e

echo "=== Building On-Chain Programs (Solana) ==="
echo "This builds only programs that run on the Solana blockchain"
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Change to project root
cd "$(dirname "$0")/.."

# Build the kernel program
echo -e "${YELLOW}Building valence-kernel...${NC}"
cd programs/kernel

if cargo build-sbf; then
    echo -e "${GREEN}✓ valence-kernel built successfully${NC}"
    echo -e "  Deployed program: ../../target/deploy/valence_kernel.so"
else
    echo -e "${RED}✗ Failed to build valence-kernel${NC}"
    exit 1
fi

cd ../..

# Add other on-chain programs here as they are created
# echo -e "${YELLOW}Building other-program...${NC}"
# cd programs/other-program
# cargo build-sbf
# cd ../..

echo ""
echo -e "${GREEN}=== On-Chain Build Complete ===${NC}"
echo ""
echo "Generated artifacts:"
find target/deploy -name "*.so" -type f | while read -r file; do
    echo "  - $(basename "$file")"
done

echo ""
echo "Next steps:"
echo "1. Deploy programs using: solana program deploy target/deploy/<program>.so"
echo "2. Build off-chain components using: ./scripts/build-offchain.sh"