#!/bin/bash

# Valence Protocol Development Environment Setup
# Sets up the development environment for the new singleton architecture

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Setting up Valence Protocol development environment...${NC}"

# Check if in nix shell
if [ -z "$IN_NIX_SHELL" ]; then
    echo -e "${YELLOW}Warning: Not in nix shell. Run 'nix develop' first for best results.${NC}"
fi

# Create necessary directories
echo -e "${BLUE}Creating project directories...${NC}"
mkdir -p target/deploy
mkdir -p tests/integration
mkdir -p logs

# Generate keypair if needed
if [ ! -f ~/.config/solana/id.json ]; then
    echo -e "${BLUE}Generating Solana keypair...${NC}"
    solana-keygen new --no-passphrase || echo "Solana tools not available"
fi

# Set Solana to localnet by default
echo -e "${BLUE}Configuring Solana for localnet...${NC}"
solana config set --url localhost || echo "Solana tools not available"

# Create local environment file
echo -e "${BLUE}Creating local environment configuration...${NC}"
cat > .env.local << EOF
# Valence Protocol Local Development Configuration
RUST_LOG=info
SOLANA_RPC_URL=http://localhost:8899
ANCHOR_PROVIDER_URL=http://localhost:8899
ANCHOR_WALLET=~/.config/solana/id.json

# Singleton Configuration
CORE_PROGRAM_ID=11111111111111111111111111111112
SESSION_BUILDER_SERVICE_ID=11111111111111111111111111111114

# Development Settings
SKIP_TESTS=false
ENABLE_LOGS=true
EOF

echo -e "${GREEN}✓ Development environment setup complete!${NC}"
echo ""
echo "Next steps:"
echo "1. Run 'nix develop' to enter the development shell"
echo "2. Run 'solana-test-validator' to start local validator"
echo "3. Run 'anchor build' to build the programs"
echo "4. Run 'anchor test' to run tests"