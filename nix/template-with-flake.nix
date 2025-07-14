# Template project commands with flake support
{
  pkgs,
  inputs',
  ...
}: let
  # Store the source path at nix evaluation time
  valenceSourcePath = toString ../.;
in {
  # Create a new Valence project with flake
  valence-new-flake = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-new-flake" ''
      set -e
      
      # Export project root for SDK path resolution
      export PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
      
      # Colors for output
      GREEN='\033[0;32m'
      YELLOW='\033[1;33m'
      RED='\033[0;31m'
      BLUE='\033[0;34m'
      NC='\033[0m' # No Color
      
      # Check if project name is provided
      if [ $# -eq 0 ]; then
        echo -e "''${RED}Error: Project name required''${NC}"
        echo "Usage: nix run .#valence-new-flake <project-name>"
        exit 1
      fi
      
      PROJECT_NAME="$1"
      PROJECT_DIR="$PROJECT_NAME"
      # Convert hyphens to underscores for Rust library name
      LIB_NAME=$(echo "$PROJECT_NAME" | tr '-' '_')
      
      # Check if directory already exists
      if [ -d "$PROJECT_DIR" ]; then
        echo -e "''${RED}Error: Directory $PROJECT_DIR already exists''${NC}"
        exit 1
      fi
      
      echo -e "''${BLUE}Creating new Valence project with flake: $PROJECT_NAME''${NC}"
      
      # Create project structure (inline from template.nix)
      mkdir -p "$PROJECT_DIR/functions"
      mkdir -p "$PROJECT_DIR/src"
      
      # Create echo module
      cat > "$PROJECT_DIR/src/echo.rs" << 'EOF'
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct EchoInput {
    pub message: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct EchoOutput {
    pub response: String,
}

/// Echo function that returns the input message with "echo: " prefix
pub fn process_echo(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    input_data: &[u8],
) -> ProgramResult {
    msg!("Echo function called");
    
    // Deserialize input
    let input = EchoInput::try_from_slice(input_data)?;
    msg!("Received message: {}", input.message);
    
    // Create output
    let output = EchoOutput {
        response: format!("echo: {}", input.message),
    };
    
    // For now, just log the output
    msg!("Returning: {}", output.response);
    
    Ok(())
}
EOF

      # Create a minimal lib.rs using solana-program
      cat > "$PROJECT_DIR/src/lib.rs" << 'EOF'
use solana_program::{
    account_info::AccountInfo,
    declare_id,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

declare_id!("11111111111111111111111111111111");

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Echo program entrypoint");
    
    if instruction_data.is_empty() {
        msg!("No instruction data provided");
        return Ok(());
    }
    
    match instruction_data[0] {
        0 => {
            msg!("Initialize instruction");
            Ok(())
        }
        1 => {
            msg!("Echo instruction");
            if instruction_data.len() > 1 {
                // Just log the number of bytes received
                msg!("Received {} bytes", instruction_data.len() - 1);
            }
            Ok(())
        }
        _ => {
            msg!("Unknown instruction");
            Ok(())
        }
    }
}
EOF

      # Create client program
      cat > "$PROJECT_DIR/src/client.rs" << 'EOF'
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Echo Program Client");
    println!("==================");
    
    let rpc_url = "http://localhost:8899";
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    
    // Load keypair
    let keypair_path = std::env::var("HOME")? + "/.config/solana/id.json";
    let keypair_bytes: Vec<u8> = serde_json::from_str(&std::fs::read_to_string(&keypair_path)?)?;
    let payer = Keypair::from_bytes(&keypair_bytes)?;
    
    println!("Using RPC: {}", rpc_url);
    println!("Payer: {}", payer.pubkey());
    
    // Get program ID from environment
    let program_id = std::env::var("SHARD_PROGRAM_ID")
        .or_else(|_| {
            std::fs::read_to_string(".valence-env")
                .ok()
                .and_then(|content| {
                    content.lines()
                        .find(|line| line.starts_with("export SHARD_PROGRAM_ID="))
                        .and_then(|line| line.split('=').nth(1))
                        .map(|s| s.to_string())
                })
                .ok_or_else(|| std::env::VarError::NotPresent)
        })
        .map(|s| Pubkey::from_str(&s).expect("Invalid program ID"))
        .expect("SHARD_PROGRAM_ID not found");
    
    println!("Echo Program: {}", program_id);
    
    // First, initialize the program
    println!("\nInitializing program...");
    let init_data = vec![0u8]; // Instruction 0 = initialize
    
    let init_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
        ],
        data: init_data,
    };
    
    let recent_blockhash = client.get_latest_blockhash()?;
    let init_tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    
    match client.send_and_confirm_transaction(&init_tx) {
        Ok(signature) => {
            println!("Initialized! Signature: {}", signature);
        }
        Err(e) => {
            println!("Init error (may already be initialized): {}", e);
        }
    }
    
    // Now send echo message
    let message = "Hello, Solana!";
    println!("\nSending echo message: {}", message);
    
    let mut echo_data = vec![1u8]; // Instruction 1 = echo
    echo_data.extend_from_slice(message.as_bytes());
    
    let echo_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(payer.pubkey(), true),
        ],
        data: echo_data,
    };
    
    let recent_blockhash = client.get_latest_blockhash()?;
    let echo_tx = Transaction::new_signed_with_payer(
        &[echo_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    
    match client.send_and_confirm_transaction(&echo_tx) {
        Ok(signature) => {
            println!("\nSuccess! Transaction signature: {}", signature);
        }
        Err(e) => {
            println!("\nError: {}", e);
            return Err(e.into());
        }
    }
    
    println!("\nClient execution complete!");
    Ok(())
}
EOF

      # Create Cargo.toml for the shard program
      cat > "$PROJECT_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "$PROJECT_NAME"
version = "0.1.0"
edition = "2021"

# Standalone workspace (not part of parent workspace)
[workspace]

[lib]
crate-type = ["cdylib", "lib"]
name = "$LIB_NAME"

[[bin]]
name = "''${PROJECT_NAME}_client"
path = "src/client.rs"

[dependencies]
solana-program = "1.18"
borsh = "0.10.3"

# Client dependencies
[dependencies.solana-client]
version = "1.18"
optional = true

[dependencies.solana-sdk]
version = "1.18"

[dependencies.anyhow]
version = "1.0"
optional = true

[dependencies.tokio]
version = "1.0"
features = ["full"]
optional = true

[dependencies.serde]
version = "1.0"
optional = true

[dependencies.serde_json]
version = "1.0"
optional = true

[features]
default = []
client = ["solana-client", "anyhow", "tokio", "serde", "serde_json"]

[profile.release]
overflow-checks = true
lto = "fat"
codegen-units = 1

CARGO_EOF

      # Create Anchor.toml
      cat > "$PROJECT_DIR/Anchor.toml" << ANCHOR_EOF
[features]
resolution = true
skip-lint = false

[programs.localnet]
$PROJECT_NAME = "11111111111111111111111111111111"

[registry]
url = "https://api.apr.dev"

[provider]
cluster = "localnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
ANCHOR_EOF

      # Create run script
      cat > "$PROJECT_DIR/run.sh" << 'EOF'
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
echo -e "''${YELLOW}Step 1: Building shard and client...''${NC}"
nix run ..#valence-template-build

# Step 2: Deploy shard
echo -e "''${YELLOW}Step 2: Deploying shard program...''${NC}"
nix run ..#valence-template-deploy

# Load the deployed program ID
source .valence-env

# Step 3: Initialize shard
echo -e "''${YELLOW}Step 3: Initializing shard...''${NC}"
nix run ..#valence-template-init

# Step 4: Register functions
echo -e "''${YELLOW}Step 4: Registering functions...''${NC}"
nix run ..#valence-template-register

# Step 5: Create session
echo -e "''${YELLOW}Step 5: Creating session...''${NC}"
nix run ..#valence-template-session

# Step 6: Run client
echo -e "''${YELLOW}Step 6: Running client...''${NC}"
./target/release/PROJECT_NAME_PLACEHOLDER_client

echo ""
echo -e "''${GREEN}=== Complete! ===''${NC}"
EOF
      chmod +x "$PROJECT_DIR/run.sh"

      # Replace PROJECT_NAME in run.sh
      sed -i.bak "s/PROJECT_NAME_PLACEHOLDER/$PROJECT_NAME/g" "$PROJECT_DIR/run.sh"
      rm "$PROJECT_DIR/run.sh.bak"

      # Create README
      cat > "$PROJECT_DIR/README.md" << README_EOF
# $PROJECT_NAME

A Valence shard project with an echo function.

## Structure

- \`functions/echo.rs\` - Echo function implementation
- \`src/lib.rs\` - Shard program with session management
- \`src/client.rs\` - Client to interact with the shard
- \`run.sh\` - Complete deployment and execution script

## Quick Start

1. Make sure you have a local Solana node running:
   \`\`\`bash
   nix run ..#valence-local
   \`\`\`

2. Run the complete flow:
   \`\`\`bash
   ./run.sh
   \`\`\`

Note: The project uses \`.valence-env\` to store deployment information. This file is created automatically during deployment.

This will:
- Build the shard program and client
- Deploy the shard program
- Initialize the shard
- Register the echo function
- Create a session with echo capability
- Execute the echo function via the client

## Manual Steps

### Build
\`\`\`bash
nix run ..#valence-template-build
\`\`\`

### Deploy
\`\`\`bash
nix run ..#valence-template-deploy
\`\`\`

### Initialize Shard
\`\`\`bash
nix run ..#valence-template-init
\`\`\`

### Register Functions
\`\`\`bash
nix run ..#valence-template-register
\`\`\`

### Create Session
\`\`\`bash
nix run ..#valence-template-session
\`\`\`

### Run Client
\`\`\`bash
./target/release/''${PROJECT_NAME}_client
\`\`\`
README_EOF

      # Create .gitignore
      cat > "$PROJECT_DIR/.gitignore" << 'EOF'
/target
/node_modules
.anchor
.DS_Store
*.log
.valence-env
/test-ledger
EOF
      
      # Now add the flake.nix
      cat > "$PROJECT_DIR/flake.nix" << 'FLAKE_EOF'
{
  description = "PROJECT_NAME_PLACEHOLDER - A Valence shard project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    valence.url = "path:VALENCE_SOURCE_PATH";
  };

  outputs = { self, nixpkgs, flake-utils, valence }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.''${system};
        valencePackages = valence.inputs.zero-nix.packages.''${system};
        
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            valencePackages.nightly-rust
            valencePackages.solana-tools
            pkg-config
            openssl
            jq
          ];
          
          shellHook = '''
            echo "PROJECT_NAME_PLACEHOLDER Development Environment"
            echo "========================================="
            echo ""
            echo "Available commands:"
            echo "  cargo build-sbf              - Build Solana program"
            echo "  cargo build --features client - Build client"
            echo "  nix build .#program          - Build program with nix"
            echo "  nix run .#deploy             - Deploy program"
            echo ""
            export RUST_LOG=info
            export RUST_BACKTRACE=1
          ''';
        };
        
        packages = {
          # Build the Solana program
          program = pkgs.stdenv.mkDerivation {
            pname = "PROJECT_NAME_PLACEHOLDER";
            version = "0.1.0";
            
            src = pkgs.lib.cleanSource ./.;
            
            nativeBuildInputs = [ 
              valencePackages.nightly-rust
              valencePackages.solana-tools
              pkgs.pkg-config 
            ];
            
            buildInputs = [ pkgs.openssl ];
            
            preBuild = '''
              export CARGO_HOME=$TMPDIR/.cargo
              export RUST_BACKTRACE=1
              
              # Create a wrapper script that provides getrandom stub
              mkdir -p .cargo
              cat > .cargo/config.toml << 'CONFIG_EOF'
              [target.sbf-solana-solana]
              rustflags = ["-C", "link-arg=--undefined=getrandom"]
CONFIG_EOF
            ''';
            
            buildPhase = '''
              # Build with cargo-build-sbf
              cargo build-sbf
            ''';
            
            installPhase = '''
              mkdir -p $out/lib
              if [ -d "target/deploy" ]; then
                cp target/deploy/*.so $out/lib/ || true
              fi
              if [ -d "target/sbf-solana-solana/release" ]; then
                find target/sbf-solana-solana/release -name "*.so" -type f | grep -v deps | head -1 | xargs -I {} cp {} $out/lib/
              fi
            ''';
          };
          
          # Build the client
          client = pkgs.stdenv.mkDerivation {
            pname = "PROJECT_NAME_PLACEHOLDER-client";
            version = "0.1.0";
            
            src = pkgs.lib.cleanSource ./.;
            
            nativeBuildInputs = [ 
              valencePackages.nightly-rust
              pkgs.pkg-config 
            ];
            
            buildInputs = [ pkgs.openssl ];
            
            buildPhase = '''
              export CARGO_HOME=$TMPDIR/.cargo
              cargo build --release --features client --bin PROJECT_NAME_PLACEHOLDER_client
            ''';
            
            installPhase = '''
              mkdir -p $out/bin
              cp target/release/PROJECT_NAME_PLACEHOLDER_client $out/bin/
            ''';
          };
          
          default = self.packages.''${system}.program;
        };
        
        apps = {
          # Deploy the program
          deploy = {
            type = "app";
            program = pkgs.writeShellScript "deploy" '''
              set -e
              echo "Deploying PROJECT_NAME_PLACEHOLDER..."
              SO_FILE="''${self.packages.''${system}.program}/lib/*.so"
              if [ -f $SO_FILE ]; then
                ''${valencePackages.solana-tools}/bin/solana program deploy $SO_FILE
              else
                echo "Error: No .so file found in nix build"
                echo "Try: nix build .#program"
                exit 1
              fi
            ''';
          };
          
          # Run the client
          client = {
            type = "app";
            program = "''${self.packages.''${system}.client}/bin/PROJECT_NAME_PLACEHOLDER_client";
          };
        };
      });
}
FLAKE_EOF

      # Replace placeholders in flake.nix
      sed -i.bak "s/PROJECT_NAME_PLACEHOLDER/$PROJECT_NAME/g" "$PROJECT_DIR/flake.nix"
      # Use the actual project root, not the nix store path
      REAL_PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
      sed -i.bak "s|VALENCE_SOURCE_PATH|$REAL_PROJECT_ROOT|g" "$PROJECT_DIR/flake.nix"
      rm "$PROJECT_DIR/flake.nix.bak"

      echo -e "''${GREEN}Project created successfully with flake support!''${NC}"
      echo ""
      echo "Next steps:"
      echo "  1. cd $PROJECT_DIR"
      echo "  2. nix develop  # Enter development shell"
      echo "  3. nix build .#program  # Build program with nix"
      echo ""
      echo "Or use the traditional approach:"
      echo "  1. cd $PROJECT_DIR"
      echo "  2. ./run.sh"
    ''}/bin/valence-new-flake";
  };
}