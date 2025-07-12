# Simple template project commands for testing
{
  pkgs,
  inputs',
  ...
}: {
  # Create a simple Valence project without complex dependencies
  valence-new-simple = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-new-simple" ''
      set -e
      
      # Colors for output
      GREEN='\033[0;32m'
      YELLOW='\033[1;33m'
      RED='\033[0;31m'
      BLUE='\033[0;34m'
      NC='\033[0m' # No Color
      
      # Check if project name is provided
      if [ $# -eq 0 ]; then
        echo -e "''${RED}Error: Project name required''${NC}"
        echo "Usage: nix run .#valence-new-simple <project-name>"
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
      
      echo -e "''${BLUE}Creating simple Valence project: $PROJECT_NAME''${NC}"
      
      # Create project structure
      mkdir -p "$PROJECT_DIR/src"
      
      # Create a minimal lib.rs that uses only Solana 1.18
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
    msg!("Simple echo program");
    
    if !instruction_data.is_empty() {
        msg!("Received {} bytes of data", instruction_data.len());
    }
    
    Ok(())
}
EOF

      # Create simple client
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
    println!("Simple Echo Client");
    println!("=================");
    
    let rpc_url = "http://localhost:8899";
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    
    println!("RPC: {}", rpc_url);
    
    // Load keypair
    let keypair_path = std::env::var("HOME")? + "/.config/solana/id.json";
    let payer = Keypair::read_from_file(&keypair_path)?;
    
    println!("Payer: {}", payer.pubkey());
    
    // Get program ID
    let program_id = std::env::var("PROGRAM_ID")
        .unwrap_or_else(|_| "11111111111111111111111111111111".to_string());
    let program_id = Pubkey::from_str(&program_id)?;
    
    println!("Program: {}", program_id);
    
    // Create simple instruction
    let instruction = Instruction {
        program_id,
        accounts: vec![AccountMeta::new_readonly(payer.pubkey(), true)],
        data: vec![1, 2, 3, 4], // Simple test data
    };
    
    // Send transaction
    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    
    match client.send_and_confirm_transaction(&transaction) {
        Ok(signature) => {
            println!("\nSuccess! Signature: {}", signature);
        }
        Err(e) => {
            println!("\nError: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}
EOF

      # Create Cargo.toml with Solana 1.18
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
solana-program = "~1.18"

# Client dependencies
[dependencies.solana-client]
version = "~1.18"
optional = true

[dependencies.solana-sdk]
version = "~1.18"
optional = true

[dependencies.anyhow]
version = "1.0"
optional = true

[dependencies.tokio]
version = "1.0"
features = ["full"]
optional = true

[features]
default = []
client = ["solana-client", "solana-sdk", "anyhow", "tokio"]

[profile.release]
overflow-checks = true
lto = "fat"
codegen-units = 1
CARGO_EOF
      
      # Replace placeholders
      sed -i.bak "s/PROJECT_NAME_PLACEHOLDER/$PROJECT_NAME/g" "$PROJECT_DIR/Cargo.toml"
      sed -i.bak "s/LIB_NAME_PLACEHOLDER/$LIB_NAME/g" "$PROJECT_DIR/Cargo.toml"
      rm "$PROJECT_DIR/Cargo.toml.bak"

      echo -e "''${GREEN}Simple project created successfully!''${NC}"
      echo ""
      echo "Project structure:"
      echo "  $PROJECT_DIR/"
      echo "  ├── src/"
      echo "  │   ├── lib.rs"
      echo "  │   └── client.rs"
      echo "  └── Cargo.toml"
      echo ""
      echo "To build:"
      echo "  cd $PROJECT_DIR"
      echo "  cargo build-sbf              # Build program"
      echo "  cargo build --features client # Build client"
    ''}/bin/valence-new-simple";
  };
}