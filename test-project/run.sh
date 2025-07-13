#!/bin/bash
set -e

echo "=== Valence Shard Development Script ==="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Step 1: Build everything
echo -e "${YELLOW}Step 1: Building shard and client...${NC}"
nix run ..#valence-template-build

# Step 2: Deploy shard
echo -e "${YELLOW}Step 2: Deploying shard program...${NC}"
nix run ..#valence-template-deploy

# Load the deployed program ID
source .valence-env

# Step 3: Initialize shard
echo -e "${YELLOW}Step 3: Initializing shard...${NC}"
nix run ..#valence-template-init

# Step 4: Register functions
echo -e "${YELLOW}Step 4: Registering functions...${NC}"
nix run ..#valence-template-register

# Step 5: Create session
echo -e "${YELLOW}Step 5: Creating session...${NC}"
nix run ..#valence-template-session

# Step 6: Run client
echo -e "${YELLOW}Step 6: Running client...${NC}"
./target/release/test-project_client

echo ""
echo -e "${GREEN}=== Complete! ===${NC}"
