{
  description = "Solana development environment for Valence Protocol (updated)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        
        # Specify Solana version explicitly
        solana-version = "1.17.20";
        anchor-version = "0.29.0";
        defaultDevnet = "https://api.devnet.solana.com";
        
        # Use stable Rust with specific components needed for Solana/Anchor development
        rustWithComponents = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        # Environment variables for macOS with Apple Silicon
        macosMacOSEnvironment = pkgs.lib.optionalAttrs (system == "aarch64-darwin" || system == "x86_64-darwin") {
          MACOSX_DEPLOYMENT_TARGET = "11.0";
          CARGO_BUILD_TARGET = if system == "aarch64-darwin" then "aarch64-apple-darwin" else "x86_64-apple-darwin";
          RUSTFLAGS = "-C link-arg=-undefined -C link-arg=dynamic_lookup";
          BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.libclang.lib}/lib/clang/${pkgs.libclang.version}/include";
        };

        # Define a script to install Anchor with specific version
        setupAnchorScript = pkgs.writeShellScriptBin "setup-anchor" ''
          if ! command -v anchor &> /dev/null; then
            echo "Installing Anchor CLI ${anchor-version}..."
            cargo install --git https://github.com/coral-xyz/anchor --tag v${anchor-version} anchor-cli
          else
            current_version=$(anchor --version | cut -d' ' -f2)
            if [ "$current_version" != "${anchor-version}" ]; then
              echo "Updating Anchor CLI to ${anchor-version}..."
              cargo install --git https://github.com/coral-xyz/anchor --tag v${anchor-version} anchor-cli --force
            else
              echo "Anchor CLI ${anchor-version} is already installed"
            fi
          fi
        '';
        
        # Comprehensive script to start a Solana validator with test accounts
        enhancedSolanaValidatorScript = pkgs.writeShellScriptBin "enhanced-validator" ''
          #!/usr/bin/env bash
          # Start a local Solana validator with test accounts
          # This script provides a convenient way to start, stop, and manage a local Solana validator

          set -e

          # Default values
          LEDGER_DIR="test-ledger"
          CLEAN=false
          WAIT_FOR_RESTART=false
          LOG_FILE="validator.log"
          QUIET=false
          FAUCET=false
          RESET_ACCOUNTS=false
          VERBOSE=false

          # Parse command line arguments
          while [[ $# -gt 0 ]]; do
            case "$1" in
              --ledger|-l)
                LEDGER_DIR="$2"
                shift 2
                ;;
              --clean|-c)
                CLEAN=true
                shift
                ;;
              --wait|-w)
                WAIT_FOR_RESTART=true
                shift
                ;;
              --log|-f)
                LOG_FILE="$2"
                shift 2
                ;;
              --quiet|-q)
                QUIET=true
                shift
                ;;
              --faucet|-t)
                FAUCET=true
                shift
                ;;
              --reset-accounts|-r)
                RESET_ACCOUNTS=true
                shift
                ;;
              --verbose|-v)
                VERBOSE=true
                shift
                ;;
              --help|-h)
                echo "Usage: enhanced-validator [OPTIONS]"
                echo "Options:"
                echo "  -l, --ledger <dir>       Ledger directory (default: test-ledger)"
                echo "  -c, --clean              Clean ledger before starting"
                echo "  -w, --wait               Wait for validator to restart if it crashes"
                echo "  -f, --log <file>         Log file (default: validator.log)"
                echo "  -q, --quiet              Minimize output"
                echo "  -t, --faucet             Start a faucet alongside the validator"
                echo "  -r, --reset-accounts     Reset accounts (useful during development)"
                echo "  -v, --verbose            Verbose output"
                echo "  -h, --help               Show this help message"
                exit 0
                ;;
              *)
                echo "Unknown option: $1"
                exit 1
                ;;
            esac
          done

          # Function to check if solana-test-validator is running
          is_validator_running() {
            pgrep -f "solana-test-validator" > /dev/null
          }

          # Function to check if solana-faucet is running
          is_faucet_running() {
            pgrep -f "solana-faucet" > /dev/null
          }

          # Function to kill running validator and faucet
          kill_validator_and_faucet() {
            if is_validator_running; then
              echo "Stopping existing validator..."
              pkill -f "solana-test-validator" || true
              sleep 2
            fi

            if is_faucet_running; then
              echo "Stopping existing faucet..."
              pkill -f "solana-faucet" || true
              sleep 1
            fi
          }

          # Clean the ledger directory if requested
          if $CLEAN && [ -d "$LEDGER_DIR" ]; then
            echo "Cleaning ledger directory: $LEDGER_DIR"
            rm -rf "$LEDGER_DIR"
          fi

          # Kill any existing validator and faucet processes
          kill_validator_and_faucet

          # Create log directory if it doesn't exist
          mkdir -p "$(dirname "$LOG_FILE")"
          touch "$LOG_FILE"

          # Create accounts directory
          mkdir -p accounts

          # Prepare validator args
          VALIDATOR_ARGS=()

          # Add RPC Bind to allow connections from other machines (useful for testing)
          VALIDATOR_ARGS+=(--rpc-port 8899 --rpc-bind-address 0.0.0.0)

          # Add bpf program log filter settings
          VALIDATOR_ARGS+=(--log "solana=info")

          if $VERBOSE; then
            VALIDATOR_ARGS+=(--log "solana_runtime::message_processor=debug")
            VALIDATOR_ARGS+=(--log "solana_bpf_loader=debug")
            VALIDATOR_ARGS+=(--log "solana_rbpf=debug")
          else
            VALIDATOR_ARGS+=(--log "solana_runtime::message_processor=info")
          fi

          # Add account wipe setting if requested
          if $RESET_ACCOUNTS; then
            VALIDATOR_ARGS+=(--reset)
          fi

          # Add ledger directory
          VALIDATOR_ARGS+=(--ledger "$LEDGER_DIR")

          echo "Starting Solana validator with args: ''${VALIDATOR_ARGS[@]}"

          # Start the validator in the background
          if $QUIET; then
            solana-test-validator "''${VALIDATOR_ARGS[@]}" > "$LOG_FILE" 2>&1 &
          else
            solana-test-validator "''${VALIDATOR_ARGS[@]}" | tee "$LOG_FILE" &
          fi

          VALIDATOR_PID=$!

          # Setup trap to clean up on exit
          trap "echo 'Shutting down validator (PID: $VALIDATOR_PID)'; kill -9 $VALIDATOR_PID 2>/dev/null || true; exit 0" INT TERM

          # Wait for validator to start
          echo "Waiting for validator to start..."
          for i in {1..30}; do
            if solana cluster-version; then
              break
            fi
            sleep 1
            echo -n "."
            if [ $i -eq 30 ]; then
              echo "Timed out waiting for validator to start"
              exit 1
            fi
          done
          echo "Validator started successfully!"

          # Start faucet if requested
          if $FAUCET; then
            echo "Starting Solana faucet..."
            solana-faucet &
            FAUCET_PID=$!
            trap "echo 'Shutting down validator and faucet'; kill -9 $VALIDATOR_PID $FAUCET_PID 2>/dev/null || true; exit 0" INT TERM
          fi

          # Display validator status
          solana validators
          solana block-production
          solana epoch-info

          # Airdrop some SOL to the default wallet for testing
          WALLET_FILE="$HOME/.config/solana/id.json"
          if [ -f "$WALLET_FILE" ]; then
            WALLET_PUBKEY=$(solana-keygen pubkey "$WALLET_FILE")
            echo "Airdropping 1000 SOL to wallet: $WALLET_PUBKEY"
            solana airdrop 1000 "$WALLET_PUBKEY"
            echo "Current balance: $(solana balance) SOL"
          fi

          # If --wait is specified, restart the validator if it crashes
          if $WAIT_FOR_RESTART; then
            echo "Validator will automatically restart if it crashes (--wait specified)"
            while true; do
              if ! kill -0 "$VALIDATOR_PID" 2>/dev/null; then
                echo "Validator crashed, restarting..."
                solana-test-validator "''${VALIDATOR_ARGS[@]}" | tee -a "$LOG_FILE" &
                VALIDATOR_PID=$!
                echo "New validator PID: $VALIDATOR_PID"
              fi
              sleep 5
            done
          else
            # Otherwise, just wait for the validator to exit
            wait "$VALIDATOR_PID"
          fi
        '';
        
        # Simple Script to start a Solana validator
        solanaNodeScript = pkgs.writeShellScriptBin "start-solana-node" ''
          echo "Starting Solana validator..."
          solana-test-validator "$@"
        '';
        
        # Script to setup a local Solana wallet and configuration
        setupLocalScript = pkgs.writeShellScriptBin "setup-solana-local" ''
          # Configure Solana to use localhost
          solana config set --url http://127.0.0.1:8899
          
          # Create a wallet if it doesn't exist
          if [ ! -f "$HOME/.config/solana/id.json" ]; then
            echo "Creating a new Solana wallet..."
            solana-keygen new --no-bip39-passphrase --force
          fi
          
          # Airdrop some SOL
          echo "Airdropping SOL to wallet..."
          solana airdrop 100
          solana balance
          
          echo "Local Solana configuration completed!"
        '';
        
        # Script to create a new Anchor workspace
        createAnchorWorkspaceScript = pkgs.writeShellScriptBin "create-anchor-workspace" ''
          if [ $# -ne 1 ]; then
            echo "Usage: create-anchor-workspace <workspace-name>"
            exit 1
          fi
          
          WORKSPACE_NAME=$1
          
          if [ -d "$WORKSPACE_NAME" ]; then
            echo "Directory $WORKSPACE_NAME already exists!"
            exit 1
          fi
          
          echo "Creating new Anchor workspace: $WORKSPACE_NAME"
          anchor init "$WORKSPACE_NAME"
          cd "$WORKSPACE_NAME"
          
          # Update Anchor.toml with better defaults
          cat > Anchor.toml << EOF
[features]
seeds = true
skip-lint = false

[programs.localnet]
$WORKSPACE_NAME = "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"

[registry]
url = "https://api.apr.dev"

[provider]
cluster = "localnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
EOF
          
          echo "Workspace created successfully! CD into $WORKSPACE_NAME to get started."
        '';
        
        # Script to build and test the Anchor project
        buildAndTestScript = pkgs.writeShellScriptBin "build-and-test" ''
          echo "Building Anchor project..."
          anchor build
          
          echo "Running tests..."
          anchor test
        '';
        
        # Script for creating a new program in the workspace
        createProgramScript = pkgs.writeShellScriptBin "create-program" ''
          if [ $# -ne 1 ]; then
            echo "Usage: create-program <program-name>"
            exit 1
          fi
          
          PROGRAM_NAME=$1
          
          if [ ! -f "Anchor.toml" ]; then
            echo "Not in an Anchor workspace! Please run this from the root of an Anchor workspace."
            exit 1
          fi
          
          if [ -d "programs/$PROGRAM_NAME" ]; then
            echo "Program $PROGRAM_NAME already exists!"
            exit 1
          fi
          
          echo "Creating new program: $PROGRAM_NAME"
          mkdir -p "programs/$PROGRAM_NAME/src"
          
          # Create lib.rs with basic program structure
          cat > "programs/$PROGRAM_NAME/src/lib.rs" << EOF
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod $PROGRAM_NAME {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
EOF
          
          # Create Cargo.toml for the program
          cat > "programs/$PROGRAM_NAME/Cargo.toml" << EOF
[package]
name = "$PROGRAM_NAME"
version = "0.1.0"
description = "Created with Solana Nix environment"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]
name = "$PROGRAM_NAME"

[features]
no-entrypoint = []
no-idl = []
no-log-ix-name = []
cpi = ["no-entrypoint"]
default = []

[dependencies]
anchor-lang = "${anchor-version}"
EOF
          
          # Update workspace Cargo.toml to include the new program
          sed -i "/members = \[/a \    \"programs/$PROGRAM_NAME\"," Cargo.toml
          
          # Add the program to Anchor.toml
          PROGRAM_LINE="\n$PROGRAM_NAME = \"Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS\""
          sed -i "/\[programs.localnet\]/a $PROGRAM_LINE" Anchor.toml
          
          echo "Program $PROGRAM_NAME created successfully!"
        '';
        
      in {
        packages = {
          inherit setupAnchorScript;
          inherit solanaNodeScript;
          inherit setupLocalScript;
          inherit createAnchorWorkspaceScript;
          inherit buildAndTestScript;
          inherit createProgramScript;
          inherit enhancedSolanaValidatorScript;
          default = pkgs.buildEnv {
            name = "valence-solana-environment";
            paths = [
              setupAnchorScript
              solanaNodeScript
              setupLocalScript
              createAnchorWorkspaceScript
              buildAndTestScript
              createProgramScript
              enhancedSolanaValidatorScript
              pkgs.solana-cli
            ];
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Solana tools with explicit version
            solana-cli
            setupAnchorScript
            solanaNodeScript
            setupLocalScript
            createAnchorWorkspaceScript
            buildAndTestScript
            createProgramScript
            enhancedSolanaValidatorScript
            
            # Rust development
            rustWithComponents
            pkg-config
            
            # Build dependencies
            openssl
            libiconv
            nodePackages.pnpm
            nodejs
            yarn
            python3
            
            # Development tools
            gnused
            jq
            ripgrep
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            pkgs.darwin.apple_sdk.frameworks.AppKit
          ];

          shellHook = ''
            echo "Entering Valence Solana development environment..."
            export PATH=$PATH:$HOME/.cargo/bin
            ${setupAnchorScript}/bin/setup-anchor
            # Check if devnet needs to be updated
            if [ -z "$(solana config get | grep 'RPC URL: ${defaultDevnet}')" ]; then
              echo "Setting up devnet configuration..."
              solana config set --url ${defaultDevnet}
            fi
            echo "Solana configuration:"
            solana config get
          '';

          # Include Apple Silicon environment variables
          inherit (macosMacOSEnvironment) MACOSX_DEPLOYMENT_TARGET CARGO_BUILD_TARGET RUSTFLAGS BINDGEN_EXTRA_CLANG_ARGS;
        };
      }
    );
} 