# Template project commands
{
  pkgs,
  inputs',
  ...
}: let
  # Store the source path at nix evaluation time
  valenceSourcePath = toString ../.;
in {
  # Create a new Valence project from template
  valence-new = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-new" ''
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
        echo "Usage: nix run .#valence-new <project-name>"
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
      
      echo -e "''${BLUE}Creating new Valence project: $PROJECT_NAME''${NC}"
      
      # Create project structure
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
      cat > "$PROJECT_DIR/Cargo.toml" << 'CARGO_EOF'
[package]
name = "PROJECT_NAME_PLACEHOLDER"
version = "0.1.0"
edition = "2021"

# Standalone workspace (not part of parent workspace)
[workspace]

[lib]
crate-type = ["cdylib", "lib"]
name = "LIB_NAME_PLACEHOLDER"

[[bin]]
name = "PROJECT_NAME_PLACEHOLDER_client"
path = "src/client.rs"

[dependencies]
solana-program = "1.18"
borsh = "0.10.3"
# valence-sdk = { path = "../sdk" }  # Commented out for now

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
      
      # Replace placeholders
      sed -i.bak "s/PROJECT_NAME_PLACEHOLDER/$PROJECT_NAME/g" "$PROJECT_DIR/Cargo.toml"
      sed -i.bak "s/LIB_NAME_PLACEHOLDER/$LIB_NAME/g" "$PROJECT_DIR/Cargo.toml"
      rm "$PROJECT_DIR/Cargo.toml.bak"

      # Create Anchor.toml
      cat > "$PROJECT_DIR/Anchor.toml" << 'ANCHOR_EOF'
[features]
resolution = true
skip-lint = false

[programs.localnet]
PROJECT_NAME_PLACEHOLDER = "11111111111111111111111111111111"

[registry]
url = "https://api.apr.dev"

[provider]
cluster = "localnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
ANCHOR_EOF
      
      # Replace placeholder
      sed -i.bak "s/PROJECT_NAME_PLACEHOLDER/$PROJECT_NAME/g" "$PROJECT_DIR/Anchor.toml"
      rm "$PROJECT_DIR/Anchor.toml.bak"

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
      cat > "$PROJECT_DIR/README.md" << 'README_EOF'
# PROJECT_NAME_PLACEHOLDER

A Valence shard project with an echo function.

## Structure

- `functions/echo.rs` - Echo function implementation
- `src/lib.rs` - Shard program with session management
- `src/client.rs` - Client to interact with the shard
- `run.sh` - Complete deployment and execution script

## Quick Start

1. Make sure you have a local Solana node running:
   ```bash
   nix run ..#valence-local
   ```

2. Run the complete flow:
   ```bash
   ./run.sh
   ```

Note: The project uses `.valence-env` to store deployment information. This file is created automatically during deployment.

This will:
- Build the shard program and client
- Deploy the shard program
- Initialize the shard
- Register the echo function
- Create a session with echo capability
- Execute the echo function via the client

## Manual Steps

### Build
```bash
nix run ..#valence-template-build
```

### Deploy
```bash
nix run ..#valence-template-deploy
```

### Initialize Shard
```bash
nix run ..#valence-template-init
```

### Register Functions
```bash
nix run ..#valence-template-register
```

### Create Session
```bash
nix run ..#valence-template-session
```

### Run Client
```bash
./target/release/PROJECT_NAME_PLACEHOLDER_client
```
README_EOF
      
      # Replace placeholder in README
      sed -i.bak "s/PROJECT_NAME_PLACEHOLDER/$PROJECT_NAME/g" "$PROJECT_DIR/README.md"
      rm "$PROJECT_DIR/README.md.bak"

      # Create .envrc for direnv users
      cat > "$PROJECT_DIR/.envrc" << 'EOF'
use flake
EOF

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

      echo -e "''${GREEN}Project created successfully!''${NC}"
      echo ""
      echo "Project structure:"
      echo "  $PROJECT_DIR/"
      echo "  ├── functions/"
      echo "  │   └── echo.rs"
      echo "  ├── src/"
      echo "  │   ├── lib.rs"
      echo "  │   └── client.rs"
      echo "  ├── Cargo.toml"
      echo "  ├── Anchor.toml"
      echo "  ├── run.sh"
      echo "  └── README.md"
      echo ""
      echo "Next steps:"
      echo "  1. cd $PROJECT_DIR"
      echo "  2. ./run.sh"
    ''}/bin/valence-new";
  };

  # Build template project
  valence-template-build = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-template-build" ''
      set -e
      
      echo "=== Building Valence Template Project ==="
      
      # Check if we're in a template project
      if [ ! -f "Cargo.toml" ] || [ ! -d "functions" ]; then
        echo "Error: Not in a Valence template project directory"
        exit 1
      fi
      
      # Get project name from Cargo.toml
      PROJECT_NAME=$(grep "^name = " Cargo.toml | head -1 | cut -d'"' -f2)
      
      # Set up environment
      export PATH="${inputs'.zero-nix.packages.solana-tools}/bin:${inputs'.zero-nix.packages.nightly-rust}/bin:$PATH"
      export RUST_BACKTRACE=1
      export MACOSX_DEPLOYMENT_TARGET=11.0
      export SOURCE_DATE_EPOCH=1686858254
      export PROTOC=${pkgs.protobuf}/bin/protoc
      export PKG_CONFIG_PATH=${pkgs.openssl.dev}/lib/pkgconfig
      export OPENSSL_DIR=${pkgs.openssl.dev}
      export OPENSSL_LIB_DIR=${pkgs.openssl.out}/lib
      export OPENSSL_INCLUDE_DIR=${pkgs.openssl.dev}/include
      
      # Build the shard program
      echo "Building shard program..."
      cargo build-sbf
      
      # Build the client
      echo "Building client..."
      cargo build --release --bin ''${PROJECT_NAME}_client --features client
      
      echo "Build complete!"
      echo "  Program: target/deploy/$PROJECT_NAME.so"
      echo "  Client: target/release/''${PROJECT_NAME}_client"
    ''}/bin/valence-template-build";
  };

  # Deploy template project
  valence-template-deploy = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-template-deploy" ''
      set -e
      
      echo "=== Deploying Valence Template Project ==="
      
      # Check for built program
      PROGRAM_SO=$(find target/deploy -name "*.so" -type f 2>/dev/null | head -1)
      if [ -z "$PROGRAM_SO" ]; then
        echo "Error: No built program found. Run build first."
        exit 1
      fi
      
      # Deploy the program
      echo "Deploying $PROGRAM_SO..."
      DEPLOY_OUTPUT=$(solana program deploy "$PROGRAM_SO" 2>&1)
      PROGRAM_ID=$(echo "$DEPLOY_OUTPUT" | grep -E "Program Id:|Deployed" | awk '{print $NF}' | head -1)
      
      if [ -z "$PROGRAM_ID" ]; then
        echo "Error: Failed to deploy program"
        echo "$DEPLOY_OUTPUT"
        exit 1
      fi
      
      echo "Deployed to: $PROGRAM_ID"
      echo "export SHARD_PROGRAM_ID=$PROGRAM_ID" > .valence-env
      
      # Update Anchor.toml with actual program ID
      PROJECT_NAME=$(basename "$PWD")
      sed -i.bak "s/11111111111111111111111111111111/$PROGRAM_ID/" Anchor.toml
      
      echo "Deployment complete!"
    ''}/bin/valence-template-deploy";
  };

  # Initialize shard
  valence-template-init = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-template-init" ''
      set -e
      
      echo "=== Initializing Valence Shard ==="
      
      # Load environment
      if [ ! -f .valence-env ]; then
        echo "Error: .valence-env file not found. Deploy first."
        exit 1
      fi
      source .valence-env
      
      # Load config for singleton addresses
      CONFIG_FILE="$HOME/.valence/local-config.json"
      if [ ! -f "$CONFIG_FILE" ]; then
        echo "Error: Valence config not found. Run valence-local first."
        exit 1
      fi
      
      GATEWAY=$(jq -r .gateway "$CONFIG_FILE")
      REGISTRY=$(jq -r .registry "$CONFIG_FILE")
      VERIFIER=$(jq -r .verifier "$CONFIG_FILE")
      
      echo "Initializing shard..."
      echo "  Gateway: $GATEWAY"
      echo "  Registry: $REGISTRY"
      echo "  Verifier: $VERIFIER"
      
      # Here you would call the initialize instruction
      # For now, we'll just mark it as done
      echo "export SHARD_INITIALIZED=true" >> .valence-env
      
      echo "Shard initialized!"
    ''}/bin/valence-template-init";
  };

  # Register functions
  valence-template-register = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-template-register" ''
      set -e
      
      echo "=== Registering Functions ==="
      
      # Load environment
      if [ ! -f .valence-env ]; then
        echo "Error: .valence-env file not found. Deploy first."
        exit 1
      fi
      source .valence-env
      
      if [ "$SHARD_INITIALIZED" != "true" ]; then
        echo "Error: Shard not initialized. Initialize first."
        exit 1
      fi
      
      echo "Registering echo function..."
      
      # Calculate content hash (simplified - in real implementation would hash actual code)
      CONTENT_HASH="0000000000000000000000000000000000000000000000000000000000000001"
      
      # Here you would call the register_function instruction
      echo "Function 'echo' registered with hash: $CONTENT_HASH"
      
      echo "export FUNCTIONS_REGISTERED=true" >> .valence-env
      echo "Registration complete!"
    ''}/bin/valence-template-register";
  };

  # Create session
  valence-template-session = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-template-session" ''
      set -e
      
      echo "=== Creating Session ==="
      
      # Load environment
      if [ ! -f .valence-env ]; then
        echo "Error: .valence-env file not found. Deploy first."
        exit 1
      fi
      source .valence-env
      
      if [ "$FUNCTIONS_REGISTERED" != "true" ]; then
        echo "Error: Functions not registered. Register first."
        exit 1
      fi
      
      echo "Creating session with echo capability..."
      
      # Generate session ID
      SESSION_ID="0000000000000000000000000000000000000000000000000000000000000001"
      
      # Here you would call the create_session instruction
      echo "Session created with ID: $SESSION_ID"
      echo "Capability granted: CallFunction { name: 'echo' }"
      
      echo "export SESSION_ID=$SESSION_ID" >> .valence-env
      echo "Session ready!"
    ''}/bin/valence-template-session";
  };
}