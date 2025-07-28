# Local development environment commands
{
  pkgs,
  inputs',
  ...
}: {
  # Run local Solana node (without dev tools)
  node = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-node" ''
      set -e
      
      echo "Starting local Solana test validator..."
      echo "This uses only the solana-node package (no dev tools)"
      
      # Use only the node package
      ${inputs'.zero-nix.packages.solana-node}/bin/solana-test-validator
    ''}/bin/valence-node";
  };
  
  # Launch local devnet with deployed programs for e2e testing
  local-devnet = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "local-devnet" ''
      set -e
      
      echo "=== Launching Valence Local Devnet ==="
      echo ""
      
      # Colors for output
      GREEN='\033[0;32m'
      YELLOW='\033[1;33m'
      RED='\033[0;31m'
      BLUE='\033[0;34m'
      NC='\033[0m' # No Color
      
      # Set up environment
      export PATH="${inputs'.zero-nix.packages.solana-node}/bin:${inputs'.zero-nix.packages.solana-tools}/bin:${inputs'.zero-nix.packages.nightly-rust}/bin:$PATH"
      export RUST_BACKTRACE=1
      export RUST_LOG=info
      export MACOSX_DEPLOYMENT_TARGET=11.0
      export PKG_CONFIG_PATH=${pkgs.openssl.dev}/lib/pkgconfig
      export OPENSSL_DIR=${pkgs.openssl.dev}
      export OPENSSL_LIB_DIR=${pkgs.openssl.out}/lib
      export OPENSSL_INCLUDE_DIR=${pkgs.openssl.dev}/include
      
      # Create temporary directory for validator
      VALIDATOR_DIR=$(mktemp -d)
      echo -e "''${BLUE}Validator directory: $VALIDATOR_DIR''${NC}"
      
      # Start Solana validator in background
      echo -e "''${YELLOW}Starting local Solana validator...''${NC}"
      solana-test-validator \
        --ledger "$VALIDATOR_DIR" \
        --rpc-port 8899 \
        --quiet &
      VALIDATOR_PID=$!
      
      # Function to cleanup on exit
      cleanup() {
        echo ""
        echo -e "''${YELLOW}Shutting down...''${NC}"
        
        # Kill validator
        kill $VALIDATOR_PID 2>/dev/null || true
        
        # Clean up temp directory
        rm -rf "$VALIDATOR_DIR"
        
        echo -e "''${GREEN}Cleanup complete''${NC}"
      }
      trap cleanup EXIT
      
      # Wait for validator to start
      echo -e "''${YELLOW}Waiting for validator to start...''${NC}"
      sleep 5
      
      # Check if validator is running
      if ! solana cluster-version --url http://localhost:8899 >/dev/null 2>&1; then
        echo -e "''${RED}Failed to start validator''${NC}"
        exit 1
      fi
      
      echo -e "''${GREEN}Validator started successfully''${NC}"
      
      # Configure Solana CLI
      solana config set --url http://localhost:8899
      
      # Create keypair if it doesn't exist
      if [ ! -f ~/.config/solana/id.json ]; then
        echo -e "''${YELLOW}Creating default keypair...''${NC}"
        solana-keygen new --no-passphrase
      fi
      
      # Get some SOL for deployment
      echo -e "''${YELLOW}Requesting airdrop...''${NC}"
      solana airdrop 10 >/dev/null 2>&1 || true
      
      # Build programs if not already built
      echo -e "''${YELLOW}Checking for built programs...''${NC}"
      
      # Create deploy directory if it doesn't exist
      mkdir -p target/deploy
      
      # Check if programs exist in either location
      REGISTRY_SO=""
      SHARD_SO=""
      
      # Try deploy directory first
      if [ -f target/deploy/valence_registry.so ]; then
        REGISTRY_SO="target/deploy/valence_registry.so"
      elif [ -f target/sbf-solana-solana/release/valence_registry.so ]; then
        REGISTRY_SO="target/sbf-solana-solana/release/valence_registry.so"
      fi
      
      if [ -f target/deploy/valence_shard.so ]; then
        SHARD_SO="target/deploy/valence_shard.so"
      elif [ -f target/sbf-solana-solana/release/valence_shard.so ]; then
        SHARD_SO="target/sbf-solana-solana/release/valence_shard.so"
      fi
      
      # Build if any are missing
      if [ -z "$REGISTRY_SO" ] || [ -z "$SHARD_SO" ]; then
        echo "Programs not found, building..."
        nix run .#build-onchain
        
        # Set paths after build
        REGISTRY_SO="target/sbf-solana-solana/release/valence_registry.so"
        SHARD_SO="target/sbf-solana-solana/release/valence_shard.so"
        
        # Verify build succeeded
        if [ ! -f "$REGISTRY_SO" ]; then
          echo -e "''${RED}Failed to build programs''${NC}"
          exit 1
        fi
      else
        echo -e "''${GREEN}Programs already built''${NC}"
      fi
      
      # Copy to deploy directory for consistency (if not already there)
      if [ "$REGISTRY_SO" != "target/deploy/valence_registry.so" ]; then
        cp "$REGISTRY_SO" target/deploy/valence_registry.so
      fi
      if [ "$SHARD_SO" != "target/deploy/valence_shard.so" ]; then
        cp "$SHARD_SO" target/deploy/valence_shard.so
      fi
      
      # Deploy singleton programs
      echo ""
      echo -e "''${YELLOW}Deploying singleton programs...''${NC}"
      
      # Deploy Registry
      echo -e "''${BLUE}Deploying Registry...''${NC}"
      DEPLOY_OUTPUT=$(solana program deploy target/deploy/valence_registry.so 2>&1)
      REGISTRY_ID=$(echo "$DEPLOY_OUTPUT" | grep -E "Program Id:|Deployed program" | awk '{print $NF}' | head -1)
      
      if [ -z "$REGISTRY_ID" ] || [[ "$REGISTRY_ID" == *"Error"* ]]; then
        echo -e "''${RED}Failed to deploy Registry''${NC}"
        echo "$DEPLOY_OUTPUT"
        exit 1
      fi
      echo -e "''${GREEN}Registry deployed at: $REGISTRY_ID''${NC}"
      
      # Save program IDs to config file
      CONFIG_FILE="$HOME/.valence/local-config.json"
      mkdir -p "$HOME/.valence"
      cat > "$CONFIG_FILE" <<EOF
      {
        "registry": "$REGISTRY_ID",
        "rpc_url": "http://localhost:8899"
      }
      EOF
      
      echo ""
      echo -e "''${GREEN}Program IDs saved to: $CONFIG_FILE''${NC}"
      

      
      # Print summary
      echo ""
      echo "========================================="
      echo -e "''${GREEN}Valence Local Environment Running!''${NC}"
      echo "========================================="
      echo ""
      echo "Deployed Programs:"
      echo "  Registry: $REGISTRY_ID"  
      echo ""
      echo "Services:"
      echo "  Solana RPC: http://localhost:8899"
      echo ""
      echo "Configuration saved to:"
      echo "  $CONFIG_FILE"
      echo ""
      echo "To deploy a shard:"
      echo "  solana program deploy target/deploy/valence_shard.so"
      echo ""
      echo "To stop the environment:"
      echo "  Press Ctrl+C"
      echo ""
      echo "========================================="
      
      # Wait for interrupt
      wait $VALIDATOR_PID
    ''}/bin/valence-local";
  };
}