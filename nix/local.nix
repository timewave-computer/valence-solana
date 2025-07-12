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
  
  # Launch complete local Valence environment
  valence-local = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-local" ''
      set -e
      
      echo "=== Launching Valence Local Development Environment ==="
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
        
        # Kill session builder if running
        if [ ! -z "$SESSION_BUILDER_PID" ]; then
          kill $SESSION_BUILDER_PID 2>/dev/null || true
        fi
        
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
      GATEWAY_SO=""
      REGISTRY_SO=""
      VERIFIER_SO=""
      
      # Try deploy directory first
      if [ -f target/deploy/valence_gateway.so ]; then
        GATEWAY_SO="target/deploy/valence_gateway.so"
      elif [ -f target/sbf-solana-solana/release/valence_gateway.so ]; then
        GATEWAY_SO="target/sbf-solana-solana/release/valence_gateway.so"
      fi
      
      if [ -f target/deploy/valence_registry.so ]; then
        REGISTRY_SO="target/deploy/valence_registry.so"
      elif [ -f target/sbf-solana-solana/release/valence_registry.so ]; then
        REGISTRY_SO="target/sbf-solana-solana/release/valence_registry.so"
      fi
      
      if [ -f target/deploy/valence_verifier.so ]; then
        VERIFIER_SO="target/deploy/valence_verifier.so"
      elif [ -f target/sbf-solana-solana/release/valence_verifier.so ]; then
        VERIFIER_SO="target/sbf-solana-solana/release/valence_verifier.so"
      fi
      
      # Build if any are missing
      if [ -z "$GATEWAY_SO" ] || [ -z "$REGISTRY_SO" ] || [ -z "$VERIFIER_SO" ]; then
        echo "Programs not found, building..."
        nix run .#build-onchain
        
        # Set paths after build
        GATEWAY_SO="target/sbf-solana-solana/release/valence_gateway.so"
        REGISTRY_SO="target/sbf-solana-solana/release/valence_registry.so"
        VERIFIER_SO="target/sbf-solana-solana/release/valence_verifier.so"
        
        # Verify build succeeded
        if [ ! -f "$GATEWAY_SO" ]; then
          echo -e "''${RED}Failed to build programs''${NC}"
          exit 1
        fi
      else
        echo -e "''${GREEN}Programs already built''${NC}"
      fi
      
      # Copy to deploy directory for consistency (if not already there)
      if [ "$GATEWAY_SO" != "target/deploy/valence_gateway.so" ]; then
        cp "$GATEWAY_SO" target/deploy/valence_gateway.so
      fi
      if [ "$REGISTRY_SO" != "target/deploy/valence_registry.so" ]; then
        cp "$REGISTRY_SO" target/deploy/valence_registry.so
      fi
      if [ "$VERIFIER_SO" != "target/deploy/valence_verifier.so" ]; then
        cp "$VERIFIER_SO" target/deploy/valence_verifier.so
      fi
      
      # Deploy singleton programs
      echo ""
      echo -e "''${YELLOW}Deploying singleton programs...''${NC}"
      
      # Deploy Gateway
      echo -e "''${BLUE}Deploying Gateway...''${NC}"
      DEPLOY_OUTPUT=$(solana program deploy target/deploy/valence_gateway.so 2>&1)
      GATEWAY_ID=$(echo "$DEPLOY_OUTPUT" | grep -E "Program Id:|Deployed program" | awk '{print $NF}' | head -1)
      
      if [ -z "$GATEWAY_ID" ] || [[ "$GATEWAY_ID" == *"Error"* ]]; then
        echo -e "''${RED}Failed to deploy Gateway''${NC}"
        echo "$DEPLOY_OUTPUT"
        exit 1
      fi
      echo -e "''${GREEN}Gateway deployed at: $GATEWAY_ID''${NC}"
      
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
      
      # Deploy Verifier
      echo -e "''${BLUE}Deploying Verifier...''${NC}"
      DEPLOY_OUTPUT=$(solana program deploy target/deploy/valence_verifier.so 2>&1)
      VERIFIER_ID=$(echo "$DEPLOY_OUTPUT" | grep -E "Program Id:|Deployed program" | awk '{print $NF}' | head -1)
      
      if [ -z "$VERIFIER_ID" ] || [[ "$VERIFIER_ID" == *"Error"* ]]; then
        echo -e "''${RED}Failed to deploy Verifier''${NC}"
        echo "$DEPLOY_OUTPUT"
        exit 1
      fi
      echo -e "''${GREEN}Verifier deployed at: $VERIFIER_ID''${NC}"
      
      # Save program IDs to config file
      CONFIG_FILE="$HOME/.valence/local-config.json"
      mkdir -p "$HOME/.valence"
      cat > "$CONFIG_FILE" <<EOF
      {
        "gateway": "$GATEWAY_ID",
        "registry": "$REGISTRY_ID",
        "verifier": "$VERIFIER_ID",
        "rpc_url": "http://localhost:8899"
      }
      EOF
      
      echo ""
      echo -e "''${GREEN}Program IDs saved to: $CONFIG_FILE''${NC}"
      
      # Build session builder if not already built
      echo ""
      echo -e "''${YELLOW}Checking session builder service...''${NC}"
      if [ ! -f services/session_builder/target/release/session_builder ]; then
        echo -e "''${YELLOW}Session builder not found. Build it separately with:''${NC}"
        echo "  cargo build --release --manifest-path services/session_builder/Cargo.toml"
        echo ""
        echo -e "''${YELLOW}Continuing without session builder for now...''${NC}"
        SKIP_SESSION_BUILDER=true
      else
        echo -e "''${GREEN}Session builder already built''${NC}"
        SKIP_SESSION_BUILDER=false
      fi
      
      # Start session builder service if available
      if [ "$SKIP_SESSION_BUILDER" = "false" ]; then
        echo ""
        echo -e "''${YELLOW}Starting session builder service...''${NC}"
        
        # Create session builder config
        SESSION_CONFIG="$HOME/.valence/session-builder-config.json"
        cat > "$SESSION_CONFIG" <<EOF
      {
        "rpc_url": "http://localhost:8899",
        "keypair_path": "$HOME/.config/solana/id.json",
        "shard_program_id": "11111111111111111111111111111114",
        "poll_interval_secs": 5,
        "max_retries": 3
      }
      EOF
        
        # Run session builder in background
        ./services/session_builder/target/release/session_builder --config "$SESSION_CONFIG" &
        SESSION_BUILDER_PID=$!
        
        sleep 2
        
        # Check if session builder is running
        if ! kill -0 $SESSION_BUILDER_PID 2>/dev/null; then
          echo -e "''${RED}Failed to start session builder''${NC}"
          exit 1
        fi
        
        echo -e "''${GREEN}Session builder started successfully''${NC}"
      else
        SESSION_BUILDER_PID=""
      fi
      
      # Print summary
      echo ""
      echo "========================================="
      echo -e "''${GREEN}Valence Local Environment Running!''${NC}"
      echo "========================================="
      echo ""
      echo "Deployed Programs:"
      echo "  Gateway:  $GATEWAY_ID"
      echo "  Registry: $REGISTRY_ID"  
      echo "  Verifier: $VERIFIER_ID"
      echo ""
      echo "Services:"
      echo "  Solana RPC: http://localhost:8899"
      if [ -n "$SESSION_BUILDER_PID" ]; then
        echo "  Session Builder: Running (PID: $SESSION_BUILDER_PID)"
      else
        echo "  Session Builder: Not running (build separately)"
      fi
      echo ""
      echo "Configuration saved to:"
      echo "  $CONFIG_FILE"
      echo "  $SESSION_CONFIG"
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