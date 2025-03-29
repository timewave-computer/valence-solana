#!/usr/bin/env bash

set -e

echo "=== Solana + Anchor Hello World Demo ==="

# Export PATH to include cargo binaries
export PATH=$PATH:$HOME/.cargo/bin

# Cleanup function
cleanup() {
  echo "Cleaning up..."
  if [ ! -z "$VALIDATOR_PID" ]; then
    kill $VALIDATOR_PID 2>/dev/null || true
  fi
  rm -f validator.log
}
trap cleanup EXIT

# Step 1: Start Solana validator
echo "=== Starting Solana validator ==="
if pgrep solana-test-validator >/dev/null; then
  echo "Validator is already running"
else
  echo "Starting new validator..."
  solana-test-validator --reset > validator.log 2>&1 &
  VALIDATOR_PID=$!
  
  echo "Waiting for validator to start..."
  for i in {1..30}; do
    if solana cluster-version &>/dev/null; then
      echo "Validator is running!"
      break
    fi
    
    if [ $i -eq 30 ]; then
      echo "Error: Validator failed to start in time."
      exit 1
    fi
    
    echo -n "."
    sleep 1
  done
fi

# Step 2: Configure wallet
echo "=== Setting up wallet ==="
KEYPAIR_FILE="$HOME/.config/solana/id.json"
mkdir -p "$(dirname "$KEYPAIR_FILE")"

if [ ! -f "$KEYPAIR_FILE" ]; then
  echo "Creating new Solana keypair..."
  solana-keygen new --no-bip39-passphrase -o "$KEYPAIR_FILE" --force --silent
fi

solana config set --url http://127.0.0.1:8899 --keypair "$KEYPAIR_FILE"
WALLET_ADDRESS=$(solana address)
echo "Using wallet: $WALLET_ADDRESS"

# Step 3: Fund wallet
BALANCE=$(solana balance | awk '{print $1}')
if (( $(echo "$BALANCE < 5" | bc -l) )); then
  echo "Adding SOL to wallet..."
  solana airdrop 100
  solana balance
fi

# Step 4: Check for Anchor
echo "=== Checking for Anchor ==="
if ! command -v anchor &> /dev/null; then
  echo "Anchor CLI not found. Installing..."
  cargo install --git https://github.com/coral-xyz/anchor --tag v0.29.0 anchor-cli
else
  echo "Anchor $(anchor --version) is installed"
fi

# Step 5: Hello World program setup
echo "=== Setting up Hello World program ==="

if [ ! -d "hello_world" ]; then
  echo "Creating a new hello_world project..."
  anchor init hello_world
fi

cd hello_world

# Step 6: Generate program keypair if it doesn't exist
if [ ! -f target/deploy/hello_world-keypair.json ]; then
  echo "Creating program keypair..."
  mkdir -p target/deploy
  solana-keygen new -o target/deploy/hello_world-keypair.json --no-bip39-passphrase --force --silent
fi

# Step 7: Update program ID in configs
PROGRAM_ID=$(solana address -k target/deploy/hello_world-keypair.json)
echo "Program ID: $PROGRAM_ID"
sed -i.bak "s/hello_world = \"[^\"]*\"/hello_world = \"$PROGRAM_ID\"/" Anchor.toml
sed -i.bak "s/declare_id!(\"[^\"]*\")/declare_id!(\"$PROGRAM_ID\")/" programs/hello_world/src/lib.rs
rm -f Anchor.toml.bak programs/hello_world/src/lib.rs.bak

# Step 8: Build and deploy
echo "=== Building and deploying program ==="
echo "Note: This step requires the Solana BPF SDK to be installed."
echo "If you get build-bpf errors, please install the Solana SDK from https://docs.solana.com/cli/install-solana-cli-tools"

echo "Building program..."
if anchor build; then
  echo "Build successful"
  
  echo "Deploying program..."
  if anchor deploy; then
    echo "Deployment successful"
    
    echo "Running tests..."
    anchor test --skip-deploy
  else
    echo "Deployment failed. Check your Solana installation and wallet balance."
  fi
else
  echo "Build failed. Please ensure Solana SDK with BPF support is installed."
fi

echo "=== Demo completed ==="
echo "Validator is running at: http://127.0.0.1:8899"
echo "Wallet address: $WALLET_ADDRESS"
echo "Program ID: $PROGRAM_ID" 